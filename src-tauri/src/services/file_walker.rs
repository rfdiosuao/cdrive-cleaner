use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore, RwLock};
use tokio::fs;

use crate::error::AppError;
use crate::models::scan::{FileMetadata, ScanCategory};

pub struct FileWalker {
    semaphore: Arc<Semaphore>,
    batch_size: usize,
}

impl FileWalker {
    pub fn new(max_concurrent: usize, batch_size: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            batch_size,
        }
    }

    pub async fn walk_directory(
        &self,
        root: &Path,
        tx: mpsc::Sender<Vec<FileMetadata>>,
        cancel: Arc<RwLock<bool>>,
        excluded_dirs: &[PathBuf],
    ) -> Result<u64, AppError> {
        let mut total_files = 0u64;
        let mut batch = Vec::with_capacity(self.batch_size);

        self.walk_recursive(
            root,
            &tx,
            &mut batch,
            &mut total_files,
            cancel.clone(),
            excluded_dirs,
        )
        .await?;

        if !batch.is_empty() {
            let _ = tx.send(batch).await;
        }

        Ok(total_files)
    }

    fn walk_recursive<'a>(
        &'a self,
        dir: &'a Path,
        tx: &'a mpsc::Sender<Vec<FileMetadata>>,
        batch: &'a mut Vec<FileMetadata>,
        total: &'a mut u64,
        cancel: Arc<RwLock<bool>>,
        excluded: &'a [PathBuf],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            if *cancel.read().await {
                return Ok(());
            }

            if excluded.iter().any(|e| dir.starts_with(e)) {
                return Ok(());
            }

            let mut permit = self.semaphore.clone().acquire_owned().await
                .map_err(|e| AppError::ScanError(format!("并发限制获取失败: {}", e)))?;

            let mut entries = match fs::read_dir(dir).await {
                Ok(entries) => entries,
                Err(_) => {
                    drop(permit);
                    return Ok(());
                }
            };

            while let Some(entry) = entries.next_entry().await.map_err(|e| {
                AppError::FileSystemError(format!("读取目录条目失败: {}", e))
            })? {
                if *cancel.read().await {
                    drop(permit);
                    return Ok(());
                }

                let path = entry.path();

                let metadata = match entry.metadata().await {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                if metadata.is_dir() {
                    drop(permit);
                    self.walk_recursive(
                        &path,
                        tx,
                        batch,
                        total,
                        cancel.clone(),
                        excluded,
                    )
                    .await?;
                    permit = self.semaphore.clone().acquire_owned().await
                        .map_err(|e| AppError::ScanError(format!("并发限制获取失败: {}", e)))?;
                    continue;
                }

                let file_meta = self.extract_metadata(&path, &metadata).await;

                batch.push(file_meta);
                *total += 1;

                if batch.len() >= self.batch_size {
                    let chunk: Vec<FileMetadata> = batch.drain(..).collect();
                    if tx.send(chunk).await.is_err() {
                        drop(permit);
                        return Ok(());
                    }
                }
            }

            drop(permit);
            Ok(())
        })
    }

    async fn extract_metadata(
        &self,
        path: &Path,
        metadata: &std::fs::Metadata,
    ) -> FileMetadata {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string()
            .to_lowercase();

        let category = Self::categorize_path(path, &extension);

        let modified = metadata.modified().ok();
        let created = metadata.created().ok();

        let modified_at = modified
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let created_at = created
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        FileMetadata {
            path: path.to_str().unwrap_or("").to_string(),
            name,
            size: metadata.len(),
            modified_at,
            created_at,
            is_directory: metadata.is_dir(),
            extension,
            category,
            hash: String::new(),
        }
    }

    fn categorize_path(path: &Path, ext: &str) -> ScanCategory {
        let path_text = path.to_string_lossy().replace('/', "\\").to_lowercase();

        if path_text.contains(r"\$recycle.bin\") || path_text.ends_with(r"\$recycle.bin") {
            return ScanCategory::Recycle;
        }
        if path_text.contains(r"\inetcache\")
            || path_text.contains(r"\google\chrome\")
            || path_text.contains(r"\microsoft\edge\")
            || path_text.contains(r"\mozilla\firefox\")
        {
            return ScanCategory::Browser;
        }
        if path_text.contains(r"\softwaredistribution\download\")
            || path_text.contains(r"\prefetch\")
        {
            return ScanCategory::Cache;
        }
        if path_text.contains(r"\temp\") || path_text.ends_with(r"\temp") {
            return ScanCategory::Temp;
        }

        match ext {
            "tmp" | "temp" | "bak" => ScanCategory::Temp,
            "log" | "old" => ScanCategory::Log,
            "cache" | "dat" => ScanCategory::Cache,
            "exe" | "msi" | "dll" | "sys" | "com" | "scr" => ScanCategory::App,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => ScanCategory::Download,
            "db" | "sqlite" | "json" | "xml" | "ini" | "cfg" | "conf" => ScanCategory::App,
            _ => ScanCategory::Other,
        }
    }
}

pub struct WalkConfig {
    pub max_concurrent: usize,
    pub batch_size: usize,
    pub excluded_dirs: Vec<PathBuf>,
}

impl Default for WalkConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 64,
            batch_size: 100,
            excluded_dirs: vec![
                PathBuf::from(r"C:\Windows"),
                PathBuf::from(r"C:\Program Files"),
                PathBuf::from(r"C:\Program Files (x86)"),
            ],
        }
    }
}
