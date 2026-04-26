use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize)]
pub enum AppError {
    #[error("扫描错误: {0}")]
    ScanError(String),

    #[error("清理错误: {0}")]
    CleanError(String),

    #[error("AI服务错误: {0}")]
    AIError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(String),

    #[error("文件系统错误: {0}")]
    FileSystemError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("权限不足: {0}")]
    PermissionError(String),

    #[error("插件错误: {0}")]
    PluginError(String),

    #[error("迁移错误: {0}")]
    MigrateError(String),

    #[error("系统错误: {0}")]
    SystemError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("未知错误: {0}")]
    UnknownError(String),
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ConfigError(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::FileSystemError(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::NetworkError(e.to_string())
    }
}
