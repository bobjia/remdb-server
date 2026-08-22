use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

use remdb::RemDb;

use crate::milvus::auth;
use crate::milvus::catalog::MilvusCatalog;
use crate::milvus::handler;

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
        if let Err(e) = catalog.init().await {
            tracing::error!("Failed to init Milvus catalog: {:?}", e);
            return;
        }

        let auth = if let Some(ref key) = self.api_key {
            let hash = auth::hash_api_key(key);
            auth::auth_filter(hash).boxed()
        } else {
            warp::any().map(|| ()).boxed()
        };

        let catalog_filter = warp::any().map(move || catalog.clone());

        let create_collection = warp::path!("v2" / "vectordb" / "collections" / "create")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_collection(&catalog, body).await
            });

        let drop_collection = warp::path!("v2" / "vectordb" / "collections" / "drop")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_collection(&catalog, body).await
            });

        let list_collections = warp::path!("v2" / "vectordb" / "collections" / "list")
            .and(warp::get())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .then(|catalog: Arc<MilvusCatalog>| async move {
                handler::handle_list_collections(&catalog).await
            });

        let describe_collection = warp::path!("v2" / "vectordb" / "collections" / "describe")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_describe_collection(&catalog, body).await
            });

        let has_collection = warp::path!("v2" / "vectordb" / "collections" / "has")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_has_collection(&catalog, body).await
            });

        let db_filter = warp::any().map(move || self.db.clone());

        let insert = warp::path!("v2" / "vectordb" / "entities" / "insert")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_insert(db, &catalog, body).await
            });

        let upsert = warp::path!("v2" / "vectordb" / "entities" / "upsert")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_upsert(db, &catalog, body).await
            });

        let delete = warp::path!("v2" / "vectordb" / "entities" / "delete")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_delete(db, &catalog, body).await
            });

        let get = warp::path!("v2" / "vectordb" / "entities" / "get")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_get(db, &catalog, body).await
            });

        let query = warp::path!("v2" / "vectordb" / "entities" / "query")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_query(db, &catalog, body).await
            });

        let search = warp::path!("v2" / "vectordb" / "entities" / "search")
            .and(warp::post())
            .and(auth.clone())
            .and(db_filter.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|db: Arc<Mutex<&'static mut RemDb>>, catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_search(db, &catalog, body).await
            });

        let create_index = warp::path!("v2" / "vectordb" / "indexes" / "create")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_create_index(&catalog, body).await
            });

        let drop_index = warp::path!("v2" / "vectordb" / "indexes" / "drop")
            .and(warp::post())
            .and(auth.clone())
            .and(catalog_filter.clone())
            .and(warp::body::json())
            .then(|catalog: Arc<MilvusCatalog>, body| async move {
                handler::handle_drop_index(&catalog, body).await
            });

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
