import { useMemo, useState } from "react";
import { X, ChevronRight } from "lucide-react";
import { api, errMsg } from "../api/tauri";
import type {
  CategoryGender,
  ZnackMappingRule,
  ZnackProduct,
} from "../types/znack";
import type { Notify } from "../App";

/** Per-GTIN mapping editor: categories on the left, that category's shop
 *  genders (all checked by default) on the right. */
export default function ZnackMappingModal({
  product,
  rules,
  catGenders,
  notify,
  onClose,
  onChanged,
}: {
  product: ZnackProduct;
  rules: ZnackMappingRule[];
  catGenders: CategoryGender[];
  notify: Notify;
  onClose: () => void;
  onChanged: () => Promise<void>;
}) {
  const subjects = useMemo(() => {
    const seen = new Map<string, number>();
    for (const cg of catGenders) {
      seen.set(cg.subjectName, (seen.get(cg.subjectName) ?? 0) + cg.productCount);
    }
    return [...seen.entries()].map(([name, count]) => ({ name, count }));
  }, [catGenders]);

  const [subject, setSubject] = useState<string>("");
  const [checked, setChecked] = useState<Set<string>>(new Set());
  const [saving, setSaving] = useState(false);
  const [localRules, setLocalRules] = useState(rules);

  const genders = useMemo(
    () => catGenders.filter((cg) => cg.subjectName === subject),
    [catGenders, subject]
  );

  function linksFor(subjectName: string, ruleList = localRules) {
    return ruleList.filter((r) => r.subjectName === subjectName);
  }

  function pickSubject(name: string) {
    setSubject(name);
    const mine = linksFor(name).filter((r) => r.gtin === product.gtin);
    const all = catGenders.filter((cg) => cg.subjectName === name).map((cg) => cg.gender);
    if (mine.some((r) => r.wildcardGender) || mine.length === 0) {
      // Default (and wildcard): everything checked.
      setChecked(new Set(all));
    } else {
      setChecked(new Set(mine.map((r) => r.genderValue)));
    }
  }

  async function save() {
    if (!subject) return;
    const all = genders.map((g) => g.gender);
    const allChecked = all.length > 0 && all.every((g) => checked.has(g));
    setSaving(true);
    try {
      await api.znackApplyMapping(
        product.gtin,
        subject,
        allChecked ? [] : [...checked],
        allChecked
      );
      const fresh = await api.znackListRules();
      setLocalRules(fresh);
      await onChanged();
      notify(
        checked.size === 0
          ? `Đã gỡ liên kết "${subject}".`
          : `Đã liên kết "${subject}" với GTIN ${product.gtin}.`,
        "ok"
      );
    } catch (e) {
      notify(`Lưu mapping thất bại: ${errMsg(e)}`, "err");
    } finally {
      setSaving(false);
    }
  }

  function toggle(gender: string) {
    setChecked((prev) => {
      const next = new Set(prev);
      if (next.has(gender)) next.delete(gender);
      else next.add(gender);
      return next;
    });
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal znack-map-modal" onClick={(e) => e.stopPropagation()}>
        <div className="sheet-head">
          <h2>
            Mapping — <span className="mono">{product.gtin}</span>
          </h2>
          <button className="icon-btn" onClick={onClose}>
            <X size={18} />
          </button>
        </div>
        <p className="muted">
          Chọn danh mục, sau đó bỏ tích những giới tính không muốn gắn với GTIN này. Một
          GTIN có thể nối với nhiều danh mục.
        </p>

        <div className="znack-map-body">
          <div className="znack-map-subjects">
            {subjects.map(({ name, count }) => {
              const links = linksFor(name);
              const mine = links.filter((r) => r.gtin === product.gtin);
              const other = links.length - mine.length;
              return (
                <button
                  key={name}
                  className={`znack-subject ${subject === name ? "active" : ""}`}
                  onClick={() => pickSubject(name)}
                >
                  <span className="znack-subject-name">{name}</span>
                  <span className="znack-subject-meta muted">
                    {count} sp
                    {mine.length > 0 && <span className="chip">đã nối</span>}
                    {other > 0 && <span className="chip">GTIN khác</span>}
                  </span>
                  <ChevronRight size={15} className="sc-chev" />
                </button>
              );
            })}
            {subjects.length === 0 && (
              <p className="muted" style={{ padding: 10 }}>
                Chưa có sản phẩm WB nào — đồng bộ sản phẩm trước.
              </p>
            )}
          </div>

          <div className="znack-map-genders">
            {!subject && <p className="muted">Chọn một danh mục bên trái.</p>}
            {subject && genders.length === 0 && (
              <p className="muted">Danh mục này chưa có sản phẩm.</p>
            )}
            {subject &&
              genders.map((g) => {
                const owner = localRules.find(
                  (r) =>
                    r.subjectName === subject &&
                    !r.wildcardGender &&
                    r.genderValue === g.gender &&
                    r.gtin !== product.gtin
                );
                return (
                  <label key={g.gender || "__none__"} className="sort-check znack-gender">
                    <input
                      type="checkbox"
                      checked={checked.has(g.gender)}
                      onChange={() => toggle(g.gender)}
                    />
                    {g.gender || "(Chưa có giới tính)"}
                    <span className="muted">· {g.productCount} sp</span>
                    {owner && (
                      <span className="muted znack-owner">đang thuộc {owner.gtin}</span>
                    )}
                  </label>
                );
              })}
            {subject && (
              <div className="modal-actions">
                <button className="primary" onClick={save} disabled={saving}>
                  {saving ? "Đang lưu..." : "Lưu liên kết"}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
