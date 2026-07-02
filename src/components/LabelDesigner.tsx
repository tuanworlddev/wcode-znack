import { useEffect, useMemo, useRef, useState } from "react";
import * as bwipjs from "bwip-js/browser";
import {
  Bold,
  Italic,
  AlignLeft,
  AlignCenter,
  AlignRight,
  Trash2,
  Plus,
  ZoomIn,
  ZoomOut,
  Save,
  RotateCcw,
} from "lucide-react";
import {
  FIELD_DEFS,
  LABEL_H_MM,
  LABEL_W_MM,
  SAMPLE_ORDER,
  defaultElement,
  defaultTemplate,
  fieldName,
  loadTemplate,
  newElementId,
  resolveText,
  saveTemplate,
  type LabelElement,
  type LabelTemplate,
  type TemplateKind,
} from "../lib/labelTemplate";
import type { Notify } from "../App";

type DragMode =
  | "move"
  | "n" | "s" | "e" | "w"
  | "nw" | "ne" | "sw" | "se";

const MIN_SIZE_MM = 2;

function clamp(v: number, lo: number, hi: number) {
  return Math.min(Math.max(v, lo), hi);
}
function round1(v: number) {
  return Math.round(v * 10) / 10;
}

/** Number input that stays editable: holds raw text while focused (so it can
 *  be cleared / retyped freely) and only commits valid values; on blur the
 *  final value is clamped and the text resyncs. */
function NumInput({
  value,
  min,
  max,
  step,
  onCommit,
}: {
  value: number;
  min: number;
  max: number;
  step?: number;
  onCommit: (v: number) => void;
}) {
  const [text, setText] = useState(String(value));
  const [focused, setFocused] = useState(false);
  useEffect(() => {
    if (!focused) setText(String(value));
  }, [value, focused]);
  return (
    <input
      type="number"
      step={step ?? 1}
      min={min}
      max={max}
      value={text}
      onFocus={() => setFocused(true)}
      onChange={(e) => {
        setText(e.target.value);
        const v = parseFloat(e.target.value);
        if (!Number.isNaN(v)) onCommit(clamp(v, min, max));
      }}
      onBlur={() => {
        setFocused(false);
        const v = parseFloat(text);
        onCommit(clamp(Number.isNaN(v) ? value : v, min, max));
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
      }}
    />
  );
}

export default function LabelDesigner({ notify }: { notify: Notify }) {
  const [kind, setKind] = useState<TemplateKind>("fbs");
  const [templates, setTemplates] = useState<Record<TemplateKind, LabelTemplate>>(() => ({
    fbs: loadTemplate("fbs"),
    fbo: loadTemplate("fbo"),
  }));
  const template = templates[kind];
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [scale, setScale] = useState(6); // px per mm on screen (6 = 50%)
  const [dirtyMap, setDirtyMap] = useState<Record<TemplateKind, boolean>>({
    fbs: false,
    fbo: false,
  });
  const dirty = dirtyMap[kind];
  const kindRef = useRef(kind);
  useEffect(() => {
    kindRef.current = kind;
  }, [kind]);

  function setTemplate(update: (t: LabelTemplate) => LabelTemplate) {
    const k = kindRef.current;
    setTemplates((all) => ({ ...all, [k]: update(all[k]) }));
  }
  function setDirty(v: boolean) {
    const k = kindRef.current;
    setDirtyMap((m) => ({ ...m, [k]: v }));
  }
  const canvasRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{
    mode: DragMode;
    id: string;
    startX: number;
    startY: number;
    ex: number;
    ey: number;
    ew: number;
    eh: number;
  } | null>(null);

  const selected = template.elements.find((e) => e.id === selectedId) ?? null;

  function mutate(id: string, patch: Partial<LabelElement>) {
    setTemplate((t) => ({
      ...t,
      elements: t.elements.map((e) => (e.id === id ? { ...e, ...patch } : e)),
    }));
    setDirty(true);
  }

  // --- drag / resize ---------------------------------------------------------

  function startDrag(e: React.PointerEvent, el: LabelElement, mode: DragMode) {
    e.preventDefault();
    e.stopPropagation();
    setSelectedId(el.id);
    dragRef.current = {
      mode,
      id: el.id,
      startX: e.clientX,
      startY: e.clientY,
      ex: el.x,
      ey: el.y,
      ew: el.w,
      eh: el.h,
    };
    window.addEventListener("pointermove", onDragMove);
    window.addEventListener("pointerup", endDrag);
  }

  function onDragMove(e: PointerEvent) {
    const d = dragRef.current;
    if (!d) return;
    const dx = (e.clientX - d.startX) / scaleRef.current;
    const dy = (e.clientY - d.startY) / scaleRef.current;
    let { ex: x, ey: y, ew: w, eh: h } = d;
    const m = d.mode;
    if (m === "move") {
      x = d.ex + dx;
      y = d.ey + dy;
    } else {
      if (m.includes("e")) w = d.ew + dx;
      if (m.includes("s")) h = d.eh + dy;
      if (m.includes("w")) {
        w = d.ew - dx;
        x = d.ex + dx;
      }
      if (m.includes("n")) {
        h = d.eh - dy;
        y = d.ey + dy;
      }
    }
    w = clamp(w, MIN_SIZE_MM, LABEL_W_MM);
    h = clamp(h, MIN_SIZE_MM, LABEL_H_MM);
    x = clamp(x, 0, LABEL_W_MM - w);
    y = clamp(y, 0, LABEL_H_MM - h);
    mutate(d.id, { x: round1(x), y: round1(y), w: round1(w), h: round1(h) });
  }

  function endDrag() {
    dragRef.current = null;
    window.removeEventListener("pointermove", onDragMove);
    window.removeEventListener("pointerup", endDrag);
  }

  // scale inside window listeners needs a ref
  const scaleRef = useRef(scale);
  useEffect(() => {
    scaleRef.current = scale;
  }, [scale]);

  // Keyboard nudge for the selected element (0.1mm; Shift = 1mm).
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (!selectedId) return;
      const target = e.target as HTMLElement;
      if (target && ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)) return;
      const step = e.shiftKey ? 1 : 0.1;
      const el = template.elements.find((x) => x.id === selectedId);
      if (!el) return;
      let handled = true;
      if (e.key === "ArrowLeft") mutate(el.id, { x: round1(clamp(el.x - step, 0, LABEL_W_MM - el.w)) });
      else if (e.key === "ArrowRight") mutate(el.id, { x: round1(clamp(el.x + step, 0, LABEL_W_MM - el.w)) });
      else if (e.key === "ArrowUp") mutate(el.id, { y: round1(clamp(el.y - step, 0, LABEL_H_MM - el.h)) });
      else if (e.key === "ArrowDown") mutate(el.id, { y: round1(clamp(el.y + step, 0, LABEL_H_MM - el.h)) });
      else if (e.key === "Delete" || e.key === "Backspace") removeElement(el.id);
      else handled = false;
      if (handled) e.preventDefault();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedId, template]);

  // --- element management ------------------------------------------------------

  function addElement(type: LabelElement["type"]) {
    if (type !== "text" && template.elements.some((e) => e.type === type)) {
      notify("Phần tử này đã có trên tem.", "info");
      return;
    }
    const el = { ...defaultElement(type), id: newElementId() };
    setTemplate((t) => ({ ...t, elements: [...t.elements, el] }));
    setSelectedId(el.id);
    setDirty(true);
  }

  function removeElement(id: string) {
    setTemplate((t) => ({ ...t, elements: t.elements.filter((e) => e.id !== id) }));
    if (selectedId === id) setSelectedId(null);
    setDirty(true);
  }

  function save() {
    saveTemplate(kind, template);
    setDirty(false);
    notify(
      `Đã lưu mẫu tem ${kind.toUpperCase()}. Các lần in ${kind.toUpperCase()} tiếp theo sẽ dùng bố cục này.`,
      "ok"
    );
  }

  function reset() {
    const t = defaultTemplate(kind);
    setTemplate(() => t);
    setSelectedId(null);
    setDirty(true);
  }

  // --- render -------------------------------------------------------------------

  const usedTypes = useMemo(
    () => new Set(template.elements.map((e) => e.type)),
    [template.elements]
  );

  return (
    <div className="page designer-page">
      <div className="page-head">
        <h1>Thiết kế tem 58×40</h1>
        <div className="designer-actions">
          <div className="designer-kind">
            {(["fbs", "fbo"] as TemplateKind[]).map((k) => (
              <button
                key={k}
                className={`tab ${kind === k ? "active" : ""}`}
                onClick={() => {
                  setKind(k);
                  setSelectedId(null);
                }}
              >
                Tem {k.toUpperCase()}
                {dirtyMap[k] && " •"}
              </button>
            ))}
          </div>
          <button className="ghost" onClick={reset}>
            <RotateCcw size={15} /> Mặc định
          </button>
          <button className="primary" onClick={save} disabled={!dirty}>
            <Save size={16} /> Lưu tem {kind.toUpperCase()}
          </button>
        </div>
      </div>

      <div className="designer-body">
        {/* Palette */}
        <div className="designer-palette card">
          <h3>Thành phần</h3>
          {FIELD_DEFS.map((f) => (
            <button
              key={f.type}
              className="designer-add"
              disabled={f.type !== "text" && usedTypes.has(f.type)}
              onClick={() => addElement(f.type)}
            >
              <Plus size={13} /> {f.name}
            </button>
          ))}
          <p className="muted designer-hint">
            Kéo để di chuyển, kéo cạnh/góc để đổi kích thước. Phím mũi tên = 0.1mm
            (Shift = 1mm).
          </p>
        </div>

        {/* Canvas */}
        <div className="designer-center">
          <div className="designer-toolbar">
            <button className="icon-btn" onClick={() => setScale((s) => clamp(s - 2, 6, 24))}>
              <ZoomOut size={16} />
            </button>
            <span className="muted">{Math.round((scale / 12) * 100)}%</span>
            <button className="icon-btn" onClick={() => setScale((s) => clamp(s + 2, 6, 24))}>
              <ZoomIn size={16} />
            </button>
            <span className="muted designer-size">58 × 40 mm</span>
          </div>
          <div className="designer-canvas-wrap">
            <div
              ref={canvasRef}
              className="designer-canvas"
              style={{ width: LABEL_W_MM * scale, height: LABEL_H_MM * scale }}
              onPointerDown={() => setSelectedId(null)}
            >
              {/* mm grid every 5mm */}
              <div
                className="designer-grid"
                style={{ backgroundSize: `${5 * scale}px ${5 * scale}px` }}
              />
              {template.elements.map((el) => (
                <ElementBox
                  key={el.id}
                  el={el}
                  scale={scale}
                  selected={el.id === selectedId}
                  onStartDrag={startDrag}
                />
              ))}
            </div>
          </div>
        </div>

        {/* Properties */}
        <div className="designer-props card">
          <h3>Thuộc tính</h3>
          {!selected && <p className="muted">Chọn một phần tử trên tem.</p>}
          {selected && (
            <>
              <div className="prop-title">
                <b>{fieldName(selected.type)}</b>
                <button
                  className="icon-btn"
                  title="Xoá phần tử"
                  onClick={() => removeElement(selected.id)}
                >
                  <Trash2 size={15} />
                </button>
              </div>

              {selected.type === "text" && (
                <label className="prop-field">
                  <span>Nội dung</span>
                  <input
                    value={selected.text ?? ""}
                    onChange={(e) => mutate(selected.id, { text: e.target.value })}
                  />
                </label>
              )}
              {selected.type !== "text" &&
                selected.type !== "barcode" &&
                selected.type !== "datamatrix" &&
                selected.type !== "line" && (
                  <label className="prop-field">
                    <span>Tiền tố (vd "Màu: ")</span>
                    <input
                      value={selected.prefix ?? ""}
                      onChange={(e) => mutate(selected.id, { prefix: e.target.value })}
                    />
                  </label>
                )}

              <div className="prop-grid">
                {(
                  [
                    ["X (mm)", "x", LABEL_W_MM - selected.w],
                    ["Y (mm)", "y", LABEL_H_MM - selected.h],
                    ["Rộng", "w", LABEL_W_MM],
                    ["Cao", "h", LABEL_H_MM],
                  ] as const
                ).map(([label, key, max]) => (
                  <label key={key} className="prop-field">
                    <span>{label}</span>
                    <NumInput
                      step={0.1}
                      min={0}
                      max={max}
                      value={selected[key]}
                      onCommit={(v) =>
                        mutate(selected.id, { [key]: round1(v) } as Partial<LabelElement>)
                      }
                    />
                  </label>
                ))}
              </div>

              {selected.type !== "barcode" &&
                selected.type !== "datamatrix" &&
                selected.type !== "line" && (
                <>
                  <label className="prop-field">
                    <span>Cỡ chữ (px @12px/mm ≈ {round1(selected.fontSize / 12)}mm)</span>
                    <NumInput
                      min={6}
                      max={200}
                      value={selected.fontSize}
                      onCommit={(v) => mutate(selected.id, { fontSize: Math.round(v) })}
                    />
                  </label>
                  <div className="prop-toggles">
                    <button
                      className={`icon-btn ${selected.bold ? "on" : ""}`}
                      title="In đậm"
                      onClick={() => mutate(selected.id, { bold: !selected.bold })}
                    >
                      <Bold size={15} />
                    </button>
                    <button
                      className={`icon-btn ${selected.italic ? "on" : ""}`}
                      title="In nghiêng"
                      onClick={() => mutate(selected.id, { italic: !selected.italic })}
                    >
                      <Italic size={15} />
                    </button>
                    <span className="prop-sep" />
                    {(
                      [
                        ["left", AlignLeft],
                        ["center", AlignCenter],
                        ["right", AlignRight],
                      ] as const
                    ).map(([align, Icon]) => (
                      <button
                        key={align}
                        className={`icon-btn ${selected.align === align ? "on" : ""}`}
                        onClick={() => mutate(selected.id, { align })}
                      >
                        <Icon size={15} />
                      </button>
                    ))}
                  </div>
                </>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

/** One element on the design canvas, with drag + resize handles. */
function ElementBox({
  el,
  scale,
  selected,
  onStartDrag,
}: {
  el: LabelElement;
  scale: number;
  selected: boolean;
  onStartDrag: (e: React.PointerEvent, el: LabelElement, mode: DragMode) => void;
}) {
  const style: React.CSSProperties = {
    left: el.x * scale,
    top: el.y * scale,
    width: el.w * scale,
    height: el.h * scale,
  };
  const handles: DragMode[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
  return (
    <div
      className={`designer-el ${selected ? "sel" : ""}`}
      style={style}
      onPointerDown={(e) => onStartDrag(e, el, "move")}
    >
      <ElementPreview el={el} scale={scale} />
      {selected &&
        handles.map((h) => (
          <div
            key={h}
            className={`designer-handle h-${h}`}
            onPointerDown={(e) => onStartDrag(e, el, h)}
          />
        ))}
    </div>
  );
}

/** WYSIWYG preview of an element using the sample order. */
function ElementPreview({ el, scale }: { el: LabelElement; scale: number }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (el.type !== "barcode" && el.type !== "datamatrix") return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    try {
      const tmp = document.createElement("canvas");
      if (el.type === "barcode") {
        bwipjs.toCanvas(tmp, {
          bcid: "code128",
          text: SAMPLE_ORDER.barcode,
          scale: 2,
          height: 8,
          includetext: false,
        });
      } else {
        bwipjs.toCanvas(tmp, {
          bcid: "datamatrix",
          text: "^FNC1" + SAMPLE_ORDER.kizCode,
          parsefnc: true,
          scale: 3,
        });
      }
      canvas.width = el.w * scale;
      canvas.height = el.h * scale;
      const ctx = canvas.getContext("2d")!;
      ctx.imageSmoothingEnabled = false;
      if (el.type === "datamatrix") {
        // Square + centred, exactly how the PDF renderer prints it.
        const side = Math.min(canvas.width, canvas.height);
        ctx.drawImage(tmp, (canvas.width - side) / 2, (canvas.height - side) / 2, side, side);
      } else {
        ctx.drawImage(tmp, 0, 0, canvas.width, canvas.height);
      }
    } catch {
      /* preview only */
    }
  }, [el.type, el.w, el.h, scale]);

  if (el.type === "barcode" || el.type === "datamatrix") {
    return <canvas ref={canvasRef} className="designer-el-canvas" />;
  }
  if (el.type === "line") {
    return <div className="designer-el-line" />;
  }
  const text = resolveText(el, SAMPLE_ORDER) || fieldName(el.type);
  // fontSize is px at 12px/mm -> on-screen px at current scale:
  const fontPx = (el.fontSize / 12) * scale;
  return (
    <div
      className="designer-el-text"
      style={{
        fontSize: fontPx,
        fontWeight: el.bold ? 700 : 400,
        fontStyle: el.italic ? "italic" : "normal",
        textAlign: el.align,
        lineHeight: 1.15,
      }}
    >
      {text}
    </div>
  );
}
