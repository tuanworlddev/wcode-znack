mod commands;
mod config;
mod db;
mod error;
mod models;
mod secrets;
mod state;
mod stores;
mod wb;
mod znack;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(dir.join("stores"))?;

            // Open the active store's DB, or a scratch DB if no store selected.
            let meta = stores::load(&dir);
            let active = meta
                .active
                .clone()
                .filter(|id| meta.stores.iter().any(|s| &s.id == id));
            let db_file = match &active {
                Some(id) => stores::db_path(&dir, id),
                None => dir.join("scratch.db"),
            };
            let conn =
                db::open(&db_file).map_err(|e| format!("Không mở được cơ sở dữ liệu: {e}"))?;

            app.manage(AppState::new(conn, active, dir));
            // Resume KIZ purchase pipelines and release stale reservations.
            tauri::async_runtime::spawn(znack::pipeline::run_poller(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_stores,
            commands::add_store,
            commands::switch_store,
            commands::remove_store,
            commands::rename_store,
            commands::active_store,
            commands::set_token,
            commands::set_store_token,
            commands::get_active_token,
            commands::delete_token,
            commands::sync_products,
            commands::sync_orders,
            commands::sync_supplies,
            commands::list_products,
            commands::list_categories,
            commands::open_url,
            commands::list_orders,
            commands::order_status_counts,
            commands::list_supplies,
            commands::create_supply,
            commands::add_orders_to_supply,
            commands::get_supply_orders,
            commands::get_supply_print_data,
            commands::save_and_open_pdf,
            commands::fetch_image,
            commands::finish_kiz_reservation,
            commands::assign_order_sgtins,
            commands::list_sku_items,
            commands::reserve_fbo_codes,
            znack::commands::znack_get_settings,
            znack::commands::znack_save_settings,
            znack::commands::znack_test_sign,
            znack::commands::znack_list_certificates,
            znack::commands::znack_category_genders,
            znack::commands::znack_apply_mapping,
            znack::commands::znack_sync_products,
            znack::commands::znack_list_products,
            znack::commands::znack_list_rules,
            znack::commands::znack_save_rule,
            znack::commands::znack_delete_rule,
            znack::commands::znack_buy_kiz,
            znack::commands::znack_list_pipelines,
            znack::commands::znack_abort_pipeline,
            znack::commands::znack_retry_pipeline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
