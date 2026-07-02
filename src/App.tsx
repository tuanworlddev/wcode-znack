import { useEffect, useState, useCallback } from "react";
import {
  Package,
  ClipboardList,
  QrCode,
  PenTool,
  Barcode,
  Plus,
  Menu,
  type LucideIcon,
} from "lucide-react";
import { api, errMsg } from "./api/tauri";
import type { StoreInfo } from "./types/wb";
import Settings from "./components/Settings";
import Products from "./components/Products";
import Orders from "./components/Orders";
import Znack from "./components/Znack";
import LabelDesigner from "./components/LabelDesigner";
import Fbo from "./components/Fbo";
import AddStoreModal from "./components/AddStoreModal";
import StoreMenu from "./components/StoreMenu";
import "./App.css";

export type Toast = { id: number; text: string; kind: "ok" | "err" | "info" };
export type Notify = (text: string, kind?: Toast["kind"]) => void;

type Tab = "settings" | "products" | "orders" | "fbo" | "znack" | "label";

const TABS: { key: Tab; label: string; icon: LucideIcon }[] = [
  { key: "products", label: "Sản phẩm", icon: Package },
  { key: "orders", label: "Đơn hàng", icon: ClipboardList },
  { key: "fbo", label: "In tem FBO", icon: Barcode },
  { key: "znack", label: "KIZ / ЧЗ", icon: QrCode },
  { key: "label", label: "Thiết kế tem", icon: PenTool },
];

export default function App() {
  const [tab, setTab] = useState<Tab>("products");
  const [stores, setStores] = useState<StoreInfo[]>([]);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [showAdd, setShowAdd] = useState(false);

  const active = stores.find((s) => s.active) ?? null;
  const activeId = active?.id ?? null;

  const notify = useCallback<Notify>((text, kind = "info") => {
    const id = Date.now() + Math.floor(Math.random() * 1000);
    setToasts((t) => [...t, { id, text, kind }]);
    setTimeout(() => setToasts((t) => t.filter((x) => x.id !== id)), 6000);
  }, []);

  const loadStores = useCallback(async () => {
    try {
      setStores(await api.listStores());
    } catch (e) {
      notify(errMsg(e), "err");
    }
  }, [notify]);

  useEffect(() => {
    loadStores();
  }, [loadStores]);

  useEffect(() => {
    if (active && !active.hasToken) setTab("settings");
  }, [active]);

  const [sidebarOpen, setSidebarOpen] = useState(false);

  async function switchStore(id: string) {
    if (id === activeId) return;
    try {
      await api.switchStore(id);
      await loadStores();
      notify("Đã chuyển cửa hàng.", "info");
    } catch (e) {
      notify(errMsg(e), "err");
    }
  }

  // After creating a store we switch to Products; the Products view auto-syncs
  // itself when the store is connected and empty (with the button's loading state).
  const handleStoreCreated = useCallback(async () => {
    await loadStores();
    setTab("products");
  }, [loadStores]);

  return (
    <div className="app">
      <header className="topbar">
        <div className="topbar-left">
          <button
            className="hamburger"
            onClick={() => setSidebarOpen((o) => !o)}
            aria-label="Menu"
          >
            <Menu size={20} />
          </button>
          <div className="brand">
            <span className="brand-logo">W</span>
            <span>WCode</span>
          </div>
        </div>
        <div className="topbar-right">
          <StoreMenu
            stores={stores}
            onSwitch={switchStore}
            onAdd={() => setShowAdd(true)}
            onSettings={() => setTab("settings")}
          />
        </div>
      </header>

      <div className="body">
        <nav className={`sidebar ${sidebarOpen ? "open" : ""}`}>
          {TABS.map((t) => {
            const Icon = t.icon;
            return (
              <button
                key={t.key}
                className={`nav-item ${tab === t.key ? "active" : ""}`}
                onClick={() => {
                  setTab(t.key);
                  setSidebarOpen(false);
                }}
              >
                <Icon size={18} className="nav-icon" />
                {t.label}
              </button>
            );
          })}
          <div className="sidebar-footer">
            <span className={`token-dot ${active?.hasToken ? "on" : "off"}`} />
            {active
              ? active.hasToken
                ? "Sẵn sàng"
                : "Chưa kết nối"
              : "Chưa chọn cửa hàng"}
          </div>
        </nav>

        {sidebarOpen && (
          <div className="sidebar-scrim" onClick={() => setSidebarOpen(false)} />
        )}

        {/* keyed by active store so views remount & refetch on switch */}
        <main className="content" key={activeId ?? "none"}>
          {tab === "settings" && (
            <Settings notify={notify} active={active} onStoresChange={loadStores} />
          )}
          {tab !== "settings" && !active && (
            <div className="page empty-state">
              <h1>Chưa có cửa hàng</h1>
              <p className="muted">
                Bấm <b>Thêm cửa hàng</b> ở góc phải để bắt đầu.
              </p>
              <button className="primary" onClick={() => setShowAdd(true)}>
                <Plus size={17} /> Thêm cửa hàng
              </button>
            </div>
          )}
          {tab === "products" && active && (
            <Products notify={notify} connected={!!active.hasToken} />
          )}
          {tab === "orders" && active && (
            <Orders notify={notify} connected={!!active.hasToken} />
          )}
          {tab === "fbo" && active && <Fbo notify={notify} />}
          {tab === "znack" && active && <Znack notify={notify} />}
          {tab === "label" && active && <LabelDesigner notify={notify} />}
        </main>
      </div>

      {showAdd && (
        <AddStoreModal
          notify={notify}
          onClose={() => setShowAdd(false)}
          onDone={handleStoreCreated}
        />
      )}

      <div className="toasts">
        {toasts.map((t) => (
          <div key={t.id} className={`toast ${t.kind}`}>
            {t.text}
          </div>
        ))}
      </div>
    </div>
  );
}
