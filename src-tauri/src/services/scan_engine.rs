use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};

use crate::error::AppError;
use crate::models::scan::{
    FileMetadata, RiskLevel, ScanCategory, ScanMode, ScanPhase, ScanProgress, ScanResult, ScanTarget,
};
use crate::services::ai_service::{AIFileAnalysis, AIService};
use crate::services::file_walker::{FileWalker, WalkConfig};
use crate::services::usn_scanner::UsnScanner;

const AI_SAMPLE_PER_RESULT: usize = 25;
const AI_MAX_SAMPLE_FILES: usize = 160;

#[derive(Clone)]
pub struct ScanEngine {
    progress: Arc<RwLock<ScanProgress>>,
    results: Arc<RwLock<Vec<ScanResult>>>,
    is_scanning: Arc<RwLock<bool>>,
    cancel_token: Arc<RwLock<bool>>,
    usn_scanner: Arc<UsnScanner>,
    scan_history: Arc<RwLock<Vec<ScanResult>>>,
}

impl ScanEngine {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(RwLock::new(ScanProgress::default())),
            results: Arc::new(RwLock::new(Vec::new())),
            is_scanning: Arc::new(RwLock::new(false)),
            cancel_token: Arc::new(RwLock::new(false)),
            usn_scanner: Arc::new(UsnScanner::new("C")),
            scan_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_scan(
        &self,
        mode: ScanMode,
        ai_service: Option<Arc<AIService>>,
    ) -> Result<(), AppError> {
        let mut scanning = self.is_scanning.write().await;
        if *scanning {
            return Err(AppError::ScanError("扫描正在进行中".to_string()));
        }
        *scanning = true;
        drop(scanning);

        let mut cancel = self.cancel_token.write().await;
        *cancel = false;
        drop(cancel);

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Initializing;
        progress.percent = 0.0;
        progress.scanned_files = 0;
        progress.total_files = 0;
        progress.scanned_size = 0;
        progress.elapsed_time_ms = 0;
        drop(progress);

        let result = match &mode {
            ScanMode::Quick => self.quick_scan().await,
            ScanMode::Deep => self.deep_scan().await,
            ScanMode::Custom(paths) => self.custom_scan(paths).await,
        };

        if let Err(e) = result {
            let mut progress = self.progress.write().await;
            progress.phase = ScanPhase::Error;
            drop(progress);

            let mut scanning = self.is_scanning.write().await;
            *scanning = false;

            return Err(e);
        }

        if !*self.cancel_token.read().await {
            if let Some(ai_service) = ai_service {
                if let Err(e) = self.apply_ai_analysis(&ai_service).await {
                    tracing::warn!("AI识别失败，保留规则扫描结果: {}", e);
                }
            }
        }

        let mut progress = self.progress.write().await;
        if *self.cancel_token.read().await {
            progress.phase = ScanPhase::Cancelled;
        } else {
            progress.phase = ScanPhase::Completed;
            progress.percent = 100.0;
        }
        drop(progress);

        let mut scanning = self.is_scanning.write().await;
        *scanning = false;

        Ok(())
    }

    pub async fn start_scan_with_progress<F>(
        &self,
        mode: ScanMode,
        on_progress: F,
    ) -> Result<(), AppError>
    where
        F: Fn(ScanProgress) + Send + Sync + 'static,
    {
        let on_progress = Arc::new(on_progress);

        let mut scanning = self.is_scanning.write().await;
        if *scanning {
            return Err(AppError::ScanError("扫描正在进行中".to_string()));
        }
        *scanning = true;
        drop(scanning);

        let mut cancel = self.cancel_token.write().await;
        *cancel = false;
        drop(cancel);

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Initializing;
        progress.percent = 0.0;
        progress.scanned_files = 0;
        progress.total_files = 0;
        progress.scanned_size = 0;
        drop(progress);

        let result = match &mode {
            ScanMode::Quick => self.quick_scan().await,
            ScanMode::Deep => self.deep_scan().await,
            ScanMode::Custom(paths) => self.custom_scan(paths).await,
        };

        if let Err(e) = &result {
            let mut progress = self.progress.write().await;
            progress.phase = ScanPhase::Error;
            let mut scanning = self.is_scanning.write().await;
            *scanning = false;
            return Err(e.clone());
        }

        let current = self.progress.read().await.clone();
        on_progress(current);

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Completed;
        progress.percent = 100.0;
        drop(progress);

        let mut scanning = self.is_scanning.write().await;
        *scanning = false;

        Ok(())
    }

    pub async fn incremental_scan(&self) -> Result<Vec<FileMetadata>, AppError> {
        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Scanning;
        progress.current_path = "USN Journal".to_string();
        drop(progress);

        let start = Instant::now();
        let changed = self.usn_scanner.get_changed_files().await?;
        let elapsed = start.elapsed().as_millis() as u64;

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Completed;
        progress.percent = 100.0;
        progress.elapsed_time_ms = elapsed;
        progress.scanned_files = changed.len() as u64;
        drop(progress);

        Ok(changed)
    }

    async fn quick_scan(&self) -> Result<(), AppError> {
        let start = Instant::now();
        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Scanning;
        drop(progress);

        let quick_paths = self.get_quick_scan_paths();
        let cpu_count = num_cpus::get();

        let (tx, mut rx) = mpsc::channel::<Vec<FileMetadata>>(256);
        let _cancel = self.cancel_token.clone();
        let excluded = Vec::new();

        let walker = FileWalker::new(cpu_count * 4, 100);
        let cancel_clone = self.cancel_token.clone();

        let walk_handle = tokio::spawn(async move {
            let mut total = 0u64;
            for path in &quick_paths {
                if *cancel_clone.read().await {
                    break;
                }
                if path.exists() {
                    let result = walker
                        .walk_directory(
                            path,
                            tx.clone(),
                            cancel_clone.clone(),
                            &excluded,
                        )
                        .await;
                    if let Ok(count) = result {
                        total += count;
                    }
                }
            }
            drop(tx);
            total
        });

        let mut all_files: Vec<FileMetadata> = Vec::new();
        let mut total_scanned = 0u64;
        let mut total_size = 0u64;

        while let Some(batch) = rx.recv().await {
            if *self.cancel_token.read().await {
                break;
            }
            let batch_size: u64 = batch.len() as u64;
            let batch_total_size: u64 = batch.iter().map(|f| f.size).sum();
            total_scanned += batch_size;
            total_size += batch_total_size;

            let mut progress = self.progress.write().await;
            progress.scanned_files = total_scanned;
            progress.scanned_size = total_size;
            progress.elapsed_time_ms = start.elapsed().as_millis() as u64;
            let elapsed_secs = start.elapsed().as_secs_f64();
            let estimated_total_secs = 15.0;
            progress.percent = ((elapsed_secs / estimated_total_secs) * 90.0).min(95.0);
            drop(progress);

            all_files.extend(batch);
        }

        let total = walk_handle.await.unwrap_or(0);

        let results = self.categorize_results(all_files, start.elapsed().as_millis() as u64).await;
        self.set_results(results.clone()).await;

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Analyzing;
        progress.total_files = total;
        progress.elapsed_time_ms = start.elapsed().as_millis() as u64;
        drop(progress);

        Ok(())
    }

    async fn deep_scan(&self) -> Result<(), AppError> {
        let start = Instant::now();
        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Scanning;
        drop(progress);

        let c_drive = PathBuf::from(r"C:\");
        let cpu_count = num_cpus::get();

        let (tx, mut rx) = mpsc::channel::<Vec<FileMetadata>>(512);
        let excluded = WalkConfig::default().excluded_dirs;

        let walker = FileWalker::new(cpu_count * 8, 200);
        let cancel_clone = self.cancel_token.clone();

        let walk_handle = tokio::spawn(async move {
            let result = walker
                .walk_directory(&c_drive, tx, cancel_clone, &excluded)
                .await;
            result.unwrap_or(0)
        });

        let mut all_files: Vec<FileMetadata> = Vec::new();
        let mut total_scanned = 0u64;
        let mut total_size = 0u64;

        while let Some(batch) = rx.recv().await {
            if *self.cancel_token.read().await {
                break;
            }
            let batch_size: u64 = batch.len() as u64;
            let batch_total_size: u64 = batch.iter().map(|f| f.size).sum();
            total_scanned += batch_size;
            total_size += batch_total_size;

            let mut progress = self.progress.write().await;
            progress.scanned_files = total_scanned;
            progress.scanned_size = total_size;
            progress.elapsed_time_ms = start.elapsed().as_millis() as u64;
            drop(progress);

            all_files.extend(batch);
        }

        let total = walk_handle.await.unwrap_or(0);

        let results = self.categorize_results(all_files, start.elapsed().as_millis() as u64).await;
        self.set_results(results.clone()).await;

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Evaluating;
        progress.total_files = total;
        drop(progress);

        self.evaluate_safety_scores().await?;

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Evaluating;
        progress.percent = 95.0;
        progress.elapsed_time_ms = start.elapsed().as_millis() as u64;
        drop(progress);

        Ok(())
    }

    async fn custom_scan(&self, paths: &[String]) -> Result<(), AppError> {
        let start = Instant::now();
        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Scanning;
        drop(progress);

        let cpu_count = num_cpus::get();

        let (tx, mut rx) = mpsc::channel::<Vec<FileMetadata>>(256);
        let excluded = vec![];

        let walker = FileWalker::new(cpu_count * 4, 100);
        let cancel_clone = self.cancel_token.clone();
        let custom_paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

        let walk_handle = tokio::spawn(async move {
            let mut total = 0u64;
            for path in &custom_paths {
                if *cancel_clone.read().await {
                    break;
                }
                if path.exists() {
                    let result = walker
                        .walk_directory(path, tx.clone(), cancel_clone.clone(), &excluded)
                        .await;
                    if let Ok(count) = result {
                        total += count;
                    }
                }
            }
            drop(tx);
            total
        });

        let mut all_files: Vec<FileMetadata> = Vec::new();
        let mut total_scanned = 0u64;
        let mut total_size = 0u64;

        while let Some(batch) = rx.recv().await {
            if *self.cancel_token.read().await {
                break;
            }
            let batch_size: u64 = batch.len() as u64;
            let batch_total_size: u64 = batch.iter().map(|f| f.size).sum();
            total_scanned += batch_size;
            total_size += batch_total_size;

            let mut progress = self.progress.write().await;
            progress.scanned_files = total_scanned;
            progress.scanned_size = total_size;
            progress.elapsed_time_ms = start.elapsed().as_millis() as u64;
            drop(progress);

            all_files.extend(batch);
        }

        let total = walk_handle.await.unwrap_or(0);

        let results = self.categorize_results(all_files, start.elapsed().as_millis() as u64).await;
        self.set_results(results).await;

        Ok(())
    }

    fn get_quick_scan_paths(&self) -> Vec<PathBuf> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".to_string());
        let windows_dir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string());

        vec![
            PathBuf::from(format!(r"{}\AppData\Local\Temp", user_profile)),
            PathBuf::from(format!(r"{}\Temp", windows_dir)),
            PathBuf::from(format!(r"{}\SoftwareDistribution\Download", windows_dir)),
            PathBuf::from(format!(r"{}\Prefetch", windows_dir)),
            PathBuf::from(r"C:\$Recycle.Bin"),
            PathBuf::from(format!(r"{}\AppData\Local\Microsoft\Windows\INetCache", user_profile)),
            PathBuf::from(format!(r"{}\AppData\Local\Microsoft\Windows\Explorer", user_profile)),
            PathBuf::from(format!(r"{}\AppData\Local\Google\Chrome\User Data\Default\Cache", user_profile)),
            PathBuf::from(format!(r"{}\AppData\Local\Microsoft\Edge\User Data\Default\Cache", user_profile)),
            PathBuf::from(format!(r"{}\AppData\Local\Mozilla\Firefox\Profiles", user_profile)),
        ]
    }

    async fn categorize_results(
        &self,
        files: Vec<FileMetadata>,
        _elapsed_ms: u64,
    ) -> Vec<ScanResult> {
        let mut category_map: std::collections::HashMap<String, Vec<FileMetadata>> =
            std::collections::HashMap::new();

        for file in files {
            let key = format!("{:?}", file.category);
            category_map.entry(key).or_default().push(file);
        }

        let mut results = Vec::new();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        for (category_key, category_files) in category_map {
            let total_size: u64 = category_files.iter().map(|f| f.size).sum();
            let file_count = category_files.len() as u64;
            let safety_score = self.calculate_category_safety(&category_key, total_size);

            let risk_level = if safety_score >= 80.0 {
                RiskLevel::Safe
            } else if safety_score >= 60.0 {
                RiskLevel::Low
            } else if safety_score >= 40.0 {
                RiskLevel::Medium
            } else if safety_score >= 20.0 {
                RiskLevel::High
            } else {
                RiskLevel::Critical
            };

            let category = match category_key.as_str() {
                "Temp" => ScanCategory::Temp,
                "Cache" => ScanCategory::Cache,
                "Log" => ScanCategory::Log,
                "Download" => ScanCategory::Download,
                "Recycle" => ScanCategory::Recycle,
                "Browser" => ScanCategory::Browser,
                "System" => ScanCategory::System,
                "App" => ScanCategory::App,
                _ => ScanCategory::Other,
            };

            let recommendation = match &risk_level {
                RiskLevel::Safe => "安全，建议清理".to_string(),
                RiskLevel::Low => "低风险，可以清理".to_string(),
                RiskLevel::Medium => "中等风险，建议谨慎操作".to_string(),
                RiskLevel::High => "高风险，不建议清理".to_string(),
                RiskLevel::Critical => "极高风险，请勿清理".to_string(),
            };

            let name = match category {
                ScanCategory::Temp => "临时文件",
                ScanCategory::Cache => "缓存文件",
                ScanCategory::Log => "日志文件",
                ScanCategory::Download => "下载文件",
                ScanCategory::Recycle => "回收站",
                ScanCategory::Browser => "浏览器缓存",
                ScanCategory::System => "系统文件",
                ScanCategory::App => "应用数据",
                _ => "其他文件",
            };

            results.push(ScanResult {
                id: uuid::Uuid::new_v4().to_string(),
                target: ScanTarget {
                    id: uuid::Uuid::new_v4().to_string(),
                    path: if let Some(f) = category_files.first() {
                        let p = Path::new(&f.path);
                        p.parent().map(|pp| pp.to_str().unwrap_or("").to_string()).unwrap_or_default()
                    } else {
                        String::new()
                    },
                    name: name.to_string(),
                    category,
                    size: total_size,
                    file_count,
                    description: format!("{}: {} 个文件, {:.2} MB", name, file_count, total_size as f64 / (1024.0 * 1024.0)),
                },
                files: category_files,
                total_size,
                file_count,
                safety_score,
                risk_level,
                recommendation,
                scanned_at: now.clone(),
            });
        }

        results.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        results
    }

    fn calculate_category_safety(&self, category: &str, _total_size: u64) -> f64 {
        match category {
            "Temp" => 95.0,
            "Cache" => 90.0,
            "Log" => 85.0,
            "Download" => 60.0,
            "Recycle" => 95.0,
            "Browser" => 88.0,
            "System" => 20.0,
            "App" => 50.0,
            _ => 40.0,
        }
    }

    async fn evaluate_safety_scores(&self) -> Result<(), AppError> {
        let mut results = self.results.write().await;

        for result in results.iter_mut() {
            let path_score = self.score_by_path(&result.target.path);
            let type_score = self.score_by_category(&result.target.category);
            let size_score = self.score_by_size(result.total_size);

            result.safety_score = (path_score * 0.3 + type_score * 0.4 + size_score * 0.3).min(100.0);
            result.risk_level = if result.safety_score >= 80.0 {
                RiskLevel::Safe
            } else if result.safety_score >= 60.0 {
                RiskLevel::Low
            } else if result.safety_score >= 40.0 {
                RiskLevel::Medium
            } else if result.safety_score >= 20.0 {
                RiskLevel::High
            } else {
                RiskLevel::Critical
            };
        }

        Ok(())
    }

    fn score_by_path(&self, path: &str) -> f64 {
        let dangerous_paths = [
            r"C:\Windows",
            r"C:\Program Files",
            r"C:\Program Files (x86)",
        ];
        for dp in &dangerous_paths {
            if path.starts_with(dp) {
                return 10.0;
            }
        }
        80.0
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

    fn score_by_size(&self, size: u64) -> f64 {
        if size > 1024 * 1024 * 1024 {
            70.0
        } else if size > 1024 * 1024 * 100 {
            80.0
        } else {
            90.0
        }
    }

    pub async fn apply_ai_analysis(&self, ai_service: &AIService) -> Result<(), AppError> {
        let config = ai_service.get_config().await;
        if !config.provider.is_cloud()
            || config.api_key.trim().is_empty()
            || config.model.trim().is_empty()
        {
            return Ok(());
        }

        let snapshot = self.results.read().await.clone();
        if snapshot.is_empty() {
            return Ok(());
        }

        let samples = Self::select_ai_samples(&snapshot);
        if samples.is_empty() {
            return Ok(());
        }

        {
            let mut progress = self.progress.write().await;
            progress.phase = ScanPhase::Evaluating;
            progress.current_path = "AI识别扫描结果".to_string();
            progress.percent = progress.percent.max(96.0);
        }

        let analyses = ai_service.analyze_files(&samples).await?;
        if analyses.is_empty() {
            return Ok(());
        }

        let analysis_by_path: HashMap<String, AIFileAnalysis> = analyses
            .into_iter()
            .map(|analysis| (Self::path_key(&analysis.path), analysis))
            .collect();

        let mut results = self.results.write().await;
        for result in results.iter_mut() {
            let mut matched = Vec::new();
            for file in result.files.iter_mut() {
                if let Some(analysis) = analysis_by_path.get(&Self::path_key(&file.path)) {
                    file.category = analysis.category.clone();
                    matched.push(analysis.clone());
                }
            }

            if matched.is_empty() {
                continue;
            }

            let category = Self::majority_category(&matched)
                .unwrap_or_else(|| result.target.category.clone());
            let average_score = matched
                .iter()
                .map(|analysis| analysis.safety_score)
                .sum::<f64>()
                / matched.len() as f64;
            let worst_risk = matched
                .iter()
                .map(|analysis| analysis.risk_level.clone())
                .max_by_key(Self::risk_weight)
                .unwrap_or_else(|| Self::risk_from_score(average_score));
            let unsafe_count = matched
                .iter()
                .filter(|analysis| !analysis.safe_to_clean)
                .count();

            result.target.category = category.clone();
            result.target.name = Self::category_name(&category).to_string();
            result.safety_score = Self::cap_score_for_risk(average_score, &worst_risk);
            result.risk_level = worst_risk.clone();
            result.recommendation =
                Self::build_ai_recommendation(&matched, unsafe_count, &worst_risk);
            result.target.description = format!(
                "{}: {} 个文件, {:.2} MB，AI抽样识别 {} 个",
                Self::category_name(&category),
                result.file_count,
                result.total_size as f64 / (1024.0 * 1024.0),
                matched.len()
            );
        }

        Ok(())
    }

    fn select_ai_samples(results: &[ScanResult]) -> Vec<FileMetadata> {
        let mut samples = Vec::new();
        let mut seen_paths = HashSet::new();

        for result in results {
            let mut files = result.files.clone();
            files.sort_by(|a, b| b.size.cmp(&a.size));

            for file in files.into_iter().take(AI_SAMPLE_PER_RESULT) {
                if samples.len() >= AI_MAX_SAMPLE_FILES {
                    return samples;
                }

                let path_key = Self::path_key(&file.path);
                if seen_paths.insert(path_key) {
                    samples.push(file);
                }
            }
        }

        samples
    }

    fn majority_category(analyses: &[AIFileAnalysis]) -> Option<ScanCategory> {
        let mut counts: HashMap<String, (ScanCategory, usize)> = HashMap::new();
        for analysis in analyses {
            let entry = counts
                .entry(format!("{:?}", analysis.category))
                .or_insert_with(|| (analysis.category.clone(), 0));
            entry.1 += 1;
        }

        counts
            .into_values()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category)
    }

    fn risk_weight(risk_level: &RiskLevel) -> u8 {
        match risk_level {
            RiskLevel::Safe => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }

    fn cap_score_for_risk(score: f64, risk_level: &RiskLevel) -> f64 {
        let cap = match risk_level {
            RiskLevel::Safe => 100.0,
            RiskLevel::Low => 79.0,
            RiskLevel::Medium => 59.0,
            RiskLevel::High => 39.0,
            RiskLevel::Critical => 19.0,
        };

        score.min(cap).clamp(0.0, 100.0)
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

    fn build_ai_recommendation(
        analyses: &[AIFileAnalysis],
        unsafe_count: usize,
        worst_risk: &RiskLevel,
    ) -> String {
        let mut reasons = Vec::new();
        let mut seen = HashSet::new();

        for analysis in analyses {
            let reason = analysis.reason.trim();
            if !reason.is_empty() && seen.insert(reason.to_string()) {
                reasons.push(reason.to_string());
            }

            if reasons.len() >= 2 {
                break;
            }
        }

        let reason_text = if reasons.is_empty() {
            "未提供详细原因".to_string()
        } else {
            reasons.join("；")
        };

        if unsafe_count == 0 {
            format!(
                "AI识别：{}，抽样 {} 个文件均可清理。{}",
                Self::risk_label(worst_risk),
                analyses.len(),
                reason_text
            )
        } else {
            format!(
                "AI识别：{}，抽样 {} 个文件中 {} 个不建议清理。{}",
                Self::risk_label(worst_risk),
                analyses.len(),
                unsafe_count,
                reason_text
            )
        }
    }

    fn risk_label(risk_level: &RiskLevel) -> &'static str {
        match risk_level {
            RiskLevel::Safe => "安全",
            RiskLevel::Low => "低风险",
            RiskLevel::Medium => "中等风险",
            RiskLevel::High => "高风险",
            RiskLevel::Critical => "极高风险",
        }
    }

    fn category_name(category: &ScanCategory) -> &'static str {
        match category {
            ScanCategory::Temp => "临时文件",
            ScanCategory::Cache => "缓存文件",
            ScanCategory::Log => "日志文件",
            ScanCategory::Download => "下载文件",
            ScanCategory::Recycle => "回收站",
            ScanCategory::Browser => "浏览器缓存",
            ScanCategory::System => "系统文件",
            ScanCategory::App => "应用数据",
            ScanCategory::Other => "其他文件",
        }
    }

    fn path_key(path: &str) -> String {
        path.replace('/', "\\").to_lowercase()
    }

    pub async fn stop_scan(&self) -> Result<(), AppError> {
        let scanning = self.is_scanning.read().await;
        if !*scanning {
            return Err(AppError::ScanError("没有正在进行的扫描".to_string()));
        }
        drop(scanning);

        let mut cancel = self.cancel_token.write().await;
        *cancel = true;

        let mut progress = self.progress.write().await;
        progress.phase = ScanPhase::Cancelled;

        let mut scanning = self.is_scanning.write().await;
        *scanning = false;

        Ok(())
    }

    pub async fn get_progress(&self) -> ScanProgress {
        self.progress.read().await.clone()
    }

    pub async fn get_results(&self) -> Vec<ScanResult> {
        self.results.read().await.clone()
    }

    pub async fn is_scanning(&self) -> bool {
        *self.is_scanning.read().await
    }

    pub async fn set_progress(&self, p: ScanProgress) {
        let mut progress = self.progress.write().await;
        *progress = p;
    }

    pub async fn set_results(&self, r: Vec<ScanResult>) {
        let mut results = self.results.write().await;
        *results = r;
    }

    pub async fn set_scanning(&self, v: bool) {
        let mut scanning = self.is_scanning.write().await;
        *scanning = v;
    }

    pub async fn is_cancelled(&self) -> bool {
        *self.cancel_token.read().await
    }

    pub async fn get_scan_history(&self) -> Vec<ScanResult> {
        self.scan_history.read().await.clone()
    }
}
