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
        
        // Categorize tools for better clarity
        let search_tools: Vec<&str> = vec!["web_search", "reddit_search", "image_search", "research", "fetch_url"];
        let doc_tools: Vec<&str> = vec!["create_pdf", "download_file", "save_note", "read_notes"];
        let security_tools: Vec<&str> = vec!["scan_xss", "scan_sqli", "scan_headers", "scan_ssl", "scan_deps", "scan_secrets", "scan_cors"];
        let custom_tools: Vec<&str> = vec!["create_tool", "list_custom_tools", "delete_tool"];
        let other_tools: Vec<&str> = vec!["get_current_time", "calculate"];
        
        let mut categorized = String::new();
        categorized.push_str("\n## 🔍 Arama ve Araştırma\n");
        for t in tools.iter() {
            if search_tools.contains(&t.name.as_str()) {
                categorized.push_str(&format!("- **{}**: {}\n", t.name, t.description));
            }
        }
        categorized.push_str("\n## 📄 Belge ve Not\n");
        for t in tools.iter() {
            if doc_tools.contains(&t.name.as_str()) {
                categorized.push_str(&format!("- **{}**: {}\n", t.name, t.description));
            }
        }
        categorized.push_str("\n## 🔒 Güvenlik ve Zafiyet Tarama\n");
        for t in tools.iter() {
            if security_tools.contains(&t.name.as_str()) {
                categorized.push_str(&format!("- **{}**: {}\n", t.name, t.description));
            }
        }
        categorized.push_str("\n## 🔧 Özel Araçlar\n");
        for t in tools.iter() {
            if custom_tools.contains(&t.name.as_str()) {
                categorized.push_str(&format!("- **{}**: {}\n", t.name, t.description));
            }
        }
        categorized.push_str("\n## ⚡ Diğer\n");
        for t in tools.iter() {
            if other_tools.contains(&t.name.as_str()) {
                categorized.push_str(&format!("- **{}**: {}\n", t.name, t.description));
            }
        }
        
        format!(
            "You are claWasm, a helpful AI assistant running entirely in the browser as WebAssembly (WASM). \
            You are fast, private, and ready to help with any task.\n\n\
            You have access to the following tools:{}\n\n\
            To use a tool, respond with a JSON object in this format:\n\
            ```tool\n{{\"name\": \"tool_name\", \"arguments\": {{...}}}}\n```\n\n\
            Or simply: {{\"name\": \"tool_name\", \"query\": \"...\", ...}}\n\n\
            After using a tool, you will receive its result and can continue helping the user.\n\n\
            CRITICAL RULES:\n\
            1. When asked about your tools/capabilities, ALWAYS list ALL tools including the security scanners (scan_xss, scan_sqli, scan_headers, scan_ssl, scan_deps, scan_secrets, scan_cors)\n\
            2. When asked about security, vulnerabilities, or code analysis, ALWAYS use the scan_* tools\n\
            3. NEVER skip or hide tools from the user - show everything available!\n\n\
            ⚠️ WASM LIMITATIONS:\n\
            Since I run entirely in the browser as WASM, I have certain limitations:\n\
            - I cannot access the file system directly (only browser storage/localStorage)\n\
            - I cannot make direct API calls to external services (I use a local proxy at localhost:3000)\n\
            - I cannot record audio directly, but I can use text_to_speech tool to generate downloadable MP3s\n\
            - I cannot execute system commands\n\
            - Custom tools via create_tool are limited to JavaScript browser APIs\n\n\
            When you ask for something I cannot do directly, I will:\n\
            1. Explain my WASM limitations clearly\n\
            2. Propose alternative solutions using available tools\n\
            3. If needed, suggest workarounds or external services that could help\n\n\
            For example: If you want downloadable audio, I use text_to_speech (Google TTS API) instead of browser speechSynthesis which only speaks but doesn't create files.",
            categorized
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
            
            // Loop: if AI calls tools, execute ALL of them and send results back
            let mut iterations = 0;
            while iterations < 10 {  // Max 10 iterations
                iterations += 1;
                
                let calls = Self::parse_all_tool_calls(&response);
                if calls.is_empty() {
                    // No tool calls, we have a final response
                    break;
                }
                
                // Execute ALL tool calls found
                let mut tool_results = Vec::new();
                for tool_call in calls {
                    tool_calls.push(tool_call.clone());
                    
                    let tool_result = match execute_tool(&tool_call.name, &tool_call.arguments).await {
                        Ok(result) => result,
                        Err(e) => format!("Error: {:?}", e),
                    };
                    
                    // Truncate long tool results to prevent context overflow
                    let truncated_result = if tool_result.len() > 2000 {
                        format!("{}...\n[Result truncated, {} chars total]", 
                            &tool_result[..2000], tool_result.len())
                    } else {
                        tool_result
                    };
                    
                    tool_results.push(format!("Tool '{}' returned:\n{}", tool_call.name, truncated_result));
                }
                
                // Add assistant's response to messages
                current_messages.push(Message::assistant(&response));
                
                // Add all tool results as one message
                current_messages.push(Message::user(&tool_results.join("\n\n---\n\n")));
                
                // Trim context if too many messages (keep last 10 exchanges = 20 messages)
                if current_messages.len() > 20 {
                    // Keep system message and last 10 exchanges
                    let system_msgs: Vec<Message> = current_messages.iter()
                        .filter(|m| matches!(m.role, Role::System))
                        .cloned()
                        .collect();
                    let recent_msgs: Vec<Message> = current_messages.iter()
                        .rev()
                        .take(20)
                        .rev()
                        .cloned()
                        .collect();
                    current_messages = [system_msgs, recent_msgs].concat();
                    web_sys::console::log_1(&JsValue::from_str("Context trimmed to prevent overflow"));
                }
                
                // Get AI's response to tool results
                response = provider.chat(&current_messages, &config).await?;
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

    /// Parse ALL tool calls from response
    fn parse_all_tool_calls(response: &str) -> Vec<ToolCall> {
        let mut calls = Vec::new();
        
        // Check for incomplete JSON (response ends with incomplete JSON)
        let open_braces = response.matches('{').count();
        let close_braces = response.matches('}').count();
        let open_brackets = response.matches('[').count();
        let close_brackets = response.matches(']').count();
        
        if open_braces > close_braces || open_brackets > close_brackets {
            // Incomplete JSON detected - try to find complete JSONs only
            // This means the response was truncated
            web_sys::console::log_1(&JsValue::from_str(&format!(
                "Warning: Incomplete JSON detected ({{:{}/}}:{}, [:{}/]:{})", 
                open_braces, close_braces, open_brackets, close_brackets
            )));
        }
        
        // Find all ```tool ... ``` blocks
        let mut search_start = 0;
        while let Some(start) = response[search_start..].find("```tool") {
            let rest = &response[search_start + start + 7..];
            if let Some(end_relative) = rest.find("```") {
                let tool_json = rest[..end_relative].trim();
                if let Ok(call) = serde_json::from_str::<ToolCall>(tool_json) {
                    calls.push(call);
                }
            }
            search_start += start + 7;
        }
        
        // Find all JSON objects with "name" field
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
                            // Avoid duplicates
                            if !calls.iter().any(|c| c.name == call.name && c.arguments == call.arguments) {
                                calls.push(call);
                            }
                        } else if let Ok(obj) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                let mut args = serde_json::Map::new();
                                for (key, value) in obj.as_object().unwrap_or(&serde_json::Map::new()) {
                                    if key != "name" {
                                        args.insert(key.clone(), value.clone());
                                    }
                                }
                                let call = ToolCall {
                                    name: name.to_string(),
                                    arguments: serde_json::Value::Object(args),
                                };
                                // Avoid duplicates
                                if !calls.iter().any(|c| c.name == call.name && c.arguments == call.arguments) {
                                    calls.push(call);
                                }
                            }
                        }
                    }
                    start_idx = None;
                }
            }
        }
        
        calls
    }

    /// Parse single tool call (for backwards compatibility)
    fn parse_tool_call(response: &str) -> Option<ToolCall> {
        Self::parse_all_tool_calls(response).first().cloned()
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
