use serde::{Deserialize, Serialize};

use super::scan::RiskLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Active,
    Inactive,
    Error,
    Loading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    Browser,
    System,
    App,
    Development,
    Gaming,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub status: PluginStatus,
    pub category: PluginCategory,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub scan_count: u64,
    pub clean_count: u64,
    pub installed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: PluginCategory,
    pub entry_point: String,
    pub permissions: Vec<String>,
    pub min_app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanTarget {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub safe: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScanResult {
    pub plugin_id: String,
    pub targets: Vec<PluginScanTarget>,
    pub total_size: u64,
    pub file_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCleanResult {
    pub plugin_id: String,
    pub success: bool,
    pub freed_space: u64,
    pub cleaned_files: u64,
    pub errors: Vec<String>,
}

pub trait Plugin: Send + Sync {
    fn info(&self) -> &PluginInfo;
    fn scan(&self) -> Result<PluginScanResult, crate::error::AppError>;
    fn estimate(&self) -> Result<u64, crate::error::AppError>;
    fn clean(&self, targets: &[PluginScanTarget]) -> Result<PluginCleanResult, crate::error::AppError>;
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}
