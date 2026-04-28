use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::models::scan::RiskLevel;

pub struct SafetyGuard {
    protected_dirs: Arc<Vec<PathBuf>>,
    protected_extensions: Arc<Vec<String>>,
    privacy_dirs: Arc<Vec<PathBuf>>,
    safe_cleanup_dirs: Arc<Vec<PathBuf>>,
    signature_verifier: Arc<crate::services::signature_verifier::SignatureVerifier>,
}

impl SafetyGuard {
    pub fn new() -> Self {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();

        Self {
            protected_dirs: Arc::new(vec![
                PathBuf::from(r"C:\Windows"),
                PathBuf::from(r"C:\Windows\System32"),
                PathBuf::from(r"C:\Windows\SysWOW64"),
                PathBuf::from(r"C:\Program Files"),
                PathBuf::from(r"C:\Program Files (x86)"),
                PathBuf::from(r"C:\ProgramData"),
                PathBuf::from(r"C:\Boot"),
                PathBuf::from(r"C:\Recovery"),
                PathBuf::from(r"C:\System Volume Information"),
                PathBuf::from(r"C:\$Recycle.Bin"),
            ]),
            protected_extensions: Arc::new(vec![
                "dll".to_string(),
                "sys".to_string(),
                "exe".to_string(),
                "ocx".to_string(),
                "drv".to_string(),
                "inf".to_string(),
                "cat".to_string(),
                "msi".to_string(),
                "mui".to_string(),
                "efi".to_string(),
                "iso".to_string(),
                "vhd".to_string(),
                "vhdx".to_string(),
            ]),
            privacy_dirs: Arc::new(vec![
                PathBuf::from(format!(r"{}\Documents", user_profile)),
                PathBuf::from(format!(r"{}\Pictures", user_profile)),
                PathBuf::from(format!(r"{}\Videos", user_profile)),
                PathBuf::from(format!(r"{}\Music", user_profile)),
                PathBuf::from(format!(r"{}\Desktop", user_profile)),
                PathBuf::from(format!(r"{}\Downloads", user_profile)),
                PathBuf::from(format!(r"{}\OneDrive", user_profile)),
            ]),
            safe_cleanup_dirs: Arc::new(vec![
                PathBuf::from(r"C:\Windows\Temp"),
                PathBuf::from(r"C:\Windows\Prefetch"),
                PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"),
                PathBuf::from(r"C:\$Recycle.Bin"),
            ]),
            signature_verifier: Arc::new(crate::services::signature_verifier::SignatureVerifier::new()),
        }
    }

    pub fn check_path_safety(&self, file_path: &str) -> PathSafetyResult {
        let path = PathBuf::from(file_path);
        let is_known_cleanup_path = self
            .safe_cleanup_dirs
            .iter()
            .any(|safe_dir| Self::path_is_same_or_child(&path, safe_dir));

        for protected in self.protected_dirs.iter() {
            if !is_known_cleanup_path && Self::path_is_same_or_child(&path, protected) {
                return PathSafetyResult {
                    is_safe: false,
                    risk_level: RiskLevel::Critical,
                    reason: format!("一级防护：系统核心目录白名单 - {:?}", protected),
                    protection_level: ProtectionLevel::Level1DirectoryWhitelist,
                };
            }
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.protected_extensions.contains(&ext.to_lowercase()) {
                return PathSafetyResult {
                    is_safe: false,
                    risk_level: RiskLevel::High,
                    reason: format!("二级防护：系统文件类型白名单 - .{}", ext),
                    protection_level: ProtectionLevel::Level2ExtensionWhitelist,
                };
            }
        }

        for privacy in self.privacy_dirs.iter() {
            if Self::path_is_same_or_child(&path, privacy) {
                return PathSafetyResult {
                    is_safe: false,
                    risk_level: RiskLevel::Medium,
                    reason: format!("隐私保护：用户隐私目录 - {:?}", privacy),
                    protection_level: ProtectionLevel::PrivacyProtection,
                };
            }
        }

        PathSafetyResult {
            is_safe: true,
            risk_level: RiskLevel::Safe,
            reason: String::new(),
            protection_level: ProtectionLevel::None,
        }
    }

    pub async fn check_signature_safety(&self, file_path: &str) -> PathSafetyResult {
        match self.signature_verifier.verify_signature(file_path) {
            Ok(info) => {
                if info.is_microsoft {
                    PathSafetyResult {
                        is_safe: false,
                        risk_level: RiskLevel::Critical,
                        reason: format!("三级防护：微软数字签名文件 - {}", info.signer_name),
                        protection_level: ProtectionLevel::Level3SignatureVerification,
                    }
                } else if info.is_signed && info.is_trusted {
                    PathSafetyResult {
                        is_safe: false,
                        risk_level: RiskLevel::High,
                        reason: format!("可信签名文件 - {}", info.signer_name),
                        protection_level: ProtectionLevel::Level3SignatureVerification,
                    }
                } else {
                    PathSafetyResult {
                        is_safe: true,
                        risk_level: RiskLevel::Safe,
                        reason: String::new(),
                        protection_level: ProtectionLevel::None,
                    }
                }
            }
            Err(_) => PathSafetyResult {
                is_safe: true,
                risk_level: RiskLevel::Low,
                reason: "无法验证签名".to_string(),
                protection_level: ProtectionLevel::None,
            },
        }
    }

    pub fn filter_safe_files(&self, files: &[crate::models::scan::FileMetadata]) -> Vec<crate::models::scan::FileMetadata> {
        files
            .iter()
            .filter(|f| self.check_path_safety(&f.path).is_safe)
            .cloned()
            .collect()
    }

    pub fn requires_confirmation(&self, risk_level: &RiskLevel, is_beginner_mode: bool) -> bool {
        match risk_level {
            RiskLevel::Safe => false,
            RiskLevel::Low => is_beginner_mode,
            RiskLevel::Medium => true,
            RiskLevel::High => true,
            RiskLevel::Critical => true,
        }
    }

    pub fn is_hidden_for_beginner(&self, risk_level: &RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::High | RiskLevel::Critical)
    }

    pub fn get_privacy_dirs(&self) -> &[PathBuf] {
        &self.privacy_dirs
    }

    pub fn get_protected_dirs(&self) -> &[PathBuf] {
        &self.protected_dirs
    }

    pub fn get_protected_extensions(&self) -> &[String] {
        &self.protected_extensions
    }

    fn path_is_same_or_child(path: &Path, base: &Path) -> bool {
        let path = Self::normalize_path(path);
        let base = Self::normalize_path(base);

        path == base || path.starts_with(&format!("{}\\", base))
    }

    fn normalize_path(path: &Path) -> String {
        path.to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    }
}

#[derive(Debug, Clone)]
pub struct PathSafetyResult {
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub reason: String,
    pub protection_level: ProtectionLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtectionLevel {
    None,
    Level1DirectoryWhitelist,
    Level2ExtensionWhitelist,
    Level3SignatureVerification,
    PrivacyProtection,
}
