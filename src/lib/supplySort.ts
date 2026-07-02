// Sort orders inside a supply by category / article / color / size.
// The enabled keys are persisted so the picking order stays consistent
// between the detail view and the printed files.

import type { OrderRow, PrintOrder } from "../types/wb";

export type SortKey = "category" | "article" | "color" | "size";

export const SORT_KEYS: { key: SortKey; label: string }[] = [
  { key: "category", label: "Danh mục" },
  { key: "article", label: "Article" },
  { key: "color", label: "Màu" },
  { key: "size", label: "Size" },
];

export type SortPrefs = Record<SortKey, boolean>;

const STORAGE_KEY = "wcode.supplySortKeys";
const DEFAULT_PREFS: SortPrefs = {
  category: true,
  article: true,
  color: true,
  size: true,
};

export function loadSortPrefs(): SortPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_PREFS };
    return { ...DEFAULT_PREFS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_PREFS };
  }
}

export function saveSortPrefs(p: SortPrefs) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(p));
  } catch {
    /* ignore */
  }
}

type SortFields = Record<SortKey, string>;

function fieldsOfOrderRow(o: OrderRow): SortFields {
  return {
    category: o.subjectName ?? "",
    article: o.article ?? "",
    color: o.color ?? "",
    size: o.techSize ?? "",
  };
}

function fieldsOfPrintOrder(o: PrintOrder): SortFields {
  return {
    category: o.subjectName,
    article: o.vendorCode,
    color: o.color,
    size: o.techSize,
  };
}

function compareBy(prefs: SortPrefs, a: SortFields, b: SortFields): number {
  for (const { key } of SORT_KEYS) {
    if (!prefs[key]) continue;
    const c = a[key].localeCompare(b[key], undefined, {
      numeric: true,
      sensitivity: "base",
    });
    if (c !== 0) return c;
  }
  return 0;
}

export function sortOrderRows(rows: OrderRow[], prefs: SortPrefs): OrderRow[] {
  return [...rows].sort((a, b) =>
    compareBy(prefs, fieldsOfOrderRow(a), fieldsOfOrderRow(b))
  );
}

export function sortPrintOrders(rows: PrintOrder[], prefs: SortPrefs): PrintOrder[] {
  return [...rows].sort((a, b) =>
    compareBy(prefs, fieldsOfPrintOrder(a), fieldsOfPrintOrder(b))
  );
}
