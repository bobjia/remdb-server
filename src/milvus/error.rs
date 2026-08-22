use serde::Serialize;
use std::fmt;

/// Milvus-compatible error codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MilvusCode {
    Success = 0,
    CollectionNotFound = 1001,
    InvalidSchema = 1002,
    InvalidDimension = 1003,
    InvalidMetricType = 1004,
    InvalidIndexType = 1005,
    InvalidFieldName = 1006,
    DuplicateCollection = 1007,
    InsertFailed = 1008,
    SearchFailed = 1009,
    AuthenticationFailed = 2001,
    InternalError = 9999,
}

/// Milvus error type
#[derive(Debug, Clone)]
pub enum MilvusError {
    Success,
    CollectionNotFound(String),
    InvalidSchema(String),
    InvalidDimension(String),
    InvalidMetricType(String),
    InvalidIndexType(String),
    InvalidFieldName(String),
    DuplicateCollection(String),
    InsertFailed(String),
    SearchFailed(String),
    AuthenticationFailed,
    InternalError(String),
}

impl MilvusError {
    pub fn code(&self) -> i32 {
        match self {
            MilvusError::Success => 0,
            MilvusError::CollectionNotFound(_) => 1001,
            MilvusError::InvalidSchema(_) => 1002,
            MilvusError::InvalidDimension(_) => 1003,
            MilvusError::InvalidMetricType(_) => 1004,
            MilvusError::InvalidIndexType(_) => 1005,
            MilvusError::InvalidFieldName(_) => 1006,
            MilvusError::DuplicateCollection(_) => 1007,
            MilvusError::InsertFailed(_) => 1008,
            MilvusError::SearchFailed(_) => 1009,
            MilvusError::AuthenticationFailed => 2001,
            MilvusError::InternalError(_) => 9999,
        }
    }

    pub fn http_status(&self) -> u16 {
        match self {
            MilvusError::AuthenticationFailed => 401,
            MilvusError::InternalError(_) => 500,
            _ => 400,
        }
    }

    pub fn message(&self) -> String {
        match self {
            MilvusError::Success => "success".to_string(),
            MilvusError::CollectionNotFound(name) => format!("collection '{}' not found", name),
            MilvusError::InvalidSchema(msg) => format!("invalid schema: {}", msg),
            MilvusError::InvalidDimension(msg) => format!("invalid dimension: {}", msg),
            MilvusError::InvalidMetricType(msg) => format!("invalid metric type: {}", msg),
            MilvusError::InvalidIndexType(msg) => format!("invalid index type: {}", msg),
            MilvusError::InvalidFieldName(msg) => format!("invalid field name: {}", msg),
            MilvusError::DuplicateCollection(name) => format!("collection '{}' already exists", name),
            MilvusError::InsertFailed(msg) => format!("insert failed: {}", msg),
            MilvusError::SearchFailed(msg) => format!("search failed: {}", msg),
            MilvusError::AuthenticationFailed => "authentication failed".to_string(),
            MilvusError::InternalError(msg) => format!("internal error: {}", msg),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "code": self.code(),
            "message": self.message()
        })
    }
}

impl fmt::Display for MilvusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MilvusError({}): {}", self.code(), self.message())
    }
}

impl warp::reject::Reject for MilvusError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        let err = MilvusError::CollectionNotFound("test".to_string());
        assert_eq!(err.code(), 1001);
        assert_eq!(err.http_status(), 400);
    }

    #[test]
    fn test_error_response_json() {
        let err = MilvusError::CollectionNotFound("my_coll".to_string());
        let resp = err.to_json();
        assert_eq!(resp["code"], 1001);
        assert!(resp["message"].as_str().unwrap().contains("my_coll"));
    }

    #[test]
    fn test_auth_failed_status() {
        let err = MilvusError::AuthenticationFailed;
        assert_eq!(err.http_status(), 401);
        assert_eq!(err.code(), 2001);
    }

    #[test]
    fn test_internal_error_status() {
        let err = MilvusError::InternalError("test".to_string());
        assert_eq!(err.http_status(), 500);
        assert_eq!(err.code(), 9999);
    }
}
