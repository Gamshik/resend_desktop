mod commands;
mod db;
mod error;
mod keystore;
mod resend;

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let conn = db::open(&dir.join("resend-desktop.sqlite3"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
                http: resend::client(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::key_status,
            commands::save_key,
            commands::check_key,
            commands::delete_key,
            commands::send_email,
            commands::list_history,
            commands::delete_history_item,
            commands::clear_history,
            commands::get_settings,
            commands::set_default_from,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
