use std::path::PathBuf;

use crate::error::AppError;
use crate::models::plugin::*;

pub struct LargeFilePlugin {
    info: PluginInfo,
    size_threshold: u64,
}

impl LargeFilePlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "large-file".to_string(),
                name: "大文件分析".to_string(),
                version: "1.0.0".to_string(),
                description: "定位100MB以上大文件，支持智能筛选".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Active,
                category: PluginCategory::System,
                enabled: true,
                config: serde_json::json!({}),
                scan_count: 0,
                clean_count: 0,
                installed_at: now.clone(),
                updated_at: now,
            },
            size_threshold: 100 * 1024 * 1024,
        }
    }

    pub fn with_threshold(threshold_mb: u64) -> Self {
        let mut plugin = Self::new();
        plugin.size_threshold = threshold_mb * 1024 * 1024;
        plugin
    }
}

impl Plugin for LargeFilePlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        let mut targets = Vec::new();
        let mut total_size = 0u64;
        let mut file_count = 0u64;

        let c_drive = PathBuf::from(r"C:\");
        self.scan_large_files(&c_drive, &mut targets, &mut total_size, &mut file_count);

        Ok(PluginScanResult { plugin_id: self.info.id.clone(), targets, total_size, file_count })
    }

    fn estimate(&self) -> Result<u64, AppError> { Ok(self.scan()?.total_size) }

    fn clean(&self, targets: &[PluginScanTarget]) -> Result<PluginCleanResult, AppError> {
        let mut freed = 0u64;
        let mut cleaned = 0u64;
        let mut errors = Vec::new();

        for target in targets {
            if !target.safe { continue; }
            let path = std::path::Path::new(&target.path);
            if path.exists() {
                if path.is_dir() {
                    match std::fs::remove_dir_all(path) {
                        Ok(_) => { freed += target.size; cleaned += 1; }
                        Err(e) => errors.push(format!("清理失败: {} - {}", target.path, e)),
                    }
                } else {
                    match std::fs::remove_file(path) {
                        Ok(_) => { freed += target.size; cleaned += 1; }
                        Err(e) => errors.push(format!("删除失败: {} - {}", target.path, e)),
                    }
                }
            }
        }

        Ok(PluginCleanResult { plugin_id: self.info.id.clone(), success: errors.is_empty(), freed_space: freed, cleaned_files: cleaned, errors })
    }

    fn enabled(&self) -> bool { self.info.enabled }
    fn set_enabled(&mut self, enabled: bool) {
        self.info.enabled = enabled;
        self.info.status = if enabled { PluginStatus::Active } else { PluginStatus::Inactive };
    }
}

impl LargeFilePlugin {
    fn scan_large_files(&self, dir: &std::path::Path, targets: &mut Vec<PluginScanTarget>, total_size: &mut u64, file_count: &mut u64) {
        let excluded = [
            r"C:\Windows",
            r"C:\Program Files",
            r"C:\Program Files (x86)",
            r"C:\ProgramData",
        ];

        if excluded.iter().any(|e| dir.starts_with(e)) {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        self.scan_large_files(&path, targets, total_size, file_count);
                    } else if meta.len() >= self.size_threshold {
                        let name = path.file_name().unwrap_or_default().to_str().unwrap_or("").to_string();
                        let size = meta.len();
                        targets.push(PluginScanTarget {
                            path: path.to_str().unwrap_or("").to_string(),
                            name,
                            size,
                            safe: false,
                            reason: format!("大文件({:.1}MB)", size as f64 / (1024.0 * 1024.0)),
                        });
                        *total_size += size;
                        *file_count += 1;
                    }
                }
            }
        }
    }
}
