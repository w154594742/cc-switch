#![allow(non_snake_case)]

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::services::ConfigService;
use crate::store::AppState;

/// 导出配置文件
#[tauri::command]
pub async fn export_config_to_file(file_path: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let target_path = PathBuf::from(&file_path);
        ConfigService::export_config_to_path(&target_path)?;
        Ok::<_, AppError>(json!({
            "success": true,
            "message": "Configuration exported successfully",
            "filePath": file_path
        }))
    })
    .await
    .map_err(|e| format!("导出配置失败: {}", e))?
    .map_err(|e: AppError| e.to_string())
}

/// 从文件导入配置
#[tauri::command]
pub async fn import_config_from_file(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    let (new_config, backup_id) = tauri::async_runtime::spawn_blocking(move || {
        let path_buf = PathBuf::from(&file_path);
        ConfigService::load_config_for_import(&path_buf)
    })
    .await
    .map_err(|e| format!("导入配置失败: {}", e))?
    .map_err(|e: AppError| e.to_string())?;

    {
        let mut guard = state
            .config
            .write()
            .map_err(|e| AppError::from(e).to_string())?;
        *guard = new_config;
    }

    Ok(json!({
        "success": true,
        "message": "Configuration imported successfully",
        "backupId": backup_id
    }))
}

/// 同步当前供应商配置到对应的 live 文件
#[tauri::command]
pub async fn sync_current_providers_live(state: State<'_, AppState>) -> Result<Value, String> {
    {
        let mut config_state = state
            .config
            .write()
            .map_err(|e| AppError::from(e).to_string())?;
        ConfigService::sync_current_providers_to_live(&mut config_state)
            .map_err(|e| e.to_string())?;
    }

    Ok(json!({
        "success": true,
        "message": "Live configuration synchronized"
    }))
}

/// 保存文件对话框
#[tauri::command]
pub async fn save_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    default_name: String,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name(&default_name)
        .blocking_save_file();

    Ok(result.map(|p| p.to_string()))
}

/// 打开文件对话框
#[tauri::command]
pub async fn open_file_dialog<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Option<String>, String> {
    let dialog = app.dialog();
    let result = dialog
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file();

    Ok(result.map(|p| p.to_string()))
}
