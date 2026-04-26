use tauri::State;

use crate::error::AppError;
use crate::AppState;
use crate::models::scan::{ScanMode, ScanProgress, ScanResult};

#[tauri::command]
pub async fn scan_start(
    mode: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if state.scan_engine.is_scanning().await {
        return Err(AppError::ScanError("扫描正在进行中".to_string()));
    }

    let scan_mode = match mode.as_str() {
        "quick" => ScanMode::Quick,
        "deep" => ScanMode::Deep,
        _ => ScanMode::Quick,
    };

    let engine_clone = state.scan_engine.clone();
    tokio::spawn(async move {
        if let Err(e) = engine_clone.start_scan(scan_mode).await {
            tracing::error!("扫描失败: {}", e);
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn scan_stop(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.scan_engine.stop_scan().await
}

#[tauri::command]
pub async fn scan_progress(
    state: State<'_, AppState>,
) -> Result<ScanProgress, AppError> {
    Ok(state.scan_engine.get_progress().await)
}

#[tauri::command]
pub async fn scan_result(
    state: State<'_, AppState>,
) -> Result<Vec<ScanResult>, AppError> {
    Ok(state.scan_engine.get_results().await)
}
