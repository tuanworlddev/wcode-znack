//! Tauri commands exposed to the React frontend.

use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::secrets;
use crate::state::AppState;
use crate::wb;
use crate::stores::{self, StoreMeta};
use base64::Engine;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

// --- stores -----------------------------------------------------------------

fn store_info_list(state: &AppState) -> AppResult<Vec<StoreInfo>> {
    let meta = stores::load(&state.app_data_dir);
    let active = state.active_store.lock().unwrap().clone();
    let mut out = Vec::new();
    for s in &meta.stores {
        let has_token = secrets::get_token(&s.id)?
            .filter(|t| !t.trim().is_empty())
            .is_some();
        out.push(StoreInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            active: active.as_deref() == Some(s.id.as_str()),
            has_token,
        });
    }
    Ok(out)
}

#[tauri::command]
pub async fn list_stores(state: State<'_, AppState>) -> AppResult<Vec<StoreInfo>> {
    store_info_list(&state)
}

#[tauri::command]
pub async fn active_store(state: State<'_, AppState>) -> AppResult<Option<StoreInfo>> {
    let active = state.active_store.lock().unwrap().clone();
    Ok(store_info_list(&state)?.into_iter().find(|s| Some(&s.id) == active.as_ref()))
}

/// Open a store's DB and make it the active connection.
fn activate(state: &AppState, id: &str) -> AppResult<()> {
    let path = stores::db_path(&state.app_data_dir, id);
    let conn = db::open(&path)?;
    // Recover KIZ reservations left by an interrupted print of this store.
    let _ = crate::znack::db::release_stale_reservations(&conn);
    *state.db.lock().unwrap() = conn;
    *state.active_store.lock().unwrap() = Some(id.to_string());
    // Each store has its own Znack credentials — drop cached tokens.
    state.znack_tokens.invalidate();
    let mut meta = stores::load(&state.app_data_dir);
    meta.active = Some(id.to_string());
    stores::save(&state.app_data_dir, &meta)?;
    Ok(())
}

#[tauri::command]
pub async fn add_store(state: State<'_, AppState>, name: String) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Msg("Tên cửa hàng không được để trống.".into()));
    }
    let id = stores::new_id();
    let mut meta = stores::load(&state.app_data_dir);
    meta.stores.push(StoreMeta {
        id: id.clone(),
        name: name.to_string(),
    });
    stores::save(&state.app_data_dir, &meta)?;
    activate(&state, &id)?;
    Ok(id)
}

#[tauri::command]
pub async fn switch_store(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let meta = stores::load(&state.app_data_dir);
    if !meta.stores.iter().any(|s| s.id == id) {
        return Err(AppError::Msg("Cửa hàng không tồn tại.".into()));
    }
    activate(&state, &id)
}

#[tauri::command]
pub async fn remove_store(state: State<'_, AppState>, id: String) -> AppResult<()> {
    secrets::delete_token(&id)?;
    let _ = std::fs::remove_file(stores::db_path(&state.app_data_dir, &id));
    let mut meta = stores::load(&state.app_data_dir);
    meta.stores.retain(|s| s.id != id);
    let was_active = meta.active.as_deref() == Some(id.as_str());
    stores::save(&state.app_data_dir, &meta)?;
    if was_active {
        match meta.stores.first().map(|s| s.id.clone()) {
            Some(next) => activate(&state, &next)?,
            None => {
                // No stores left: fall back to a scratch DB.
                let scratch = state.app_data_dir.join("scratch.db");
                *state.db.lock().unwrap() = db::open(&scratch)?;
                *state.active_store.lock().unwrap() = None;
                let mut m = stores::load(&state.app_data_dir);
                m.active = None;
                stores::save(&state.app_data_dir, &m)?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn rename_store(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Msg("Tên cửa hàng không được để trống.".into()));
    }
    let mut meta = stores::load(&state.app_data_dir);
    let s = meta
        .stores
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| AppError::Msg("Cửa hàng không tồn tại.".into()))?;
    s.name = name.to_string();
    stores::save(&state.app_data_dir, &meta)?;
    Ok(())
}

/// Set/replace the API token for a specific store (validated live).
#[tauri::command]
pub async fn set_store_token(
    state: State<'_, AppState>,
    id: String,
    token: String,
) -> AppResult<()> {
    let meta = stores::load(&state.app_data_dir);
    if !meta.stores.iter().any(|s| s.id == id) {
        return Err(AppError::Msg("Cửa hàng không tồn tại.".into()));
    }
    secrets::set_token(&id, token.trim())?;
    wb::validate_with(&state, token.trim()).await
}

// --- token (for the active store) -------------------------------------------

#[tauri::command]
pub async fn set_token(state: State<'_, AppState>, token: String) -> AppResult<()> {
    let sid = state
        .active_store
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| AppError::Msg("Chưa chọn cửa hàng.".into()))?;
    secrets::set_token(&sid, token.trim())?;
    // Validate against the live API so the user gets immediate feedback.
    wb::validate_token(&state).await
}

/// Return the active store's stored token (for display/editing in Settings).
#[tauri::command]
pub async fn get_active_token(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let sid = state.active_store.lock().unwrap().clone();
    match sid {
        Some(id) => secrets::get_token(&id),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn delete_token(state: State<'_, AppState>) -> AppResult<()> {
    let sid = state.active_store.lock().unwrap().clone();
    if let Some(sid) = sid {
        secrets::delete_token(&sid)?;
    }
    Ok(())
}

// --- sync -------------------------------------------------------------------

#[tauri::command]
pub async fn sync_products(state: State<'_, AppState>) -> AppResult<SyncResult> {
    // Full sync from the start, paginating through ALL pages until a page
    // returns fewer than the page size (100). Break on `got < 100` rather than
    // the cursor's `total` field, which is the most reliable end-of-data signal.
    let mut cursor: Option<(String, i64)> = None;
    let mut total = 0i64;
    let mut pages = 0;
    loop {
        let resp = wb::cards_list(&state, cursor.clone()).await?;
        let got = resp.cards.len() as i64;
        {
            let conn = state.db.lock().unwrap();
            // Batch the whole page into ONE transaction so we do a single fsync
            // instead of one per card/size/sku (huge speedup on large catalogs).
            let tx = conn.unchecked_transaction()?;
            for card in &resp.cards {
                db::upsert_card(&tx, card)?;
            }
            tx.commit()?;
        }
        total += got;
        pages += 1;
        // Last page reached.
        if got < 100 || pages > 5000 {
            break;
        }
        match (resp.cursor.updated_at, resp.cursor.nm_id) {
            (Some(u), Some(n)) => cursor = Some((u, n)),
            _ => break, // no cursor to continue with
        }
    }
    let db_total = {
        let conn = state.db.lock().unwrap();
        db::count_products(&conn)?
    };
    Ok(SyncResult {
        count: db_total,
        message: format!("Đã đồng bộ {} sản phẩm ({} trang). Tổng trong máy: {}.", total, pages, db_total),
    })
}

#[tauri::command]
pub async fn sync_orders(state: State<'_, AppState>) -> AppResult<SyncResult> {
    // Resolve seller warehouse ids to names for display.
    let wmap: HashMap<i64, String> = wb::warehouses(&state)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|w| (w.id, w.name))
        .collect();
    let orders = wb::orders_new(&state).await?;
    {
        let conn = state.db.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        for o in &orders {
            let wh = wmap.get(&o.warehouse_id).cloned().unwrap_or_default();
            db::upsert_order(&tx, o, "new", &wh)?;
        }
        tx.commit()?;
    }
    let new_count = {
        let conn = state.db.lock().unwrap();
        db::count_new_orders(&conn)?
    };
    Ok(SyncResult {
        count: orders.len() as i64,
        message: format!(
            "Nhận {} đơn mới từ WB. Tổng đơn 'new' đang chờ: {}.",
            orders.len(),
            new_count
        ),
    })
}

#[tauri::command]
pub async fn sync_supplies(state: State<'_, AppState>) -> AppResult<SyncResult> {
    // Only keep supplies still being prepared (done = false). Closed/handed-over
    // supplies are removed from the local list.
    let mut next = 0i64;
    let mut kept = 0i64;
    loop {
        let resp = wb::supplies_list(&state, next, 1000).await?;
        let got = resp.supplies.len() as i64;
        {
            let conn = state.db.lock().unwrap();
            let tx = conn.unchecked_transaction()?;
            for s in &resp.supplies {
                if s.done {
                    let _ = db::delete_supply(&tx, &s.id);
                } else {
                    db::upsert_supply(&tx, s)?;
                    kept += 1;
                }
            }
            tx.commit()?;
        }
        if got < 1000 {
            break;
        }
        next = resp.next;
    }
    Ok(SyncResult {
        count: kept,
        message: format!("Đã đồng bộ {} supply đang chuẩn bị.", kept),
    })
}

// --- queries ----------------------------------------------------------------

#[tauri::command]
pub async fn list_products(
    state: State<'_, AppState>,
    search: Option<String>,
    categories: Option<Vec<String>>,
    limit: Option<i64>,
) -> AppResult<Vec<ProductRow>> {
    let conn = state.db.lock().unwrap();
    db::list_products(
        &conn,
        &search.unwrap_or_default(),
        &categories.unwrap_or_default(),
        limit.unwrap_or(1000),
    )
}

#[tauri::command]
pub async fn list_categories(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let conn = state.db.lock().unwrap();
    db::list_categories(&conn)
}

#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> AppResult<()> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Msg(format!("Không mở được liên kết: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn list_orders(
    state: State<'_, AppState>,
    status: Option<String>,
    supply_id: Option<String>,
) -> AppResult<Vec<OrderRow>> {
    let conn = state.db.lock().unwrap();
    db::list_orders(
        &conn,
        &status.unwrap_or_default(),
        supply_id.as_deref(),
    )
}

#[tauri::command]
pub async fn order_status_counts(state: State<'_, AppState>) -> AppResult<OrderCounts> {
    let conn = state.db.lock().unwrap();
    Ok(OrderCounts {
        new_orders: db::count_orders_by_status(&conn, "new")?,
        confirm: db::count_orders_by_status(&conn, "confirm")?,
        complete: db::count_orders_by_status(&conn, "complete")?,
    })
}

#[tauri::command]
pub async fn list_supplies(state: State<'_, AppState>) -> AppResult<Vec<SupplyRow>> {
    let conn = state.db.lock().unwrap();
    db::list_supplies(&conn)
}

// --- supply management (writes to WB) --------------------------------------

#[tauri::command]
pub async fn create_supply(state: State<'_, AppState>, name: String) -> AppResult<String> {
    let id = wb::create_supply(&state, name.trim()).await?;
    {
        let conn = state.db.lock().unwrap();
        db::insert_local_supply(&conn, &id, name.trim())?;
    }
    Ok(id)
}

#[tauri::command]
pub async fn add_orders_to_supply(
    state: State<'_, AppState>,
    supply_id: String,
    order_ids: Vec<i64>,
) -> AppResult<()> {
    if order_ids.is_empty() {
        return Err(AppError::Msg("Chưa chọn đơn nào.".into()));
    }
    wb::add_orders_to_supply(&state, &supply_id, &order_ids).await?;
    {
        let conn = state.db.lock().unwrap();
        db::set_orders_supply(&conn, &order_ids, &supply_id, "confirm")?;
    }
    Ok(())
}

// --- printing ---------------------------------------------------------------

/// Fetch a supply's order ids from WB and make sure each order's details exist
/// locally (backfilling from the assembly-orders list when needed). Also marks
/// them as belonging to this supply.
async fn ensure_local_supply_orders(state: &AppState, supply_id: &str) -> AppResult<Vec<i64>> {
    let ids = wb::supply_order_ids(state, supply_id).await?;
    if ids.is_empty() {
        return Ok(ids);
    }
    let missing: Vec<i64> = {
        let conn = state.db.lock().unwrap();
        ids.iter()
            .cloned()
            .filter(|id| db::get_order_core(&conn, *id).is_none())
            .collect()
    };
    if !missing.is_empty() {
        let missing_set: HashSet<i64> = missing.iter().cloned().collect();
        let mut found = 0usize;
        let mut next = 0i64;
        for _ in 0..50 {
            let (orders, nx) = wb::assembly_orders(state, next, 1000).await?;
            if orders.is_empty() {
                break;
            }
            {
                let conn = state.db.lock().unwrap();
                let tx = conn.unchecked_transaction()?;
                for o in &orders {
                    if missing_set.contains(&o.id) {
                        let _ = db::upsert_order(&tx, o, "confirm", "");
                        found += 1;
                    }
                }
                tx.commit()?;
            }
            if found >= missing.len() || nx == 0 {
                break;
            }
            next = nx;
        }
    }
    {
        let conn = state.db.lock().unwrap();
        let _ = db::set_orders_supply(&conn, &ids, supply_id, "confirm");
    }
    Ok(ids)
}

/// Orders inside a supply (for the supply detail page).
#[tauri::command]
pub async fn get_supply_orders(
    state: State<'_, AppState>,
    supply_id: String,
) -> AppResult<Vec<OrderRow>> {
    ensure_local_supply_orders(&state, &supply_id).await?;
    let conn = state.db.lock().unwrap();
    db::list_orders(&conn, "", Some(&supply_id))
}

/// Build the print payload for one supply: for each order, product info for
/// page 1 (Code128 from SKU) plus the WB waybill sticker PNG for page 2.
#[tauri::command]
pub async fn get_supply_print_data(
    state: State<'_, AppState>,
    supply_id: String,
) -> AppResult<SupplyPrintData> {
    let ids = ensure_local_supply_orders(&state, &supply_id).await?;
    if ids.is_empty() {
        return Err(AppError::Msg("Supply này chưa có đơn nào.".into()));
    }

    let stickers = wb::orders_stickers(&state, &ids).await?;
    let sticker_map: HashMap<i64, Sticker> =
        stickers.into_iter().map(|s| (s.order_id, s)).collect();

    // Which orders require a Честный ЗНАК code? Prefer the requiredMeta field
    // stored at sync time; fall back to the WB meta endpoint when unknown.
    let mut sgtin_required: HashMap<i64, bool> = HashMap::new();
    let mut unknown: Vec<i64> = vec![];
    {
        let conn = state.db.lock().unwrap();
        let _ = db::set_orders_supply(&conn, &ids, &supply_id, "confirm");
        for id in &ids {
            match db::order_requires_sgtin(&conn, *id) {
                Some(required) => {
                    sgtin_required.insert(*id, required);
                }
                None => unknown.push(*id),
            }
        }
    }
    if !unknown.is_empty() {
        // WB rejects GET orders/{id}/meta (405), so refresh requiredMeta by
        // re-fetching these orders from the assembly-orders list.
        let unknown_set: HashSet<i64> = unknown.iter().cloned().collect();
        let mut found = 0usize;
        let mut next = 0i64;
        for _ in 0..50 {
            let (orders, nx) = wb::assembly_orders(&state, next, 1000).await?;
            if orders.is_empty() {
                break;
            }
            {
                let conn = state.db.lock().unwrap();
                let tx = conn.unchecked_transaction()?;
                for o in &orders {
                    if unknown_set.contains(&o.id) {
                        let _ = db::upsert_order(&tx, o, "confirm", "");
                        found += 1;
                    }
                }
                tx.commit()?;
            }
            if found >= unknown.len() || nx == 0 {
                break;
            }
            next = nx;
        }
        let conn = state.db.lock().unwrap();
        for id in unknown {
            // Still unknown after the refresh (order no longer listed by WB)
            // → treat as not requiring a code rather than blocking forever.
            let required = db::order_requires_sgtin(&conn, id).unwrap_or(false);
            sgtin_required.insert(id, required);
        }
    }

    let mut out = Vec::with_capacity(ids.len());
    let mut kiz_reserved = false;
    let mut missing: Vec<String> = vec![];
    // One reservation token for the whole print run.
    let kiz_token = format!(
        "print-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        std::process::id()
    );
    {
        let conn = state.db.lock().unwrap();
        for id in &ids {
            let (chrt_id, sku, article, nm_id) =
                db::get_order_core(&conn, *id).unwrap_or((0, String::new(), String::new(), 0));
            let p = db::product_for_order(&conn, chrt_id, &sku);
            let sticker = sticker_map.get(id);

            // Codes are attached only to orders WB actually requires them for.
            let mut kiz_code = String::new();
            if sgtin_required.get(id).copied().unwrap_or(false) {
                let group = if p.gender.is_empty() {
                    p.subject_name.clone()
                } else {
                    format!("{} · {}", p.subject_name, p.gender)
                };
                if p.subject_name.is_empty() {
                    missing.push(format!("Đơn {id}: chưa có thông tin sản phẩm"));
                } else {
                    // Never `?` inside this loop: reserved codes must be
                    // released via the `missing` branch below, not leaked.
                    match crate::znack::db::resolve_gtin(&conn, &p.subject_name, &p.gender)
                        .unwrap_or(None)
                    {
                        Some(gtin) => {
                            // Reuses the code from a previous print of this
                            // order; otherwise reserves a fresh one.
                            match crate::znack::db::code_for_order(&conn, &gtin, *id, &kiz_token) {
                                Ok((raw, newly_reserved)) => {
                                    kiz_code = raw;
                                    kiz_reserved |= newly_reserved;
                                }
                                Err(_) => missing.push(format!("{group}: hết mã KIZ (GTIN {gtin})")),
                            }
                        }
                        None => missing.push(format!("{group}: chưa mapping GTIN")),
                    }
                }
            }

            let kiz_sgtin = if kiz_code.is_empty() {
                String::new()
            } else {
                crate::znack::models::sgtin_display(&kiz_code)
            };
            out.push(PrintOrder {
                order_id: *id,
                barcode: sku,
                title: p.title,
                vendor_code: if p.vendor_code.is_empty() { article } else { p.vendor_code },
                tech_size: p.tech_size,
                color: p.color,
                brand: p.brand,
                subject_name: p.subject_name,
                photo: p.photo,
                nm_id,
                sticker_png: sticker.map(|s| s.file.clone()).unwrap_or_default(),
                part_a: sticker.map(|s| s.part_a.clone()).unwrap_or_default(),
                part_b: sticker.map(|s| s.part_b.clone()).unwrap_or_default(),
                kiz_code,
                kiz_sgtin,
            });
        }

        // WB requires a code but we can't provide one — block printing so the
        // user maps/buys first, and return the reserved codes to the pool.
        if !missing.is_empty() {
            let _ = crate::znack::db::finish_reservation(&conn, &kiz_token, false);
            let total = missing.len();
            let mut unique: Vec<String> = missing;
            unique.sort();
            unique.dedup();
            return Err(AppError::Msg(format!(
                "Không thể in: {total} đơn thiếu mã KIZ bắt buộc — {}",
                unique.join("; ")
            )));
        }
    }
    Ok(SupplyPrintData {
        orders: out,
        kiz_token: kiz_reserved.then_some(kiz_token),
        kiz_missing: 0,
    })
}

/// Product size (SKU) rows for FBO barcode printing.
#[tauri::command]
pub async fn list_sku_items(
    state: State<'_, AppState>,
    search: String,
    categories: Vec<String>,
) -> AppResult<Vec<SkuItem>> {
    let conn = state.db.lock().unwrap();
    db::list_sku_items(&conn, &search, &categories, 2000)
}

/// Reserve KIZ codes for an FBO print job. Items whose category has a mapping
/// MUST have enough codes (otherwise the whole job is blocked and everything
/// is released); items without a mapping print without a KIZ.
#[tauri::command]
pub async fn reserve_fbo_codes(
    state: State<'_, AppState>,
    requests: Vec<FboReserveRequest>,
) -> AppResult<FboReservation> {
    let token = format!(
        "fbo-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        std::process::id()
    );
    let conn = state.db.lock().unwrap();
    let mut codes: Vec<Vec<String>> = Vec::with_capacity(requests.len());
    let mut missing: Vec<String> = vec![];
    let mut reserved_any = false;
    for req in &requests {
        if req.quantity <= 0 || req.subject_name.trim().is_empty() {
            codes.push(vec![]);
            continue;
        }
        match crate::znack::db::resolve_gtin(&conn, &req.subject_name, &req.gender).unwrap_or(None) {
            Some(gtin) => match crate::znack::db::reserve_bulk(&conn, &gtin, req.quantity, &token) {
                Ok(list) => {
                    reserved_any = true;
                    codes.push(list);
                }
                Err(e) => {
                    missing.push(format!("{} · {}: {}", req.subject_name, req.gender, e));
                    codes.push(vec![]);
                }
            },
            None => codes.push(vec![]), // no mapping = no KIZ required for FBO item
        }
    }
    if !missing.is_empty() {
        let _ = crate::znack::db::finish_reservation(&conn, &token, false);
        return Err(AppError::Msg(format!(
            "Không thể in FBO: thiếu mã KIZ — {}",
            missing.join("; ")
        )));
    }
    Ok(FboReservation {
        token: reserved_any.then_some(token),
        codes,
    })
}

/// Push printed Честный ЗНАК codes to WB assembly orders
/// (PUT /api/v3/orders/{id}/meta/sgtin). Returns per-order failures.
#[tauri::command]
pub async fn assign_order_sgtins(
    state: State<'_, AppState>,
    assignments: Vec<SgtinAssignment>,
) -> AppResult<SgtinPushResult> {
    let mut ok = 0i64;
    let mut failed: Vec<String> = vec![];
    for a in &assignments {
        match wb::order_set_sgtin(&state, a.order_id, &a.sgtin).await {
            Ok(()) => ok += 1,
            Err(e) => failed.push(format!("Đơn {}: {}", a.order_id, e)),
        }
    }
    Ok(SgtinPushResult { ok, failed })
}

/// Finish a KIZ print reservation: consume the codes after a successful PDF
/// save, or release them back to the pool when printing failed.
#[tauri::command]
pub async fn finish_kiz_reservation(
    state: State<'_, AppState>,
    token: String,
    consume: bool,
) -> AppResult<i64> {
    let conn = state.db.lock().unwrap();
    crate::znack::db::finish_reservation(&conn, &token, consume)
}

/// Save a base64-encoded PDF (built by the frontend) to the app data dir and
/// open it in the default viewer.
#[tauri::command]
pub async fn save_and_open_pdf(
    app: AppHandle,
    supply_id: String,
    pdf_base64: String,
    suffix: Option<String>,
) -> AppResult<String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(pdf_base64.as_bytes())?;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Msg(format!("Không lấy được thư mục dữ liệu: {e}")))?
        .join("labels");
    std::fs::create_dir_all(&dir)?;
    let safe = supply_id.replace(['/', '\\'], "-");
    let suffix = suffix
        .map(|s| s.replace(['/', '\\'], "-"))
        .filter(|s| !s.is_empty())
        .map(|s| format!("-{s}"))
        .unwrap_or_default();
    let path = dir.join(format!("{}{}.pdf", safe, suffix));
    std::fs::write(&path, &bytes)?;
    let path_str = path.to_string_lossy().to_string();
    app.opener()
        .open_path(path_str.clone(), None::<&str>)
        .map_err(|e| AppError::Msg(format!("Không mở được PDF: {e}")))?;
    Ok(path_str)
}

/// Fetch a remote image (product photo) and return it as a data URL, so the
/// frontend can draw it on a canvas without CORS tainting.
#[tauri::command]
pub async fn fetch_image(state: State<'_, AppState>, url: String) -> AppResult<String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::Msg("URL ảnh không hợp lệ.".into()));
    }
    let resp = state.http.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Msg(format!("Tải ảnh thất bại: HTTP {}", resp.status())));
    }
    let bytes = resp.bytes().await?;
    let mime = match bytes.as_ref() {
        b if b.starts_with(&[0xFF, 0xD8]) => "image/jpeg",
        b if b.starts_with(&[0x89, 0x50, 0x4E, 0x47]) => "image/png",
        b if b.len() > 11 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" => "image/webp",
        _ => "image/jpeg",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
