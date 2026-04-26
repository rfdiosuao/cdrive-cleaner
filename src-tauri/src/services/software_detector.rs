use std::path::PathBuf;

use crate::error::AppError;

pub struct SoftwareDetector;

impl SoftwareDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_wechat_paths(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let wechat_doc = format!(r"{}\Documents\WeChat Files", user_profile);
        if PathBuf::from(&wechat_doc).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "微信".to_string(),
                cache_path: wechat_doc,
                category: "社交软件".to_string(),
                description: "微信聊天记录和缓存文件".to_string(),
            });
        }

        let wechat_appdata = format!(r"{}\AppData\Roaming\Tencent\WeChat", user_profile);
        if PathBuf::from(&wechat_appdata).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "微信".to_string(),
                cache_path: wechat_appdata,
                category: "社交软件".to_string(),
                description: "微信应用数据".to_string(),
            });
        }

        results
    }

    pub fn detect_qq_paths(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let qq_doc = format!(r"{}\Documents\Tencent Files", user_profile);
        if PathBuf::from(&qq_doc).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "QQ".to_string(),
                cache_path: qq_doc,
                category: "社交软件".to_string(),
                description: "QQ聊天记录和缓存文件".to_string(),
            });
        }

        let qq_appdata = format!(r"{}\AppData\Roaming\Tencent\QQ", user_profile);
        if PathBuf::from(&qq_appdata).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "QQ".to_string(),
                cache_path: qq_appdata,
                category: "社交软件".to_string(),
                description: "QQ应用数据".to_string(),
            });
        }

        results
    }

    pub fn detect_wechat_work_paths(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let work_path = format!(r"{}\Documents\WXWork", user_profile);
        if PathBuf::from(&work_path).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "企业微信".to_string(),
                cache_path: work_path,
                category: "社交软件".to_string(),
                description: "企业微信聊天记录和缓存".to_string(),
            });
        }

        results
    }

    pub fn detect_chrome_cache(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let chrome_cache = format!(r"{}\AppData\Local\Google\Chrome\User Data\Default\Cache", user_profile);
        if PathBuf::from(&chrome_cache).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "Chrome".to_string(),
                cache_path: chrome_cache,
                category: "浏览器".to_string(),
                description: "Chrome浏览器缓存".to_string(),
            });
        }

        let chrome_code_cache = format!(r"{}\AppData\Local\Google\Chrome\User Data\Default\Code Cache", user_profile);
        if PathBuf::from(&chrome_code_cache).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "Chrome".to_string(),
                cache_path: chrome_code_cache,
                category: "浏览器".to_string(),
                description: "Chrome代码缓存".to_string(),
            });
        }

        results
    }

    pub fn detect_edge_cache(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let edge_cache = format!(r"{}\AppData\Local\Microsoft\Edge\User Data\Default\Cache", user_profile);
        if PathBuf::from(&edge_cache).exists() {
            results.push(SoftwareCacheInfo {
                software_name: "Edge".to_string(),
                cache_path: edge_cache,
                category: "浏览器".to_string(),
                description: "Edge浏览器缓存".to_string(),
            });
        }

        results
    }

    pub fn detect_firefox_cache(&self) -> Vec<SoftwareCacheInfo> {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let mut results = Vec::new();

        let firefox_path = format!(r"{}\AppData\Local\Mozilla\Firefox\Profiles", user_profile);
        if PathBuf::from(&firefox_path).exists() {
            if let Ok(entries) = std::fs::read_dir(&firefox_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let cache_dir = path.join("cache2");
                        if cache_dir.exists() {
                            results.push(SoftwareCacheInfo {
                                software_name: "Firefox".to_string(),
                                cache_path: cache_dir.to_str().unwrap_or("").to_string(),
                                category: "浏览器".to_string(),
                                description: "Firefox浏览器缓存".to_string(),
                            });
                        }
                    }
                }
            }
        }

        results
    }

    pub fn detect_all(&self) -> Vec<SoftwareCacheInfo> {
        let mut all = Vec::new();
        all.extend(self.detect_wechat_paths());
        all.extend(self.detect_qq_paths());
        all.extend(self.detect_wechat_work_paths());
        all.extend(self.detect_chrome_cache());
        all.extend(self.detect_edge_cache());
        all.extend(self.detect_firefox_cache());
        all
    }

    pub fn detect_installed_software(&self) -> Vec<InstalledSoftware> {
        let mut results = Vec::new();

        #[cfg(target_os = "windows")]
        {
            let uninstall_paths = [
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ];

            use windows::Win32::System::Registry::*;

            for path in &uninstall_paths {
                let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
                let mut key: HKEY = HKEY(0isize);
                let result = unsafe {
                    RegOpenKeyExW(
                        HKEY_LOCAL_MACHINE,
                        windows::core::PCWSTR(path_wide.as_ptr()),
                        0,
                        KEY_READ | KEY_WOW64_64KEY,
                        &mut key,
                    )
                };
                if result.is_err() {
                    continue;
                }

                let mut index = 0u32;
                loop {
                    let mut name_buf = [0u16; 256];
                    let mut name_len = name_buf.len() as u32;
                    let mut class_buf = [0u16; 256];
                    let mut class_len = class_buf.len() as u32;
                    let mut ft: windows::Win32::Foundation::FILETIME = windows::Win32::Foundation::FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
                    let result = unsafe {
                        RegEnumKeyExW(
                            key,
                            index,
                            windows::core::PWSTR(name_buf.as_mut_ptr()),
                            &mut name_len,
                            None,
                            windows::core::PWSTR(class_buf.as_mut_ptr()),
                            Some(&mut class_len),
                            Some(&mut ft),
                        )
                    };
                    if result.is_err() {
                        break;
                    }
                    index += 1;

                    let subkey_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    let full_path = format!(r"{}\{}", path, subkey_name);

                    let full_path_wide: Vec<u16> = full_path.encode_utf16().chain(std::iter::once(0)).collect();
                    let mut subkey: HKEY = HKEY(0isize);
                    let result = unsafe {
                        RegOpenKeyExW(
                            HKEY_LOCAL_MACHINE,
                            windows::core::PCWSTR(full_path_wide.as_ptr()),
                            0,
                            KEY_READ | KEY_WOW64_64KEY,
                            &mut subkey,
                        )
                    };
                    if result.is_ok() {
                        let display_name = Self::read_reg_string(&subkey, "DisplayName");
                        let install_location = Self::read_reg_string(&subkey, "InstallLocation");
                        let estimated_size = Self::read_reg_u32(&subkey, "EstimatedSize");

                        if let Some(name) = display_name {
                            results.push(InstalledSoftware {
                                name,
                                install_path: install_location.unwrap_or_default(),
                                estimated_size: estimated_size.unwrap_or(0) as u64 * 1024,
                                publisher: Self::read_reg_string(&subkey, "Publisher").unwrap_or_default(),
                                version: Self::read_reg_string(&subkey, "DisplayVersion").unwrap_or_default(),
                            });
                        }
                    }
                    unsafe { RegCloseKey(subkey); }
                }
                unsafe { RegCloseKey(key); }
            }
        }

        results
    }

    #[cfg(target_os = "windows")]
    fn read_reg_string(key: &windows::Win32::System::Registry::HKEY, name: &str) -> Option<String> {
        use windows::Win32::System::Registry::*;

        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut buf = [0u16; 512];
        let mut buf_len = (buf.len() * 2) as u32;
        let mut reg_type: REG_VALUE_TYPE = REG_NONE;

        let result = unsafe {
            RegQueryValueExW(
                *key,
                windows::core::PCWSTR(name_wide.as_ptr()),
                None,
                Some(&mut reg_type),
                Some(buf.as_mut_ptr() as *mut u8),
                Some(&mut buf_len),
            )
        };

        if result.is_ok() && reg_type == REG_SZ {
            let len = (buf_len as usize) / 2;
            if len > 0 && buf[len - 1] == 0 {
                Some(String::from_utf16_lossy(&buf[..len - 1]))
            } else {
                Some(String::from_utf16_lossy(&buf[..len]))
            }
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    fn read_reg_u32(key: &windows::Win32::System::Registry::HKEY, name: &str) -> Option<u32> {
        use windows::Win32::System::Registry::*;

        let name_wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut value = 0u32;
        let mut buf_len = std::mem::size_of::<u32>() as u32;
        let mut reg_type: REG_VALUE_TYPE = REG_NONE;

        let result = unsafe {
            RegQueryValueExW(
                *key,
                windows::core::PCWSTR(name_wide.as_ptr()),
                None,
                Some(&mut reg_type),
                Some(&mut value as *mut u32 as *mut u8),
                Some(&mut buf_len),
            )
        };

        if result.is_ok() && reg_type == REG_DWORD {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct SoftwareCacheInfo {
    pub software_name: String,
    pub cache_path: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct InstalledSoftware {
    pub name: String,
    pub install_path: String,
    pub estimated_size: u64,
    pub publisher: String,
    pub version: String,
}
