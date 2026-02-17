//! Configuration module for WebClaw

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// AI Provider settings
    pub provider: ProviderConfig,
    /// System prompt
    pub system_prompt: String,
    /// Maximum tokens in response
    pub max_tokens: u32,
    /// Temperature for response generation
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Active provider name (openai, anthropic, ollama, etc.)
    pub active: String,
    /// API key (stored in memory, not persisted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL for API (for custom endpoints)
    pub base_url: Option<String>,
    /// Model to use
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            provider: ProviderConfig {
                active: "openai".to_string(),
                api_key: None,
                base_url: None,
                model: "gpt-4o-mini".to_string(),
            },
            system_prompt: "You are WebClaw, a helpful AI assistant running entirely in the browser. \
                You are fast, private, and ready to help with any task."
                .to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}
