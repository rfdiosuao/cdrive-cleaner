use tauri::State;

use crate::error::AppError;
use crate::AppState;
use crate::models::config::{AIConfig, UserPreferences};

#[tauri::command]
pub async fn settings_get(
    state: State<'_, AppState>,
) -> Result<UserPreferences, AppError> {
    let config = state.config_store.get_config().await;
    Ok(config.preferences)
}

#[tauri::command]
pub async fn settings_set(
    prefs: UserPreferences,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.config_store.save_preferences(&prefs).await
}

#[tauri::command]
pub async fn ai_config_get(
    state: State<'_, AppState>,
) -> Result<AIConfig, AppError> {
    Ok(state.ai_service.get_config().await)
}

#[tauri::command]
pub async fn ai_config_set(
    config: AIConfig,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.ai_service.set_config(config).await;
    Ok(())
}

#[tauri::command]
pub async fn ai_test_connection(
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    state.ai_service.test_connection().await
}
