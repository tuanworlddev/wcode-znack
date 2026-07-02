import { useEffect, useState } from "react";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import type { Update } from "@tauri-apps/plugin-updater";
import { BadgeCheck, Download, ExternalLink, Info, RefreshCw } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import { checkForUpdate, installUpdate } from "../lib/updater";
import type { Notify } from "../App";

const REPO_URL = "https://github.com/tuanworlddev/wcode-znack";

export default function AboutCard({ notify }: { notify: Notify }) {
  const [version, setVersion] = useState("");
  const [tauriVersion, setTauriVersion] = useState("");
  const [checking, setChecking] = useState(false);
  const [checked, setChecked] = useState(false);
  const [update, setUpdate] = useState<Update | null>(null);
  const [progress, setProgress] = useState<number | null>(null);

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
    getTauriVersion().then(setTauriVersion).catch(() => {});
  }, []);

  async function checkUpdates() {
    setChecking(true);
    setChecked(false);
    try {
      const u = await checkForUpdate();
      setUpdate(u);
      setChecked(true);
      if (!u) notify("Bạn đang dùng phiên bản mới nhất.", "ok");
    } catch (e) {
      notify(`Kiểm tra cập nhật thất bại: ${errMsg(e)}`, "err");
    } finally {
      setChecking(false);
    }
  }

  async function install() {
    if (!update) return;
    setProgress(0);
    try {
      await installUpdate(update, setProgress);
      // relaunch() replaces the process — code below only runs on failure.
    } catch (e) {
      setProgress(null);
      notify(`Cập nhật thất bại: ${errMsg(e)}`, "err");
    }
  }

  return (
    <div className="card">
      <h2>
        <Info size={16} /> Giới thiệu ứng dụng
      </h2>
      <div className="about-grid">
        <span className="muted">Ứng dụng</span>
        <span>
          <b>WCode</b> — in tem FBS/FBO Wildberries + Честный ЗНАК
        </span>
        <span className="muted">Phiên bản</span>
        <span className="mono">{version || "…"}</span>
        <span className="muted">Nền tảng</span>
        <span>Tauri {tauriVersion || "…"}</span>
        <span className="muted">Mã nguồn</span>
        <button className="link-btn" onClick={() => api.openUrl(REPO_URL).catch(() => {})}>
          {REPO_URL.replace("https://", "")} <ExternalLink size={12} />
        </button>
      </div>

      <div className="setting-actions about-update">
        {update && progress === null && (
          <span className="chip about-new">
            Có bản mới: <b className="mono">{update.version}</b>
          </span>
        )}
        {checked && !update && (
          <span className="muted about-uptodate">
            <BadgeCheck size={14} /> Mới nhất
          </span>
        )}
        {progress !== null ? (
          <span className="muted">
            <RefreshCw size={14} className="spin" /> Đang tải bản cập nhật…{" "}
            {Math.round(progress * 100)}%
          </span>
        ) : update ? (
          <button className="primary" onClick={install}>
            <Download size={16} /> Cập nhật lên {update.version} & khởi động lại
          </button>
        ) : (
          <button className="ghost" onClick={checkUpdates} disabled={checking}>
            <RefreshCw size={15} className={checking ? "spin" : ""} />
            {checking ? "Đang kiểm tra..." : "Kiểm tra cập nhật"}
          </button>
        )}
      </div>
    </div>
  );
}
