import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ImageOff, Printer, Search, X } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type { PrintOrder, SkuItem } from "../types/wb";
import { sgtinDisplay } from "../lib/labelTemplate";
import { buildFboPdfBase64 } from "../pdf";
import { CategoryFilter } from "./Products";
import type { Notify } from "../App";

export default function Fbo({ notify }: { notify: Notify }) {
  const [rows, setRows] = useState<SkuItem[]>([]);
  const [search, setSearch] = useState("");
  const [cats, setCats] = useState<string[]>([]);
  const [allCats, setAllCats] = useState<string[]>([]);
  const [qty, setQty] = useState<Record<string, string>>({});
  const [printing, setPrinting] = useState(false);
  const didInit = useRef(false);

  const load = useCallback(
    async (term: string, categories: string[]) => {
      try {
        setRows(await api.listSkuItems(term, categories));
      } catch (e) {
        notify(errMsg(e), "err");
      }
    },
    [notify]
  );

  useEffect(() => {
    api.listCategories().then(setAllCats).catch(() => {});
  }, []);

  // Debounced search + filters.
  useEffect(() => {
    const t = setTimeout(() => load(search, cats), didInit.current ? 350 : 0);
    didInit.current = true;
    return () => clearTimeout(t);
  }, [search, cats, load]);

  const selected = useMemo(
    () =>
      rows
        .map((r) => ({ item: r, count: parseInt(qty[r.barcode] ?? "", 10) || 0 }))
        .filter((x) => x.count > 0),
    [rows, qty]
  );
  const totalUnits = selected.reduce((sum, x) => sum + x.count, 0);

  function setCount(barcode: string, value: string) {
    setQty((q) => ({ ...q, [barcode]: value }));
  }

  async function print() {
    if (selected.length === 0) return;
    setPrinting(true);
    let token: string | null = null;
    try {
      // Reserve KIZ codes (blocks the whole job if a mapped item lacks codes).
      const reservation = await api.reserveFboCodes(
        selected.map(({ item, count }) => ({
          subjectName: item.subjectName,
          gender: item.gender,
          quantity: count,
        }))
      );
      token = reservation.token;

      // One unit = one label rendered once, printed twice, sharing one KIZ.
      const units: PrintOrder[] = [];
      let pairNo = 0;
      selected.forEach(({ item, count }, i) => {
        const codes = reservation.codes[i] ?? [];
        for (let u = 0; u < count; u++) {
          pairNo += 1;
          const kizCode = codes[u] ?? "";
          units.push({
            orderId: 0,
            barcode: item.barcode,
            title: item.title,
            vendorCode: item.vendorCode,
            techSize: item.techSize,
            color: item.color,
            brand: item.brand,
            subjectName: item.subjectName,
            photo: item.photo,
            nmId: item.nmId,
            stickerPng: "",
            partA: "",
            partB: "",
            kizCode,
            kizSgtin: kizCode ? sgtinDisplay(kizCode) : "",
            pairNo,
          });
        }
      });

      const pdf = await buildFboPdfBase64(units);
      const stamp = new Date();
      const name = `FBO-${String(stamp.getDate()).padStart(2, "0")}.${String(
        stamp.getMonth() + 1
      ).padStart(2, "0")}.${stamp.getFullYear()}-${String(stamp.getHours()).padStart(2, "0")}${String(
        stamp.getMinutes()
      ).padStart(2, "0")}`;
      await api.saveAndOpenPdf(name, pdf);

      if (token) {
        try {
          await api.finishKizReservation(token, true);
        } catch (e) {
          notify(`Không chốt được kho mã KIZ: ${errMsg(e)}`, "err");
        }
        token = null;
      }
      notify(
        `Đã tạo file in FBO: ${totalUnits} cặp (${totalUnits * 2} trang), đánh số 1–${totalUnits}.`,
        "ok"
      );
      setQty({});
    } catch (e) {
      if (token) await api.finishKizReservation(token, false).catch(() => {});
      notify(`In FBO thất bại: ${errMsg(e)}`, "err");
    } finally {
      setPrinting(false);
    }
  }

  return (
    <div className="page fbo-page">
      <div className="page-header">
        <div className="page-head">
          <h1>In tem FBO</h1>
          <button
            className="primary"
            onClick={print}
            disabled={printing || totalUnits === 0}
          >
            <Printer size={16} />
            {printing
              ? "Đang tạo file in..."
              : totalUnits > 0
                ? `In ${totalUnits} cặp (${totalUnits * 2} trang)`
                : "In tem"}
          </button>
        </div>

        <div className="fbo-filters">
          <div className="search-box">
            <Search size={16} className="search-ico" />
            <input
              placeholder="Tìm theo tên, article, barcode, nmID..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            {search && (
              <button className="icon-btn ghost" onClick={() => setSearch("")}>
                <X size={14} />
              </button>
            )}
          </div>
          <CategoryFilter all={allCats} selected={cats} onChange={setCats} />
          <span className="muted">{rows.length} barcode</span>
        </div>
      </div>

      <div className="table-wrap fbo-table-wrap">
        <table>
          <thead>
            <tr>
              <th></th>
              <th>Sản phẩm</th>
              <th>Danh mục</th>
              <th>Màu</th>
              <th>Size</th>
              <th>Barcode</th>
              <th className="fbo-qty-col">Số lượng in</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.barcode} className={qty[r.barcode] ? "sel" : ""}>
                <td className="fbo-img-cell">
                  {r.photo ? (
                    <img src={r.photo} alt="" loading="lazy" className="fbo-img" />
                  ) : (
                    <span className="pimg-placeholder fbo-img">
                      <ImageOff size={14} />
                    </span>
                  )}
                </td>
                <td>
                  <div
                    className="fbo-title oc-title-link"
                    title={r.title}
                    onClick={() =>
                      r.nmId &&
                      api
                        .openUrl(`https://www.wildberries.ru/catalog/${r.nmId}/detail.aspx`)
                        .catch(() => {})
                    }
                  >
                    {r.title || "—"}
                  </div>
                  <div className="muted">
                    {[r.brand, r.vendorCode].filter(Boolean).join(" · ")}
                  </div>
                </td>
                <td>{r.subjectName}</td>
                <td>{r.color}</td>
                <td>
                  {r.techSize}
                  {r.wbSize && r.wbSize !== r.techSize ? ` - ${r.wbSize}` : ""}
                </td>
                <td className="mono">{r.barcode}</td>
                <td className="fbo-qty-col">
                  <input
                    className="fbo-qty"
                    type="number"
                    min={0}
                    placeholder="0"
                    value={qty[r.barcode] ?? ""}
                    onChange={(e) => setCount(r.barcode, e.target.value)}
                  />
                </td>
              </tr>
            ))}
            {rows.length === 0 && (
              <tr>
                <td className="empty" colSpan={7}>
                  Không có sản phẩm. Đồng bộ sản phẩm hoặc đổi bộ lọc.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
}
