use std::sync::Arc;

use crate::error::AppError;
use crate::store::database::Database;

pub struct AuditLogger {
    db: Arc<Database>,
    encryption_key: [u8; 32],
}

impl AuditLogger {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            encryption_key: Self::derive_key("cdrive-cleaner-audit-key"),
        }
    }

    pub fn log_operation(
        &self,
        action: &str,
        target_path: &str,
        detail: Option<&str>,
        risk_level: Option<&str>,
    ) -> Result<(), AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let detail_str = detail.unwrap_or("");
        let encrypted = self.encrypt_if_needed(detail_str);

        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        conn.execute(
            "INSERT INTO audit_log (id, action, target_path, detail, risk_level, timestamp, encrypted) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                action,
                target_path,
                encrypted,
                risk_level.unwrap_or(""),
                timestamp,
                1i32,
            ],
        )
        .map_err(|e| AppError::DatabaseError(format!("写入审计日志失败: {}", e)))?;

        Ok(())
    }

    pub fn query_logs(
        &self,
        action_filter: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<AuditLogEntry>, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let sql = match action_filter {
            Some(_) => "SELECT id, action, target_path, detail, risk_level, timestamp, encrypted FROM audit_log WHERE action = ?1 ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3",
            None => "SELECT id, action, target_path, detail, risk_level, timestamp, encrypted FROM audit_log ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3",
        };

        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let rows: Vec<AuditLogEntry> = match action_filter {
            Some(action) => {
                let params = rusqlite::params![action, limit as i64, offset as i64];
                stmt.query_map(params, |row| self.row_to_entry(row))
                    .map_err(|e| AppError::DatabaseError(format!("查询审计日志失败: {}", e)))?
                    .filter_map(|r| r.ok())
                    .collect()
            }
            None => {
                let params = rusqlite::params![limit as i64, offset as i64];
                stmt.query_map(params, |row| self.row_to_entry(row))
                    .map_err(|e| AppError::DatabaseError(format!("查询审计日志失败: {}", e)))?
                    .filter_map(|r| r.ok())
                    .collect()
            }
        };
        Ok(rows)
    }

    pub fn cleanup_old_logs(&self, days: u32) -> Result<u64, AppError> {
        let conn = self.db.conn.lock().map_err(|e| {
            AppError::DatabaseError(format!("数据库锁获取失败: {}", e))
        })?;

        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

        let count = conn
            .execute(
                "DELETE FROM audit_log WHERE timestamp < ?1",
                rusqlite::params![cutoff_str],
            )
            .map_err(|e| AppError::DatabaseError(format!("清理审计日志失败: {}", e)))?;

        Ok(count as u64)
    }

    fn row_to_entry(&self, row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditLogEntry> {
        let detail_raw: String = row.get(3)?;
        let is_encrypted: i32 = row.get(6)?;

        let detail = if is_encrypted != 0 {
            self.decrypt_if_needed(&detail_raw)
        } else {
            detail_raw
        };

        Ok(AuditLogEntry {
            id: row.get(0)?,
            action: row.get(1)?,
            target_path: row.get(2)?,
            detail,
            risk_level: row.get(4)?,
            timestamp: row.get(5)?,
        })
    }

    fn encrypt_if_needed(&self, data: &str) -> String {
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce_bytes = Self::new_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        match cipher.encrypt(nonce, data.as_bytes()) {
            Ok(encrypted) => {
                let mut payload = Vec::with_capacity(nonce_bytes.len() + encrypted.len());
                payload.extend_from_slice(&nonce_bytes);
                payload.extend_from_slice(&encrypted);
                base64_encode(&payload)
            }
            Err(_) => data.to_string(),
        }
    }

    fn decrypt_if_needed(&self, data: &str) -> String {
        use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);

        match base64_decode(data) {
            Some(payload) if payload.len() > 12 => {
                let (nonce_bytes, encrypted) = payload.split_at(12);
                let nonce = Nonce::from_slice(nonce_bytes);
                match cipher.decrypt(nonce, encrypted) {
                    Ok(decrypted) => String::from_utf8_lossy(&decrypted).to_string(),
                    Err(_) => data.to_string(),
                }
            }
            None => data.to_string(),
            _ => data.to_string(),
        }
    }

    fn new_nonce() -> [u8; 12] {
        let id = uuid::Uuid::new_v4();
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&id.as_bytes()[..12]);
        nonce
    }

    fn derive_key(seed: &str) -> [u8; 32] {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        seed.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher2 = std::collections::hash_map::DefaultHasher::new();
        format!("{}-salt", seed).hash(&mut hasher2);
        let hash2 = hasher2.finish();

        let mut key = [0u8; 32];
        key[..8].copy_from_slice(&hash1.to_le_bytes());
        key[8..16].copy_from_slice(&hash2.to_le_bytes());
        key[16..24].copy_from_slice(&hash1.to_be_bytes());
        key[24..].copy_from_slice(&hash2.to_be_bytes());
        key
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
        result.push(CHARS[(b0 >> 2) as usize] as char);
        result.push(CHARS[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[(b2 & 0x3F) as usize] as char);
        }
    }
    result
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    let bytes: Vec<u8> = input.bytes().filter(|b| *b != b'=').collect();
    let chunks: Vec<&[u8]> = bytes.chunks(4).collect();
    for chunk in chunks {
        let lookup = |b: u8| CHARS.iter().position(|&c| c == b).map(|p| p as u8);
        let c0 = lookup(chunk.get(0).copied().unwrap_or(0))?;
        let c1 = lookup(chunk.get(1).copied().unwrap_or(0))?;
        result.push((c0 << 2) | (c1 >> 4));
        if chunk.len() > 2 {
            let c2 = lookup(chunk.get(2).copied().unwrap_or(0))?;
            result.push(((c1 & 0x0F) << 4) | (c2 >> 2));
            if chunk.len() > 3 {
                let c3 = lookup(chunk.get(3).copied().unwrap_or(0))?;
                result.push(((c2 & 0x03) << 6) | c3);
            }
        }
    }
    Some(result)
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    pub target_path: String,
    pub detail: String,
    pub risk_level: String,
    pub timestamp: String,
}
