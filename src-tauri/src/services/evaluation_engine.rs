use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::clean::{CleanPriority, CleanTask, SafetyScore, SafetyCategory};
use crate::models::config::{AIConfig, AIProvider};
use crate::models::scan::{RiskLevel, ScanCategory, ScanResult, FileMetadata};

pub struct EvaluationEngine {
    cache: Arc<RwLock<lru::LruCache<String, SafetyScore>>>,
    ai_config: Arc<RwLock<AIConfig>>,
    behavior_weights: Arc<RwLock<BehaviorWeights>>,
}

#[derive(Debug, Clone)]
struct BehaviorWeights {
    path_weight: f64,
    type_weight: f64,
    time_weight: f64,
    frequency_weight: f64,
    signature_weight: f64,
}

impl Default for BehaviorWeights {
    fn default() -> Self {
        Self {
            path_weight: 0.25,
            type_weight: 0.30,
            time_weight: 0.20,
            frequency_weight: 0.15,
            signature_weight: 0.10,
        }
    }
}

impl EvaluationEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                NonZeroUsize::new(1024).unwrap(),
            ))),
            ai_config: Arc::new(RwLock::new(AIConfig::default())),
            behavior_weights: Arc::new(RwLock::new(BehaviorWeights::default())),
        }
    }

    pub async fn evaluate_safety(&self, results: &[ScanResult]) -> Result<SafetyScore, AppError> {
        let cache_key = results
            .iter()
            .map(|r| r.id.clone())
            .collect::<Vec<_>>()
            .join("|");

        {
            let mut cache = self.cache.write().await;
            if let Some(score) = cache.get(&cache_key) {
                return Ok(score.clone());
            }
        }

        let mut categories = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        let mut total_score = 0.0;
        let mut count = 0;

        for result in results {
            let score = self.calculate_composite_score(result).await;
            let level = self.score_to_risk_level(score);

            categories.push(SafetyCategory {
                name: result.target.name.clone(),
                score,
                level: level.clone(),
                description: self.risk_description(&level),
            });

            if score < 50.0 {
                warnings.push(format!("「{}」存在较高风险，建议谨慎操作", result.target.name));
            }

            total_score += score;
            count += 1;
        }

        if count > 0 {
            total_score /= count as f64;
        }

        if total_score < 70.0 {
            recommendations.push("建议仅清理标记为安全的项".to_string());
        }
        if total_score < 40.0 {
            recommendations.push("部分项目风险较高，建议创建还原点后再操作".to_string());
        }

        let safety_score = SafetyScore {
            overall: total_score,
            categories,
            warnings,
            recommendations,
        };

        {
            let mut cache = self.cache.write().await;
            cache.put(cache_key, safety_score.clone());
        }

        Ok(safety_score)
    }

    pub async fn evaluate_task(&self, task: &CleanTask) -> Result<SafetyScore, AppError> {
        let level = task.risk_level.clone();
        let score = self.risk_level_to_score(&level);

        Ok(SafetyScore {
            overall: score,
            categories: vec![SafetyCategory {
                name: task.target_name.clone(),
                score,
                level,
                description: String::new(),
            }],
            warnings: if score < 50.0 {
                vec![format!("「{}」风险较高", task.target_name)]
            } else {
                vec![]
            },
            recommendations: vec![],
        })
    }

    async fn calculate_composite_score(&self, result: &ScanResult) -> f64 {
        let weights = self.behavior_weights.read().await;

        let path_score = self.score_by_path(&result.target.path);
        let type_score = self.score_by_category(&result.target.category);
        let time_score = self.score_by_time(&result.scanned_at);
        let frequency_score = self.score_by_frequency(&result.target.path);
        let signature_score = self.score_by_signature(&result.target.path);

        let composite = (path_score * weights.path_weight
            + type_score * weights.type_weight
            + time_score * weights.time_weight
            + frequency_score * weights.frequency_weight
            + signature_score * weights.signature_weight)
            .min(100.0);

        let ai_adjusted = self.apply_ai_adjustment(composite, result).await;
        ai_adjusted
    }

    fn score_by_path(&self, path: &str) -> f64 {
        let dangerous_paths = [
            r"C:\Windows",
            r"C:\Program Files",
            r"C:\Program Files (x86)",
            r"C:\ProgramData",
        ];
        for dp in &dangerous_paths {
            if path.starts_with(dp) {
                return 10.0;
            }
        }

        let safe_paths = [
            r"\Temp",
            r"\Cache",
            r"\tmp",
            r"\Recycle",
            r"INetCache",
            r"SoftwareDistribution\Download",
        ];
        for sp in &safe_paths {
            if path.contains(sp) {
                return 95.0;
            }
        }

        60.0
    }

    fn score_by_category(&self, category: &ScanCategory) -> f64 {
        match category {
            ScanCategory::Temp => 95.0,
            ScanCategory::Cache => 90.0,
            ScanCategory::Log => 85.0,
            ScanCategory::Recycle => 95.0,
            ScanCategory::Browser => 88.0,
            ScanCategory::Download => 60.0,
            ScanCategory::System => 15.0,
            ScanCategory::App => 50.0,
            ScanCategory::Other => 40.0,
        }
    }

    fn score_by_time(&self, scanned_at: &str) -> f64 {
        if let Ok(scanned) = chrono::NaiveDateTime::parse_from_str(scanned_at, "%Y-%m-%d %H:%M:%S") {
            let now = chrono::Local::now().naive_local();
            let diff = now - scanned;
            let days = diff.num_days();
            if days > 90 {
                return 90.0;
            } else if days > 30 {
                return 75.0;
            } else if days > 7 {
                return 60.0;
            }
        }
        50.0
    }

    fn score_by_frequency(&self, _path: &str) -> f64 {
        70.0
    }

    fn score_by_signature(&self, path: &str) -> f64 {
        let verifier = crate::services::signature_verifier::SignatureVerifier::new();
        if verifier.is_system_file(path) || verifier.is_system_extension(
            &std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        ) {
            return 10.0;
        }
        80.0
    }

    async fn apply_ai_adjustment(&self, base_score: f64, result: &ScanResult) -> f64 {
        let config = self.ai_config.read().await;
        match &config.provider {
            AIProvider::Local => base_score,
            AIProvider::Cloud(_) => {
                let _ = result;
                base_score
            }
        }
    }

    pub async fn calculate_priority(&self, result: &ScanResult) -> f64 {
        let safety = self.calculate_composite_score(result).await / 100.0;
        let size_factor = (result.total_size as f64).log2().max(0.0) / 30.0;
        let idle_factor = self.score_by_time(&result.scanned_at) / 100.0;

        (size_factor * safety * idle_factor * 100.0).min(100.0)
    }

    pub async fn prioritize_results(&self, results: Vec<ScanResult>) -> Vec<PrioritizedResult> {
        let mut prioritized: Vec<PrioritizedResult> = Vec::new();

        for result in results {
            let priority = self.calculate_priority(&result).await;
            let clean_priority = if priority >= 80.0 {
                CleanPriority::Critical
            } else if priority >= 60.0 {
                CleanPriority::High
            } else if priority >= 30.0 {
                CleanPriority::Normal
            } else {
                CleanPriority::Low
            };

            prioritized.push(PrioritizedResult {
                result,
                priority_score: priority,
                clean_priority,
            });
        }

        prioritized.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap_or(std::cmp::Ordering::Equal));
        prioritized
    }

    pub async fn update_behavior_weights(&self, action_type: &str, _category: &str, was_cleaned: bool) {
        let mut weights = self.behavior_weights.write().await;
        let adjustment = if was_cleaned { 0.01 } else { -0.01 };

        match action_type {
            "path" => weights.path_weight = (weights.path_weight + adjustment).max(0.05).min(0.50),
            "type" => weights.type_weight = (weights.type_weight + adjustment).max(0.05).min(0.50),
            "time" => weights.time_weight = (weights.time_weight + adjustment).max(0.05).min(0.50),
            "frequency" => weights.frequency_weight = (weights.frequency_weight + adjustment).max(0.05).min(0.50),
            "signature" => weights.signature_weight = (weights.signature_weight + adjustment).max(0.05).min(0.50),
            _ => {}
        }

        let total = weights.path_weight + weights.type_weight + weights.time_weight
            + weights.frequency_weight + weights.signature_weight;
        weights.path_weight /= total;
        weights.type_weight /= total;
        weights.time_weight /= total;
        weights.frequency_weight /= total;
        weights.signature_weight /= total;
    }

    pub async fn set_ai_config(&self, config: AIConfig) {
        let mut c = self.ai_config.write().await;
        *c = config;
    }

    pub async fn get_ai_config(&self) -> AIConfig {
        self.ai_config.read().await.clone()
    }

    fn score_to_risk_level(&self, score: f64) -> RiskLevel {
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

    fn risk_level_to_score(&self, level: &RiskLevel) -> f64 {
        match level {
            RiskLevel::Safe => 90.0,
            RiskLevel::Low => 70.0,
            RiskLevel::Medium => 50.0,
            RiskLevel::High => 30.0,
            RiskLevel::Critical => 10.0,
        }
    }

    fn risk_description(&self, level: &RiskLevel) -> String {
        match level {
            RiskLevel::Safe => "安全，可以放心清理".to_string(),
            RiskLevel::Low => "低风险，建议清理".to_string(),
            RiskLevel::Medium => "中等风险，建议谨慎操作".to_string(),
            RiskLevel::High => "高风险，不建议清理".to_string(),
            RiskLevel::Critical => "极高风险，请勿清理".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrioritizedResult {
    pub result: ScanResult,
    pub priority_score: f64,
    pub clean_priority: CleanPriority,
}
