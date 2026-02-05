use tauri::{command, AppHandle, Manager};
use serde_json::json;
use crate::services;

/// 获取应用版本信息
#[command]
pub async fn get_app_version(app: AppHandle) -> Result<serde_json::Value, String> {
    let package_info = app.package_info();
    
    Ok(json!({
        "version": package_info.version.to_string(),
        "name": package_info.name,
        "authors": package_info.authors,
        "description": package_info.description
    }))
}

/// 获取应用标题（包含版本号）
#[command]
pub async fn get_app_title(app: AppHandle) -> Result<String, String> {
    let version = app.package_info().version.to_string();
    Ok(format!("windsurf-account-manager-simple v{}", version))
}

/// 重置HTTP客户端（用于从网络故障中恢复）
#[command]
pub async fn reset_http_client() -> Result<serde_json::Value, String> {
    services::rebuild_http_client();
    Ok(json!({
        "success": true,
        "message": "HTTP客户端已重置"
    }))
}

/// 打开开发者工具（仅开发模式可用）
#[command]
pub async fn open_devtools(app: AppHandle) -> Result<serde_json::Value, String> {
    #[cfg(debug_assertions)]
    {
        if let Some(window) = app.get_webview_window("main") {
            window.open_devtools();
            return Ok(json!({
                "success": true,
                "message": "DevTools已打开"
            }));
        } else {
            return Err("找不到主窗口".to_string());
        }
    }
    
    #[cfg(not(debug_assertions))]
    {
        let _ = app;
        Err("DevTools仅在开发模式下可用".to_string())
    }
}
