//! KIZ purchase pipeline: order → poll buffer → download codes →
//! (optional) introduction. Port of docs/znack_api/znack/ZnackPurchaseCoordinator.java
//! and ZnackKizOrderService / ZnackKizCodeService.
//!
//! Ambiguity protection: if order creation fails with a 5xx/network error we
//! cannot know whether СУЗ accepted it, so the pipeline parks in
//! CREATING_ORDER and is never auto-retried (avoids duplicate paid orders).
//! The user resolves it manually (abort) after checking the ЛК.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::znack::models::{self, stage, ZnackSettings};
use crate::znack::{auth, client, db as zdb, introduction, sign};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

/// Pipelines currently being advanced (prevents overlapping runs).
static RUNNING: Mutex<Option<HashSet<i64>>> = Mutex::new(None);

fn try_claim(id: i64) -> bool {
    let mut guard = RUNNING.lock().unwrap();
    guard.get_or_insert_with(HashSet::new).insert(id)
}

fn release(id: i64) {
    if let Some(set) = RUNNING.lock().unwrap().as_mut() {
        set.remove(&id);
    }
}

/// Start a purchase: validate, create the pipeline row, advance once inline.
pub async fn start(state: &AppState, gtin: &str, quantity: i64) -> AppResult<i64> {
    if quantity <= 0 {
        return Err(AppError::Msg("Số lượng phải lớn hơn 0.".into()));
    }
    let gtin = models::require_production_orderable(gtin)?;
    let settings = {
        let conn = state.db.lock().unwrap();
        if !zdb::product_exists(&conn, &gtin)? {
            return Err(AppError::Msg("GTIN chưa được đồng bộ cho cửa hàng này.".into()));
        }
        if zdb::find_active_pipeline(&conn, &gtin)?.is_some() {
            return Err(AppError::Msg("Đang có một phiên mua KIZ khác cho GTIN này.".into()));
        }
        zdb::get_settings(&conn)?
    };
    settings.require_oms()?;

    let pipeline_id = {
        let conn = state.db.lock().unwrap();
        zdb::create_pipeline(&conn, &gtin, quantity)?
    };
    let _ = advance(state, pipeline_id).await;
    Ok(pipeline_id)
}

/// Advance a pipeline one step. Errors are persisted on the pipeline row.
pub async fn advance(state: &AppState, pipeline_id: i64) -> AppResult<()> {
    if !try_claim(pipeline_id) {
        return Ok(());
    }
    let result = advance_inner(state, pipeline_id).await;
    release(pipeline_id);
    result
}

async fn advance_inner(state: &AppState, pipeline_id: i64) -> AppResult<()> {
    let (pipeline, settings) = {
        let conn = state.db.lock().unwrap();
        let Some(p) = zdb::find_pipeline(&conn, pipeline_id)? else {
            return Ok(());
        };
        (p, zdb::get_settings(&conn)?)
    };
    let step = match pipeline.stage.as_str() {
        stage::VALIDATING => create_order(state, &settings, &pipeline).await,
        stage::POLLING_ORDER => poll_order(state, &settings, &pipeline).await,
        stage::DOWNLOADING_CODES => download_codes(state, &settings, &pipeline).await,
        stage::WAITING_INTRODUCTION_READINESS => check_readiness(state, &settings, &pipeline).await,
        stage::SUBMITTING_INTRODUCTION => submit_introduction(state, &settings, &pipeline).await,
        stage::POLLING_INTRODUCTION => poll_introduction(state, &settings, &pipeline).await,
        // CREATING_ORDER = parked ambiguous state; terminal stages: nothing to do.
        _ => Ok(()),
    };
    if let Err(e) = step {
        let message = e.to_string();
        let conn = state.db.lock().unwrap();
        if let Some(current) = zdb::find_pipeline(&conn, pipeline_id)? {
            match current.stage.as_str() {
                // Retryable stages keep their stage with the error recorded.
                stage::POLLING_ORDER
                | stage::DOWNLOADING_CODES
                | stage::WAITING_INTRODUCTION_READINESS
                | stage::POLLING_INTRODUCTION => {
                    zdb::update_pipeline(&conn, pipeline_id, None, &current.stage, Some(&message))?;
                }
                // CREATING_ORDER stays parked (ambiguous) — keep message.
                stage::CREATING_ORDER => {
                    zdb::update_pipeline(&conn, pipeline_id, None, stage::CREATING_ORDER, Some(&message))?;
                }
                _ => {
                    zdb::update_pipeline(&conn, pipeline_id, None, stage::FAILED, Some(&message))?;
                }
            }
        }
        return Err(e);
    }
    Ok(())
}

async fn create_order(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    settings.require_oms()?;
    // Build + sign the СУЗ order body (lp, template 10, operator serials).
    let body = json!({
        "productGroup": "lp",
        "attributes": { "releaseMethodType": "PRODUCTION" },
        "products": [{
            "gtin": pipeline.gtin,
            "quantity": pipeline.quantity,
            "serialNumberType": "OPERATOR",
            "templateId": 10,
            "cisType": "UNIT",
        }],
    });
    let body_bytes = serde_json::to_vec(&body)?;
    // Signing and token acquisition happen BEFORE the stage moves to
    // CREATING_ORDER: a failure here is definitive (no order request has been
    // sent), so it must fail the pipeline, not park it as ambiguous.
    let signature = {
        let settings = settings.clone();
        let bytes = body_bytes.clone();
        tauri::async_runtime::spawn_blocking(move || {
            sign::sign(&settings, &bytes, sign::SignMode::Detached)
        })
        .await
        .map_err(|e| AppError::Msg(format!("Lỗi luồng ký: {e}")))??
    };
    let signature = sign::to_base64(&signature);
    let token = auth::suz_token(&state.http, &state.znack_tokens, settings).await?;

    // From here on a failure may mean СУЗ accepted the order — enter the
    // parked-on-error CREATING_ORDER stage only now.
    let order_row = {
        let conn = state.db.lock().unwrap();
        zdb::update_pipeline(&conn, pipeline.id, None, stage::CREATING_ORDER, None)?;
        zdb::create_order_draft(&conn, &pipeline.gtin, pipeline.quantity)?
    };

    let result = client::create_order(
        &state.http,
        &settings.resolved_suz_base(),
        &token,
        settings.oms_id.trim(),
        body_bytes,
        &signature,
    )
    .await;

    let conn = state.db.lock().unwrap();
    match result {
        Ok(response) => {
            let external = response
                .get("orderId")
                .or_else(|| response.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if external.is_empty() {
                let msg = "Phản hồi tạo đơn СУЗ không có orderId — kiểm tra ЛК trước khi thử lại.";
                zdb::update_pipeline(&conn, pipeline.id, Some(order_row), stage::CREATING_ORDER, Some(msg))?;
                return Ok(());
            }
            zdb::update_order(&conn, order_row, Some(&external), Some("CREATED"), Some("SUBMITTED"), None)?;
            zdb::update_pipeline(&conn, pipeline.id, Some(order_row), stage::POLLING_ORDER, None)?;
            Ok(())
        }
        Err(AppError::Api { status, body }) if status < 500 => {
            // Definitive rejection — safe to mark failed.
            let msg = format!("СУЗ từ chối đơn (HTTP {status}): {body}");
            zdb::update_order(&conn, order_row, None, None, Some("FAILED"), Some(&msg))?;
            zdb::update_pipeline(&conn, pipeline.id, Some(order_row), stage::FAILED, Some(&msg))?;
            Ok(())
        }
        Err(e) => {
            // Ambiguous: server error / network failure after the request may
            // have been accepted. Park; never auto-retry.
            let msg = format!(
                "Kết quả tạo đơn không xác định ({e}). Kiểm tra đơn trong ЛК СУЗ rồi huỷ phiên này nếu cần."
            );
            zdb::update_order(&conn, order_row, None, None, Some("FAILED"), Some(&msg))?;
            zdb::update_pipeline(&conn, pipeline.id, Some(order_row), stage::CREATING_ORDER, Some(&msg))?;
            Ok(())
        }
    }
}

async fn poll_order(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    let order_id = required_order(pipeline)?;
    let external = {
        let conn = state.db.lock().unwrap();
        zdb::find_order(&conn, order_id)?
            .external_order_id
            .ok_or_else(|| AppError::Msg("Đơn KIZ không có ID СУЗ.".into()))?
    };
    let token = auth::suz_token(&state.http, &state.znack_tokens, settings).await?;
    let response = client::order_status(
        &state.http,
        &settings.resolved_suz_base(),
        &token,
        settings.oms_id.trim(),
        &external,
    )
    .await?;

    let mut remote = "PENDING".to_string();
    let mut available = 0i64;
    let mut rejected = false;
    let mut reason: Option<String> = None;
    if let Value::Array(buffers) = &response {
        for b in buffers {
            let status = b
                .get("bufferStatus")
                .or_else(|| b.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if !status.is_empty() {
                remote = status.to_string();
            }
            available += b.get("availableCodes").and_then(Value::as_i64).unwrap_or(0);
            if let Some(r) = b.get("rejectionReason").and_then(Value::as_str) {
                if !r.is_empty() {
                    reason = Some(r.to_string());
                }
            }
            rejected |= status.eq_ignore_ascii_case("REJECTED");
        }
    }

    let conn = state.db.lock().unwrap();
    if rejected || remote.eq_ignore_ascii_case("DECLINED") {
        let msg = reason.unwrap_or_else(|| "Đơn bị СУЗ từ chối.".to_string());
        zdb::update_order(&conn, order_id, None, Some(&remote), Some("FAILED"), Some(&msg))?;
        zdb::update_pipeline(&conn, pipeline.id, None, stage::FAILED, Some(&msg))?;
    } else if remote.eq_ignore_ascii_case("READY") || available > 0 {
        zdb::update_order(&conn, order_id, None, Some(&remote), Some("CODES_READY"), None)?;
        zdb::update_pipeline(&conn, pipeline.id, None, stage::DOWNLOADING_CODES, None)?;
    } else {
        zdb::update_order(&conn, order_id, None, Some(&remote), Some("WAITING_CODES"), None)?;
        zdb::update_pipeline(&conn, pipeline.id, None, stage::POLLING_ORDER, None)?;
    }
    Ok(())
}

async fn download_codes(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    let order_id = required_order(pipeline)?;
    let order = {
        let conn = state.db.lock().unwrap();
        zdb::find_order(&conn, order_id)?
    };
    let external = order
        .external_order_id
        .clone()
        .ok_or_else(|| AppError::Msg("Đơn KIZ không có ID СУЗ.".into()))?;
    let token = auth::suz_token(&state.http, &state.znack_tokens, settings).await?;
    let response = client::codes(
        &state.http,
        &settings.resolved_suz_base(),
        &token,
        settings.oms_id.trim(),
        &external,
        &order.gtin,
        order.quantity,
    )
    .await?;

    let mut codes: Vec<String> = vec![];
    let array = match &response {
        Value::Object(o) => o.get("codes").and_then(Value::as_array).cloned(),
        Value::Array(a) => Some(a.clone()),
        _ => None,
    };
    if let Some(items) = array {
        for item in items {
            match item {
                Value::String(s) => codes.push(s),
                Value::Object(o) => {
                    if let Some(Value::String(s)) = o.get("cis") {
                        codes.push(s.clone());
                    }
                }
                _ => {}
            }
        }
    }
    let block_id = response
        .get("blockId")
        .and_then(Value::as_str)
        .map(str::to_string);

    let downloaded = {
        let conn = state.db.lock().unwrap();
        zdb::insert_codes(&conn, order_id, &order.gtin, &codes, block_id.as_deref())?;
        zdb::count_codes(&conn, order_id)?
    };
    if downloaded < order.quantity {
        return Err(AppError::Msg(format!(
            "Đã tải {downloaded}/{} mã KIZ; sẽ thử tải tiếp.",
            order.quantity
        )));
    }

    let conn = state.db.lock().unwrap();
    zdb::update_order(&conn, order_id, None, Some("READY"), Some("CODES_DOWNLOADED"), None)?;
    // Introduction runs automatically only when the conformity document is
    // configured; without it the purchase simply completes.
    if settings.has_goods_document() {
        zdb::update_pipeline(&conn, pipeline.id, None, stage::WAITING_INTRODUCTION_READINESS, None)?;
    } else {
        zdb::update_pipeline(&conn, pipeline.id, None, stage::COMPLETED, None)?;
    }
    Ok(())
}

async fn check_readiness(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    let order_id = required_order(pipeline)?;
    if !settings.has_goods_document() {
        let conn = state.db.lock().unwrap();
        zdb::update_pipeline(&conn, pipeline.id, None, stage::COMPLETED, None)?;
        return Ok(());
    }
    let codes = {
        let conn = state.db.lock().unwrap();
        zdb::codes_for_order(&conn, order_id)?
    };
    let readiness = introduction::check_readiness(state, settings, &pipeline.gtin, &codes).await?;
    let conn = state.db.lock().unwrap();
    if readiness.all_introduced {
        zdb::mark_codes_legal(&conn, order_id, models::legal_status::IN_CIRCULATION, None)?;
        zdb::update_order(&conn, order_id, None, None, Some("INTRODUCED"), None)?;
        zdb::update_pipeline(&conn, pipeline.id, None, stage::INTRODUCED, None)?;
    } else if readiness.ready {
        zdb::update_pipeline(&conn, pipeline.id, None, stage::SUBMITTING_INTRODUCTION, None)?;
    } else {
        zdb::update_pipeline(
            &conn,
            pipeline.id,
            None,
            stage::WAITING_INTRODUCTION_READINESS,
            readiness.message.as_deref(),
        )?;
    }
    Ok(())
}

async fn submit_introduction(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    let order_id = required_order(pipeline)?;
    let (existing, codes) = {
        let conn = state.db.lock().unwrap();
        (zdb::latest_document(&conn, order_id)?, zdb::codes_for_order(&conn, order_id)?)
    };
    match existing {
        Some((_, Some(external), _)) if !external.is_empty() => {
            // Already submitted — just move on to polling.
            let conn = state.db.lock().unwrap();
            zdb::update_pipeline(&conn, pipeline.id, None, stage::POLLING_INTRODUCTION, None)?;
            return Ok(());
        }
        Some((_, _, status)) if status == "REJECTED" => {
            // Definitive 4xx rejection — allow a fresh submission below.
        }
        Some((_, _, status)) if status == "FAILED" || status == "DRAFT" || status == "SUBMITTED" => {
            let msg = "Kết quả gửi lưu thông không xác định; không tự thử lại. Kiểm tra document trong ЛК.";
            let conn = state.db.lock().unwrap();
            zdb::update_pipeline(&conn, pipeline.id, None, stage::FAILED, Some(msg))?;
            return Ok(());
        }
        _ => {}
    }
    introduction::submit(state, settings, order_id, &pipeline.gtin, &codes).await?;
    let conn = state.db.lock().unwrap();
    zdb::update_pipeline(&conn, pipeline.id, None, stage::POLLING_INTRODUCTION, None)?;
    Ok(())
}

async fn poll_introduction(
    state: &AppState,
    settings: &ZnackSettings,
    pipeline: &models::ZnackPipeline,
) -> AppResult<()> {
    let order_id = required_order(pipeline)?;
    let codes = {
        let conn = state.db.lock().unwrap();
        zdb::codes_for_order(&conn, order_id)?
    };
    if introduction::confirm(state, settings, order_id, &codes).await? {
        let conn = state.db.lock().unwrap();
        zdb::update_pipeline(&conn, pipeline.id, None, stage::INTRODUCED, None)?;
    }
    Ok(())
}

fn required_order(pipeline: &models::ZnackPipeline) -> AppResult<i64> {
    pipeline
        .order_id
        .ok_or_else(|| AppError::Msg("Phiên mua chưa gắn với đơn KIZ nào.".into()))
}

/// Abort a parked/failed pipeline (user confirmed in the СУЗ ЛК that no order
/// was charged, or gives up). Only non-terminal stages can be aborted.
pub fn abort(state: &AppState, pipeline_id: i64) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    let Some(p) = zdb::find_pipeline(&conn, pipeline_id)? else {
        return Err(AppError::Msg("Không tìm thấy phiên mua.".into()));
    };
    if !stage::is_active(&p.stage) {
        return Ok(());
    }
    zdb::update_pipeline(&conn, pipeline_id, None, stage::FAILED, Some("Đã huỷ thủ công."))?;
    Ok(())
}

/// Minimum seconds between advances for a pipeline, by stage/error state —
/// waiting stages hit remote APIs, so they poll slower.
fn poll_interval(p: &models::ZnackPipeline) -> i64 {
    if p.error_message.as_deref().is_some_and(|m| !m.is_empty()) {
        return 30;
    }
    match p.stage.as_str() {
        stage::POLLING_ORDER => 10,
        stage::WAITING_INTRODUCTION_READINESS | stage::POLLING_INTRODUCTION => 30,
        _ => 5,
    }
}

fn due(p: &models::ZnackPipeline) -> bool {
    match chrono::DateTime::parse_from_rfc3339(&p.updated_at) {
        Ok(updated) => {
            let age = chrono::Utc::now().signed_duration_since(updated).num_seconds();
            age >= poll_interval(p)
        }
        Err(_) => true,
    }
}

/// Background poller: advances all active pipelines forever. Spawned once at
/// startup; also releases stale print reservations from a previous run.
pub async fn run_poller(app: tauri::AppHandle) {
    use tauri::Manager;
    {
        let state: tauri::State<AppState> = app.state();
        let conn = state.db.lock().unwrap();
        let _ = zdb::release_stale_reservations(&conn);
    }
    loop {
        let (ids, listed_store) = {
            let state: tauri::State<AppState> = app.state();
            let store = state.active_store.lock().unwrap().clone();
            let conn = state.db.lock().unwrap();
            let ids: Vec<i64> = zdb::active_pipelines(&conn)
                .map(|list| {
                    list.iter()
                        .filter(|p| p.stage != stage::CREATING_ORDER) // parked = manual
                        .filter(|p| due(p))
                        .map(|p| p.id)
                        .collect()
                })
                .unwrap_or_default();
            (ids, store)
        };
        for id in ids {
            let state: tauri::State<AppState> = app.state();
            // The DB handle follows the active store; if the user switched
            // stores since we listed pipelines, these ids belong to another
            // database — stop and re-list on the next tick.
            if *state.active_store.lock().unwrap() != listed_store {
                break;
            }
            let _ = advance(&state, id).await;
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
