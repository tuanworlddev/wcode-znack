import { useEffect, useState } from "react";
import { RefreshCw, Save, ShieldCheck } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type { ZnackCertificate, ZnackSettings } from "../types/znack";
import type { Notify } from "../App";

const EMPTY: ZnackSettings = {
  trueApiBaseUrl: "",
  suzBaseUrl: "",
  omsId: "",
  omsConnection: "",
  participantInn: "",
  producerInn: "",
  ownerInn: "",
  cryptcpPath: "",
  certThumbprint: "",
  certLabel: "",
  cryptoproTimeoutSeconds: 60,
  documentType: "",
  documentNumber: "",
  documentDate: "",
  autoIntroduction: false,
};

// Thumbprints already test-signed successfully on this machine — the
// signature check runs automatically only on the first connect per cert.
const TESTED_KEY = "wcode.znack.testedCerts.v1";
function certTested(thumbprint: string): boolean {
  try {
    return (JSON.parse(localStorage.getItem(TESTED_KEY) ?? "[]") as string[]).includes(thumbprint);
  } catch {
    return false;
  }
}
function markCertTested(thumbprint: string) {
  try {
    const list = JSON.parse(localStorage.getItem(TESTED_KEY) ?? "[]") as string[];
    if (!list.includes(thumbprint)) list.push(thumbprint);
    localStorage.setItem(TESTED_KEY, JSON.stringify(list));
  } catch {
    localStorage.setItem(TESTED_KEY, JSON.stringify([thumbprint]));
  }
}

function certLabel(c: ZnackCertificate): string {
  const cn = /CN=([^,]+)/.exec(c.subject)?.[1]?.trim() ?? c.subject;
  const until = c.notAfter ? ` · hết hạn ${c.notAfter}` : "";
  return `${cn}${c.inn ? ` · ИНН ${c.inn}` : ""}${until}`;
}

export default function ZnackSettingsCard({ notify }: { notify: Notify }) {
  const [s, setS] = useState<ZnackSettings>(EMPTY);
  const [certs, setCerts] = useState<ZnackCertificate[]>([]);
  const [loadingCerts, setLoadingCerts] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [busy, setBusy] = useState(false);
  const [testing, setTesting] = useState(false);

  useEffect(() => {
    api
      .znackGetSettings()
      .then((loaded) => setS({ ...EMPTY, ...loaded }))
      .catch((e) => notify(errMsg(e), "err"));
    // Pre-fill the signature dropdown; stay silent if CryptoPro is absent —
    // the refresh button reports errors when asked explicitly.
    api.znackListCertificates().then(setCerts).catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function set<K extends keyof ZnackSettings>(key: K, value: ZnackSettings[K]) {
    setS((prev) => ({ ...prev, [key]: value }));
    setDirty(true);
  }

  async function loadCerts() {
    setLoadingCerts(true);
    try {
      const list = await api.znackListCertificates();
      setCerts(list);
      if (list.length === 0) {
        notify("Không tìm thấy chữ ký còn hạn nào trong CryptoPro.", "info");
      }
    } catch (e) {
      notify(`Đọc chữ ký thất bại: ${errMsg(e)}`, "err");
    } finally {
      setLoadingCerts(false);
    }
  }

  function pickCert(thumbprint: string) {
    const cert = certs.find((c) => c.thumbprint === thumbprint);
    setS((prev) => ({
      ...prev,
      certThumbprint: thumbprint,
      certLabel: cert ? certLabel(cert) : prev.certLabel,
    }));
    setDirty(true);
  }

  async function save() {
    setBusy(true);
    try {
      await api.znackSaveSettings(s);
      notify("Đã lưu cài đặt Честный ЗНАК.", "ok");
      setDirty(false);
      const thumbprint = s.certThumbprint.trim();
      if (thumbprint) {
        // First connect with this cert: verify signing once before touching
        // the Честный ЗНАК APIs; skip the sync if the signature is broken.
        if (!certTested(thumbprint)) {
          try {
            const result = await api.znackTestSign();
            markCertTested(thumbprint);
            notify(`Kiểm tra chữ ký lần đầu: ${result}`, "ok");
          } catch (e) {
            notify(`Chữ ký chưa dùng được — chưa đồng bộ GTIN: ${errMsg(e)}`, "err");
            return;
          }
        }
        // Connection details changed — refresh the GTIN list right away so
        // the mapping/buy screens work without a manual sync.
        try {
          notify(await api.znackSyncProducts(), "ok");
        } catch (e) {
          notify(`Tự đồng bộ GTIN thất bại: ${errMsg(e)}`, "err");
        }
      }
    } catch (e) {
      notify(`Lưu thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  async function testSign() {
    setTesting(true);
    try {
      if (dirty) {
        await api.znackSaveSettings(s);
        setDirty(false);
      }
      const result = await api.znackTestSign();
      if (s.certThumbprint.trim()) markCertTested(s.certThumbprint.trim());
      notify(`Ký thử thành công: ${result}`, "ok");
    } catch (e) {
      notify(`Ký thử thất bại: ${errMsg(e)}`, "err");
    } finally {
      setTesting(false);
    }
  }

  return (
    <div className="card">
      <h2>Честный ЗНАК (маркировка)</h2>

      <div className="znack-grid">
        <div className="setting-field znack-field">
          <label>omsId (СУЗ)</label>
          <input
            value={s.omsId}
            placeholder="UUID trong cài đặt СУЗ"
            onChange={(e) => set("omsId", e.target.value)}
          />
        </div>
        <div className="setting-field znack-field">
          <label>omsConnection</label>
          <input
            value={s.omsConnection}
            placeholder="UUID kết nối tích hợp"
            onChange={(e) => set("omsConnection", e.target.value)}
          />
        </div>

        <div className="setting-field znack-field znack-cert">
          <label>Chữ ký УКЭП (CryptoPro)</label>
          <div className="key-wrap">
            <select value={s.certThumbprint} onChange={(e) => pickCert(e.target.value)}>
              {s.certThumbprint && !certs.some((c) => c.thumbprint === s.certThumbprint) && (
                <option value={s.certThumbprint}>
                  {s.certLabel || s.certThumbprint}
                </option>
              )}
              {!s.certThumbprint && <option value="">— Chọn chữ ký —</option>}
              {certs.map((c) => (
                <option key={c.thumbprint} value={c.thumbprint}>
                  {certLabel(c)}
                </option>
              ))}
            </select>
            <button
              className="icon-btn ghost"
              title="Tải danh sách chữ ký còn hạn từ CryptoPro"
              onClick={loadCerts}
              disabled={loadingCerts}
            >
              <RefreshCw size={15} className={loadingCerts ? "spin" : ""} />
            </button>
          </div>
        </div>

        <div className="setting-field znack-field">
          <label>Số giấy tờ hợp quy</label>
          <input
            value={s.documentNumber}
            placeholder="bỏ trống = không tự lưu thông"
            onChange={(e) => set("documentNumber", e.target.value)}
          />
        </div>
        <div className="setting-field znack-field">
          <label>Ngày làm giấy tờ (dd.MM.yyyy)</label>
          <input
            value={s.documentDate}
            placeholder="vd: 15.03.2026"
            onChange={(e) => set("documentDate", e.target.value)}
          />
        </div>
      </div>

      <p className="muted">
        Loại giấy tờ tự chọn: <b>Tuyên bố hợp quy (декларация о соответствии)</b>. Nếu điền
        số + ngày giấy tờ, hệ thống sẽ tự động đưa mã vào lưu thông sau khi mua; bỏ trống thì
        chỉ mua mã, không lưu thông.
      </p>

      <div className="setting-actions">
        <button className="ghost" onClick={testSign} disabled={testing || busy}>
          <ShieldCheck size={16} /> {testing ? "Đang ký thử..." : "Kiểm tra ký УКЭП"}
        </button>
        <button className="primary" onClick={save} disabled={!dirty || busy}>
          <Save size={16} /> {busy ? "Đang lưu..." : "Lưu cài đặt ЧЗ"}
        </button>
      </div>
    </div>
  );
}
