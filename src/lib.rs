//! claWasm - WebAssembly AI Assistant
//! 
//! A browser-native AI assistant inspired by ZeroClaw and OpenClaw.
//! Runs entirely in the browser with no server dependencies.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use js_sys::Promise;
use wasm_bindgen_futures::future_to_promise;

mod config;
mod chat;
mod providers;
mod tools;
mod memory;
mod security;

use config::Config;
use chat::{Chat, Message, Role};
use providers::Provider;
use tools::{get_tool_definitions, execute_tool};
use memory::{MemorySystem, MemoryConfig, MemoryBackend, EmbeddingProvider};
use security::{SecurityManager, SecurityConfig};

/// Tool call structure
#[derive(Debug, Clone, Deserialize)]
struct ToolCall {
    name: String,
    arguments: serde_json::Value,
}

/// Initialize the claWasm WASM module
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// claWasm - Main entry point for the AI assistant
#[wasm_bindgen]
pub struct ClaWasm {
    chat: Chat,
    config: Config,
    provider: Provider,
    memory: MemorySystem,
    security: SecurityManager,
}

#[wasm_bindgen]
impl ClaWasm {
    /// Create a new claWasm instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> ClaWasm {
        init();
        let config = Config::default();
        let chat = Chat::with_system_prompt(&Self::build_system_prompt());
        let provider = Provider::from_name(&config.provider.active, config.provider.base_url.as_deref());
        let memory = MemorySystem::new(MemoryConfig::default());
        let security = SecurityManager::new(SecurityConfig::default());
        ClaWasm { chat, config, provider, memory, security }
    }

    /// Build system prompt with tools info
    fn build_system_prompt() -> String {
        let tools = get_tool_definitions();
        let tool_list: Vec<String> = tools.iter()
            .map(|t| format!("- {}: {}", t.name, t.description))
            .collect();
        
        format!(
            "You are claWasm, a helpful AI assistant running entirely in the browser. \
            You are fast, private, and ready to help with any task.\n\n\
            You have access to the following tools:\n{}\n\n\
            To use a tool, respond with a JSON object in this format:\n\
            ```tool\n{{\"name\": \"tool_name\", \"arguments\": {{...}}}}\n```\n\n\
            After using a tool, you will receive its result and can continue helping the user.",
            tool_list.join("\n")
        )
    }

    /// Create with custom configuration
    #[wasm_bindgen(js_name = "withConfig")]
    pub fn with_config(config_json: &str) -> Result<ClaWasm, JsValue> {
        init();
        let config: Config = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config error: {}", e)))?;
        let chat = Chat::with_system_prompt(&Self::build_system_prompt());
        let provider = Provider::from_name(&config.provider.active, config.provider.base_url.as_deref());
        let memory = MemorySystem::new(MemoryConfig::default());
        let security = SecurityManager::new(SecurityConfig::default());
        Ok(ClaWasm { chat, config, provider, memory, security })
    }

    /// Send a message and get a response (returns Promise)
    #[wasm_bindgen]
    pub fn chat(&mut self, message: &str) -> Promise {
        self.chat_verbose(message, false)
    }

    /// Send a message and get a response with optional verbose mode
    #[wasm_bindgen(js_name = "chatVerbose")]
    pub fn chat_verbose(&mut self, message: &str, verbose: bool) -> Promise {
        // Add user message to chat
        self.chat.add_user(message);
        let messages = self.chat.messages.clone();
        let config = self.config.clone();
        let provider = self.provider.clone();
        
        let future = async move {
            let mut current_messages = messages;
            let mut response = provider.chat(&current_messages, &config).await?;
            let mut tool_calls: Vec<ToolCall> = Vec::new();
            
            // Loop: if AI calls a tool, execute it and send result back
            let mut iterations = 0;
            while iterations < 5 {  // Max 5 tool calls per message
                iterations += 1;
                
                if let Some(tool_call) = Self::parse_tool_call(&response) {
                    // Store tool call for verbose mode
                    tool_calls.push(tool_call.clone());
                    
                    // Execute tool
                    let tool_result = match execute_tool(&tool_call.name, &tool_call.arguments).await {
                        Ok(result) => result,
                        Err(e) => format!("Error: {:?}", e),
                    };
                    
                    // Add assistant's tool call to messages
                    current_messages.push(Message::assistant(&response));
                    
                    // Add tool result as user message
                    current_messages.push(Message::user(&format!(
                        "Tool '{}' returned:\n{}",
                        tool_call.name, tool_result
                    )));
                    
                    // Get AI's response to tool result
                    response = provider.chat(&current_messages, &config).await?;
                } else {
                    // No tool call, we have a final response
                    break;
                }
            }
            
            // Return result based on verbose mode
            if verbose && !tool_calls.is_empty() {
                let result = serde_json::json!({
                    "response": response,
                    "toolCalls": tool_calls.iter().map(|t| serde_json::json!({
                        "name": t.name,
                        "arguments": t.arguments
                    })).collect::<Vec<_>>()
                });
                Ok(JsValue::from_str(&serde_json::to_string(&result).unwrap()))
            } else {
                Ok(JsValue::from_str(&response))
            }
        };
        
        future_to_promise(future)
    }

    /// Parse tool call from response
    fn parse_tool_call(response: &str) -> Option<ToolCall> {
        // Look for ```tool ... ``` block
        if let Some(start) = response.find("```tool") {
            let rest = &response[start + 7..];
            if let Some(end_relative) = rest.find("```") {
                let tool_json = rest[..end_relative].trim();
                if let Ok(call) = serde_json::from_str::<ToolCall>(tool_json) {
                    return Some(call);
                }
            }
        }
        
        // Look for JSON with "name" and "arguments" or just "name" field
        // Try to find complete JSON objects
        let mut depth = 0;
        let mut start_idx = None;
        
        for (i, c) in response.char_indices() {
            if c == '{' {
                if depth == 0 {
                    start_idx = Some(i);
                }
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start_idx {
                        let json_str = &response[start..i+1];
                        // Try to parse as ToolCall with arguments
                        if let Ok(call) = serde_json::from_str::<ToolCall>(json_str) {
                            return Some(call);
                        }
                        // Try to parse as simple {"name": "...", "query": "..."} format
                        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                // Build arguments from remaining fields
                                let mut args = serde_json::Map::new();
                                for (key, value) in obj.as_object().unwrap_or(&serde_json::Map::new()) {
                                    if key != "name" {
                                        args.insert(key.clone(), value.clone());
                                    }
                                }
                                return Some(ToolCall {
                                    name: name.to_string(),
                                    arguments: serde_json::Value::Object(args),
                                });
                            }
                        }
                    }
                    start_idx = None;
                }
            }
        }
        
        None
    }

    /// Get available tools
    #[wasm_bindgen(js_name = "getTools")]
    pub fn get_tools() -> Vec<JsValue> {
        get_tool_definitions()
            .iter()
            .map(|t| {
                JsValue::from_str(&serde_json::to_string(t).unwrap_or_default())
            })
            .collect()
    }

    /// Execute a tool directly
    #[wasm_bindgen(js_name = "executeTool")]
    pub fn execute_tool_direct(name: &str, args_json: &str) -> Promise {
        let name = name.to_string();
        let args: serde_json::Value = serde_json::from_str(args_json)
            .unwrap_or(serde_json::json!({}));
        
        let future = async move {
            let result = execute_tool(&name, &args).await?;
            Ok(JsValue::from_str(&result))
        };
        
        future_to_promise(future)
    }

    /// Get chat history as JSON
    #[wasm_bindgen(js_name = "getHistory")]
    pub fn get_history(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.chat.messages)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Clear chat history
    #[wasm_bindgen(js_name = "clearHistory")]
    pub fn clear_history(&mut self) {
        self.chat.clear(&Self::build_system_prompt());
    }

    /// Set the AI provider
    #[wasm_bindgen(js_name = "setProvider")]
    pub fn set_provider(&mut self, name: &str, api_key: Option<String>) -> Result<(), JsValue> {
        self.config.provider.active = name.to_string();
        self.config.provider.api_key = api_key;
        self.provider = Provider::from_name(name, self.config.provider.base_url.as_deref());
        Ok(())
    }

    /// Get available providers
    #[wasm_bindgen(js_name = "getProviders")]
    pub fn get_providers() -> Vec<JsValue> {
        providers::AVAILABLE_PROVIDERS
            .iter()
            .map(|s| JsValue::from_str(s))
            .collect()
    }

    /// Get configuration as JSON
    #[wasm_bindgen(js_name = "getConfig")]
    pub fn get_config(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Update configuration
    #[wasm_bindgen(js_name = "updateConfig")]
    pub fn update_config(&mut self, config_json: &str) -> Result<(), JsValue> {
        let new_config: Config = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&format!("Config error: {}", e)))?;
        self.config = new_config;
        self.provider = Provider::from_name(&self.config.provider.active, self.config.provider.base_url.as_deref());
        Ok(())
    }

    /// Set API key
    #[wasm_bindgen(js_name = "setApiKey")]
    pub fn set_api_key(&mut self, api_key: String) {
        self.config.provider.api_key = Some(api_key);
    }

    /// Set model
    #[wasm_bindgen(js_name = "setModel")]
    pub fn set_model(&mut self, model: String) {
        self.config.provider.model = model;
    }
}

impl Default for ClaWasm {
    fn default() -> Self {
        Self::new()
    }
}
