import { invoke } from "@tauri-apps/api/core";
import type {
  OrderCounts,
  OrderRow,
  ProductRow,
  SkuItem,
  StoreInfo,
  SupplyPrintData,
  SupplyRow,
  SyncResult,
} from "../types/wb";
import type {
  CategoryGender,
  ZnackCertificate,
  ZnackMappingRule,
  ZnackPipeline,
  ZnackProduct,
  ZnackSettings,
} from "../types/znack";

export const api = {
  // stores
  listStores: () => invoke<StoreInfo[]>("list_stores"),
  activeStore: () => invoke<StoreInfo | null>("active_store"),
  addStore: (name: string) => invoke<string>("add_store", { name }),
  switchStore: (id: string) => invoke<void>("switch_store", { id }),
  removeStore: (id: string) => invoke<void>("remove_store", { id }),
  renameStore: (id: string, name: string) =>
    invoke<void>("rename_store", { id, name }),

  // token
  setToken: (token: string) => invoke<void>("set_token", { token }),
  setStoreToken: (id: string, token: string) =>
    invoke<void>("set_store_token", { id, token }),
  getActiveToken: () => invoke<string | null>("get_active_token"),
  deleteToken: () => invoke<void>("delete_token"),

  syncProducts: () => invoke<SyncResult>("sync_products"),
  syncOrders: () => invoke<SyncResult>("sync_orders"),
  syncSupplies: () => invoke<SyncResult>("sync_supplies"),

  listProducts: (search = "", categories: string[] = [], limit = 1000) =>
    invoke<ProductRow[]>("list_products", { search, categories, limit }),
  listCategories: () => invoke<string[]>("list_categories"),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  listOrders: (status = "", supplyId: string | null = null) =>
    invoke<OrderRow[]>("list_orders", { status, supplyId }),
  orderStatusCounts: () => invoke<OrderCounts>("order_status_counts"),
  listSupplies: () => invoke<SupplyRow[]>("list_supplies"),

  createSupply: (name: string) => invoke<string>("create_supply", { name }),
  addOrdersToSupply: (supplyId: string, orderIds: number[]) =>
    invoke<void>("add_orders_to_supply", { supplyId, orderIds }),

  getSupplyOrders: (supplyId: string) =>
    invoke<OrderRow[]>("get_supply_orders", { supplyId }),
  getSupplyPrintData: (supplyId: string) =>
    invoke<SupplyPrintData>("get_supply_print_data", { supplyId }),
  saveAndOpenPdf: (supplyId: string, pdfBase64: string, suffix?: string) =>
    invoke<string>("save_and_open_pdf", { supplyId, pdfBase64, suffix: suffix ?? null }),
  fetchImage: (url: string) => invoke<string>("fetch_image", { url }),
  finishKizReservation: (token: string, consume: boolean) =>
    invoke<number>("finish_kiz_reservation", { token, consume }),
  assignOrderSgtins: (assignments: { orderId: number; sgtin: string }[]) =>
    invoke<{ ok: number; failed: string[] }>("assign_order_sgtins", { assignments }),
  listSkuItems: (search: string, categories: string[]) =>
    invoke<SkuItem[]>("list_sku_items", { search, categories }),
  reserveFboCodes: (
    requests: { subjectName: string; gender: string; quantity: number }[]
  ) =>
    invoke<{ token: string | null; codes: string[][] }>("reserve_fbo_codes", { requests }),

  // Честный ЗНАК
  znackGetSettings: () => invoke<ZnackSettings>("znack_get_settings"),
  znackSaveSettings: (settings: ZnackSettings) =>
    invoke<void>("znack_save_settings", { settings }),
  znackTestSign: () => invoke<string>("znack_test_sign"),
  znackListCertificates: () => invoke<ZnackCertificate[]>("znack_list_certificates"),
  znackCategoryGenders: () => invoke<CategoryGender[]>("znack_category_genders"),
  znackApplyMapping: (
    gtin: string,
    subjectName: string,
    genders: string[],
    allGenders: boolean
  ) => invoke<void>("znack_apply_mapping", { gtin, subjectName, genders, allGenders }),
  znackSyncProducts: () => invoke<string>("znack_sync_products"),
  znackListProducts: () => invoke<ZnackProduct[]>("znack_list_products"),
  znackListRules: () => invoke<ZnackMappingRule[]>("znack_list_rules"),
  znackSaveRule: (
    gtin: string,
    subjectName: string,
    genderValue: string,
    wildcardGender: boolean
  ) => invoke<void>("znack_save_rule", { gtin, subjectName, genderValue, wildcardGender }),
  znackDeleteRule: (id: number) => invoke<void>("znack_delete_rule", { id }),
  znackBuyKiz: (gtin: string, quantity: number) =>
    invoke<number>("znack_buy_kiz", { gtin, quantity }),
  znackListPipelines: () => invoke<ZnackPipeline[]>("znack_list_pipelines"),
  znackAbortPipeline: (id: number) => invoke<void>("znack_abort_pipeline", { id }),
  znackRetryPipeline: (id: number) => invoke<number>("znack_retry_pipeline", { id }),
};

export function errMsg(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}
