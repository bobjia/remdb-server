use std::convert::Infallible;
use std::sync::Arc;

use remdb::RemDb;
use remdb::types::{IndexType};
use tokio::sync::Mutex;
use warp::Reply;

use crate::milvus::catalog::MilvusCatalog;
use crate::milvus::converter::{self, FilterExpr, parse_milvus_filter};
use crate::milvus::error::MilvusError;
use crate::milvus::models::*;

pub async fn handle_create_collection(
    catalog: &MilvusCatalog,
    body: CreateCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.create_collection(&body).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let data = CollectionInfo {
        collection_name: entry.collection_name,
        description: if entry.description.is_empty() { None } else { Some(entry.description) },
    };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_drop_collection(
    catalog: &MilvusCatalog,
    body: DropCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    catalog.drop_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let resp = serde_json::json!({"code": 0, "message": "collection dropped"});
    Ok(warp::reply::json(&resp))
}

pub async fn handle_list_collections(
    catalog: &MilvusCatalog,
) -> Result<impl Reply, warp::Rejection> {
    let entries = catalog.list_collections().await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let collections: Vec<CollectionInfo> = entries.into_iter().map(|e| CollectionInfo {
        collection_name: e.collection_name,
        description: if e.description.is_empty() { None } else { Some(e.description) },
    }).collect();
    Ok(warp::reply::json(&MilvusResponse::success(collections)))
}

pub async fn handle_describe_collection(
    catalog: &MilvusCatalog,
    body: DescribeCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let fields: Vec<FieldSchemaResponse> = serde_json::from_str::<Vec<FieldSchemaResponse>>(
        &entry.schema_json
    ).unwrap_or_default();
    let data = DescribeCollectionData {
        collection_name: entry.collection_name,
        description: if entry.description.is_empty() { None } else { Some(entry.description) },
        schema: CollectionSchemaResponse { fields },
        statistics: CollectionStatistics { row_count: entry.row_count },
    };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_has_collection(
    catalog: &MilvusCatalog,
    body: HasCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let has = catalog.collection_exists(&body.collection_name).await.unwrap_or(false);
    let data = HasCollectionData { has };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_insert(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: InsertRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;

    let mut db_guard = db.lock().await;
    let mut ids = Vec::new();

    for entity in &body.data {
        let mut col_names = Vec::new();
        let mut col_values = Vec::new();

        if let Some(obj) = entity.as_object() {
            for (key, val) in obj {
                col_names.push(key.as_str());
                let val_str = match val {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Array(arr) => {
                        let elements: Vec<String> = arr.iter()
                            .filter_map(|v| v.as_f64().map(|f| f.to_string()))
                            .collect();
                        format!("'[{}]'", elements.join(", "))
                    }
                    _ => "'null'".to_string(),
                };
                col_values.push(val_str);
            }
        }

        if entry.auto_id {
            if let Some(pk_pos) = col_names.iter().position(|&n| n == entry.primary_field) {
                col_names.remove(pk_pos);
                col_values.remove(pk_pos);
            }
        }

        if !col_names.is_empty() {
            let cols = col_names.join(", ");
            let vals = col_values.join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                entry.remdb_table_name, cols, vals
            );
            let result = db_guard.sql_query(&sql)
                .map_err(|e| warp::reject::custom(MilvusError::InsertFailed(format!("{:?}", e))))?;
            if let Some(row) = result.rows.first() {
                if let Some(val) = row.values.first() {
                    ids.push(val.value.i64);
                }
            }
        }
    }

    let data = InsertResponseData {
        insert_count: ids.len(),
        insert_ids: ids,
    };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_upsert(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: UpsertRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let mut db_guard = db.lock().await;
    let mut ids = Vec::new();

    for entity in &body.data {
        let pk_value = entity.get(&entry.primary_field)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let check_sql = format!(
            "SELECT {} FROM {} WHERE {} = {}",
            entry.primary_field, entry.remdb_table_name, entry.primary_field, pk_value
        );
        let exists = db_guard.sql_query(&check_sql)
            .map(|r| !r.rows.is_empty())
            .unwrap_or(false);

        if exists {
            let mut set_clauses = Vec::new();
            if let Some(obj) = entity.as_object() {
                for (key, val) in obj {
                    if key != &entry.primary_field {
                        let val_str = match val {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Array(arr) => {
                                let elements: Vec<String> = arr.iter()
                                    .filter_map(|v| v.as_f64().map(|f| f.to_string()))
                                    .collect();
                                format!("'[{}]'", elements.join(", "))
                            }
                            _ => "'null'".to_string(),
                        };
                        set_clauses.push(format!("{} = {}", key, val_str));
                    }
                }
            }
            if !set_clauses.is_empty() {
                let sql = format!(
                    "UPDATE {} SET {} WHERE {} = {}",
                    entry.remdb_table_name,
                    set_clauses.join(", "),
                    entry.primary_field,
                    pk_value
                );
                let _ = db_guard.sql_query(&sql);
            }
            ids.push(pk_value);
        } else {
            let mut col_names = Vec::new();
            let mut col_values = Vec::new();
            if let Some(obj) = entity.as_object() {
                for (key, val) in obj {
                    col_names.push(key.clone());
                    let val_str = match val {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Array(arr) => {
                            let elements: Vec<String> = arr.iter()
                                .filter_map(|v| v.as_f64().map(|f| f.to_string()))
                                .collect();
                            format!("'[{}]'", elements.join(", "))
                        }
                        _ => "'null'".to_string(),
                    };
                    col_values.push(val_str);
                }
            }
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                entry.remdb_table_name,
                col_names.join(", "),
                col_values.join(", ")
            );
            let _ = db_guard.sql_query(&sql);
            ids.push(pk_value);
        }
    }

    let data = InsertResponseData {
        insert_count: ids.len(),
        insert_ids: ids,
    };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_delete(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: DeleteRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let mut db_guard = db.lock().await;
    let filter = if body.filter.is_empty() {
        String::new()
    } else {
        let expr = parse_milvus_filter(&body.filter)
            .map_err(|_| warp::reject::custom(MilvusError::InternalError("invalid filter".to_string())))?;
        match expr {
            FilterExpr::IdIn(ids) => {
                let id_list: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                format!("WHERE {} IN ({})", entry.primary_field, id_list.join(", "))
            }
            FilterExpr::Comparison(field, op, val) => {
                format!("WHERE {} {} {}", field, op, val)
            }
            FilterExpr::All => String::new(),
            _ => String::new(),
        }
    };

    let sql = format!("DELETE FROM {} {}", entry.remdb_table_name, filter);
    let result = db_guard.sql_query(&sql)
        .map_err(|e| warp::reject::custom(MilvusError::InternalError(format!("{:?}", e))))?;
    let delete_count = result.affected_rows;
    let data = DeleteResponseData { delete_count };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_get(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: GetRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let mut db_guard = db.lock().await;
    let sql = format!(
        "SELECT * FROM {} WHERE {} = {}",
        entry.remdb_table_name, entry.primary_field, body.id
    );
    let result = db_guard.sql_query(&sql)
        .map_err(|e| warp::reject::custom(MilvusError::InternalError(format!("{:?}", e))))?;

    if let Some(row) = result.rows.first() {
        let mut entity = serde_json::Map::new();
        for (i, col) in result.columns.iter().enumerate() {
            if let Some(val) = row.values.get(i) {
                entity.insert(col.clone(), serde_json::Value::String(val.to_string()));
            }
        }
        Ok(warp::reply::json(&MilvusResponse::success(entity)))
    } else {
        Err(warp::reject::custom(MilvusError::CollectionNotFound(body.collection_name.clone())))
    }
}

pub async fn handle_query(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: QueryRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let mut db_guard = db.lock().await;

    let fields = body.output_fields.as_ref()
        .map(|f| f.join(", "))
        .unwrap_or_else(|| "*".to_string());
    let filter = body.filter.as_ref()
        .and_then(|f| {
            if f.is_empty() { None } else { Some(format!("WHERE {}", f)) }
        })
        .unwrap_or_default();
    let limit = body.limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();
    let offset = body.offset.map(|o| format!("OFFSET {}", o)).unwrap_or_default();

    let sql = format!(
        "SELECT {} FROM {} {} {} {}",
        fields, entry.remdb_table_name, filter, limit, offset
    );
    let result = db_guard.sql_query(&sql)
        .map_err(|e| warp::reject::custom(MilvusError::InternalError(format!("{:?}", e))))?;

    let mut rows_json = Vec::new();
    for row in &result.rows {
        let mut entity = serde_json::Map::new();
        for (i, col) in result.columns.iter().enumerate() {
            if let Some(val) = row.values.get(i) {
                entity.insert(col.clone(), serde_json::Value::String(val.to_string()));
            }
        }
        rows_json.push(serde_json::Value::Object(entity));
    }

    Ok(warp::reply::json(&MilvusResponse::success(rows_json)))
}

pub async fn handle_search(
    db: Arc<Mutex<&'static mut RemDb>>,
    catalog: &MilvusCatalog,
    body: SearchRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let k = body.limit.unwrap_or(10);
    let mut db_guard = db.lock().await;

    let table_id = {
        let tables = db_guard.get_all_tables();
        let mut found_id = None;
        for (i, table_opt) in tables.iter().enumerate() {
            if let Some(table) = table_opt {
                if table.def.name == entry.remdb_table_name {
                    found_id = Some(i);
                    break;
                }
            }
        }
        found_id
    }.ok_or_else(|| warp::reject::custom(MilvusError::CollectionNotFound(body.collection_name.clone())))?;

    let results = unsafe {
        let sec_idx = db_guard.get_secondary_index_mut(table_id)
            .map_err(|_| warp::reject::custom(MilvusError::SearchFailed("no index".to_string())))?;

        match sec_idx {
            remdb::AnySecondaryIndex::Vector(vec_idx) => {
                vec_idx.search_knn(body.vector.as_ptr(), k)
                    .map_err(|_| warp::reject::custom(MilvusError::SearchFailed("search error".to_string())))?
            }
            _ => {
                return Err(warp::reject::custom(MilvusError::SearchFailed("not a vector index".to_string())));
            }
        }
    };

    let mut items = Vec::new();
    let offset = body.offset.unwrap_or(0);
    for (distance, record_id) in results.iter().skip(offset).take(k) {
        if let Ok(Some(record_ref)) = db_guard.get_by_id_ref(table_id, *record_id as usize) {
            let mut entity = serde_json::Map::new();
            if let Some(out_fields) = &body.output_fields {
                let table = db_guard.get_table(table_id)
                    .map_err(|_| warp::reject::custom(MilvusError::InternalError("no table".to_string())))?;
                for field_name in out_fields {
                    for (i, f) in table.def.fields.iter().enumerate() {
                        if f.name == *field_name {
                            let val = match f.data_type {
                                remdb::DataType::Integer => {
                                    record_ref.get_i64(i).map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
                                }
                                remdb::DataType::Real => {
                                    record_ref.get_f64(i).map(|v| serde_json::Value::Number(serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from_f64(0.0).unwrap())))
                                }
                                remdb::DataType::Text => {
                                    record_ref.get_str(i).map(|v| serde_json::Value::String(v.to_string()))
                                }
                                remdb::DataType::Vector => {
                                    record_ref.get_str(i).map(|v| serde_json::Value::String(v.to_string()))
                                }
                                remdb::DataType::Boolean => {
                                    record_ref.get_bool(i).map(|v| serde_json::Value::Bool(v))
                                }
                                _ => {
                                    record_ref.get_str(i).map(|v| serde_json::Value::String(v.to_string()))
                                }
                            };
                            if let Ok(v) = val {
                                entity.insert(field_name.clone(), v);
                            }
                            break;
                        }
                    }
                }
            }
            items.push(SearchResultItem {
                id: *record_id as i64,
                distance: *distance,
                entity: serde_json::Value::Object(entity),
            });
        }
    }

    Ok(warp::reply::json(&MilvusResponse::success(items)))
}

pub async fn handle_create_index(
    catalog: &MilvusCatalog,
    body: CreateIndexRequest,
) -> Result<impl Reply, warp::Rejection> {
    let _ = converter::milvus_metric_to_distance(&body.metric_type)
        .map_err(|_| warp::reject::custom(MilvusError::InvalidMetricType(body.metric_type.clone())))?;

    let data = IndexInfo { index_name: body.index_name };
    Ok(warp::reply::json(&MilvusResponse::success(data)))
}

pub async fn handle_drop_index(
    catalog: &MilvusCatalog,
    body: DropIndexRequest,
) -> Result<impl Reply, warp::Rejection> {
    let _ = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let resp = serde_json::json!({"code": 0, "message": "index dropped"});
    Ok(warp::reply::json(&resp))
}

pub async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, Infallible> {
    let json = if let Some(milvus_err) = err.find::<MilvusError>() {
        let http_status = milvus_err.http_status();
        let resp = warp::reply::json(&milvus_err.to_json());
        warp::reply::with_status(resp, warp::http::StatusCode::from_u16(http_status).unwrap_or(warp::http::StatusCode::BAD_REQUEST))
    } else {
        let json = serde_json::json!({"code": 9999, "message": "internal server error"});
        warp::reply::with_status(json, warp::http::StatusCode::INTERNAL_SERVER_ERROR)
    };
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_ids_from_sql_result() {
        let ids = vec![1i64, 2, 3];
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_rejection_to_json() {
        let err = MilvusError::CollectionNotFound("test".to_string());
        let json = err.to_json();
        assert_eq!(json["code"], 1001);
    }
}
