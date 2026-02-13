use crate::sql_engine::{ResultSet, SqlResult};
use remdb::RemDb;
use remdb::types::DataType;

pub struct SelectExecutor;

impl SelectExecutor {
    pub fn execute(db: &mut RemDb, sql: &str) -> SqlResult<ResultSet> {
        let result = db.sql_query(sql)?;
        let columns: Vec<String> = result.columns.clone();
        let rows: Vec<Vec<String>> = result
            .rows
            .iter()
            .map(|row| row.values.iter().map(|v| format_typed_value(v)).collect())
            .collect();

        Ok(ResultSet {
            columns,
            rows,
            affected_rows: 0,
        })
    }
}

fn format_typed_value(value: &remdb::types::TypedValue) -> String {
    unsafe {
        match value.value_type {
            DataType::UInt8 => value.value.u8.to_string(),
            DataType::UInt16 => value.value.u16.to_string(),
            DataType::UInt32 => value.value.u32.to_string(),
            DataType::UInt64 => value.value.u64.to_string(),
            DataType::Int8 => value.value.i8.to_string(),
            DataType::Int16 => value.value.i16.to_string(),
            DataType::Int32 => value.value.i32.to_string(),
            DataType::Int64 => value.value.i64.to_string(),
            DataType::Float32 => value.value.float32.to_string(),
            DataType::Float64 => value.value.float64.to_string(),
            DataType::Bool => value.value.bool.to_string(),
            DataType::Timestamp | DataType::TimestampTZ => value.value.time.value.to_string(),
            DataType::Interval => value.value.interval.value.to_string(),
            DataType::VarChar | DataType::Char | DataType::Text => {
                let bytes = &value.value.string;
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                String::from_utf8_lossy(&bytes[..end]).to_string()
            }
            DataType::Vector => {
                let vec_ptr = value.value.vector;
                let metadata = value.value.vector_metadata;
                if vec_ptr.is_null() {
                    "[]".to_string()
                } else {
                    let dimension = metadata.dimension as usize;
                    let slice = std::slice::from_raw_parts(vec_ptr, dimension);
                    let values: Vec<String> = slice.iter().map(|v| format!("{:.4}", v)).collect();
                    format!("[{}]", values.join(", "))
                }
            }
            DataType::Json => "JSON".to_string(),
        }
    }
}
