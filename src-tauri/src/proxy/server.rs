use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::{IntoResponse, Response, sse::{Event, Sse}},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tokio::sync::oneshot;
use futures::stream::StreamExt;
use crate::proxy::{TokenManager, converter, client::GeminiClient};

/// Axum 应用状态
#[derive(Clone)]
pub struct AppState {
    pub token_manager: Arc<TokenManager>,

}

/// Axum 服务器实例
pub struct AxumServer {
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl AxumServer {
    /// 启动 Axum 服务器
    pub async fn start(
        port: u16,
        token_manager: Arc<TokenManager>,
    ) -> Result<(Self, tokio::task::JoinHandle<()>), String> {
        let state = AppState {
            token_manager,
        };
        
        // 构建路由
        let app = Router::new()
            .route("/v1/chat/completions", post(chat_completions_handler))
            .route("/v1/models", get(list_models_handler))
            .route("/healthz", get(health_check_handler))
            .with_state(state);
        
        // 绑定地址
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("端口 {} 绑定失败: {}", port, e))?;
        
        tracing::info!("反代服务器启动在 http://{}", addr);
        
        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        
        // 在新任务中启动服务器
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();
        });
        
        Ok((
            Self {
                shutdown_tx: Some(shutdown_tx),
            },
            handle,
        ))
    }
    
    /// 停止服务器
    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

// ===== API 处理器 =====

/// 请求处理结果
enum RequestResult {
    Success(Response),
    Retry(String), // 包含重试原因
    Error(Response),
}

/// 聊天补全处理器
async fn chat_completions_handler(
    State(state): State<AppState>,
    Json(request): Json<converter::OpenAIChatRequest>,
) -> Response {
    let max_retries = state.token_manager.len().max(1);
    let mut attempts = 0;
    
    // 克隆请求以支持重试
    let request = Arc::new(request);

    loop {
        attempts += 1;
        
        // 1. 获取 Token
        let token = match state.token_manager.get_token().await {
            Some(t) => t,
            None => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "error": {
                            "message": "没有可用账号",
                            "type": "no_accounts"
                        }
                    }))
                ).into_response();
            }
        };
        
        tracing::info!("尝试使用账号: {} (第 {}/{} 次尝试)", token.email, attempts, max_retries);

        // 2. 处理请求
        let result = process_request(state.clone(), request.clone(), token.clone()).await;
        
        match result {
            RequestResult::Success(response) => return response,
            RequestResult::Retry(reason) => {
                tracing::warn!("账号 {} 请求失败，准备重试: {}", token.email, reason);
                if attempts >= max_retries {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(serde_json::json!({
                            "error": {
                                "message": format!("所有账号配额已耗尽或请求失败。最后错误: {}", reason),
                                "type": "all_accounts_exhausted"
                            }
                        }))
                    ).into_response();
                }
                // 继续下一次循环，token_manager.get_token() 会自动轮换
                continue;
            },
            RequestResult::Error(response) => return response,
        }
    }
}

/// 统一请求分发入口
async fn process_request(
    state: AppState,
    request: Arc<converter::OpenAIChatRequest>,
    token: crate::proxy::token_manager::ProxyToken,
) -> RequestResult {
    let is_stream = request.stream.unwrap_or(false);
    let is_image_model = request.model == "gemini-3-pro-image";
    
    if is_stream {
        if is_image_model {
            handle_image_stream_request(state, request, token).await
        } else {
            handle_stream_request(state, request, token).await
        }
    } else {
        handle_non_stream_request(state, request, token).await
    }
}

/// 处理画图模型的流式请求（模拟流式）
async fn handle_image_stream_request(
    _state: AppState,
    request: Arc<converter::OpenAIChatRequest>,
    token: crate::proxy::token_manager::ProxyToken,
) -> RequestResult {
    let client = GeminiClient::new();
    let model = request.model.clone();
    
    let project_id = match get_project_id(&token) {
        Ok(id) => id,
        Err(e) => return RequestResult::Error(e),
    };
    
    let response_result = client.generate(
        &request,
        &token.access_token,
        project_id,
        &token.session_id,
    ).await;
    
    match response_result {
        Ok(response) => {
            // 2. 处理图片转 Markdown
            let processed_json = process_inline_data(response);
            
            // 3. 提取 Markdown 文本
            // 移除详细调试日志以免刷屏
            // tracing::info!("Processed Image Response: {}", serde_json::to_string_pretty(&processed_json).unwrap_or_default());
            tracing::info!("Image generation successful, processing response...");

            let content = processed_json["response"]["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .or_else(|| {
                    // 尝试备用路径：有时候 structure 可能略有不同
                    tracing::warn!("Standard path for image content failed. Checking response structure...");
                    processed_json["candidates"][0]["content"]["parts"][0]["text"].as_str()
                })
                .unwrap_or("生成图片失败或格式错误")
                .to_string();
                
            // 4. 构造 SSE 流
            let stream = async_stream::stream! {
                let chunk = serde_json::json!({
                    "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model,
                    "choices": [
                        {
                            "index": 0,
                            "delta": { "content": content },
                            "finish_reason": null
                        }
                    ]
                });
                yield Ok::<_, axum::Error>(Event::default().data(chunk.to_string()));
                
                let end_chunk = serde_json::json!({
                    "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model,
                    "choices": [
                        {
                            "index": 0,
                            "delta": {},
                            "finish_reason": "stop"
                        }
                    ]
                });
                yield Ok(Event::default().data(end_chunk.to_string()));
                yield Ok(Event::default().data("[DONE]"));
            };
            
            RequestResult::Success(Sse::new(stream).into_response())
        },
        Err(e) => check_retry_error(&e),
    }
}

/// 处理流式请求
async fn handle_stream_request(
    _state: AppState,
    request: Arc<converter::OpenAIChatRequest>,
    token: crate::proxy::token_manager::ProxyToken,
) -> RequestResult {
    let client = GeminiClient::new();
    
    let project_id = match get_project_id(&token) {
        Ok(id) => id,
        Err(e) => return RequestResult::Error(e),
    };
    
    let stream_result = client.stream_generate(
        &request,
        &token.access_token,
        project_id,
        &token.session_id,
    ).await;
    
    match stream_result {
        Ok(stream) => {
            let sse_stream = stream.map(move |chunk| {
                match chunk {
                    Ok(data) => Ok(Event::default().data(data)),
                    Err(e) => {
                        tracing::error!("Stream error: {}", e);
                        Err(axum::Error::new(e))
                    }
                }
            });
            RequestResult::Success(Sse::new(sse_stream).into_response())
        },
        Err(e) => check_retry_error(&e),
    }
}

/// 处理非流式请求
async fn handle_non_stream_request(
    _state: AppState,
    request: Arc<converter::OpenAIChatRequest>,
    token: crate::proxy::token_manager::ProxyToken,
) -> RequestResult {
    let client = GeminiClient::new();
    
    let project_id = match get_project_id(&token) {
        Ok(id) => id,
        Err(e) => return RequestResult::Error(e),
    };
    
    let response_result = client.generate(
        &request,
        &token.access_token,
        project_id,
        &token.session_id,
    ).await;
    
    match response_result {
        Ok(response) => {
            let processed_response = process_inline_data(response);
            RequestResult::Success(Json(processed_response).into_response())
        },
        Err(e) => check_retry_error(&e),
    }
}

/// 辅助函数：获取 Project ID
fn get_project_id(token: &crate::proxy::token_manager::ProxyToken) -> Result<&str, Response> {
    token.project_id.as_ref()
        .map(|s| s.as_str())
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "message": "没有 project_id",
                        "type": "config_error"
                    }
                }))
            ).into_response()
        })
}

/// 辅助函数：检查错误是否需要重试
fn check_retry_error(error_msg: &str) -> RequestResult {
    // 检查 429 或者 配额耗尽 关键字
    if error_msg.contains("429") || 
       error_msg.contains("RESOURCE_EXHAUSTED") || 
       error_msg.contains("QUOTA_EXHAUSTED") ||
       error_msg.contains("The request has been rate limited") ||
       // 新增：网络错误或响应解析失败也进行重试
       error_msg.contains("读取响应文本失败") ||
       error_msg.contains("error decoding response body") ||
       error_msg.contains("closed connection") {
        return RequestResult::Retry(error_msg.to_string());
    }
    
    // 其他错误直接返回
    RequestResult::Error((
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": {
                "message": format!("Antigravity API 错误: {}", error_msg),
                "type": "api_error"
            }
        }))
    ).into_response())
}

/// 模型列表处理器
async fn list_models_handler(
    State(_state): State<AppState>,
) -> Response {
    // 返回 Antigravity 实际可用的模型列表
    let models = serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "gemini-2.5-flash",
                "object": "model",
                "created": 1734336000,
                "owned_by": "google"
            },
            {
                "id": "gemini-3-pro-low",
                "object": "model",
                "created": 1734336000,
                "owned_by": "google"
            },
            {
                "id": "gemini-3-pro-high",
                "object": "model",
                "created": 1734336000,
                "owned_by": "google"
            },
            {
                "id": "gemini-3-pro-image",
                "object": "model",
                "created": 1734336000,
                "owned_by": "google"
            },
            {
                "id": "claude-sonnet-4-5",
                "object": "model",
                "created": 1734336000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-sonnet-4-5-thinking",
                "object": "model",
                "created": 1734336000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-opus-4-5-thinking",
                "object": "model",
                "created": 1734336000,
                "owned_by": "anthropic"
            },
            {
                "id": "gemini-2.5-flash-thinking",
                "object": "model",
                "created": 1734336000,
                "owned_by": "google"
            }
        ]
    });
    
    Json(models).into_response()
}

/// 健康检查处理器
async fn health_check_handler() -> Response {
    Json(serde_json::json!({
        "status": "ok"
    })).into_response()
}

/// 处理 Antigravity 响应中的 inlineData(生成的图片)
/// 将 base64 图片转换为 Markdown 格式
/// 处理 Inline Data (base64 图片) 转 Markdown
fn process_inline_data(mut response: serde_json::Value) -> serde_json::Value {
    // 1. 定位 candidates 节点
    // Antigravity 响应可能是 { "candidates": ... } 或 { "response": { "candidates": ... } }
    let candidates_node = if response.get("candidates").is_some() {
        response.get_mut("candidates")
    } else if let Some(r) = response.get_mut("response") {
         r.get_mut("candidates")
    } else {
        None
    };

    if let Some(candidates_val) = candidates_node {
        if let Some(candidates) = candidates_val.as_array_mut() {
            for candidate in candidates {
                if let Some(content) = candidate["content"].as_object_mut() {
                    if let Some(parts) = content["parts"].as_array_mut() {
                        let mut new_parts = Vec::new();
                        
                        for part in parts.iter() {
                            // 检查是否有 inlineData
                            if let Some(inline_data) = part.get("inlineData") {
                                let mime_type = inline_data["mimeType"]
                                    .as_str()
                                    .unwrap_or("image/jpeg");
                                let data = inline_data["data"]
                                    .as_str()
                                    .unwrap_or("");
                                
                                // 构造 Markdown 图片语法
                                let image_markdown = format!(
                                    "\n\n![Generated Image](data:{};base64,{})\n\n",
                                    mime_type, data
                                );
                                
                                // 替换为文本 part
                                new_parts.push(serde_json::json!({
                                    "text": image_markdown
                                }));
                            } else {
                                // 保留原始 part
                                new_parts.push(part.clone());
                            }
                        }
                        
                        // 更新 parts
                        *parts = new_parts;
                    }
                }
            }
        }
    }
    
    // 直接返回修改后的对象，不再包裹 "response"
    response
}
