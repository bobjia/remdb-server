# Embedding API Endpoint Design

## Overview

Add a `/v2/vectordb/embedding` endpoint to the remdb-server's existing Milvus-compatible REST API that converts text to vector embeddings using ONNX embedding models. The endpoint is OpenAI-compatible in request/response format but wrapped in the existing Milvus server infrastructure.

## Decisions

| Decision | Choice |
|---|---|
| Model selection | Configurable default model, overridable per-request via `model` field |
| Request/response format | OpenAI-compatible (`object`, `data[].embedding`, `model`) |
| Model loading | Pre-load default model at startup; override models loaded on-demand and cached |
| Input formats | Both single string and array of strings |
| Token usage | Skipped entirely |
| Config location | `[milvus.embedding]` section in `remdb-master.toml` |
| Embedding normalization | L2-normalize before returning |
| Input truncation | Truncate silently to model's `max_input_length` |
| Tokenization | Hugging Face `tokenizers` Rust crate |
| Architecture | `EmbeddingEngine` + `EmbeddingTokenizer` in remdb core library (`remdb/src/model/embedding.rs`); thin HTTP handler in `src/milvus/embedding/handler.rs` |

## Route

`POST /v2/vectordb/embedding`

## Request

```json
{
  "model": "bge-m3",
  "input": "what is remdb?"
}
```

- `model` (optional string) — overrides the server default model
- `input` (required string or array of strings) — text(s) to embed

## Response (success)

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "index": 0,
      "embedding": [0.012, -0.034, 0.078, ...]
    }
  ],
  "model": "bge-m3"
}
```

## Error response

```json
{
  "code": 1,
  "message": "Model 'bge-m3' not found. Available models: all-minilm-l6-v2, bge-small-zh"
}
```

## Configuration

```toml
[milvus]
enabled = true
port = 19530

[milvus.embedding]
default_model = "bge-m3"
models_dir = "./models"
auto_download = true
max_models = 5
hf_mirror = "https://hf-mirror.com"
```

### Config fields

| Field | Type | Default | Description |
|---|---|---|---|
| `default_model` | string | `None` | Default model name used when request omits `model` |
| `models_dir` | string | `"./models"` | Directory where ONNX model files are stored |
| `auto_download` | bool | `false` | Download model from HuggingFace if not found locally |
| `max_models` | usize | `5` | Max number of models to keep in the in-memory cache |
| `hf_mirror` | string | `None` | When set, download URLs are rewritten to use this mirror instead of `huggingface.co` (config takes precedence over `HF_MIRROR` env var) |

### Model resolution

1. If `model` field in request → use that model name
2. Else if `default_model` in config → use that
3. Else → return error (code 6, "No model specified")

## Architecture

```
remdb/                          ← 核心库
└── src/model/
    ├── mod.rs                  ← pub mod embedding; (gate: model-runtime)
    └── embedding.rs            ← EmbeddingTokenizer + EmbeddingEngine

remdb-server/                   ← 应用层
└── src/milvus/embedding/
    ├── mod.rs                  ← public exports
    ├── models.rs               ← EmbeddingRequest, EmbeddingResponse types
    └── handler.rs              ← warp handler function (thin)
```

### Component responsibilities

**`remdb/src/model/embedding.rs`** (core library) — The heavy lifting, behind `#[cfg(feature = "model-runtime")]`:

- `EmbeddingTokenizer` — Wraps Hugging Face `tokenizers` crate:
  - Loads a tokenizer from `{models_dir}/{model_name}/tokenizer.json`
  - `encode(text, max_length) -> (input_ids, attention_mask, token_type_ids)`
  - `encode_batch(texts, max_length) -> Vec<(input_ids, attention_mask, token_type_ids)>`
  - Caches tokenizers in memory (one per model)

- `EmbeddingEngine` — Orchestrates model + tokenizer:
  - `load_model(name, path)` — loads an ONNX model + tokenizer, caches them
  - `embed(model_name, texts) -> Result<Vec<Vec<f32>>>` — tokenizes → runs inference → extracts → L2-normalizes
  - Manages the model cache (LRU, up to `max_models`)
  - Handles truncation (truncates tokenized input to model's `max_input_length`)
  - Reuses `OnnxModel` from the existing `remdb::model::onnx_runtime` for inference

**`src/milvus/embedding/models.rs`** (server) — Request/response types matching the OpenAI-compatible format:
- `EmbeddingRequest { model: Option<String>, input: InputValue }` where `InputValue` is an enum: `Single(String)` or `Batch(Vec<String>)`
- `EmbeddingResponse { object: String, data: Vec<EmbeddingData>, model: String }`
- `EmbeddingData { object: String, index: usize, embedding: Vec<f32> }`

**`src/milvus/embedding/handler.rs`** (server) — Thin warp handler:
- Parses `EmbeddingRequest` from JSON body
- Resolves the model name (request override → config default → error)
- Calls `engine.embed(model_name, texts)` on the core `EmbeddingEngine`
- Builds the OpenAI-compatible response
- Routes errors to appropriate HTTP status codes via `MilvusError`

### Integration

In `server.rs`, the embedding route is added alongside the existing routes:

```rust
let embedding = warp::path!("v2" / "vectordb" / "embedding")
    .and(warp::post())
    .and(auth.clone())
    .and(embedding_engine.clone())
    .and(warp::body::json())
    .and_then(handler::handle_embedding);
```

The `EmbeddingEngine` is initialized at server startup (pre-loading the default model) and shared via `Arc<EmbeddingEngine>`.

## Data Flow

```
Client                     Milvus Server (warp)
  │                              │
  │  POST /v2/vectordb/embedding │
  │  { "input": "hello world" }  │
  │─────────────────────────────>│
  │                              │
  │                  ┌───────────┴────────────┐
  │                  │  1. Auth filter        │
  │                  │  2. Route match        │
  │                  │  3. Parse JSON body    │
  │                  │  4. Resolve model name │
  │                  └───────────┬────────────┘
  │                              │
  │                  ┌───────────┴────────────┐
  │                  │  EmbeddingEngine        │
  │                  │                         │
  │                  │  ┌───────────────────┐  │
  │                  │  │ 5. Get/Cache model │  │
  │                  │  │    (OnnxModel)     │  │
  │                  │  └────────┬──────────┘  │
  │                  │           │              │
  │                  │  ┌────────┴──────────┐  │
  │                  │  │ 6. Tokenize text  │  │
  │                  │  │    (Tokenizer)    │  │
  │                  │  └────────┬──────────┘  │
  │                  │           │              │
  │                  │  ┌────────┴──────────┐  │
  │                  │  │ 7. Model inference│  │
  │                  │  │    (onnx_runtime) │  │
  │                  │  └────────┬──────────┘  │
  │                  │           │              │
  │                  │  ┌────────┴──────────┐  │
  │                  │  │ 8. Extract +      │  │
  │                  │  │    L2 normalize   │  │
  │                  │  └────────┬──────────┘  │
  │                  └───────────┬────────────┘
  │                              │
  │  OpenAI-compatible response  │
  │  { "data": [{ "embedding":  │
  │      [0.012, -0.034, ...] }]│
  │      "model": "bge-m3" }    │
  │<─────────────────────────────│
```

## Error Handling

| Scenario | HTTP Status | `code` | `message` |
|---|---|---|---|
| Model not found | 404 | 1 | `"Model 'xyz' not found"` |
| Invalid input (empty string/array) | 400 | 2 | `"Input must be a non-empty string or array of non-empty strings"` |
| Input exceeds max tokens | 400 | 3 | `"Input exceeds maximum length of N tokens"` |
| Model load failure | 500 | 4 | `"Failed to load model 'xyz': ..."` |
| Inference failure | 500 | 5 | `"Inference failed: ..."` |
| No model specified | 400 | 6 | `"No model specified and no default model configured"` |

Errors are returned as `MilvusError` variants (reusing the existing error type) and converted to JSON by the existing `handle_rejection` recovery function.

## Testing

### Unit tests

- **`models.rs`** — Test request deserialization (single string, array, with/without model field)
- **`engine.rs`** — Mock tests for normalization, tokenization, model resolution
- **`tokenizer.rs`** — Test tokenizer encoding (if a test tokenizer is available)

### Integration tests

- **Route tests** — Test the embedding route with mocked engine, verify correct HTTP responses for all scenarios
- **End-to-end** — If a small ONNX model + tokenizer fixture is available, test the full pipeline

## Dependencies

- `tokenizers` crate (Hugging Face) — text tokenization (in remdb core, behind `model-runtime` feature)
- `hf-hub` crate (optional) — model download from HuggingFace Hub
- No new HTTP server — reuses existing warp server
- `EmbeddingEngine` and `EmbeddingTokenizer` are in remdb core (`remdb/src/model/embedding.rs`), available to all consumers (HTTP API, SQL UDF, etc.)

## Open Questions

- None. All design decisions are finalized.