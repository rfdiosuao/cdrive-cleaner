use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use rusqlite::OptionalExtension;

use crate::error::AppError;
use crate::store::database::Database;

pub struct FingerprintCache {
    db: Arc<Database>,
}

impl FingerprintCache {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_fingerprint(&self, file_path: &str) -> Result<Option<FileFingerprint>, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT file_path, md5_hash, xxhash, file_size, modified_at, computed_at FROM file_fingerprints WHERE file_path = ?1")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result = stmt
            .query_row(rusqlite::params![file_path], |row| {
                Ok(FileFingerprint {
                    file_path: row.get(0)?,
                    md5_hash: row.get(1)?,
                    xxhash: row.get(2)?,
                    file_size: row.get(3)?,
                    modified_at: row.get(4)?,
                    computed_at: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询指纹失败: {}", e)))?;

        Ok(result)
    }

    pub async fn save_fingerprint(&self, fp: &FileFingerprint) -> Result<(), AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO file_fingerprints (file_path, md5_hash, xxhash, file_size, modified_at, computed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                fp.file_path,
                fp.md5_hash,
                fp.xxhash,
                fp.file_size,
                fp.modified_at,
                fp.computed_at,
            ],
        )
        .map_err(|e| AppError::DatabaseError(format!("保存指纹失败: {}", e)))?;

        Ok(())
    }

    pub async fn compute_md5(&self, file_path: &str) -> Result<String, AppError> {
        let data = fs::read(file_path).await.map_err(|e| {
            AppError::FileSystemError(format!("读取文件失败: {}: {}", file_path, e))
        })?;

        use md5::{Md5, Digest};
        let hash = Md5::digest(&data);
        Ok(format!("{:x}", hash))
    }

    pub async fn compute_xxhash(&self, file_path: &str) -> Result<String, AppError> {
        let data = fs::read(file_path).await.map_err(|e| {
            AppError::FileSystemError(format!("读取文件失败: {}: {}", file_path, e))
        })?;

        let hash = xxhash_rust::xxh3::xxh3_64(&data);
        Ok(format!("{:016x}", hash))
    }

    pub async fn get_or_compute(&self, file_path: &str, file_size: u64, modified_at: &str) -> Result<FileFingerprint, AppError> {
        if let Some(fp) = self.get_fingerprint(file_path).await? {
            if fp.file_size == file_size && fp.modified_at == modified_at {
                return Ok(fp);
            }
        }

        let md5_hash = self.compute_md5(file_path).await?;
        let xxhash = self.compute_xxhash(file_path).await?;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let fp = FileFingerprint {
            file_path: file_path.to_string(),
            md5_hash,
            xxhash,
            file_size,
            modified_at: modified_at.to_string(),
            computed_at: now,
        };

        self.save_fingerprint(&fp).await?;
        Ok(fp)
    }

    pub async fn find_duplicates(&self, file_paths: &[String]) -> Result<Vec<Vec<String>>, AppError> {
        let mut hash_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for path in file_paths {
            let meta = fs::metadata(path).await.map_err(|e| {
                AppError::FileSystemError(format!("获取文件元数据失败: {}: {}", path, e))
            })?;

            let modified = meta.modified().ok().and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH).ok()
            }).map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            }).unwrap_or_default();

            let fp = self.get_or_compute(path, meta.len(), &modified).await?;
            hash_groups
                .entry(fp.xxhash)
                .or_default()
                .push(path.clone());
        }

        let duplicates: Vec<Vec<String>> = hash_groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect();

        Ok(duplicates)
    }

    pub async fn cleanup_expired(&self, days: u32) -> Result<u64, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        let count = conn
            .execute(
                "DELETE FROM file_fingerprints WHERE computed_at < ?1",
                rusqlite::params![cutoff_str],
            )
            .map_err(|e| AppError::DatabaseError(format!("清理过期指纹失败: {}", e)))?;

        Ok(count as u64)
    }
}

#[derive(Debug, Clone)]
pub struct FileFingerprint {
    pub file_path: String,
    pub md5_hash: String,
    pub xxhash: String,
    pub file_size: u64,
    pub modified_at: String,
    pub computed_at: String,
}
