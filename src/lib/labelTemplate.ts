// Label template model for the 58x40mm product label (page 1).
// All geometry is stored in MILLIMETERS so the designer and the PDF renderer
// (12 px/mm canvas) map 1:1 onto the physical label — exact positioning.
// Font sizes are stored in canvas pixels at 12 px/mm (26px ≈ 2.2mm cap height),
// matching the legacy hard-coded layout values.

import type { PrintOrder } from "../types/wb";

export const LABEL_W_MM = 58;
export const LABEL_H_MM = 40;

export type LabelElementType =
  | "brand"
  | "title"
  | "category"
  | "article"
  | "color"
  | "size"
  | "barcode"
  | "barcodeText"
  | "datamatrix"
  | "sgtin"
  | "partB"
  | "pairNo"
  | "text"
  | "line";

export interface LabelElement {
  id: string;
  type: LabelElementType;
  x: number; // mm
  y: number; // mm
  w: number; // mm
  h: number; // mm
  fontSize: number; // px at 12 px/mm
  bold: boolean;
  italic: boolean;
  align: "left" | "center" | "right";
  /** Static prefix rendered before the value, e.g. "Màu: ". */
  prefix?: string;
  /** Fixed content — only for type "text". */
  text?: string;
}

export interface LabelTemplate {
  version: 1;
  elements: LabelElement[];
}

export const FIELD_DEFS: { type: LabelElementType; name: string; textual: boolean }[] = [
  { type: "brand", name: "Brand", textual: true },
  { type: "title", name: "Tên sản phẩm", textual: true },
  { type: "category", name: "Danh mục", textual: true },
  { type: "article", name: "Article", textual: true },
  { type: "color", name: "Màu", textual: true },
  { type: "size", name: "Size", textual: true },
  { type: "barcode", name: "Barcode (Code128)", textual: false },
  { type: "barcodeText", name: "Số barcode", textual: true },
  { type: "datamatrix", name: "DataMatrix ЧЗ", textual: false },
  { type: "sgtin", name: "SGTIN (chữ)", textual: true },
  { type: "partB", name: "Part B (sticker)", textual: true },
  { type: "pairNo", name: "Số cặp (FBO)", textual: true },
  { type: "text", name: "Văn bản cố định", textual: true },
  { type: "line", name: "Đường kẻ ngang", textual: false },
];

export function fieldName(type: LabelElementType): string {
  return FIELD_DEFS.find((f) => f.type === type)?.name ?? type;
}

/** Zero-pad sticker part B to 4 digits ("396" -> "0396"). */
export function padPartB(partB: string): string {
  const t = (partB || "").trim();
  return t && t.length < 4 ? t.padStart(4, "0") : t;
}

/** "gtin serial" display form of a raw KIZ code (mirror of the Rust
 *  sgtin_display in znack/models.rs). */
export function sgtinDisplay(rawCode: string): string {
  let code = (rawCode || "").trim();
  if (code.startsWith("]d2")) code = code.slice(3);
  code = code.replace(/^+/, "");
  const isUnit =
    code.length >= 18 &&
    code.startsWith("01") &&
    /^\d{14}$/.test(code.slice(2, 16)) &&
    code.slice(16, 18) === "21";
  if (!isUnit) return code;
  const gsPos = code.indexOf("", 18);
  // lp serials are 13 chars; fall back to that when no GS is present.
  const end = gsPos >= 18 ? gsPos : Math.min(31, code.length);
  const cis = code.slice(0, end);
  return `${cis.slice(2, 16)} ${cis.slice(18)}`;
}

/** Resolve the printable text of a textual element for one order. */
export function resolveText(el: LabelElement, o: PrintOrder): string {
  const value = (() => {
    switch (el.type) {
      case "brand":
        return o.brand;
      case "title":
        return o.title;
      case "category":
        return o.subjectName;
      case "article":
        return o.vendorCode;
      case "color":
        return o.color;
      case "size":
        return o.techSize;
      case "barcodeText":
        return o.barcode;
      case "sgtin":
        return o.kizSgtin;
      case "partB":
        return padPartB(o.partB);
      case "pairNo":
        return o.pairNo ? String(o.pairNo) : "";
      case "text":
        return el.text ?? "";
      default:
        return "";
    }
  })();
  if (!value) return "";
  return (el.prefix ?? "") + value;
}

let nextId = 1;
export function newElementId(): string {
  return `el-${Date.now().toString(36)}-${nextId++}`;
}

export function defaultElement(type: LabelElementType): LabelElement {
  const base: LabelElement = {
    id: newElementId(),
    type,
    x: 2,
    y: 2,
    w: 30,
    h: 4,
    fontSize: 22,
    bold: false,
    italic: false,
    align: "left",
  };
  switch (type) {
    case "brand":
      return { ...base, w: 38, h: 3, fontSize: 26, bold: true };
    case "title":
      return { ...base, w: 38, h: 5, fontSize: 24 };
    case "barcode":
      return { ...base, w: 34, h: 7 };
    case "barcodeText":
      return { ...base, w: 34, h: 2.6, align: "center" };
    case "datamatrix":
      return { ...base, w: 15, h: 15 };
    case "partB":
      return { ...base, w: 13, h: 5, fontSize: 42, bold: true, align: "right" };
    case "pairNo":
      return { ...base, w: 6.5, h: 3.5, fontSize: 34, bold: true, align: "right" };
    case "text":
      return { ...base, text: "Văn bản", w: 20 };
    case "line":
      return { ...base, w: 54, h: 0.3 };
    default:
      return base;
  }
}

/** FBS labels and FBO labels are designed independently. */
export type TemplateKind = "fbs" | "fbo";

/**
 * Default layouts = the user's saved designs (03.07.2026):
 *   - DataMatrix ЧЗ 19.5mm on the LEFT
 *   - brand (bold, centered) / category / article / color / size on the right
 *   - horizontal rule, then big Code128 (45×8.3) + its digits across the bottom
 *   - bottom-right corner: sticker Part B (FBS) or the pair number (FBO)
 */
export function defaultTemplate(kind: TemplateKind): LabelTemplate {
  const el = (partial: Partial<LabelElement> & { type: LabelElementType }): LabelElement => ({
    ...defaultElement(partial.type),
    ...partial,
    id: newElementId(),
  });
  const elements =
    kind === "fbs"
      ? [
          el({ type: "datamatrix", x: 2.1, y: 2.9, w: 19.5, h: 19.5 }),
          el({ type: "brand", x: 22.5, y: 2.3, w: 34.5, h: 3.7, fontSize: 32, bold: true, align: "center" }),
          el({ type: "category", x: 22.5, y: 6.3, w: 34.5, h: 3.7, fontSize: 32 }),
          el({ type: "article", x: 22.5, y: 10.7, w: 34.5, h: 3.7, fontSize: 32, prefix: "Арт: " }),
          el({ type: "color", x: 22.5, y: 15, w: 34.5, h: 3.7, fontSize: 32, prefix: "Цвет: " }),
          el({ type: "size", x: 22.5, y: 19.5, w: 34.5, h: 3.7, fontSize: 32, prefix: "Раз: " }),
          el({ type: "line", x: 1.4, y: 24.7, w: 54, h: 0.3 }),
          el({ type: "barcode", x: 7.1, y: 27, w: 45, h: 8.3 }),
          el({ type: "barcodeText", x: 9, y: 35.8, w: 40, h: 3.4, fontSize: 30, align: "center" }),
          el({ type: "partB", x: 47.3, y: 35.9, w: 9.4, h: 3.1, fontSize: 32, bold: true, align: "right" }),
        ]
      : [
          el({ type: "datamatrix", x: 2.1, y: 2.9, w: 19.5, h: 19.5 }),
          el({ type: "brand", x: 22.5, y: 2.3, w: 34.5, h: 3.7, fontSize: 32, bold: true, align: "center" }),
          el({ type: "category", x: 22.5, y: 6.3, w: 34.5, h: 3.7, fontSize: 32 }),
          el({ type: "article", x: 22.5, y: 10.7, w: 34.5, h: 3.7, fontSize: 32, prefix: "Art: " }),
          el({ type: "color", x: 22.5, y: 15, w: 34.5, h: 3.7, fontSize: 32, prefix: "Цвет: " }),
          el({ type: "size", x: 22.5, y: 19.5, w: 34.5, h: 3.7, fontSize: 32, prefix: "Раз: " }),
          el({ type: "line", x: 2, y: 25.2, w: 54, h: 0.3 }),
          el({ type: "barcode", x: 7.1, y: 27, w: 45, h: 8.3 }),
          el({ type: "barcodeText", x: 9, y: 35.8, w: 40, h: 3.4, fontSize: 30, align: "center" }),
          el({ type: "pairNo", x: 48.3, y: 35.9, w: 8.4, h: 3.1, fontSize: 36, bold: true, align: "right" }),
        ];
  return { version: 1, elements };
}

const STORAGE_KEYS: Record<TemplateKind, string> = {
  fbs: "wcode.labelTemplate.fbs.v1",
  fbo: "wcode.labelTemplate.fbo.v1",
};

export function loadTemplate(kind: TemplateKind): LabelTemplate {
  try {
    const raw = localStorage.getItem(STORAGE_KEYS[kind]);
    if (!raw) return defaultTemplate(kind);
    const parsed = JSON.parse(raw) as LabelTemplate;
    if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.elements)) {
      return defaultTemplate(kind);
    }
    return parsed;
  } catch {
    return defaultTemplate(kind);
  }
}

export function saveTemplate(kind: TemplateKind, t: LabelTemplate) {
  localStorage.setItem(STORAGE_KEYS[kind], JSON.stringify(t));
}

/** Sample order used for the designer preview. */
export const SAMPLE_ORDER: PrintOrder = {
  orderId: 5265465239,
  barcode: "4650001234567",
  title: "Спортивные брюки джоггеры для подростков",
  vendorCode: "брюки6/черный",
  techSize: "170",
  color: "черный",
  brand: "LYXURY",
  subjectName: "Брюки",
  gender: "Мальчики",
  photo: "",
  nmId: 123456789,
  stickerPng: "",
  partA: "5576657",
  partB: "7396",
  kizCode:
    "0104650001234567215EirD_orEif7X91EE1292VygELprJ87wrc/qF/Vq/KEFUVpZm4vDPQt2kqUw308Y=",
  kizSgtin: "04650001234567 5EirD_orEif7X",
  pairNo: 1,
} as PrintOrder;
