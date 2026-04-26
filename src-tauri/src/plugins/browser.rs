use crate::error::AppError;
use crate::models::plugin::*;
use crate::utils::file_utils::calc_dir_size;
use crate::services::software_detector::SoftwareDetector;

pub struct BrowserPlugin {
    info: PluginInfo,
    detector: SoftwareDetector,
}

impl BrowserPlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "browser".to_string(),
                name: "浏览器清理".to_string(),
                version: "1.0.0".to_string(),
                description: "清理Chrome、Edge、Firefox等浏览器缓存".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Active,
                category: PluginCategory::Browser,
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

impl Plugin for BrowserPlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        let all: Vec<_> = self.detector.detect_chrome_cache()
            .into_iter()
            .chain(self.detector.detect_edge_cache())
            .chain(self.detector.detect_firefox_cache())
            .collect();

        let mut targets = Vec::new();
        let mut total_size = 0u64;
        let mut file_count = 0u64;

        for info in &all {
            let path = std::path::Path::new(&info.cache_path);
            if path.exists() {
                let mut dir_size = 0u64;
                let mut dir_files = 0u64;
                calc_dir_size(path, &mut dir_size, &mut dir_files);
                targets.push(PluginScanTarget {
                    path: info.cache_path.clone(),
                    name: format!("{}缓存", info.software_name),
                    size: dir_size,
                    safe: true,
                    reason: info.description.clone(),
                });
                total_size += dir_size;
                file_count += dir_files;
            }
        }

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
                match std::fs::remove_dir_all(path) {
                    Ok(_) => { freed += target.size; cleaned += 1; }
                    Err(e) => errors.push(format!("清理失败: {} - {}", target.path, e)),
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
