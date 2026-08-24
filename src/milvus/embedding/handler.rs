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