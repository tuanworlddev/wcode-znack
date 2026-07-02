import { useEffect, useRef, useState } from "react";
import { RefreshCw, FolderPlus, PackagePlus, X, QrCode, Boxes, ChevronRight, MoreVertical, Printer } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type { OrderCounts, OrderRow, SupplyRow } from "../types/wb";
import { cargoLabel, deliveryLabel, fmtDate } from "../lib/format";
import { printSupply } from "../lib/printSupply";
import OrderCard from "./OrderCard";
import SupplyDetail from "./SupplyDetail";
import type { Notify } from "../App";

type TabKey = "new" | "confirm" | "complete";
const TABS: { key: TabKey; label: string }[] = [
  { key: "new", label: "Đơn mới" },
  { key: "confirm", label: "Đang chuẩn bị" },
  { key: "complete", label: "Đang giao" },
];

export default function Orders({
  notify,
  connected,
}: {
  notify: Notify;
  connected: boolean;
}) {
  const [tab, setTab] = useState<TabKey>("new");
  const [rows, setRows] = useState<OrderRow[]>([]);
  const [counts, setCounts] = useState<OrderCounts>({ new: 0, confirm: 0, complete: 0 });
  const [selected, setSelected] = useState<Set<number>>(new Set());
  const [supplies, setSupplies] = useState<SupplyRow[]>([]);
  const [showSheet, setShowSheet] = useState(false);
  const [sheetLoading, setSheetLoading] = useState(false);
  const [showNew, setShowNew] = useState(false);
  const [newName, setNewName] = useState("");
  const [busy, setBusy] = useState(false);
  const [detailSupply, setDetailSupply] = useState<SupplyRow | null>(null);
  const [menuFor, setMenuFor] = useState<string | null>(null);
  const [quickPrinting, setQuickPrinting] = useState<string | null>(null);
  const didMount = useRef(false);

  const selectable = tab === "new";

  async function loadRows(t = tab) {
    try {
      setRows(await api.listOrders(t, null));
      setSelected(new Set());
    } catch (e) {
      notify(errMsg(e), "err");
    }
  }
  async function loadCounts() {
    try {
      setCounts(await api.orderStatusCounts());
    } catch {
      /* ignore */
    }
  }
  async function loadSupplies() {
    try {
      const s = await api.listSupplies();
      setSupplies(s.filter((x) => !x.done));
    } catch {
      /* ignore */
    }
  }

  // Open the "add to existing supply" sheet, syncing supplies first so the
  // in-preparation list is up to date.
  async function openSupplySheet() {
    setShowSheet(true);
    setSheetLoading(true);
    try {
      await api.syncSupplies();
      await loadSupplies();
    } catch (e) {
      notify(errMsg(e), "err");
    } finally {
      setSheetLoading(false);
    }
  }

  useEffect(() => {
    // Skip the very first run (handled by the mount effect below) to avoid a
    // duplicate load right after the auto-sync.
    if (!didMount.current) return;
    if (tab === "confirm") {
      syncPreparingSupplies();
    } else {
      loadRows();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tab]);

  // The "confirm" tab lists supplies in preparation; refresh them from WB.
  async function syncPreparingSupplies() {
    setBusy(true);
    try {
      if (connected) await api.syncSupplies();
      await Promise.all([loadSupplies(), loadCounts()]);
    } catch (e) {
      notify(errMsg(e), "err");
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    didMount.current = true;
    loadCounts();
    loadSupplies();
    // Auto-fetch latest orders from WB when opening the page.
    if (connected) {
      syncOrders();
    } else {
      loadRows();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function toggle(id: number) {
    setSelected((s) => {
      const n = new Set(s);
      n.has(id) ? n.delete(id) : n.add(id);
      return n;
    });
  }
  function toggleAll() {
    setSelected((s) =>
      s.size === rows.length ? new Set() : new Set(rows.map((r) => r.id))
    );
  }

  async function syncOrders() {
    setBusy(true);
    try {
      await api.syncOrders();
      await Promise.all([loadRows(), loadCounts(), loadSupplies()]);
    } catch (e) {
      notify(`Cập nhật thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  function defaultSupplyName() {
    const d = new Date();
    const dd = String(d.getDate()).padStart(2, "0");
    const mm = String(d.getMonth() + 1).padStart(2, "0");
    return `Shipment ${dd}.${mm}.${d.getFullYear()}`;
  }

  async function createSupply() {
    const name = newName.trim() || defaultSupplyName();
    setBusy(true);
    try {
      const id = await api.createSupply(name);
      await api.addOrdersToSupply(id, [...selected]);
      notify(`Đã tạo lô "${name}" và thêm ${selected.size} đơn.`, "ok");
      setShowNew(false);
      setNewName("");
      await Promise.all([loadRows(), loadCounts(), loadSupplies()]);
    } catch (e) {
      notify(`Tạo lô hàng thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  // Quick print from the supply card's ⋮ menu, without opening the detail page.
  async function quickPrint(s: SupplyRow) {
    setMenuFor(null);
    setQuickPrinting(s.id);
    try {
      await printSupply(s.id, s.name, notify);
    } finally {
      setQuickPrinting(null);
    }
  }

  async function addToSupply(id: string) {
    setBusy(true);
    try {
      await api.addOrdersToSupply(id, [...selected]);
      notify(`Đã thêm ${selected.size} đơn vào lô hàng.`, "ok");
      setShowSheet(false);
      await Promise.all([loadRows(), loadCounts(), loadSupplies()]);
    } catch (e) {
      notify(`Thêm đơn thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  if (detailSupply) {
    return (
      <SupplyDetail
        supply={detailSupply}
        notify={notify}
        onBack={() => {
          setDetailSupply(null);
          syncPreparingSupplies();
        }}
      />
    );
  }

  return (
    <div className="page orders-page">
      <div className="page-header">
        <div className="page-head">
          <h1>Đơn hàng</h1>
          <button
            className="primary"
            onClick={tab === "confirm" ? syncPreparingSupplies : syncOrders}
            disabled={busy}
          >
            <RefreshCw size={16} className={busy ? "spin" : ""} />
            {busy ? "Đang cập nhật..." : tab === "confirm" ? "Cập nhật lô hàng" : "Cập nhật đơn"}
          </button>
        </div>

        <div className="tabs">
          {TABS.map((t) => (
            <button
              key={t.key}
              className={`tab ${tab === t.key ? "active" : ""}`}
              onClick={() => setTab(t.key)}
            >
              {t.label}
              <span className="tab-count">
                {t.key === "confirm" ? supplies.length : counts[t.key]}
              </span>
            </button>
          ))}
        </div>

        {selectable && rows.length > 0 && (
          <label className="select-all">
            <input
              type="checkbox"
              checked={selected.size === rows.length}
              onChange={toggleAll}
            />
            Chọn tất cả ({rows.length})
          </label>
        )}
      </div>

      {tab === "confirm" ? (
        <div className="supply-list">
          {supplies.map((s) => (
            <div key={s.id} className="supply-card" onClick={() => setDetailSupply(s)}>
              <div className="sc-main">
                <div className="sc-name">{s.name}</div>
                <div className="muted">{fmtDate(s.createdAt)}</div>
                {cargoLabel(s.cargoType) && (
                  <span className="chip">{cargoLabel(s.cargoType)}</span>
                )}
                <div className="muted sc-delivery">{deliveryLabel(s.isPickup)}</div>
              </div>

              <div className="sc-col">
                <div className="sc-id">
                  <QrCode size={14} /> {s.id}
                </div>
                <div className="muted">{s.done ? "Có mã QR" : "Chưa có mã QR"}</div>
              </div>

              <div className="sc-col">
                <div className="sc-count">
                  <Boxes size={14} /> {s.orderCount} đơn
                </div>
              </div>

              <div className="sc-col">
                <span className={`badge ${s.done ? "complete" : "confirm"}`}>
                  {s.done ? "Đã bàn giao" : "Đang chuẩn bị"}
                </span>
              </div>

              <div className="sc-actions">
                <span className="muted sc-open">Xem &amp; in nhãn</span>
                <ChevronRight size={20} className="sc-chev" />
                <div className="sc-menu-wrap" onClick={(e) => e.stopPropagation()}>
                  <button
                    className="icon-btn sc-dots"
                    disabled={quickPrinting !== null}
                    onClick={() => setMenuFor(menuFor === s.id ? null : s.id)}
                  >
                    {quickPrinting === s.id ? (
                      <RefreshCw size={17} className="spin" />
                    ) : (
                      <MoreVertical size={17} />
                    )}
                  </button>
                  {menuFor === s.id && (
                    <div className="sc-menu">
                      <button onClick={() => quickPrint(s)} disabled={quickPrinting !== null}>
                        <Printer size={15} /> In nhãn
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
          {supplies.length === 0 && (
            <div className="empty-card muted">
              Chưa có lô hàng đang chuẩn bị. Chọn đơn ở mục "Đơn mới" để tạo lô.
            </div>
          )}
        </div>
      ) : (
        <div className="order-list">
          {rows.map((o) => (
            <OrderCard
              key={o.id}
              o={o}
              selectable={selectable}
              selected={selected.has(o.id)}
              onToggle={toggle}
            />
          ))}
          {rows.length === 0 && (
            <div className="empty-card muted">
              Không có đơn ở mục này. Bấm "Cập nhật đơn".
            </div>
          )}
        </div>
      )}

      {/* Click-away layer for the supply-card ⋮ menu */}
      {menuFor && <div className="menu-scrim" onClick={() => setMenuFor(null)} />}

      {/* Floating selection bar (rises from the bottom) */}
      {selectable && selected.size > 0 && (
        <div className="sel-bar">
          <span className="sel-count">
            <b>{selected.size}</b> đơn đã chọn
          </span>
          <div className="sel-bar-right">
            <button onClick={openSupplySheet} disabled={busy}>
              <PackagePlus size={16} /> Thêm vào lô có sẵn
            </button>
            <button className="primary" onClick={() => { setNewName(defaultSupplyName()); setShowNew(true); }} disabled={busy}>
              <FolderPlus size={16} /> Tạo lô hàng mới
            </button>
          </div>
        </div>
      )}

      {/* Bottom sheet: pick an existing in-preparation supply */}
      {showSheet && (
        <div className="sheet-overlay" onClick={() => setShowSheet(false)}>
          <div className="bottom-sheet" onClick={(e) => e.stopPropagation()}>
            <div className="sheet-head">
              <h3>Chọn lô hàng đang chuẩn bị</h3>
              <button className="icon-btn" onClick={() => setShowSheet(false)}>
                <X size={18} />
              </button>
            </div>
            <div className="sheet-list">
              {sheetLoading && (
                <p className="muted" style={{ padding: 12 }}>
                  Đang tải danh sách lô hàng...
                </p>
              )}
              {!sheetLoading &&
                supplies.map((s) => (
                <button
                  key={s.id}
                  className="sheet-item"
                  onClick={() => addToSupply(s.id)}
                  disabled={busy}
                >
                  <div className="sheet-item-main">
                    <b>{s.name}</b>
                    <span className="muted">{s.id}</span>
                  </div>
                  <div className="sheet-item-meta muted">
                    {s.orderCount} đơn · {fmtDate(s.createdAt)}
                    {cargoLabel(s.cargoType) ? ` · ${cargoLabel(s.cargoType)}` : ""}
                  </div>
                </button>
              ))}
              {!sheetLoading && supplies.length === 0 && (
                <p className="muted" style={{ padding: 12 }}>
                  Chưa có lô hàng đang chuẩn bị. Hãy tạo lô hàng mới.
                </p>
              )}
            </div>
          </div>
        </div>
      )}

      {/* New supply modal */}
      {showNew && (
        <div className="modal-overlay" onClick={() => setShowNew(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>Tạo lô hàng mới</h2>
            <label className="field">
              <span>Tên lô hàng</span>
              <input
                autoFocus
                placeholder="vd: Lô sáng 01/07"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && createSupply()}
              />
            </label>
            <p className="muted">{selected.size} đơn sẽ được thêm vào lô này.</p>
            <div className="modal-actions">
              <button className="ghost" onClick={() => setShowNew(false)} disabled={busy}>
                Huỷ
              </button>
              <button className="primary" onClick={createSupply} disabled={busy}>
                {busy ? "Đang tạo..." : "Tạo lô hàng"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
