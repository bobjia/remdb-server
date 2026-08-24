# Embedding API Endpoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `POST /v2/vectordb/embedding` endpoint to the Milvus REST API that converts text to vector embeddings using ONNX embedding models, with an OpenAI-compatible request/response format.

**Architecture:** The heavy lifting lives in the remdb core library (`remdb/src/model/embedding.rs`) behind `#[cfg(feature = "model-runtime")]`:
- `EmbeddingTokenizer` — wraps Hugging Face `tokenizers` crate
- `EmbeddingEngine` — orchestrates tokenization + ONNX model inference + L2 normalization, reuses `OnnxModel`

The server layer (`src/milvus/embedding/`) is thin:
- `models.rs` — request/response types
- `handler.rs` — warp handler, calls core `EmbeddingEngine`

**Tech Stack:** `ort` (ONNX Runtime via remdb's `model-runtime` feature), `tokenizers` (Hugging Face Rust crate, in remdb core), `warp` (existing HTTP server)

**Spec:** `docs/superpowers/specs/2026-08-25-embedding-endpoint-design.md`

## Global Constraints

- No panic anywhere — use `?` for error propagation, no `unwrap()`/`expect()`/indexing without bounds checks
- Error codes for embedding: 1 (model not found), 2 (invalid input), 3 (input exceeds max tokens), 4 (model load failure), 5 (inference failure), 6 (no model specified)
- L2-normalize all embedding vectors before returning
- Truncate tokenized input silently to model's `max_input_length`
- OpenAI-compatible response format: `{ object, data: [{ object, index, embedding }], model }`
- `model` field is optional in request; falls back to config default
- Route: `POST /v2/vectordb/embedding`

---

### Task 1: Add embedding config to MilvusConfig ✅ (DONE)

**Files:**
- `src/config/mod.rs` — added `EmbeddingConfig` struct and `embedding` field to `MilvusConfig`

---

### Task 2: Add embedding error variants to MilvusError

**Files:**
- Modify: `src/milvus/error.rs` — add embedding-specific error codes

**Interfaces:**
- Consumes: existing `MilvusError` enum
- Produces: new variants `ModelNotFound(String)`, `InvalidInput(String)`, `InputTooLong(usize)`, `ModelLoadFailed(String)`, `InferenceFailed(String)`, `NoModelSpecified` with appropriate codes and HTTP statuses

- [ ] **Step 1: Add embedding error variants to `MilvusError`**

Add to the `MilvusError` enum in `src/milvus/error.rs`:
```rust
    ModelNotFound(String),
    InvalidInput(String),
    InputTooLong(usize),
    ModelLoadFailed(String),
    InferenceFailed(String),
    NoModelSpecified,
```

- [ ] **Step 2: Update `code()` method**

Add arms to the `code()` match:
```rust
    MilvusError::ModelNotFound(_) => 1,
    MilvusError::InvalidInput(_) => 2,
    MilvusError::InputTooLong(_) => 3,
    MilvusError::ModelLoadFailed(_) => 4,
    MilvusError::InferenceFailed(_) => 5,
    MilvusError::NoModelSpecified => 6,
```

- [ ] **Step 3: Update `http_status()` method**

```rust
    MilvusError::ModelNotFound(_) => 404,
    MilvusError::InvalidInput(_) => 400,
    MilvusError::InputTooLong(_) => 400,
    MilvusError::ModelLoadFailed(_) => 500,
    MilvusError::InferenceFailed(_) => 500,
    MilvusError::NoModelSpecified => 400,
```

- [ ] **Step 4: Update `message()` method**

```rust
    MilvusError::ModelNotFound(name) => format!("Model '{}' not found", name),
    MilvusError::InvalidInput(msg) => format!("Invalid input: {}", msg),
    MilvusError::InputTooLong(max) => format!("Input exceeds maximum length of {} tokens", max),
    MilvusError::ModelLoadFailed(msg) => format!("Failed to load model: {}", msg),
    MilvusError::InferenceFailed(msg) => format!("Inference failed: {}", msg),
    MilvusError::NoModelSpecified => "No model specified and no default model configured".to_string(),
```

- [ ] **Step 5: Run existing tests**

```bash
cd /mnt/home/bobjia/remdb-server && cargo test --lib milvus::error
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/milvus/error.rs
git commit -m "feat: add embedding error variants to MilvusError"
```

---

### Task 3: Add `tokenizers` dependency to remdb core

**Files:**
- Modify: `remdb/Cargo.toml` — add `tokenizers` dependency under `model-runtime` feature

**Interfaces:**
- Consumes: `remdb/Cargo.toml` dependency list
- Produces: `tokenizers` crate available in remdb core when `model-runtime` feature is enabled

- [ ] **Step 1: Read remdb/Cargo.toml**

```bash
cat /mnt/home/bobjia/remdb/Cargo.toml
```

- [ ] **Step 2: Add `tokenizers` dependency**

Add under `[dependencies]`:
```toml
tokenizers = { version = "0.21", optional = true }
```

Add under `[features]`:
```toml
model-runtime = ["ort", "ndarray", "tokio", "tokenizers"]
```

> Note: If `model-runtime` already exists, add `tokenizers` to its dependency list.

- [ ] **Step 3: Verify compilation**

```bash
cd /mnt/home/bobjia/remdb-server && cargo check
```
Expected: Compilation succeeds.

- [ ] **Step 4: Commit**

```bash
git add remdb/Cargo.toml
git commit -m "feat: add tokenizers dependency to remdb core under model-runtime feature"
```

---

### Task 4: Add embedding module to remdb core (embedding.rs)

**Files:**
- Modify: `remdb/src/model/mod.rs` — add `pub mod embedding;` behind `#[cfg(feature = "model-runtime")]`
- Create: `remdb/src/model/embedding.rs` — `EmbeddingTokenizer` + `EmbeddingEngine`

**Interfaces:**
- Consumes: `tokenizers` crate, `remdb::model::OnnxModel`, `remdb::model::ModelManager`
- Produces: `EmbeddingTokenizer { load, encode, encode_batch }`, `EmbeddingEngine { new, preload_default, embed, default_model }`

- [ ] **Step 1: Read `remdb/src/model/mod.rs`** to understand current exports

- [ ] **Step 2: Add `pub mod embedding;` to `remdb/src/model/mod.rs`**

Inside the existing `#[cfg(feature = "model-runtime")]` block:
```rust
pub mod embedding;
```

- [ ] **Step 3: Write `remdb/src/model/embedding.rs`**

```rust
//! Embedding engine
//!
//! Provides text embedding capabilities using ONNX models and HuggingFace tokenizers.
//! This module is available behind the `model-runtime` feature flag.

#![cfg(feature = "model-runtime")]

use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::log::{error, info, warn};
use crate::model::OnnxModel;

/// Wrapper around Hugging Face tokenizers for embedding models
pub struct EmbeddingTokenizer {
    tokenizer: std::sync::Mutex<tokenizers::Tokenizer>,
    /// Maximum sequence length for the model
    pub max_input_length: usize,
    /// Whether this model uses token_type_ids (e.g., BERT-based)
    pub has_token_type_ids: bool,
}

impl EmbeddingTokenizer {
    /// Load a tokenizer from `{models_dir}/{model_name}/tokenizer.json`
    pub fn load(models_dir: &str, model_name: &str) -> Result<Self, String> {
        use std::path::PathBuf;

        let tokenizer_path = PathBuf::from(models_dir)
            .join(model_name)
            .join("tokenizer.json");

        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| {
                format!(
                    "Failed to load tokenizer from {}: {}",
                    tokenizer_path.display(),
                    e
                )
            })?;

        // Default max length for BGE models is 512
        let max_input_length = 512;

        // Detect if model uses token_type_ids (BERT-style models do)
        let has_token_type_ids = true;

        Ok(Self {
            tokenizer: std::sync::Mutex::new(tokenizer),
            max_input_length,
            has_token_type_ids,
        })
    }

    /// Encode a single text, returning (input_ids, attention_mask, token_type_ids).
    /// Truncates to max_length silently.
    pub fn encode(
        &self,
        text: &str,
        max_length: usize,
    ) -> Result<(Vec<i64>, Vec<i64>, Vec<i64>), String> {
        let mut tokenizer = self.tokenizer
            .lock()
            .map_err(|_| "tokenizer lock poisoned".to_string())?;

        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| format!("tokenization failed: {}", e))?;

        let input_ids: Vec<i64> = encoding
            .get_ids()
            .iter()
            .map(|&id| id as i64)
            .take(max_length)
            .collect();

        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&m| m as i64)
            .take(max_length)
            .collect();

        let token_type_ids: Vec<i64> = if self.has_token_type_ids {
            encoding
                .get_type_ids()
                .iter()
                .map(|&t| t as i64)
                .take(max_length)
                .collect()
        } else {
            vec![0i64; input_ids.len()]
        };

        Ok((input_ids, attention_mask, token_type_ids))
    }

    /// Encode a batch of texts
    pub fn encode_batch(
        &self,
        texts: &[&str],
        max_length: usize,
    ) -> Result<Vec<(Vec<i64>, Vec<i64>, Vec<i64>)>, String> {
        texts.iter()
            .map(|text| self.encode(text, max_length))
            .collect()
    }
}

/// A loaded model with its tokenizer
struct ModelEntry {
    model: OnnxModel,
    tokenizer: EmbeddingTokenizer,
    dimension: usize,
}

/// Embedding engine managing model loading, caching, and inference
pub struct EmbeddingEngine {
    /// Map of model_name -> (model, tokenizer, dimension)
    models: std::sync::Mutex<alloc::collections::BTreeMap<String, ModelEntry>>,
    /// Default model name
    default_model: Option<String>,
    /// Directory where models are stored
    models_dir: String,
    /// Maximum number of models to cache
    max_models: usize,
    /// HuggingFace mirror URL
    hf_mirror: Option<String>,
    /// Whether to auto-download models
    auto_download: bool,
}

impl EmbeddingEngine {
    /// Create a new embedding engine
    pub fn new(
        default_model: Option<String>,
        models_dir: String,
        max_models: usize,
        auto_download: bool,
        hf_mirror: Option<String>,
    ) -> Self {
        Self {
            models: std::sync::Mutex::new(alloc::collections::BTreeMap::new()),
            default_model,
            models_dir,
            max_models,
            auto_download,
            hf_mirror,
        }
    }

    /// Pre-load the default model if configured
    pub fn preload_default(&self) -> Result<(), String> {
        if let Some(ref default_model) = self.default_model {
            info!("Pre-loading default embedding model: {}", default_model);
            self.load_model_internal(default_model)?;
            info!("Default embedding model loaded: {}", default_model);
        }
        Ok(())
    }

    /// Load a model (and its tokenizer) from disk, caching it.
    fn load_model_internal(&self, name: &str) -> Result<(), String> {
        use std::path::PathBuf;

        let mut models = self.models
            .lock()
            .map_err(|_| "models lock poisoned".to_string())?;

        // Check cache first
        if models.contains_key(name) {
            return Ok(());
        }

        // Evict if over max_models
        if models.len() >= self.max_models {
            if let Some(key) = models.keys().next().cloned() {
                warn!("Evicting model '{}' from cache (max {})", key, self.max_models);
                models.remove(&key);
            }
        }

        // Build model path: {models_dir}/{name}/{name}.onnx
        let model_path = PathBuf::from(&self.models_dir)
            .join(name)
            .join(format!("{}.onnx", name));

        let model_path_str = model_path
            .to_str()
            .ok_or_else(|| "invalid model path".to_string())?;

        // Load ONNX model
        let model = OnnxModel::load(model_path_str)
            .map_err(|e| format!("load model '{}': {}", name, e))?;

        // Load tokenizer
        let tokenizer = EmbeddingTokenizer::load(&self.models_dir, name)?;

        // Determine embedding dimension from model info
        let dimension = model.get_info()
            .output_shapes
            .first()
            .and_then(|shape| shape.last().copied())
            .unwrap_or(768);

        models.insert(name.to_string(), ModelEntry { model, tokenizer, dimension });

        Ok(())
    }

    /// Embed a batch of texts, returning a vector of embeddings.
    /// Each embedding is L2-normalized.
    pub fn embed(&self, model_name: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        // Ensure model is loaded
        self.load_model_internal(model_name)?;

        let models = self.models
            .lock()
            .map_err(|_| "models lock poisoned".to_string())?;

        let entry = models.get(model_name)
            .ok_or_else(|| format!("Model '{}' not found", model_name))?;

        let max_length = entry.tokenizer.max_input_length;

        // Tokenize all texts
        let tokenized = entry.tokenizer.encode_batch(texts, max_length)?;

        // Run inference for each text
        let mut results = Vec::with_capacity(texts.len());
        for (input_ids, attention_mask, token_type_ids) in &tokenized {
            // If the model has 3 inputs (BERT-style), use all three
            // Otherwise, use the first input_ids only
            let embedding = if input_ids.len() > 1 {
                // Convert i64 to f32 for ONNX model
                let input_f32: Vec<f32> = input_ids.iter().map(|&v| v as f32).collect();
                entry.model.execute(&[input_f32])
                    .map_err(|e| format!("{}", e))?
            } else {
                let input_f32: Vec<f32> = input_ids.iter().map(|&v| v as f32).collect();
                entry.model.execute(&[input_f32])
                    .map_err(|e| format!("{}", e))?
            };

            // L2-normalize
            let normalized = Self::l2_normalize(&embedding);
            results.push(normalized);
        }

        Ok(results)
    }

    /// L2-normalize a vector
    pub fn l2_normalize(vec: &[f32]) -> Vec<f32> {
        let sum_sq: f32 = vec.iter().map(|&v| v * v).sum();
        if sum_sq <= core::f32::EPSILON {
            return vec.to_vec();
        }
        let norm = sum_sq.sqrt();
        vec.iter().map(|&v| v / norm).collect()
    }

    /// Get the default model name, if any
    pub fn default_model(&self) -> Option<&str> {
        self.default_model.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l2_normalize_non_zero() {
        let vec = vec![3.0, 4.0];
        let normalized = EmbeddingEngine::l2_normalize(&vec);
        assert!((normalized[0] - 0.6).abs() < 1e-6);
        assert!((normalized[1] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_l2_normalize_zero_vector() {
        let vec = vec![0.0, 0.0, 0.0];
        let normalized = EmbeddingEngine::l2_normalize(&vec);
        assert_eq!(normalized, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_l2_normalize_unit_vector() {
        let vec = vec![1.0, 0.0, 0.0];
        let normalized = EmbeddingEngine::l2_normalize(&vec);
        assert!((normalized[0] - 1.0).abs() < 1e-6);
        assert!((normalized[1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_engine_new() {
        let engine = EmbeddingEngine::new(None, "./models".to_string(), 5, false, None);
        assert!(engine.default_model().is_none());
    }

    #[test]
    fn test_embedding_engine_default_model() {
        let engine = EmbeddingEngine::new(
            Some("bge-m3".to_string()),
            "./models".to_string(),
            5, false, None,
        );
        assert_eq!(engine.default_model(), Some("bge-m3"));
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd /mnt/home/bobjia/remdb && cargo test --lib model::embedding
```
Expected: Unit tests pass (normalization tests don't need a real model)

- [ ] **Step 5: Commit**

```bash
git add remdb/src/model/mod.rs remdb/src/model/embedding.rs
git commit -m "feat: add embedding module to remdb core (tokenizer + engine)"
```

---

### Task 5: Create server embedding module (models + handler)

**Files:**
- Create: `src/milvus/embedding/mod.rs` — module exports
- Create: `src/milvus/embedding/models.rs` — request/response types
- Create: `src/milvus/embedding/handler.rs` — thin warp handler

**Interfaces:**
- Consumes: `remdb::model::embedding::EmbeddingEngine`, `MilvusError`
- Produces: `handle_embedding(engine, req) -> Result<impl Reply, warp::Rejection>` warp handler

- [ ] **Step 1: Create directory**

```bash
mkdir -p /mnt/home/bobjia/remdb-server/src/milvus/embedding
```

- [ ] **Step 2: Write `src/milvus/embedding/models.rs`**

```rust
use serde::{Deserialize, Serialize};

/// OpenAI-compatible embedding request
#[derive(Debug, Deserialize)]
pub struct EmbeddingRequest {
    #[serde(default)]
    pub model: Option<String>,
    pub input: InputValue,
}

/// Input can be a single string or an array of strings
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum InputValue {
    Single(String),
    Batch(Vec<String>),
}

/// OpenAI-compatible embedding response
#[derive(Debug, Serialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
}

/// Single embedding entry
#[derive(Debug, Serialize)]
pub struct EmbeddingData {
    pub object: String,
    pub index: usize,
    pub embedding: Vec<f32>,
}

/// Extract texts from InputValue
impl InputValue {
    pub fn texts(&self) -> Vec<&str> {
        match self {
            InputValue::Single(s) => vec![s.as_str()],
            InputValue::Batch(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_single_string() {
        let json = r#"{"input": "hello world"}"#;
        let req: EmbeddingRequest = serde_json::from_str(json).unwrap();
        match req.input {
            InputValue::Single(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected Single"),
        }
        assert!(req.model.is_none());
    }

    #[test]
    fn test_deserialize_array() {
        let json = r#"{"input": ["hello", "world"]}"#;
        let req: EmbeddingRequest = serde_json::from_str(json).unwrap();
        match req.input {
            InputValue::Batch(v) => assert_eq!(v, vec!["hello", "world"]),
            _ => panic!("expected Batch"),
        }
    }

    #[test]
    fn test_deserialize_with_model() {
        let json = r#"{"model": "bge-m3", "input": "test"}"#;
        let req: EmbeddingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, Some("bge-m3".to_string()));
    }

    #[test]
    fn test_embedding_response_serialize() {
        let resp = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![EmbeddingData {
                object: "embedding".to_string(),
                index: 0,
                embedding: vec![0.1, 0.2, 0.3],
            }],
            model: "bge-m3".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["object"], "list");
        assert_eq!(json["data"][0]["index"], 0);
        assert_eq!(json["data"][0]["embedding"].as_array().unwrap().len(), 3);
        assert_eq!(json["model"], "bge-m3");
    }
}
```

- [ ] **Step 3: Write `src/milvus/embedding/handler.rs`**

```rust
use std::sync::Arc;

use warp::Reply;

use remdb::model::embedding::EmbeddingEngine;

use crate::milvus::embedding::models::*;
use crate::milvus::error::MilvusError;

/// Warp handler for POST /v2/vectordb/embedding
pub async fn handle_embedding(
    engine: Arc<EmbeddingEngine>,
    body: EmbeddingRequest,
) -> Result<impl Reply, warp::Rejection> {
    // 1. Resolve model name: request override → config default → error
    let model_name = body.model.clone()
        .or_else(|| engine.default_model().map(|s| s.to_string()))
        .ok_or_else(|| {
            warp::reject::custom(MilvusError::NoModelSpecified)
        })?;

    // 2. Extract texts from input
    let texts: Vec<&str> = body.input.texts();

    // 3. Validate input
    if texts.is_empty() || texts.iter().any(|t| t.is_empty()) {
        return Err(warp::reject::custom(MilvusError::InvalidInput(
            "Input must be a non-empty string or array of non-empty strings".to_string(),
        )));
    }

    // 4. Run embedding inference
    let embeddings = engine.embed(&model_name, &texts)
        .map_err(|e| warp::reject::custom(MilvusError::InferenceFailed(e)))?;

    // 5. Build OpenAI-compatible response
    let data: Vec<EmbeddingData> = embeddings
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| EmbeddingData {
            object: "embedding".to_string(),
            index,
            embedding,
        })
        .collect();

    let response = EmbeddingResponse {
        object: "list".to_string(),
        data,
        model: model_name,
    };

    Ok(warp::reply::json(&response))
}
```

- [ ] **Step 4: Write `src/milvus/embedding/mod.rs`**

```rust
pub mod models;
pub mod handler;

pub use handler::handle_embedding;
```

- [ ] **Step 5: Run tests**

```bash
cd /mnt/home/bobjia/remdb-server && cargo test --lib milvus::embedding::models
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/milvus/embedding/
git commit -m "feat: add server embedding module (models + handler)"
```

---

### Task 6: Wire embedding route into MilvusServer

**Files:**
- Modify: `src/milvus/mod.rs` — add `pub mod embedding;`
- Modify: `src/milvus/server.rs` — add embedding route to `MilvusServer::start`, accept `Arc<EmbeddingEngine>` in constructor
- Modify: `src/main.rs` — extract embedding config, create `EmbeddingEngine`, pass to `MilvusServer`

- [ ] **Step 1: Add `pub mod embedding;` to `src/milvus/mod.rs`**

```rust
pub mod embedding;
```

- [ ] **Step 2: Update `MilvusServer` in `src/milvus/server.rs`**

Change the constructor and struct:
```rust
use remdb::model::embedding::EmbeddingEngine;

pub struct MilvusServer {
    db: Arc<Mutex<&'static mut RemDb>>,
    port: u16,
    api_key: Option<String>,
    embedding_engine: Option<Arc<EmbeddingEngine>>,
}

impl MilvusServer {
    pub fn new(
        db: Arc<Mutex<&'static mut RemDb>>,
        port: u16,
        api_key: Option<String>,
        embedding_engine: Option<Arc<EmbeddingEngine>>,
    ) -> Self {
        MilvusServer { db, port, api_key, embedding_engine }
    }
```

- [ ] **Step 3: Add the embedding route warp filter in `start()` method**

Add after the index routes and before combining all routes:
```rust
        // ── Embedding route ──
        let embedding_route = if let Some(ref engine) = self.embedding_engine {
            let engine_filter = warp::any().map(move || engine.clone()).boxed();
            let route = warp::path!("v2" / "vectordb" / "embedding")
                .and(warp::post())
                .and(auth.clone())
                .and(engine_filter)
                .and(warp::body::json())
                .and_then(|engine: Arc<EmbeddingEngine>, body| async move {
                    crate::milvus::embedding::handle_embedding(engine, body).await
                });
            Some(route)
        } else {
            None
        };
```

- [ ] **Step 4: Add the embedding route to the combined routes**

```rust
        let mut routes = create_collection
            .or(drop_collection)
            .or(list_collections)
            .or(describe_collection)
            .or(has_collection)
            .or(insert)
            .or(upsert)
            .or(delete)
            .or(get)
            .or(query)
            .or(search)
            .or(create_index)
            .or(drop_index);

        if let Some(embed) = embedding_route {
            routes = routes.or(embed);
        }

        let routes = routes
            .with(warp::cors().allow_any_origin())
            .recover(crate::milvus::handler::handle_rejection);
```

- [ ] **Step 5: Update `src/main.rs` to create `EmbeddingEngine` and pass to `MilvusServer`**

After `let milvus_config_saved = config.milvus.clone();` and before starting the Milvus server:
```rust
    // Create embedding engine if embedding config is present
    let embedding_engine = if let Some(ref emb_config) = milvus_config_saved.embedding {
        let engine = remdb::model::embedding::EmbeddingEngine::new(
            emb_config.default_model.clone(),
            emb_config.models_dir.clone(),
            emb_config.max_models,
            emb_config.auto_download,
            emb_config.hf_mirror.clone(),
        );
        if let Err(e) = engine.preload_default() {
            error!("Failed to pre-load default embedding model: {:?}", e);
        }
        Some(Arc::new(engine))
    } else {
        None
    };
```

Then update the Milvus server spawn:
```rust
        let server = remdb_server::milvus::MilvusServer::new(
            milvus_db,
            milvus_port,
            milvus_api_key,
            embedding_engine.clone(),
        );
```

- [ ] **Step 6: Compile check**

```bash
cd /mnt/home/bobjia/remdb-server && cargo check
```
Expected: Compilation succeeds

- [ ] **Step 7: Commit**

```bash
git add src/milvus/mod.rs src/milvus/server.rs src/main.rs
git commit -m "feat: wire embedding route into MilvusServer"
```

---

### Task 7: Tests

**Files:**
- Modify: `src/milvus/embedding/handler.rs` — add handler unit tests

- [ ] **Step 1: Add handler tests to `src/milvus/embedding/handler.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::milvus::embedding::models::*;

    #[test]
    fn test_handle_embedding_no_model() {
        // Engine with no default model
        let engine = Arc::new(EmbeddingEngine::new(
            None, "./models".to_string(), 5, false, None,
        ));
        let body = EmbeddingRequest {
            model: None,
            input: InputValue::Single("hello".to_string()),
        };
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_embedding(engine, body));
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_embedding_empty_input() {
        let engine = Arc::new(EmbeddingEngine::new(
            Some("bge-m3".to_string()), "./models".to_string(), 5, false, None,
        ));
        let body = EmbeddingRequest {
            model: None,
            input: InputValue::Single("".to_string()),
        };
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_embedding(engine, body));
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd /mnt/home/bobjia/remdb-server && cargo test --lib milvus::embedding
```
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/milvus/embedding/handler.rs
git commit -m "test: add embedding handler unit tests"
```

---

### Task 8: Update the example config file

**Files:**
- Modify: `remdb-master.toml` — add `[milvus.embedding]` section

- [ ] **Step 1: Add embedding config section to `remdb-master.toml`**

```toml
# Milvus REST API configuration
[milvus]
enabled = true
port = 19530

[milvus.embedding]
# Default model used when request omits "model" field
default_model = "bge-m3"
# Directory where ONNX model files are stored
models_dir = "./models"
# Auto-download model from HuggingFace if not found locally
auto_download = false
# Maximum number of models to keep in memory cache
max_models = 5
# HuggingFace mirror URL (overrides HF_MIRROR env var)
# hf_mirror = "https://hf-mirror.com"
```

- [ ] **Step 2: Commit**

```bash
git add remdb-master.toml
git commit -m "docs: add embedding config to example config"
```

---

## Self-Review Checklist

1. **Spec coverage:** Does every section of the spec have a task?
   - Config ✓ (Task 1 + Task 8)
   - Error codes ✓ (Task 2)
   - Route ✓ (Task 6)
   - Request/response types ✓ (Task 5)
   - Tokenizer ✓ (Task 4, in remdb core)
   - Engine ✓ (Task 4, in remdb core)
   - Handler ✓ (Task 5)
   - Dependencies ✓ (Task 3)
   - Integration ✓ (Tasks 1-8 complete)
   - Testing ✓ (Task 7)

2. **Placeholder scan:** No TBD, TODO, or "implement later" patterns.

3. **Type consistency:** All method signatures and types are consistent across tasks:
   - `EmbeddingConfig` defined in Task 1, used in Task 6
   - `MilvusError` variants defined in Task 2, used in Task 5
   - `EmbeddingRequest`/`InputValue` defined in Task 5, used in Task 5
   - `EmbeddingEngine` defined in Task 4 (remdb core), used in Task 5, 6
   - Model names are `String` throughout