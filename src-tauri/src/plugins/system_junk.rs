use std::path::PathBuf;

use crate::error::AppError;
use crate::models::plugin::*;
use crate::services::safety_guard::SafetyGuard;
use crate::utils::file_utils::calc_dir_size;

pub struct SystemJunkPlugin {
    info: PluginInfo,
    safety_guard: SafetyGuard,
}

impl SystemJunkPlugin {
    pub fn new() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            info: PluginInfo {
                id: "system-junk".to_string(),
                name: "系统垃圾清理".to_string(),
                version: "1.0.0".to_string(),
                description: "清理临时文件、日志、更新缓存、缩略图等系统垃圾".to_string(),
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
            safety_guard: SafetyGuard::new(),
        }
    }

    fn get_scan_paths(&self) -> Vec<(String, PathBuf)> {
        let user = std::env::var("USERPROFILE").unwrap_or_default();
        let win = std::env::var("WINDIR").unwrap_or_default();

        vec![
            ("用户临时文件".to_string(), PathBuf::from(format!(r"{}\AppData\Local\Temp", user))),
            ("系统临时文件".to_string(), PathBuf::from(format!(r"{}\Temp", win))),
            ("Windows更新缓存".to_string(), PathBuf::from(format!(r"{}\SoftwareDistribution\Download", win))),
            ("预读取文件".to_string(), PathBuf::from(format!(r"{}\Prefetch", win))),
            ("缩略图缓存".to_string(), PathBuf::from(format!(r"{}\AppData\Local\Microsoft\Windows\Explorer", user))),
            ("IE缓存".to_string(), PathBuf::from(format!(r"{}\AppData\Local\Microsoft\Windows\INetCache", user))),
            ("Windows日志".to_string(), PathBuf::from(format!(r"{}\Logs", win))),
            ("旧Windows日志".to_string(), PathBuf::from(format!(r"{}\Panther", win))),
        ]
    }
}

impl Plugin for SystemJunkPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    fn scan(&self) -> Result<PluginScanResult, AppError> {
        let mut targets = Vec::new();
        let mut total_size = 0u64;
        let mut file_count = 0u64;

        for (name, path) in self.get_scan_paths() {
            if path.exists() {
                let mut dir_size = 0u64;
                let mut dir_files = 0u64;
                calc_dir_size(&path, &mut dir_size, &mut dir_files);

                let safe = self.safety_guard.check_path_safety(path.to_str().unwrap_or("")).is_safe;

                targets.push(PluginScanTarget {
                    path: path.to_str().unwrap_or("").to_string(),
                    name: name.clone(),
                    size: dir_size,
                    safe,
                    reason: if safe { "安全可清理".to_string() } else { "受保护目录".to_string() },
                });

                total_size += dir_size;
                file_count += dir_files;
            }
        }

        Ok(PluginScanResult {
            plugin_id: self.info.id.clone(),
            targets,
            total_size,
            file_count,
        })
    }

    fn estimate(&self) -> Result<u64, AppError> {
        let result = self.scan()?;
        Ok(result.total_size)
    }

    fn clean(&self, targets: &[PluginScanTarget]) -> Result<PluginCleanResult, AppError> {
        let mut freed = 0u64;
        let mut cleaned = 0u64;
        let mut errors = Vec::new();

        for target in targets {
            if !target.safe {
                continue;
            }
            let path = std::path::Path::new(&target.path);
            if path.exists() {
                match std::fs::remove_dir_all(path) {
                    Ok(_) => {
                        freed += target.size;
                        cleaned += 1;
                    }
                    Err(e) => errors.push(format!("清理失败: {} - {}", target.path, e)),
                }
            }
        }

        Ok(PluginCleanResult {
            plugin_id: self.info.id.clone(),
            success: errors.is_empty(),
            freed_space: freed,
            cleaned_files: cleaned,
            errors,
        })
    }

    fn enabled(&self) -> bool {
        self.info.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.info.enabled = enabled;
        self.info.status = if enabled { PluginStatus::Active } else { PluginStatus::Inactive };
    }
}
