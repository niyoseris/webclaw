//! Security Module for WebClaw
//! 
//! Inspired by ZeroClaw's security model with pairing, sandboxing, and allowlists.

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use std::collections::{HashMap, HashSet};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable pairing mode (requires approval for actions)
    pub pairing_enabled: bool,
    /// Enable sandbox mode (restrict dangerous operations)
    pub sandbox_enabled: bool,
    /// Allowed domains for fetch_url
    pub allowed_domains: Vec<String>,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Allowed tools
    pub allowed_tools: Vec<String>,
    /// Blocked tools
    pub blocked_tools: Vec<String>,
    /// Max tool calls per message
    pub max_tool_calls: u32,
    /// Require approval for tool calls
    pub require_tool_approval: bool,
    /// Workspace scope (restrict file access)
    pub workspace_scope: Option<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            pairing_enabled: true,
            sandbox_enabled: true,
            allowed_domains: vec![
                "wikipedia.org".to_string(),
                "github.com".to_string(),
                "stackoverflow.com".to_string(),
                "docs.rs".to_string(),
            ],
            blocked_domains: vec![],
            allowed_tools: vec![
                "web_search".to_string(),
                "get_current_time".to_string(),
                "calculate".to_string(),
                "save_note".to_string(),
                "read_notes".to_string(),
            ],
            blocked_tools: vec![],
            max_tool_calls: 5,
            require_tool_approval: false,
            workspace_scope: None,
        }
    }
}

/// Security action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    ToolCall { name: String, args: serde_json::Value },
    FetchUrl { url: String },
    SaveData { key: String },
}

/// Security decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityDecision {
    Allow,
    Deny { reason: String },
    RequireApproval { message: String },
}

/// Security manager
pub struct SecurityManager {
    config: SecurityConfig,
    pending_approvals: HashMap<String, SecurityAction>,
    approved_actions: HashSet<String>,
    denied_actions: HashSet<String>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        SecurityManager {
            config,
            pending_approvals: HashMap::new(),
            approved_actions: HashSet::new(),
            denied_actions: HashSet::new(),
        }
    }

    /// Check if an action is allowed
    pub fn check_action(&self, action: &SecurityAction) -> SecurityDecision {
        // Sandbox check
        if self.config.sandbox_enabled {
            if let Some(reason) = self.check_sandbox(action) {
                return SecurityDecision::Deny { reason };
            }
        }

        // Allowlist check
        if let Some(reason) = self.check_allowlist(action) {
            return SecurityDecision::Deny { reason };
        }

        // Pairing check
        if self.config.pairing_enabled && self.config.require_tool_approval {
            let action_id = self.generate_action_id(action);
            if !self.approved_actions.contains(&action_id) {
                return SecurityDecision::RequireApproval {
                    message: format!("Approval required for: {:?}", action),
                };
            }
        }

        SecurityDecision::Allow
    }

    /// Check sandbox restrictions
    fn check_sandbox(&self, action: &SecurityAction) -> Option<String> {
        match action {
            SecurityAction::FetchUrl { url } => {
                // Check blocked domains
                if let Some(domain) = extract_domain(url) {
                    if self.config.blocked_domains.iter().any(|d| domain.contains(d)) {
                        return Some(format!("Domain '{}' is blocked", domain));
                    }
                }
            }
            SecurityAction::ToolCall { name, .. } => {
                // Check blocked tools
                if self.config.blocked_tools.contains(name) {
                    return Some(format!("Tool '{}' is blocked", name));
                }
            }
            _ => {}
        }
        None
    }

    /// Check allowlist restrictions
    fn check_allowlist(&self, action: &SecurityAction) -> Option<String> {
        match action {
            SecurityAction::FetchUrl { url } => {
                // Check if domain is in allowed list
                if let Some(domain) = extract_domain(url) {
                    if !self.config.allowed_domains.is_empty() 
                        && !self.config.allowed_domains.iter().any(|d| domain.contains(d)) {
                        return Some(format!("Domain '{}' is not in allowlist", domain));
                    }
                }
            }
            SecurityAction::ToolCall { name, .. } => {
                // Check if tool is in allowed list
                if !self.config.allowed_tools.is_empty() 
                    && !self.config.allowed_tools.contains(name) {
                    return Some(format!("Tool '{}' is not in allowlist", name));
                }
            }
            _ => {}
        }
        None
    }

    /// Generate unique action ID
    fn generate_action_id(&self, action: &SecurityAction) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{:?}", action).hash(&mut hasher);
        format!("action_{:x}", hasher.finish())
    }

    /// Approve a pending action
    pub fn approve_action(&mut self, action_id: &str) -> Result<(), JsValue> {
        if let Some(action) = self.pending_approvals.remove(action_id) {
            let id = self.generate_action_id(&action);
            self.approved_actions.insert(id);
            Ok(())
        } else {
            Err(JsValue::from_str(&format!("No pending action with ID: {}", action_id)))
        }
    }

    /// Deny a pending action
    pub fn deny_action(&mut self, action_id: &str) -> Result<(), JsValue> {
        if let Some(action) = self.pending_approvals.remove(action_id) {
            let id = self.generate_action_id(&action);
            self.denied_actions.insert(id);
            Ok(())
        } else {
            Err(JsValue::from_str(&format!("No pending action with ID: {}", action_id)))
        }
    }

    /// Add a pending action for approval
    pub fn add_pending_action(&mut self, action: SecurityAction) -> String {
        let action_id = self.generate_action_id(&action);
        self.pending_approvals.insert(action_id.clone(), action);
        action_id
    }

    /// Check if a tool is allowed
    pub fn is_tool_allowed(&self, name: &str) -> bool {
        if self.config.blocked_tools.contains(&name.to_string()) {
            return false;
        }
        if !self.config.allowed_tools.is_empty() {
            return self.config.allowed_tools.contains(&name.to_string());
        }
        true
    }

    /// Check if a URL is allowed
    pub fn is_url_allowed(&self, url: &str) -> bool {
        if let Some(domain) = extract_domain(url) {
            if self.config.blocked_domains.iter().any(|d| domain.contains(d)) {
                return false;
            }
            if !self.config.allowed_domains.is_empty() {
                return self.config.allowed_domains.iter().any(|d| domain.contains(d));
            }
        }
        true
    }

    /// Get allowed tools
    pub fn get_allowed_tools(&self) -> &[String] {
        &self.config.allowed_tools
    }

    /// Get allowed domains
    pub fn get_allowed_domains(&self) -> &[String] {
        &self.config.allowed_domains
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Clear all approvals
    pub fn clear_approvals(&mut self) {
        self.approved_actions.clear();
        self.denied_actions.clear();
        self.pending_approvals.clear();
    }

    /// Enable/disable pairing mode
    pub fn set_pairing_enabled(&mut self, enabled: bool) {
        self.config.pairing_enabled = enabled;
    }

    /// Enable/disable sandbox mode
    pub fn set_sandbox_enabled(&mut self, enabled: bool) {
        self.config.sandbox_enabled = enabled;
    }

    /// Add domain to allowlist
    pub fn allow_domain(&mut self, domain: String) {
        if !self.config.allowed_domains.contains(&domain) {
            self.config.allowed_domains.push(domain.clone());
        }
        // Remove from blocked if present
        self.config.blocked_domains.retain(|d| d != &domain);
    }

    /// Block a domain
    pub fn block_domain(&mut self, domain: String) {
        if !self.config.blocked_domains.contains(&domain) {
            self.config.blocked_domains.push(domain.clone());
        }
        // Remove from allowed if present
        self.config.allowed_domains.retain(|d| d != &domain);
    }

    /// Add tool to allowlist
    pub fn allow_tool(&mut self, tool: String) {
        if !self.config.allowed_tools.contains(&tool) {
            self.config.allowed_tools.push(tool.clone());
        }
        // Remove from blocked if present
        self.config.blocked_tools.retain(|t| t != &tool);
    }

    /// Block a tool
    pub fn block_tool(&mut self, tool: String) {
        if !self.config.blocked_tools.contains(&tool) {
            self.config.blocked_tools.push(tool.clone());
        }
        // Remove from allowed if present
        self.config.allowed_tools.retain(|t| t != &tool);
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    let url = url.trim();
    
    // Remove protocol
    let url = url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    
    // Get domain part
    let domain = url.split('/').next()?;
    
    // Remove port
    let domain = domain.split(':').next()?;
    
    Some(domain.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(extract_domain("http://sub.example.com:8080/path"), Some("sub.example.com".to_string()));
        assert_eq!(extract_domain("example.com"), Some("example.com".to_string()));
    }

    #[test]
    fn test_tool_allowlist() {
        let config = SecurityConfig {
            allowed_tools: vec!["web_search".to_string()],
            blocked_tools: vec!["fetch_url".to_string()],
            ..Default::default()
        };
        let manager = SecurityManager::new(config);
        
        assert!(manager.is_tool_allowed("web_search"));
        assert!(!manager.is_tool_allowed("fetch_url"));
        assert!(!manager.is_tool_allowed("unknown_tool"));
    }

    #[test]
    fn test_domain_allowlist() {
        let config = SecurityConfig {
            allowed_domains: vec!["example.com".to_string()],
            blocked_domains: vec!["blocked.com".to_string()],
            ..Default::default()
        };
        let manager = SecurityManager::new(config);
        
        assert!(manager.is_url_allowed("https://example.com/page"));
        assert!(!manager.is_url_allowed("https://blocked.com/page"));
        assert!(!manager.is_url_allowed("https://other.com/page"));
    }
}
