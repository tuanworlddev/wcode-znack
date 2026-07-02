// Mirrors the serde structs returned by the Rust commands (camelCase).

export interface StoreInfo {
  id: string;
  name: string;
  active: boolean;
  hasToken: boolean;
}

export interface SyncResult {
  count: number;
  message: string;
}

export interface ProductRow {
  nmId: number;
  vendorCode: string;
  title: string;
  brand: string;
  subjectName: string;
  color: string;
  sizes: string;
  photo: string;
  updatedAt: string | null;
}

export interface OrderRow {
  id: number;
  rid: string;
  article: string;
  nmId: number;
  chrtId: number;
  sku: string;
  title: string | null;
  techSize: string | null;
  color: string | null;
  brand: string | null;
  subjectName: string | null;
  gender: string | null;
  photo: string | null;
  status: string;
  supplyId: string | null;
  cargoType: number;
  price: number;
  warehouse: string;
  offices: string;
  pickup: boolean;
  createdAt: string;
  sgtinRequired: boolean | null; // WB requires Честный ЗНАК (null = unknown)
}

export interface OrderCounts {
  new: number;
  confirm: number;
  complete: number;
}

export interface SupplyRow {
  id: string;
  name: string;
  done: boolean;
  cargoType: number;
  isPickup: boolean;
  createdAt: string;
  closedAt: string | null;
  orderCount: number;
}

export interface PrintOrder {
  orderId: number;
  barcode: string;
  title: string;
  vendorCode: string;
  techSize: string;
  color: string;
  brand: string;
  subjectName: string;
  photo: string;
  nmId: number;
  stickerPng: string; // base64 PNG (no data: prefix)
  partA: string;
  partB: string;
  kizCode: string; // full Честный ЗНАК code with GS separators ("" = none)
  kizSgtin: string; // display form "gtin serial"
  /** FBO printing only: pair number shown on both copies of a unit. */
  pairNo?: number;
}

/** One product size (SKU) row for FBO barcode printing. */
export interface SkuItem {
  nmId: number;
  barcode: string;
  techSize: string;
  wbSize: string;
  title: string;
  brand: string;
  subjectName: string;
  gender: string;
  color: string;
  vendorCode: string;
  photo: string;
}

export interface SupplyPrintData {
  orders: PrintOrder[];
  kizToken: string | null;
  kizMissing: number;
}
