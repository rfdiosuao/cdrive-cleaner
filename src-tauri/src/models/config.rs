use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudModelProvider {
    OpenAI,
    Anthropic,
    DeepSeek,
    Qwen,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    Local,
    Cloud(CloudModelProvider),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::Local,
            api_key: String::new(),
            model: String::new(),
            base_url: String::new(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub auto_scan_on_start: bool,
    pub scan_mode: String,
    pub clean_confirm_required: bool,
    pub backup_before_clean: bool,
    pub language: String,
    pub theme: String,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            auto_scan_on_start: false,
            scan_mode: "quick".to_string(),
            clean_confirm_required: true,
            backup_before_clean: true,
            language: "zh-CN".to_string(),
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: String,
    pub path: String,
    pub reason: String,
    pub added_at: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistConfig {
    pub paths: Vec<WhitelistEntry>,
    pub global_enabled: bool,
}

impl Default for WhitelistConfig {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            global_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ai: AIConfig,
    pub preferences: UserPreferences,
    pub whitelist: WhitelistConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            preferences: UserPreferences::default(),
            whitelist: WhitelistConfig::default(),
        }
    }
}
