use crate::error::AppError;
use crate::models::plugin::*;

pub struct AutoCleanPlugin {
    info: PluginInfo,
    auto_enabled: bool,
    min_free_space_gb: u64,
}

impl AutoCleanPlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "auto-clean".to_string(),
                name: "自动清理".to_string(),
                version: "1.0.0".to_string(),
                description: "定时自动清理，空间不足时触发紧急清理".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Inactive,
                category: PluginCategory::System,
                enabled: false,
                config: serde_json::json!({
                    "schedule_hours": 24,
                    "min_free_space_gb": 10,
                    "auto_enabled": false
                }),
                scan_count: 0,
                clean_count: 0,
                installed_at: now.clone(),
                updated_at: now,
            },
            auto_enabled: false,
            min_free_space_gb: 10,
        }
    }

    pub fn should_trigger_emergency(&self) -> bool {
        use sysinfo::Disks;
        
        let disks = Disks::new_with_refreshed_list();

        for disk in disks.iter() {
            let mount = disk.mount_point().to_str().unwrap_or("");
            if mount == "C:\\" || mount == "C:" {
                let available = disk.available_space();
                let threshold = self.min_free_space_gb * 1024 * 1024 * 1024;
                return available < threshold;
            }
        }
        false
    }
}

impl Plugin for AutoCleanPlugin {
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
        self.auto_enabled = enabled;
        self.info.status = if enabled { PluginStatus::Active } else { PluginStatus::Inactive };
    }
}
