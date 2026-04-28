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

    let safety = SafetyScore {
        overall: if count > 0 { total_score / count as f64 } else { 0.0 },
        categories: vec![],
        warnings: preview.warnings,
        recommendations: vec![],
    };

    let detail = serde_json::json!({
        "taskCount": tasks.len(),
        "totalSize": preview.total_size,
        "totalFiles": preview.total_files,
        "warningCount": safety.warnings.len(),
        "estimatedTimeMs": preview.estimated_time_ms,
        "overallSafety": safety.overall,
    })
    .to_string();

    if let Err(e) = state.audit_logger.log_operation(
        "clean_preview",
        "batch",
        Some(&detail),
        Some(if safety.warnings.is_empty() { "safe" } else { "warning" }),
    ) {
        tracing::warn!("write clean preview audit log failed: {}", e);
    }

    Ok(safety)
}

#[tauri::command]
pub async fn clean_execute(
    tasks: Vec<CleanTask>,
    state: State<'_, AppState>,
) -> Result<Vec<CleanResult>, AppError> {
    let start_detail = serde_json::json!({
        "taskCount": tasks.len(),
        "totalSize": tasks.iter().map(|task| task.size).sum::<u64>(),
        "totalFiles": tasks.iter().map(|task| task.file_count).sum::<u64>(),
        "backupRequired": tasks.iter().filter(|task| task.backup_required).count(),
    })
    .to_string();

    if let Err(e) = state.audit_logger.log_operation(
        "clean_execute_start",
        "batch",
        Some(&start_detail),
        None,
    ) {
        tracing::warn!("write clean start audit log failed: {}", e);
    }

    let task_index: std::collections::HashMap<String, (String, String)> = tasks
        .iter()
        .map(|task| {
            (
                task.id.clone(),
                (task.target_path.clone(), format!("{:?}", task.risk_level)),
            )
        })
        .collect();

    let results = state.clean_engine.execute(tasks).await?;

    for result in &results {
        let task = task_index.get(&result.task_id);
        let target_path = task.map(|(path, _)| path.as_str()).unwrap_or("");
        let risk_level = task.map(|(_, risk)| risk.as_str());
        let detail = serde_json::json!({
            "taskId": result.task_id,
            "success": result.success,
            "freedSpace": result.freed_space,
            "cleanedFiles": result.cleaned_files,
            "failedFiles": result.failed_files,
            "backupPath": result.backup_path,
            "elapsedTimeMs": result.elapsed_time_ms,
            "errors": result.errors,
        })
        .to_string();

        if let Err(e) = state.audit_logger.log_operation(
            "clean_execute_result",
            target_path,
            Some(&detail),
            risk_level,
        ) {
            tracing::warn!("write clean result audit log failed: {}", e);
        }
    }

    let summary_detail = serde_json::json!({
        "taskCount": results.len(),
        "successCount": results.iter().filter(|result| result.success).count(),
        "failedCount": results.iter().filter(|result| !result.success).count(),
        "freedSpace": results.iter().map(|result| result.freed_space).sum::<u64>(),
        "cleanedFiles": results.iter().map(|result| result.cleaned_files).sum::<u64>(),
        "failedFiles": results.iter().map(|result| result.failed_files).sum::<u64>(),
    })
    .to_string();

    if let Err(e) = state.audit_logger.log_operation(
        "clean_execute_complete",
        "batch",
        Some(&summary_detail),
        Some(if results.iter().all(|result| result.success) { "safe" } else { "warning" }),
    ) {
        tracing::warn!("write clean complete audit log failed: {}", e);
    }

    Ok(results)
}

#[tauri::command]
pub async fn clean_restore(
    restore_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let result = state.clean_engine.restore(&restore_id).await;
    let detail = serde_json::json!({
        "restoreId": restore_id,
        "success": result.is_ok(),
        "error": result.as_ref().err().map(|error| error.to_string()),
    })
    .to_string();

    if let Err(e) = state.audit_logger.log_operation(
        "clean_restore",
        "restore_point",
        Some(&detail),
        Some(if result.is_ok() { "safe" } else { "warning" }),
    ) {
        tracing::warn!("write restore audit log failed: {}", e);
    }

    result
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
    let result = state.clean_engine.cancel().await;
    let detail = serde_json::json!({
        "success": result.is_ok(),
        "error": result.as_ref().err().map(|error| error.to_string()),
    })
    .to_string();

    if let Err(e) = state.audit_logger.log_operation(
        "clean_cancel",
        "batch",
        Some(&detail),
        Some(if result.is_ok() { "warning" } else { "error" }),
    ) {
        tracing::warn!("write clean cancel audit log failed: {}", e);
    }

    result
}
