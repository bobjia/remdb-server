use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schema: CollectionSchema,
    #[serde(default, rename = "indexParams")]
    pub index_params: Option<Vec<CreateIndexParam>>,
}

#[derive(Debug, Deserialize)]
pub struct CollectionSchema {
    #[serde(default)]
    pub auto_id: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub is_primary: Option<bool>,
    #[serde(default)]
    pub auto_id: Option<bool>,
    #[serde(default)]
    pub params: Option<FieldParams>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FieldParams {
    #[serde(default)]
    pub dim: Option<u16>,
    #[serde(default, rename = "max_length")]
    pub max_length: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DropCollectionRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DescribeCollectionRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
}

#[derive(Debug, Deserialize)]
pub struct HasCollectionRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
}

#[derive(Debug, Deserialize)]
pub struct InsertRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub filter: String,
}

#[derive(Debug, Deserialize)]
pub struct GetRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub id: i64,
    #[serde(default, rename = "outputFields")]
    pub output_fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default, rename = "outputFields")]
    pub output_fields: Option<Vec<String>>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub vector: Vec<f32>,
    #[serde(default, rename = "annsField")]
    pub anns_field: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default, rename = "outputFields")]
    pub output_fields: Option<Vec<String>>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub params: Option<SearchParams>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    pub ef: Option<u32>,
    #[serde(default)]
    pub nprobe: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIndexRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(rename = "indexName")]
    pub index_name: String,
    #[serde(rename = "fieldName")]
    pub field_name: String,
    #[serde(rename = "metricType")]
    pub metric_type: String,
    pub params: Option<IndexParams>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateIndexParam {
    #[serde(rename = "fieldName")]
    pub field_name: String,
    #[serde(rename = "indexName")]
    pub index_name: String,
    #[serde(rename = "metricType")]
    pub metric_type: String,
    pub params: Option<IndexParams>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IndexParams {
    #[serde(default)]
    pub nlist: Option<u32>,
    #[serde(default)]
    pub M: Option<u32>,
    #[serde(default)]
    pub efConstruction: Option<u32>,
    #[serde(default, rename = "index_type")]
    pub index_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DropIndexRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(rename = "indexName")]
    pub index_name: String,
}

#[derive(Debug, Serialize)]
pub struct MilvusResponse<T: Serialize> {
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> MilvusResponse<T> {
    pub fn success(data: T) -> Self {
        MilvusResponse { code: 0, message: None, data: Some(data) }
    }

    pub fn error(code: i32, message: String) -> Self {
        MilvusResponse { code, message: Some(message), data: None }
    }
}

#[derive(Debug, Serialize)]
pub struct CollectionInfo {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DescribeCollectionData {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub schema: CollectionSchemaResponse,
    pub statistics: CollectionStatistics,
}

#[derive(Debug, Serialize)]
pub struct CollectionSchemaResponse {
    pub fields: Vec<FieldSchemaResponse>,
}

#[derive(Debug, Serialize)]
pub struct FieldSchemaResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_id: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<FieldParams>,
}

#[derive(Debug, Serialize)]
pub struct CollectionStatistics {
    #[serde(rename = "rowCount")]
    pub row_count: usize,
}

#[derive(Debug, Serialize)]
pub struct InsertResponseData {
    #[serde(rename = "insertCount")]
    pub insert_count: usize,
    #[serde(rename = "insertIds")]
    pub insert_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct DeleteResponseData {
    #[serde(rename = "deleteCount")]
    pub delete_count: usize,
}

#[derive(Debug, Serialize)]
pub struct HasCollectionData {
    pub has: bool,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: i64,
    pub distance: f32,
    pub entity: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IndexInfo {
    #[serde(rename = "indexName")]
    pub index_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_collection_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "schema": {
                "autoId": true,
                "fields": [
                    {"name": "id", "type": "Int64", "isPrimary": true, "autoId": true},
                    {"name": "vector", "type": "FloatVector", "params": {"dim": 128}}
                ]
            }
        }"#;
        let req: CreateCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.schema.fields.len(), 2);
    }

    #[test]
    fn test_search_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "vector": [0.1, 0.2, 0.3],
            "annsField": "vector",
            "limit": 5
        }"#;
        let req: SearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.limit, Some(5));
    }

    #[test]
    fn test_milvus_response_success() {
        let resp = MilvusResponse::success(42);
        assert_eq!(resp.code, 0);
        assert_eq!(resp.data, Some(42));
        assert!(resp.message.is_none());
    }

    #[test]
    fn test_milvus_response_error() {
        let resp: MilvusResponse<()> = MilvusResponse::error(1001, "not found".to_string());
        assert_eq!(resp.code, 1001);
        assert_eq!(resp.message.as_deref(), Some("not found"));
    }
}
