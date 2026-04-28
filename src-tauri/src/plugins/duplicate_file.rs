use crate::error::AppError;
use crate::models::plugin::*;

pub struct DuplicateFilePlugin {
    info: PluginInfo,
}

impl DuplicateFilePlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "duplicate-file".to_string(),
                name: "重复文件清理".to_string(),
                version: "1.0.0".to_string(),
                description: "识别重复文件、相似图片，智能推荐删除".to_string(),
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
        }
    }
}

impl Plugin for DuplicateFilePlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        Ok(PluginScanResult {
            plugin_id: self.info.id.clone(),
            targets: vec![],
            total_size: 0,
            file_count: 0,
        })
    }

    fn estimate(&self) -> Result<u64, AppError> { Ok(0) }

    fn clean(&self, targets: &[PluginScanTarget]) -> Result<PluginCleanResult, AppError> {
        let mut freed = 0u64;
        let mut cleaned = 0u64;
        let mut errors = Vec::new();

        for target in targets {
            if !target.safe { continue; }
            let path = std::path::Path::new(&target.path);
            if path.exists() {
                match std::fs::remove_file(path) {
                    Ok(_) => { freed += target.size; cleaned += 1; }
                    Err(e) => errors.push(format!("删除失败: {} - {}", target.path, e)),
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
