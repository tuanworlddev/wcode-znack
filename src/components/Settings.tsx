import { useEffect, useState } from "react";
import { Pencil, Eye, EyeOff, Trash2, Save } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type { StoreInfo } from "../types/wb";
import type { Notify } from "../App";
import ConfirmModal from "./ConfirmModal";
import ZnackSettingsCard from "./ZnackSettings";

export default function Settings({
  notify,
  active,
  onStoresChange,
}: {
  notify: Notify;
  active: StoreInfo | null;
  onStoresChange: () => Promise<void> | void;
}) {
  const [nameVal, setNameVal] = useState("");
  const [keyVal, setKeyVal] = useState("");
  const [origName, setOrigName] = useState("");
  const [origKey, setOrigKey] = useState("");
  const [editName, setEditName] = useState(false);
  const [editKey, setEditKey] = useState(false);
  const [showKey, setShowKey] = useState(false);
  const [busy, setBusy] = useState(false);
  const [confirmDel, setConfirmDel] = useState(false);

  useEffect(() => {
    if (!active) return;
    setNameVal(active.name);
    setOrigName(active.name);
    api
      .getActiveToken()
      .then((t) => {
        setKeyVal(t ?? "");
        setOrigKey(t ?? "");
      })
      .catch(() => {});
  }, [active]);

  if (!active) {
    return (
      <div className="page">
        <h1>Cài đặt</h1>
        <div className="card">
          <p className="muted">
            Chưa chọn cửa hàng. Thêm hoặc chọn cửa hàng ở menu góc phải trên.
          </p>
        </div>
      </div>
    );
  }

  const dirty = editName || editKey;

  function cancel() {
    setNameVal(origName);
    setKeyVal(origKey);
    setEditName(false);
    setEditKey(false);
    setShowKey(false);
  }

  async function save() {
    setBusy(true);
    try {
      if (editName && nameVal.trim() && nameVal.trim() !== origName) {
        await api.renameStore(active!.id, nameVal.trim());
      }
      if (editKey && keyVal.trim() && keyVal.trim() !== origKey) {
        await api.setStoreToken(active!.id, keyVal.trim());
      }
      notify("Đã lưu thay đổi.", "ok");
      setEditName(false);
      setEditKey(false);
      setShowKey(false);
      await onStoresChange();
    } catch (e) {
      notify(`Lưu thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  async function doDelete() {
    setBusy(true);
    try {
      await api.removeStore(active!.id);
      notify(`Đã xoá cửa hàng "${active!.name}".`, "info");
      setConfirmDel(false);
      await onStoresChange();
    } catch (e) {
      notify(errMsg(e), "err");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="page">
      <h1>Cài đặt cửa hàng</h1>

      <div className="card">
        <h2>{active.name}</h2>

        <div className="setting-row">
          <div className="setting-field">
            <label>Tên cửa hàng</label>
            <input
              value={nameVal}
              disabled={!editName}
              onChange={(e) => setNameVal(e.target.value)}
            />
          </div>
          <button
            className="icon-btn"
            title="Chỉnh sửa tên"
            onClick={() => setEditName(true)}
            disabled={editName}
          >
            <Pencil size={16} />
          </button>
        </div>

        <div className="setting-row">
          <div className="setting-field">
            <label>Khóa kết nối Wildberries</label>
            <div className="key-wrap">
              <input
                type={showKey ? "text" : "password"}
                value={keyVal}
                disabled={!editKey}
                placeholder={origKey ? "" : "Chưa có khóa kết nối"}
                onChange={(e) => setKeyVal(e.target.value)}
              />
              <button
                className="icon-btn ghost"
                title={showKey ? "Ẩn" : "Hiện"}
                onClick={() => setShowKey((s) => !s)}
              >
                {showKey ? <EyeOff size={16} /> : <Eye size={16} />}
              </button>
            </div>
          </div>
          <button
            className="icon-btn"
            title="Chỉnh sửa khóa kết nối"
            onClick={() => setEditKey(true)}
            disabled={editKey}
          >
            <Pencil size={16} />
          </button>
        </div>

        <div className="setting-actions">
          {dirty && (
            <button className="ghost" onClick={cancel} disabled={busy}>
              Huỷ
            </button>
          )}
          <button className="primary" onClick={save} disabled={!dirty || busy}>
            <Save size={16} /> {busy ? "Đang lưu..." : "Lưu thay đổi"}
          </button>
        </div>
      </div>

      <ZnackSettingsCard notify={notify} />

      <div className="card danger-card">
        <h2>Xoá cửa hàng</h2>
        <p className="muted">
          Xoá toàn bộ dữ liệu và khóa kết nối của cửa hàng này khỏi máy. Không thể
          hoàn tác.
        </p>
        <button
          className="ghost danger"
          onClick={() => setConfirmDel(true)}
          disabled={busy}
        >
          <Trash2 size={16} /> Xoá cửa hàng "{active.name}"
        </button>
      </div>

      {confirmDel && (
        <ConfirmModal
          title="Xoá cửa hàng?"
          message={`Cửa hàng "${active.name}" cùng toàn bộ dữ liệu và khóa kết nối sẽ bị xoá khỏi máy. Hành động này không thể hoàn tác.`}
          confirmLabel="Xoá cửa hàng"
          danger
          busy={busy}
          onConfirm={doDelete}
          onCancel={() => setConfirmDel(false)}
        />
      )}
    </div>
  );
}
