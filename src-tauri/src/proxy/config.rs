use serde::{Deserialize, Serialize};
// use std::path::PathBuf;

/// 反代服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 是否启用反代服务
    pub enabled: bool,
    
    /// 监听端口
    pub port: u16,
    
    /// API 密钥
    pub api_key: String,
    

    /// 是否自动启动
    pub auto_start: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 8045,
            api_key: format!("sk-{}", uuid::Uuid::new_v4().simple()),
            auto_start: false,
        }
    }
}
