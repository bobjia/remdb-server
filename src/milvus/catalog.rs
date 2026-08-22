use std::collections::HashMap;
use std::result::Result;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use remdb::types::{DataType, DistanceType, IndexType, Value};
use remdb::RemDb;
use tokio::sync::RwLock;

use crate::milvus::converter;
use crate::milvus::error::MilvusError;
use crate::milvus::models;

/// Catalog entry describing a Milvus collection
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub collection_id: i64,
    pub collection_name: String,
    pub description: String,
    pub schema_json: String,
    pub primary_field: String,
    pub vector_field: String,
    pub auto_id: bool,
    pub dimension: u16,
    pub metric_type: String,
    pub index_type: String,
    pub index_params: String,
    pub remdb_table_name: String,
    pub created_at: i64,
    pub row_count: usize,
}

/// System table name for catalog
const CATALOG_TABLE: &str = "_milvus_catalog";

/// Generate the data table name for a collection
pub fn data_table_name(collection_id: i64) -> String {
    format!("_milvus_coll_{}", collection_id)
}

/// Field indices for the _milvus_catalog table
const CAT_COL_COLLECTION_ID: usize = 0;
const CAT_COL_COLLECTION_NAME: usize = 1;
const CAT_COL_DESCRIPTION: usize = 2;
const CAT_COL_SCHEMA_JSON: usize = 3;
const CAT_COL_PRIMARY_FIELD: usize = 4;
const CAT_COL_VECTOR_FIELD: usize = 5;
const CAT_COL_AUTO_ID: usize = 6;
const CAT_COL_DIMENSION: usize = 7;
const CAT_COL_METRIC_TYPE: usize = 8;
const CAT_COL_INDEX_TYPE: usize = 9;
const CAT_COL_INDEX_PARAMS: usize = 10;
const CAT_COL_REMDB_TABLE_NAME: usize = 11;
const CAT_COL_CREATED_AT: usize = 12;
const CAT_COL_ROW_COUNT: usize = 13;

/// Collection catalog managing Milvus collection metadata
pub struct MilvusCatalog {
    db: Arc<Mutex<&'static mut RemDb>>,
    /// In-memory cache of collection_name → CatalogEntry
    cache: tokio::sync::RwLock<HashMap<String, CatalogEntry>>,
    /// Atomically incrementing counter for collection IDs
    next_id: AtomicI64,
}

impl MilvusCatalog {
    /// Get a reference to the database mutex
    pub fn db(&self) -> Arc<Mutex<&'static mut RemDb>> {
        Arc::clone(&self.db)
    }

    pub fn new(db: Arc<Mutex<&'static mut RemDb>>) -> Self {
        MilvusCatalog {
            db,
            cache: tokio::sync::RwLock::new(HashMap::new()),
            next_id: AtomicI64::new(1),
        }
    }

    /// Initialize the catalog system table if it doesn't exist
    pub async fn init(&self) -> Result<(), MilvusError> {
        // Create the catalog table if it doesn't exist
        let catalog_fields: &[(&str, DataType, u16, Option<DistanceType>, Option<Value>)] = &[
            ("collection_id", DataType::Int64, 0, None, None),
            ("collection_name", DataType::VarChar, 64, None, None),
            ("description", DataType::VarChar, 256, None, None),
            ("schema_json", DataType::VarChar, 4096, None, None),
            ("primary_field", DataType::VarChar, 64, None, None),
            ("vector_field", DataType::VarChar, 64, None, None),
            ("auto_id", DataType::Bool, 0, None, None),
            ("dimension", DataType::Int64, 0, None, None),
            ("metric_type", DataType::VarChar, 32, None, None),
            ("index_type", DataType::VarChar, 32, None, None),
            ("index_params", DataType::VarChar, 1024, None, None),
            ("remdb_table_name", DataType::VarChar, 64, None, None),
            ("created_at", DataType::Int64, 0, None, None),
            ("row_count", DataType::Int64, 0, None, None),
        ];
        {
            let mut db = self.db.lock().map_err(|_| {
                MilvusError::InternalError("database lock poisoned".to_string())
            })?;
            match db.create_table(CATALOG_TABLE, catalog_fields, Some(vec![0])) {
                Ok(()) => {}
                Err(_) => {
                    // Catalog table already exists (e.g., re-initialization);
                    // refresh_cache below will load the existing entries.
                }
            }
        }
        // Refresh cache
        self.refresh_cache().await;
        Ok(())
    }

    /// Create a new Milvus collection
    pub async fn create_collection(
        &self,
        req: &models::CreateCollectionRequest,
    ) -> Result<CatalogEntry, MilvusError> {
        // 1. Validate schema
        let fields = &req.schema.fields;
        let mut primary_field = None;
        let mut vector_field = None;
        let mut auto_id = req.schema.auto_id.unwrap_or(false);
        let mut dimension = 0u16;

        for f in fields {
            if f.field_type == "FloatVector" {
                vector_field = Some(f.name.clone());
                dimension = converter::parse_vector_dim(&f.params)
                    .map_err(|_| MilvusError::InvalidDimension("missing dim".to_string()))?;
            }
            if f.is_primary.unwrap_or(false) {
                primary_field = Some(f.name.clone());
                if f.auto_id.unwrap_or(false) {
                    auto_id = true;
                }
            }
        }

        let primary_field = primary_field
            .ok_or_else(|| MilvusError::InvalidSchema("no primary key field".to_string()))?;
        let vector_field = vector_field
            .ok_or_else(|| MilvusError::InvalidSchema("no vector field".to_string()))?;

        if dimension == 0 || dimension > 1024 {
            return Err(MilvusError::InvalidDimension("dim must be 1-1024".to_string()));
        }

        // 2. Check for duplicate
        if self.collection_exists(&req.collection_name).await {
            return Err(MilvusError::DuplicateCollection(req.collection_name.clone()));
        }

        // 3. Get next collection_id
        let collection_id = self.next_collection_id();
        let remdb_table = data_table_name(collection_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // 4. Create remdb table for this collection (synchronous, no await)
        let entry = {
            let mut db = self.db.lock().map_err(|_| {
                MilvusError::InternalError("database lock poisoned".to_string())
            })?;

            let mut remdb_fields: Vec<(&str, DataType, u16, Option<DistanceType>, Option<Value>)> =
                Vec::new();

            for f in fields {
                let dt = converter::milvus_type_to_remdb(&f.field_type)
                    .map_err(|_| MilvusError::InvalidSchema(format!("unknown type: {}", f.field_type)))?;

                let (size, dist) = if f.field_type == "FloatVector" {
                    (dimension, Some(DistanceType::L2))
                } else if dt == DataType::VarChar {
                    let max_len = f.params.as_ref().and_then(|p| p.max_length).unwrap_or(256) as u16;
                    (max_len, None)
                } else {
                    (0, None)
                };

                remdb_fields.push((f.name.as_str(), dt, size, dist, None));
            }

            // Default metric type from index params
            let metric_type = req.index_params.as_ref()
                .and_then(|params| params.first())
                .map(|p| p.metric_type.clone())
                .unwrap_or_else(|| "L2".to_string());

            let index_type_str = req.index_params.as_ref()
                .and_then(|params| params.first())
                .and_then(|p| p.params.as_ref())
                .and_then(|p| p.index_type.clone())
                .unwrap_or_else(|| "HNSW".to_string());

            let index_params_json = req.index_params.as_ref()
                .and_then(|params| params.first())
                .map(|p| serde_json::to_string(p).unwrap_or_default())
                .unwrap_or_default();

            // Create the table
            db.create_table(&remdb_table, &remdb_fields, Some(vec![0]))
                .map_err(|e| MilvusError::InternalError(format!("create table: {:?}", e)))?;

            // Create vector index on the vector field
            db.create_index(&remdb_table, &vector_field, IndexType::Vector)
                .map_err(|e| {
                    MilvusError::InternalError(format!("create vector index: {:?}", e))
                })?;

            // 5. Insert into catalog table using SQL
            let schema_json = serde_json::to_string(&req.schema).unwrap_or_default();
            let auto_id_int = if auto_id { 1 } else { 0 };
            let sql = format!(
                "INSERT INTO {} (collection_id, collection_name, description, schema_json, \
                 primary_field, vector_field, auto_id, dimension, metric_type, index_type, \
                 index_params, remdb_table_name, created_at, row_count) \
                 VALUES ({}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', {}, 0)",
                CATALOG_TABLE,
                collection_id,
                escape_sql_string(&req.collection_name),
                escape_sql_string(req.description.as_deref().unwrap_or("")),
                escape_sql_string(&schema_json),
                escape_sql_string(&primary_field),
                escape_sql_string(&vector_field),
                auto_id_int,
                dimension,
                escape_sql_string(&metric_type),
                escape_sql_string(&index_type_str),
                escape_sql_string(&index_params_json),
                escape_sql_string(&remdb_table),
                now,
            );
            db.sql_query(&sql).map_err(|e| {
                MilvusError::InternalError(format!("catalog insert: {:?}", e))
            })?;

            CatalogEntry {
                collection_id,
                collection_name: req.collection_name.clone(),
                description: req.description.clone().unwrap_or_default(),
                schema_json,
                primary_field,
                vector_field,
                auto_id,
                dimension,
                metric_type,
                index_type: index_type_str,
                index_params: index_params_json,
                remdb_table_name: remdb_table,
                created_at: now,
                row_count: 0,
            }
        }; // db lock is dropped here

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(req.collection_name.clone(), entry.clone());

        Ok(entry)
    }

    /// Drop a collection
    pub async fn drop_collection(&self, name: &str) -> Result<(), MilvusError> {
        let entry = self.resolve_collection(name).await?;
        {
            let mut db = self.db.lock().map_err(|_| {
                MilvusError::InternalError("database lock poisoned".to_string())
            })?;
            // Drop the data table
            db.drop_table(&entry.remdb_table_name, true, false).map_err(|e| {
                MilvusError::InternalError(format!("drop table: {:?}", e))
            })?;
            // Remove from catalog using the numeric primary key for reliable matching
            let sql = format!(
                "DELETE FROM {} WHERE collection_id = {}",
                CATALOG_TABLE,
                entry.collection_id
            );
            db.sql_query(&sql).map_err(|e| {
                MilvusError::InternalError(format!("catalog delete: {:?}", e))
            })?;
        }
        // Update cache
        let mut cache = self.cache.write().await;
        cache.remove(name);
        Ok(())
    }

    /// Resolve a collection name to its catalog entry
    pub async fn resolve_collection(&self, name: &str) -> Result<CatalogEntry, MilvusError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name) {
                return Ok(entry.clone());
            }
        }
        // Fall back to querying the catalog table
        let entry_opt = {
            let mut db = self.db.lock().map_err(|_| {
                MilvusError::InternalError("database lock poisoned".to_string())
            })?;
            let sql = format!(
                "SELECT * FROM {} WHERE collection_name = '{}'",
                CATALOG_TABLE,
                escape_sql_string(name)
            );
            let result = db.sql_query(&sql)
                .map_err(|_| MilvusError::CollectionNotFound(name.to_string()))?;
            // Parse the first row
            result.rows.first().map(|row| parse_catalog_row(row, &result.columns))
        }; // db lock is dropped here
        if let Some(entry) = entry_opt {
            // Update cache
            let mut cache = self.cache.write().await;
            cache.insert(name.to_string(), entry.clone());
            Ok(entry)
        } else {
            Err(MilvusError::CollectionNotFound(name.to_string()))
        }
    }

    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<CatalogEntry>, MilvusError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    /// Check if a collection exists
    pub async fn collection_exists(&self, name: &str) -> bool {
        let cache = self.cache.read().await;
        cache.contains_key(name)
    }

    /// Get the next collection ID
    fn next_collection_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Refresh the in-memory cache from the catalog table
    async fn refresh_cache(&self) {
        let entries = {
            let mut db_lock = match self.db.lock() {
                Ok(guard) => guard,
                Err(_) => return, // lock poisoned; keep current cache
            };
            let sql = format!("SELECT * FROM {}", CATALOG_TABLE);
            match db_lock.sql_query(&sql) {
                Ok(result) => {
                    let mut entries = Vec::new();
                    for row in &result.rows {
                        let entry = parse_catalog_row(row, &result.columns);
                        entries.push(entry);
                    }
                    entries
                }
                Err(_) => Vec::new(),
            }
        }; // db lock is dropped here
        let mut cache = self.cache.write().await;
        cache.clear();
        let mut max_id = 0i64;
        for entry in entries {
            if entry.collection_id > max_id {
                max_id = entry.collection_id;
            }
            cache.insert(entry.collection_name.clone(), entry);
        }
        // Ensure next_id is at least max_id + 1
        let target = max_id + 1;
        let current = self.next_id.load(Ordering::SeqCst);
        if target > current {
            self.next_id.store(target, Ordering::SeqCst);
        }
    }
}

/// Parse a catalog row from the ResultSet
fn parse_catalog_row(row: &remdb::sql::ResultRow, _columns: &[String]) -> CatalogEntry {
    // Helper to read string from a TypedValue
    let read_string = |idx: usize| -> String {
        if let Some(val) = row.values.get(idx) {
            unsafe {
                let bytes = &val.value.string;
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                String::from_utf8_lossy(&bytes[..end]).to_string()
            }
        } else {
            String::new()
        }
    };

    // Helper to read i64 from a TypedValue
    let read_i64 = |idx: usize| -> i64 {
        if let Some(val) = row.values.get(idx) {
            unsafe { val.value.i64 }
        } else {
            0
        }
    };

    // Helper to read bool from a TypedValue
    let read_bool = |idx: usize| -> bool {
        if let Some(val) = row.values.get(idx) {
            unsafe { val.value.bool }
        } else {
            false
        }
    };

    let collection_id = read_i64(CAT_COL_COLLECTION_ID);
    let collection_name = read_string(CAT_COL_COLLECTION_NAME);
    let description = read_string(CAT_COL_DESCRIPTION);
    let schema_json = read_string(CAT_COL_SCHEMA_JSON);
    let primary_field = read_string(CAT_COL_PRIMARY_FIELD);
    let vector_field = read_string(CAT_COL_VECTOR_FIELD);
    let auto_id = read_bool(CAT_COL_AUTO_ID);
    let dimension = read_i64(CAT_COL_DIMENSION) as u16;
    let metric_type = read_string(CAT_COL_METRIC_TYPE);
    let index_type = read_string(CAT_COL_INDEX_TYPE);
    let index_params = read_string(CAT_COL_INDEX_PARAMS);
    let remdb_table_name = read_string(CAT_COL_REMDB_TABLE_NAME);
    let created_at = read_i64(CAT_COL_CREATED_AT);
    let row_count = read_i64(CAT_COL_ROW_COUNT) as usize;

    CatalogEntry {
        collection_id,
        collection_name,
        description,
        schema_json,
        primary_field,
        vector_field,
        auto_id,
        dimension,
        metric_type,
        index_type,
        index_params,
        remdb_table_name,
        created_at,
        row_count,
    }
}

/// Escape a string for SQL (single quote escaping)
fn escape_sql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_table_name() {
        assert_eq!(data_table_name(1), "_milvus_coll_1");
        assert_eq!(data_table_name(42), "_milvus_coll_42");
        assert_eq!(data_table_name(0), "_milvus_coll_0");
    }

    #[test]
    fn test_escape_sql_string_no_quotes() {
        assert_eq!(escape_sql_string("hello"), "hello");
    }

    #[test]
    fn test_escape_sql_string_with_quotes() {
        assert_eq!(escape_sql_string("it's"), "it''s");
    }

    #[test]
    fn test_escape_sql_string_multiple_quotes() {
        assert_eq!(escape_sql_string("a'b'c"), "a''b''c");
    }

    #[test]
    fn test_escape_sql_string_empty() {
        assert_eq!(escape_sql_string(""), "");
    }

    #[test]
    fn test_data_table_name_large_id() {
        assert_eq!(data_table_name(999999), "_milvus_coll_999999");
    }

    #[test]
    fn test_data_table_name_negative() {
        assert_eq!(data_table_name(-1), "_milvus_coll_-1");
    }
}