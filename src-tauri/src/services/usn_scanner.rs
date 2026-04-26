use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::models::scan::FileMetadata;

#[repr(C)]
#[derive(Default)]
struct MFT_ENUM_DATA_V0 {
    StartFileReferenceNumber: i64,
    LowUsn: i64,
    HighUsn: i64,
    MinMajorVersion: u32,
    MaxMajorVersion: u32,
}

// DeviceIoControl and FSCTL_ENUM_USN_DATA are not exposed by the windows crate
#[cfg(target_os = "windows")]
extern "system" {
    fn DeviceIoControl(
        hDevice: windows::Win32::Foundation::HANDLE,
        dwIoControlCode: u32,
        lpInBuffer: *const std::ffi::c_void,
        nInBufferSize: u32,
        lpOutBuffer: *mut std::ffi::c_void,
        nOutBufferSize: u32,
        lpBytesReturned: *mut u32,
        lpOverlapped: *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(target_os = "windows")]
const FSCTL_ENUM_USN_DATA: u32 = 0x000900b4;

#[derive(Clone)]
pub struct UsnScanner {
    last_usn: Arc<RwLock<u64>>,
    drive_letter: String,
}

impl UsnScanner {
    pub fn new(drive_letter: &str) -> Self {
        Self {
            last_usn: Arc::new(RwLock::new(0)),
            drive_letter: drive_letter.to_uppercase(),
        }
    }

    pub async fn read_usn_journal(&self) -> Result<Vec<UsnEntry>, AppError> {
        #[cfg(target_os = "windows")]
        {
            self.read_usn_journal_windows()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Ok(vec![])
        }
    }

    #[cfg(target_os = "windows")]
    fn read_usn_journal_windows(&self) -> Result<Vec<UsnEntry>, AppError> {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        use windows::Win32::Foundation::*;
        use windows::Win32::Storage::FileSystem::*;

        let drive_path = format!(r"\\.\{}:", self.drive_letter.trim_end_matches(':'));
        let os_str = OsStr::new(&drive_path);
        let drive_path_wide: Vec<u16> = os_str.encode_wide().chain(std::iter::once(0)).collect();

        unsafe {
            let handle = CreateFileW(
                windows::core::PCWSTR(drive_path_wide.as_ptr()),
                FILE_READ_ATTRIBUTES.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                None,
            )
            .map_err(|e| AppError::ScanError(format!("打开卷失败: {:?}", e)))?;

            if handle.is_invalid() {
                return Err(AppError::ScanError("无法打开卷句柄".to_string()));
            }

            let entries = self.enum_usn_data(&handle)?;

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            Ok(entries)
        }
    }

    #[cfg(target_os = "windows")]
    unsafe fn enum_usn_data(&self, handle: &windows::Win32::Foundation::HANDLE) -> Result<Vec<UsnEntry>, AppError> {
        use windows::Win32::Storage::FileSystem::*;

        let buffer_size = 65536u32;
        let mut buffer = vec![0u8; buffer_size as usize];
        let mut bytes_returned = 0u32;

        let start_usn = self.last_usn.try_read().map(|g| *g).unwrap_or(0) as i64;

        let mut med = MFT_ENUM_DATA_V0::default();
        med.StartFileReferenceNumber = 0;
        med.LowUsn = start_usn;
        med.HighUsn = i64::MAX;
        med.MinMajorVersion = 2;
        med.MaxMajorVersion = 3;

        let success = DeviceIoControl(
            *handle,
            FSCTL_ENUM_USN_DATA,
            &med as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<MFT_ENUM_DATA_V0>() as u32,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            buffer_size,
            &mut bytes_returned,
            std::ptr::null_mut(),
        );

        if success == 0 {
            return Ok(vec![]);
        }

        if bytes_returned < 8 {
            return Ok(vec![]);
        }

        let mut entries = Vec::new();
        let mut offset = 8usize;

        while offset + 8 < bytes_returned as usize {
            if offset + 4 > buffer.len() {
                break;
            }
            let record_len = u32::from_le_bytes(
                buffer[offset..offset + 4].try_into().unwrap_or([0; 4])
            ) as usize;
            if record_len == 0 || offset + record_len > bytes_returned as usize || offset + record_len > buffer.len() {
                break;
            }

            if offset + 60 > buffer.len() {
                break;
            }
            let usn = i64::from_le_bytes(
                buffer[offset + 8..offset + 16].try_into().unwrap_or([0; 8])
            );
            let reason = u32::from_le_bytes(
                buffer[offset + 16..offset + 20].try_into().unwrap_or([0; 4])
            );
            let file_name_len = u32::from_le_bytes(
                buffer[offset + 56..offset + 60].try_into().unwrap_or([0; 4])
            ) as usize;
            let file_name_offset = offset + 60;

            let file_name = if file_name_offset + file_name_len <= buffer.len() && file_name_len > 0 {
                let name_bytes: Vec<u16> = buffer[file_name_offset..file_name_offset + file_name_len]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                String::from_utf16_lossy(&name_bytes)
            } else {
                String::new()
            };

            let change_type = Self::reason_to_change_type(reason);

            entries.push(UsnEntry {
                usn: usn as u64,
                file_name,
                change_type,
            });

            offset += record_len;
            if record_len == 0 {
                break;
            }
        }

        Ok(entries)
    }

    fn reason_to_change_type(reason: u32) -> UsnChangeType {
        if reason & 0x00000001 != 0 || reason & 0x00008000 != 0 {
            UsnChangeType::Created
        } else if reason & 0x00000002 != 0 {
            UsnChangeType::Deleted
        } else if reason & 0x00010000 != 0 {
            UsnChangeType::Renamed
        } else {
            UsnChangeType::Modified
        }
    }

    pub async fn get_changed_files(&self) -> Result<Vec<FileMetadata>, AppError> {
        let usn_entries = self.read_usn_journal().await?;

        let changed_files: Vec<FileMetadata> = usn_entries
            .into_iter()
            .filter(|e| e.change_type != UsnChangeType::Deleted)
            .map(|e| FileMetadata {
                path: e.file_name.clone(),
                name: e.file_name,
                size: 0,
                modified_at: String::new(),
                created_at: String::new(),
                is_directory: false,
                extension: String::new(),
                category: crate::models::scan::ScanCategory::Other,
                hash: String::new(),
            })
            .collect();

        Ok(changed_files)
    }

    pub async fn update_last_usn(&self, usn: u64) {
        let mut last = self.last_usn.write().await;
        *last = usn;
    }

    pub async fn get_last_usn(&self) -> u64 {
        *self.last_usn.read().await
    }
}

#[derive(Debug, Clone)]
pub struct UsnEntry {
    pub usn: u64,
    pub file_name: String,
    pub change_type: UsnChangeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UsnChangeType {
    Created,
    Deleted,
    Modified,
    Renamed,
}
