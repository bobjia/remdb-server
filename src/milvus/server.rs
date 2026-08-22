use std::sync::{Arc, Mutex};
use warp::Filter;

use remdb::RemDb;

use crate::milvus::auth;
use crate::milvus::catalog::MilvusCatalog;
use crate::milvus::handler;

/// Milvus-compatible RESTful API server
pub struct MilvusServer {
    db: Arc<Mutex<&'static mut RemDb>>,
    port: u16,
    api_key: Option<String>,
}

impl MilvusServer {
    pub fn new(
        db: Arc<Mutex<&'static mut RemDb>>,
        port: u16,
        api_key: Option<String>,
    ) -> Self {
        MilvusServer { db, port, api_key }
    }

    pub async fn start(&self) {
        let catalog = Arc::new(MilvusCatalog::new(self.db.clone()));
        // Initialize catalog
        if let Err(e) = catalog.init().await {
            tracing::error!("Failed to init Milvus catalog: {:?}", e);
            return;
        }

        // Build auth filter (or no-op if no api_key configured)
        let auth = if let Some(ref key) = self.api_key {
            let hash = auth::hash_api_key(key);
            auth::auth_filter(hash).boxed()
        } else {
            // No auth required - use a filter with the same error type
            warp::any()
                .and_then(|| async move { Ok::<_, warp::Rejection>(()) })
                .map(|_: ()| ())
                .untuple_one()
                .boxed()
        };

        let catalog_filter = warp::any().map(move || catalog.clone()).boxed();

        // ── Collection routes ──
        let create_collection = warp::path!("v2" / "vectordb" / "collections" / "create")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_collection(catalog, body).await
            });

        let drop_collection = warp::path!("v2" / "vectordb" / "collections" / "drop")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_collection(catalog, body).await
            });

        let list_collections = warp::path!("v2" / "vectordb" / "collections" / "list")
            .and(warp::get())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and_then(|catalog: Arc<MilvusCatalog>| async move {
                handler::handle_list_collections(catalog).await
            });

        let describe_collection = warp::path!("v2" / "vectordb" / "collections" / "describe")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_describe_collection(catalog, body).await
            });

        let has_collection = warp::path!("v2" / "vectordb" / "collections" / "has")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_has_collection(catalog, body).await
            });

        // ── Entity routes ──
        // Entity handlers access the database through the catalog

        let insert = warp::path!("v2" / "vectordb" / "entities" / "insert")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_insert(catalog, body).await
            });

        let upsert = warp::path!("v2" / "vectordb" / "entities" / "upsert")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_upsert(catalog, body).await
            });

        let delete = warp::path!("v2" / "vectordb" / "entities" / "delete")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_delete(catalog, body).await
            });

        let get = warp::path!("v2" / "vectordb" / "entities" / "get")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_get(catalog, body).await
            });

        let query = warp::path!("v2" / "vectordb" / "entities" / "query")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_query(catalog, body).await
            });

        let search = warp::path!("v2" / "vectordb" / "entities" / "search")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_search(catalog, body).await
            });

        // ── Index routes ──
        let create_index = warp::path!("v2" / "vectordb" / "indexes" / "create")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_index(catalog, body).await
            });

        let drop_index = warp::path!("v2" / "vectordb" / "indexes" / "drop")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_index(catalog, body).await
            });

        // Combine all routes
        let routes = create_collection
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
            .or(drop_index)
            .with(warp::cors().allow_any_origin())
            .recover(handler::handle_rejection);

        tracing::info!("Milvus RESTful API server starting on port {}", self.port);
        warp::serve(routes).run(([0, 0, 0, 0], self.port)).await;
    }
}