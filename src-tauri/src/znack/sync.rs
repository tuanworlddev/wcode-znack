//! GTIN sync: True API /product/gtin list + National Catalog feed-product
//! enrichment. Port of docs/znack_api/znack/ZnackProductService.java.

use crate::error::AppResult;
use crate::state::AppState;
use crate::znack::models::{normalize_gtin, ZnackSettings};
use crate::znack::{auth, client, db as zdb};
use serde_json::Value;
use std::collections::BTreeMap;

const PAGE_SIZE: u32 = 10_000;
const CATALOG_BATCH: usize = 25;

#[derive(Debug, Clone, Default)]
struct SyncedProduct {
    name: String,
    tn_ved: String,
    good_mark_flag: Option<bool>,
    good_turn_flag: Option<bool>,
    card_status: String,
    card_detailed_status: String,
}

/// Non-orderable card statuses (docs/znack_api/znack/ZnackCardStatus.java).
fn is_non_published(status: &str) -> bool {
    matches!(
        status.trim().to_lowercase().as_str(),
        "draft" | "moderation" | "errors" | "notsigned" | "archived"
            | "черновик" | "на модерации" | "требует изменений"
            | "ожидает подписания" | "в архиве" | "архив"
    )
}

pub async fn sync_products(state: &AppState, settings: &ZnackSettings) -> AppResult<(usize, usize)> {
    settings.require_signer()?;
    let token = auth::true_api_token(&state.http, &state.znack_tokens, settings).await?;
    let base = settings.resolved_true_api_base();

    let mut by_gtin: BTreeMap<String, SyncedProduct> = BTreeMap::new();
    let mut technical = 0usize;
    let mut page = 0u32;
    let mut fetched = 0usize;
    let mut total: Option<usize> = None;
    loop {
        let response = client::products_page(&state.http, &base, &token, page, PAGE_SIZE).await?;
        let array = match &response {
            Value::Array(a) => Some(a.clone()),
            Value::Object(o) => o.get("results").and_then(Value::as_array).cloned(),
            _ => None,
        };
        if let Value::Object(o) = &response {
            if let Some(t) = o.get("total").and_then(Value::as_u64) {
                total = Some(t as usize);
            }
        }
        let received = array.as_ref().map(Vec::len).unwrap_or(0);
        if let Some(items) = array {
            for item in items {
                let gtin_raw = text(&item, &["gtin", "productGtin"]);
                if gtin_raw.is_empty() {
                    continue;
                }
                let Ok(gtin) = normalize_gtin(&gtin_raw) else { continue };
                if gtin.starts_with("029") {
                    technical += 1;
                    continue;
                }
                by_gtin.insert(
                    gtin,
                    SyncedProduct {
                        name: text(&item, &["productName", "name"]),
                        tn_ved: tn_ved(&item),
                        good_mark_flag: boolean(&item, &["goodMarkFlag", "good_mark_flag"]),
                        good_turn_flag: boolean(&item, &["goodTurnFlag", "good_turn_flag"]),
                        card_status: text(&item, &["goodStatus", "good_status", "cardStatus"]),
                        card_detailed_status: text(&item, &["goodDetailedStatus", "good_detailed_status"]),
                    },
                );
            }
        }
        fetched += received;
        page += 1;
        if received == 0 || (total.is_none() && received < PAGE_SIZE as usize) {
            break;
        }
        if let Some(t) = total {
            if fetched >= t {
                break;
            }
        }
    }

    // Enrich from the National Catalog in batches of 25 GTINs.
    let gtins: Vec<String> = by_gtin.keys().cloned().collect();
    for batch in gtins.chunks(CATALOG_BATCH) {
        let joined = batch.join(";");
        let response = match client::product_cards(&state.http, &base, &token, &joined).await {
            Ok(v) => v,
            Err(_) => continue, // enrichment is best-effort
        };
        let cards = match &response {
            Value::Array(a) => a.clone(),
            Value::Object(o) => o.get("result").and_then(Value::as_array).cloned().unwrap_or_default(),
            _ => vec![],
        };
        for card in &cards {
            let name = text(card, &["good_name", "productName", "name"]);
            let tnved = tn_ved(card);
            let mark = boolean(card, &["goodMarkFlag", "good_mark_flag"]);
            let turn = boolean(card, &["goodTurnFlag", "good_turn_flag"]);
            let status = text(card, &["goodStatus", "good_status", "cardStatus"]);
            let detailed = text(card, &["goodDetailedStatus", "good_detailed_status"]);
            let Some(identifiers) = card.get("identified_by").and_then(Value::as_array) else {
                continue;
            };
            for identifier in identifiers {
                let id_type = text(identifier, &["type"]);
                if !id_type.is_empty() && !id_type.eq_ignore_ascii_case("gtin") {
                    continue;
                }
                let value = text(identifier, &["value", "gtin"]);
                let Ok(gtin) = normalize_gtin(&value) else { continue };
                if let Some(current) = by_gtin.get_mut(&gtin) {
                    if !name.is_empty() && name != "-" {
                        current.name = name.clone();
                    }
                    if !tnved.is_empty() {
                        current.tn_ved = tnved.clone();
                    }
                    if mark.is_some() {
                        current.good_mark_flag = mark;
                    }
                    if turn.is_some() {
                        current.good_turn_flag = turn;
                    }
                    if !status.is_empty() {
                        current.card_status = status.clone();
                    }
                    if !detailed.is_empty() {
                        current.card_detailed_status = detailed.clone();
                    }
                }
            }
        }
    }

    // Persist publishable cards; drop non-published ones.
    let mut publishable = 0usize;
    let mut skipped = technical;
    {
        let conn = state.db.lock().unwrap();
        for (gtin, p) in &by_gtin {
            if is_non_published(&p.card_status) || is_non_published(&p.card_detailed_status) {
                skipped += 1;
                continue;
            }
            zdb::upsert_product(
                &conn,
                &zdb::ProductUpsert {
                    gtin: gtin.clone(),
                    product_name: p.name.clone(),
                    tn_ved: p.tn_ved.clone(),
                    good_mark_flag: p.good_mark_flag,
                    good_turn_flag: p.good_turn_flag,
                    card_status: p.card_status.clone(),
                    card_detailed_status: p.card_detailed_status.clone(),
                },
            )?;
            publishable += 1;
        }
    }
    Ok((publishable, skipped))
}

fn text(v: &Value, keys: &[&str]) -> String {
    for k in keys {
        match v.get(*k) {
            Some(Value::String(s)) => return s.clone(),
            Some(Value::Number(n)) => return n.to_string(),
            Some(Value::Bool(b)) => return b.to_string(),
            Some(Value::Array(a)) => {
                let parts: Vec<String> = a
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect();
                if !parts.is_empty() {
                    return parts.join(",");
                }
            }
            _ => {}
        }
    }
    String::new()
}

fn boolean(v: &Value, keys: &[&str]) -> Option<bool> {
    for k in keys {
        match v.get(*k) {
            Some(Value::Bool(b)) => return Some(*b),
            Some(Value::String(s)) => {
                if s.eq_ignore_ascii_case("true") || s == "1" {
                    return Some(true);
                }
                if s.eq_ignore_ascii_case("false") || s == "0" {
                    return Some(false);
                }
            }
            Some(Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    return Some(i != 0);
                }
            }
            _ => {}
        }
    }
    None
}

/// ТН ВЭД lookup: direct fields, then good_attrs 13933 (full code) / 3959 (group).
fn tn_ved(product: &Value) -> String {
    let direct = text(
        product,
        &[
            "tnVedCode10", "tnvedCode10", "tnVed10", "tnved10", "tnved_code_10", "tnved_10",
            "tnVedEaes", "productTnved", "product_tnved", "goodsTnvedCode", "tnVed", "tnved",
            "tnVedCode", "tnvedCode", "tnved_code", "tnVedEaesGroup",
        ],
    );
    let full = attribute(product, 13933, &["Код ТНВЭД", "FEACN code"]);
    let group = attribute(product, 3959, &["Группа ТНВЭД", "FEACN group"]);
    let chosen = if !full.is_empty() { full } else if !direct.is_empty() { direct } else { group };
    chosen.split_whitespace().collect()
}

fn attribute(product: &Value, id: i64, names: &[&str]) -> String {
    let attrs = product
        .get("good_attrs")
        .or_else(|| product.get("goodAttrs"))
        .and_then(Value::as_array);
    let Some(attrs) = attrs else { return String::new() };
    for attr in attrs {
        let attr_id = text(attr, &["attr_id", "attrId"]);
        let attr_name = text(attr, &["attr_name", "attrName"]);
        let matches_id = attr_id == id.to_string();
        let matches_name = names.iter().any(|n| n.eq_ignore_ascii_case(&attr_name));
        if matches_id || matches_name {
            return text(attr, &["attr_value", "attrValue", "value"]);
        }
    }
    String::new()
}
