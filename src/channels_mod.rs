//! Channel Support for WebClaw Proxy
//! 
//! Supports Telegram, Discord, Slack, and WhatsApp Business API.
//! This module runs on the proxy server, not in WASM.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Telegram
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub username: Option<String>,  // @username without @ prefix (optional filter)
    pub webhook_url: Option<String>,  // For setting up webhook
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub chat_id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
}

pub async fn telegram_send(
    config: web::Json<TelegramConfig>,
    message: web::Json<TelegramMessage>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // If username is provided, get chat_id from username
    let chat_id = if let Some(username) = &config.username {
        // Get chat_id from username
        let get_chat_url = format!(
            "https://api.telegram.org/bot{}/getChat?chat_id=@{}",
            config.bot_token, username
        );
        
        match client.get(&get_chat_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text().await.unwrap_or_default();
                    if let Ok(chat_response) = serde_json::from_str::<TelegramChatResponse>(&body) {
                        if chat_response.ok {
                            chat_response.result.id.to_string()
                        } else {
                            return HttpResponse::BadRequest()
                                .body(format!("Username @{} not found. Make sure the bot has started a conversation with this user.", username));
                        }
                    } else {
                        return HttpResponse::InternalServerError()
                            .body(format!("Failed to parse chat response: {}", body));
                    }
                } else {
                    return HttpResponse::BadRequest()
                        .body(format!("Failed to get chat for @{}", username));
                }
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Telegram API error: {}", e));
            }
        }
    } else {
        // Use chat_id from message
        message.chat_id.clone()
    };
    
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );
    
    let send_body = serde_json::json!({
        "chat_id": chat_id,
        "text": message.text,
        "parse_mode": message.parse_mode
    });
    
    match client.post(&url)
        .json(&send_body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .body(format!("Telegram error: {}", e))
        }
    }
}

#[derive(Debug, Deserialize)]
struct TelegramChatResponse {
    ok: bool,
    result: TelegramChatInfo,
}

#[derive(Debug, Deserialize)]
struct TelegramChatInfo {
    id: i64,
}

pub async fn telegram_webhook(
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
) -> HttpResponse {
    // Handle webhook verification
    if let Some(mode) = query.get("hub.mode") {
        if mode == "subscribe" {
            if let Some(token) = query.get("hub.verify_token") {
                // Return challenge for verification
                if let Some(challenge) = query.get("hub.challenge") {
                    return HttpResponse::Ok().body(challenge.clone());
                }
            }
        }
    }
    
    // Parse Telegram update
    let update: Result<TelegramUpdate, _> = serde_json::from_slice(&body);
    
    match update {
        Ok(update) => {
            // Process incoming message
            if let Some(message) = update.message {
                if let Some(text) = message.text {
                    // Get bot token from config (stored globally or env)
                    // For now, just log and acknowledge
                    println!("📩 Telegram message from @{}: {}", 
                        message.from.username.as_deref().unwrap_or("unknown"),
                        text
                    );
                    
                    // Return OK - actual response would need bot token
                    return HttpResponse::Ok()
                        .content_type("application/json")
                        .json(serde_json::json!({
                            "status": "received",
                            "from": message.from.username,
                            "text": text
                        }));
                }
            }
            
            HttpResponse::Ok().body("OK")
        }
        Err(e) => {
            HttpResponse::BadRequest()
                .body(format!("Invalid Telegram update: {}", e))
        }
    }
}

/// Set up Telegram webhook
pub async fn telegram_set_webhook(
    config: web::Json<TelegramConfig>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // Webhook URL should be your public URL
    let webhook_url = config.webhook_url.as_deref().unwrap_or("http://localhost:3000/channel/telegram/webhook");
    
    let url = format!(
        "https://api.telegram.org/bot{}/setWebhook",
        config.bot_token
    );
    
    let body = serde_json::json!({
        "url": webhook_url
    });
    
    match client.post(&url)
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .body(format!("Telegram webhook error: {}", e))
        }
    }
}

/// Poll for Telegram updates and respond (for local development)
pub async fn telegram_poll(
    config: web::Json<TelegramConfig>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // Get updates
    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates",
        config.bot_token
    );
    
    let updates_response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Failed to get updates: {}", e))
    };
    
    let updates_body = match updates_response.text().await {
        Ok(b) => b,
        Err(e) => return HttpResponse::InternalServerError()
            .body(format!("Failed to read response: {}", e))
    };
    
    let updates: TelegramUpdatesResponse = match serde_json::from_str(&updates_body) {
        Ok(u) => u,
        Err(e) => return HttpResponse::BadRequest()
            .body(format!("Failed to parse updates: {} - {}", e, updates_body))
    };
    
    if !updates.ok {
        return HttpResponse::BadRequest().body("Failed to get updates");
    }
    
    let mut responses = Vec::new();
    
    // Process each update
    for update in updates.result {
        if let Some(message) = update.message {
            if let Some(text) = &message.text {
                // Respond to the message
                let chat_id = message.chat.id;
                let response_text = match text.as_str() {
                    "/start" => "👋 Merhaba! Ben WebClaw bot. Size nasıl yardımcı olabilirim?".to_string(),
                    "hey" | "hi" | "hello" | "merhaba" => "👋 Merhaba! Size nasıl yardımcı olabilirim?".to_string(),
                    _ => format!("📩 Mesajınız alındı: \"{}\"\n\nYapay zeka ile cevap vermek için WebClaw'ı kullanın.", text)
                };
                
                // Send response
                let send_url = format!(
                    "https://api.telegram.org/bot{}/sendMessage",
                    config.bot_token
                );
                
                let send_body = serde_json::json!({
                    "chat_id": chat_id,
                    "text": response_text
                });
                
                if let Ok(send_response) = client.post(&send_url).json(&send_body).send().await {
                    if send_response.status().is_success() {
                        responses.push(format!("✅ @{}: {}", 
                            message.from.username.as_deref().unwrap_or("unknown"),
                            text
                        ));
                    }
                }
            }
        }
    }
    
    HttpResponse::Ok()
        .content_type("application/json")
        .json(serde_json::json!({
            "processed": responses.len(),
            "messages": responses
        }))
}

#[derive(Debug, Deserialize)]
struct TelegramUpdatesResponse {
    ok: bool,
    result: Vec<TelegramUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<TelegramUpdateMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUpdateMessage {
    pub message_id: i64,
    pub from: TelegramUser,
    pub chat: TelegramChat,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub first_name: String,
    #[serde(default)]
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

// ============================================================================
// Discord
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    pub channel_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<DiscordEmbed>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordEmbed {
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<i32>,
}

pub async fn discord_send(
    config: web::Json<DiscordConfig>,
    message: web::Json<DiscordMessage>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://discord.com/api/v10/channels/{}/messages",
        config.channel_id
    );
    
    match client.post(&url)
        .header("Authorization", format!("Bot {}", config.bot_token))
        .header("Content-Type", "application/json")
        .json(&message.into_inner())
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .body(format!("Discord error: {}", e))
        }
    }
}

// ============================================================================
// Slack
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SlackConfig {
    pub bot_token: String,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct SlackResponse {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub ts: Option<String>,
}

pub async fn slack_send(
    config: web::Json<SlackConfig>,
    message: web::Json<SlackMessage>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    let mut msg = message.into_inner();
    msg.channel = config.channel.clone();
    
    match client.post("https://slack.com/api/chat.postMessage")
        .header("Authorization", format!("Bearer {}", config.bot_token))
        .header("Content-Type", "application/json")
        .json(&msg)
        .send()
        .await
    {
        Ok(response) => {
            let body = response.text().await.unwrap_or_default();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .body(format!("Slack error: {}", e))
        }
    }
}

pub async fn slack_webhook(
    body: web::Bytes,
) -> HttpResponse {
    // Parse Slack event
    let event: Result<SlackEvent, _> = serde_json::from_slice(&body);
    
    match event {
        Ok(event) => {
            // URL verification challenge
            if event.type_field == "url_verification" {
                return HttpResponse::Ok()
                    .content_type("text/plain")
                    .body(event.challenge.unwrap_or_default());
            }
            
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        }
        Err(e) => {
            HttpResponse::BadRequest()
                .body(format!("Invalid Slack event: {}", e))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SlackEvent {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub challenge: Option<String>,
    #[serde(default)]
    pub event: Option<SlackEventPayload>,
}

#[derive(Debug, Deserialize)]
pub struct SlackEventPayload {
    #[serde(rename = "type")]
    pub type_field: String,
    pub user: String,
    pub text: String,
    pub channel: String,
    pub ts: String,
}

// ============================================================================
// WhatsApp Business API
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct WhatsAppConfig {
    pub access_token: String,
    pub phone_number_id: String,
    #[serde(default)]
    pub verify_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub messaging_product: String,
    pub to: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: WhatsAppText,
}

impl WhatsAppMessage {
    pub fn new(to: String, body: String) -> Self {
        WhatsAppMessage {
            messaging_product: "whatsapp".to_string(),
            to,
            type_field: "text".to_string(),
            text: WhatsAppText { body },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppText {
    pub body: String,
}

pub async fn whatsapp_send(
    config: web::Json<WhatsAppConfig>,
    to: web::Query<HashMap<String, String>>,
    message: web::Json<WhatsAppMessage>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    let url = format!(
        "https://graph.facebook.com/v18.0/{}/messages",
        config.phone_number_id
    );
    
    let mut msg = message.into_inner();
    if let Some(to_number) = to.get("to") {
        msg.to = to_number.clone();
    }
    
    match client.post(&url)
        .header("Authorization", format!("Bearer {}", config.access_token))
        .header("Content-Type", "application/json")
        .json(&msg)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .body(format!("WhatsApp error: {}", e))
        }
    }
}

pub async fn whatsapp_webhook(
    query: web::Query<HashMap<String, String>>,
    body: Option<web::Bytes>,
) -> HttpResponse {
    // Webhook verification
    if let Some(mode) = query.get("hub.mode") {
        if mode == "subscribe" {
            if let Some(token) = query.get("hub.verify_token") {
                // In production, verify this token against config
                if let Some(challenge) = query.get("hub.challenge") {
                    return HttpResponse::Ok()
                        .content_type("text/plain")
                        .body(challenge.clone());
                }
            }
        }
    }
    
    // Handle incoming message
    if let Some(body) = body {
        let entry: Result<WhatsAppWebhookEntry, _> = serde_json::from_slice(&body);
        match entry {
            Ok(entry) => {
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(e) => {
                HttpResponse::BadRequest()
                    .body(format!("Invalid WhatsApp webhook: {}", e))
            }
        }
    } else {
        HttpResponse::Ok().finish()
    }
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppWebhookEntry {
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppEntry {
    pub id: String,
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppChange {
    pub field: String,
    pub value: WhatsAppValue,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    pub messages: Vec<WhatsAppIncomingMessage>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppIncomingMessage {
    pub from: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub text: Option<WhatsAppTextContent>,
}

#[derive(Debug, Deserialize)]
pub struct WhatsAppTextContent {
    pub body: String,
}

// ============================================================================
// Channel Registry
// ============================================================================

pub fn register_channels(cfg: &mut web::ServiceConfig) {
    cfg
        // Telegram
        .route("/channel/telegram/send", web::post().to(telegram_send))
        .route("/channel/telegram/webhook", web::get().to(telegram_webhook))
        .route("/channel/telegram/webhook", web::post().to(telegram_webhook))
        .route("/channel/telegram/set-webhook", web::post().to(telegram_set_webhook))
        .route("/channel/telegram/poll", web::post().to(telegram_poll))
        
        // Discord
        .route("/channel/discord/send", web::post().to(discord_send))
        
        // Slack
        .route("/channel/slack/send", web::post().to(slack_send))
        .route("/channel/slack/webhook", web::post().to(slack_webhook))
        
        // WhatsApp
        .route("/channel/whatsapp/send", web::post().to(whatsapp_send))
        .route("/channel/whatsapp/webhook", web::get().to(whatsapp_webhook))
        .route("/channel/whatsapp/webhook", web::post().to(whatsapp_webhook));
}
