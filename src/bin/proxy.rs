//! Simple CORS Proxy Server for WebClaw
//! 
//! Usage: cargo run --bin proxy
//! 
//! This proxy bypasses CORS restrictions by acting as a middleman
//! between the browser and external APIs.

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_cors::Cors;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Channel support is compiled into proxy binary
#[path = "../channels_mod.rs"]
mod channels_mod;

#[derive(Debug, Serialize, Deserialize)]
struct ProxyRequest {
    url: String,
    #[serde(default)]
    method: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<String>,
}

async fn proxy_handler(
    req: web::Json<ProxyRequest>,
    _http_req: HttpRequest,
) -> HttpResponse {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    
    let method = match req.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        _ => reqwest::Method::GET,
    };
    
    let mut request = client.request(method, &req.url);
    
    // Add headers
    for (key, value) in &req.headers {
        request = request.header(key, value);
    }
    
    // Add body if present
    if let Some(body) = &req.body {
        request = request.body(body.clone());
    }
    
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            
            let mut builder = HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            );
            
            // Forward response headers
            for (name, value) in headers {
                if let Some(name) = name {
                    // Skip headers that might cause issues
                    if name != "content-encoding" && name != "transfer-encoding" {
                        if let Ok(v) = value.to_str() {
                            builder.append_header((name.as_str(), v));
                        }
                    }
                }
            }
            
            builder.body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Proxy error: {}", e))
        }
    }
}

async fn web_search_handler(
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let search_query = query.get("q").cloned().unwrap_or_default();
    
    // Use DuckDuckGo Instant Answer API
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(&search_query)
    );
    
    let client = Client::new();
    
    match client.get(&url).send().await {
        Ok(response) => {
            let body = response.text().await.unwrap_or_default();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Search error: {}", e))
        }
    }
}

async fn ollama_search_handler(
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    let client = Client::new();
    
    // Get Authorization header from request
    let auth_header = req.headers().get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    
    // Forward to Ollama Web Search API
    let mut request = client.post("https://ollama.com/api/web_search")
        .header("Content-Type", "application/json");
    
    if let Some(auth) = auth_header {
        request = request.header("Authorization", auth);
    }
    
    request = request.body(Vec::from(body));
    
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .content_type("application/json")
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Ollama search error: {}", e))
        }
    }
}

async fn reddit_search_handler(
    query: web::Query<HashMap<String, String>>,
) -> HttpResponse {
    let search_query = query.get("q").cloned().unwrap_or_default();
    let subreddit = query.get("subreddit").cloned().unwrap_or_else(|| "all".to_string());
    let limit: usize = query.get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    
    let client = Client::new();
    
    // Use Reddit's JSON API (no auth required for read-only)
    let url = format!(
        "https://www.reddit.com/r/{}/search.json?q={}&restrict_sr=on&limit={}&sort=relevance",
        subreddit,
        urlencoding::encode(&search_query),
        limit
    );
    
    // If subreddit is "all", use the regular search endpoint
    let url = if subreddit == "all" {
        format!(
            "https://www.reddit.com/search.json?q={}&limit={}&sort=relevance",
            urlencoding::encode(&search_query),
            limit
        )
    } else {
        url
    };
    
    match client.get(&url)
        .header("User-Agent", "WebClaw/0.1.0")
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            
            // Parse Reddit response and format it
            if let Ok(reddit_response) = serde_json::from_str::<RedditResponse>(&body) {
                let posts: Vec<RedditPostFormatted> = reddit_response.data.children.iter()
                    .filter_map(|c| {
                        let d = &c.data;
                        Some(RedditPostFormatted {
                            title: d.title.clone(),
                            subreddit: d.subreddit.clone(),
                            selftext: d.selftext.clone().unwrap_or_default(),
                            score: d.score.unwrap_or(0),
                            num_comments: d.num_comments.unwrap_or(0),
                            url: format!("https://reddit.com{}", d.permalink.clone().unwrap_or_default()),
                        })
                    })
                    .take(limit)
                    .collect();
                
                let result = RedditSearchResult { posts };
                
                return HttpResponse::build(
                    actix_web::http::StatusCode::from_u16(status.as_u16())
                        .unwrap_or(actix_web::http::StatusCode::OK)
                )
                .content_type("application/json")
                .json(&result);
            }
            
            HttpResponse::build(
                actix_web::http::StatusCode::from_u16(status.as_u16())
                    .unwrap_or(actix_web::http::StatusCode::OK)
            )
            .content_type("application/json")
            .body(body)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Reddit search error: {}", e))
        }
    }
}

#[derive(Debug, Deserialize)]
struct RedditResponse {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPostData,
}

#[derive(Debug, Deserialize)]
struct RedditPostData {
    title: String,
    subreddit: String,
    selftext: Option<String>,
    score: Option<i32>,
    num_comments: Option<i32>,
    permalink: Option<String>,
}

#[derive(Debug, Serialize)]
struct RedditSearchResult {
    posts: Vec<RedditPostFormatted>,
}

#[derive(Debug, Serialize)]
struct RedditPostFormatted {
    title: String,
    subreddit: String,
    selftext: String,
    score: i32,
    num_comments: i32,
    url: String,
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"
<!DOCTYPE html>
<html>
<head><title>WebClaw Proxy</title></head>
<body>
    <h1>WebClaw CORS Proxy</h1>
    <p>Proxy is running!</p>
    <h2>Endpoints:</h2>
    <ul>
        <li>POST /proxy - Generic proxy (JSON body: {"url": "...", "method": "GET", "headers": {}, "body": null})</li>
        <li>GET /search?q=query - DuckDuckGo search</li>
        <li>POST /ollama-search - Ollama Web Search API (JSON body: {"query": "...", "max_results": 5})</li>
    </ul>
    <h2>Channel Support:</h2>
    <ul>
        <li>POST /channel/telegram/send - Send Telegram message</li>
        <li>POST /channel/telegram/webhook - Telegram webhook</li>
        <li>POST /channel/discord/send - Send Discord message</li>
        <li>POST /channel/slack/send - Send Slack message</li>
        <li>POST /channel/slack/webhook - Slack webhook</li>
        <li>POST /channel/whatsapp/send - Send WhatsApp message</li>
        <li>GET/POST /channel/whatsapp/webhook - WhatsApp webhook</li>
    </ul>
</body>
</html>
"#)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 WebClaw CORS Proxy starting on http://localhost:3000");
    println!("   POST /proxy - Generic proxy endpoint");
    println!("   GET /search?q=query - DuckDuckGo search");
    println!("   POST /ollama-search - Ollama Web Search API");
    println!("   Channel endpoints: /channel/{{telegram,discord,slack,whatsapp}}/*");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/proxy", web::post().to(proxy_handler))
            .route("/search", web::get().to(web_search_handler))
            .route("/ollama-search", web::post().to(ollama_search_handler))
            .route("/reddit/search", web::get().to(reddit_search_handler))
            .configure(channels_mod::register_channels)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
