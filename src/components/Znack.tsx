import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw, ShoppingCart, X, Link2, CircleAlert } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import {
  ACTIVE_STAGES,
  STAGE_LABELS,
  type CategoryGender,
  type ZnackMappingRule,
  type ZnackProduct,
} from "../types/znack";
import ZnackMappingModal from "./ZnackMappingModal";
import type { Notify } from "../App";

export default function Znack({ notify }: { notify: Notify }) {
  const [products, setProducts] = useState<ZnackProduct[]>([]);
  const [rules, setRules] = useState<ZnackMappingRule[]>([]);
  const [catGenders, setCatGenders] = useState<CategoryGender[]>([]);
  const [busy, setBusy] = useState(false);
  const pollTimer = useRef<number | null>(null);

  const [buyFor, setBuyFor] = useState<ZnackProduct | null>(null);
  const [buyQty, setBuyQty] = useState("10");
  const [buying, setBuying] = useState(false);

  const [mapFor, setMapFor] = useState<ZnackProduct | null>(null);

  const load = useCallback(async () => {
    try {
      const [p, r, cg] = await Promise.all([
        api.znackListProducts(),
        api.znackListRules(),
        api.znackCategoryGenders(),
      ]);
      setProducts(p);
      setRules(r);
      setCatGenders(cg);
    } catch (e) {
      notify(errMsg(e), "err");
    }
  }, [notify]);

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Poll while any pipeline is active so buy progress stays live.
  const hasActive = products.some((p) => p.pipeline && ACTIVE_STAGES.has(p.pipeline.stage));
  useEffect(() => {
    if (!hasActive) return;
    pollTimer.current = window.setInterval(() => {
      api.znackListProducts().then(setProducts).catch(() => {});
    }, 4000);
    return () => {
      if (pollTimer.current) window.clearInterval(pollTimer.current);
    };
  }, [hasActive]);

  async function sync() {
    setBusy(true);
    try {
      const message = await api.znackSyncProducts();
      notify(message, "ok");
      await load();
    } catch (e) {
      notify(`Đồng bộ GTIN thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBusy(false);
    }
  }

  async function buy() {
    if (!buyFor) return;
    const qty = parseInt(buyQty, 10);
    if (!qty || qty <= 0) {
      notify("Số lượng không hợp lệ.", "err");
      return;
    }
    setBuying(true);
    try {
      await api.znackBuyKiz(buyFor.gtin, qty);
      notify(`Đã bắt đầu mua ${qty} mã KIZ cho GTIN ${buyFor.gtin}.`, "ok");
      setBuyFor(null);
      await load();
    } catch (e) {
      notify(`Mua KIZ thất bại: ${errMsg(e)}`, "err");
    } finally {
      setBuying(false);
    }
  }

  async function abortPipeline(id: number) {
    try {
      await api.znackAbortPipeline(id);
      notify("Đã huỷ phiên mua.", "info");
      await load();
    } catch (e) {
      notify(errMsg(e), "err");
    }
  }

  function ruleCountFor(gtin: string): number {
    return rules.filter((r) => r.gtin === gtin).length;
  }

  function statusBadge(p: ZnackProduct) {
    const orderable =
      p.goodMarkFlag === true &&
      !["draft", "moderation", "errors", "notsigned", "archived"].includes(
        p.cardStatus.toLowerCase()
      );
    return (
      <span className={`badge ${orderable ? "complete" : "confirm"}`}>
        {orderable ? "Sẵn sàng" : p.cardStatus || "Chưa rõ"}
      </span>
    );
  }

  return (
    <div className="page">
      <div className="page-head">
        <h1>Честный ЗНАК — KIZ</h1>
        <button className="primary" onClick={sync} disabled={busy}>
          <RefreshCw size={16} className={busy ? "spin" : ""} />
          {busy ? "Đang đồng bộ..." : "Đồng bộ GTIN"}
        </button>
      </div>

      <div className="card">
        <h2>GTIN đã đăng ký ({products.length})</h2>
        {products.length === 0 && (
          <p className="muted">
            Chưa có GTIN nào. Cấu hình Честный ЗНАК trong Cài đặt rồi bấm "Đồng bộ GTIN".
          </p>
        )}
        {products.length > 0 && (
          <table className="znack-table">
            <thead>
              <tr>
                <th>GTIN</th>
                <th>Tên hàng</th>
                <th>Trạng thái thẻ</th>
                <th>KIZ còn</th>
                <th>Mapping</th>
                <th>Tiến trình mua</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {products.map((p) => (
                <tr key={p.gtin}>
                  <td className="mono">{p.gtin}</td>
                  <td>{p.productName || <span className="muted">—</span>}</td>
                  <td>{statusBadge(p)}</td>
                  <td className={p.available > 0 ? "znack-avail" : "muted"}>
                    {p.available}
                    {p.reserved > 0 && (
                      <span className="muted"> (+{p.reserved} giữ)</span>
                    )}
                  </td>
                  <td>
                    {ruleCountFor(p.gtin) > 0 ? (
                      <span className="chip">{ruleCountFor(p.gtin)} liên kết</span>
                    ) : (
                      <span className="muted">chưa có</span>
                    )}
                  </td>
                  <td>
                    {p.pipeline ? (
                      <span
                        className="znack-stage"
                        title={p.pipeline.errorMessage ?? undefined}
                      >
                        {ACTIVE_STAGES.has(p.pipeline.stage) && (
                          <RefreshCw size={12} className="spin" />
                        )}
                        {p.pipeline.errorMessage && <CircleAlert size={13} />}
                        {STAGE_LABELS[p.pipeline.stage] ?? p.pipeline.stage}
                        {ACTIVE_STAGES.has(p.pipeline.stage) && (
                          <button
                            className="icon-btn"
                            title="Huỷ phiên mua"
                            onClick={() => abortPipeline(p.pipeline!.id)}
                          >
                            <X size={13} />
                          </button>
                        )}
                      </span>
                    ) : (
                      <span className="muted">—</span>
                    )}
                  </td>
                  <td className="znack-row-actions">
                    <button className="ghost znack-buy" onClick={() => setMapFor(p)}>
                      <Link2 size={14} /> Mapping
                    </button>
                    <button
                      className="icon-btn"
                      title="Mua mã KIZ"
                      disabled={!!(p.pipeline && ACTIVE_STAGES.has(p.pipeline.stage))}
                      onClick={() => {
                        setBuyFor(p);
                        setBuyQty("10");
                      }}
                    >
                      <ShoppingCart size={16} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {mapFor && (
        <ZnackMappingModal
          product={mapFor}
          rules={rules}
          catGenders={catGenders}
          notify={notify}
          onClose={() => setMapFor(null)}
          onChanged={load}
        />
      )}

      {buyFor && (
        <div className="modal-overlay" onClick={() => !buying && setBuyFor(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>Mua mã KIZ</h2>
            <p className="muted">
              GTIN <b className="mono">{buyFor.gtin}</b>
              {buyFor.productName ? ` — ${buyFor.productName}` : ""}
            </p>
            <label className="field">
              <span>Số lượng mã</span>
              <input
                autoFocus
                type="number"
                min={1}
                value={buyQty}
                onChange={(e) => setBuyQty(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && buy()}
              />
            </label>
            <div className="modal-actions">
              <button className="ghost" onClick={() => setBuyFor(null)} disabled={buying}>
                Huỷ
              </button>
              <button className="primary" onClick={buy} disabled={buying}>
                {buying ? "Đang gửi..." : "Mua mã"}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
