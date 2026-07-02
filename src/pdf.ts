// Builds the 2-page-per-order label PDF entirely in the browser:
//   page 1 = Code128(SKU) + product info  (rendered to a canvas -> PNG)
//   page 2 = WB waybill sticker PNG
// Rendering page 1 as an image avoids PDF font-embedding issues with Cyrillic.

import { PDFDocument } from "pdf-lib";
import * as bwipjs from "bwip-js/browser";
import type { PrintOrder } from "./types/wb";
import { api } from "./api/tauri";
import {
  loadTemplate,
  padPartB,
  resolveText,
  type LabelElement,
  type LabelTemplate,
} from "./lib/labelTemplate";

export { padPartB };

const PT_PER_MM = 2.834645669;
const LABEL_W_MM = 58;
const LABEL_H_MM = 40;
const PX_PER_MM = 12; // page-1 canvas resolution (~305 dpi)

function wrapText(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number,
  maxLines: number
): string[] {
  const words = text.split(/\s+/);
  const lines: string[] = [];
  let cur = "";
  for (const w of words) {
    const test = cur ? `${cur} ${w}` : w;
    if (ctx.measureText(test).width > maxWidth && cur) {
      lines.push(cur);
      cur = w;
      if (lines.length === maxLines - 1) break;
    } else {
      cur = test;
    }
  }
  if (cur && lines.length < maxLines) lines.push(cur);
  // Ellipsize overflow on the last line.
  if (lines.length === maxLines) {
    let last = lines[maxLines - 1];
    while (ctx.measureText(last + "…").width > maxWidth && last.length > 1) {
      last = last.slice(0, -1);
    }
    if (last !== lines[maxLines - 1]) lines[maxLines - 1] = last + "…";
  }
  return lines;
}

function truncate(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number
): string {
  if (ctx.measureText(text).width <= maxWidth) return text;
  let t = text;
  while (t.length > 1 && ctx.measureText(t + "…").width > maxWidth) {
    t = t.slice(0, -1);
  }
  return t + "…";
}

/// Render one label element onto the 12 px/mm canvas. Geometry comes from the
/// user-designed template (millimetres × PX_PER_MM = exact pixels).
function drawTemplateElement(
  ctx: CanvasRenderingContext2D,
  el: LabelElement,
  o: PrintOrder
) {
  const x = el.x * PX_PER_MM;
  const y = el.y * PX_PER_MM;
  const w = el.w * PX_PER_MM;
  const h = el.h * PX_PER_MM;

  if (el.type === "barcode") {
    if (!o.barcode) return;
    try {
      const bc = document.createElement("canvas");
      bwipjs.toCanvas(bc, {
        bcid: "code128",
        text: o.barcode,
        scale: 3,
        height: 7,
        includetext: false,
      });
      // WYSIWYG: fill the designed box exactly, like the designer preview.
      // Nearest-neighbour keeps the bar edges sharp when stretching.
      const prevSmooth = ctx.imageSmoothingEnabled;
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(bc, x, y, w, h);
      ctx.imageSmoothingEnabled = prevSmooth;
    } catch {
      /* ignore barcode failures */
    }
    return;
  }

  if (el.type === "datamatrix") {
    if (!o.kizCode) return;
    try {
      const dm = document.createElement("canvas");
      // GS1 DataMatrix: FNC1 escape first; the raw GS bytes (ASCII 29)
      // already inside the code pass through as data separators.
      bwipjs.toCanvas(dm, {
        bcid: "datamatrix",
        text: "^FNC1" + o.kizCode,
        parsefnc: true,
        scale: 4,
      });
      // Must stay square to scan; centred in the box (designer shows the same).
      const side = Math.min(w, h);
      const prevSmooth = ctx.imageSmoothingEnabled;
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(dm, x + (w - side) / 2, y + (h - side) / 2, side, side);
      ctx.imageSmoothingEnabled = prevSmooth;
    } catch {
      /* ignore DataMatrix failures — label still prints */
    }
    return;
  }

  if (el.type === "line") {
    // Horizontal rule — at least one device pixel so thin lines still print.
    ctx.fillRect(x, y, w, Math.max(h, 1));
    return;
  }

  // Textual element
  const text = resolveText(el, o);
  if (!text) return;
  const family =
    el.type === "barcodeText" || el.type === "sgtin"
      ? "'Courier New', monospace"
      : "Arial, 'Segoe UI', sans-serif";
  ctx.font = `${el.italic ? "italic " : ""}${el.bold ? "bold " : ""}${el.fontSize}px ${family}`;
  ctx.textBaseline = "top";
  ctx.textAlign = el.align;
  const anchorX = el.align === "center" ? x + w / 2 : el.align === "right" ? x + w : x;
  const lineH = Math.round(el.fontSize * 1.15);
  const maxLines = Math.max(1, Math.floor(h / lineH));
  const lines = wrapText(ctx, text, w, maxLines);
  let ty = y;
  for (const line of lines) {
    ctx.fillText(line, anchorX, ty);
    ty += lineH;
  }
  ctx.textAlign = "left";
}

function renderProductLabel(o: PrintOrder, template: LabelTemplate): string {
  const W = LABEL_W_MM * PX_PER_MM; // 696
  const H = LABEL_H_MM * PX_PER_MM; // 480
  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext("2d")!;
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, W, H);
  ctx.fillStyle = "#000000";
  ctx.textBaseline = "top";

  // The label layout is user-designed (Thiết kế tem); geometry in mm maps
  // exactly onto the 58×40 canvas.
  for (const el of template.elements) {
    drawTemplateElement(ctx, el, o);
  }
  return canvas.toDataURL("image/png");
}

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode.apply(
      null,
      Array.from(bytes.subarray(i, i + chunk))
    );
  }
  return btoa(binary);
}

/** Build a supply PDF (2 pages per order) and return it as base64. */
export async function buildSupplyPdfBase64(
  orders: PrintOrder[]
): Promise<{ base64: string; missingStickers: number }> {
  const pdf = await PDFDocument.create();
  const W = LABEL_W_MM * PT_PER_MM;
  const H = LABEL_H_MM * PT_PER_MM;
  let missingStickers = 0;
  const template = loadTemplate("fbs");

  for (const o of orders) {
    // Page 1 — product label
    const p1 = renderProductLabel(o, template);
    const img1 = await pdf.embedPng(p1);
    const pg1 = pdf.addPage([W, H]);
    pg1.drawImage(img1, { x: 0, y: 0, width: W, height: H });

    // Page 2 — waybill sticker
    if (o.stickerPng) {
      const img2 = await pdf.embedPng(o.stickerPng);
      const pg2 = pdf.addPage([W, H]);
      const s = Math.min(W / img2.width, H / img2.height);
      const w = img2.width * s;
      const h = img2.height * s;
      pg2.drawImage(img2, { x: (W - w) / 2, y: (H - h) / 2, width: w, height: h });
    } else {
      missingStickers += 1;
    }
  }

  const bytes = await pdf.save();
  return { base64: bytesToBase64(bytes), missingStickers };
}

/** Build the FBO barcode PDF: each unit prints as TWO identical 58×40 pages
 *  sharing one KIZ code, with the pair number rendered by the label template
 *  (element "Số cặp"). */
export async function buildFboPdfBase64(units: PrintOrder[]): Promise<string> {
  const pdf = await PDFDocument.create();
  const W = LABEL_W_MM * PT_PER_MM;
  const H = LABEL_H_MM * PT_PER_MM;
  const template = loadTemplate("fbo");
  for (const unit of units) {
    const png = renderProductLabel(unit, template);
    const img = await pdf.embedPng(png);
    for (let copy = 0; copy < 2; copy++) {
      const page = pdf.addPage([W, H]);
      page.drawImage(img, { x: 0, y: 0, width: W, height: H });
    }
  }
  const bytes = await pdf.save();
  return bytesToBase64(bytes);
}

// ---------------------------------------------------------------------------
// Picking list ("файл nhặt hàng") — an A4 sheet listing every order in the
// supply with its photo, so the picker can collect goods off the shelves.
// Pages are rendered to a canvas and embedded as PNG (Cyrillic-safe, like the
// labels above).
// ---------------------------------------------------------------------------

const A4_W = 595.28; // pt
const A4_H = 841.89;
const PX_PER_PT = 3; // canvas resolution (~216 dpi) — keeps small text crisp

/** Fetch a product photo through Rust (avoids CORS canvas tainting). */
async function loadPhoto(
  url: string,
  cache: Map<string, HTMLImageElement | null>
): Promise<HTMLImageElement | null> {
  if (!url) return null;
  const hit = cache.get(url);
  if (hit !== undefined) return hit;
  try {
    const dataUrl = await api.fetchImage(url);
    const img = await new Promise<HTMLImageElement>((resolve, reject) => {
      const im = new Image();
      im.onload = () => resolve(im);
      im.onerror = () => reject(new Error("bad image"));
      im.src = dataUrl;
    });
    cache.set(url, img);
    return img;
  } catch {
    cache.set(url, null);
    return null;
  }
}

/** Draw `img` into the cell rect with cover-crop (keeps the 3:4 frame full). */
function drawCover(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  x: number,
  y: number,
  w: number,
  h: number
) {
  const scale = Math.max(w / img.width, h / img.height);
  const sw = w / scale;
  const sh = h / scale;
  const sx = (img.width - sw) / 2;
  const sy = (img.height - sh) / 2;
  ctx.drawImage(img, sx, sy, sw, sh, x, y, w, h);
}

/** Wrap by characters (not words) — for values typed as one long run, like
 *  supplier articles, where we must break at the cell edge regardless of
 *  separators. */
function wrapHard(
  ctx: CanvasRenderingContext2D,
  text: string,
  maxWidth: number,
  maxLines: number
): string[] {
  const lines: string[] = [];
  let cur = "";
  for (const ch of text) {
    if (cur && ctx.measureText(cur + ch).width > maxWidth) {
      lines.push(cur);
      cur = ch;
    } else {
      cur += ch;
    }
  }
  if (cur) lines.push(cur);
  if (lines.length > maxLines) {
    const cut = lines.slice(0, maxLines);
    let last = cut[maxLines - 1];
    while (ctx.measureText(last + "…").width > maxWidth && last.length > 1) {
      last = last.slice(0, -1);
    }
    cut[maxLines - 1] = last + "…";
    return cut;
  }
  return lines;
}

type Col = { title: string; w: number };
// Widths in pt; must sum to A4_W - 2*MARGIN. Photo cell keeps a 3:4 image.
const COLS: Col[] = [
  { title: "Order ID", w: 58 },
  { title: "Photo", w: 42 },
  { title: "Brand", w: 62 },
  { title: "Name", w: 145 },
  { title: "Size", w: 35 },
  { title: "Color", w: 55 },
  { title: "Supplier article", w: 81 },
  { title: "Sticker", w: 77 },
];
const MARGIN = 20;
const ROW_H = 48;
const HEAD_ROW_H = 18;
const PHOTO_W = 30;
const PHOTO_H = 40; // 3:4
const CELL_FONT = 8;
const LINE_H = 10;
const BORDER = "#94a3b8";

export async function buildPickingPdfBase64(
  supplyId: string,
  orders: PrintOrder[]
): Promise<string> {
  const S = PX_PER_PT;
  const photoCache = new Map<string, HTMLImageElement | null>();
  // Prefetch photos sequentially (they're cached per URL, most repeat).
  for (const o of orders) await loadPhoto(o.photo, photoCache);

  const pdf = await PDFDocument.create();
  const colX: number[] = [];
  let acc = MARGIN;
  for (const c of COLS) {
    colX.push(acc);
    acc += c.w;
  }
  const tableRight = acc;

  const font = (size: number, bold = false) =>
    `${bold ? "bold " : ""}${size * S}px Arial, 'Segoe UI', sans-serif`;

  let idx = 0;
  let pageNo = 0;
  while (idx < orders.length) {
    pageNo += 1;
    const canvas = document.createElement("canvas");
    canvas.width = Math.round(A4_W * S);
    canvas.height = Math.round(A4_H * S);
    const ctx = canvas.getContext("2d")!;
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.textBaseline = "top";

    let y = MARGIN;
    if (pageNo === 1) {
      // Document header
      ctx.fillStyle = "#7c3aed"; // WCode mark, purple, top-right
      ctx.font = font(16, true);
      ctx.textAlign = "right";
      ctx.fillText("WCode", (A4_W - MARGIN) * S, y * S);
      ctx.textAlign = "left";
      ctx.fillStyle = "#000000";

      const d = new Date();
      const dateStr = `${String(d.getDate()).padStart(2, "0")}.${String(
        d.getMonth() + 1
      ).padStart(2, "0")}.${d.getFullYear()}`;
      ctx.font = font(11);
      ctx.fillText(`Date: ${dateStr}`, MARGIN * S, y * S);
      y += 20;
      ctx.font = font(14, true);
      ctx.fillText(`Selection sheet ${supplyId}`, MARGIN * S, y * S);
      y += 22;
      ctx.font = font(11);
      ctx.fillText(`Product amount: ${orders.length}`, MARGIN * S, y * S);
      y += 24;
    }

    // Table header row
    const tableTop = y;
    ctx.fillStyle = "#e2e8f0";
    ctx.fillRect(MARGIN * S, y * S, (tableRight - MARGIN) * S, HEAD_ROW_H * S);
    ctx.fillStyle = "#000000";
    ctx.font = font(CELL_FONT, true);
    COLS.forEach((c, i) => {
      ctx.fillText(
        truncate(ctx, c.title, (c.w - 8) * S),
        (colX[i] + 4) * S,
        (y + 5) * S
      );
    });
    y += HEAD_ROW_H;

    // Rows (zebra background on every other one)
    let rowNo = 0;
    while (idx < orders.length && y + ROW_H <= A4_H - MARGIN) {
      const o = orders[idx];
      const top = y;
      const pad = 4;

      if (rowNo % 2 === 1) {
        ctx.fillStyle = "#f8fafc";
        ctx.fillRect(MARGIN * S, top * S, (tableRight - MARGIN) * S, ROW_H * S);
      }

      ctx.fillStyle = "#000000";
      // Text cells: wrap to the cell width and center the block vertically.
      const cellText = (col: number, text: string, maxLines: number, hard = false) => {
        ctx.font = font(CELL_FONT);
        const wrap = hard ? wrapHard : wrapText;
        const lines = wrap(ctx, text || "—", (COLS[col].w - pad * 2) * S, maxLines);
        let ty = top + (ROW_H - lines.length * LINE_H) / 2 + 1;
        for (const line of lines) {
          ctx.fillText(line, (colX[col] + pad) * S, ty * S);
          ty += LINE_H;
        }
      };
      cellText(0, String(o.orderId), 2);
      cellText(2, o.brand, 4);
      cellText(3, o.title, 4);
      cellText(4, o.techSize, 2);
      cellText(5, o.color, 4);
      // Articles are often one long unbroken string — break at the cell edge.
      cellText(6, o.vendorCode, 4, true);

      // Photo (3:4), centered in its cell
      const img = photoCache.get(o.photo) ?? null;
      const px = colX[1] + (COLS[1].w - PHOTO_W) / 2;
      const py = top + (ROW_H - PHOTO_H) / 2;
      if (img) {
        drawCover(ctx, img, px * S, py * S, PHOTO_W * S, PHOTO_H * S);
      } else {
        ctx.strokeStyle = "#cbd5e1";
        ctx.strokeRect(px * S, py * S, PHOTO_W * S, PHOTO_H * S);
      }

      // Sticker: "partA partB", both 9pt, part B bold. One line if it fits,
      // otherwise part B drops to a second line. Block centered vertically.
      const stkSize = 9;
      const stkLineH = 11;
      const partA = o.partA || "";
      const partB = padPartB(o.partB);
      const maxW = (COLS[7].w - pad * 2) * S;
      ctx.font = font(stkSize);
      const aW = ctx.measureText(partA + " ").width;
      ctx.font = font(stkSize, true);
      const oneLine = partA && aW + ctx.measureText(partB).width <= maxW;
      const nLines = !partA || oneLine ? 1 : 2;
      const sy = top + (ROW_H - nLines * stkLineH) / 2 + 1;
      const sx = (colX[7] + pad) * S;
      ctx.font = font(stkSize);
      if (partA) ctx.fillText(partA + (oneLine ? " " : ""), sx, sy * S);
      ctx.font = font(stkSize, true);
      ctx.fillText(
        partB,
        oneLine ? sx + aW : sx,
        (oneLine || !partA ? sy : sy + stkLineH) * S
      );

      y += ROW_H;
      idx += 1;
      rowNo += 1;
    }

    // Table grid: horizontal row lines, vertical column lines, outer frame.
    ctx.strokeStyle = BORDER;
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let ly = tableTop + HEAD_ROW_H; ly < y; ly += ROW_H) {
      ctx.moveTo(MARGIN * S, ly * S);
      ctx.lineTo(tableRight * S, ly * S);
    }
    for (let i = 1; i < COLS.length; i++) {
      ctx.moveTo(colX[i] * S, tableTop * S);
      ctx.lineTo(colX[i] * S, y * S);
    }
    ctx.stroke();
    ctx.strokeRect(
      MARGIN * S,
      tableTop * S,
      (tableRight - MARGIN) * S,
      (y - tableTop) * S
    );

    const png = canvas.toDataURL("image/png");
    const embedded = await pdf.embedPng(png);
    const page = pdf.addPage([A4_W, A4_H]);
    page.drawImage(embedded, { x: 0, y: 0, width: A4_W, height: A4_H });
  }

  const bytes = await pdf.save();
  return bytesToBase64(bytes);
}
