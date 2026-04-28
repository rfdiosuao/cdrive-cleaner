use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::fs;

use crate::error::AppError;
use crate::models::clean::{CleanProgress, CleanResult, CleanTask, CleanPhase, CleanError, RestorePoint};
use crate::services::safety_guard::SafetyGuard;

const MAX_PREVIEW_WARNINGS: usize = 50;

pub struct CleanEngine {
    progress: Arc<RwLock<Option<CleanProgress>>>,
    is_cleaning: Arc<RwLock<bool>>,
    cancel_token: Arc<RwLock<bool>>,
    safety_guard: Arc<SafetyGuard>,
    backup_dir: Arc<RwLock<PathBuf>>,
    restore_points: Arc<RwLock<Vec<RestorePoint>>>,
}

impl CleanEngine {
    pub fn new() -> Self {
        let backup_path = std::env::temp_dir().join("cdrive-cleaner-backup");
        Self {
            progress: Arc::new(RwLock::new(None)),
            is_cleaning: Arc::new(RwLock::new(false)),
            cancel_token: Arc::new(RwLock::new(false)),
            safety_guard: Arc::new(SafetyGuard::new()),
            backup_dir: Arc::new(RwLock::new(backup_path)),
            restore_points: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn preview(&self, tasks: &[CleanTask]) -> Result<CleanPreview, AppError> {
        let mut total_size = 0u64;
        let mut total_files = 0u64;
        let mut warnings = Vec::new();
        let mut hidden_warning_count = 0usize;

        for task in tasks {
            let safety = self.safety_guard.check_path_safety(&task.target_path);
            if !safety.is_safe {
                if warnings.len() < MAX_PREVIEW_WARNINGS {
                    warnings.push(format!("「{}」: {}", task.target_name, safety.reason));
                } else {
                    hidden_warning_count += 1;
                }
            }

            total_size += task.size;
            total_files += task.file_count;
        }

        if hidden_warning_count > 0 {
            warnings.push(format!(
                "还有 {} 条风险提示已折叠，请谨慎确认。",
                hidden_warning_count
            ));
        }

        Ok(CleanPreview {
            total_size,
            total_files,
            task_count: tasks.len(),
            warnings,
            estimated_time_ms: total_files * 10,
        })
    }

    pub async fn execute(&self, tasks: Vec<CleanTask>) -> Result<Vec<CleanResult>, AppError> {
        if tasks.is_empty() {
            return Err(AppError::CleanError("清理任务列表不能为空".to_string()));
        }

        let mut cleaning = self.is_cleaning.write().await;
        if *cleaning {
            return Err(AppError::CleanError("清理任务正在进行中".to_string()));
        }
        *cleaning = true;
        drop(cleaning);

        let mut cancel = self.cancel_token.write().await;
        *cancel = false;
        drop(cancel);

        let backup_dir = self.backup_dir.read().await.clone();
        let _ = fs::create_dir_all(&*backup_dir).await;

        let restore_id = uuid::Uuid::new_v4().to_string();
        let restore_backup_dir = backup_dir.join(&restore_id);
        let _ = fs::create_dir_all(&restore_backup_dir).await;

        let start = Instant::now();
        let total_tasks = tasks.len();
        let mut results = Vec::new();

        let mut progress = self.progress.write().await;
        *progress = Some(CleanProgress {
            task_id: String::new(),
            current_file: "正在准备清理任务".to_string(),
            completed_files: 0,
            total_files: total_tasks as u64,
            freed_space: 0,
            percent: 0.0,
            phase: CleanPhase::Preparing,
        });
        drop(progress);

        for (idx, task) in tasks.iter().enumerate() {
            if *self.cancel_token.read().await {
                break;
            }

            let mut progress = self.progress.write().await;
            *progress = Some(CleanProgress {
                task_id: task.id.clone(),
                current_file: task.target_path.clone(),
                completed_files: idx as u64,
                total_files: total_tasks as u64,
                freed_space: results.iter().map(|r: &CleanResult| r.freed_space).sum(),
                percent: (idx as f64 / total_tasks as f64) * 100.0,
                phase: CleanPhase::BackingUp,
            });
            drop(progress);

            let task_backup_dir = restore_backup_dir.join(&task.id);
            if task.backup_required {
                if let Err(e) = self.backup_file(&task.target_path, &task_backup_dir).await {
                    tracing::warn!("备份失败: {} - {}", task.target_path, e);
                }
            }

            let mut progress = self.progress.write().await;
            *progress = Some(CleanProgress {
                task_id: task.id.clone(),
                current_file: task.target_path.clone(),
                completed_files: idx as u64,
                total_files: total_tasks as u64,
                freed_space: results.iter().map(|r: &CleanResult| r.freed_space).sum(),
                percent: (idx as f64 / total_tasks as f64) * 100.0,
                phase: CleanPhase::Cleaning,
            });
            drop(progress);

            let result = self.clean_single_task(task).await;
            match result {
                Ok(mut clean_result) => {
                    if task.backup_required {
                        clean_result.backup_path = task_backup_dir.to_str().unwrap_or("").to_string();
                    }
                    results.push(clean_result)
                }
                Err(e) => {
                    results.push(CleanResult {
                        task_id: task.id.clone(),
                        success: false,
                        freed_space: 0,
                        cleaned_files: 0,
                        failed_files: task.file_count,
                        backup_path: if task.backup_required {
                            task_backup_dir.to_str().unwrap_or("").to_string()
                        } else {
                            String::new()
                        },
                        elapsed_time_ms: start.elapsed().as_millis() as u64,
                        errors: vec![CleanError {
                            file_path: task.target_path.clone(),
                            error_code: "CLEAN_FAILED".to_string(),
                            message: e.to_string(),
                        }],
                    });
                }
            }

            let mut progress = self.progress.write().await;
            *progress = Some(CleanProgress {
                task_id: task.id.clone(),
                current_file: task.target_path.clone(),
                completed_files: (idx + 1) as u64,
                total_files: total_tasks as u64,
                freed_space: results.iter().map(|r: &CleanResult| r.freed_space).sum(),
                percent: ((idx + 1) as f64 / total_tasks as f64) * 100.0,
                phase: CleanPhase::Verifying,
            });
            drop(progress);
        }

        let mut progress = self.progress.write().await;
        *progress = Some(CleanProgress {
            task_id: String::new(),
            current_file: String::new(),
            completed_files: total_tasks as u64,
            total_files: total_tasks as u64,
            freed_space: results.iter().map(|r: &CleanResult| r.freed_space).sum(),
            percent: 100.0,
            phase: CleanPhase::Completed,
        });
        drop(progress);

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let total_size: u64 = results.iter().map(|r| r.freed_space).sum();

        let mut restore_points = self.restore_points.write().await;
        restore_points.push(RestorePoint {
            id: restore_id.clone(),
            created_at: now,
            tasks: tasks.clone(),
            total_size,
            description: format!("清理前备份 - {} 个任务", tasks.len()),
        });

        let mut cleaning = self.is_cleaning.write().await;
        *cleaning = false;

        Ok(results)
    }

    async fn clean_single_task(&self, task: &CleanTask) -> Result<CleanResult, AppError> {
        let start = Instant::now();
        let path = Path::new(&task.target_path);

        let safety = self.safety_guard.check_path_safety(&task.target_path);
        if !safety.is_safe {
            return Ok(CleanResult {
                task_id: task.id.clone(),
                success: false,
                freed_space: 0,
                cleaned_files: 0,
                failed_files: task.file_count,
                backup_path: String::new(),
                elapsed_time_ms: start.elapsed().as_millis() as u64,
                errors: vec![CleanError {
                    file_path: task.target_path.clone(),
                    error_code: "PROTECTED_BY_SAFETY_GUARD".to_string(),
                    message: format!("防护拦截: {}", safety.reason),
                }],
            });
        }

        if !path.exists() {
            return Ok(CleanResult {
                task_id: task.id.clone(),
                success: true,
                freed_space: 0,
                cleaned_files: 0,
                failed_files: 0,
                backup_path: String::new(),
                elapsed_time_ms: start.elapsed().as_millis() as u64,
                errors: vec![],
            });
        }

        let metadata = fs::metadata(path).await.map_err(|e| {
            AppError::FileSystemError(format!("获取文件元数据失败: {} - {}", task.target_path, e))
        })?;

        let size = metadata.len();

        if path.is_dir() {
            let stats = self.remove_dir_contents(path).await?;

            Ok(CleanResult {
                task_id: task.id.clone(),
                success: stats.failed == 0,
                freed_space: stats.freed,
                cleaned_files: stats.cleaned,
                failed_files: stats.failed,
                backup_path: String::new(),
                elapsed_time_ms: start.elapsed().as_millis() as u64,
                errors: stats.errors,
            })
        } else {
            match fs::remove_file(path).await {
                Ok(_) => Ok(CleanResult {
                    task_id: task.id.clone(),
                    success: true,
                    freed_space: size,
                    cleaned_files: 1,
                    failed_files: 0,
                    backup_path: String::new(),
                    elapsed_time_ms: start.elapsed().as_millis() as u64,
                    errors: vec![],
                }),
                Err(e) => Ok(CleanResult {
                    task_id: task.id.clone(),
                    success: false,
                    freed_space: 0,
                    cleaned_files: 0,
                    failed_files: 1,
                    backup_path: String::new(),
                    elapsed_time_ms: start.elapsed().as_millis() as u64,
                    errors: vec![CleanError {
                        file_path: task.target_path.clone(),
                        error_code: "DELETE_FAILED".to_string(),
                        message: e.to_string(),
                    }],
                }),
            }
        }
    }

    fn remove_dir_contents<'a>(
        &'a self,
        dir: &'a Path,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<RemoveDirStats, AppError>> + Send + 'a>> {
        Box::pin(async move {
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            AppError::FileSystemError(format!("读取目录失败: {} - {}", dir.display(), e))
        })?;

        let mut stats = RemoveDirStats::default();

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AppError::FileSystemError(format!("读取目录条目失败: {}", e))
        })? {
            if *self.cancel_token.read().await {
                break;
            }

            let path = entry.path();
            let meta = match entry.metadata().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            if path.is_dir() {
                let sub_stats = self.remove_dir_contents(&path).await?;
                stats.cleaned += sub_stats.cleaned;
                stats.failed += sub_stats.failed;
                stats.freed += sub_stats.freed;
                stats.errors.extend(sub_stats.errors);
                if let Err(e) = fs::remove_dir(&path).await {
                    stats.failed += 1;
                    stats.errors.push(CleanError {
                        file_path: path.to_str().unwrap_or("").to_string(),
                        error_code: "RMDIR_FAILED".to_string(),
                        message: e.to_string(),
                    });
                }
            } else {
                let size = meta.len();
                match fs::remove_file(&path).await {
                    Ok(_) => {
                        stats.cleaned += 1;
                        stats.freed += size;
                    }
                    Err(e) => {
                        stats.failed += 1;
                        stats.errors.push(CleanError {
                            file_path: path.to_str().unwrap_or("").to_string(),
                            error_code: "DELETE_FAILED".to_string(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(stats)
        })
    }

    async fn backup_file(&self, target_path: &str, backup_dir: &Path) -> Result<(), AppError> {
        let source = Path::new(target_path);
        if !source.exists() {
            return Ok(());
        }

        let file_name = source.file_name().unwrap_or_default();
        let dest = backup_dir.join(file_name);

        if source.is_dir() {
            self.copy_dir_recursive(source, &dest).await?;
        } else {
            let _ = fs::create_dir_all(backup_dir).await;
            fs::copy(source, &dest).await.map_err(|e| {
                AppError::FileSystemError(format!("备份文件失败: {} - {}", target_path, e))
            })?;
        }

        Ok(())
    }

    fn copy_dir_recursive<'a>(&'a self, src: &'a Path, dest: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
        fs::create_dir_all(dest).await.map_err(|e| {
            AppError::FileSystemError(format!("创建目录失败: {} - {}", dest.display(), e))
        })?;

        let mut entries = fs::read_dir(src).await.map_err(|e| {
            AppError::FileSystemError(format!("读取目录失败: {} - {}", src.display(), e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AppError::FileSystemError(format!("读取目录条目失败: {}", e))
        })? {
            let path = entry.path();
            let dest_path = dest.join(path.file_name().unwrap_or_default());

            if path.is_dir() {
                self.copy_dir_recursive(&path, &dest_path).await?;
            } else {
                fs::copy(&path, &dest_path).await.map_err(|e| {
                    AppError::FileSystemError(format!("复制文件失败: {} - {}", path.display(), e))
                })?;
            }
        }

        Ok(())
        })
    }

    pub async fn restore(&self, restore_id: &str) -> Result<(), AppError> {
        let restore_tasks = {
            let restore_points = self.restore_points.read().await;
            restore_points
                .iter()
                .find(|rp| rp.id == restore_id)
                .ok_or_else(|| AppError::CleanError(format!("还原点不存在: {}", restore_id)))?
                .tasks
                .clone()
        };

        let backup_dir = self.backup_dir.read().await.clone();
        let restore_backup_dir = backup_dir.join(restore_id);

        if !restore_backup_dir.exists() {
            return Err(AppError::CleanError("备份目录不存在".to_string()));
        }

        let mut errors = Vec::new();
        for task in &restore_tasks {
            let task_backup_dir = restore_backup_dir.join(&task.id);
            if let Err(e) = self.restore_single_file(&task_backup_dir, &task.target_path).await {
                errors.push(format!("{}: {}", task.target_path, e));
            }
        }

        if !errors.is_empty() {
            return Err(AppError::CleanError(format!(
                "部分文件恢复失败: {}",
                errors.join("; ")
            )));
        }

        Ok(())
    }

    async fn restore_single_file(&self, backup_dir: &Path, target_path: &str) -> Result<(), AppError> {
        let target = Path::new(target_path);
        let file_name = target.file_name().unwrap_or_default();
        let source = backup_dir.join(file_name);

        if !source.exists() {
            return Ok(());
        }

        if let Some(parent) = target.parent() {
            let _ = fs::create_dir_all(parent).await;
        }

        if source.is_dir() {
            self.copy_dir_recursive(&source, target).await?;
        } else {
            fs::copy(&source, target).await.map_err(|e| {
                AppError::FileSystemError(format!("恢复文件失败: {} - {}", target_path, e))
            })?;
        }

        Ok(())
    }

    pub async fn cleanup_old_backups(&self, days: u32) -> Result<u64, AppError> {
        let backup_dir = self.backup_dir.read().await.clone();
        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);

        let mut removed = 0u64;
        if backup_dir.exists() {
            let mut entries = fs::read_dir(&*backup_dir).await.map_err(|e| {
                AppError::FileSystemError(format!("读取备份目录失败: {}", e))
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                AppError::FileSystemError(format!("读取目录条目失败: {}", e))
            })? {
                let path = entry.path();
                if let Ok(meta) = entry.metadata().await {
                    if let Ok(modified) = meta.modified() {
                        if let Ok(modified_time) = modified.duration_since(std::time::UNIX_EPOCH) {
                            if let Some(modified_dt) = chrono::DateTime::from_timestamp(modified_time.as_secs() as i64, 0) {
                                if modified_dt.naive_local() < cutoff.naive_local() {
                                    let _ = fs::remove_dir_all(&path).await;
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    pub async fn get_progress(&self) -> Option<CleanProgress> {
        self.progress.read().await.clone()
    }

    pub async fn is_cleaning(&self) -> bool {
        *self.is_cleaning.read().await
    }

    pub async fn cancel(&self) -> Result<(), AppError> {
        let cleaning = self.is_cleaning.read().await;
        if !*cleaning {
            return Err(AppError::CleanError("没有正在进行的清理任务".to_string()));
        }
        drop(cleaning);

        let mut cancel = self.cancel_token.write().await;
        *cancel = true;

        let mut progress = self.progress.write().await;
        if let Some(p) = progress.as_mut() {
            p.phase = CleanPhase::Cancelled;
        }

        let mut cleaning = self.is_cleaning.write().await;
        *cleaning = false;

        Ok(())
    }

    pub async fn get_restore_points(&self) -> Vec<RestorePoint> {
        self.restore_points.read().await.clone()
    }
}

#[derive(Debug, Clone, Default)]
struct RemoveDirStats {
    cleaned: u64,
    failed: u64,
    freed: u64,
    errors: Vec<CleanError>,
}

#[derive(Debug, Clone)]
pub struct CleanPreview {
    pub total_size: u64,
    pub total_files: u64,
    pub task_count: usize,
    pub warnings: Vec<String>,
    pub estimated_time_ms: u64,
}
