//! Integration tests for the Milvus-compatible RESTful API.
//!
//! These tests use `warp::test::request()` to exercise the route handlers
//! without starting a real HTTP server. The database is initialized once
//! per test process.

use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::sync::Once;

use remdb::RemDb;
use serde_json::Value;
use warp::Filter;

use remdb_server::milvus::catalog::MilvusCatalog;
use remdb_server::milvus::handler;
use remdb_server::milvus::models::*;

// ── Global test state ──

static INIT_DB: Once = Once::new();
static TEST_DB: OnceLock<Arc<Mutex<&'static mut RemDb>>> = OnceLock::new();
static SHARED_CATALOG: OnceLock<Arc<MilvusCatalog>> = OnceLock::new();

/// Initialize the global memory allocator exactly once.
///
/// Tests run in parallel threads, so several tests may reach the init path at
/// once. A second `init_global_allocator` call would clobber the allocator and
/// corrupt shared state (segfault), so every init path must go through this
/// `Once` guard.
fn init_global_allocator_once() {
    static INIT_ALLOCATOR: Once = Once::new();
    INIT_ALLOCATOR.call_once(|| {
        let total_memory = 1024 * 1024 * 200; // 200 MB
        let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
        let memory_ptr = memory_vec.as_ptr() as *mut u8;
        std::mem::forget(memory_vec);

        unsafe {
            remdb::memory::allocator::init_global_allocator(memory_ptr, total_memory)
                .expect("Failed to initialize global memory allocator");
        }
    });
}

/// Initialize the global database once for all tests.
fn init_test_db() -> Arc<Mutex<&'static mut RemDb>> {
    INIT_DB.call_once(|| {
        // Create necessary directories
        let _ = std::fs::create_dir_all("./test_logs");
        let _ = std::fs::create_dir_all("./test_snapshots");
        let _ = std::fs::create_dir_all("./wal");

        let total_memory = 1024 * 1024 * 200; // 200 MB

        let config = remdb_server::config::RuntimeConfig {
            snapshot_dir: Some("./test_snapshots".to_string()),
            full_image: None,
            total_memory,
            default_max_records: 1000,
            low_power_mode_supported: true,
            low_power_max_records: None,
            log_path: None,
            log_file_name: "./test_logs/test.log".to_string(),
            snapshot_interval: None,
            snapshot_type: None,
            max_incremental_snapshots: None,
            debug_mode: true,
            jdbc: remdb_server::config::JdbcConfig {
                port: Some(16666),
                enabled: Some(true),
                max_connections: Some(10),
                timeout: Some(30),
                auth_enabled: Some(false),
                username: None,
                password_hash: None,
            },
            pubsub: remdb_server::config::PubSubConfig::default(),
            ha: remdb_server::config::HaConfig::default(),
            wal: remdb_server::config::WALConfig::default(),
            ddl_path: None,
        };

        // Initialize global memory allocator (required before database init)
        init_global_allocator_once();

        let ctx = remdb_server::context::AppContextBuilder::new()
            .with_config(config)
            .with_tables(vec![])
            .build()
            .expect("Failed to initialize test database");

        let _ = TEST_DB.set(ctx.db.clone());
    });

    TEST_DB.get().unwrap().clone()
}

/// Get or initialize the shared Milvus catalog.
/// Only the first call initializes the catalog (calls catalog.init()).
async fn get_catalog() -> Arc<MilvusCatalog> {
    if let Some(catalog) = SHARED_CATALOG.get() {
        return catalog.clone();
    }
    let db = init_test_db();
    let catalog = Arc::new(MilvusCatalog::new(db));
    catalog.init().await.expect("Failed to init catalog");
    let _ = SHARED_CATALOG.set(catalog);
    SHARED_CATALOG.get().unwrap().clone()
}

// ── Test harness ──

/// Helper to build Milvus routes for testing.
/// Returns the routes and the catalog (held alive for the test duration).
macro_rules! build_test_routes {
    ($catalog:expr) => {{
        let __catalog: Arc<MilvusCatalog> = $catalog;
        let __auth = warp::any()
            .and_then(|| async move { Ok::<_, warp::Rejection>(()) })
            .map(|_: ()| ())
            .untuple_one()
            .boxed();
        let __catalog_filter = warp::any().map(move || __catalog.clone()).boxed();

        let __create_collection = warp::path!("v2" / "vectordb" / "collections" / "create")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_collection(catalog, body).await
            });

        let __drop_collection = warp::path!("v2" / "vectordb" / "collections" / "drop")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_collection(catalog, body).await
            });

        let __list_collections = warp::path!("v2" / "vectordb" / "collections" / "list")
            .and(warp::get())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and_then(|catalog: Arc<MilvusCatalog>| async move {
                handler::handle_list_collections(catalog).await
            });

        let __describe_collection = warp::path!("v2" / "vectordb" / "collections" / "describe")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_describe_collection(catalog, body).await
            });

        let __has_collection = warp::path!("v2" / "vectordb" / "collections" / "has")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_has_collection(catalog, body).await
            });

        let __insert = warp::path!("v2" / "vectordb" / "entities" / "insert")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_insert(catalog, body).await
            });

        let __upsert = warp::path!("v2" / "vectordb" / "entities" / "upsert")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_upsert(catalog, body).await
            });

        let __delete = warp::path!("v2" / "vectordb" / "entities" / "delete")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_delete(catalog, body).await
            });

        let __get = warp::path!("v2" / "vectordb" / "entities" / "get")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_get(catalog, body).await
            });

        let __query = warp::path!("v2" / "vectordb" / "entities" / "query")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_query(catalog, body).await
            });

        let __search = warp::path!("v2" / "vectordb" / "entities" / "search")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_search(catalog, body).await
            });

        let __create_index = warp::path!("v2" / "vectordb" / "indexes" / "create")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_index(catalog, body).await
            });

        let __drop_index = warp::path!("v2" / "vectordb" / "indexes" / "drop")
            .and(warp::post())
            .and(__auth.clone())
            .and(__catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_index(catalog, body).await
            });

        __create_collection
            .or(__drop_collection)
            .or(__list_collections)
            .or(__describe_collection)
            .or(__has_collection)
            .or(__insert)
            .or(__upsert)
            .or(__delete)
            .or(__get)
            .or(__query)
            .or(__search)
            .or(__create_index)
            .or(__drop_index)
            .with(warp::cors().allow_any_origin())
            .recover(handler::handle_rejection)
    }};
}

/// Helper to send a test request and get the response.
async fn test_request<F>(
    routes: &F,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> warp::http::Response<warp::hyper::body::Bytes>
where
    F: Filter + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    let mut req = warp::test::request().method(method).path(path);
    if let Some(b) = body {
        req = req.json(&b);
    }
    req.reply(routes).await
}

/// Helper to parse a response body as JSON.
async fn parse_response(resp: warp::http::Response<warp::hyper::body::Bytes>) -> (u16, Value) {
    let status = resp.status().as_u16();
    let body_bytes = resp.into_body();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();
    (status, json)
}

// ============================================================================
// Collection CRUD tests
// ============================================================================

#[tokio::test]
async fn test_create_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let body = serde_json::json!({
        "collectionName": "test_create",
        "schema": {
            "autoId": true,
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true, "autoId": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });

    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(body)).await;
    let (status, json) = parse_response(resp).await;

    assert_eq!(status, 200, "Create collection failed: {:?}", json);
    assert_eq!(json["code"], 0, "Expected success code, got: {:?}", json);
    assert_eq!(json["data"]["collectionName"], "test_create");

    // Clean up: drop the collection
    let drop_body = serde_json::json!({"collectionName": "test_create"});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
    let (_status, _json) = parse_response(resp).await;
}

#[tokio::test]
async fn test_collection_full_crud() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);
    let coll_name = "test_crud";

    // 1. Has collection (should be false initially)
    let has_body = serde_json::json!({"collectionName": coll_name});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/has", Some(has_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert_eq!(json["data"]["has"], false, "Collection should not exist yet");

    // 2. Create collection
    let create_body = serde_json::json!({
        "collectionName": coll_name,
        "schema": {
            "autoId": true,
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true, "autoId": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Create failed: {:?}", json);

    // 3. Has collection (should be true now)
    let has_body = serde_json::json!({"collectionName": coll_name});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/has", Some(has_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert_eq!(json["data"]["has"], true, "Collection should exist now");

    // 4. List collections
    let resp = test_request(&routes, "GET", "/v2/vectordb/collections/list", None).await;
    let (_status, json) = parse_response(resp).await;
    let collections = json["data"].as_array().unwrap();
    assert!(collections.iter().any(|c| c["collectionName"] == coll_name));

    // 5. Describe collection
    let describe_body = serde_json::json!({"collectionName": coll_name});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/describe", Some(describe_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert_eq!(json["data"]["collectionName"], coll_name);

    // 6. Insert entities
    let insert_body = serde_json::json!({
        "collectionName": coll_name,
        "data": [
            {"id": 1, "vector": [0.1, 0.2, 0.3, 0.4], "name": "item1"},
            {"id": 2, "vector": [0.5, 0.6, 0.7, 0.8], "name": "item2"}
        ]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/insert", Some(insert_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert_eq!(json["data"]["insertCount"], 2, "Insert failed: {:?}", json);

    // 7. Query entities
    let query_body = serde_json::json!({
        "collectionName": coll_name,
        "outputFields": ["id", "name"]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/query", Some(query_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Query failed: {:?}", json);

    // 8. Get entity by ID
    let get_body = serde_json::json!({"collectionName": coll_name, "id": 1});
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/get", Some(get_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Get failed: {:?}", json);

    // 9. Delete entity
    let delete_body = serde_json::json!({"collectionName": coll_name, "filter": "id in [1]"});
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/delete", Some(delete_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Delete failed: {:?}", json);

    // 10. Drop collection
    let drop_body = serde_json::json!({"collectionName": coll_name});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Drop failed: {:?}", json);
}

// ============================================================================
// Error handling tests
// ============================================================================

#[tokio::test]
async fn test_describe_nonexistent_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let body = serde_json::json!({"collectionName": "nonexistent"});
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/describe", Some(body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error code, got success: {:?}", json);
}

#[tokio::test]
async fn test_create_duplicate_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let create_body = serde_json::json!({
        "collectionName": "test_dup",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}}
            ]
        }
    });

    // First create should succeed
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body.clone())).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Duplicate should fail
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for duplicate: {:?}", json);

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_dup"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_insert_into_nonexistent_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let body = serde_json::json!({
        "collectionName": "does_not_exist",
        "data": [{"id": 1, "name": "test"}]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/insert", Some(body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for nonexistent collection");
}

#[tokio::test]
async fn test_invalid_collection_schema() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Missing primary key
    let body = serde_json::json!({
        "collectionName": "bad_schema",
        "schema": {
            "fields": [
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for bad schema: {:?}", json);
}

// ============================================================================
// Vector search test
// ============================================================================

#[tokio::test]
async fn test_vector_search() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create collection with vector field
    let create_body = serde_json::json!({
        "collectionName": "test_search",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Insert vectors
    let insert_body = serde_json::json!({
        "collectionName": "test_search",
        "data": [
            {"id": 1, "vector": [0.1, 0.2, 0.3, 0.4], "name": "a"},
            {"id": 2, "vector": [0.5, 0.6, 0.7, 0.8], "name": "b"},
            {"id": 3, "vector": [0.9, 0.8, 0.7, 0.6], "name": "c"}
        ]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/insert", Some(insert_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Search
    let search_body = serde_json::json!({
        "collectionName": "test_search",
        "vector": [0.1, 0.2, 0.3, 0.4],
        "annsField": "vector",
        "limit": 2,
        "outputFields": ["id", "name"]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/search", Some(search_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert_eq!(json["code"], 0, "Search returned error: {:?}", json);

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_search"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_upsert_entity() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create collection
    let create_body = serde_json::json!({
        "collectionName": "test_upsert",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Insert first
    let insert_body = serde_json::json!({
        "collectionName": "test_upsert",
        "data": [{"id": 1, "vector": [0.1, 0.2, 0.3, 0.4], "name": "original"}]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/insert", Some(insert_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Upsert (update existing record)
    let upsert_body = serde_json::json!({
        "collectionName": "test_upsert",
        "data": [{"id": 1, "vector": [0.1, 0.2, 0.3, 0.4], "name": "updated"}]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/upsert", Some(upsert_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200, "Upsert failed");

    // Upsert (insert new record)
    let upsert_body = serde_json::json!({
        "collectionName": "test_upsert",
        "data": [{"id": 2, "vector": [0.5, 0.6, 0.7, 0.8], "name": "new"}]
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/entities/upsert", Some(upsert_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200, "Upsert new failed");

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_upsert"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

/// Minimal test: the global allocator initializes exactly once, even under
/// concurrent test execution.
#[tokio::test]
async fn test_init_allocator_only() {
    init_global_allocator_once();
}

/// Minimal test: the shared database (built once for all tests) initializes.
#[tokio::test]
async fn test_init_db_only() {
    init_test_db();
    assert!(
        TEST_DB.get().is_some(),
        "shared test database should be initialized"
    );
}

// ============================================================================
// Index creation tests
// ============================================================================

#[tokio::test]
async fn test_create_index_on_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create a collection with a vector field
    let create_body = serde_json::json!({
        "collectionName": "test_create_index",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200, "Failed to create collection");

    // Create index on the vector field
    let create_index_body = serde_json::json!({
        "collectionName": "test_create_index",
        "indexName": "idx_vector",
        "fieldName": "vector",
        "metricType": "L2",
        "params": {
            "nlist": 128,
            "M": 16,
            "efConstruction": 200,
            "index_type": "HNSW"
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/create", Some(create_index_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Create index failed: {:?}", json);
    assert_eq!(json["code"], 0, "Expected success code, got: {:?}", json);
    assert_eq!(json["data"]["indexName"], "idx_vector");

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_create_index"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_create_index_invalid_metric_type() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create a collection first
    let create_body = serde_json::json!({
        "collectionName": "test_invalid_metric",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Try to create index with invalid metric type
    let create_index_body = serde_json::json!({
        "collectionName": "test_invalid_metric",
        "indexName": "bad_idx",
        "fieldName": "vector",
        "metricType": "INVALID_METRIC",
        "params": {}
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/create", Some(create_index_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for invalid metric type");

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_invalid_metric"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_create_index_on_nonexistent_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let create_index_body = serde_json::json!({
        "collectionName": "nonexistent_collection",
        "indexName": "idx",
        "fieldName": "vector",
        "metricType": "L2",
        "params": {}
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/create", Some(create_index_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for nonexistent collection");
}

#[tokio::test]
async fn test_create_index_on_non_vector_field() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create a collection with a vector field and a scalar field
    let create_body = serde_json::json!({
        "collectionName": "test_non_vector",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Try to create index on a non-vector field
    let create_index_body = serde_json::json!({
        "collectionName": "test_non_vector",
        "indexName": "bad_idx",
        "fieldName": "name",
        "metricType": "L2",
        "params": {}
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/create", Some(create_index_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for non-vector field");

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_non_vector"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_drop_index_works() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    // Create a collection
    let create_body = serde_json::json!({
        "collectionName": "test_drop_index",
        "schema": {
            "fields": [
                {"name": "id", "type": "Int64", "isPrimary": true},
                {"name": "vector", "type": "FloatVector", "params": {"dim": 4}},
                {"name": "name", "type": "VarChar", "params": {"max_length": 64}}
            ]
        }
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/collections/create", Some(create_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Create index first
    let create_index_body = serde_json::json!({
        "collectionName": "test_drop_index",
        "indexName": "idx_to_drop",
        "fieldName": "vector",
        "metricType": "L2",
        "params": {"nlist": 128}
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/create", Some(create_index_body)).await;
    let (status, _json) = parse_response(resp).await;
    assert_eq!(status, 200);

    // Drop index
    let drop_index_body = serde_json::json!({
        "collectionName": "test_drop_index",
        "indexName": "idx_to_drop"
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/drop", Some(drop_index_body)).await;
    let (status, json) = parse_response(resp).await;
    assert_eq!(status, 200, "Drop index failed: {:?}", json);
    assert_eq!(json["code"], 0, "Expected success code, got: {:?}", json);

    // Clean up
    let drop_body = serde_json::json!({"collectionName": "test_drop_index"});
    let _ = test_request(&routes, "POST", "/v2/vectordb/collections/drop", Some(drop_body)).await;
}

#[tokio::test]
async fn test_drop_index_on_nonexistent_collection() {
    let catalog = get_catalog().await;
    let routes = build_test_routes!(catalog);

    let drop_index_body = serde_json::json!({
        "collectionName": "nonexistent",
        "indexName": "some_idx"
    });
    let resp = test_request(&routes, "POST", "/v2/vectordb/indexes/drop", Some(drop_index_body)).await;
    let (_status, json) = parse_response(resp).await;
    assert!(json["code"].as_i64().unwrap() != 0, "Expected error for nonexistent collection");
}