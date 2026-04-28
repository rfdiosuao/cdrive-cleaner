use tauri::State;

use crate::error::AppError;
use crate::store::audit_logger::AuditLogEntry;
use crate::AppState;

#[tauri::command]
pub async fn audit_log_list(
    action_filter: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<AuditLogEntry>, AppError> {
    state.audit_logger.query_logs(
        action_filter.as_deref(),
        limit.unwrap_or(100).min(500),
        offset.unwrap_or(0),
    )
}
