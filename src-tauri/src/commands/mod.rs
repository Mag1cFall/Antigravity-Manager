use crate::models::{Account, TokenData, QuotaData, AppConfig};
use crate::modules;
use tauri::Emitter;

// 导出 proxy 命令
pub mod proxy;

/// 列出所有账号
#[tauri::command]
pub async fn list_accounts() -> Result<Vec<Account>, String> {
    modules::list_accounts()
}

/// 添加账号
#[tauri::command]
pub async fn add_account(_email: String, refresh_token: String) -> Result<Account, String> {
    // 1. 使用 refresh_token 获取 access_token
    // 注意：这里我们忽略传入的 _email，而是直接去 Google 获取真实的邮箱
    let token_res = modules::oauth::refresh_access_token(&refresh_token).await?;

    // 2. 获取用户信息
    let user_info = modules::oauth::get_user_info(&token_res.access_token).await?;
    
    // 3. 构造 TokenData
    let token = TokenData::new(
        token_res.access_token,
        refresh_token, // 继续使用用户传入的 refresh_token
        token_res.expires_in,
        Some(user_info.email.clone()),
        None, // project_id 将在需要时获取
        None,  // session_id
    );
    
    // 4. 使用真实的 email 添加或更新账号
    let account = modules::upsert_account(user_info.email.clone(), user_info.get_display_name(), token)?;
    
    modules::logger::log_info(&format!("添加账号成功: {}", account.email));
    
    Ok(account)
}

/// 删除账号
#[tauri::command]
pub async fn delete_account(account_id: String) -> Result<(), String> {
    modules::logger::log_info(&format!("收到删除账号请求: {}", account_id));
    modules::delete_account(&account_id).map_err(|e| {
        modules::logger::log_error(&format!("删除账号失败: {}", e));
        e
    })?;
    modules::logger::log_info(&format!("账号删除成功: {}", account_id));
    Ok(())
}

/// 切换账号
#[tauri::command]
pub async fn switch_account(app: tauri::AppHandle, account_id: String) -> Result<(), String> {
    let res = modules::switch_account(&account_id).await;
    if res.is_ok() {
        crate::modules::tray::update_tray_menus(&app);
    }
    res
}

/// 获取当前账号
#[tauri::command]
pub async fn get_current_account() -> Result<Option<Account>, String> {
    // println!("🚀 Backend Command: get_current_account called"); // Commented out to reduce noise for frequent calls, relies on frontend log for frequency
    // Actually user WANTS to see it.
    modules::logger::log_info("Backend Command: get_current_account called");
    
    let account_id = modules::get_current_account_id()?;
    
    if let Some(id) = account_id {
        // modules::logger::log_info(&format!("   Found current account ID: {}", id));
        modules::load_account(&id).map(Some)
    } else {
        modules::logger::log_info("   No current account set");
        Ok(None)
    }
}



/// 查询账号配额
#[tauri::command]
pub async fn fetch_account_quota(app: tauri::AppHandle, account_id: String) -> crate::error::AppResult<QuotaData> {
    modules::logger::log_info(&format!("手动刷新配额请求: {}", account_id));
    let mut account = modules::load_account(&account_id).map_err(crate::error::AppError::Account)?;
    
    // 使用带重试的查询 (Shared logic)
    let quota = modules::account::fetch_quota_with_retry(&mut account).await?;
    
    // 4. 更新账号配额
    modules::update_account_quota(&account_id, quota.clone()).map_err(crate::error::AppError::Account)?;
    
    crate::modules::tray::update_tray_menus(&app);

    Ok(quota)
}

#[derive(serde::Serialize)]
pub struct RefreshStats {
    total: usize,
    success: usize,
    failed: usize,
    details: Vec<String>,
}

/// 刷新所有账号配额
#[tauri::command]
pub async fn refresh_all_quotas() -> Result<RefreshStats, String> {
    modules::logger::log_info("开始批量刷新所有账号配额");
    let accounts = modules::list_accounts()?;
    
    let mut success = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    // 串行处理以确保持久化安全 (SQLite)
    for mut account in accounts {
        if let Some(ref q) = account.quota {
            if q.is_forbidden {
                modules::logger::log_info(&format!("  - Skipping {} (Forbidden)", account.email));
                continue;
            }
        }
        
        modules::logger::log_info(&format!("  - Processing {}", account.email));
        
        match modules::account::fetch_quota_with_retry(&mut account).await {
            Ok(quota) => {
                 // 保存配额
                 if let Err(e) = modules::update_account_quota(&account.id, quota) {
                     failed += 1;
                     let msg = format!("Account {}: Save quota failed - {}", account.email, e);
                     details.push(msg.clone());
                     modules::logger::log_error(&msg);
                 } else {
                     success += 1;
                     modules::logger::log_info("    ✅ Success");
                 }
            },
            Err(e) => {
                failed += 1;
                // e might be AppError, assume it implements Display
                let msg = format!("Account {}: Fetch quota failed - {}", account.email, e);
                details.push(msg.clone());
                modules::logger::log_error(&msg);
            }
        }
    }
    
    modules::logger::log_info(&format!("批量刷新完成: {} 成功, {} 失败", success, failed));
    Ok(RefreshStats { total: success + failed, success, failed, details })
}

/// 加载配置
#[tauri::command]
pub async fn load_config() -> Result<AppConfig, String> {
    modules::load_app_config()
}

/// 保存配置
#[tauri::command]
pub async fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    modules::save_app_config(&config)?;
    
    // 通知托盘配置已更新
    let _ = app.emit("config://updated", ());
    
    Ok(())
}

// --- OAuth 命令 ---

#[tauri::command]
pub async fn start_oauth_login(app_handle: tauri::AppHandle) -> Result<Account, String> {
    // 1. 启动 OAuth 流程获取 Token
    let token_res = modules::oauth_server::start_oauth_flow(app_handle).await?;
    
    // 2. 获取用户信息
    let user_info = modules::oauth::get_user_info(&token_res.access_token).await?;
    
    // 3. 尝试获取项目ID
    let project_id = crate::proxy::project_resolver::fetch_project_id(&token_res.access_token)
        .await
        .ok(); // 失败不阻塞，后续可懒加载
    
    // 4. 构造 TokenData
    let token_data = TokenData::new(
        token_res.access_token,
        token_res.refresh_token.ok_or("未获取到 Refresh Token")?,
        token_res.expires_in,
        Some(user_info.email.clone()),
        project_id, // 保存项目ID
        None,  // session_id
    );
    
    // 5. 添加或更新到账号列表
    modules::upsert_account(user_info.email.clone(), user_info.get_display_name(), token_data)
}

#[tauri::command]
pub async fn cancel_oauth_login() -> Result<(), String> {
    modules::oauth_server::cancel_oauth_flow();
    Ok(())
}

// --- 导入命令 ---

#[tauri::command]
pub async fn import_v1_accounts() -> Result<Vec<Account>, String> {
    modules::migration::import_from_v1().await
}

#[tauri::command]
pub async fn import_from_db() -> Result<Account, String> {
    // 同步函数包装为 async
    modules::migration::import_from_db().await
}

/// 保存文本文件 (绕过前端 Scope 限制)
#[tauri::command]
pub async fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 清理日志缓存
#[tauri::command]
pub async fn clear_log_cache() -> Result<(), String> {
    modules::logger::clear_logs()
}

/// 打开数据目录
#[tauri::command]
pub async fn open_data_folder() -> Result<(), String> {
    let path = modules::account::get_data_dir()?;
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {}", e))?;
    }

    Ok(())
}

/// 获取数据目录绝对路径
#[tauri::command]
pub async fn get_data_dir_path() -> Result<String, String> {
    let path = modules::account::get_data_dir()?;
    Ok(path.to_string_lossy().to_string())
}

/// 显示主窗口
#[tauri::command]
pub async fn show_main_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())
}

/// 获取 Antigravity 可执行文件路径
#[tauri::command]
pub async fn get_antigravity_path() -> Result<String, String> {
    match modules::process::get_antigravity_executable_path() {
        Some(path) => Ok(path.to_string_lossy().to_string()),
        None => Err("未找到 Antigravity 安装路径".to_string())
    }
}
