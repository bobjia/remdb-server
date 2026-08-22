use remdb::types::*;
use remdb::{DdlExecutor, RemDb};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::milvus::converter;
use crate::milvus::error::MilvusError;
use crate::milvus::models;

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

const CATALOG_TABLE: &str = "_milvus_catalog";

pub fn data_table_name(collection_id: i64) -> String {
    format!("_milvus_coll_{}", collection_id)
}

pub struct MilvusCatalog {
    db: Arc<Mutex<&'static mut RemDb>>,
    cache: tokio::sync::RwLock<HashMap<String, CatalogEntry>>,
}

impl MilvusCatalog {
    pub fn new(db: Arc<Mutex<&'static mut RemDb>>) -> Self {
        MilvusCatalog {
            db,
            cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn init(&self) -> Result<(), MilvusError> {
        let mut db = self.db.lock().await;
        let fields = [
            ("collection_id", DataType::Integer, 0u16, None, None),
            ("collection_name", DataType::Text, 64u16, None, None),
            ("description", DataType::Text, 256u16, None, None),
            ("schema_json", DataType::Text, 4096u16, None, None),
            ("primary_field", DataType::Text, 64u16, None, None),
            ("vector_field", DataType::Text, 64u16, None, None),
            ("auto_id", DataType::Boolean, 0u16, None, None),
            ("dimension", DataType::Integer, 0u16, None, None),
            ("metric_type", DataType::Text, 32u16, None, None),
            ("index_type", DataType::Text, 32u16, None, None),
            ("index_params", DataType::Text, 1024u16, None, None),
            ("remdb_table_name", DataType::Text, 64u16, None, None),
            ("created_at", DataType::Integer, 0u16, None, None),
            ("row_count", DataType::Integer, 0u16, None, None),
        ];
        let _ = db.create_table(CATALOG_TABLE, &fields, Some(vec![0]));
        self.refresh_cache().await;
        Ok(())
    }

    pub async fn create_collection(
        &self,
        req: &models::CreateCollectionRequest,
    ) -> Result<CatalogEntry, MilvusError> {
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

        if self.collection_exists(&req.collection_name).await? {
            return Err(MilvusError::DuplicateCollection(req.collection_name.clone()));
        }

        let collection_id = self.next_collection_id().await;
        let remdb_table = data_table_name(collection_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut db = self.db.lock().await;
        let mut remdb_fields: Vec<(&str, DataType, u16, Option<DistanceType>, Option<Value>)> =
            Vec::new();

        for f in fields {
            let dt = converter::milvus_type_to_remdb(&f.field_type)
                .map_err(|_| MilvusError::InvalidSchema(format!("unknown type: {}", f.field_type)))?;

            let (size, dist) = if f.field_type == "FloatVector" {
                (dimension as u16, Some(DistanceType::L2))
            } else if dt == DataType::Text {
                let max_len = f.params.as_ref().and_then(|p| p.max_length).unwrap_or(256) as u16;
                (max_len, None)
            } else {
                (0, None)
            };

            remdb_fields.push((f.name.as_str(), dt, size, dist, None));
        }

        let metric_type = req.index_params.as_ref()
            .and_then(|params| params.first())
            .map(|p| p.metric_type.clone())
            .unwrap_or_else(|| "L2".to_string());

        let index_type = req.index_params.as_ref()
            .and_then(|params| params.first())
            .and_then(|p| p.params.as_ref())
            .and_then(|p| p.index_type.clone())
            .unwrap_or_else(|| "HNSW".to_string());

        let index_params_json = req.index_params.as_ref()
            .and_then(|params| params.first())
            .map(|p| serde_json::to_string(p).unwrap_or_default())
            .unwrap_or_default();

        db.create_table(&remdb_table, &remdb_fields, Some(vec![0]))
            .map_err(|e| MilvusError::InternalError(format!("create table: {:?}", e)))?;

        if let Ok(v_idx) = converter::milvus_index_to_vector_index(&index_type) {
            let remdb_idx_type = match v_idx {
                VectorIndexType::HNSW => IndexType::Vector,
                VectorIndexType::IVF => IndexType::Vector,
                VectorIndexType::IVF_PQ => IndexType::Vector,
                _ => IndexType::Vector,
            };
            let _ = db.create_index(&remdb_table, &vector_field, remdb_idx_type);
        }

        let schema_json = serde_json::to_string(&req.schema).unwrap_or_default();
        let sql = format!(
            "INSERT INTO {} (collection_id, collection_name, description, schema_json, \
             primary_field, vector_field, auto_id, dimension, metric_type, index_type, \
             index_params, remdb_table_name, created_at, row_count) \
             VALUES ({}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', {}, 0)",
            CATALOG_TABLE,
            collection_id,
            req.collection_name.replace('\'', "''"),
            req.description.as_deref().unwrap_or("").replace('\'', "''"),
            schema_json.replace('\'', "''"),
            primary_field.replace('\'', "''"),
            vector_field.replace('\'', "''"),
            if auto_id { 1 } else { 0 },
            dimension,
            metric_type.replace('\'', "''"),
            index_type.replace('\'', "''"),
            index_params_json.replace('\'', "''"),
            remdb_table.replace('\'', "''"),
            now,
        );
        let _ = db.sql_query(&sql);

        let entry = CatalogEntry {
            collection_id,
            collection_name: req.collection_name.clone(),
            description: req.description.clone().unwrap_or_default(),
            schema_json,
            primary_field,
            vector_field,
            auto_id,
            dimension,
            metric_type,
            index_type,
            index_params: index_params_json,
            remdb_table_name: remdb_table,
            created_at: now,
            row_count: 0,
        };

        {
            let mut cache = self.cache.write().await;
            cache.insert(req.collection_name.clone(), entry.clone());
        }

        Ok(entry)
    }

    pub async fn drop_collection(&self, name: &str) -> Result<(), MilvusError> {
        let entry = self.resolve_collection(name).await?;
        let mut db = self.db.lock().await;
        let _ = db.drop_table(&entry.remdb_table_name, true, false);
        let sql = format!("DELETE FROM {} WHERE collection_name = '{}'", CATALOG_TABLE, name.replace('\'', "''"));
        let _ = db.sql_query(&sql);
        let mut cache = self.cache.write().await;
        cache.remove(name);
        Ok(())
    }

    pub async fn resolve_collection(&self, name: &str) -> Result<CatalogEntry, MilvusError> {
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(name) {
                return Ok(entry.clone());
            }
        }
        let mut db = self.db.lock().await;
        let sql = format!("SELECT * FROM {} WHERE collection_name = '{}'", CATALOG_TABLE, name.replace('\'', "''"));
        let result = db.sql_query(&sql).map_err(|_| MilvusError::CollectionNotFound(name.to_string()))?;
        if let Some(row) = result.rows.first() {
            if row.values.len() >= 14 {
                let entry = CatalogEntry {
                    collection_id: row.values[0].value.i64,
                    collection_name: row.values[1].to_string(),
                    description: row.values[2].to_string(),
                    schema_json: row.values[3].to_string(),
                    primary_field: row.values[4].to_string(),
                    vector_field: row.values[5].to_string(),
                    auto_id: row.values[6].value.u64 != 0,
                    dimension: row.values[7].value.u64 as u16,
                    metric_type: row.values[8].to_string(),
                    index_type: row.values[9].to_string(),
                    index_params: row.values[10].to_string(),
                    remdb_table_name: row.values[11].to_string(),
                    created_at: row.values[12].value.i64,
                    row_count: row.values[13].value.u64 as usize,
                };
                let mut cache = self.cache.write().await;
                cache.insert(entry.collection_name.clone(), entry.clone());
                return Ok(entry);
            }
        }
        Err(MilvusError::CollectionNotFound(name.to_string()))
    }

    pub async fn list_collections(&self) -> Result<Vec<CatalogEntry>, MilvusError> {
        let cache = self.cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    pub async fn collection_exists(&self, name: &str) -> Result<bool, MilvusError> {
        let cache = self.cache.read().await;
        Ok(cache.contains_key(name))
    }

    pub async fn next_collection_id(&self) -> i64 {
        let cache = self.cache.read().await;
        let max_id = cache.values().map(|e| e.collection_id).max().unwrap_or(0);
        max_id + 1
    }

    async fn refresh_cache(&self) {
        let mut db = self.db.lock().await;
        let sql = format!("SELECT * FROM {}", CATALOG_TABLE);
        if let Ok(result) = db.sql_query(&sql) {
            let mut cache = self.cache.write().await;
            cache.clear();
            for row in &result.rows {
                if row.values.len() >= 14 {
                    let entry = CatalogEntry {
                        collection_id: row.values[0].value.i64,
                        collection_name: row.values[1].to_string(),
                        description: row.values[2].to_string(),
                        schema_json: row.values[3].to_string(),
                        primary_field: row.values[4].to_string(),
                        vector_field: row.values[5].to_string(),
                        auto_id: row.values[6].value.u64 != 0,
                        dimension: row.values[7].value.u64 as u16,
                        metric_type: row.values[8].to_string(),
                        index_type: row.values[9].to_string(),
                        index_params: row.values[10].to_string(),
                        remdb_table_name: row.values[11].to_string(),
                        created_at: row.values[12].value.i64,
                        row_count: row.values[13].value.u64 as usize,
                    };
                    cache.insert(entry.collection_name.clone(), entry);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_table_name() {
        assert_eq!(data_table_name(1), "_milvus_coll_1");
        assert_eq!(data_table_name(42), "_milvus_coll_42");
    }

    #[test]
    fn test_catalog_table_name_format() {
        let name = data_table_name(100);
        assert!(name.starts_with("_milvus_coll_"));
        assert_eq!(name, "_milvus_coll_100");
    }
}
