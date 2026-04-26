use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::config::{AIConfig, AIProvider, CloudModelProvider};
use crate::models::scan::{FileMetadata, RiskLevel, ScanCategory};

use serde::{Deserialize, Serialize};

pub struct AIService {
    config: Arc<RwLock<AIConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIFileAnalysis {
    pub path: String,
    pub category: ScanCategory,
    pub safety_score: f64,
    pub risk_level: RiskLevel,
    pub safe_to_clean: bool,
    pub reason: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AIFileInput<'a> {
    path: &'a str,
    name: &'a str,
    size: u64,
    extension: &'a str,
    modified_at: &'a str,
    current_category: &'a ScanCategory,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AIFileAnalysisRaw {
    path: String,
    category: Option<String>,
    safety_score: Option<f64>,
    risk_level: Option<String>,
    safe_to_clean: Option<bool>,
    reason: Option<String>,
    recommendation: Option<String>,
}

impl AIService {
    pub fn new(config: AIConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_config(&self) -> AIConfig {
        self.config.read().await.clone()
    }

    pub async fn set_config(&self, config: AIConfig) {
        let mut c = self.config.write().await;
        *c = config;
    }

    pub async fn test_connection(&self) -> Result<bool, AppError> {
        let config = self.config.read().await.clone();
        if !config.provider.is_cloud() {
            return Ok(true);
        }

        if config.api_key.is_empty() {
            return Err(AppError::AIError("API密钥不能为空".to_string()));
        }
        if config.model.is_empty() {
            return Err(AppError::AIError("请配置模型名称".to_string()));
        }
        if matches!(&config.provider, AIProvider::Custom) && config.base_url.is_empty() {
            return Err(AppError::AIError("请配置自定义API地址".to_string()));
        }

        match config.provider.as_cloud_model_provider() {
            Some(provider) => self.test_cloud_connection(&provider, &config).await,
            None => Ok(true),
        }
    }

    async fn test_cloud_connection(
        &self,
        provider: &CloudModelProvider,
        config: &AIConfig,
    ) -> Result<bool, AppError> {
        let url = self.build_models_url(provider, &config.base_url);

        let client = reqwest::Client::new();
        let request = self.apply_auth_headers(client.get(&url), provider, config);

        match request.timeout(std::time::Duration::from_secs(10)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(true)
                } else {
                    Err(AppError::AIError(format!(
                        "连接测试失败: HTTP {}",
                        response.status()
                    )))
                }
            }
            Err(e) => Err(AppError::AIError(format!("连接测试失败: {}", e))),
        }
    }

    pub async fn analyze_files(
        &self,
        files: &[FileMetadata],
    ) -> Result<Vec<AIFileAnalysis>, AppError> {
        let config = self.config.read().await.clone();
        match config.provider.as_cloud_model_provider() {
            Some(provider) => self.cloud_analyze(&provider, &config, files).await,
            None => Ok(vec![]),
        }
    }

    async fn cloud_analyze(
        &self,
        provider: &CloudModelProvider,
        config: &AIConfig,
        files: &[FileMetadata],
    ) -> Result<Vec<AIFileAnalysis>, AppError> {
        if files.is_empty() {
            return Ok(vec![]);
        }

        let file_inputs: Vec<AIFileInput<'_>> = files
            .iter()
            .map(|file| AIFileInput {
                path: &file.path,
                name: &file.name,
                size: file.size,
                extension: &file.extension,
                modified_at: &file.modified_at,
                current_category: &file.category,
            })
            .collect();

        let file_json = serde_json::to_string(&file_inputs)
            .map_err(|e| AppError::AIError(format!("构造AI请求失败: {}", e)))?;

        let prompt = format!(
            concat!(
                "你正在为 Windows C 盘清理工具做文件识别。请根据路径、文件名、扩展名、大小、修改时间和当前规则分类，",
                "判断这些文件更像哪一类，清理是否安全。\n\n",
                "只返回 JSON，不要 Markdown，不要解释性文本。返回值必须是数组，每个元素格式为：",
                "{{\"path\":\"原路径\",\"category\":\"temp|cache|log|download|recycle|browser|system|app|other\",",
                "\"safetyScore\":0到100的数字,\"riskLevel\":\"safe|low|medium|high|critical\",",
                "\"safeToClean\":true或false,\"reason\":\"简短原因\",\"recommendation\":\"给用户的简短建议\"}}。\n\n",
                "安全原则：系统目录、程序安装目录、用户文档、数据库、配置、证书、可执行文件、卸载器、驱动、服务文件默认不要标为安全清理；",
                "临时文件、浏览器缓存、更新下载缓存、日志、回收站内容在证据充分时可以标为更安全。\n\n",
                "待识别文件 JSON：\n{}"
            ),
            file_json
        );

        let url = self.build_api_url(provider, &config.base_url, &config.model);

        let body = self.build_request_body(provider, &prompt, config);

        let client = reqwest::Client::new();
        let request = self
            .apply_auth_headers(client.post(&url), provider, config)
            .header("Content-Type", "application/json");

        let response = request
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::AIError(format!("云端AI请求失败: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::AIError(format!("读取响应失败: {}", e)))?;

        if !status.is_success() {
            return Err(AppError::AIError(format!(
                "云端AI请求失败: HTTP {} {}",
                status,
                Self::truncate_for_error(&response_text)
            )));
        }

        let content = self.extract_message_content(provider, &response_text)?;
        self.parse_analysis_response(&content)
    }

    fn build_api_url(&self, provider: &CloudModelProvider, base_url: &str, _model: &str) -> String {
        match provider {
            CloudModelProvider::OpenAI => {
                if base_url.is_empty() {
                    format!("https://api.openai.com/v1/chat/completions")
                } else {
                    format!("{}/chat/completions", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Anthropic => {
                if base_url.is_empty() {
                    "https://api.anthropic.com/v1/messages".to_string()
                } else if base_url.trim_end_matches('/').ends_with("/v1") {
                    format!("{}/messages", base_url.trim_end_matches('/'))
                } else {
                    format!("{}/v1/messages", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::DeepSeek => {
                if base_url.is_empty() {
                    "https://api.deepseek.com/v1/chat/completions".to_string()
                } else {
                    format!("{}/chat/completions", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Qwen => {
                if base_url.is_empty() {
                    "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".to_string()
                } else {
                    format!("{}/chat/completions", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Custom => {
                format!("{}/chat/completions", base_url.trim_end_matches('/'))
            }
        }
    }

    fn build_models_url(&self, provider: &CloudModelProvider, base_url: &str) -> String {
        match provider {
            CloudModelProvider::OpenAI => {
                if base_url.is_empty() {
                    "https://api.openai.com/v1/models".to_string()
                } else {
                    format!("{}/models", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Anthropic => {
                if base_url.is_empty() {
                    "https://api.anthropic.com/v1/models".to_string()
                } else if base_url.trim_end_matches('/').ends_with("/v1") {
                    format!("{}/models", base_url.trim_end_matches('/'))
                } else {
                    format!("{}/v1/models", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::DeepSeek => {
                if base_url.is_empty() {
                    "https://api.deepseek.com/v1/models".to_string()
                } else {
                    format!("{}/models", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Qwen => {
                if base_url.is_empty() {
                    "https://dashscope.aliyuncs.com/compatible-mode/v1/models".to_string()
                } else {
                    format!("{}/models", base_url.trim_end_matches('/'))
                }
            }
            CloudModelProvider::Custom => {
                format!("{}/models", base_url.trim_end_matches('/'))
            }
        }
    }

    fn apply_auth_headers(
        &self,
        request: reqwest::RequestBuilder,
        provider: &CloudModelProvider,
        config: &AIConfig,
    ) -> reqwest::RequestBuilder {
        match provider {
            CloudModelProvider::Anthropic => request
                .header("x-api-key", config.api_key.clone())
                .header("anthropic-version", "2023-06-01"),
            _ => request.header("Authorization", format!("Bearer {}", config.api_key)),
        }
    }

    fn build_request_body(
        &self,
        provider: &CloudModelProvider,
        prompt: &str,
        config: &AIConfig,
    ) -> serde_json::Value {
        match provider {
            CloudModelProvider::Anthropic => {
                serde_json::json!({
                    "model": config.model,
                    "max_tokens": config.max_tokens,
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                })
            }
            _ => {
                serde_json::json!({
                    "model": config.model,
                    "temperature": config.temperature,
                    "max_tokens": config.max_tokens,
                    "messages": [
                        {
                            "role": "system",
                            "content": "你是一个Windows系统文件安全分析专家，负责判断文件是否可以安全清理。"
                        },
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                })
            }
        }
    }

    fn extract_message_content(
        &self,
        provider: &CloudModelProvider,
        response_text: &str,
    ) -> Result<String, AppError> {
        let value: serde_json::Value = serde_json::from_str(response_text)
            .map_err(|e| AppError::AIError(format!("AI响应不是有效JSON: {}", e)))?;

        match provider {
            CloudModelProvider::Anthropic => {
                let text = value
                    .get("content")
                    .and_then(|v| v.as_array())
                    .map(|blocks| {
                        blocks
                            .iter()
                            .filter_map(|block| {
                                let block_type = block.get("type").and_then(|v| v.as_str());
                                if block_type == Some("text") {
                                    block.get("text").and_then(|v| v.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .filter(|s| !s.trim().is_empty());

                text.ok_or_else(|| AppError::AIError("AI响应中没有文本内容".to_string()))
            }
            _ => value
                .get("choices")
                .and_then(|v| v.as_array())
                .and_then(|choices| choices.first())
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
                .and_then(|content| content.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::AIError("AI响应中没有choices[0].message.content".to_string())),
        }
    }

    fn parse_analysis_response(&self, content: &str) -> Result<Vec<AIFileAnalysis>, AppError> {
        let json = Self::extract_json_payload(content)?;

        let raw_items: Vec<AIFileAnalysisRaw> = match serde_json::from_str(&json) {
            Ok(items) => items,
            Err(array_error) => {
                let value: serde_json::Value = serde_json::from_str(&json)
                    .map_err(|e| AppError::AIError(format!("解析AI识别结果失败: {}; {}", array_error, e)))?;
                let items = value
                    .get("files")
                    .or_else(|| value.get("results"))
                    .or_else(|| value.get("items"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| AppError::AIError("AI识别结果不是数组".to_string()))?;
                serde_json::from_value(serde_json::Value::Array(items.clone()))
                    .map_err(|e| AppError::AIError(format!("解析AI识别结果失败: {}", e)))?
            }
        };

        let analyses = raw_items
            .into_iter()
            .filter_map(Self::normalize_analysis)
            .collect();

        Ok(analyses)
    }

    fn normalize_analysis(raw: AIFileAnalysisRaw) -> Option<AIFileAnalysis> {
        let path = raw.path.trim().to_string();
        if path.is_empty() {
            return None;
        }

        let safety_score = raw.safety_score.unwrap_or_else(|| {
            if raw.safe_to_clean.unwrap_or(false) {
                80.0
            } else {
                35.0
            }
        }).clamp(0.0, 100.0);

        let risk_level = raw
            .risk_level
            .as_deref()
            .map(Self::parse_risk_level)
            .unwrap_or_else(|| Self::risk_from_score(safety_score));

        let safe_to_clean = raw.safe_to_clean.unwrap_or_else(|| {
            safety_score >= 70.0 && matches!(&risk_level, RiskLevel::Safe | RiskLevel::Low)
        });

        Some(AIFileAnalysis {
            path,
            category: raw
                .category
                .as_deref()
                .map(Self::parse_category)
                .unwrap_or(ScanCategory::Other),
            safety_score,
            risk_level,
            safe_to_clean,
            reason: raw.reason.unwrap_or_else(|| "AI未提供具体原因".to_string()),
            recommendation: raw
                .recommendation
                .unwrap_or_else(|| Self::recommendation_from_score(safety_score).to_string()),
        })
    }

    fn extract_json_payload(content: &str) -> Result<String, AppError> {
        let mut text = content.trim().to_string();

        if text.starts_with("```") {
            let without_opening = text.lines().skip(1).collect::<Vec<_>>().join("\n");
            text = if let Some(end) = without_opening.rfind("```") {
                without_opening[..end].trim().to_string()
            } else {
                without_opening.trim().to_string()
            };
        }

        if serde_json::from_str::<serde_json::Value>(&text).is_ok() {
            return Ok(text);
        }

        if let (Some(start), Some(end)) = (text.find('['), text.rfind(']')) {
            if start <= end {
                return Ok(text[start..=end].to_string());
            }
        }

        if let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) {
            if start <= end {
                return Ok(text[start..=end].to_string());
            }
        }

        Err(AppError::AIError("AI响应中没有可解析的JSON".to_string()))
    }

    fn parse_category(value: &str) -> ScanCategory {
        let normalized = value
            .trim()
            .to_lowercase()
            .replace('-', "_")
            .replace(' ', "_");

        match normalized.as_str() {
            "temp" | "temporary" | "temporary_file" | "temporary_files" | "临时文件" => {
                ScanCategory::Temp
            }
            "cache" | "cache_file" | "cache_files" | "缓存" | "缓存文件" => ScanCategory::Cache,
            "log" | "logs" | "log_file" | "log_files" | "日志" | "日志文件" => ScanCategory::Log,
            "download" | "downloads" | "下载" | "下载文件" => ScanCategory::Download,
            "recycle" | "recycle_bin" | "trash" | "回收站" => ScanCategory::Recycle,
            "browser" | "browser_cache" | "浏览器" | "浏览器缓存" => ScanCategory::Browser,
            "system" | "system_file" | "system_files" | "系统" | "系统文件" => ScanCategory::System,
            "app" | "application" | "app_data" | "应用" | "应用数据" => ScanCategory::App,
            _ => ScanCategory::Other,
        }
    }

    fn parse_risk_level(value: &str) -> RiskLevel {
        match value.trim().to_lowercase().replace('-', "_").as_str() {
            "safe" | "安全" => RiskLevel::Safe,
            "low" | "低" | "低风险" => RiskLevel::Low,
            "medium" | "中" | "中等" | "中等风险" => RiskLevel::Medium,
            "high" | "高" | "高风险" => RiskLevel::High,
            "critical" | "极高" | "严重" | "极高风险" => RiskLevel::Critical,
            _ => RiskLevel::Medium,
        }
    }

    fn risk_from_score(score: f64) -> RiskLevel {
        if score >= 80.0 {
            RiskLevel::Safe
        } else if score >= 60.0 {
            RiskLevel::Low
        } else if score >= 40.0 {
            RiskLevel::Medium
        } else if score >= 20.0 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }

    fn recommendation_from_score(score: f64) -> &'static str {
        if score >= 80.0 {
            "AI识别为安全，建议清理"
        } else if score >= 60.0 {
            "AI识别为低风险，可以清理"
        } else if score >= 40.0 {
            "AI识别为中等风险，建议确认后再清理"
        } else {
            "AI识别为高风险，不建议清理"
        }
    }

    fn truncate_for_error(text: &str) -> String {
        const LIMIT: usize = 500;
        let trimmed = text.trim();
        if trimmed.chars().count() <= LIMIT {
            return trimmed.to_string();
        }

        let mut output = trimmed.chars().take(LIMIT).collect::<String>();
        output.push_str("...");
        output
    }

    pub async fn suggest_migration(&self, software_name: &str) -> Result<Vec<String>, AppError> {
        let config = self.config.read().await;
        if !config.provider.is_cloud() {
            return Ok(vec![]);
        }

        let prompt = format!(
            "分析软件\"{}\"是否可以安全迁移到其他盘，以及迁移时需要注意的依赖项和注册表项。返回JSON格式的分析结果。",
            software_name
        );

        let _ = prompt;
        Ok(vec![])
    }

    pub async fn smart_switch(&self, file_count: usize, avg_path_length: usize) -> AIProvider {
        let config = self.config.read().await;

        if file_count < 100 && avg_path_length < 50 {
            return AIProvider::Local;
        }

        config.provider.clone()
    }
}
