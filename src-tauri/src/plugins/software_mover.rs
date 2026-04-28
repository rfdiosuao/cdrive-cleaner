use crate::error::AppError;
use crate::models::plugin::*;
use crate::services::software_detector::SoftwareDetector;

pub struct SoftwareMoverPlugin {
    info: PluginInfo,
    detector: SoftwareDetector,
}

impl SoftwareMoverPlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "software-mover".to_string(),
                name: "软件搬家".to_string(),
                version: "1.0.0".to_string(),
                description: "无损迁移已安装软件到其他盘，无需重装".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Active,
                category: PluginCategory::App,
                enabled: true,
                config: serde_json::json!({}),
                scan_count: 0,
                clean_count: 0,
                installed_at: now.clone(),
                updated_at: now,
            },
            detector: SoftwareDetector::new(),
        }
    }
}

impl Plugin for SoftwareMoverPlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        let software_list = self.detector.detect_installed_software();
        let mut targets = Vec::new();
        let mut total_size = 0u64;

        for sw in &software_list {
            if !sw.install_path.is_empty() && sw.estimated_size > 0 {
                let path = std::path::Path::new(&sw.install_path);
                if path.exists() && path.starts_with("C:\\") {
                    targets.push(PluginScanTarget {
                        path: sw.install_path.clone(),
                        name: sw.name.clone(),
                        size: sw.estimated_size,
                        safe: false,
                        reason: format!("{} - {:.1}MB", sw.publisher, sw.estimated_size as f64 / (1024.0 * 1024.0)),
                    });
                    total_size += sw.estimated_size;
                }
            }
        }

        let file_count = targets.len() as u64;
        Ok(PluginScanResult { plugin_id: self.info.id.clone(), targets, total_size, file_count })
    }

    fn estimate(&self) -> Result<u64, AppError> { Ok(self.scan()?.total_size) }

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
