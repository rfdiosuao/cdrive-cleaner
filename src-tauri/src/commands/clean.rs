use tauri::State;

use crate::error::AppError;
use crate::AppState;
use crate::models::clean::{CleanProgress, CleanResult, CleanTask, SafetyScore};

#[tauri::command]
pub async fn clean_preview(
    tasks: Vec<CleanTask>,
    state: State<'_, AppState>,
) -> Result<SafetyScore, AppError> {
    if tasks.is_empty() {
        return Err(AppError::CleanError("清理任务列表不能为空".to_string()));
    }

    let preview = state.clean_engine.preview(&tasks).await?;

    let mut total_score = 0.0;
    let mut count = 0;
    for task in &tasks {
        let score = state.evaluation_engine.evaluate_task(task).await?;
        total_score += score.overall;
        count += 1;
    }
    Ok(SafetyScore {
        overall: if count > 0 { total_score / count as f64 } else { 0.0 },
        categories: vec![],
        warnings: preview.warnings,
        recommendations: vec![],
    })
}

#[tauri::command]
pub async fn clean_execute(
    tasks: Vec<CleanTask>,
    state: State<'_, AppState>,
) -> Result<Vec<CleanResult>, AppError> {
    state.clean_engine.execute(tasks).await
}

#[tauri::command]
pub async fn clean_restore(
    restore_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.clean_engine.restore(&restore_id).await
}

#[tauri::command]
pub async fn clean_progress(
    state: State<'_, AppState>,
) -> Result<Option<CleanProgress>, AppError> {
    Ok(state.clean_engine.get_progress().await)
}

#[tauri::command]
pub async fn clean_stop(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.clean_engine.cancel().await
}
