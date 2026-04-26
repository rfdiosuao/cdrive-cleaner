use std::sync::Arc;
use tokio::sync::RwLock;
use rusqlite::OptionalExtension;

use crate::error::AppError;
use crate::models::config::{AIConfig, AppConfig, UserPreferences, WhitelistConfig, WhitelistEntry};
use crate::store::database::Database;

pub struct ConfigStore {
    db: Arc<Database>,
    cache: Arc<RwLock<AppConfig>>,
}

impl ConfigStore {
    pub fn new(db: Arc<Database>) -> Self {
        let config = AppConfig::default();
        Self {
            db,
            cache: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn load_config(&self) -> Result<AppConfig, AppError> {
        let prefs = self.load_preferences()?;
        let ai = self.load_ai_config()?;
        let whitelist = self.load_whitelist()?;

        let config = AppConfig {
            ai,
            preferences: prefs,
            whitelist,
        };

        let mut cache = self.cache.write().await;
        *cache = config.clone();

        Ok(config)
    }

    pub async fn get_config(&self) -> AppConfig {
        self.cache.read().await.clone()
    }

    pub async fn save_preferences(&self, prefs: &UserPreferences) -> Result<(), AppError> {
        let prefs_json = serde_json::to_string(prefs)
            .map_err(|e| AppError::ConfigError(format!("序列化失败: {}", e)))?;

        let execute_result = {
            let conn = self.db.conn.lock().map_err(|e| {
                AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
            })?;

            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
                rusqlite::params!["user_preferences", prefs_json],
            )
            .map_err(|e| AppError::DatabaseError(format!("保存偏好设置失败: {}", e)))
        };

        execute_result?;

        let mut cache = self.cache.write().await;
        cache.preferences = prefs.clone();

        Ok(())
    }

    pub async fn save_ai_config(&self, ai: &AIConfig) -> Result<(), AppError> {
        let ai_json = serde_json::to_string(ai)
            .map_err(|e| AppError::ConfigError(format!("序列化失败: {}", e)))?;

        let execute_result = {
            let conn = self.db.conn.lock().map_err(|e| {
                AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
            })?;

            conn.execute(
                "INSERT OR REPLACE INTO app_config (key, value) VALUES (?1, ?2)",
                rusqlite::params!["ai_config", ai_json],
            )
            .map_err(|e| AppError::DatabaseError(format!("保存AI配置失败: {}", e)))
        };

        execute_result?;

        let mut cache = self.cache.write().await;
        cache.ai = ai.clone();

        Ok(())
    }

    fn load_preferences(&self) -> Result<UserPreferences, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT value FROM app_config WHERE key = 'user_preferences'")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?;

        match result {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::ConfigError(format!("反序列化失败: {}", e))),
            None => Ok(UserPreferences::default()),
        }
    }

    fn load_ai_config(&self) -> Result<AIConfig, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT value FROM app_config WHERE key = 'ai_config'")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let result: Option<String> = stmt
            .query_row([], |row| row.get(0))
            .optional()
            .map_err(|e| AppError::DatabaseError(format!("查询失败: {}", e)))?;

        match result {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| AppError::ConfigError(format!("反序列化失败: {}", e))),
            None => Ok(AIConfig::default()),
        }
    }

    fn load_whitelist(&self) -> Result<WhitelistConfig, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let mut stmt = conn
            .prepare("SELECT id, path, reason, added_at, enabled FROM whitelist WHERE enabled = 1")
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(WhitelistEntry {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    reason: row.get(2).unwrap_or_default(),
                    added_at: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                })
            })
            .map_err(|e| AppError::DatabaseError(format!("查询白名单失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(WhitelistConfig {
            paths: entries,
            global_enabled: true,
        })
    }

    pub async fn add_whitelist(&self, entry: &WhitelistEntry) -> Result<(), AppError> {
        let entry_clone = entry.clone();
        let execute_result = {
            let conn = self.db.conn.lock().map_err(|e| {
                AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
            })?;

            conn.execute(
                "INSERT OR REPLACE INTO whitelist (id, path, reason, added_at, enabled) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    entry_clone.id,
                    entry_clone.path,
                    entry_clone.reason,
                    entry_clone.added_at,
                    entry_clone.enabled as i32,
                ],
            )
            .map_err(|e| AppError::DatabaseError(format!("添加白名单失败: {}", e)))
        };

        execute_result?;

        let mut cache = self.cache.write().await;
        cache.whitelist.paths.push(entry.clone());

        Ok(())
    }

    pub async fn remove_whitelist(&self, path: &str) -> Result<(), AppError> {
        let path_owned = path.to_string();
        let execute_result = {
            let conn = self.db.conn.lock().map_err(|e| {
                AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
            })?;

            conn.execute("DELETE FROM whitelist WHERE path = ?1", rusqlite::params![path_owned])
                .map_err(|e| AppError::DatabaseError(format!("删除白名单失败: {}", e)))
        };

        execute_result?;

        let mut cache = self.cache.write().await;
        cache.whitelist.paths.retain(|e| e.path != path);

        Ok(())
    }
}
