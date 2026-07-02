//! Token acquisition + cache for True API (Bearer) and СУЗ (clientToken).
//! Port of docs/znack_api/znack/ZnackAuthService.java.
//!
//! Flow: GET /auth/key → sign `data` with УКЭП (attached CMS, base64) →
//! POST /auth/simpleSignIn[/{omsConnection}]. Tokens live ≤10h; we refresh
//! 30s before expiry.

use crate::error::{AppError, AppResult};
use crate::znack::models::ZnackSettings;
use crate::znack::{client, sign};
use base64::Engine;
use serde_json::json;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct TokenCache {
    true_api: Mutex<Option<CachedToken>>,
    suz: Mutex<Option<CachedToken>>,
    /// INN extracted from the last JWT — used as participant INN.
    participant_inn: Mutex<Option<String>>,
}

struct CachedToken {
    value: String,
    expires_at: Instant,
    /// Base URL + connection the token was issued for; invalidated on change.
    key: String,
}

impl TokenCache {
    pub fn invalidate(&self) {
        *self.true_api.lock().unwrap() = None;
        *self.suz.lock().unwrap() = None;
        *self.participant_inn.lock().unwrap() = None;
    }
    pub fn participant_inn(&self) -> Option<String> {
        self.participant_inn.lock().unwrap().clone()
    }
}

pub async fn true_api_token(
    http: &reqwest::Client,
    cache: &TokenCache,
    settings: &ZnackSettings,
) -> AppResult<String> {
    settings.require_signer()?;
    token(http, cache, settings, false).await
}

pub async fn suz_token(
    http: &reqwest::Client,
    cache: &TokenCache,
    settings: &ZnackSettings,
) -> AppResult<String> {
    settings.require_oms()?;
    token(http, cache, settings, true).await
}

async fn token(
    http: &reqwest::Client,
    cache: &TokenCache,
    settings: &ZnackSettings,
    suz: bool,
) -> AppResult<String> {
    let base = settings.resolved_true_api_base();
    let connection = if suz { settings.oms_connection.trim().to_string() } else { String::new() };
    let cache_key = format!("{base}|{connection}|{}", settings.cert_thumbprint.trim());

    {
        let slot = if suz { cache.suz.lock().unwrap() } else { cache.true_api.lock().unwrap() };
        if let Some(t) = slot.as_ref() {
            if t.key == cache_key && t.expires_at > Instant::now() + Duration::from_secs(30) {
                return Ok(t.value.clone());
            }
        }
    }

    let (uuid, data) = client::auth_key(http, &base).await?;
    let signed = {
        let settings = settings.clone();
        tauri::async_runtime::spawn_blocking(move || {
            sign::sign(&settings, data.as_bytes(), sign::SignMode::Attached)
        })
        .await
        .map_err(|e| AppError::Msg(format!("Lỗi luồng ký: {e}")))??
    };
    let mut body = json!({ "uuid": uuid, "data": sign::to_base64(&signed) });
    let inn = settings.participant_inn.trim();
    if !inn.is_empty() {
        body["inn"] = json!(inn);
    }
    let response = client::sign_in(
        http,
        &base,
        if suz { Some(connection.as_str()) } else { None },
        &body,
    )
    .await?;

    let value = ["clientToken", "token", "sessionToken", "jwt"]
        .iter()
        .find_map(|k| response.get(*k).and_then(serde_json::Value::as_str))
        .map(str::to_string)
        .ok_or_else(|| AppError::Msg("Phản hồi xác thực Znack không chứa token.".into()))?;
    let expires_in = response.get("expiresIn").and_then(serde_json::Value::as_u64).unwrap_or(36_000);

    let cached = CachedToken {
        value: value.clone(),
        expires_at: Instant::now() + Duration::from_secs(expires_in),
        key: cache_key,
    };
    {
        let mut slot = if suz { cache.suz.lock().unwrap() } else { cache.true_api.lock().unwrap() };
        *slot = Some(cached);
    }
    if let Some(derived) = participant_inn_from_jwt(&value) {
        *cache.participant_inn.lock().unwrap() = Some(derived);
    }
    Ok(value)
}

/// Extract the participant INN from the JWT claims (port of
/// ZnackAuthService.participantInn in the Java reference).
fn participant_inn_from_jwt(jwt: &str) -> Option<String> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    find_inn(&claims)
}

fn find_inn(value: &serde_json::Value) -> Option<String> {
    use serde_json::Value;
    match value {
        Value::Object(o) => {
            for key in ["participant_inn", "participantInn", "inn", "userInn", "user_inn"] {
                if let Some(Value::String(s)) = o.get(key) {
                    if (s.len() == 10 || s.len() == 12) && s.bytes().all(|b| b.is_ascii_digit()) {
                        return Some(s.clone());
                    }
                }
            }
            o.values().find_map(find_inn)
        }
        Value::Array(a) => a.iter().find_map(find_inn),
        _ => None,
    }
}
