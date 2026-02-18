//! AI Provider implementations for claWasm
//! 
//! Supports OpenAI, Anthropic, Ollama, and custom OpenAI-compatible endpoints

use crate::chat::{Message, Role};
use crate::config::Config;
use serde::Deserialize;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast;

use crate::tools::get_tools_openai_format;

pub const AVAILABLE_PROVIDERS: &[&str] = &[
    "openai",
    "anthropic", 
    "ollama",
    "ollama_cloud",
    "groq",
    "together",
    "custom",
];

/// Provider enum (simpler than trait for WASM)
#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI { base_url: String },
    Anthropic,
    Ollama { base_url: String, api_key: Option<String> },
}

impl Provider {
    /// Create a provider from name
    pub fn from_name(name: &str, base_url: Option<&str>) -> Self {
        match name {
            "openai" => Provider::OpenAI { 
                base_url: base_url.unwrap_or("https://api.openai.com/v1").to_string() 
            },
            "anthropic" => Provider::Anthropic,
            "ollama" => Provider::Ollama { 
                base_url: base_url.unwrap_or("http://localhost:11434").to_string(),
                api_key: None,
            },
            "ollama_cloud" => Provider::Ollama { 
                base_url: "https://ollama.com".to_string(),
                api_key: None, // Will be set via config
            },
            "groq" => Provider::OpenAI { 
                base_url: base_url.unwrap_or("https://api.groq.com/openai/v1").to_string() 
            },
            "together" => Provider::OpenAI { 
                base_url: base_url.unwrap_or("https://api.together.xyz/v1").to_string() 
            },
            _ => Provider::OpenAI { 
                base_url: base_url.unwrap_or("https://api.openai.com/v1").to_string() 
            },
        }
    }

    /// Send a chat completion request
    pub async fn chat(&self, messages: &[Message], config: &Config) -> Result<String, JsValue> {
        match self {
            Provider::OpenAI { base_url } => self.chat_openai(messages, config, base_url).await,
            Provider::Anthropic => self.chat_anthropic(messages, config).await,
            Provider::Ollama { base_url, .. } => self.chat_ollama(messages, config, base_url).await,
        }
    }

    async fn chat_openai(&self, messages: &[Message], config: &Config, base_url: &str) -> Result<String, JsValue> {
        let api_key = config.provider.api_key.as_ref()
            .ok_or_else(|| JsValue::from_str("API key not set"))?;
        
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        headers.set("Authorization", &format!("Bearer {}", api_key))?;
        
        let body = serde_json::json!({
            "model": config.provider.model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "max_tokens": config.max_tokens,
            "temperature": config.temperature,
            "tools": get_tools_openai_format(),
        });
        
        let mut request_init = RequestInit::new();
        request_init.method("POST");
        request_init.headers(headers.as_ref());
        request_init.body(Some(&JsValue::from_str(&serde_json::to_string(&body).unwrap())));
        request_init.mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init(
            &format!("{}/chat/completions", base_url),
            &request_init,
        )?;
        
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        if !response.ok() {
            let error_text = JsFuture::from(response.text()?).await?;
            return Err(JsValue::from_str(&format!("API error: {}", error_text.as_string().unwrap_or_default())));
        }
        
        let json = JsFuture::from(response.json()?).await?;
        let result: OpenAIResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        Ok(result.choices[0].message.content.clone())
    }

    async fn chat_anthropic(&self, messages: &[Message], config: &Config) -> Result<String, JsValue> {
        let api_key = config.provider.api_key.as_ref()
            .ok_or_else(|| JsValue::from_str("API key not set"))?;
        
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        headers.set("x-api-key", api_key)?;
        headers.set("anthropic-version", "2023-06-01")?;
        
        // Extract system prompt and other messages
        let system_prompt: String = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");
        
        let anthropic_messages: Vec<serde_json::Value> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| serde_json::json!({
                "role": match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "user",
                },
                "content": m.content,
            }))
            .collect();
        
        let body = serde_json::json!({
            "model": config.provider.model,
            "max_tokens": config.max_tokens,
            "system": system_prompt,
            "messages": anthropic_messages,
        });
        
        let mut request_init = RequestInit::new();
        request_init.method("POST");
        request_init.headers(headers.as_ref());
        request_init.body(Some(&JsValue::from_str(&serde_json::to_string(&body).unwrap())));
        request_init.mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init(
            "https://api.anthropic.com/v1/messages",
            &request_init,
        )?;
        
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        if !response.ok() {
            let error_text = JsFuture::from(response.text()?).await?;
            return Err(JsValue::from_str(&format!("API error: {}", error_text.as_string().unwrap_or_default())));
        }
        
        let json = JsFuture::from(response.json()?).await?;
        let result: AnthropicResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        let content = result.content
            .into_iter()
            .filter_map(|c| if c.content_type == "text" { Some(c.text) } else { None })
            .collect::<Vec<_>>()
            .join("");
        
        Ok(content)
    }

    async fn chat_ollama(&self, messages: &[Message], config: &Config, base_url: &str) -> Result<String, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        // Normalize model name (remove :cloud suffix if present)
        let model = config.provider.model.replace(":cloud", "");
        
        // Check if this is Ollama Cloud - route through proxy for CORS
        let is_ollama_cloud = base_url.contains("ollama.com");
        
        let endpoint = if is_ollama_cloud {
            // Use proxy for Ollama Cloud
            "http://localhost:3000/proxy".to_string()
        } else {
            // Direct connection for local Ollama
            format!("{}/v1/chat/completions", base_url)
        };
        
        let actual_url = if is_ollama_cloud {
            format!("{}/v1/chat/completions", base_url)
        } else {
            endpoint.clone()
        };
        
        let body = serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "stream": false,
            "tools": get_tools_openai_format(),
        });
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        
        // Add API key if available
        if let Some(ref api_key) = config.provider.api_key {
            headers.set("Authorization", &format!("Bearer {}", api_key))?;
        }
        
        let mut request_init = RequestInit::new();
        request_init.set_method("POST");
        request_init.set_headers(headers.as_ref());
        
        // For Ollama Cloud via proxy, wrap the request
        let request_body = if is_ollama_cloud {
            let mut proxy_headers = HashMap::new();
            proxy_headers.insert("Content-Type".to_string(), "application/json".to_string());
            if let Some(ref api_key) = config.provider.api_key {
                proxy_headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
            }
            
            serde_json::json!({
                "url": actual_url,
                "method": "POST",
                "headers": proxy_headers,
                "body": serde_json::to_string(&body).unwrap()
            })
        } else {
            body.clone()
        };
        
        request_init.set_body(&JsValue::from_str(&serde_json::to_string(&request_body).unwrap()));
        request_init.set_mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init(&endpoint, &request_init)?;
        
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        if !response.ok() {
            let status = response.status();
            let error_text = JsFuture::from(response.text()?).await?;
            let error_str = error_text.as_string().unwrap_or_default();
            
            // If OpenAI-compatible fails for local Ollama, try native API
            if !is_ollama_cloud && (error_str.contains("404") || error_str.contains("Not Found")) {
                return self.chat_ollama_native(messages, config, base_url).await;
            }
            
            // Clear error for unauthorized
            if status == 401 || error_str.contains("unauthorized") || error_str.contains("Unauthorized") {
                return Err(JsValue::from_str(
                    "Ollama Cloud API key required. Go to Settings and enter your Ollama Cloud API key."
                ));
            }
            
            return Err(JsValue::from_str(&format!(
                "Ollama error ({}): {}. Make sure {} is running",
                status,
                error_str,
                if is_ollama_cloud { "Ollama Cloud API key is set in Settings" } else { "Ollama (ollama serve)" }
            )));
        }
        
        // Parse OpenAI-compatible response
        let json = JsFuture::from(response.json()?).await?;
        let result: OpenAIResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        let message = &result.choices[0].message;
        
        // If tool_calls exist, return them as JSON for parsing
        if let Some(ref tool_calls) = message.tool_calls {
            if !tool_calls.is_empty() {
                let tc = &tool_calls[0];
                // Return as JSON string that parse_tool_call can find
                let args: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::json!({}));
                return Ok(serde_json::to_string(&serde_json::json!({
                    "name": tc.function.name,
                    "arguments": args
                })).unwrap_or_else(|_| message.content.clone()));
            }
        }
        
        Ok(message.content.clone())
    }
    
    /// Fallback to native Ollama API if OpenAI-compatible fails
    async fn chat_ollama_native(&self, messages: &[Message], config: &Config, base_url: &str) -> Result<String, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        let model = config.provider.model.replace(":cloud", "");
        
        let body = serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "stream": false,
        });
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        
        let mut request_init = RequestInit::new();
        request_init.method("POST");
        request_init.headers(headers.as_ref());
        request_init.body(Some(&JsValue::from_str(&serde_json::to_string(&body).unwrap())));
        request_init.mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init(
            &format!("{}/api/chat", base_url),
            &request_init,
        )?;
        
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        if !response.ok() {
            let error_text = JsFuture::from(response.text()?).await?;
            return Err(JsValue::from_str(&format!(
                "Ollama native error: {}. Make sure Ollama is running (ollama serve)",
                error_text.as_string().unwrap_or_default()
            )));
        }
        
        let json = JsFuture::from(response.json()?).await?;
        let result: OllamaResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        Ok(result.message.content)
    }
}

// Response types
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ToolCall {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    index: Option<i32>,
    #[serde(default)]
    r#type: Option<String>,
    function: ToolCallFunction,
}

#[derive(Debug, Deserialize, Clone)]
struct ToolCallFunction {
    name: String,
    arguments: String,  // JSON string
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}
