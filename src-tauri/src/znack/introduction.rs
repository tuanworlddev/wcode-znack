//! Ввод в оборот (introduction into circulation) — LP_INTRODUCE_GOODS.
//! Port of docs/znack_api/znack/ZnackIntroductionService.java and
//! ZnackIntroductionReadinessService.java.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::znack::models::{cis_for_true_api, legal_status, ZnackSettings};
use crate::znack::{auth, client, db as zdb, sign};
use base64::Engine;
use serde_json::{json, Value};

const CISES_BATCH: usize = 1_000;

pub struct Readiness {
    pub ready: bool,
    pub all_introduced: bool,
    pub message: Option<String>,
}

/// Check whether the downloaded codes are visible in True API as APPLIED and
/// the goods card is complete enough to submit the introduction document.
pub async fn check_readiness(
    state: &AppState,
    settings: &ZnackSettings,
    gtin: &str,
    codes: &[(i64, String)],
) -> AppResult<Readiness> {
    let token = auth::true_api_token(&state.http, &state.znack_tokens, settings).await?;
    let base = settings.resolved_true_api_base();

    if codes.is_empty() {
        return Ok(Readiness {
            ready: false,
            all_introduced: false,
            message: Some("Chưa có mã KIZ nào được tải về.".into()),
        });
    }

    let mut applied = 0usize;
    let mut introduced = 0usize;
    let mut pending = 0usize;
    let mut reason: Option<String> = None;
    for batch in codes.chunks(CISES_BATCH) {
        let request: Vec<Value> = batch
            .iter()
            .map(|(_, raw)| Value::String(cis_for_true_api(raw)))
            .collect();
        let response = client::cises_info(&state.http, &base, &token, &Value::Array(request)).await?;
        let entries: Vec<Value> = match response {
            Value::Array(a) => a,
            v @ Value::Object(_) => vec![v],
            _ => {
                pending += batch.len();
                reason.get_or_insert("True API không trả về danh sách KIZ.".into());
                continue;
            }
        };
        let mut seen = 0usize;
        for entry in &entries {
            let error = text(entry, &["errorMessage", "error_message", "errorCode", "error_code"]);
            let info = entry
                .get("cisInfo")
                .filter(|v| v.is_object())
                .cloned()
                .or_else(|| entry.get("status").map(|_| entry.clone()));
            seen += 1;
            let Some(info) = info else {
                pending += 1;
                reason.get_or_insert("True API chưa trả về chi tiết KIZ.".into());
                continue;
            };
            if !error.is_empty() {
                pending += 1;
                reason.get_or_insert(format!("True API chưa sẵn sàng: {error}"));
                continue;
            }
            let info_gtin = text(&info, &["gtin"]);
            if !info_gtin.is_empty() && info_gtin.trim_start_matches('0') != gtin.trim_start_matches('0') {
                pending += 1;
                reason.get_or_insert("True API trả về KIZ của GTIN khác.".into());
                continue;
            }
            let status = text(&info, &["status"]);
            let status_ex = text(&info, &["statusEx", "status_ex"]);
            if status.eq_ignore_ascii_case("INTRODUCED") {
                introduced += 1;
            } else if status.eq_ignore_ascii_case("APPLIED") {
                if !status_ex.is_empty() && !status_ex.eq_ignore_ascii_case("EMPTY") {
                    pending += 1;
                    reason.get_or_insert(format!("KIZ có trạng thái đặc biệt: {status_ex}"));
                    continue;
                }
                applied += 1;
            } else {
                pending += 1;
                reason.get_or_insert(if status.is_empty() {
                    "KIZ chưa ở trạng thái APPLIED.".into()
                } else {
                    format!("KIZ chưa APPLIED: {status}")
                });
            }
        }
        if seen < batch.len() {
            pending += batch.len() - seen;
            reason.get_or_insert("Chưa thấy đủ mã KIZ trên True API.".into());
        }
    }

    if introduced == codes.len() {
        return Ok(Readiness { ready: false, all_introduced: true, message: None });
    }
    if pending > 0 || applied + introduced != codes.len() || introduced > 0 {
        let msg = format!(
            "True API: {}/{} KIZ sẵn sàng{}{}.{}",
            applied + introduced,
            codes.len(),
            if introduced > 0 { format!(" ({introduced} đã lưu thông)") } else { String::new() },
            if pending > 0 { format!(", {pending} đang chờ") } else { String::new() },
            reason.map(|r| format!(" {r}")).unwrap_or_default()
        );
        return Ok(Readiness { ready: false, all_introduced: false, message: Some(msg) });
    }

    // Card completeness gate.
    let (mark, turn) = {
        let conn = state.db.lock().unwrap();
        zdb::product_flags(&conn, gtin)?
    };
    if mark != Some(true) || turn != Some(true) {
        let mut missing = vec![];
        if mark != Some(true) {
            missing.push("goodMarkFlag");
        }
        if turn != Some(true) {
            missing.push("goodTurnFlag");
        }
        return Ok(Readiness {
            ready: false,
            all_introduced: false,
            message: Some(format!(
                "Thẻ hàng chưa sẵn sàng (thiếu {}). Bổ sung thuộc tính trong Национальный каталог.",
                missing.join(", ")
            )),
        });
    }
    Ok(Readiness { ready: true, all_introduced: false, message: None })
}

/// Build, sign and submit the LP_INTRODUCE_GOODS document. Returns document row id.
pub async fn submit(
    state: &AppState,
    settings: &ZnackSettings,
    order_id: i64,
    gtin: &str,
    codes: &[(i64, String)],
) -> AppResult<i64> {
    let tn_ved = {
        let conn = state.db.lock().unwrap();
        zdb::product_tnved(&conn, gtin)?
    };
    if tn_ved.is_empty() {
        return Err(AppError::Msg("Thiếu mã ТН ВЭД cho GTIN này — đồng bộ lại GTIN.".into()));
    }
    // Participant INN: explicit setting → INN from the auth JWT (derived on login).
    let mut participant = settings.participant_inn.trim().to_string();
    if participant.is_empty() {
        // Ensure we authenticated at least once so the INN is derived.
        let _ = auth::true_api_token(&state.http, &state.znack_tokens, settings).await?;
        participant = state.znack_tokens.participant_inn().unwrap_or_default();
    }
    if participant.is_empty() {
        return Err(AppError::Msg(
            "Không xác định được INN участник từ token — kiểm tra chứng thư đã chọn.".into(),
        ));
    }
    let doc_type = settings.resolved_document_type();
    let doc_number = settings.document_number.trim().to_string();
    let doc_date = settings.document_date.trim().to_string();
    if doc_number.is_empty() || doc_date.is_empty() {
        return Err(AppError::Msg(
            "Cần số + ngày giấy chứng nhận hợp quy (документ соответствия) trong cài đặt.".into(),
        ));
    }
    let producer = or_default(&settings.producer_inn, &participant);
    let owner = or_default(&settings.owner_inn, &participant);

    let products: Vec<Value> = codes
        .iter()
        .map(|(_, raw)| {
            json!({
                "uit_code": cis_for_true_api(raw),
                "tnved_code": tn_ved,
                "certificate_document_data": [{
                    "certificate_type": doc_type,
                    "certificate_number": doc_number,
                    "certificate_date": doc_date,
                }]
            })
        })
        .collect();
    let payload = json!({
        "participant_inn": participant,
        "producer_inn": producer,
        "owner_inn": owner,
        "production_type": "OWN_PRODUCTION",
        "products": products,
    });
    let payload_bytes = serde_json::to_vec(&payload)?;

    let document_row = {
        let conn = state.db.lock().unwrap();
        zdb::create_document(&conn, order_id, &payload.to_string())?
    };

    let signature = {
        let settings = settings.clone();
        let bytes = payload_bytes.clone();
        tauri::async_runtime::spawn_blocking(move || {
            sign::sign(&settings, &bytes, sign::SignMode::Detached)
        })
        .await
        .map_err(|e| AppError::Msg(format!("Lỗi luồng ký: {e}")))?
    };
    let signature = match signature {
        Ok(s) => s,
        Err(e) => {
            let conn = state.db.lock().unwrap();
            let _ = zdb::update_document(&conn, document_row, None, "FAILED", Some(&e.to_string()));
            return Err(e);
        }
    };

    let token = auth::true_api_token(&state.http, &state.znack_tokens, settings).await?;
    let request = json!({
        "document_format": "MANUAL",
        "type": "LP_INTRODUCE_GOODS",
        "product_document": base64::engine::general_purpose::STANDARD.encode(&payload_bytes),
        "signature": sign::to_base64(&signature),
    });
    let result =
        client::create_document(&state.http, &settings.resolved_true_api_base(), &token, &request).await;

    let conn = state.db.lock().unwrap();
    match result {
        Ok(external_id) => {
            zdb::update_document(&conn, document_row, Some(&external_id), "SUBMITTED", None)?;
            zdb::mark_codes_legal(&conn, order_id, legal_status::INTRO_SENT, Some(document_row))?;
            zdb::update_order(&conn, order_id, None, None, Some("INTRO_SENT"), None)?;
            Ok(document_row)
        }
        Err(e) => {
            let status = if matches!(e, AppError::Api { status, .. } if status < 500) {
                "REJECTED"
            } else {
                "FAILED"
            };
            let _ = zdb::update_document(&conn, document_row, None, status, Some(&e.to_string()));
            Err(e)
        }
    }
}

/// Poll: document CHECKED_OK + every CIS INTRODUCED → mark codes IN_CIRCULATION.
pub async fn confirm(
    state: &AppState,
    settings: &ZnackSettings,
    order_id: i64,
    codes: &[(i64, String)],
) -> AppResult<bool> {
    let (document_row, external_id) = {
        let conn = state.db.lock().unwrap();
        match zdb::latest_document(&conn, order_id)? {
            Some((id, Some(ext), _)) if !ext.is_empty() => (id, ext),
            _ => return Err(AppError::Msg("Đơn này chưa có document lưu thông.".into())),
        }
    };
    let token = auth::true_api_token(&state.http, &state.znack_tokens, settings).await?;
    let base = settings.resolved_true_api_base();

    let doc = client::document_info(&state.http, &base, &token, &external_id).await?;
    if !document_checked_ok(&doc, &external_id) {
        return Ok(false);
    }
    let request: Vec<Value> = codes
        .iter()
        .map(|(_, raw)| Value::String(cis_for_true_api(raw)))
        .collect();
    let info = client::cises_info(&state.http, &base, &token, &Value::Array(request)).await?;
    let mut count = 0usize;
    count_introduced(&info, &mut count);
    if count < codes.len() {
        return Ok(false);
    }
    let conn = state.db.lock().unwrap();
    zdb::update_document(&conn, document_row, None, "CHECKED_OK", None)?;
    zdb::mark_codes_legal(&conn, order_id, legal_status::IN_CIRCULATION, None)?;
    zdb::update_order(&conn, order_id, None, None, Some("INTRODUCED"), None)?;
    Ok(true)
}

fn document_checked_ok(value: &Value, external_id: &str) -> bool {
    match value {
        Value::Object(o) => {
            let id = text(value, &["document_id", "documentId", "id"]);
            let status = text(value, &["status", "documentStatus"]);
            if (id.is_empty() || id == external_id) && status.eq_ignore_ascii_case("CHECKED_OK") {
                return true;
            }
            o.values().any(|v| document_checked_ok(v, external_id))
        }
        Value::Array(a) => a.iter().any(|v| document_checked_ok(v, external_id)),
        _ => false,
    }
}

fn count_introduced(value: &Value, count: &mut usize) {
    match value {
        Value::Object(o) => {
            if let Some(Value::String(s)) = o.get("status") {
                if s.eq_ignore_ascii_case("INTRODUCED") {
                    *count += 1;
                    return;
                }
            }
            for v in o.values() {
                count_introduced(v, count);
            }
        }
        Value::Array(a) => {
            for v in a {
                count_introduced(v, count);
            }
        }
        _ => {}
    }
}

fn text(v: &Value, keys: &[&str]) -> String {
    for k in keys {
        match v.get(*k) {
            Some(Value::String(s)) => return s.clone(),
            Some(Value::Number(n)) => return n.to_string(),
            _ => {}
        }
    }
    String::new()
}

fn or_default(value: &str, fallback: &str) -> String {
    let v = value.trim();
    if v.is_empty() { fallback.to_string() } else { v.to_string() }
}
