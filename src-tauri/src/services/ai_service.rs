use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::config::{AIConfig, AIProvider, CloudModelProvider};
use crate::models::scan::ScanResult;

pub struct AIService {
    config: Arc<RwLock<AIConfig>>,
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
        let config = self.config.read().await;
        match &config.provider {
            AIProvider::Local => {
                Ok(true)
            }
            AIProvider::Cloud(_) => {
                if config.api_key.is_empty() {
                    return Err(AppError::AIError("API密钥不能为空".to_string()));
                }
                if config.base_url.is_empty() && config.model.is_empty() {
                    return Err(AppError::AIError("请配置API地址和模型".to_string()));
                }

                self.test_cloud_connection(&config).await
            }
        }
    }

    async fn test_cloud_connection(&self, config: &AIConfig) -> Result<bool, AppError> {
        let url = format!("{}/models", config.base_url.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let mut request = client.get(&url);

        request = request.header("Authorization", format!("Bearer {}", config.api_key));

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

    pub async fn analyze_files(&self, file_paths: &[String]) -> Result<Vec<ScanResult>, AppError> {
        let config = self.config.read().await;
        match &config.provider {
            AIProvider::Local => {
                Ok(vec![])
            }
            AIProvider::Cloud(provider) => {
                self.cloud_analyze(provider, &config, file_paths).await
            }
        }
    }

    async fn cloud_analyze(
        &self,
        provider: &CloudModelProvider,
        config: &AIConfig,
        file_paths: &[String],
    ) -> Result<Vec<ScanResult>, AppError> {
        let prompt = format!(
            "分析以下Windows文件路径，判断是否可以安全清理。返回JSON数组，每个元素包含path、safe(true/false)、reason字段：\n{}",
            file_paths.join("\n")
        );

        let url = self.build_api_url(provider, &config.base_url, &config.model);

        let body = self.build_request_body(provider, &prompt, config);

        let client = reqwest::Client::new();
        let mut request = client.post(&url);

        request = request.header("Authorization", format!("Bearer {}", config.api_key));
        request = request.header("Content-Type", "application/json");

        let response = request
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::AIError(format!("云端AI请求失败: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| AppError::AIError(format!("读取响应失败: {}", e)))?;

        let _ = response_text;

        Ok(vec![])
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
                } else {
                    format!("{}/messages", base_url.trim_end_matches('/'))
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

    pub async fn suggest_migration(&self, software_name: &str) -> Result<Vec<String>, AppError> {
        let config = self.config.read().await;
        match &config.provider {
            AIProvider::Local => {
                Ok(vec![])
            }
            AIProvider::Cloud(_) => {
                let prompt = format!(
                    "分析软件\"{}\"是否可以安全迁移到其他盘，以及迁移时需要注意的依赖项和注册表项。返回JSON格式的分析结果。",
                    software_name
                );

                let _ = prompt;
                Ok(vec![])
            }
        }
    }

    pub async fn smart_switch(&self, file_count: usize, avg_path_length: usize) -> AIProvider {
        let config = self.config.read().await;

        if file_count < 100 && avg_path_length < 50 {
            return AIProvider::Local;
        }

        match &config.provider {
            AIProvider::Local => AIProvider::Local,
            AIProvider::Cloud(p) => AIProvider::Cloud(p.clone()),
        }
    }
}
