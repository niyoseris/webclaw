//! Tools module for claWasm - Skills and function calling

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, RequestMode, Response, Blob, BlobPropertyBag};
use wasm_bindgen::JsCast;
use js_sys::Array;

/// Tool definition for AI function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub result: String,
    pub success: bool,
}

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "web_search".to_string(),
            description: "Search the web for current information. Returns search results with titles, URLs, and snippets.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "reddit_search".to_string(),
            description: "Search Reddit for posts and discussions. Returns post titles, content, scores, and URLs.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query for Reddit posts"
                    },
                    "subreddit": {
                        "type": "string",
                        "description": "Optional subreddit to search in (without r/ prefix)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "image_search".to_string(),
            description: "Search for images on the web. Returns image URLs, titles, and source pages. Use this to find images for PDFs or research.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query for images"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of images to return (default: 5)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "get_current_time".to_string(),
            description: "Get the current date and time".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "calculate".to_string(),
            description: "Perform a mathematical calculation. Supports basic arithmetic, powers, and common functions.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "Mathematical expression to evaluate (e.g., '2+2', 'sqrt(16)', 'sin(3.14)')"
                    }
                },
                "required": ["expression"]
            }),
        },
        ToolDefinition {
            name: "fetch_url".to_string(),
            description: "Fetch and extract text content from a URL".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch content from"
                    }
                },
                "required": ["url"]
            }),
        },
        ToolDefinition {
            name: "save_note".to_string(),
            description: "Save a note to browser local storage for later retrieval".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Note title"
                    },
                    "content": {
                        "type": "string",
                        "description": "Note content"
                    }
                },
                "required": ["title", "content"]
            }),
        },
        ToolDefinition {
            name: "read_notes".to_string(),
            description: "Read all saved notes from browser local storage".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "create_pdf".to_string(),
            description: "Create a PDF document with text content and optional images. Returns a downloadable file ID. Images can be URLs or base64 data.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "PDF document title"
                    },
                    "content": {
                        "type": "string",
                        "description": "PDF content (markdown format supported)"
                    },
                    "filename": {
                        "type": "string",
                        "description": "Optional filename for the PDF (without .pdf extension)"
                    },
                    "images": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "url": {"type": "string", "description": "Image URL or base64 data URI"},
                                "caption": {"type": "string", "description": "Optional image caption"},
                                "width": {"type": "number", "description": "Image width in mm (default: 170)"},
                                "height": {"type": "number", "description": "Image height in mm (auto if not set)"}
                            }
                        },
                        "description": "Array of images to include in the PDF"
                    }
                },
                "required": ["title", "content"]
            }),
        },
        ToolDefinition {
            name: "download_file".to_string(),
            description: "Trigger download of a previously created file (PDF or Audio). Returns download status.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_id": {
                        "type": "string",
                        "description": "The file ID returned from create_pdf or text_to_speech"
                    }
                },
                "required": ["file_id"]
            }),
        },
        ToolDefinition {
            name: "list_files".to_string(),
            description: "List all previously created files (PDFs, audio files) that can be downloaded. Use this to see available files and their IDs.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "get_conversation".to_string(),
            description: "Get the current conversation history as text. Use this when the user asks to create a PDF or summary of the current discussion - you can use the conversation content directly instead of doing new research.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "description": "Output format: 'text' (plain text), 'markdown' (formatted), or 'summary' (brief summary)"
                    }
                },
                "required": []
            }),
        },
        // Self-evolving tools
        ToolDefinition {
            name: "create_tool".to_string(),
            description: "Create a new custom tool with JavaScript code. The tool will be saved and can be used immediately. Use this to extend your own capabilities!".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Tool name (lowercase, underscores allowed)"
                    },
                    "description": {
                        "type": "string",
                        "description": "What this tool does"
                    },
                    "parameters_schema": {
                        "type": "object",
                        "description": "JSON schema for tool parameters"
                    },
                    "code": {
                        "type": "string",
                        "description": "JavaScript code. Use 'args' for parameters. Return a string result. Example: 'return args.query.toUpperCase();'"
                    }
                },
                "required": ["name", "description", "parameters_schema", "code"]
            }),
        },
        ToolDefinition {
            name: "list_custom_tools".to_string(),
            description: "List all custom tools created by the AI".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "research".to_string(),
            description: "Deep research on a topic. Searches web, fetches URLs, and synthesizes findings into a comprehensive report.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": {
                        "type": "string",
                        "description": "The topic to research"
                    },
                    "depth": {
                        "type": "string",
                        "enum": ["quick", "normal", "deep"],
                        "description": "Research depth (default: normal)"
                    }
                },
                "required": ["topic"]
            }),
        },
        ToolDefinition {
            name: "delete_tool".to_string(),
            description: "Delete a custom tool by name".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the tool to delete"
                    }
                },
                "required": ["name"]
            }),
        },
        // Security & Vulnerability Scanners
        ToolDefinition {
            name: "scan_xss".to_string(),
            description: "Scan a URL or HTML content for XSS (Cross-Site Scripting) vulnerabilities. Tests for common injection points and sanitization issues.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to scan for XSS vulnerabilities"
                    },
                    "html": {
                        "type": "string",
                        "description": "HTML content to scan (alternative to URL)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "scan_sqli".to_string(),
            description: "Scan a URL for SQL Injection vulnerabilities. Tests common injection patterns and reports potential risks.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL with parameters to test for SQL injection"
                    },
                    "param": {
                        "type": "string",
                        "description": "Specific parameter to test (optional, tests all if not specified)"
                    }
                },
                "required": ["url"]
            }),
        },
        ToolDefinition {
            name: "scan_headers".to_string(),
            description: "Check security headers of a URL. Analyzes HTTP headers for security best practices (CSP, HSTS, X-Frame-Options, etc.).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to check security headers"
                    }
                },
                "required": ["url"]
            }),
        },
        ToolDefinition {
            name: "scan_ssl".to_string(),
            description: "Check SSL/TLS configuration of a domain. Verifies certificate validity, protocol support, and common weaknesses.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "domain": {
                        "type": "string",
                        "description": "Domain to check SSL/TLS configuration"
                    }
                },
                "required": ["domain"]
            }),
        },
        ToolDefinition {
            name: "scan_deps".to_string(),
            description: "Scan package dependencies for known vulnerabilities. Checks against CVE database for outdated or vulnerable packages.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "package": {
                        "type": "string",
                        "description": "Package name to check (e.g., 'lodash', 'express')"
                    },
                    "version": {
                        "type": "string",
                        "description": "Package version (optional)"
                    },
                    "ecosystem": {
                        "type": "string",
                        "description": "Package ecosystem: npm, pip, cargo, maven (default: npm)"
                    }
                },
                "required": ["package"]
            }),
        },
        ToolDefinition {
            name: "scan_secrets".to_string(),
            description: "Scan code or text for exposed secrets (API keys, tokens, passwords). Detects patterns for AWS keys, GitHub tokens, JWTs, etc.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code or text to scan for secrets"
                    }
                },
                "required": ["code"]
            }),
        },
        ToolDefinition {
            name: "scan_cors".to_string(),
            description: "Check CORS (Cross-Origin Resource Sharing) configuration of a URL. Tests for misconfigurations that could allow unauthorized access.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "URL to check CORS configuration"
                    }
                },
                "required": ["url"]
            }),
        },
        // Audio & Media Tools
        ToolDefinition {
            name: "text_to_speech".to_string(),
            description: "Convert text to speech audio file and download it. Creates an MP3 audio file from text using Google Translate TTS. Supports multiple languages including Turkish (tr), English (en), German (de), French (fr), etc.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to convert to speech (max 200 characters per call)"
                    },
                    "lang": {
                        "type": "string",
                        "description": "Language code: tr (Turkish), en (English), de (German), fr (French), es (Spanish), it (Italian), ru (Russian), ar (Arabic). Default: tr"
                    },
                    "filename": {
                        "type": "string",
                        "description": "Filename for the audio file (without .mp3 extension)"
                    }
                },
                "required": ["text"]
            }),
        },
        ToolDefinition {
            name: "speak".to_string(),
            description: "Speak text aloud using browser's built-in speech synthesis. Does NOT create a file, just speaks the text. Use text_to_speech if you need a downloadable audio file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The text to speak aloud"
                    },
                    "lang": {
                        "type": "string",
                        "description": "Language code (e.g., 'tr-TR', 'en-US'). Default: tr-TR"
                    },
                    "rate": {
                        "type": "number",
                        "description": "Speech rate (0.1 to 10, default: 1)"
                    }
                },
                "required": ["text"]
            }),
        },
    ]
}

/// Get tools in OpenAI function format
pub fn get_tools_openai_format() -> Vec<serde_json::Value> {
    get_tool_definitions()
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                }
            })
        })
        .collect()
}

/// Execute a tool by name with given arguments
pub async fn execute_tool(name: &str, args: &serde_json::Value) -> Result<String, JsValue> {
    match name {
        "web_search" => execute_web_search(args).await,
        "reddit_search" => execute_reddit_search(args).await,
        "image_search" => execute_image_search(args).await,
        "get_current_time" => execute_get_time(args).await,
        "calculate" => execute_calculate(args).await,
        "fetch_url" => execute_fetch_url(args).await,
        "save_note" => execute_save_note(args).await,
        "read_notes" => execute_read_notes(args).await,
        "create_pdf" => execute_create_pdf(args).await,
        "download_file" => execute_download_file(args).await,
        "list_files" => execute_list_files(args).await,
        "get_conversation" => execute_get_conversation(args).await,
        // Self-evolving tools
        "create_tool" => execute_create_tool(args).await,
        "list_custom_tools" => execute_list_custom_tools(args).await,
        "research" => execute_research(args).await,
        "delete_tool" => execute_delete_tool(args).await,
        // Security & Vulnerability Scanners
        "scan_xss" => execute_scan_xss(args).await,
        "scan_sqli" => execute_scan_sqli(args).await,
        "scan_headers" => execute_scan_headers(args).await,
        "scan_ssl" => execute_scan_ssl(args).await,
        "scan_deps" => execute_scan_deps(args).await,
        "scan_secrets" => execute_scan_secrets(args).await,
        "scan_cors" => execute_scan_cors(args).await,
        // Audio & Media
        "text_to_speech" => execute_text_to_speech(args).await,
        "speak" => execute_speak(args).await,
        // Dynamic custom tool execution
        other => execute_custom_tool(other, args).await,
    }
}

/// Web search using DuckDuckGo via local CORS proxy
async fn execute_web_search(args: &serde_json::Value) -> Result<String, JsValue> {
    let query = args["query"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'query' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Use DuckDuckGo via proxy /search endpoint (no API key needed)
    let encoded_query = urlencoding::encode(query);
    let url = format!("http://localhost:3000/search?q={}", encoded_query);
    
    let request_init = RequestInit::new();
    request_init.set_method("GET");
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(&url, &request_init)?;
    
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "Search failed: {}. Make sure proxy server is running (./start.sh)",
            response.status()
        )));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let ddg: serde_json::Value = serde_wasm_bindgen::from_value(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    let mut results: Vec<String> = Vec::new();
    
    // DuckDuckGo Abstract (top result)
    if let Some(abstract_text) = ddg["Abstract"].as_str() {
        if !abstract_text.is_empty() {
            let source = ddg["AbstractSource"].as_str().unwrap_or("");
            let url = ddg["AbstractURL"].as_str().unwrap_or("");
            results.push(format!("**{}**\n{}\n{}", source, abstract_text, url));
        }
    }
    
    // Related topics
    if let Some(topics) = ddg["RelatedTopics"].as_array() {
        for topic in topics.iter().take(8) {
            if let (Some(text), Some(url)) = (
                topic["Text"].as_str(),
                topic["FirstURL"].as_str()
            ) {
                if !text.is_empty() {
                    results.push(format!("• {}\n  {}", text, url));
                }
            }
        }
    }
    
    if results.is_empty() {
        return Ok(format!("No results found for: {}", query));
    }
    
    Ok(format!("Search results for '{}':\n\n{}", query, results.join("\n\n")))
}

/// Image search using Wikipedia API via proxy
async fn execute_image_search(args: &serde_json::Value) -> Result<String, JsValue> {
    let query = args["query"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'query' parameter"))?;
    let limit = args["limit"].as_i64().unwrap_or(5) as usize;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Use Wikipedia API for images
    let proxy_url = "http://localhost:3000/proxy";
    let encoded_query = urlencoding::encode(query);
    
    // Wikipedia API: search for images
    let search_url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srnamespace=6&srlimit={}&format=json",
        encoded_query, limit
    );
    
    let body = serde_json::json!({
        "url": search_url,
        "method": "GET",
        "headers": {}
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    let body_json = JsValue::from_str(&serde_json::to_string(&body).unwrap());
    request_init.set_body(&body_json);
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(&proxy_url, &request_init)?;
    
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    let text = JsFuture::from(response.text()?).await?;
    let text = text.as_string().unwrap_or_default();
    
    // Parse Wikipedia search results and get image URLs
    let images = parse_wikipedia_images(&text, limit);
    
    if images.is_empty() {
        // Fallback: provide direct Wikipedia image search URL
        return Ok(format!(
            "No images found via API. Try these:\n\n🖼️ **Wikipedia Images:**\nhttps://commons.wikimedia.org/w/index.php?search={}&title=Special:MediaSearch\n\n🖼️ **Google Images:**\nhttps://www.google.com/search?tbm=isch&q={}\n\nYou can use these URLs in create_pdf with the images parameter.",
            urlencoding::encode(query), urlencoding::encode(query)
        ));
    }
    
    let results: Vec<String> = images.iter()
        .map(|img| format!("🖼️ **{}**\nURL: {}\nSource: {}", img.title, img.url, img.source))
        .collect();
    
    Ok(format!("Image search results for '{}':\n\n{}", query, results.join("\n\n")))
}

#[derive(Debug, Clone)]
struct ImageResult {
    title: String,
    url: String,
    source: String,
}

/// Parse Wikipedia image search results
fn parse_wikipedia_images(json: &str, limit: usize) -> Vec<ImageResult> {
    let mut images = Vec::new();
    
    // Parse Wikipedia API response
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(search_results) = parsed["query"]["search"].as_array() {
            for result in search_results.iter().take(limit) {
                if let Some(title) = result["title"].as_str() {
                    // Wikipedia image URLs follow a pattern
                    // File:Example.jpg -> https://upload.wikimedia.org/wikipedia/commons/thumb/...
                    let image_url = format!(
                        "https://commons.wikimedia.org/wiki/{}",
                        urlencoding::encode(title)
                    );
                    
                    images.push(ImageResult {
                        title: title.replace("File:", ""),
                        url: image_url,
                        source: "Wikipedia Commons".to_string(),
                    });
                }
            }
        }
    }
    
    // Also try to extract direct image URLs from text
    let urls = extract_urls(json, limit);
    for url in urls {
        if (url.contains(".jpg") || url.contains(".png") || url.contains(".gif") || 
            url.contains(".jpeg") || url.contains(".webp") || url.contains("upload.wikimedia.org"))
            && !images.iter().any(|i| i.url == url) {
            images.push(ImageResult {
                title: "Image".to_string(),
                url: url.clone(),
                source: url,
            });
        }
    }
    
    images
}

/// Get current time
async fn execute_get_time(_args: &serde_json::Value) -> Result<String, JsValue> {
    let now = chrono::Local::now();
    Ok(format!(
        "Current date and time: {}",
        now.format("%Y-%m-%d %H:%M:%S %Z")
    ))
}

/// Calculate mathematical expression
async fn execute_calculate(args: &serde_json::Value) -> Result<String, JsValue> {
    let expression = args["expression"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'expression' parameter"))?;
    
    // Simple expression evaluator
    let result = evaluate_math(expression)?;
    Ok(format!("Result: {}", result))
}

/// Simple math expression evaluator
fn evaluate_math(expr: &str) -> Result<f64, JsValue> {
    let expr = expr.trim();
    
    // Handle basic operations
    // This is a simplified evaluator - for production use a proper math parser
    
    // Try to parse as a simple number first
    if let Ok(n) = expr.parse::<f64>() {
        return Ok(n);
    }
    
    // Handle basic arithmetic
    let expr = expr.replace(" ", "");
    
    // Addition
    if let Some(pos) = expr.find('+') {
        if pos > 0 {
            let left = evaluate_math(&expr[..pos])?;
            let right = evaluate_math(&expr[pos+1..])?;
            return Ok(left + right);
        }
    }
    
    // Subtraction (not at start)
    if let Some(pos) = expr[1..].find('-') {
        let pos = pos + 1;
        let left = evaluate_math(&expr[..pos])?;
        let right = evaluate_math(&expr[pos+1..])?;
        return Ok(left - right);
    }
    
    // Multiplication
    if let Some(pos) = expr.find('*') {
        let left = evaluate_math(&expr[..pos])?;
        let right = evaluate_math(&expr[pos+1..])?;
        return Ok(left * right);
    }
    
    // Division
    if let Some(pos) = expr.find('/') {
        let left = evaluate_math(&expr[..pos])?;
        let right = evaluate_math(&expr[pos+1..])?;
        if right == 0.0 {
            return Err(JsValue::from_str("Division by zero"));
        }
        return Ok(left / right);
    }
    
    // Power
    if let Some(pos) = expr.find('^') {
        let left = evaluate_math(&expr[..pos])?;
        let right = evaluate_math(&expr[pos+1..])?;
        return Ok(left.powf(right));
    }
    
    // Functions
    if expr.starts_with("sqrt(") && expr.ends_with(')') {
        let inner = &expr[5..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.sqrt());
    }
    
    if expr.starts_with("sin(") && expr.ends_with(')') {
        let inner = &expr[4..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.sin());
    }
    
    if expr.starts_with("cos(") && expr.ends_with(')') {
        let inner = &expr[4..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.cos());
    }
    
    if expr.starts_with("tan(") && expr.ends_with(')') {
        let inner = &expr[4..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.tan());
    }
    
    if expr.starts_with("abs(") && expr.ends_with(')') {
        let inner = &expr[4..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.abs());
    }
    
    if expr.starts_with("log(") && expr.ends_with(')') {
        let inner = &expr[4..expr.len()-1];
        let val = evaluate_math(inner)?;
        return Ok(val.ln());
    }
    
    // Handle parentheses
    if expr.starts_with('(') && expr.ends_with(')') {
        return evaluate_math(&expr[1..expr.len()-1]);
    }
    
    Err(JsValue::from_str(&format!("Cannot evaluate: {}", expr)))
}

/// Fetch URL content via proxy server (CORS bypass)
async fn execute_fetch_url(args: &serde_json::Value) -> Result<String, JsValue> {
    let url = args["url"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'url' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Use proxy server for CORS bypass
    let proxy_url = format!(
        "http://localhost:3000/proxy",
    );
    
    let body = serde_json::json!({
        "url": url,
        "method": "GET"
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    let body_json = JsValue::from_str(&serde_json::to_string(&body).unwrap());
    request_init.set_body(&body_json);
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(&proxy_url, &request_init)?;
    
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "Fetch failed: {}. Make sure proxy server is running (cargo run --bin proxy --features proxy)",
            response.status()
        )));
    }
    
    let text = JsFuture::from(response.text()?).await?;
    let text = text.as_string().unwrap_or_default();
    
    // Simple text extraction - remove HTML tags
    let text = remove_html_tags(&text);
    
    // Limit to first 3000 characters (UTF-8 safe)
    if text.chars().count() > 3000 {
        Ok(format!("{}...(truncated)", text.chars().take(3000).collect::<String>()))
    } else {
        Ok(text)
    }
}

/// Simple HTML tag removal
fn remove_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(c);
        }
    }
    
    // Clean up whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Save note to localStorage
async fn execute_save_note(args: &serde_json::Value) -> Result<String, JsValue> {
    let title = args["title"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'title' parameter"))?;
    let content = args["content"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'content' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    // Get existing notes
    let notes_json = storage.get_item("clawasm_notes")?.unwrap_or_default();
    let mut notes: Vec<Note> = if notes_json.is_empty() {
        Vec::new()
    } else {
        serde_json::from_str(&notes_json).unwrap_or_default()
    };
    
    // Add new note
    notes.push(Note {
        title: title.to_string(),
        content: content.to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
    });
    
    // Save
    let notes_json = serde_json::to_string(&notes)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    storage.set_item("clawasm_notes", &notes_json)?;
    
    Ok(format!("Note '{}' saved successfully", title))
}

/// Read notes from localStorage
async fn execute_read_notes(_args: &serde_json::Value) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let notes_json = storage.get_item("clawasm_notes")?.unwrap_or_default();
    
    if notes_json.is_empty() {
        return Ok("No notes found".to_string());
    }
    
    let notes: Vec<Note> = serde_json::from_str(&notes_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    if notes.is_empty() {
        return Ok("No notes found".to_string());
    }
    
    let result: Vec<String> = notes.iter().map(|n| {
        format!("Title: {}\nContent: {}\nCreated: {}", n.title, n.content, n.created_at)
    }).collect();
    
    Ok(result.join("\n\n---\n\n"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    title: String,
    content: String,
    created_at: String,
}

/// Reddit search via proxy server
async fn execute_reddit_search(args: &serde_json::Value) -> Result<String, JsValue> {
    let query = args["query"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'query' parameter"))?;
    let subreddit = args["subreddit"].as_str().unwrap_or("all");
    let limit = args["limit"].as_u64().unwrap_or(10) as usize;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Use proxy server for Reddit API
    let url = format!(
        "http://localhost:3000/reddit/search?q={}&subreddit={}&limit={}",
        urlencoding::encode(query),
        urlencoding::encode(subreddit),
        limit
    );
    
    let request_init = RequestInit::new();
    request_init.set_method("GET");
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init(&url, &request_init)?;
    
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "Reddit search failed: {}. Make sure proxy server is running",
            response.status()
        )));
    }
    
    let json = JsFuture::from(response.json()?).await?;
    let search_result: RedditSearchResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
    
    if search_result.posts.is_empty() {
        return Ok(format!("No Reddit posts found for: {}", query));
    }
    
    let results: Vec<String> = search_result.posts.iter()
        .map(|p| {
            format!(
                "**{}** (r/{})\n⬆️ {} | 💬 {} comments\n{}\n{}",
                p.title, p.subreddit, p.score, p.num_comments,
                p.selftext,  // Full text, no truncation
                p.url
            )
        })
        .collect();
    
    Ok(format!("Reddit search results for '{}':\n\n{}", query, results.join("\n\n---\n\n")))
}

#[derive(Debug, Deserialize)]
struct RedditSearchResponse {
    posts: Vec<RedditPost>,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    title: String,
    subreddit: String,
    selftext: String,
    score: i32,
    num_comments: i32,
    url: String,
}

/// Create PDF document using HTML-to-PDF (browser print dialog)
async fn execute_create_pdf(args: &serde_json::Value) -> Result<String, JsValue> {
    let title = args["title"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'title' parameter"))?;
    let content = args["content"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'content' parameter"))?;
    let filename = args["filename"].as_str()
        .unwrap_or(title)
        .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-', "_");
    let images = args["images"].as_array();
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Generate unique file ID
    let file_id = format!("pdf_{}", chrono::Utc::now().timestamp_millis());
    
    // Store PDF data in localStorage for later download
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let pdf_data = PdfFile {
        id: file_id.clone(),
        title: title.to_string(),
        content: content.to_string(),
        filename: format!("{}.pdf", filename),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    let pdf_json = serde_json::to_string(&pdf_data)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    storage.set_item(&file_id, &pdf_json)?;
    
    // Also store in file index
    let mut file_index: Vec<String> = storage.get_item("clawasm_files")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    file_index.push(file_id.clone());
    storage.set_item("clawasm_files", &serde_json::to_string(&file_index).unwrap())?;
    
    // Prepare images JSON
    let images_json = images
        .map(|imgs| serde_json::to_string(imgs).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|| "[]".to_string());
    
    // Escape content for HTML
    let content_escaped = content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    
    // Escape title for HTML
    let title_escaped = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    
    // Convert markdown-like content to HTML
    let html_content = markdown_to_html(&content_escaped);
    
    // Create HTML document for printing
    let js_code = format!(r#"
        (async function() {{
            const title = "{}";
            const contentHtml = `{}`;
            const images = {};
            const filename = "{}";
            
            // Build images HTML
            let imagesHtml = '';
            if (images && images.length > 0) {{
                imagesHtml = '<div class="images-section"><h2>Images</h2>';
                for (const img of images) {{
                    try {{
                        // Fetch image via proxy
                        const proxyBody = JSON.stringify({{
                            url: img.url,
                            method: 'GET',
                            headers: {{}}
                        }});
                        
                        const response = await fetch('http://localhost:3000/proxy', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: proxyBody
                        }});
                        
                        if (response.ok) {{
                            const blob = await response.blob();
                            const dataUrl = await new Promise((resolve, reject) => {{
                                const reader = new FileReader();
                                reader.onload = () => resolve(reader.result);
                                reader.onerror = reject;
                                reader.readAsDataURL(blob);
                            }});
                            
                            imagesHtml += `<figure class="image-figure">
                                <img src="${{dataUrl}}" alt="${{img.caption || 'Image'}}" class="document-image">
                                ${{img.caption ? `<figcaption>${{img.caption}}</figcaption>` : ''}}
                            </figure>`;
                        }}
                    }} catch (e) {{
                        console.error('Image load error:', e);
                    }}
                }}
                imagesHtml += '</div>';
            }}
            
            // Create full HTML document
            const htmlDoc = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${{title}}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 20px;
            background: #fff;
        }}
        
        .document-header {{
            text-align: center;
            border-bottom: 3px solid #2563eb;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        
        .document-title {{
            font-size: 28px;
            font-weight: 700;
            color: #1a1a1a;
            margin-bottom: 10px;
        }}
        
        .document-meta {{
            font-size: 12px;
            color: #666;
        }}
        
        .document-content {{
            font-size: 14px;
            color: #333;
        }}
        
        .document-content h1 {{
            font-size: 24px;
            color: #1a1a1a;
            margin: 30px 0 15px 0;
            padding-bottom: 8px;
            border-bottom: 2px solid #e5e7eb;
        }}
        
        .document-content h2 {{
            font-size: 20px;
            color: #1f2937;
            margin: 25px 0 12px 0;
        }}
        
        .document-content h3 {{
            font-size: 16px;
            color: #374151;
            margin: 20px 0 10px 0;
        }}
        
        .document-content p {{
            margin: 12px 0;
            text-align: justify;
        }}
        
        .document-content ul, .document-content ol {{
            margin: 12px 0 12px 25px;
        }}
        
        .document-content li {{
            margin: 6px 0;
        }}
        
        .document-content code {{
            background: #f3f4f6;
            padding: 2px 6px;
            border-radius: 4px;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 13px;
        }}
        
        .document-content pre {{
            background: #1f2937;
            color: #e5e7eb;
            padding: 15px;
            border-radius: 8px;
            overflow-x: auto;
            margin: 15px 0;
        }}
        
        .document-content pre code {{
            background: none;
            color: inherit;
        }}
        
        .document-content blockquote {{
            border-left: 4px solid #2563eb;
            padding-left: 15px;
            margin: 15px 0;
            color: #4b5563;
            font-style: italic;
        }}
        
        .document-content strong {{
            font-weight: 600;
        }}
        
        .document-content em {{
            font-style: italic;
        }}
        
        .images-section {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 2px solid #e5e7eb;
        }}
        
        .image-figure {{
            margin: 20px 0;
            text-align: center;
        }}
        
        .document-image {{
            max-width: 100%;
            height: auto;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        figcaption {{
            font-size: 12px;
            color: #666;
            margin-top: 8px;
            font-style: italic;
        }}
        
        .document-footer {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #e5e7eb;
            text-align: center;
            font-size: 11px;
            color: #9ca3af;
        }}
        
        /* Print styles */
        @media print {{
            body {{
                padding: 0;
                max-width: none;
            }}
            
            .document-header {{
                border-bottom-color: #000;
            }}
            
            .document-image {{
                page-break-inside: avoid;
            }}
            
            .image-figure {{
                page-break-inside: avoid;
            }}
        }}
    </style>
</head>
<body>
    <header class="document-header">
        <h1 class="document-title">${{title}}</h1>
        <div class="document-meta">Created by claWasm • ${{new Date().toLocaleDateString()}}</div>
    </header>
    
    <main class="document-content">
        ${{contentHtml}}
        ${{imagesHtml}}
    </main>
    
    <footer class="document-footer">
        Generated by claWasm - Browser-based AI Assistant
    </footer>
</body>
</html>`;
            
            // Open in new window and trigger print
            const printWindow = window.open('', '_blank');
            if (!printWindow) {{
                return 'Error: Pop-up blocked. Please allow pop-ups and try again.';
            }}
            
            printWindow.document.write(htmlDoc);
            printWindow.document.close();
            
            // Wait for images to load then print
            setTimeout(() => {{
                printWindow.print();
            }}, 500);
            
            return 'PDF ready! Use the print dialog to save as PDF.';
        }})()
    "#, 
        title_escaped,
        html_content,
        images_json,
        filename
    );
    
    // Execute JavaScript
    let result = js_sys::eval(&js_code)
        .map_err(|e| JsValue::from_str(&format!("JavaScript error: {:?}", e)))?;
    
    let result_str = result.as_string().unwrap_or_else(|| "PDF created".to_string());
    
    Ok(format!(
        "✅ PDF '{}' created!\n📄 File: {}.pdf\n{}\n\n💡 Use 'Save as PDF' in the print dialog that opened.",
        title, filename, result_str
    ))
}

/// Convert markdown-like text to HTML
fn markdown_to_html(text: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut code_content = String::new();
    
    for line in text.lines() {
        // Code blocks
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }
        
        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }
        
        let trimmed = line.trim();
        
        // Empty line
        if trimmed.is_empty() {
            html.push_str("<br>\n");
            continue;
        }
        
        // Headers
        if trimmed.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", html_escape(&trimmed[4..])));
            continue;
        }
        if trimmed.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(&trimmed[3..])));
            continue;
        }
        if trimmed.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", html_escape(&trimmed[2..])));
            continue;
        }
        
        // Bullet lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let content = process_inline_formatting(&trimmed[2..]);
            html.push_str(&format!("<li>{}</li>\n", content));
            continue;
        }
        
        // Numbered lists
        if let Some(pos) = trimmed.find(". ") {
            if pos > 0 && trimmed[..pos].chars().all(|c| c.is_numeric()) {
                let content = process_inline_formatting(&trimmed[pos + 2..]);
                html.push_str(&format!("<li>{}</li>\n", content));
                continue;
            }
        }
        
        // Blockquotes
        if trimmed.starts_with("> ") {
            let content = process_inline_formatting(&trimmed[2..]);
            html.push_str(&format!("<blockquote>{}</blockquote>\n", content));
            continue;
        }
        
        // Regular paragraph
        let content = process_inline_formatting(trimmed);
        html.push_str(&format!("<p>{}</p>\n", content));
    }
    
    html
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Process inline formatting (bold, italic, code)
fn process_inline_formatting(s: &str) -> String {
    let mut result = html_escape(s);
    
    // Bold: **text** -> <strong>text</strong>
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let bold_text = &result[start + 2..start + 2 + end];
            let replacement = format!("<strong>{}</strong>", bold_text);
            result = format!("{}{}{}", &result[..start], replacement, &result[start + 2 + end + 2..]);
        } else {
            break;
        }
    }
    
    // Inline code: `code` -> <code>code</code>
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let code_text = &result[start + 1..start + 1 + end];
            let replacement = format!("<code>{}</code>", code_text);
            result = format!("{}{}{}", &result[..start], replacement, &result[start + 1 + end + 1..]);
        } else {
            break;
        }
    }
    
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfFile {
    id: String,
    title: String,
    content: String,
    filename: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AudioFile {
    id: String,
    text: String,
    lang: String,
    filename: String,
    created_at: String,
}

/// Download a previously created file (PDF or Audio)
async fn execute_download_file(args: &serde_json::Value) -> Result<String, JsValue> {
    let file_id = args["file_id"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'file_id' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    // Get file metadata
    let file_json = storage.get_item(file_id)?
        .ok_or_else(|| JsValue::from_str(&format!("File not found: {}", file_id)))?;
    
    // Check file type by ID prefix
    if file_id.starts_with("audio_") {
        // Audio file
        let audio_data: AudioFile = serde_json::from_str(&file_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        // Get base64 audio data
        let base64_data = storage.get_item(&format!("{}_data", file_id))?
            .ok_or_else(|| JsValue::from_str("Audio data not found"))?;
        
        // Decode base64 to binary
        let binary_string = js_sys::eval(&format!("atob('{}')", base64_data))
            .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {:?}", e)))?;
        let binary_string = binary_string.dyn_into::<js_sys::JsString>()
            .map_err(|e| JsValue::from_str(&format!("Cast error: {:?}", e)))?;
        let bytes: Vec<u8> = (0..binary_string.length())
            .map(|i| binary_string.char_code_at(i) as u8)
            .collect();
        
        // Create blob
        let array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        array.copy_from(&bytes);
        
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&array);
        
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &blob_parts,
            web_sys::BlobPropertyBag::new().type_("audio/mpeg")
        ).map_err(|e| JsValue::from_str(&format!("Blob error: {:?}", e)))?;
        
        // Create object URL
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|e| JsValue::from_str(&format!("URL error: {:?}", e)))?;
        
        // Create download link and click it
        let link = document.create_element("a")?;
        let link: web_sys::HtmlElement = link.dyn_into()
            .map_err(|_| JsValue::from_str("Failed to create link"))?;
        
        link.set_attribute("href", &url)?;
        link.set_attribute("download", &audio_data.filename)?;
        link.set_attribute("style", "display: none")?;
        
        let body = document.body().ok_or_else(|| JsValue::from_str("No body"))?;
        body.append_child(&link)?;
        link.click();
        body.remove_child(&link)?;
        
        let _ = web_sys::Url::revoke_object_url(&url);
        
        Ok(format!("✅ Audio downloaded: {}\nText: \"{}\"", audio_data.filename, audio_data.text))
    } else if file_id.starts_with("pdf_") {
        // PDF file
        let pdf_data: PdfFile = serde_json::from_str(&file_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        // Get HTML content
        let html_content = storage.get_item(&format!("{}_html", file_id))?
            .unwrap_or_default();
        
        // Create blob and download link
        let html_bytes = html_content.as_bytes();
        let array = js_sys::Uint8Array::new_with_length(html_bytes.len() as u32);
        array.copy_from(html_bytes);
        
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&array);
        
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &blob_parts,
            web_sys::BlobPropertyBag::new().type_("text/html")
        ).map_err(|e| JsValue::from_str(&format!("Blob error: {:?}", e)))?;
        
        // Create object URL
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|e| JsValue::from_str(&format!("URL error: {:?}", e)))?;
        
        // Create download link and click it
        let link = document.create_element("a")?;
        let link: web_sys::HtmlElement = link.dyn_into()
            .map_err(|_| JsValue::from_str("Failed to create link"))?;
        
        link.set_attribute("href", &url)?;
        link.set_attribute("download", &format!("{}.html", pdf_data.filename.replace(".pdf", "")))?;
        link.set_attribute("style", "display: none")?;
        
        let body = document.body().ok_or_else(|| JsValue::from_str("No body"))?;
        body.append_child(&link)?;
        link.click();
        body.remove_child(&link)?;
        
        let _ = web_sys::Url::revoke_object_url(&url);
        
        Ok(format!(
            "✅ Download started: {}\n\nNote: This is an HTML file that can be opened in browser and printed as PDF.\nTo save as PDF: Open the file → Print → Save as PDF",
            pdf_data.filename
        ))
    } else {
        Err(JsValue::from_str(&format!("Unknown file type: {}", file_id)))
    }
}

/// List all saved files
async fn execute_list_files(_args: &serde_json::Value) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let file_index: Vec<String> = storage.get_item("clawasm_files")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    if file_index.is_empty() {
        return Ok("📁 No saved files found.\n\nCreate files using:\n- create_pdf (for PDFs)\n- text_to_speech (for audio)".to_string());
    }
    
    let mut result = String::from("📁 Saved Files:\n\n");
    
    for file_id in &file_index {
        if let Some(json) = storage.get_item(file_id).ok().flatten() {
            if file_id.starts_with("audio_") {
                if let Ok(audio) = serde_json::from_str::<AudioFile>(&json) {
                    result.push_str(&format!("🔊 {} - \"{}\" ({})\n   ID: {}\n   Created: {}\n\n", 
                        audio.filename, 
                        audio.text.chars().take(50).collect::<String>() + if audio.text.len() > 50 { "..." } else { "" },
                        audio.lang,
                        audio.id,
                        audio.created_at
                    ));
                }
            } else if file_id.starts_with("pdf_") {
                if let Ok(pdf) = serde_json::from_str::<PdfFile>(&json) {
                    result.push_str(&format!("📄 {} - \"{}\"\n   ID: {}\n   Created: {}\n\n", 
                        pdf.filename, 
                        pdf.title,
                        pdf.id,
                        pdf.created_at
                    ));
                }
            }
        }
    }
    
    result.push_str("\n💡 Use download_file with the file ID to download any file.");
    
    Ok(result)
}

/// Get current conversation history
async fn execute_get_conversation(args: &serde_json::Value) -> Result<String, JsValue> {
    let format = args["format"].as_str().unwrap_or("markdown");
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    // Get active session ID
    let active_session_id = storage.get_item("clawasm_active_session")
        .ok()
        .flatten()
        .unwrap_or_else(|| "default".to_string());
    
    // Get sessions
    let sessions_json = storage.get_item("clawasm_sessions")
        .ok()
        .flatten()
        .unwrap_or_else(|| "{}".to_string());
    
    let sessions: serde_json::Value = serde_json::from_str(&sessions_json)
        .unwrap_or(serde_json::json!({}));
    
    let messages = sessions.get(&active_session_id)
        .and_then(|s| s.get("messages"))
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();
    
    if messages.is_empty() {
        return Ok("📝 No conversation history found.".to_string());
    }
    
    let mut result = String::new();
    
    match format {
        "summary" => {
            result.push_str("📝 **Conversation Summary**\n\n");
            let user_count = messages.iter().filter(|m| m["role"] == "user").count();
            let assistant_count = messages.iter().filter(|m| m["role"] == "assistant").count();
            result.push_str(&format!("- {} user messages\n- {} assistant responses\n", user_count, assistant_count));
            if let Some(first) = messages.first() {
                if let Some(content) = first["content"].as_str() {
                    let preview: String = content.chars().take(100).collect();
                    result.push_str(&format!("\n**Started with:** {}...\n", preview));
                }
            }
        }
        "text" => {
            result.push_str("CONVERSATION HISTORY\n");
            result.push_str("====================\n\n");
            for msg in &messages {
                let role = msg["role"].as_str().unwrap_or("unknown");
                let content = msg["content"].as_str().unwrap_or("");
                result.push_str(&format!("[{}]: {}\n\n", role.to_uppercase(), content));
            }
        }
        _ => { // markdown
            result.push_str("# 📝 Conversation History\n\n");
            for msg in &messages {
                let role = msg["role"].as_str().unwrap_or("unknown");
                let content = msg["content"].as_str().unwrap_or("");
                match role {
                    "user" => result.push_str(&format!("**👤 User:** {}\n\n---\n\n", content)),
                    "assistant" => result.push_str(&format!("**🤖 Assistant:** {}\n\n---\n\n", content)),
                    "system" => result.push_str(&format!("**⚙️ System:** {}\n\n---\n\n", content.chars().take(200).collect::<String>())),
                    _ => result.push_str(&format!("**{}:** {}\n\n", role, content)),
                }
            }
        }
    }
    
    result.push_str("\n💡 Use this content with create_pdf to save the conversation as a PDF.");
    
    Ok(result)
}

// URL encoding module
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

// ==================== SELF-EVOLVING TOOLS ====================

/// Custom tool stored in localStorage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomTool {
    name: String,
    description: String,
    parameters_schema: serde_json::Value,
    code: String,
    created_at: String,
}

/// Create a new custom tool
async fn execute_create_tool(args: &serde_json::Value) -> Result<String, JsValue> {
    let name = args["name"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'name' parameter"))?;
    let description = args["description"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'description' parameter"))?;
    let parameters_schema = args["parameters_schema"].clone();
    let code = args["code"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'code' parameter"))?;
    
    // Validate tool name (lowercase, underscores, no spaces)
    if !name.chars().all(|c| c.is_lowercase() || c == '_' || c.is_numeric()) || name.contains(' ') {
        return Err(JsValue::from_str("Tool name must be lowercase with underscores only"));
    }
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    // Check if tool already exists
    let tools_key = "clawasm_custom_tools";
    let existing_tools: Vec<CustomTool> = storage.get_item(tools_key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    if existing_tools.iter().any(|t| t.name == name) {
        return Err(JsValue::from_str(&format!("Tool '{}' already exists. Use delete_tool first if you want to replace it.", name)));
    }
    
    // Create new tool
    let new_tool = CustomTool {
        name: name.to_string(),
        description: description.to_string(),
        parameters_schema,
        code: code.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    // Save to localStorage
    let mut tools = existing_tools;
    tools.push(new_tool);
    storage.set_item(tools_key, &serde_json::to_string(&tools).unwrap())?;
    
    Ok(format!(
        "✅ Tool '{}' created successfully!\n\nDescription: {}\n\nYou can now use this tool by calling it with the appropriate parameters.",
        name, description
    ))
}

/// List all custom tools
async fn execute_list_custom_tools(_args: &serde_json::Value) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let tools_key = "clawasm_custom_tools";
    let tools: Vec<CustomTool> = storage.get_item(tools_key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    if tools.is_empty() {
        return Ok("No custom tools created yet. Use create_tool to make one!".to_string());
    }
    
    let mut result = format!("Custom Tools ({}):\n\n", tools.len());
    for tool in tools {
        result.push_str(&format!("🔧 {} - {}\n", tool.name, tool.description));
        result.push_str(&format!("   Parameters: {}\n", serde_json::to_string(&tool.parameters_schema).unwrap_or_default()));
        result.push_str(&format!("   Created: {}\n\n", tool.created_at));
    }
    
    Ok(result)
}

/// Delete a custom tool
async fn execute_delete_tool(args: &serde_json::Value) -> Result<String, JsValue> {
    let name = args["name"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'name' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let tools_key = "clawasm_custom_tools";
    let mut tools: Vec<CustomTool> = storage.get_item(tools_key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    let initial_len = tools.len();
    tools.retain(|t| t.name != name);
    
    if tools.len() == initial_len {
        return Err(JsValue::from_str(&format!("Tool '{}' not found", name)));
    }
    
    storage.set_item(tools_key, &serde_json::to_string(&tools).unwrap())?;
    
    Ok(format!("✅ Tool '{}' deleted successfully!", name))
}

/// Execute a custom tool by running its JavaScript code
async fn execute_custom_tool(name: &str, args: &serde_json::Value) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    let tools_key = "clawasm_custom_tools";
    let tools: Vec<CustomTool> = storage.get_item(tools_key)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    
    let tool = tools.iter().find(|t| t.name == name)
        .ok_or_else(|| JsValue::from_str(&format!("Unknown tool: {}", name)))?;
    
    // Build JavaScript code with args injected
    let args_json = serde_json::to_string(args).unwrap_or_default();
    let js_code = format!(
        "(function() {{
            const args = {};
            {};
        }})()",
        args_json,
        tool.code
    );
    
    // Execute JavaScript
    let result = js_sys::eval(&js_code)
        .map_err(|e| JsValue::from_str(&format!("JavaScript error in tool '{}': {:?}", name, e)))?;
    
    let result_str = result.as_string().unwrap_or_else(|| format!("{:?}", result));
    
    Ok(result_str)
}

/// Deep research on a topic
async fn execute_research(args: &serde_json::Value) -> Result<String, JsValue> {
    let topic = args["topic"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'topic' parameter"))?;
    let depth = args["depth"].as_str().unwrap_or("normal");
    
    let max_searches = match depth {
        "quick" => 3,
        "deep" => 10,
        _ => 5,
    };
    
    let mut findings = Vec::new();
    
    // Step 1: Web search
    let search_args = serde_json::json!({"query": topic});
    let search_result = execute_web_search(&search_args).await?;
    findings.push(format!("## Web Search Results\n\n{}", search_result));
    
    // Step 2: Extract URLs and fetch content from top results
    // Simple URL extraction without regex
    let urls: Vec<String> = extract_urls(&search_result, max_searches);
    
    if !urls.is_empty() {
        findings.push("\n## Content from Sources\n".to_string());
        
        for url in urls.iter().take(max_searches) {
            let fetch_args = serde_json::json!({"url": url});
            if let Ok(content) = execute_fetch_url(&fetch_args).await {
                // Truncate to first 500 chars per source
                let truncated = if content.len() > 500 {
                    format!("{}...[truncated]", &content[..500])
                } else {
                    content
                };
                findings.push(format!("\n### {}\n{}\n", url, truncated));
            }
        }
    }
    
    // Step 3: Reddit search for discussions
    let reddit_args = serde_json::json!({"query": topic, "limit": 5});
    if let Ok(reddit_result) = execute_reddit_search(&reddit_args).await {
        findings.push(format!("\n## Reddit Discussions\n\n{}", reddit_result));
    }
    
    Ok(format!(
        "# Research Report: {}\n\nDepth: {}\n\n{}\n\n---\nResearch completed. Use this information to answer questions or create content about the topic.",
        topic,
        depth,
        findings.join("\n")
    ))
}

/// Simple URL extraction without regex
fn extract_urls(text: &str, max: usize) -> Vec<String> {
    let mut urls = Vec::new();
    let mut start = 0;
    
    while urls.len() < max {
        // Find https:// or http://
        let http_pos = text[start..].find("https://")
            .or_else(|| text[start..].find("http://"));
        
        if let Some(pos) = http_pos {
            let abs_pos = start + pos;
            let rest = &text[abs_pos..];
            
            // Find end of URL (space, newline, or closing paren)
            let end_chars = [' ', '\n', '\r', ')', ']', '}'];
            let end_pos = rest.find(|c| end_chars.contains(&c))
                .unwrap_or(rest.len().min(200));
            
            let url = rest[..end_pos].to_string();
            if url.len() > 10 {  // Minimum valid URL length
                urls.push(url);
            }
            start = abs_pos + end_pos;
        } else {
            break;
        }
    }
    
    urls
}

// ============================================
// Security & Vulnerability Scanner Functions
// ============================================

/// XSS Scanner - Tests for Cross-Site Scripting vulnerabilities
async fn execute_scan_xss(args: &serde_json::Value) -> Result<String, JsValue> {
    let url = args["url"].as_str();
    let html = args["html"].as_str();
    
    let mut findings: Vec<String> = Vec::new();
    let mut risk_level = "Low";
    
    // XSS payload patterns to check
    let xss_patterns = [
        ("<script>", "Script tag injection"),
        ("javascript:", "JavaScript protocol"),
        ("onerror=", "onerror event handler"),
        ("onload=", "onload event handler"),
        ("onclick=", "onclick event handler"),
        ("onmouseover=", "onmouseover event handler"),
        ("<img", "Image tag (potential injection)"),
        ("<svg", "SVG tag (potential injection)"),
        ("eval(", "eval() function"),
        ("document.cookie", "Cookie access"),
        ("document.write", "document.write"),
        ("innerHTML", "innerHTML assignment"),
        ("outerHTML", "outerHTML assignment"),
    ];
    
    let content = if let Some(html_content) = html {
        html_content.to_string()
    } else if let Some(target_url) = url {
        // Fetch URL content via proxy
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let body = serde_json::json!({
            "url": target_url,
            "method": "GET",
            "headers": {}
        });
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        
        let request_init = RequestInit::new();
        request_init.set_method("POST");
        request_init.set_headers(headers.as_ref());
        request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
        request_init.set_mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        JsFuture::from(response.text()?).await?.as_string().unwrap_or_default()
    } else {
        return Err(JsValue::from_str("Missing 'url' or 'html' parameter"));
    };
    
    // Scan for XSS patterns
    for (pattern, desc) in &xss_patterns {
        if content.to_lowercase().contains(pattern) {
            findings.push(format!("⚠️ Found: {} - {}", pattern, desc));
        }
    }
    
    // Check for input fields
    if content.contains("<input") || content.contains("<textarea") {
        findings.push("ℹ️ Input fields detected - check for proper sanitization".to_string());
    }
    
    // Check for form actions
    if content.contains("<form") {
        findings.push("ℹ️ Forms detected - verify CSRF protection".to_string());
    }
    
    if findings.len() > 3 {
        risk_level = "Medium";
    }
    if findings.len() > 6 {
        risk_level = "High";
    }
    
    let result = if findings.is_empty() {
        format!("✅ XSS Scan Results\n\nRisk Level: {}\n\nNo obvious XSS vulnerabilities detected.\n\nNote: This is a basic scan. For comprehensive testing, use specialized tools like OWASP ZAP.", risk_level)
    } else {
        format!("🔍 XSS Scan Results\n\nRisk Level: {}\n\nFindings:\n{}\n\nRecommendations:\n- Sanitize all user inputs\n- Use Content-Security-Policy headers\n- Implement output encoding\n- Consider using frameworks with built-in XSS protection", 
            risk_level, findings.join("\n"))
    };
    
    Ok(result)
}

/// SQL Injection Scanner
async fn execute_scan_sqli(args: &serde_json::Value) -> Result<String, JsValue> {
    let url = args["url"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'url' parameter"))?;
    let param = args["param"].as_str();
    
    let mut findings: Vec<String> = Vec::new();
    
    // SQL injection payloads to test
    let sqli_payloads = [
        ("'", "Single quote"),
        ("\"", "Double quote"),
        ("' OR '1'='1", "OR boolean injection"),
        ("' OR '1'='1' --", "OR with comment"),
        ("1' AND '1'='1", "AND boolean injection"),
        ("1; DROP TABLE", "Stacked query"),
        ("' UNION SELECT NULL--", "UNION injection"),
        ("1 OR 1=1", "Numeric OR"),
        ("-1' OR '1'='1", "Negative with OR"),
        ("admin'--", "Admin bypass"),
    ];
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    // Test each payload
    for (payload, desc) in &sqli_payloads {
        let test_url = if url.contains('?') {
            format!("{}{}{}", url, 
                if param.is_some() { "&" } else { "" },
                if let Some(p) = param { 
                    format!("{}={}", p, urlencoding::encode(payload))
                } else {
                    urlencoding::encode(payload)
                }
            )
        } else {
            url.to_string()
        };
        
        let body = serde_json::json!({
            "url": test_url,
            "method": "GET",
            "headers": {}
        });
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        
        let request_init = RequestInit::new();
        request_init.set_method("POST");
        request_init.set_headers(headers.as_ref());
        request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
        request_init.set_mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        let text = JsFuture::from(response.text()?).await?.as_string().unwrap_or_default();
        
        // Check for SQL error messages
        let sql_errors = [
            "SQL syntax",
            "mysql_fetch",
            "ORA-",
            "PLS-",
            "Unclosed quotation mark",
            "quoted string not properly terminated",
            "pg_query",
            "Warning: pg_",
            "PostgreSQL",
            "SQLite",
            "syntax error",
        ];
        
        for error in &sql_errors {
            if text.to_lowercase().contains(&error.to_lowercase()) {
                findings.push(format!("🔴 Potential SQLi: {} - Error: {}", desc, error));
                break;
            }
        }
    }
    
    let result = if findings.is_empty() {
        "✅ SQL Injection Scan Results\n\nRisk Level: Low\n\nNo SQL injection vulnerabilities detected with basic payloads.\n\nNote: This is a basic scan. For comprehensive testing, use sqlmap or similar tools.".to_string()
    } else {
        format!("🔴 SQL Injection Scan Results\n\nRisk Level: High\n\nFindings:\n{}\n\nRecommendations:\n- Use parameterized queries\n- Implement input validation\n- Use ORM libraries\n- Apply least privilege principle", findings.join("\n"))
    };
    
    Ok(result)
}

/// Security Headers Scanner
async fn execute_scan_headers(args: &serde_json::Value) -> Result<String, JsValue> {
    let url = args["url"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'url' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    let body = serde_json::json!({
        "url": url,
        "method": "HEAD",
        "headers": {}
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    let mut findings: Vec<String> = Vec::new();
    let mut score = 0;
    
    // Security headers to check
    let security_headers = [
        ("content-security-policy", "Content-Security-Policy (CSP)", 20),
        ("strict-transport-security", "Strict-Transport-Security (HSTS)", 15),
        ("x-frame-options", "X-Frame-Options", 10),
        ("x-content-type-options", "X-Content-Type-Options", 10),
        ("x-xss-protection", "X-XSS-Protection", 10),
        ("referrer-policy", "Referrer-Policy", 5),
        ("permissions-policy", "Permissions-Policy", 10),
        ("cross-origin-opener-policy", "Cross-Origin-Opener-Policy", 5),
        ("cross-origin-resource-policy", "Cross-Origin-Resource-Policy", 5),
    ];
    
    let response_headers = response.headers();
    
    for (header_name, display_name, points) in &security_headers {
        if response_headers.has(header_name).unwrap_or(false) {
            findings.push(format!("✅ {}: Present", display_name));
            score += points;
        } else {
            findings.push(format!("❌ {}: Missing", display_name));
        }
    }
    
    // Check for insecure headers
    if response_headers.has("server").unwrap_or(false) {
        findings.push("⚠️ Server header exposed - Consider removing or obscuring".to_string());
    }
    if response_headers.has("x-powered-by").unwrap_or(false) {
        findings.push("⚠️ X-Powered-By header exposed - Remove this header".to_string());
    }
    
    let grade = if score >= 80 { "A" } else if score >= 60 { "B" } else if score >= 40 { "C" } else if score >= 20 { "D" } else { "F" };
    
    Ok(format!("🔒 Security Headers Scan Results\n\nURL: {}\n\nSecurity Score: {}/100 (Grade: {})\n\nHeaders Analysis:\n{}\n\nRecommendations:\n- Implement CSP to prevent XSS\n- Enable HSTS for HTTPS enforcement\n- Set X-Frame-Options to prevent clickjacking\n- Remove server version disclosure", 
        url, score, grade, findings.join("\n")))
}

/// SSL/TLS Scanner
async fn execute_scan_ssl(args: &serde_json::Value) -> Result<String, JsValue> {
    let domain = args["domain"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'domain' parameter"))?;
    
    // Note: Full SSL scanning requires server-side implementation
    // This provides basic checks via proxy
    
    let url = format!("https://{}", domain);
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    let body = serde_json::json!({
        "url": url,
        "method": "GET",
        "headers": {}
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    let mut findings: Vec<String> = Vec::new();
    
    // Check HSTS header
    let response_headers = response.headers();
    if response_headers.has("strict-transport-security").unwrap_or(false) {
        findings.push("✅ HSTS: Enabled".to_string());
    } else {
        findings.push("❌ HSTS: Not enabled".to_string());
    }
    
    findings.push("\n📋 SSL/TLS Configuration Notes:".to_string());
    findings.push("- HTTPS connection successful".to_string());
    findings.push("- For detailed SSL analysis, use:".to_string());
    findings.push("  • sslscan command-line tool".to_string());
    findings.push("  • SSL Labs (ssllabs.com/ssltest)".to_string());
    findings.push("  • testssl.sh script".to_string());
    
    Ok(format!("🔐 SSL/TLS Scan Results\n\nDomain: {}\n\n{}\n\nNote: Browser-based SSL scanning is limited. For comprehensive certificate validation, protocol support, and cipher analysis, use server-side tools.", 
        domain, findings.join("\n")))
}

/// Dependency Vulnerability Scanner
async fn execute_scan_deps(args: &serde_json::Value) -> Result<String, JsValue> {
    let package = args["package"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'package' parameter"))?;
    let version = args["version"].as_str();
    let ecosystem = args["ecosystem"].as_str().unwrap_or("npm");
    
    // Query OSV (Google's Open Source Vulnerabilities) database
    let osv_url = format!(
        "https://api.osv.dev/v1/query",
    );
    
    let query_body = serde_json::json!({
        "package": {
            "name": package,
            "ecosystem": ecosystem
        },
        "version": version
    });
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    let body = serde_json::json!({
        "url": osv_url,
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        },
        "body": serde_json::to_string(&query_body).unwrap()
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    let text = JsFuture::from(response.text()?).await?.as_string().unwrap_or_default();
    
    // Parse OSV response
    let mut vulnerabilities: Vec<String> = Vec::new();
    
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(vulns) = parsed.get("vulns").and_then(|v| v.as_array()) {
            for vuln in vulns {
                let id = vuln.get("id").and_then(|i| i.as_str()).unwrap_or("Unknown");
                let summary = vuln.get("summary").and_then(|s| s.as_str()).unwrap_or("No description");
                let severity = vuln.get("severity")
                    .and_then(|s| s.as_array())
                    .and_then(|a| a.first())
                    .and_then(|s| s.get("score"))
                    .and_then(|s| s.as_f64())
                    .map(|s| format!("CVSS: {:.1}", s))
                    .unwrap_or_else(|| "Severity: Unknown".to_string());
                
                vulnerabilities.push(format!("🔴 {} - {} [{}]", id, summary, severity));
            }
        }
    }
    
    let result = if vulnerabilities.is_empty() {
        format!("✅ Dependency Scan Results\n\nPackage: {} ({})\nVersion: {}\n\nNo known vulnerabilities found.\n\nNote: Always keep dependencies updated and check regularly for security advisories.", 
            package, ecosystem, version.unwrap_or("latest"))
    } else {
        format!("🔴 Dependency Scan Results\n\nPackage: {} ({})\nVersion: {}\n\nVulnerabilities Found:\n{}\n\nRecommendations:\n- Update to latest version\n- Review security advisories\n- Consider alternative packages\n- Use npm audit / pip audit / cargo audit", 
            package, ecosystem, version.unwrap_or("latest"), vulnerabilities.join("\n"))
    };
    
    Ok(result)
}

/// Secret Scanner - Detects exposed secrets in code
async fn execute_scan_secrets(args: &serde_json::Value) -> Result<String, JsValue> {
    let code = args["code"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'code' parameter"))?;
    
    let mut findings: Vec<String> = Vec::new();
    
    // Secret patterns to detect
    let secret_patterns = [
        // AWS
        ("AKIA[0-9A-Z]{16}", "AWS Access Key ID"),
        ("aws(.{0,20})?['\"][0-9a-zA-Z/+=]{40}['\"]", "AWS Secret Access Key"),
        // GitHub
        ("ghp_[0-9a-zA-Z]{36}", "GitHub Personal Access Token"),
        ("gho_[0-9a-zA-Z]{36}", "GitHub OAuth Token"),
        ("ghu_[0-9a-zA-Z]{36}", "GitHub User Token"),
        ("ghs_[0-9a-zA-Z]{36}", "GitHub Server Token"),
        ("github_pat_[0-9a-zA-Z_]{22,}", "GitHub Fine-grained Token"),
        // Generic
        ("[0-9a-f]{32}", "Possible API Key (32 hex)"),
        ("[0-9a-f]{64}", "Possible API Key (64 hex)"),
        // JWT
        ("eyJ[a-zA-Z0-9_-]*\\.eyJ[a-zA-Z0-9_-]*\\.[a-zA-Z0-9_-]*", "JWT Token"),
        // Private Keys
        ("-----BEGIN (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----", "Private Key"),
        // Database URLs
        ("(mysql|postgres|mongodb)://[^\\s]+:[^\\s]+@", "Database URL with credentials"),
        // API Keys
        ("api[_-]?key['\"]?\\s*[:=]\\s*['\"][^'\"]+['\"]", "API Key assignment"),
        ("secret[_-]?key['\"]?\\s*[:=]\\s*['\"][^'\"]+['\"]", "Secret Key assignment"),
        ("password['\"]?\\s*[:=]\\s*['\"][^'\"]+['\"]", "Password assignment"),
        // Slack
        ("xox[baprs]-[0-9]{10,12}-[0-9]{10,12}-[0-9a-zA-Z]{24}", "Slack Token"),
        // Stripe
        ("sk_live_[0-9a-zA-Z]{24}", "Stripe Live Secret Key"),
        ("rk_live_[0-9a-zA-Z]{24}", "Stripe Live Restricted Key"),
        // Google
        ("AIza[0-9A-Za-z\\-_]{35}", "Google API Key"),
        // Generic tokens
        ("[a-zA-Z0-9_-]{32,45}", "Possible Token/Key"),
    ];
    
    for (pattern, desc) in &secret_patterns {
        // Simple string matching (regex would be better but limited in WASM)
        if code.contains(&pattern.split_whitespace().next().unwrap_or("")) {
            // Additional check for common patterns
            if code.contains("key") || code.contains("token") || code.contains("secret") || code.contains("password") {
                findings.push(format!("🔴 Potential {} detected", desc));
            }
        }
    }
    
    // Check for common dangerous patterns
    if code.contains("password =") || code.contains("password=") {
        findings.push("🔴 Hardcoded password detected".to_string());
    }
    if code.contains("apiKey =") || code.contains("apiKey=") {
        findings.push("🔴 Hardcoded API key detected".to_string());
    }
    if code.contains("-----BEGIN") {
        findings.push("🔴 Private key detected".to_string());
    }
    
    let result = if findings.is_empty() {
        "✅ Secret Scan Results\n\nNo obvious secrets detected in the provided code.\n\nNote: This is a pattern-based scan. Always review code manually and use tools like git-secrets, truffleHog, or gitleaks for comprehensive scanning.".to_string()
    } else {
        format!("🔴 Secret Scan Results\n\n⚠️ POTENTIAL SECRETS DETECTED!\n\n{}\n\n⚠️ IMMEDIATE ACTIONS:\n1. Rotate any exposed credentials\n2. Remove secrets from code\n3. Use environment variables or secret managers\n4. Add secrets to .gitignore\n5. Review git history for accidental commits", findings.join("\n"))
    };
    
    Ok(result)
}

/// CORS Scanner
async fn execute_scan_cors(args: &serde_json::Value) -> Result<String, JsValue> {
    let url = args["url"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'url' parameter"))?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    
    let mut findings: Vec<String> = Vec::new();
    
    // Test different origins
    let test_origins = [
        "https://evil.com",
        "https://attacker.com",
        "null",
    ];
    
    for origin in &test_origins {
        let body = serde_json::json!({
            "url": url,
            "method": "GET",
            "headers": {
                "Origin": origin
            }
        });
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        
        let request_init = RequestInit::new();
        request_init.set_method("POST");
        request_init.set_headers(headers.as_ref());
        request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
        request_init.set_mode(RequestMode::Cors);
        
        let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        let response_headers = response.headers();
        
        // Check CORS headers
        if let Some(acao) = response_headers.get("Access-Control-Allow-Origin").ok().flatten() {
            if acao == "*" {
                findings.push(format!("🔴 CORS allows any origin (*) from test origin: {}", origin));
            } else if acao == *origin || acao == "null" {
                findings.push(format!("🔴 CORS reflects origin: {} -> {}", origin, acao));
            } else {
                findings.push(format!("✅ CORS restricted to: {}", acao));
            }
        }
        
        // Check credentials
        if response_headers.has("Access-Control-Allow-Credentials").unwrap_or(false) {
            findings.push("⚠️ CORS allows credentials - ensure origin is properly restricted".to_string());
        }
    }
    
    let result = if findings.is_empty() {
        format!("✅ CORS Scan Results\n\nURL: {}\n\nNo CORS misconfigurations detected.\n\nNote: CORS is configured by the server. Ensure:\n- Origin is properly validated\n- Credentials are only allowed with specific origins\n- Wildcard (*) is not used with credentials", url)
    } else {
        format!("🔴 CORS Scan Results\n\nURL: {}\n\nFindings:\n{}\n\nRecommendations:\n- Whitelist specific origins instead of using *\n- Validate Origin header against allowed list\n- Don't use Access-Control-Allow-Credentials with *\n- Consider CSRF protection alongside CORS", url, findings.join("\n"))
    };
    
    Ok(result)
}

// ============================================
// Audio & Media Tools
// ============================================

/// Text-to-Speech with downloadable audio file (persisted for later access)
async fn execute_text_to_speech(args: &serde_json::Value) -> Result<String, JsValue> {
    let text = args["text"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'text' parameter"))?;
    let lang = args["lang"].as_str().unwrap_or("en");
    let filename = args["filename"].as_str().unwrap_or("speech");
    
    // Truncate text if too long
    let text_to_use = if text.len() > 200 { &text[..200] } else { text };
    
    // Use Google Translate TTS API via proxy
    let encoded_text = urlencoding::encode(text_to_use);
    let tts_url = format!(
        "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
        encoded_text, lang
    );
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
    
    // Generate unique file ID
    let file_id = format!("audio_{}", chrono::Utc::now().timestamp_millis());
    
    let body = serde_json::json!({
        "url": tts_url,
        "method": "GET",
        "headers": {}
    });
    
    let headers = Headers::new()?;
    headers.set("Content-Type", "application/json")?;
    
    let request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(headers.as_ref());
    request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
    request_init.set_mode(RequestMode::Cors);
    
    let request = Request::new_with_str_and_init("http://localhost:3000/proxy", &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;
    
    let blob = JsFuture::from(response.blob()?).await?;
    let blob: Blob = blob.dyn_into()?;
    
    // Convert blob to base64 for storage
    let array_buffer = JsFuture::from(blob.array_buffer()).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    
    // Convert to base64 using JavaScript
    let js_array = js_sys::Array::new();
    for i in 0..uint8_array.length() {
        js_array.push(&js_sys::Number::from(uint8_array.get_index(i)));
    }
    
    let base64 = js_sys::eval("btoa(String.fromCharCode.apply(null, arguments))")
        .map_err(|e| JsValue::from_str(&format!("Base64 eval error: {:?}", e)))?
        .dyn_into::<js_sys::Function>()
        .map_err(|e| JsValue::from_str(&format!("Base64 cast error: {:?}", e)))?
        .apply(&JsValue::NULL, &js_array)
        .map_err(|e| JsValue::from_str(&format!("Base64 apply error: {:?}", e)))?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to convert to base64"))?;
    
    // Store audio metadata
    let audio_file = AudioFile {
        id: file_id.clone(),
        text: text_to_use.to_string(),
        lang: lang.to_string(),
        filename: format!("{}.mp3", filename),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let audio_json = serde_json::to_string(&audio_file)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
    storage.set_item(&file_id, &audio_json)?;
    
    // Store base64 audio data
    storage.set_item(&format!("{}_data", file_id), &base64)?;
    
    // Update file index
    let mut file_index: Vec<String> = storage.get_item("clawasm_files")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    file_index.push(file_id.clone());
    storage.set_item("clawasm_files", &serde_json::to_string(&file_index).unwrap())?;
    
    // Create blob URL for immediate download
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;
    
    let js_code = format!(r#"
        (function() {{
            const a = document.createElement('a');
            a.href = '{}';
            a.download = '{}.mp3';
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            return 'Audio downloaded: {}.mp3';
        }})()
    "#, url, filename, filename);
    
    let result = js_sys::eval(&js_code)?.as_string().unwrap_or_else(|| "Audio created".to_string());
    
    Ok(format!("🔊 TTS completed!\n\nText: \"{}\"\nLang: {}\nFile ID: {}\n\n{}\n\n💾 Audio saved! Use download_file with file_id '{}' to download later.", 
        text_to_use, lang, file_id, result, file_id))
}

/// Speak text aloud using browser speech synthesis
async fn execute_speak(args: &serde_json::Value) -> Result<String, JsValue> {
    let text = args["text"].as_str()
        .ok_or_else(|| JsValue::from_str("Missing 'text' parameter"))?;
    let lang = args["lang"].as_str().unwrap_or("en-US");
    let rate = args["rate"].as_f64().unwrap_or(1.0);
    
    let js_code = format!(r#"
        (function() {{
            if (!('speechSynthesis' in window)) {{
                return 'TTS not supported';
            }}
            const u = new SpeechSynthesisUtterance("{}");
            u.lang = "{}";
            u.rate = {};
            speechSynthesis.speak(u);
            return 'Speaking: "{}"';
        }})()
    "#, text.replace("\"", "\\\""), lang, rate, text.replace("\"", "\\\""));
    
    let result = js_sys::eval(&js_code)?.as_string().unwrap_or_else(|| "Speaking".to_string());
    
    Ok(result)
}
