//! Memory System for WebClaw - Vector search + embedding
//! 
//! Inspired by ZeroClaw's memory system with hybrid search capabilities.

use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};
use wasm_bindgen::JsCast;
use js_sys::{Array, Object, Reflect};

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
    pub accessed_at: i64,
    pub access_count: u32,
}

/// Memory search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub entry: MemoryEntry,
    pub score: f32,
}

/// Memory backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryBackend {
    #[serde(rename = "indexeddb")]
    IndexedDB,
    #[serde(rename = "none")]
    None,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub backend: MemoryBackend,
    pub auto_save: bool,
    pub embedding_provider: EmbeddingProvider,
    pub vector_weight: f32,
    pub keyword_weight: f32,
    pub max_entries: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            backend: MemoryBackend::IndexedDB,
            auto_save: true,
            embedding_provider: EmbeddingProvider::OpenAI,
            vector_weight: 0.7,
            keyword_weight: 0.3,
            max_entries: 1000,
        }
    }
}

/// Embedding provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmbeddingProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "local")]
    Local,
    #[serde(rename = "none")]
    None,
}

/// Memory system
pub struct MemorySystem {
    config: MemoryConfig,
    entries: Vec<MemoryEntry>,
    api_key: Option<String>,
}

impl MemorySystem {
    /// Create a new memory system
    pub fn new(config: MemoryConfig) -> Self {
        MemorySystem {
            config,
            entries: Vec::new(),
            api_key: None,
        }
    }

    /// Set API key for embedding provider
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    /// Save a memory entry
    pub async fn save(&mut self, content: &str, metadata: serde_json::Value) -> Result<String, JsValue> {
        let id = generate_id();
        let now = chrono::Utc::now().timestamp();
        
        // Get embedding
        let embedding = if self.config.embedding_provider != EmbeddingProvider::None {
            self.get_embedding(content).await.ok()
        } else {
            None
        };
        
        let entry = MemoryEntry {
            id: id.clone(),
            content: content.to_string(),
            embedding,
            metadata,
            created_at: now,
            accessed_at: now,
            access_count: 0,
        };
        
        // Check max entries
        if self.entries.len() >= self.config.max_entries {
            // Remove oldest accessed entry
            self.entries.sort_by_key(|e| e.accessed_at);
            self.entries.remove(0);
        }
        
        self.entries.push(entry.clone());
        
        // Persist to IndexedDB
        if self.config.backend == MemoryBackend::IndexedDB {
            self.persist_to_indexeddb(&entry).await?;
        }
        
        Ok(id)
    }

    /// Recall memories by search query
    pub async fn recall(&mut self, query: &str, limit: usize) -> Result<Vec<MemorySearchResult>, JsValue> {
        if self.entries.is_empty() {
            // Load from IndexedDB
            self.load_from_indexeddb().await?;
        }
        
        let query_embedding = if self.config.embedding_provider != EmbeddingProvider::None {
            self.get_embedding(query).await.ok()
        } else {
            None
        };
        
        let query_keywords = extract_keywords(query);
        
        let mut results: Vec<MemorySearchResult> = self.entries.iter()
            .map(|entry| {
                let mut score = 0.0;
                
                // Vector similarity
                if let (Some(q_emb), Some(e_emb)) = (&query_embedding, &entry.embedding) {
                    let vector_score = cosine_similarity(q_emb, e_emb);
                    score += vector_score * self.config.vector_weight;
                }
                
                // Keyword matching
                let entry_keywords = extract_keywords(&entry.content);
                let keyword_score = jaccard_similarity(&query_keywords, &entry_keywords);
                score += keyword_score * self.config.keyword_weight;
                
                // Boost by access count
                score *= 1.0 + (entry.access_count as f32 * 0.01);
                
                MemorySearchResult {
                    entry: entry.clone(),
                    score,
                }
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        // Update access stats
        for result in results.iter().take(limit) {
            if let Some(entry) = self.entries.iter_mut().find(|e| e.id == result.entry.id) {
                entry.accessed_at = chrono::Utc::now().timestamp();
                entry.access_count += 1;
            }
        }
        
        Ok(results.into_iter().take(limit).collect())
    }

    /// Get embedding from provider
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, JsValue> {
        match self.config.embedding_provider {
            EmbeddingProvider::OpenAI => self.get_openai_embedding(text).await,
            EmbeddingProvider::Local => self.get_local_embedding(text),
            EmbeddingProvider::None => Err(JsValue::from_str("No embedding provider configured")),
        }
    }

    /// Get embedding from OpenAI
    async fn get_openai_embedding(&self, text: &str) -> Result<Vec<f32>, JsValue> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| JsValue::from_str("API key not set for embeddings"))?;
        
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        let headers = Headers::new()?;
        headers.set("Content-Type", "application/json")?;
        headers.set("Authorization", &format!("Bearer {}", api_key))?;
        
        let body = serde_json::json!({
            "input": text,
            "model": "text-embedding-3-small",
        });
        
        let request_init = RequestInit::new();
        request_init.set_method("POST");
        request_init.set_headers(headers.as_ref());
        request_init.set_body(&JsValue::from_str(&serde_json::to_string(&body).unwrap()));
        
        let request = Request::new_with_str_and_init(
            "https://api.openai.com/v1/embeddings",
            &request_init,
        )?;
        
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;
        
        if !response.ok() {
            return Err(JsValue::from_str(&format!("Embedding API error: {}", response.status())));
        }
        
        let json = JsFuture::from(response.json()?).await?;
        let result: EmbeddingResponse = serde_wasm_bindgen::from_value(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        Ok(result.data[0].embedding.clone())
    }

    /// Get local embedding (simple hash-based, not real embeddings)
    fn get_local_embedding(&self, text: &str) -> Result<Vec<f32>, JsValue> {
        // Simple TF-IDF style local embedding (384 dimensions)
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let mut embedding = vec![0.0f32; 384];
        
        for (i, word) in words.iter().enumerate() {
            let hash = hash_word(word);
            let idx = hash % 384;
            embedding[idx] += 1.0 / (1.0 + i as f32); // Position weighting
        }
        
        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for e in embedding.iter_mut() {
                *e /= norm;
            }
        }
        
        Ok(embedding)
    }

    /// Persist entry to IndexedDB
    async fn persist_to_indexeddb(&self, entry: &MemoryEntry) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        
        // Use localStorage as fallback (IndexedDB requires more complex setup)
        let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
        
        let key = format!("memory_{}", entry.id);
        let value = serde_json::to_string(entry)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;
        
        storage.set_item(&key, &value)?;
        
        // Store index
        let mut ids: Vec<String> = storage.get_item("memory_index")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        if !ids.contains(&entry.id) {
            ids.push(entry.id.clone());
            storage.set_item("memory_index", &serde_json::to_string(&ids).unwrap())?;
        }
        
        Ok(())
    }

    /// Load entries from IndexedDB
    async fn load_from_indexeddb(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
        
        let ids: Vec<String> = storage.get_item("memory_index")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        self.entries.clear();
        
        for id in ids {
            let key = format!("memory_{}", id);
            if let Some(json) = storage.get_item(&key).ok().flatten() {
                if let Ok(entry) = serde_json::from_str::<MemoryEntry>(&json) {
                    self.entries.push(entry);
                }
            }
        }
        
        Ok(())
    }

    /// Delete a memory entry
    pub async fn delete(&mut self, id: &str) -> Result<bool, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
        
        // Remove from entries
        self.entries.retain(|e| e.id != id);
        
        // Remove from storage
        let key = format!("memory_{}", id);
        storage.remove_item(&key)?;
        
        // Update index
        let mut ids: Vec<String> = storage.get_item("memory_index")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        ids.retain(|i| i != id);
        storage.set_item("memory_index", &serde_json::to_string(&ids).unwrap())?;
        
        Ok(true)
    }

    /// Clear all memories
    pub async fn clear(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let storage = window.local_storage()?.ok_or_else(|| JsValue::from_str("No localStorage"))?;
        
        // Get all memory IDs
        let ids: Vec<String> = storage.get_item("memory_index")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        // Remove all memory entries
        for id in &ids {
            let key = format!("memory_{}", id);
            storage.remove_item(&key)?;
        }
        
        // Clear index
        storage.remove_item("memory_index")?;
        
        // Clear in-memory entries
        self.entries.clear();
        
        Ok(())
    }

    /// Get all memories
    pub fn get_all(&self) -> &[MemoryEntry] {
        &self.entries
    }
}

// Response types
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

// Helper functions

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("mem_{}", timestamp)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

fn jaccard_similarity(a: &[String], b: &[String]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    
    let set_a: std::collections::HashSet<_> = a.iter().cloned().collect();
    let set_b: std::collections::HashSet<_> = b.iter().cloned().collect();
    
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    
    if union > 0 {
        intersection as f32 / union as f32
    } else {
        0.0
    }
}

fn extract_keywords(text: &str) -> Vec<String> {
    // Simple keyword extraction
    let stop_words = ["the", "a", "an", "is", "are", "was", "were", "be", "been", "being", 
                      "have", "has", "had", "do", "does", "did", "will", "would", "could",
                      "should", "may", "might", "must", "shall", "can", "need", "dare", "ought",
                      "used", "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
                      "into", "through", "during", "before", "after", "above", "below", "between",
                      "and", "but", "or", "nor", "so", "yet", "both", "either", "neither",
                      "not", "only", "own", "same", "than", "too", "very", "just"];
    
    text.to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 2 && !stop_words.contains(word))
        .map(|word| word.chars().filter(|c| c.is_alphanumeric()).collect())
        .filter(|word: &String| !word.is_empty())
        .collect()
}

fn hash_word(word: &str) -> usize {
    // Simple hash function
    let mut hash: usize = 0;
    for c in word.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as usize);
    }
    hash
}
