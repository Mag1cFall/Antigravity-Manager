use serde::{Deserialize, Serialize};
// use serde_json::Value;

// ===== OpenAI 格式定义 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Array(Vec<ContentPart>),
}

impl MessageContent {
    /// 获取文本内容的预览
    pub fn preview(&self) -> String {
        match self {
            MessageContent::Text(s) => if s.len() > 200 { format!("{}...", &s[..200]) } else { s.clone() },
            MessageContent::Array(parts) => {
                let mut s = String::new();
                for part in parts {
                    if let ContentPart::Text { text } = part {
                        s.push_str(text);
                    }
                }
                if s.len() > 200 { format!("{}...", &s[..200]) } else { s }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

// ===== Gemini 格式定义 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiInlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "inlineData")]
    pub inline_data: Option<GeminiInlineData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}



// ===== 格式转换 =====

/// 将 OpenAI messages 转换为 Gemini contents 数组（用于 Antigravity API）
pub fn convert_openai_to_gemini_contents(messages: &Vec<OpenAIMessage>) -> Vec<GeminiContent> {
    let mut contents = Vec::new();
    // 预编译正则，提取 markdown 图片：![alt](data:image/png;base64,.....)
    // 捕获组1: mime type, 捕获组2: base64 data (允许空格/换行)
    let re = regex::Regex::new(r"!\[.*?\]\(data:\s*(image/[a-zA-Z+-]+)\s*;\s*base64\s*,\s*([a-zA-Z0-9+/=\s]+)\)").unwrap();
    
    // 正则用于从 data URL 中提取 base64
    let re_data_url = regex::Regex::new(r"data:\s*(image/[a-zA-Z+-]+)\s*;\s*base64\s*,\s*([a-zA-Z0-9+/=\s]+)").unwrap();

    let mut pending_images: Vec<GeminiInlineData> = Vec::new();

    for (i, msg) in messages.iter().enumerate() {
        // Debug: 查看消息内容预览
        let preview = msg.content.preview();
        tracing::info!("Msg[{}][{}] content={:?}", i, msg.role, preview);

        // 角色映射
        let role = match msg.role.as_str() {
            "assistant" => "model",
            "system" => "user",
            _ => &msg.role,
        };
        
        let mut parts = Vec::new();
        
        // 1. 处理 Pending Images (Assist 历史图片注入到 User)
        if role == "user" && !pending_images.is_empty() {
             let count = pending_images.len();
             tracing::info!("向 User 消息注入 {} 张待处理图片 (上下文携带)", count);
             for img in pending_images.drain(..) {
                parts.push(GeminiPart {
                    text: None,
                    inline_data: Some(img),
                });
            }
        }

        // 2. 解析当前消息内容 (支持 String 和 Array)
        match &msg.content {
            MessageContent::Text(text) => {
                // 处理 String 格式 (解析 Markdown 图片)
                let mut last_end = 0;
                for caps in re.captures_iter(text) {
                    let match_start = caps.get(0).unwrap().start();
                    let match_end = caps.get(0).unwrap().end();
                    
                    if match_start > last_end {
                        let text_part = &text[last_end..match_start];
                        if !text_part.is_empty() {
                            parts.push(GeminiPart { text: Some(text_part.to_string()), inline_data: None });
                        }
                    }
                    
                    let mime_type = caps.get(1).unwrap().as_str().to_string();
                    let data = caps.get(2).unwrap().as_str().replace(|c: char| c.is_whitespace(), "");
                    let inline_data = GeminiInlineData { mime_type, data };

                    if role == "model" {
                        pending_images.push(inline_data);
                    } else {
                        parts.push(GeminiPart { text: None, inline_data: Some(inline_data) });
                    }
                    last_end = match_end;
                }
                if last_end < text.len() {
                    let text_part = &text[last_end..];
                    if !text_part.is_empty() {
                        parts.push(GeminiPart { text: Some(text_part.to_string()), inline_data: None });
                    }
                }
            },
            MessageContent::Array(content_parts) => {
                // 处理 Array 格式 (多模态)
                for part in content_parts {
                    match part {
                        ContentPart::Text { text } => {
                            parts.push(GeminiPart { text: Some(text.clone()), inline_data: None });
                        },
                        ContentPart::ImageUrl { image_url } => {
                            let url = &image_url.url;
                            if let Some(caps) = re_data_url.captures(url) {
                                let mime_type = caps.get(1).unwrap().as_str().to_string();
                                let data = caps.get(2).unwrap().as_str().replace(|c: char| c.is_whitespace(), "");
                                let inline_data = GeminiInlineData { mime_type: mime_type.clone(), data };
                                
                                if role == "model" {
                                    // 理论上 Model 消息不应该发这里，但防以后
                                    pending_images.push(inline_data);
                                } else {
                                    tracing::info!("解析到 Multimodal 图片数据 (Mime: {})", mime_type);
                                    parts.push(GeminiPart { text: None, inline_data: Some(inline_data) });
                                }
                            } else {
                                tracing::warn!("忽略不支持的图片 URL 格式: {}", url);
                            }
                        }
                    }
                }
            }
        }
        
        // 3. 补全与清理
        if role == "model" && parts.is_empty() && !pending_images.is_empty() {
            parts.push(GeminiPart {
                text: Some("[Image Generated]".to_string()), 
                inline_data: None,
            });
        }

        if parts.is_empty() {
            parts.push(GeminiPart {
                text: Some("".to_string()),
                inline_data: None,
            });
        }
        
        contents.push(GeminiContent {
            role: role.to_string(),
            parts,
        });
    }
    
    // 合并连续 User 消息
    let mut i = 1;
    while i < contents.len() {
        if contents[i].role == "user" && contents[i-1].role == "user" {
            let mut parts_to_append = contents[i].parts.clone();
            
            let need_separator = if let Some(last_part) = contents[i-1].parts.last() {
                if let Some(first_part) = parts_to_append.first() {
                    last_part.text.is_some() && first_part.text.is_some()
                } else {
                    false
                }
            } else {
                false
            };
            
            if need_separator {
                contents[i-1].parts.push(GeminiPart {
                    text: Some("\n\n".to_string()),
                    inline_data: None,
                });
            }
            
            contents[i-1].parts.append(&mut parts_to_append);
            contents.remove(i);
        } else {
            i += 1;
        }
    }
    
    contents
}


