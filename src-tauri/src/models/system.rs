use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub hostname: String,
    pub cpu_name: String,
    pub cpu_cores: u32,
    pub total_memory: u64,
    pub available_memory: u64,
    pub drives: Vec<DriveInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub file_system: String,
    pub drive_type: String,
    pub total_size: u64,
    pub available_space: u64,
    pub used_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub drive: DriveInfo,
    pub usage_percent: f64,
    pub temp_size: u64,
    pub cache_size: u64,
    pub log_size: u64,
    pub recycle_size: u64,
}

impl SystemInfo {
    pub fn c_drive(&self) -> Option<&DriveInfo> {
        self.drives.iter().find(|d| d.letter == "C:")
    }
}
