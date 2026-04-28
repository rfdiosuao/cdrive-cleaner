use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanMode {
    Quick,
    Deep,
    Custom(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanCategory {
    Temp,
    Cache,
    Log,
    Download,
    Recycle,
    Browser,
    System,
    App,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTarget {
    pub id: String,
    pub path: String,
    pub name: String,
    pub category: ScanCategory,
    pub size: u64,
    pub file_count: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub created_at: String,
    pub is_directory: bool,
    pub extension: String,
    pub category: ScanCategory,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanPhase {
    Initializing,
    Scanning,
    Analyzing,
    Evaluating,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub phase: ScanPhase,
    pub current_path: String,
    pub scanned_files: u64,
    pub total_files: u64,
    pub scanned_size: u64,
    pub elapsed_time_ms: u64,
    pub estimated_time_remaining_ms: u64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub id: String,
    pub target: ScanTarget,
    pub files: Vec<FileMetadata>,
    pub total_size: u64,
    pub file_count: u64,
    pub safety_score: f64,
    pub risk_level: RiskLevel,
    pub recommendation: String,
    pub scanned_at: String,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            phase: ScanPhase::Initializing,
            current_path: String::new(),
            scanned_files: 0,
            total_files: 0,
            scanned_size: 0,
            elapsed_time_ms: 0,
            estimated_time_remaining_ms: 0,
            percent: 0.0,
        }
    }
}
