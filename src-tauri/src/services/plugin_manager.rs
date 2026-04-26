use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::plugin::{PluginInfo, PluginStatus, PluginCategory};

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, PluginInfo>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let default_plugins = Self::default_plugins();
        let mut plugins = HashMap::new();
        for info in default_plugins {
            plugins.insert(info.id.clone(), info);
        }
        Self {
            plugins: Arc::new(RwLock::new(plugins)),
        }
    }

    fn default_plugins() -> Vec<PluginInfo> {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        vec![
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
                id: "system-redundant".to_string(),
                name: "系统冗余清理".to_string(),
                version: "1.0.0".to_string(),
                description: "清理Windows.old、WinSxS冗余、旧还原点等".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Active,
                category: PluginCategory::System,
                enabled: true,
                config: serde_json::json!({}),
                scan_count: 0,
                clean_count: 0,
                installed_at: now.clone(),
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
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
                updated_at: now.clone(),
            },
            PluginInfo {
                id: "auto-clean".to_string(),
                name: "自动清理".to_string(),
                version: "1.0.0".to_string(),
                description: "定时自动清理，空间不足时触发紧急清理".to_string(),
                author: "CDrive Cleaner".to_string(),
                status: PluginStatus::Inactive,
                category: PluginCategory::System,
                enabled: false,
                config: serde_json::json!({}),
                scan_count: 0,
                clean_count: 0,
                installed_at: now.clone(),
                updated_at: now,
            },
        ]
    }

    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.read().await.values().cloned().collect()
    }

    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<(), AppError> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| AppError::PluginError(format!("插件不存在: {}", plugin_id)))?;
        plugin.enabled = true;
        plugin.status = PluginStatus::Active;
        Ok(())
    }

    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<(), AppError> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins
            .get_mut(plugin_id)
            .ok_or_else(|| AppError::PluginError(format!("插件不存在: {}", plugin_id)))?;
        plugin.enabled = false;
        plugin.status = PluginStatus::Inactive;
        Ok(())
    }

    pub async fn register_plugin(&self, info: PluginInfo) -> Result<(), AppError> {
        let mut plugins = self.plugins.write().await;
        plugins.insert(info.id.clone(), info);
        Ok(())
    }

    pub async fn unregister_plugin(&self, plugin_id: &str) -> Result<(), AppError> {
        let mut plugins = self.plugins.write().await;
        plugins
            .remove(plugin_id)
            .ok_or_else(|| AppError::PluginError(format!("插件不存在: {}", plugin_id)))?;
        Ok(())
    }
}
