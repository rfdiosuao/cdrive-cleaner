use serde::{Deserialize, Serialize};

use super::scan::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanAction {
    Delete,
    Move,
    Compress,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanTask {
    pub id: String,
    pub target_id: String,
    pub target_path: String,
    pub target_name: String,
    pub action: CleanAction,
    pub priority: CleanPriority,
    pub risk_level: RiskLevel,
    pub size: u64,
    pub file_count: u64,
    pub backup_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanError {
    pub file_path: String,
    pub error_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub task_id: String,
    pub success: bool,
    pub freed_space: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub backup_path: String,
    pub elapsed_time_ms: u64,
    pub errors: Vec<CleanError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyCategory {
    pub name: String,
    pub score: f64,
    pub level: RiskLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyScore {
    pub overall: f64,
    pub categories: Vec<SafetyCategory>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanPhase {
    Preparing,
    BackingUp,
    Cleaning,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgress {
    pub task_id: String,
    pub current_file: String,
    pub completed_files: u64,
    pub total_files: u64,
    pub freed_space: u64,
    pub percent: f64,
    pub phase: CleanPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePoint {
    pub id: String,
    pub created_at: String,
    pub tasks: Vec<CleanTask>,
    pub total_size: u64,
    pub description: String,
}
