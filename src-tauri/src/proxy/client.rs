use reqwest::Client;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use crate::proxy::converter;
use uuid::Uuid;

/// Antigravity API 客户端
pub struct GeminiClient {
    client: Client,
}

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }
    
    /// 发送流式请求到 Antigravity API
    /// 注意：需要将 OpenAI 格式转换为 Antigravity 专用格式
    pub async fn stream_generate(
        &self,
        openai_request: &converter::OpenAIChatRequest,
        access_token: &str,
        project_id: &str,
        session_id: &str,  // 新增 sessionId
    ) -> Result<impl futures::Stream<Item = Result<String, String>>, String> {
        // 使用 Antigravity 内部 API
        let url = "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:streamGenerateContent?alt=sse";
        
        let contents = converter::convert_openai_to_gemini_contents(&openai_request.messages);
        let model_name = openai_request.model.clone(); // Clone for closure
        
        let request_body = serde_json::json!({
            "project": project_id,
            "requestId": Uuid::new_v4().to_string(),
            "model": openai_request.model,
            "userAgent": "antigravity",
            "request": {
                "contents": contents,
                "systemInstruction": {
                    "role": "user",
                    "parts": [{"text": ""}]
                },
                "generationConfig": {
                    "temperature": openai_request.temperature.unwrap_or(1.0),
                    "topP": openai_request.top_p.unwrap_or(0.95),
                    "maxOutputTokens": openai_request.max_tokens.unwrap_or(8096),
                    "candidateCount": 1
                },
                "toolConfig": {
                    "functionCallingConfig": {
                        "mode": "VALIDATED"
                    }
                },
                "sessionId": session_id
            }
        });
        
        let response = self.client
            .post(url)
            .bearer_auth(access_token)
            .header("Host", "daily-cloudcode-pa.sandbox.googleapis.com")
            .header("User-Agent", "antigravity/1.11.3 windows/amd64")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误 {}: {}", status, body));
        }
        
        // 将响应体转换为 OpenAI 格式的 SSE 数据 (不带 data: 前缀)
        let stream = response.bytes_stream()
            .eventsource()
            .map(move |result| {
                match result {
                    Ok(event) => {
                        let data = event.data;
                        if data == "[DONE]" {
                            return Ok("[DONE]".to_string());
                        }
                        
                        // 解析 Gemini JSON
                        let json: serde_json::Value = serde_json::from_str(&data)
                            .map_err(|e| format!("解析 Gemini 流失败: {}", e))?;
                            
                        // 兼容某些 wrap 在 response 字段下的情况
                        let candidates = if let Some(c) = json.get("candidates") {
                            c
                        } else if let Some(r) = json.get("response") {
                            r.get("candidates").unwrap_or(&serde_json::Value::Null)
                        } else {
                            &serde_json::Value::Null
                        };

                        // 提取文本
                        let text = candidates.get(0)
                            .and_then(|c| c.get("content"))
                            .and_then(|c| c.get("parts"))
                            .and_then(|p| p.get(0))
                            .and_then(|p| p.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("");

                        // 提取结束原因 (Gemini finishReason)
                        let gemini_finish_reason = candidates.get(0)
                            .and_then(|c| c.get("finishReason"))
                            .and_then(|f| f.as_str());

                        let finish_reason = match gemini_finish_reason {
                            Some("STOP") => Some("stop"),
                            Some("MAX_TOKENS") => Some("length"),
                            Some("SAFETY") => Some("content_filter"),
                            Some("RECITATION") => Some("content_filter"),
                            _ => None
                        };
                        
                        // 构造 OpenAI Chunk (仅 payload)
                        // 注意：如果 text 为空且 finish_reason 为空，这可能是一个 keep-alive 或元数据包
                        // OpenAI 允许 delta.content 为空字符串
                        
                        let chunk = serde_json::json!({
                            "id": "chatcmpl-stream",
                            "object": "chat.completion.chunk",
                            "created": chrono::Utc::now().timestamp(),
                            "model": model_name,
                            "choices": [{
                                "index": 0,
                                "delta": {
                                    "content": text
                                },
                                "finish_reason": finish_reason
                            }]
                        });
                        
                        // 注意：这里不要加 data: 前缀，因为 server.rs 中的 Sse 包装器会自动加
                        Ok(chunk.to_string())
                    }
                    Err(e) => Err(format!("流错误: {}", e)),
                }
            });
        
        Ok(stream)
    }
    
    /// 发送非流式请求到 Antigravity API
    pub async fn generate(
        &self,
        openai_request: &converter::OpenAIChatRequest,
        access_token: &str,
        project_id: &str,
        session_id: &str,  // 新增 sessionId
    ) -> Result<serde_json::Value, String> {
        // 使用 Antigravity 内部 API（非流式）
        let url = "https://daily-cloudcode-pa.sandbox.googleapis.com/v1internal:generateContent";
        
        // 转换为 Gemini contents 格式
        let contents = converter::convert_openai_to_gemini_contents(&openai_request.messages);
        
        // 构造 Antigravity 专用请求体
        let request_body = serde_json::json!({
            "project": project_id,
            "requestId": Uuid::new_v4().to_string(),
            "model": openai_request.model,
            "userAgent": "antigravity",
            "request": {
                "contents": contents,
                "systemInstruction": {
                    "role": "user",
                    "parts": [{"text": ""}]
                },
                "generationConfig": {
                    "temperature": openai_request.temperature.unwrap_or(1.0),
                    "topP": openai_request.top_p.unwrap_or(0.95),
                    "maxOutputTokens": openai_request.max_tokens.unwrap_or(8096),
                    "candidateCount": 1
                },
                "toolConfig": {
                    "functionCallingConfig": {
                        "mode": "VALIDATED"
                    }
                },
                "sessionId": session_id
            }
        });
        
        let response = self.client
            .post(url)
            .bearer_auth(access_token)
            .header("Host", "daily-cloudcode-pa.sandbox.googleapis.com")
            .header("User-Agent", "antigravity/1.11.3 windows/amd64")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API 返回错误 {}: {}", status, body));
        }
        
        let text = response.text().await
            .map_err(|e| format!("读取响应文本失败: {}", e))?;
            
        serde_json::from_str(&text)
            .map_err(|e| {
                tracing::error!("解析响应失败. 错误: {}. 原始响应: {}", e, text);
                format!("解析响应失败: {}. 原始响应: {}", e, text)
            })
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new()
    }
}
