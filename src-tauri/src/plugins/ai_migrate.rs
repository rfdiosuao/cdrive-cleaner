use crate::error::AppError;
use crate::models::plugin::*;

pub struct AiMigratePlugin {
    info: PluginInfo,
}

impl AiMigratePlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "ai-migrate".to_string(),
                name: "AI智能迁移".to_string(),
                version: "1.0.0".to_string(),
                description: "AI自动识别可迁移文件，无损迁移到其他盘".to_string(),
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

impl Plugin for AiMigratePlugin {
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

    fn clean(&self, _targets: &[PluginScanTarget]) -> Result<PluginCleanResult, AppError> {
        Ok(PluginCleanResult {
            plugin_id: self.info.id.clone(),
            success: true,
            freed_space: 0,
            cleaned_files: 0,
            errors: vec![],
        })
    }

    fn enabled(&self) -> bool { self.info.enabled }
    fn set_enabled(&mut self, enabled: bool) {
        self.info.enabled = enabled;
        self.info.status = if enabled { PluginStatus::Active } else { PluginStatus::Inactive };
    }
}
