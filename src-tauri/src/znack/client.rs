//! HTTP client for True API + СУЗ (port of docs/znack_api/znack/ZnackApiClient.java).
//!
//! True API methods use `Authorization: Bearer <token>`; СУЗ methods use the
//! `clientToken` header plus `omsId` query param; write calls also carry a
//! detached-CMS `X-Signature`.

use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde_json::Value;

/// Strip a trailing `/api/vN/true-api` (or `/lk`) so we can rebuild versioned bases.
fn api_root(base: &str) -> String {
    let trimmed = base.trim_end_matches('/');
    for suffix_version in [3, 4] {
        for kind in ["true-api", "lk"] {
            let suffix = format!("/api/v{suffix_version}/{kind}");
            if let Some(stripped) = trimmed.strip_suffix(&suffix) {
                return stripped.trim_end_matches('/').to_string();
            }
        }
    }
    trimmed.to_string()
}

fn true_api_base(base: &str, version: u8) -> String {
    format!("{}/api/v{}/true-api", api_root(base), version)
}

fn url_encode(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

async fn read_json(resp: reqwest::Response) -> AppResult<Value> {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(AppError::Api { status: status.as_u16(), body: body.chars().take(2000).collect() });
    }
    if body.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&body).map_err(|e| {
        AppError::Msg(format!("Znack API trả về JSON không hợp lệ: {e}: {}", body.chars().take(300).collect::<String>()))
    })
}

fn bearer(req: reqwest::RequestBuilder, token: &str) -> reqwest::RequestBuilder {
    req.header("Accept", "application/json")
        .header("Authorization", format!("Bearer {token}"))
}

fn suz(req: reqwest::RequestBuilder, token: &str) -> reqwest::RequestBuilder {
    req.header("Accept", "application/json").header("clientToken", token)
}

// --- auth ---------------------------------------------------------------------

pub async fn auth_key(http: &Client, base: &str) -> AppResult<(String, String)> {
    let url = format!("{}/auth/key", true_api_base(base, 3));
    let v = read_json(http.get(&url).header("Accept", "application/json").send().await?).await?;
    let uuid = v.get("uuid").and_then(Value::as_str).unwrap_or_default().to_string();
    let data = v.get("data").and_then(Value::as_str).unwrap_or_default().to_string();
    if uuid.is_empty() || data.is_empty() {
        return Err(AppError::Msg("Phản hồi /auth/key thiếu uuid/data.".into()));
    }
    Ok((uuid, data))
}

pub async fn sign_in(
    http: &Client,
    base: &str,
    oms_connection: Option<&str>,
    body: &Value,
) -> AppResult<Value> {
    let path = match oms_connection {
        Some(c) if !c.trim().is_empty() => format!("/auth/simpleSignIn/{}", c.trim()),
        _ => "/auth/simpleSignIn".to_string(),
    };
    let url = format!("{}{}", true_api_base(base, 3), path);
    read_json(
        http.post(&url)
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?,
    )
    .await
}

// --- catalog ------------------------------------------------------------------

pub async fn products_page(
    http: &Client,
    base: &str,
    token: &str,
    page: u32,
    limit: u32,
) -> AppResult<Value> {
    let url = format!(
        "{}/product/gtin?includeSubaccount=false&limit={limit}&page={page}&pg=lp",
        true_api_base(base, 4)
    );
    read_json(bearer(http.get(&url), token).send().await?).await
}

pub async fn product_cards(http: &Client, base: &str, token: &str, gtins: &str) -> AppResult<Value> {
    let url = format!(
        "{}/nk/feed-product?gtins={}",
        true_api_base(base, 3),
        url_encode(gtins)
    );
    read_json(bearer(http.get(&url), token).send().await?).await
}

// --- СУЗ (OMS) ------------------------------------------------------------------

pub async fn create_order(
    http: &Client,
    suz_base: &str,
    token: &str,
    oms_id: &str,
    body: Vec<u8>,
    signature_b64: &str,
) -> AppResult<Value> {
    let url = format!("{}/api/v3/order?omsId={}", suz_base.trim_end_matches('/'), url_encode(oms_id));
    read_json(
        suz(http.post(&url), token)
            .header("Content-Type", "application/json")
            .header("X-Signature", signature_b64)
            .body(body)
            .send()
            .await?,
    )
    .await
}

pub async fn order_status(
    http: &Client,
    suz_base: &str,
    token: &str,
    oms_id: &str,
    order_id: &str,
) -> AppResult<Value> {
    let url = format!(
        "{}/api/v3/order/status?omsId={}&orderId={}",
        suz_base.trim_end_matches('/'),
        url_encode(oms_id),
        url_encode(order_id)
    );
    read_json(suz(http.get(&url), token).send().await?).await
}

pub async fn codes(
    http: &Client,
    suz_base: &str,
    token: &str,
    oms_id: &str,
    order_id: &str,
    gtin: &str,
    quantity: i64,
) -> AppResult<Value> {
    let url = format!(
        "{}/api/v3/codes?omsId={}&orderId={}&quantity={quantity}&gtin={}",
        suz_base.trim_end_matches('/'),
        url_encode(oms_id),
        url_encode(order_id),
        url_encode(gtin)
    );
    read_json(suz(http.get(&url), token).send().await?).await
}

// --- True API documents (введение в оборот) ---------------------------------------

pub async fn create_document(http: &Client, base: &str, token: &str, body: &Value) -> AppResult<String> {
    let url = format!("{}/lk/documents/create?pg=lp", true_api_base(base, 3));
    let response = read_json(bearer(http.post(&url), token).json(body).send().await?).await?;
    let id = document_id(&response);
    if id.is_empty() {
        return Err(AppError::Msg(format!(
            "Phản hồi tạo document không có ID: {}",
            response.to_string().chars().take(300).collect::<String>()
        )));
    }
    Ok(id)
}

pub async fn document_info(http: &Client, base: &str, token: &str, document_id: &str) -> AppResult<Value> {
    let url = format!(
        "{}/doc/{}/info?pg=lp",
        true_api_base(base, 4),
        url_encode(document_id)
    );
    read_json(bearer(http.get(&url), token).send().await?).await
}

pub async fn cises_info(http: &Client, base: &str, token: &str, body: &Value) -> AppResult<Value> {
    let url = format!("{}/cises/info?pg=lp", true_api_base(base, 3));
    read_json(bearer(http.post(&url), token).json(body).send().await?).await
}

fn document_id(response: &Value) -> String {
    match response {
        Value::String(s) => s.trim().to_string(),
        Value::Object(o) => {
            for key in ["uuid", "document_id", "documentId", "id"] {
                if let Some(Value::String(s)) = o.get(key) {
                    return s.trim().to_string();
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}
