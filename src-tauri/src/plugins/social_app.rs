use std::path::PathBuf;

use crate::error::AppError;
use crate::models::plugin::*;
use crate::utils::file_utils::calc_dir_size;
use crate::services::software_detector::SoftwareDetector;

pub struct SocialAppPlugin {
    info: PluginInfo,
    detector: SoftwareDetector,
}

impl SocialAppPlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "social-app".to_string(),
                name: "社交软件清理".to_string(),
                version: "1.0.0".to_string(),
                description: "清理微信、QQ、企业微信等聊天缓存".to_string(),
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

impl Plugin for SocialAppPlugin {
    fn info(&self) -> &PluginInfo { &self.info }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        let cache_infos = self.detector.detect_wechat_paths();
        let cache_infos2 = self.detector.detect_qq_paths();
        let cache_infos3 = self.detector.detect_wechat_work_paths();
        let all: Vec<_> = cache_infos.into_iter().chain(cache_infos2).chain(cache_infos3).collect();

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
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            if let Ok(_) = std::fs::remove_dir_all(&p) {
                                freed += target.size;
                                cleaned += 1;
                            }
                        }
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
