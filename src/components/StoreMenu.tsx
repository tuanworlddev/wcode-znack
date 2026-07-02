import { useState } from "react";
import { Store, ChevronDown, Check, Plus, Settings as SettingsIcon } from "lucide-react";
import type { StoreInfo } from "../types/wb";

export default function StoreMenu({
  stores,
  onSwitch,
  onAdd,
  onSettings,
}: {
  stores: StoreInfo[];
  onSwitch: (id: string) => void;
  onAdd: () => void;
  onSettings: () => void;
}) {
  const [open, setOpen] = useState(false);
  const active = stores.find((s) => s.active) ?? null;

  function close() {
    setOpen(false);
  }

  return (
    <div className="store-menu">
      <button
        className={`store-trigger ${open ? "open" : ""}`}
        onClick={() => setOpen((o) => !o)}
      >
        <Store size={16} />
        <span className="store-trigger-name">
          {active ? active.name : "Chưa có cửa hàng"}
        </span>
        <ChevronDown size={16} className="store-trigger-chev" />
      </button>

      {open && (
        <>
          <div className="menu-backdrop" onClick={close} />
          <div className="menu-pane">
            {stores.length > 0 && (
              <>
                <div className="menu-label">Cửa hàng</div>
                {stores.map((s) => (
                  <button
                    key={s.id}
                    className={`menu-item ${s.active ? "active" : ""}`}
                    onClick={() => {
                      onSwitch(s.id);
                      close();
                    }}
                  >
                    <span className={`token-dot ${s.hasToken ? "on" : "off"}`} />
                    <span className="menu-item-name">{s.name}</span>
                    {s.active && <Check size={16} />}
                  </button>
                ))}
                <div className="menu-divider" />
              </>
            )}
            <button
              className="menu-item"
              onClick={() => {
                onAdd();
                close();
              }}
            >
              <Plus size={16} /> Thêm cửa hàng
            </button>
            <button
              className="menu-item"
              onClick={() => {
                onSettings();
                close();
              }}
            >
              <SettingsIcon size={16} /> Cài đặt
            </button>
          </div>
        </>
      )}
    </div>
  );
}
