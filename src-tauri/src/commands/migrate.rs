use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateProgress {
    pub task_id: String,
    pub percent: f64,
    pub phase: String,
    pub current_file: String,
}

#[tauri::command]
pub async fn migrate_analyze(
    software_name: String,
) -> Result<Vec<String>, AppError> {
    if software_name.is_empty() {
        return Err(AppError::MigrateError("软件名称不能为空".to_string()));
    }
    Ok(vec![])
}

#[tauri::command]
pub async fn migrate_execute(
    software_name: String,
    target_path: String,
) -> Result<(), AppError> {
    if software_name.is_empty() {
        return Err(AppError::MigrateError("软件名称不能为空".to_string()));
    }
    if target_path.is_empty() {
        return Err(AppError::MigrateError("目标路径不能为空".to_string()));
    }
    Ok(())
}

#[tauri::command]
pub async fn migrate_progress(
    task_id: String,
) -> Result<MigrateProgress, AppError> {
    Err(AppError::MigrateError(format!("迁移任务不存在: {}", task_id)))
}
