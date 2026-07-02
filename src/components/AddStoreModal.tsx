import { useState } from "react";
import { api, errMsg } from "../api/tauri";
import type { Notify } from "../App";

export default function AddStoreModal({
  notify,
  onClose,
  onDone,
}: {
  notify: Notify;
  onClose: () => void;
  onDone: (connected: boolean) => Promise<void> | void;
}) {
  const [name, setName] = useState("");
  const [key, setKey] = useState("");
  const [busy, setBusy] = useState(false);

  async function save() {
    if (!name.trim()) return notify("Nhập tên cửa hàng.", "err");
    if (!key.trim()) return notify("Nhập Wildberries API key.", "err");
    setBusy(true);
    try {
      const id = await api.addStore(name.trim());
      let connected = false;
      try {
        await api.setStoreToken(id, key.trim());
        connected = true;
        notify(`Đã thêm cửa hàng "${name.trim()}" và kết nối thành công.`, "ok");
      } catch (e) {
        // Store created but key invalid — keep it, warn.
        notify(
          `Đã tạo cửa hàng nhưng khóa kết nối lỗi: ${errMsg(e)}. Cập nhật lại trong Cài đặt.`,
          "err"
        );
      }
      onClose();
      await onDone(connected);
    } catch (e) {
      notify(`Thêm cửa hàng thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Thêm cửa hàng</h2>
        <label className="field">
          <span>Tên cửa hàng</span>
          <input
            autoFocus
            placeholder="vd: Shop A"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </label>
        <label className="field">
          <span>Khóa kết nối Wildberries</span>
          <textarea
            placeholder="Dán khóa kết nối..."
            value={key}
            rows={4}
            onChange={(e) => setKey(e.target.value)}
          />
          <small className="muted">Lấy từ trang người bán Wildberries.</small>
        </label>
        <div className="modal-actions">
          <button className="ghost" onClick={onClose} disabled={busy}>
            Huỷ
          </button>
          <button className="primary" onClick={save} disabled={busy}>
            {busy ? "Đang lưu & kiểm tra..." : "Thêm cửa hàng"}
          </button>
        </div>
      </div>
    </div>
  );
}
