use std::path::Path;
use std::ptr;

use crate::error::AppError;

#[repr(C)]
#[allow(non_snake_case)]
struct WINTRUST_FILE_INFO {
    cbStruct: u32,
    pcwszFilePath: windows::core::PCWSTR,
    hFile: windows::Win32::Foundation::HANDLE,
    pgKnownSubject: *mut windows::core::GUID,
}

#[repr(C)]
#[allow(non_snake_case)]
union WINTRUST_DATA_UNION {
    pFile: *mut WINTRUST_FILE_INFO,
}

#[repr(C)]
#[allow(non_snake_case)]
struct WINTRUST_DATA {
    cbStruct: u32,
    pPolicyCallbackData: *mut std::ffi::c_void,
    pSIPClientData: *mut std::ffi::c_void,
    dwUIChoice: u32,
    fdwRevocationChecks: u32,
    dwUnionChoice: u32,
    Anonymous: WINTRUST_DATA_UNION,
    dwStateAction: u32,
    hWVTStateData: windows::Win32::Foundation::HANDLE,
    pwszURLReference: windows::core::PWSTR,
    dwProvFlags: u32,
    dwUIContext: u32,
}

// WinVerifyTrust is not exposed by the windows crate, so we link it directly
#[cfg(target_os = "windows")]
extern "system" {
    fn WinVerifyTrust(
        hwnd: isize,
        pgActionID: *const windows::core::GUID,
        pWVTData: *mut std::ffi::c_void,
    ) -> i32;
}

pub struct SignatureVerifier;

impl SignatureVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn verify_signature(&self, file_path: &str) -> Result<SignatureInfo, AppError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::FileSystemError(format!("文件不存在: {}", file_path)));
        }

        #[cfg(target_os = "windows")]
        {
            self.verify_authenticode(file_path)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(SignatureInfo {
                is_signed: false,
                is_microsoft: false,
                is_trusted: false,
                signer_name: String::new(),
                issuer: String::new(),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn verify_authenticode(&self, file_path: &str) -> Result<SignatureInfo, AppError> {
        let file_path_wide: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let mut file_info = WINTRUST_FILE_INFO {
                cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
                pcwszFilePath: windows::core::PCWSTR(file_path_wide.as_ptr()),
                hFile: windows::Win32::Foundation::INVALID_HANDLE_VALUE,
                pgKnownSubject: std::ptr::null_mut(),
            };

            let mut trust_data = WINTRUST_DATA {
                cbStruct: std::mem::size_of::<WINTRUST_DATA>() as u32,
                pPolicyCallbackData: std::ptr::null_mut(),
                pSIPClientData: std::ptr::null_mut(),
                dwUIChoice: 2, // WTD_UI_NONE
                fdwRevocationChecks: 0x00000001, // WTD_REVOKE_NONE
                dwUnionChoice: 1, // WTD_CHOICE_FILE
                Anonymous: WINTRUST_DATA_UNION { pFile: &mut file_info },
                dwStateAction: 0,
                hWVTStateData: windows::Win32::Foundation::HANDLE(0isize),
                pwszURLReference: windows::core::PWSTR::null(),
                dwProvFlags: 0x00000000,
                dwUIContext: 0,
            };

            let wintrust_action = windows::core::GUID::from_values(
                0x00aac56b,
                0xcd44,
                0x11d0,
                [0x8c, 0xc2, 0x00, 0xc0, 0x4f, 0xc2, 0x95, 0xee],
            );

            let result = WinVerifyTrust(
                0, // HWND_DESKTOP
                &wintrust_action,
                &mut trust_data as *mut _ as *mut std::ffi::c_void,
            );

            // ERROR_SUCCESS = 0
            let is_trusted = result == 0;

            let signer_name = self.extract_signer_name(file_path);

            let is_microsoft = signer_name
                .as_ref()
                .map(|n| n.contains("Microsoft") || n.contains("微软"))
                .unwrap_or(false);

            Ok(SignatureInfo {
                is_signed: is_trusted || signer_name.as_ref().map_or(false, |s| !s.is_empty()),
                is_microsoft,
                is_trusted,
                signer_name: signer_name.unwrap_or_default(),
                issuer: String::new(),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn extract_signer_name(&self, file_path: &str) -> Option<String> {
        use windows::Win32::Security::Cryptography::*;

        let file_path_wide: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let mut encoding = CERT_QUERY_ENCODING_TYPE(0);
            let mut msg_type = CERT_QUERY_CONTENT_TYPE(0);
            let mut msg_format = CERT_QUERY_FORMAT_TYPE(0);
            let mut cert_store: HCERTSTORE = HCERTSTORE(ptr::null_mut());
            let mut msg_handle: *mut std::ffi::c_void = std::ptr::null_mut();

            let result = CryptQueryObject(
                CERT_QUERY_OBJECT_FILE,
                windows::core::PCWSTR(file_path_wide.as_ptr()).0 as *const _,
                CERT_QUERY_CONTENT_FLAG_ALL,
                CERT_QUERY_FORMAT_FLAG_ALL,
                0,
                Some(&mut encoding),
                Some(&mut msg_type),
                Some(&mut msg_format),
                Some(&mut cert_store as *mut HCERTSTORE),
                Some(&mut msg_handle),
                None,
            );

            if result.is_err() {
                return None;
            }

            if !cert_store.0.is_null() {
                return Some(String::new());
            }

            None
        }
    }

    pub fn is_system_file(&self, file_path: &str) -> bool {
        let system_dirs = [
            r"C:\Windows",
            r"C:\Program Files",
            r"C:\Program Files (x86)",
        ];

        system_dirs.iter().any(|dir| file_path.starts_with(dir))
    }

    pub fn is_system_extension(&self, extension: &str) -> bool {
        let system_extensions = [
            "dll", "sys", "exe", "ocx", "drv", "inf", "cat", "msi", "mui",
        ];

        system_extensions.contains(&extension.to_lowercase().as_str())
    }
}

#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub is_signed: bool,
    pub is_microsoft: bool,
    pub is_trusted: bool,
    pub signer_name: String,
    pub issuer: String,
}
