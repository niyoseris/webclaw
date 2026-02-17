//! Chat module for WebClaw - Message handling and conversation management

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: Role,
    /// Message content
    pub content: String,
}

impl Message {
    /// Create a new system message
    pub fn system(content: &str) -> Self {
        Message {
            role: Role::System,
            content: content.to_string(),
        }
    }

    /// Create a new user message
    pub fn user(content: &str) -> Self {
        Message {
            role: Role::User,
            content: content.to_string(),
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: &str) -> Self {
        Message {
            role: Role::Assistant,
            content: content.to_string(),
        }
    }
}

/// Chat history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    /// Messages in chronological order
    pub messages: Vec<Message>,
}

impl Chat {
    /// Create a new chat with a system message
    pub fn with_system_prompt(prompt: &str) -> Self {
        Chat {
            messages: vec![Message::system(prompt)],
        }
    }

    /// Add a user message
    pub fn add_user(&mut self, content: &str) {
        self.messages.push(Message::user(content));
    }

    /// Add an assistant message
    pub fn add_assistant(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
    }

    /// Get messages for API (includes the new user message)
    pub fn to_api_messages_with_user(&self, user_message: &str) -> Vec<Message> {
        let mut messages = self.messages.clone();
        messages.push(Message::user(user_message));
        messages
    }

    /// Clear all messages except system
    pub fn clear(&mut self, system_prompt: &str) {
        self.messages = vec![Message::system(system_prompt)];
    }
}
