import { ImageOff, MapPin, Truck } from "lucide-react";
import { api } from "../api/tauri";
import type { OrderRow } from "../types/wb";
import { cargoLabel, deliveryLabel, fmtDate, fmtPrice, timeAgo } from "../lib/format";

export default function OrderCard({
  o,
  selectable = false,
  selected = false,
  onToggle,
}: {
  o: OrderRow;
  selectable?: boolean;
  selected?: boolean;
  onToggle?: (id: number) => void;
}) {
  const categoryBrand = [o.subjectName, o.brand].filter(Boolean).join(" · ");

  function openInWb() {
    if (!o.nmId) return;
    api.openUrl(`https://www.wildberries.ru/catalog/${o.nmId}/detail.aspx`).catch(() => {});
  }

  return (
    <div className={`order-card ${selected ? "sel" : ""}`}>
      {selectable && (
        <label className="oc-check">
          <input
            type="checkbox"
            checked={selected}
            onChange={() => onToggle?.(o.id)}
          />
        </label>
      )}

      <div className="oc-meta">
        <div className="oc-id">{o.id}</div>
        <div className="oc-date">{fmtDate(o.createdAt)}</div>
        <div className="oc-ago muted">{timeAgo(o.createdAt)}</div>
        {cargoLabel(o.cargoType) && (
          <span className="chip">{cargoLabel(o.cargoType)}</span>
        )}
        <div className="oc-delivery">
          <Truck size={13} /> {deliveryLabel(o.pickup)}
        </div>
      </div>

      <div className="oc-item">
        <div className="pimg oc-thumb">
          {o.photo ? (
            <img src={o.photo} alt="" loading="lazy" decoding="async" />
          ) : (
            <div className="pimg-placeholder">
              <ImageOff size={16} />
            </div>
          )}
        </div>
        <div className="oc-item-info">
          <div
            className={`oc-title ${o.nmId ? "oc-title-link" : ""}`}
            onClick={openInWb}
            title={o.title ?? undefined}
          >
            {o.title ?? <span className="muted">— chưa có thông tin sản phẩm —</span>}
          </div>
          {categoryBrand && <div className="oc-brand muted">{categoryBrand}</div>}
          <div className="oc-code muted">Mã: {o.article}</div>
          {o.techSize && <div className="oc-code muted">Size: {o.techSize}</div>}
        </div>
      </div>

      <div className="oc-price">{fmtPrice(o.price)}</div>

      <div className="oc-wh">
        {o.warehouse && <div className="oc-wh-name">{o.warehouse}</div>}
        {o.offices && (
          <div className="oc-office muted">
            <MapPin size={13} /> {o.offices}
          </div>
        )}
      </div>
    </div>
  );
}
