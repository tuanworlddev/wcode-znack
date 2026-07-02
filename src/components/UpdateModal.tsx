import { useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";
import { Download, RefreshCw } from "lucide-react";
import { errMsg } from "../api/tauri";
import { installUpdate } from "../lib/updater";
import type { Notify } from "../App";

/** Startup prompt when a new version is available: install now or later. */
export default function UpdateModal({
  update,
  notify,
  onClose,
}: {
  update: Update;
  notify: Notify;
  onClose: () => void;
}) {
  const [progress, setProgress] = useState<number | null>(null);
  const busy = progress !== null;

  async function install() {
    setProgress(0);
    try {
      await installUpdate(update, setProgress);
      // relaunch() replaces the process — reaching here means it failed.
    } catch (e) {
      setProgress(null);
      notify(`Cập nhật thất bại: ${errMsg(e)}`, "err");
    }
  }

  return (
    <div className="modal-overlay" onClick={() => !busy && onClose()}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>
          <Download size={17} /> Có phiên bản mới
        </h2>
        <p>
          WCode <b className="mono">{update.version}</b> đã sẵn sàng
          {update.date ? ` (${update.date.slice(0, 10)})` : ""}.
        </p>
        {update.body && <p className="muted update-notes">{update.body}</p>}
        {busy ? (
          <p className="muted">
            <RefreshCw size={14} className="spin" /> Đang tải bản cập nhật…{" "}
            {Math.round((progress ?? 0) * 100)}% — app sẽ tự khởi động lại.
          </p>
        ) : (
          <p className="muted">Cập nhật ngay bây giờ? App sẽ khởi động lại sau khi cài.</p>
        )}
        <div className="modal-actions">
          <button className="ghost" onClick={onClose} disabled={busy}>
            Để sau
          </button>
          <button className="primary" onClick={install} disabled={busy}>
            <Download size={15} /> {busy ? "Đang cập nhật..." : "Cập nhật ngay"}
          </button>
        </div>
      </div>
    </div>
  );
}
