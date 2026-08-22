use std::convert::Infallible;
use std::result::Result;
use std::sync::Arc;

use remdb::types::DataType;
use warp::Reply;

use crate::milvus::catalog::MilvusCatalog;
use crate::milvus::converter::{self, FilterExpr, parse_milvus_filter};
use crate::milvus::error::MilvusError;
use crate::milvus::models::*;

// ── Collection handlers ──

pub async fn handle_create_collection(
    catalog: Arc<MilvusCatalog>,
    body: CreateCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.create_collection(&body).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let data = CollectionInfo {
        collection_name: entry.collection_name,
        description: if entry.description.is_empty() { None } else { Some(entry.description) },
    };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_drop_collection(
    catalog: Arc<MilvusCatalog>,
    body: DropCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    catalog.drop_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let resp = serde_json::json!({"code": 0, "message": "collection dropped"});
    Ok(warp::reply::json(&resp))
}

pub async fn handle_list_collections(
    catalog: Arc<MilvusCatalog>,
) -> Result<impl Reply, warp::Rejection> {
    let entries = catalog.list_collections().await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let collections: Vec<CollectionInfo> = entries.into_iter().map(|e| CollectionInfo {
        collection_name: e.collection_name,
        description: if e.description.is_empty() { None } else { Some(e.description) },
    }).collect();
    let response = MilvusResponse::success(collections);
    Ok(warp::reply::json(&response))
}

pub async fn handle_describe_collection(
    catalog: Arc<MilvusCatalog>,
    body: DescribeCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    // Parse schema_json back to fields
    let schema: CollectionSchema = serde_json::from_str(&entry.schema_json).unwrap_or_else(|_| {
        CollectionSchema {
            auto_id: Some(entry.auto_id),
            description: None,
            fields: Vec::new(),
        }
    });
    let fields: Vec<FieldSchemaResponse> = schema.fields.iter().map(|f| FieldSchemaResponse {
        name: f.name.clone(),
        field_type: f.field_type.clone(),
        is_primary: f.is_primary,
        auto_id: f.auto_id,
        params: f.params.clone(),
    }).collect();
    let data = DescribeCollectionData {
        collection_name: entry.collection_name,
        description: if entry.description.is_empty() { None } else { Some(entry.description) },
        schema: CollectionSchemaResponse { fields },
        statistics: CollectionStatistics { row_count: entry.row_count },
    };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_has_collection(
    catalog: Arc<MilvusCatalog>,
    body: HasCollectionRequest,
) -> Result<impl Reply, warp::Rejection> {
    let has = catalog.collection_exists(&body.collection_name).await;
    let data = HasCollectionData { has };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

// ── Entity handlers ──

pub async fn handle_insert(
    catalog: Arc<MilvusCatalog>,
    body: InsertRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;

    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;
    let mut ids = Vec::new();

    for entity in &body.data {
        // Build column names and values from the JSON entity
        let mut col_names = Vec::new();
        let mut col_values = Vec::new();

        if let Some(obj) = entity.as_object() {
            for (key, val) in obj {
                col_names.push(key.as_str());
                let val_str = json_value_to_sql(val);
                col_values.push(val_str);
            }
        }

        // If auto_id, remove the primary key from columns
        if entry.auto_id {
            if let Some(pk_pos) = col_names.iter().position(|&n| n == entry.primary_field) {
                col_names.remove(pk_pos);
                col_values.remove(pk_pos);
            }
        }

        // Build and execute INSERT SQL
        if !col_names.is_empty() {
            let cols = col_names.join(", ");
            let vals = col_values.join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                entry.remdb_table_name, cols, vals
            );
            let result = db_guard.sql_query(&sql)
                .map_err(|e| {
                    warp::reject::custom(MilvusError::InsertFailed(format!("{:?}", e)))
                })?;

            // The INSERT result's second column carries the record slot id.
            // Note: the slot id is NOT the primary key value when auto_id is
            // enabled (auto-increment starts at max_pk + 1), so we must read
            // the generated primary key back from the stored record.
            let slot_id = result
                .rows
                .first()
                .and_then(|row| row.values.get(1))
                .map(|val| unsafe { val.value.u64 })
                .ok_or_else(|| {
                    warp::reject::custom(MilvusError::InsertFailed(
                        "insert returned no record id".to_string(),
                    ))
                })? as usize;

            // Read back the generated primary key and update the vector index
            // for the inserted record in a single borrow scope.
            let pk_value = {
                let (table, sec_idx) = db_guard
                    .get_table_and_secondary_index_mut_by_name(&entry.remdb_table_name)
                    .map_err(|_| {
                        warp::reject::custom(MilvusError::InternalError(
                            "collection table not found after insert".to_string(),
                        ))
                    })?;

                // Read the primary key value from the inserted record.
                let pk_field_index = table
                    .def
                    .fields
                    .iter()
                    .position(|f| f.name == entry.primary_field)
                    .ok_or_else(|| {
                        warp::reject::custom(MilvusError::InternalError(
                            "primary key field not found".to_string(),
                        ))
                    })?;
                let record_ptr = unsafe { table.get_record_ptr(slot_id) };
                let pk_value = unsafe { table.get_field(record_ptr, pk_field_index) }
                    .map_err(|e| {
                        warp::reject::custom(MilvusError::InsertFailed(format!(
                            "read back primary key failed: {:?}",
                            e
                        )))
                    })?;
                let pk_field_type = table
                    .def
                    .fields
                    .get(pk_field_index)
                    .map(|f| f.data_type)
                    .unwrap_or(DataType::Int64);
                let pk_value = value_to_i64(pk_value, pk_field_type);

                // Update the vector index using the collection's configured
                // vector field name (not a hard-coded key).
                if let Some(obj) = entity.as_object() {
                    if let Some(vector_val) = obj.get(&entry.vector_field) {
                        if let Some(vector_arr) = vector_val.as_array() {
                            // Convert the vector array to raw f32 bytes
                            let dim = vector_arr.len();
                            let mut key = Vec::with_capacity(dim.saturating_mul(4));
                            for val in vector_arr {
                                let f = val.as_f64().unwrap_or(0.0) as f32;
                                key.extend_from_slice(&f.to_le_bytes());
                            }

                            unsafe {
                                sec_idx
                                    .insert(key.as_ptr(), key.len(), slot_id as u16)
                                    .map_err(|e| {
                                        warp::reject::custom(MilvusError::InsertFailed(
                                            format!("{:?}", e),
                                        ))
                                    })?;
                            }
                        }
                    }
                }

                pk_value
            };

            ids.push(pk_value);
        }
    }

    let data = InsertResponseData {
        insert_count: ids.len(),
        insert_ids: ids,
    };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_upsert(
    catalog: Arc<MilvusCatalog>,
    body: UpsertRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;
    let mut ids = Vec::new();

    for entity in &body.data {
        // Extract primary key value
        let pk_value = entity.get(&entry.primary_field)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Check if record exists
        let check_sql = format!(
            "SELECT {} FROM {} WHERE {} = {}",
            entry.primary_field, entry.remdb_table_name, entry.primary_field, pk_value
        );
        let exists = match db_guard.sql_query(&check_sql) {
            Ok(r) => !r.rows.is_empty(),
            Err(e) => {
                return Err(warp::reject::custom(MilvusError::InternalError(format!(
                    "{:?}",
                    e
                ))));
            }
        };

        if exists {
            // UPDATE
            let mut set_clauses = Vec::new();
            if let Some(obj) = entity.as_object() {
                for (key, val) in obj {
                    if key != &entry.primary_field {
                        let val_str = json_value_to_sql(val);
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
                db_guard.sql_query(&sql).map_err(|e| {
                    warp::reject::custom(MilvusError::InternalError(format!("{:?}", e)))
                })?;
            }
            ids.push(pk_value);
        } else {
            // INSERT
            let mut col_names = Vec::new();
            let mut col_values = Vec::new();
            if let Some(obj) = entity.as_object() {
                for (key, val) in obj {
                    col_names.push(key.clone());
                    let val_str = json_value_to_sql(val);
                    col_values.push(val_str);
                }
            }
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                entry.remdb_table_name,
                col_names.join(", "),
                col_values.join(", ")
            );
            db_guard.sql_query(&sql).map_err(|e| {
                warp::reject::custom(MilvusError::InsertFailed(format!("{:?}", e)))
            })?;
            ids.push(pk_value);
        }
    }

    let data = InsertResponseData {
        insert_count: ids.len(),
        insert_ids: ids,
    };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_delete(
    catalog: Arc<MilvusCatalog>,
    body: DeleteRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;
    let filter = if body.filter.is_empty() {
        String::new()
    } else {
        // Convert Milvus filter to SQL WHERE clause
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
    let delete_count = result.rows.len();
    let data = DeleteResponseData { delete_count };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_get(
    catalog: Arc<MilvusCatalog>,
    body: GetRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;
    let sql = format!(
        "SELECT * FROM {} WHERE {} = {}",
        entry.remdb_table_name, entry.primary_field, body.id
    );
    let result = db_guard.sql_query(&sql)
        .map_err(|e| {
            warp::reject::custom(MilvusError::InternalError(format!("{:?}", e)))
        })?;

    if let Some(row) = result.rows.first() {
        let mut entity = serde_json::Map::new();
        for (i, col) in result.columns.iter().enumerate() {
            if let Some(val) = row.values.get(i) {
                let str_val = typed_value_to_string(val);
                entity.insert(col.clone(), serde_json::Value::String(str_val));
            }
        }
        let response = MilvusResponse::success(serde_json::Value::Object(entity));
        Ok(warp::reply::json(&response))
    } else {
        // Record not found is not an error in Milvus semantics; return an
        // empty result instead of mis-reporting the collection as missing.
        let response = MilvusResponse::success(serde_json::Value::Null);
        Ok(warp::reply::json(&response))
    }
}

pub async fn handle_query(
    catalog: Arc<MilvusCatalog>,
    body: QueryRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;

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
        .map_err(|e| {
            warp::reject::custom(MilvusError::InternalError(format!("{:?}", e)))
        })?;

    let mut rows_json = Vec::new();
    for row in &result.rows {
        let mut entity = serde_json::Map::new();
        for (i, col) in result.columns.iter().enumerate() {
            if let Some(val) = row.values.get(i) {
                let str_val = typed_value_to_string(val);
                entity.insert(col.clone(), serde_json::Value::String(str_val));
            }
        }
        rows_json.push(serde_json::Value::Object(entity));
    }

    let response = MilvusResponse::success(rows_json);
    Ok(warp::reply::json(&response))
}

pub async fn handle_search(
    catalog: Arc<MilvusCatalog>,
    body: SearchRequest,
) -> Result<impl Reply, warp::Rejection> {
    let entry = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let k = body.limit.unwrap_or(10);
    let db = catalog.db();
    let mut db_guard = db.lock().map_err(|_| {
        warp::reject::custom(MilvusError::InternalError("database lock poisoned".to_string()))
    })?;

    // Get the table and secondary index
    let (table, sec_idx) = db_guard.get_table_and_secondary_index_mut_by_name(&entry.remdb_table_name)
        .map_err(|_| warp::reject::custom(MilvusError::CollectionNotFound(body.collection_name.clone())))?;

    let results = match sec_idx {
        remdb::AnySecondaryIndex::Vector(vec_idx) => {
            unsafe { vec_idx.search_knn(body.vector.as_ptr(), k) }
                .map_err(|_| warp::reject::custom(MilvusError::SearchFailed("search error".to_string())))?
        }
        _ => {
            return Err(warp::reject::custom(MilvusError::SearchFailed("not a vector index".to_string())));
        }
    };
    // Note: sec_idx mutable borrow is released here; table shared borrow remains

    // Build response items
    let mut items = Vec::new();
    let offset = body.offset.unwrap_or(0);
    for (distance, record_id) in results.iter().skip(offset).take(k) {
        if let Some(record_ref) = table.get_by_id_ref(*record_id as usize) {
            let mut entity = serde_json::Map::new();
            // Build entity from output fields
            if let Some(out_fields) = &body.output_fields {
                for field_name in out_fields {
                    if let Some(field_idx) = table.def.fields.iter().position(|f| f.name == *field_name) {
                        let field = &table.def.fields[field_idx];
                        let val = typed_value_from_record(&record_ref, field_idx, field.data_type);
                        if let Some(v) = val {
                            entity.insert(field_name.clone(), v);
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

    let response = MilvusResponse::success(items);
    Ok(warp::reply::json(&response))
}

// ── Index handlers ──

pub async fn handle_create_index(
    catalog: Arc<MilvusCatalog>,
    body: CreateIndexRequest,
) -> Result<impl Reply, warp::Rejection> {
    // Validate metric type
    let _ = converter::milvus_metric_to_distance(&body.metric_type)
        .map_err(|_| warp::reject::custom(MilvusError::InvalidMetricType(body.metric_type.clone())))?;

    let data = IndexInfo { index_name: body.index_name };
    let response = MilvusResponse::success(data);
    Ok(warp::reply::json(&response))
}

pub async fn handle_drop_index(
    catalog: Arc<MilvusCatalog>,
    body: DropIndexRequest,
) -> Result<impl Reply, warp::Rejection> {
    let _ = catalog.resolve_collection(&body.collection_name).await.map_err(|e| {
        warp::reject::custom(e)
    })?;
    let resp = serde_json::json!({"code": 0, "message": "index dropped"});
    Ok(warp::reply::json(&resp))
}

// ── Error recovery ──

/// Convert warp rejections into Milvus-format JSON error responses
pub async fn handle_rejection(err: warp::Rejection) -> Result<impl Reply, Infallible> {
    let (json, status) = if let Some(milvus_err) = err.find::<MilvusError>() {
        let http_status = milvus_err.http_status();
        (milvus_err.to_json(), http_status)
    } else {
        (serde_json::json!({"code": 9999, "message": "internal server error"}), 500)
    };

    let resp = warp::reply::json(&json);
    let status_code = warp::http::StatusCode::from_u16(status).unwrap_or(warp::http::StatusCode::INTERNAL_SERVER_ERROR);
    Ok(warp::reply::with_status(resp, status_code))
}

// ── Helper functions ──

/// Interpret a union `Value` as an i64 based on the field's data type.
fn value_to_i64(val: remdb::types::Value, data_type: DataType) -> i64 {
    unsafe {
        match data_type {
            DataType::Int64 => val.i64,
            DataType::Int32 => val.i32 as i64,
            DataType::Int16 => val.i16 as i64,
            DataType::Int8 => val.i8 as i64,
            DataType::UInt64 => val.u64 as i64,
            DataType::UInt32 => val.u32 as i64,
            DataType::UInt16 => val.u16 as i64,
            DataType::UInt8 => val.u8 as i64,
            _ => 0,
        }
    }
}

/// Convert a JSON value to a SQL string representation
fn json_value_to_sql(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
        serde_json::Value::Bool(b) => {
            if *b { "1".to_string() } else { "0".to_string() }
        }
        serde_json::Value::Array(arr) => {
            // Vector: format as [x, y, z]
            let elements: Vec<String> = arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f.to_string()))
                .collect();
            format!("'[{}]'", elements.join(", "))
        }
        _ => "NULL".to_string(),
    }
}

/// Convert a TypedValue to a string representation
fn typed_value_to_string(val: &remdb::types::TypedValue) -> String {
    unsafe {
        match val.value_type {
            DataType::UInt8 => format!("{}", val.value.u8),
            DataType::UInt16 => format!("{}", val.value.u16),
            DataType::UInt32 => format!("{}", val.value.u32),
            DataType::UInt64 => format!("{}", val.value.u64),
            DataType::Int8 => format!("{}", val.value.i8),
            DataType::Int16 => format!("{}", val.value.i16),
            DataType::Int32 => format!("{}", val.value.i32),
            DataType::Int64 => format!("{}", val.value.i64),
            DataType::Float32 => format!("{}", val.value.float32),
            DataType::Float64 => format!("{}", val.value.float64),
            DataType::Bool => format!("{}", val.value.bool),
            DataType::Timestamp => format!("{}", val.value.time.value),
            DataType::TimestampTZ => format!("{}", val.value.time.value),
            DataType::VarChar | DataType::Char | DataType::Text => {
                let string_slice = core::str::from_utf8(&val.value.string).unwrap_or("");
                string_slice.trim_end_matches(char::from(0)).to_string()
            }
            DataType::Interval => {
                format!("{}", val.value.interval.value)
            }
            DataType::Vector => {
                "[vector]".to_string()
            }
            DataType::Json => {
                "<json>".to_string()
            }
        }
    }
}

/// Read a typed value from a RecordRef into a JSON Value
fn typed_value_from_record(
    record: &remdb::table::RecordRef,
    col: usize,
    data_type: DataType,
) -> Option<serde_json::Value> {
    match data_type {
        DataType::Int64 => {
            record.get_i64(col).ok().map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
        }
        DataType::Int32 => {
            record.get_i32(col).ok().map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
        }
        DataType::Int16 | DataType::Int8 => {
            record.get_i64(col).ok().map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
        }
        DataType::UInt64 => {
            record.get_u64(col).ok().map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
        }
        DataType::UInt32 | DataType::UInt16 | DataType::UInt8 => {
            record.get_u64(col).ok().map(|v| serde_json::Value::Number(serde_json::Number::from(v)))
        }
        DataType::Float64 => {
            record.get_f64(col).ok().and_then(|v| {
                serde_json::Number::from_f64(v).map(|n| serde_json::Value::Number(n))
            })
        }
        DataType::Float32 => {
            record.get_f32(col).ok().map(|v| {
                serde_json::Number::from_f64(v as f64)
                    .map(|n| serde_json::Value::Number(n))
                    .unwrap_or(serde_json::Value::Null)
            })
        }
        DataType::Bool => {
            record.get_bool(col).ok().map(|v| serde_json::Value::Bool(v))
        }
        DataType::VarChar | DataType::Char | DataType::Text => {
            record.get_str(col).ok().map(|v| serde_json::Value::String(v.to_string()))
        }
        DataType::Vector => {
            record.get_str(col).ok().map(|v| serde_json::Value::String(v.to_string()))
        }
        DataType::Json => {
            record.get_str(col).ok().map(|v| serde_json::Value::String(v.to_string()))
        }
        _ => {
            record.get_str(col).ok().map(|v| serde_json::Value::String(v.to_string()))
        }
    }
}