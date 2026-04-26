use rusqlite::{Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

use crate::error::AppError;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)
            .map_err(|e| AppError::DatabaseError(format!("数据库连接失败: {}", e)))?;

        let db = Self {
            conn: Mutex::new(conn),
        };

        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scan_results (
                id TEXT PRIMARY KEY,
                target_path TEXT NOT NULL,
                target_name TEXT NOT NULL,
                category TEXT NOT NULL,
                total_size INTEGER NOT NULL,
                file_count INTEGER NOT NULL,
                safety_score REAL NOT NULL,
                risk_level TEXT NOT NULL,
                scanned_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS clean_history (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                success INTEGER NOT NULL,
                freed_space INTEGER NOT NULL,
                cleaned_files INTEGER NOT NULL,
                backup_path TEXT,
                executed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS whitelist (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                reason TEXT,
                added_at TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                installed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_fingerprints (
                file_path TEXT PRIMARY KEY,
                md5_hash TEXT NOT NULL,
                xxhash TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                modified_at TEXT NOT NULL,
                computed_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                target_path TEXT NOT NULL,
                detail TEXT,
                risk_level TEXT,
                timestamp TEXT NOT NULL,
                encrypted INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS user_behavior (
                id TEXT PRIMARY KEY,
                action_type TEXT NOT NULL,
                category TEXT NOT NULL,
                action_detail TEXT,
                timestamp TEXT NOT NULL
            );",
        )
        .map_err(|e| AppError::DatabaseError(format!("建表失败: {}", e)))?;

        Ok(())
    }

    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<u64, AppError> {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;
        let count = conn.execute(sql, params)
            .map_err(|e| AppError::DatabaseError(format!("执行SQL失败: {}", e)))?;
        Ok(count as u64)
    }

    pub fn query_one<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], mapper: F) -> Result<Option<T>, AppError>
    where
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;
        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;
        let result = stmt.query_row(params, mapper).optional()
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?;
        Ok(result)
    }

    pub fn query_many<T, F>(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql], mapper: F) -> Result<Vec<T>, AppError>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;
        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;
        let results = stmt.query_map(params, mapper)
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }
}
