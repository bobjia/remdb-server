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
    fn test_deserialize_single_string_with_spaces() {
        let json = r#"{"input": "hello world"}"#;
        let req: EmbeddingRequest = serde_json::from_str(json).unwrap();
        match req.input {
            InputValue::Single(s) => assert_eq!(s, "hello world"),
            _ => panic!("expected Single"),
        }
    }

    #[test]
    fn test_input_value_texts_single() {
        let input = InputValue::Single("hello".to_string());
        let texts = input.texts();
        assert_eq!(texts, vec!["hello"]);
    }

    #[test]
    fn test_input_value_texts_batch() {
        let input = InputValue::Batch(vec!["a".to_string(), "b".to_string()]);
        let texts = input.texts();
        assert_eq!(texts, vec!["a", "b"]);
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

    #[test]
    fn test_embedding_response_multiple_entries() {
        let resp = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![
                EmbeddingData {
                    object: "embedding".to_string(),
                    index: 0,
                    embedding: vec![0.1, 0.2],
                },
                EmbeddingData {
                    object: "embedding".to_string(),
                    index: 1,
                    embedding: vec![0.3, 0.4],
                },
            ],
            model: "bge-m3".to_string(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert_eq!(json["data"][0]["index"], 0);
        assert_eq!(json["data"][1]["index"], 1);
    }
}