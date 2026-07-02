//! Tauri commands for the Честный ЗНАК integration.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::znack::models::*;
use crate::znack::{db as zdb, pipeline, sign, sync};
use tauri::State;

#[tauri::command]
pub async fn znack_get_settings(state: State<'_, AppState>) -> AppResult<ZnackSettings> {
    let conn = state.db.lock().unwrap();
    zdb::get_settings(&conn)
}

#[tauri::command]
pub async fn znack_save_settings(
    state: State<'_, AppState>,
    settings: ZnackSettings,
) -> AppResult<()> {
    {
        let conn = state.db.lock().unwrap();
        zdb::save_settings(&conn, &settings)?;
    }
    // Credentials may have changed — drop cached tokens.
    state.znack_tokens.invalidate();
    Ok(())
}

/// List valid CryptoPro certificates (with private key) for the signer dropdown.
#[tauri::command]
pub async fn znack_list_certificates(
    state: State<'_, AppState>,
) -> AppResult<Vec<sign::CertificateInfo>> {
    let timeout = {
        let conn = state.db.lock().unwrap();
        zdb::get_settings(&conn)?.resolved_timeout_secs()
    };
    tauri::async_runtime::spawn_blocking(move || sign::list_certificates(timeout))
        .await
        .map_err(|e| AppError::Msg(format!("Lỗi luồng đọc chứng thư: {e}")))?
}

/// Shop categories + genders present in the WB catalog, for the mapping UI.
#[tauri::command]
pub async fn znack_category_genders(
    state: State<'_, AppState>,
) -> AppResult<Vec<CategoryGender>> {
    let conn = state.db.lock().unwrap();
    let rows = crate::db::list_category_genders(&conn)?;
    Ok(rows
        .into_iter()
        .map(|(subject_name, gender, product_count)| CategoryGender {
            subject_name,
            gender,
            product_count,
        })
        .collect())
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryGender {
    pub subject_name: String,
    pub gender: String,
    pub product_count: i64,
}

/// Replace the links between a GTIN and one category (list of genders or all).
#[tauri::command]
pub async fn znack_apply_mapping(
    state: State<'_, AppState>,
    gtin: String,
    subject_name: String,
    genders: Vec<String>,
    all_genders: bool,
) -> AppResult<()> {
    let gtin = normalize_gtin(&gtin)?;
    let conn = state.db.lock().unwrap();
    if !zdb::product_exists(&conn, &gtin)? {
        return Err(AppError::Msg("GTIN chưa có trong danh sách đã đồng bộ.".into()));
    }
    zdb::apply_mapping(&conn, &gtin, &subject_name, &genders, all_genders)
}

/// Sign a small payload end-to-end to verify cryptcp + certificate.
#[tauri::command]
pub async fn znack_test_sign(state: State<'_, AppState>) -> AppResult<String> {
    let settings = {
        let conn = state.db.lock().unwrap();
        zdb::get_settings(&conn)?
    };
    tauri::async_runtime::spawn_blocking(move || sign::test_sign(&settings))
        .await
        .map_err(|e| AppError::Msg(format!("Lỗi luồng ký: {e}")))?
}

#[tauri::command]
pub async fn znack_sync_products(state: State<'_, AppState>) -> AppResult<String> {
    let settings = {
        let conn = state.db.lock().unwrap();
        zdb::get_settings(&conn)?
    };
    let (publishable, skipped) = sync::sync_products(&state, &settings).await?;
    Ok(format!(
        "Đã đồng bộ {publishable} GTIN có thể đặt mã; bỏ qua {skipped} thẻ chưa публикован/kỹ thuật."
    ))
}

#[tauri::command]
pub async fn znack_list_products(state: State<'_, AppState>) -> AppResult<Vec<ZnackProduct>> {
    let conn = state.db.lock().unwrap();
    zdb::list_products(&conn)
}

// --- mapping rules ---------------------------------------------------------

#[tauri::command]
pub async fn znack_list_rules(state: State<'_, AppState>) -> AppResult<Vec<ZnackMappingRule>> {
    let conn = state.db.lock().unwrap();
    zdb::list_rules(&conn)
}

#[tauri::command]
pub async fn znack_save_rule(
    state: State<'_, AppState>,
    gtin: String,
    subject_name: String,
    gender_value: String,
    wildcard_gender: bool,
) -> AppResult<()> {
    let gtin = normalize_gtin(&gtin)?;
    let conn = state.db.lock().unwrap();
    if !zdb::product_exists(&conn, &gtin)? {
        return Err(AppError::Msg("GTIN chưa có trong danh sách đã đồng bộ.".into()));
    }
    zdb::save_rule(&conn, &gtin, &subject_name, &gender_value, wildcard_gender)
}

#[tauri::command]
pub async fn znack_delete_rule(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    zdb::delete_rule(&conn, id)
}

// --- purchase pipeline -------------------------------------------------------

#[tauri::command]
pub async fn znack_buy_kiz(
    state: State<'_, AppState>,
    gtin: String,
    quantity: i64,
) -> AppResult<i64> {
    pipeline::start(&state, &gtin, quantity).await
}

#[tauri::command]
pub async fn znack_list_pipelines(state: State<'_, AppState>) -> AppResult<Vec<ZnackPipeline>> {
    let conn = state.db.lock().unwrap();
    zdb::list_pipelines(&conn, 50)
}

#[tauri::command]
pub async fn znack_abort_pipeline(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    pipeline::abort(&state, id)
}

/// Manually retry a failed pipeline by starting a fresh one for the same GTIN.
#[tauri::command]
pub async fn znack_retry_pipeline(state: State<'_, AppState>, id: i64) -> AppResult<i64> {
    let (gtin, quantity, stage_name) = {
        let conn = state.db.lock().unwrap();
        let p = zdb::find_pipeline(&conn, id)?
            .ok_or_else(|| AppError::Msg("Không tìm thấy phiên mua.".into()))?;
        (p.gtin, p.quantity, p.stage)
    };
    if stage::is_active(&stage_name) {
        return Err(AppError::Msg("Phiên mua này vẫn đang chạy — huỷ trước khi thử lại.".into()));
    }
    pipeline::start(&state, &gtin, quantity).await
}
