pub mod error;
pub mod models;
pub mod commands;
pub mod services;
pub mod store;
pub mod utils;
pub mod plugins;

use tauri::Manager;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItem};
use std::sync::Arc;

use commands::*;
use services::*;
use store::database::Database;
use store::scan_result_store::ScanResultStore;
use store::config_store::ConfigStore;
use store::audit_logger::AuditLogger;
use utils::logger::init_logger;

pub struct AppState {
    pub database: Arc<Database>,
    pub scan_result_store: Arc<ScanResultStore>,
    pub config_store: Arc<ConfigStore>,
    pub audit_logger: Arc<AuditLogger>,
    pub scan_engine: Arc<scan_engine::ScanEngine>,
    pub evaluation_engine: Arc<evaluation_engine::EvaluationEngine>,
    pub clean_engine: Arc<clean_engine::CleanEngine>,
    pub ai_service: Arc<ai_service::AIService>,
    pub plugin_manager: Arc<plugin_manager::PluginManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logger();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()
                .expect("无法获取应用数据目录");
            std::fs::create_dir_all(&app_data).expect("创建数据目录失败");
            let db_path = app_data.join("cdrive_cleaner.db");

            let database = Arc::new(
                Database::new(&db_path).expect("数据库初始化失败")
            );
            let scan_result_store = Arc::new(ScanResultStore::new(database.clone()));
            let config_store = Arc::new(ConfigStore::new(database.clone()));
            let audit_logger = Arc::new(AuditLogger::new(database.clone()));
            let scan_engine = Arc::new(scan_engine::ScanEngine::new());
            let evaluation_engine = Arc::new(evaluation_engine::EvaluationEngine::new());
            let clean_engine = Arc::new(clean_engine::CleanEngine::new());
            let ai_service = Arc::new(ai_service::AIService::new(models::config::AIConfig::default()));
            let plugin_manager = Arc::new(plugin_manager::PluginManager::new());

            let state = AppState {
                database,
                scan_result_store,
                config_store,
                audit_logger,
                scan_engine,
                evaluation_engine,
                clean_engine,
                ai_service,
                plugin_manager,
            };

            app.manage(state);

            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&quit)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan::scan_start,
            scan::scan_stop,
            scan::scan_progress,
            scan::scan_result,
            clean::clean_preview,
            clean::clean_execute,
            clean::clean_restore,
            migrate::migrate_analyze,
            migrate::migrate_execute,
            migrate::migrate_progress,
            settings::settings_get,
            settings::settings_set,
            settings::ai_config_get,
            settings::ai_config_set,
            settings::ai_test_connection,
            plugin::plugin_list,
            plugin::plugin_enable,
            plugin::plugin_disable,
            system::system_info,
            system::whitelist_manage,
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
