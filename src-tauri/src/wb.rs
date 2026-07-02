//! Wildberries API client. All HTTP goes through here so the token stays in
//! the Rust process and per-API rate limits are enforced centrally.

use crate::config::{CONTENT_BASE, MARKETPLACE_BASE};
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::secrets;
use crate::state::{AppState, RateLimiter};
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde_json::json;

fn token(state: &AppState) -> AppResult<String> {
    let sid = state
        .active_store
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| AppError::Msg("Chưa chọn cửa hàng.".into()))?;
    secrets::get_token(&sid)?
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| {
            AppError::Msg("Cửa hàng này chưa có WB API token (vào Cài đặt để nhập).".into())
        })
}

/// Execute a request that has already been rate-limited, with basic 429 retry.
async fn send(rl: &RateLimiter, build: impl Fn() -> RequestBuilder) -> AppResult<Response> {
    let mut attempt = 0u32;
    loop {
        rl.acquire().await;
        let resp = build().send().await?;
        if resp.status() == StatusCode::TOO_MANY_REQUESTS && attempt < 3 {
            attempt += 1;
            // WB penalises 4xx heavily; back off generously.
            tokio::time::sleep(std::time::Duration::from_millis(1500 * attempt as u64)).await;
            continue;
        }
        return Ok(resp);
    }
}

async fn check(resp: Response) -> AppResult<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let body = resp.text().await.unwrap_or_default();
    Err(AppError::Api {
        status: status.as_u16(),
        body: if body.len() > 500 { body[..500].to_string() } else { body },
    })
}

// ---------------------------------------------------------------------------
// Content API — cards/list
// ---------------------------------------------------------------------------

pub async fn cards_list(
    state: &AppState,
    cursor: Option<(String, i64)>,
) -> AppResult<CardsListResponse> {
    let tok = token(state)?;
    let url = format!("{}/content/v2/get/cards/list", CONTENT_BASE);
    let mut cursor_obj = json!({ "limit": 100 });
    if let Some((updated_at, nm_id)) = &cursor {
        cursor_obj["updatedAt"] = json!(updated_at);
        cursor_obj["nmID"] = json!(nm_id);
    }
    let body = json!({
        "settings": {
            "sort": { "ascending": true },
            "filter": { "withPhoto": -1 },
            "cursor": cursor_obj
        }
    });
    let resp = send(&state.content_rl, || {
        state
            .http
            .request(Method::POST, &url)
            .header("Authorization", &tok)
            .json(&body)
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<CardsListResponse>().await?)
}

// ---------------------------------------------------------------------------
// Marketplace API — orders
// ---------------------------------------------------------------------------

pub async fn orders_new(state: &AppState) -> AppResult<Vec<WbOrder>> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/orders/new", MARKETPLACE_BASE);
    let resp = send(&state.marketplace_rl, || {
        state.http.get(&url).header("Authorization", &tok)
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<OrdersResponse>().await?.orders)
}

/// Seller FBS warehouses (id -> name).
pub async fn warehouses(state: &AppState) -> AppResult<Vec<Warehouse>> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/warehouses", MARKETPLACE_BASE);
    let resp = send(&state.marketplace_rl, || {
        state.http.get(&url).header("Authorization", &tok)
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<Vec<Warehouse>>().await?)
}

/// Assign a Честный ЗНАК labeling code to an assembly order
/// (PUT /api/v3/orders/{orderId}/meta/sgtin). The full code with GS
/// separators and crypto tail is accepted; serde escapes GS as \\u001D in JSON.
pub async fn order_set_sgtin(state: &AppState, order_id: i64, sgtin: &str) -> AppResult<()> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/orders/{}/meta/sgtin", MARKETPLACE_BASE, order_id);
    let body = json!({ "sgtins": [sgtin] });
    let resp = send(&state.marketplace_rl, || {
        state
            .http
            .put(&url)
            .header("Authorization", &tok)
            .json(&body)
    })
    .await?;
    check(resp).await?;
    Ok(())
}

/// Fetch waybill stickers (PNG, 58x40) for up to 100 order ids per call.
pub async fn orders_stickers(state: &AppState, ids: &[i64]) -> AppResult<Vec<Sticker>> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/orders/stickers", MARKETPLACE_BASE);
    let mut all = Vec::new();
    for chunk in ids.chunks(100) {
        let body = json!({ "orders": chunk });
        let resp = send(&state.marketplace_rl, || {
            state
                .http
                .post(&url)
                .header("Authorization", &tok)
                .query(&[("type", "png"), ("width", "58"), ("height", "40")])
                .json(&body)
        })
        .await?;
        let resp = check(resp).await?;
        let parsed = resp.json::<StickersResponse>().await?;
        all.extend(parsed.stickers);
    }
    Ok(all)
}

// ---------------------------------------------------------------------------
// Marketplace API — supplies
// ---------------------------------------------------------------------------

pub async fn supplies_list(state: &AppState, next: i64, limit: i64) -> AppResult<SuppliesResponse> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/supplies", MARKETPLACE_BASE);
    let resp = send(&state.marketplace_rl, || {
        state
            .http
            .get(&url)
            .header("Authorization", &tok)
            .query(&[("limit", limit.to_string()), ("next", next.to_string())])
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<SuppliesResponse>().await?)
}

pub async fn create_supply(state: &AppState, name: &str) -> AppResult<String> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/supplies", MARKETPLACE_BASE);
    let body = json!({ "name": name });
    let resp = send(&state.marketplace_rl, || {
        state
            .http
            .post(&url)
            .header("Authorization", &tok)
            .json(&body)
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<CreateSupplyResponse>().await?.id)
}

pub async fn add_orders_to_supply(
    state: &AppState,
    supply_id: &str,
    ids: &[i64],
) -> AppResult<()> {
    let tok = token(state)?;
    let url = format!(
        "{}/api/marketplace/v3/supplies/{}/orders",
        MARKETPLACE_BASE, supply_id
    );
    for chunk in ids.chunks(100) {
        let body = json!({ "orders": chunk });
        let resp = send(&state.marketplace_rl, || {
            state
                .http
                .patch(&url)
                .header("Authorization", &tok)
                .json(&body)
        })
        .await?;
        check(resp).await?;
    }
    Ok(())
}

/// Order ids assigned to a supply. (The `/orders` path only allows PATCH; the
/// GET counterpart is `/order-ids`.)
pub async fn supply_order_ids(state: &AppState, supply_id: &str) -> AppResult<Vec<i64>> {
    let tok = token(state)?;
    let url = format!(
        "{}/api/marketplace/v3/supplies/{}/order-ids",
        MARKETPLACE_BASE, supply_id
    );
    let resp = send(&state.marketplace_rl, || {
        state.http.get(&url).header("Authorization", &tok)
    })
    .await?;
    let resp = check(resp).await?;
    Ok(resp.json::<OrderIdsResponse>().await?.order_ids)
}

/// Get one page of assembly orders (any status) created in the last ~30 days.
/// Used to backfill order details (chrtId/sku) for ids not present locally.
pub async fn assembly_orders(
    state: &AppState,
    next: i64,
    limit: i64,
) -> AppResult<(Vec<WbOrder>, i64)> {
    let tok = token(state)?;
    let url = format!("{}/api/v3/orders", MARKETPLACE_BASE);
    let resp = send(&state.marketplace_rl, || {
        state
            .http
            .get(&url)
            .header("Authorization", &tok)
            .query(&[("limit", limit.to_string()), ("next", next.to_string())])
    })
    .await?;
    let resp = check(resp).await?;
    let parsed = resp.json::<AssemblyOrdersResponse>().await?;
    Ok((parsed.orders, parsed.next))
}

/// Validate a specific token string by hitting orders/new.
pub async fn validate_with(state: &AppState, tok: &str) -> AppResult<()> {
    let tok = tok.trim();
    if tok.is_empty() {
        return Err(AppError::Msg("Token rỗng.".into()));
    }
    let url = format!("{}/api/v3/orders/new", MARKETPLACE_BASE);
    let resp = send(&state.marketplace_rl, || {
        state.http.get(&url).header("Authorization", tok)
    })
    .await?;
    match resp.status() {
        s if s.is_success() => Ok(()),
        StatusCode::UNAUTHORIZED => Err(AppError::Msg("Token không hợp lệ (401).".into())),
        StatusCode::FORBIDDEN => {
            Err(AppError::Msg("Token thiếu quyền Marketplace (403).".into()))
        }
        other => Err(AppError::Api {
            status: other.as_u16(),
            body: resp.text().await.unwrap_or_default(),
        }),
    }
}

/// Validate the active store's token.
pub async fn validate_token(state: &AppState) -> AppResult<()> {
    let tok = token(state)?;
    validate_with(state, &tok).await
}
