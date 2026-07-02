import { useCallback, useEffect, useRef, useState } from "react";
import { RefreshCw, ImageOff, Search, SlidersHorizontal, X } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type { ProductRow } from "../types/wb";
import type { Notify } from "../App";

function ProductImage({ src, alt }: { src: string; alt: string }) {
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState(false);
  const showImg = !!src && !error;

  // If the <img> mounts already-complete (cached), onLoad may never fire.
  const imgRef = useCallback((node: HTMLImageElement | null) => {
    if (node && node.complete) {
      if (node.naturalWidth > 0) setLoaded(true);
      else setError(true);
    }
  }, []);

  return (
    <div className="pimg">
      {showImg ? (
        <img
          ref={imgRef}
          src={src}
          alt={alt}
          loading="lazy"
          decoding="async"
          onLoad={() => setLoaded(true)}
          onError={() => setError(true)}
        />
      ) : (
        <div className="pimg-placeholder">
          <ImageOff size={18} />
        </div>
      )}
      {showImg && !loaded && <div className="pimg-skeleton" />}
    </div>
  );
}

export function CategoryFilter({
  all,
  selected,
  onChange,
}: {
  all: string[];
  selected: string[];
  onChange: (next: string[]) => void;
}) {
  const [open, setOpen] = useState(false);
  const [q, setQ] = useState("");
  const filtered = all.filter((c) => c.toLowerCase().includes(q.toLowerCase()));

  function toggle(cat: string) {
    onChange(
      selected.includes(cat)
        ? selected.filter((c) => c !== cat)
        : [...selected, cat]
    );
  }

  return (
    <div className="cat-filter">
      <button
        className={`filter-btn ${selected.length ? "has" : ""}`}
        onClick={() => setOpen((o) => !o)}
      >
        <SlidersHorizontal size={16} />
        Danh mục
        {selected.length > 0 && <span className="filter-count">{selected.length}</span>}
      </button>
      {open && (
        <>
          <div className="menu-backdrop" onClick={() => setOpen(false)} />
          <div className="menu-pane filter-pane">
            <div className="search-box">
              <Search size={15} className="search-ico" />
              <input
                placeholder="Tìm danh mục..."
                value={q}
                onChange={(e) => setQ(e.target.value)}
              />
            </div>
            <div className="filter-list">
              {filtered.map((c) => (
                <label key={c} className="filter-item">
                  <input
                    type="checkbox"
                    checked={selected.includes(c)}
                    onChange={() => toggle(c)}
                  />
                  <span>{c}</span>
                </label>
              ))}
              {filtered.length === 0 && (
                <p className="muted" style={{ padding: "6px 10px" }}>
                  Không có danh mục.
                </p>
              )}
            </div>
            {selected.length > 0 && (
              <button className="ghost clear-filter" onClick={() => onChange([])}>
                <X size={14} /> Bỏ chọn tất cả
              </button>
            )}
          </div>
        </>
      )}
    </div>
  );
}

export default function Products({
  notify,
  connected,
}: {
  notify: Notify;
  connected: boolean;
}) {
  const [rows, setRows] = useState<ProductRow[]>([]);
  const [search, setSearch] = useState("");
  const [cats, setCats] = useState<string[]>([]);
  const [selectedCats, setSelectedCats] = useState<string[]>([]);
  const [syncing, setSyncing] = useState(false);
  const [loading, setLoading] = useState(false);
  const didInit = useRef(false);

  const load = useCallback(
    async (term: string, categories: string[]) => {
      setLoading(true);
      try {
        setRows(await api.listProducts(term, categories, 1000));
      } catch (e) {
        notify(errMsg(e), "err");
      } finally {
        setLoading(false);
      }
    },
    [notify]
  );

  async function loadCats() {
    try {
      setCats(await api.listCategories());
    } catch {
      /* ignore */
    }
  }

  const sync = useCallback(async () => {
    setSyncing(true);
    try {
      await api.syncProducts();
      await Promise.all([load(search, selectedCats), loadCats()]);
    } catch (e) {
      notify(`Cập nhật thất bại: ${errMsg(e)}`, "err");
    } finally {
      setSyncing(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [load, notify]);

  // Debounced auto-search: reloads as the user types / changes filters.
  useEffect(() => {
    const t = setTimeout(() => load(search, selectedCats), didInit.current ? 350 : 0);
    return () => clearTimeout(t);
  }, [search, selectedCats, load]);

  // On mount: load categories and auto-sync if this connected store is empty.
  useEffect(() => {
    didInit.current = true;
    loadCats();
    (async () => {
      if (!connected) return;
      try {
        const existing = await api.listProducts("", [], 1);
        if (existing.length === 0) sync();
      } catch {
        /* ignore */
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function openWb(nmId: number) {
    api
      .openUrl(`https://www.wildberries.ru/catalog/${nmId}/detail.aspx?targetUrl=GP`)
      .catch((e) => notify(errMsg(e), "err"));
  }

  function fmtDate(s: string | null): string {
    if (!s) return "";
    const d = new Date(s);
    return isNaN(d.getTime()) ? "" : d.toLocaleDateString("vi-VN");
  }

  return (
    <div className="page">
      <div className="page-head">
        <h1>Sản phẩm</h1>
        <button className="primary" onClick={sync} disabled={syncing}>
          <RefreshCw size={16} className={syncing ? "spin" : ""} />
          {syncing ? "Đang cập nhật..." : "Cập nhật sản phẩm"}
        </button>
      </div>

      <div className="row toolbar">
        <div className="search-box">
          <Search size={16} className="search-ico" />
          <input
            placeholder="Tìm theo mã WB hoặc mã hàng..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <CategoryFilter
          all={cats}
          selected={selectedCats}
          onChange={setSelectedCats}
        />
        {loading && <span className="muted inline-loading">Đang tải...</span>}
      </div>

      <p className="muted">{rows.length} sản phẩm</p>

      <div className="table-wrap">
        <table className="products-table">
          <thead>
            <tr>
              <th className="col-img">Ảnh</th>
              <th className="col-info">Sản phẩm</th>
              <th className="col-size">Cỡ</th>
              <th className="col-color">Màu</th>
              <th className="col-updated">Cập nhật</th>
              <th className="col-spacer"></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((p) => (
              <tr key={p.nmId}>
                <td className="col-img">
                  <ProductImage src={p.photo} alt={p.title} />
                </td>
                <td className="col-info">
                  <div className="pinfo">
                    <button
                      className="pinfo-title"
                      title={p.title}
                      onClick={() => openWb(p.nmId)}
                    >
                      {p.title || "(không tên)"}
                    </button>
                    <div className="pinfo-sub">
                      {[p.subjectName, p.brand].filter(Boolean).join(" · ")}
                    </div>
                    <div className="pinfo-code">WB item No.: {p.nmId}</div>
                    <div className="pinfo-code">Seller item No.: {p.vendorCode}</div>
                  </div>
                </td>
                <td className="col-size">{p.sizes}</td>
                <td className="col-color">{p.color}</td>
                <td className="col-updated">{fmtDate(p.updatedAt)}</td>
                <td className="col-spacer"></td>
              </tr>
            ))}
            {rows.length === 0 && !loading && (
              <tr>
                <td colSpan={6} className="empty">
                  Chưa có sản phẩm. Bấm "Cập nhật sản phẩm".
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
