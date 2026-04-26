use tauri::State;

use crate::error::AppError;
use crate::AppState;
use crate::models::plugin::PluginInfo;

#[tauri::command]
pub async fn plugin_list(
    state: State<'_, AppState>,
) -> Result<Vec<PluginInfo>, AppError> {
    Ok(state.plugin_manager.list_plugins().await)
}

#[tauri::command]
pub async fn plugin_enable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.plugin_manager.enable_plugin(&plugin_id).await
}

#[tauri::command]
pub async fn plugin_disable(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.plugin_manager.disable_plugin(&plugin_id).await
}
