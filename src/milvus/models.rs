use serde::{Deserialize, Serialize};

// ── Collection operations ──

#[derive(Debug, Deserialize, Clone)]
pub struct CreateCollectionRequest {
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schema: CollectionSchema,
    #[serde(default, rename = "indexParams")]
    pub index_params: Option<Vec<CreateIndexParam>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CollectionSchema {
    #[serde(default, rename = "autoId")]
    pub auto_id: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default, rename = "isPrimary")]
    pub is_primary: Option<bool>,
    #[serde(default, rename = "autoId")]
    pub auto_id: Option<bool>,
    #[serde(default)]
    pub params: Option<FieldParams>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

// ── Entity operations ──

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

// ── Index operations ──

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateIndexParam {
    #[serde(rename = "fieldName")]
    pub field_name: String,
    #[serde(rename = "indexName")]
    pub index_name: String,
    #[serde(rename = "metricType")]
    pub metric_type: String,
    #[serde(default)]
    pub params: Option<IndexParams>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

// ── Responses ──

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

#[derive(Debug, Serialize, Deserialize)]
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
        assert_eq!(req.schema.auto_id, Some(true));
        assert_eq!(req.schema.fields[0].name, "id");
        assert_eq!(req.schema.fields[0].field_type, "Int64");
        assert_eq!(req.schema.fields[0].is_primary, Some(true));
        assert_eq!(req.schema.fields[0].auto_id, Some(true));
        assert_eq!(req.schema.fields[1].name, "vector");
        assert_eq!(req.schema.fields[1].field_type, "FloatVector");
        assert_eq!(req.schema.fields[1].params.as_ref().unwrap().dim, Some(128));
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
        assert_eq!(req.vector, vec![0.1, 0.2, 0.3]);
        assert_eq!(req.anns_field, Some("vector".to_string()));
        assert_eq!(req.limit, Some(5));
    }

    #[test]
    fn test_search_request_minimal() {
        let json = r#"{
            "collectionName": "test",
            "vector": [0.5]
        }"#;
        let req: SearchRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.limit, None);
        assert_eq!(req.anns_field, None);
    }

    #[test]
    fn test_milvus_response_success() {
        let data = CollectionInfo {
            collection_name: "test".to_string(),
            description: None,
        };
        let resp = MilvusResponse::success(data);
        assert_eq!(resp.code, 0);
        assert!(resp.message.is_none());
        assert!(resp.data.is_some());
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["code"], 0);
        assert_eq!(json["data"]["collectionName"], "test");
    }

    #[test]
    fn test_insert_response_data() {
        let data = InsertResponseData {
            insert_count: 3,
            insert_ids: vec![1, 2, 3],
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["insertCount"], 3);
        assert_eq!(json["insertIds"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_delete_response_data() {
        let data = DeleteResponseData { delete_count: 5 };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["deleteCount"], 5);
    }

    #[test]
    fn test_has_collection_data() {
        let data = HasCollectionData { has: true };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["has"], true);
    }

    #[test]
    fn test_collection_info_with_description() {
        let info = CollectionInfo {
            collection_name: "test".to_string(),
            description: Some("a test collection".to_string()),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["collectionName"], "test");
        assert_eq!(json["description"], "a test collection");
    }

    #[test]
    fn test_collection_info_without_description() {
        let info = CollectionInfo {
            collection_name: "test".to_string(),
            description: None,
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["collectionName"], "test");
        assert!(json.get("description").is_none());
    }

    #[test]
    fn test_insert_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "data": [
                {"id": 1, "name": "Alice", "age": 30},
                {"id": 2, "name": "Bob", "age": 25}
            ]
        }"#;
        let req: InsertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.data.len(), 2);
    }

    #[test]
    fn test_query_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "filter": "id > 10",
            "outputFields": ["id", "name"],
            "limit": 100
        }"#;
        let req: QueryRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.filter, Some("id > 10".to_string()));
        assert_eq!(req.output_fields, Some(vec!["id".to_string(), "name".to_string()]));
        assert_eq!(req.limit, Some(100));
    }

    #[test]
    fn test_delete_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "filter": "id in [1, 2, 3]"
        }"#;
        let req: DeleteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.filter, "id in [1, 2, 3]");
    }

    #[test]
    fn test_get_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "id": 42
        }"#;
        let req: GetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.id, 42);
    }

    #[test]
    fn test_collection_schema_serialize() {
        let schema = CollectionSchema {
            auto_id: Some(true),
            description: Some("test schema".to_string()),
            fields: vec![
                FieldSchema {
                    name: "id".to_string(),
                    field_type: "Int64".to_string(),
                    is_primary: Some(true),
                    auto_id: Some(true),
                    params: None,
                },
            ],
        };
        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["autoId"], true);
        assert_eq!(json["fields"][0]["name"], "id");
        assert_eq!(json["fields"][0]["type"], "Int64");
    }

    #[test]
    fn test_search_result_item_serialize() {
        let item = SearchResultItem {
            id: 1,
            distance: 0.5,
            entity: serde_json::json!({"name": "test"}),
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["id"], 1);
        assert_eq!(json["distance"], 0.5);
        assert_eq!(json["entity"]["name"], "test");
    }

    #[test]
    fn test_upsert_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "data": [
                {"id": 1, "name": "Alice"}
            ]
        }"#;
        let req: UpsertRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.data.len(), 1);
    }

    #[test]
    fn test_describe_collection_request_deserialize() {
        let json = r#"{
            "collectionName": "test"
        }"#;
        let req: DescribeCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
    }

    #[test]
    fn test_drop_collection_request_deserialize() {
        let json = r#"{
            "collectionName": "test"
        }"#;
        let req: DropCollectionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
    }

    #[test]
    fn test_create_index_request_deserialize() {
        let json = r#"{
            "collectionName": "test",
            "indexName": "idx1",
            "fieldName": "vector",
            "metricType": "L2",
            "params": {
                "nlist": 128,
                "M": 16,
                "efConstruction": 200
            }
        }"#;
        let req: CreateIndexRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.collection_name, "test");
        assert_eq!(req.index_name, "idx1");
        assert_eq!(req.field_name, "vector");
        assert_eq!(req.params.as_ref().unwrap().nlist, Some(128));
    }
}