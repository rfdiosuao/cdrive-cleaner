use std::sync::Arc;
use rusqlite::OptionalExtension;

use crate::error::AppError;
use crate::store::database::Database;

pub struct ScanResultStore {
    db: Arc<Database>,
}

impl ScanResultStore {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn save_scan_result(&self, result: &crate::models::scan::ScanResult) -> Result<(), AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        conn.execute(
            "INSERT OR REPLACE INTO scan_results (id, target_path, target_name, category, total_size, file_count, safety_score, risk_level, scanned_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                result.id,
                result.target.path,
                result.target.name,
                format!("{:?}", result.target.category),
                result.total_size,
                result.file_count,
                result.safety_score,
                format!("{:?}", result.risk_level),
                result.scanned_at,
            ],
        )
        .map_err(|e| AppError::DatabaseError(format!("保存扫描结果失败: {}", e)))?;

        Ok(())
    }

    pub fn save_scan_results(&self, results: &[crate::models::scan::ScanResult]) -> Result<(), AppError> {
        for result in results {
            self.save_scan_result(result)?;
        }
        Ok(())
    }

    pub fn get_scan_results(&self, limit: usize) -> Result<Vec<ScanResultRow>, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT id, target_path, target_name, category, total_size, file_count, safety_score, risk_level, scanned_at FROM scan_results ORDER BY scanned_at DESC LIMIT ?1")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok(ScanResultRow {
                    id: row.get(0)?,
                    target_path: row.get(1)?,
                    target_name: row.get(2)?,
                    category: row.get(3)?,
                    total_size: row.get(4)?,
                    file_count: row.get(5)?,
                    safety_score: row.get(6)?,
                    risk_level: row.get(7)?,
                    scanned_at: row.get(8)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(format!("查询扫描结果失败: {}", e)))?;

        let results: Vec<ScanResultRow> = rows.filter_map(|r| r.ok()).collect();
        Ok(results)
    }

    pub fn get_scan_result_by_path(&self, path: &str) -> Result<Option<ScanResultRow>, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT id, target_path, target_name, category, total_size, file_count, safety_score, risk_level, scanned_at FROM scan_results WHERE target_path = ?1 ORDER BY scanned_at DESC LIMIT 1")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result = stmt
            .query_row(rusqlite::params![path], |row| {
                Ok(ScanResultRow {
                    id: row.get(0)?,
                    target_path: row.get(1)?,
                    target_name: row.get(2)?,
                    category: row.get(3)?,
                    total_size: row.get(4)?,
                    file_count: row.get(5)?,
                    safety_score: row.get(6)?,
                    risk_level: row.get(7)?,
                    scanned_at: row.get(8)?,
                })
            })
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?;

        Ok(result)
    }

    pub fn cleanup_old_results(&self, days: u32) -> Result<u64, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        let count = conn
            .execute(
                "DELETE FROM scan_results WHERE scanned_at < ?1",
                rusqlite::params![cutoff_str],
            )
            .map_err(|e| AppError::DatabaseError(format!("清理扫描结果失败: {}", e)))?;

        Ok(count as u64)
    }

    pub fn save_clean_history(
        &self,
        task_id: &str,
        success: bool,
        freed_space: u64,
        cleaned_files: u64,
        backup_path: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            "INSERT INTO clean_history (id, task_id, success, freed_space, cleaned_files, backup_path, executed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                task_id,
                success as i32,
                freed_space,
                cleaned_files,
                backup_path.unwrap_or(""),
                timestamp,
            ],
        )
        .map_err(|e| AppError::DatabaseError(format!("保存清理历史失败: {}", e)))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ScanResultRow {
    pub id: String,
    pub target_path: String,
    pub target_name: String,
    pub category: String,
    pub total_size: u64,
    pub file_count: u64,
    pub safety_score: f64,
    pub risk_level: String,
    pub scanned_at: String,
}
