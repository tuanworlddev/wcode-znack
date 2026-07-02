//! Serde models for the Wildberries API and for data exchanged with the frontend.

use serde::{Deserialize, Deserializer, Serialize};

/// Accepts either a JSON string or number and returns a String.
/// WB sometimes returns sticker `partA`/`partB` as numbers, sometimes as strings.
fn string_from_any<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(de)?;
    Ok(match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Content API — cards/list
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CardsListResponse {
    #[serde(default)]
    pub cards: Vec<Card>,
    pub cursor: CardsCursor,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CardsCursor {
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
    #[serde(rename = "nmID", default)]
    pub nm_id: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    #[serde(rename = "nmID")]
    pub nm_id: i64,
    #[serde(rename = "imtID", default)]
    pub imt_id: i64,
    #[serde(rename = "vendorCode", default)]
    pub vendor_code: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub title: String,
    #[serde(rename = "subjectName", default)]
    pub subject_name: String,
    #[serde(default)]
    pub characteristics: Vec<Characteristic>,
    #[serde(default)]
    pub sizes: Vec<CardSize>,
    #[serde(default)]
    pub photos: Vec<CardPhoto>,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CardPhoto {
    #[serde(default)]
    pub c246x328: String,
    #[serde(default)]
    pub big: String,
    #[serde(default)]
    pub square: String,
}

#[derive(Debug, Deserialize)]
pub struct Characteristic {
    #[serde(default)]
    pub name: String,
    // WB returns this as an array of strings for most characteristics, but as a
    // bare scalar (number/string) for some (e.g. "Ширина предмета": 12), so it
    // must accept any JSON value.
    #[serde(default)]
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CardSize {
    #[serde(rename = "chrtID")]
    pub chrt_id: i64,
    #[serde(rename = "techSize", default)]
    pub tech_size: String,
    #[serde(rename = "wbSize", default)]
    pub wb_size: String,
    #[serde(default)]
    pub skus: Vec<String>,
}

// ---------------------------------------------------------------------------
// Marketplace API — assembly orders
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct OrdersResponse {
    #[serde(default)]
    pub orders: Vec<WbOrder>,
}

/// GET /api/v3/orders (assembly orders list) — paginated with `next`.
#[derive(Debug, Deserialize)]
pub struct AssemblyOrdersResponse {
    #[serde(default)]
    pub next: i64,
    #[serde(default)]
    pub orders: Vec<WbOrder>,
}

#[derive(Debug, Deserialize)]
pub struct OrderIdsResponse {
    #[serde(rename = "orderIds", default)]
    pub order_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct WbOrder {
    pub id: i64,
    #[serde(default)]
    pub rid: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
    #[serde(rename = "nmId", default)]
    pub nm_id: i64,
    #[serde(rename = "chrtId", default)]
    pub chrt_id: i64,
    #[serde(default)]
    pub article: String,
    #[serde(rename = "colorCode", default)]
    pub color_code: String,
    #[serde(default)]
    pub skus: Vec<String>,
    #[serde(default)]
    pub price: i64,
    #[serde(rename = "cargoType", default)]
    pub cargo_type: i64,
    #[serde(default)]
    pub ddate: Option<String>,
    #[serde(rename = "supplyId", default)]
    pub supply_id: Option<String>,
    #[serde(rename = "warehouseId", default)]
    pub warehouse_id: i64,
    #[serde(default)]
    pub offices: Vec<String>,
    #[serde(rename = "deliveryType", default)]
    pub delivery_type: String,
    #[serde(rename = "isPickupPointShipmentAllowed", default)]
    pub is_pickup: bool,
    /// Meta fields WB requires for this order (e.g. ["sgtin"] for ЧЗ goods).
    #[serde(rename = "requiredMeta", default)]
    pub required_meta: Vec<String>,
}

/// Seller FBS warehouse (GET /api/v3/warehouses).
#[derive(Debug, Deserialize)]
pub struct Warehouse {
    pub id: i64,
    #[serde(default)]
    pub name: String,
}

// ---------------------------------------------------------------------------
// Marketplace API — stickers
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct StickersResponse {
    #[serde(default)]
    pub stickers: Vec<Sticker>,
}

#[derive(Debug, Deserialize)]
pub struct Sticker {
    #[serde(rename = "orderId")]
    pub order_id: i64,
    #[serde(rename = "partA", default, deserialize_with = "string_from_any")]
    pub part_a: String,
    #[serde(rename = "partB", default, deserialize_with = "string_from_any")]
    pub part_b: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub barcode: String,
    /// Base64-encoded sticker image in the requested format (png here).
    #[serde(default)]
    pub file: String,
}

// ---------------------------------------------------------------------------
// Marketplace API — supplies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SuppliesResponse {
    #[serde(default)]
    pub next: i64,
    #[serde(default)]
    pub supplies: Vec<WbSupply>,
}

#[derive(Debug, Deserialize)]
pub struct WbSupply {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub done: bool,
    #[serde(rename = "cargoType", default)]
    pub cargo_type: i64,
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
    #[serde(rename = "closedAt", default)]
    pub closed_at: Option<String>,
    #[serde(rename = "scanDt", default)]
    pub scan_dt: Option<String>,
    #[serde(rename = "isPickupPointShipmentAllowed", default)]
    pub is_pickup: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupplyResponse {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Frontend-facing rows (serialized as camelCase for the React app)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRow {
    pub nm_id: i64,
    pub vendor_code: String,
    pub title: String,
    pub brand: String,
    pub subject_name: String,
    pub color: String,
    pub sizes: String,
    pub photo: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRow {
    pub id: i64,
    pub rid: String,
    pub article: String,
    pub nm_id: i64,
    pub chrt_id: i64,
    pub sku: String,
    pub title: Option<String>,
    pub tech_size: Option<String>,
    pub color: Option<String>,
    pub brand: Option<String>,
    pub subject_name: Option<String>,
    pub gender: Option<String>,
    pub photo: Option<String>,
    pub status: String,
    pub supply_id: Option<String>,
    pub cargo_type: i64,
    pub price: i64,
    pub warehouse: String,
    pub offices: String,
    pub pickup: bool,
    pub created_at: String,
    /// WB requires a Честный ЗНАК code for this order (None = unknown).
    pub sgtin_required: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyRow {
    pub id: String,
    pub name: String,
    pub done: bool,
    pub cargo_type: i64,
    pub is_pickup: bool,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub order_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCounts {
    #[serde(rename = "new")]
    pub new_orders: i64,
    pub confirm: i64,
    pub complete: i64,
}

/// Everything the frontend needs to render one order's 2-page label.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintOrder {
    pub order_id: i64,
    /// Product barcode (SKU) encoded as Code128 on page 1.
    pub barcode: String,
    pub title: String,
    pub vendor_code: String,
    pub tech_size: String,
    pub color: String,
    pub brand: String,
    pub subject_name: String,
    pub photo: String,
    pub nm_id: i64,
    /// Base64 PNG of the WB waybill sticker (page 2).
    pub sticker_png: String,
    pub part_a: String,
    pub part_b: String,
    /// Честный ЗНАК: full KIZ code (with GS separators) reserved for this
    /// order, and its SGTIN display form. Empty when no code was attached.
    pub kiz_code: String,
    pub kiz_sgtin: String,
}

/// Print payload for a supply: per-order data plus the KIZ reservation token
/// (commit after the PDF is saved, release on failure).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyPrintData {
    pub orders: Vec<PrintOrder>,
    pub kiz_token: Option<String>,
    /// Orders that should have a KIZ (mapping exists) but got none.
    pub kiz_missing: i64,
}

/// One printed KIZ → WB assembly order assignment.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SgtinAssignment {
    pub order_id: i64,
    pub sgtin: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SgtinPushResult {
    pub ok: i64,
    pub failed: Vec<String>,
}

/// One product size (SKU) row for FBO barcode printing.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkuItem {
    pub nm_id: i64,
    pub barcode: String,
    pub tech_size: String,
    pub wb_size: String,
    pub title: String,
    pub brand: String,
    pub subject_name: String,
    pub gender: String,
    pub color: String,
    pub vendor_code: String,
    pub photo: String,
}

/// FBO print: one KIZ demand per item (category+gender resolves the GTIN).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FboReserveRequest {
    pub subject_name: String,
    pub gender: String,
    pub quantity: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FboReservation {
    pub token: Option<String>,
    /// Per-request list of reserved codes (empty when the item has no mapping).
    pub codes: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreInfo {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub has_token: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub count: i64,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deserialize a real cards/list response (path via CARDS_JSON env var)
    /// to catch field/type mismatches against the live API.
    #[test]
    fn parse_real_cards() {
        let Ok(path) = std::env::var("CARDS_JSON") else {
            return;
        };
        let s = std::fs::read_to_string(&path).expect("read CARDS_JSON");
        let r: CardsListResponse = serde_json::from_str(&s).expect("deserialize cards");
        assert!(!r.cards.is_empty());
        eprintln!("parsed {} cards OK", r.cards.len());
    }
}
