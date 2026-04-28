use tauri::State;

use crate::error::AppError;
use crate::AppState;
use crate::models::config::WhitelistEntry;
use crate::models::system::{SystemInfo, DriveInfo};

#[tauri::command]
pub async fn system_info() -> Result<SystemInfo, AppError> {
    use sysinfo::{System, Disks};
    
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_default();
    let os_version = System::os_version().unwrap_or_default();
    let hostname = System::host_name().unwrap_or_default();

    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();

    let cpu_cores = sys.cpus().len() as u32;
    let total_memory = sys.total_memory();
    let available_memory = sys.available_memory();

    let disks = Disks::new_with_refreshed_list();
    let drives: Vec<DriveInfo> = disks
        .iter()
        .map(|disk| {
            let letter = disk
                .mount_point()
                .to_str()
                .unwrap_or("?")
                .to_string();
            let label = disk.name().to_str().unwrap_or("").to_string();
            let file_system = disk.file_system().to_str().unwrap_or("").to_string();
            let drive_type = format!("{:?}", disk.kind());
            let total_size = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_size.saturating_sub(available_space);

            DriveInfo {
                letter,
                label,
                file_system,
                drive_type,
                total_size,
                available_space,
                used_space,
            }
        })
        .collect();

    Ok(SystemInfo {
        os_name,
        os_version,
        os_build: String::new(),
        hostname,
        cpu_name,
        cpu_cores,
        total_memory,
        available_memory,
        drives,
    })
}

#[tauri::command]
pub async fn whitelist_manage(
    action: String,
    path: Option<String>,
    reason: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    match action.as_str() {
        "add" => {
            let p = path.ok_or_else(|| AppError::ConfigError("路径不能为空".to_string()))?;
            let entry = WhitelistEntry {
                id: uuid::Uuid::new_v4().to_string(),
                path: p,
                reason: reason.unwrap_or_else(|| "用户手动添加".to_string()),
                added_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                enabled: true,
            };
            state.config_store.add_whitelist(&entry).await
        }
        "remove" => {
            let p = path.ok_or_else(|| AppError::ConfigError("路径不能为空".to_string()))?;
            state.config_store.remove_whitelist(&p).await
        }
        _ => Err(AppError::ConfigError(format!("未知操作: {}", action))),
    }
}
