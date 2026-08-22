use std::collections::HashMap;

use remdb::table::RecordRef;
use remdb::types::{DataType, DistanceType, RemDbError, Result, VectorIndexType};

/// Convert Milvus type string to remdb DataType
pub fn milvus_type_to_remdb(type_str: &str) -> Result<DataType> {
    match type_str {
        "Int8" => Ok(DataType::Int8),
        "Int16" => Ok(DataType::Int16),
        "Int32" => Ok(DataType::Int32),
        "Int64" => Ok(DataType::Int64),
        "Float" => Ok(DataType::Float64),
        "Double" => Ok(DataType::Float64),
        "Bool" => Ok(DataType::Bool),
        "VarChar" | "Varchar" => Ok(DataType::VarChar),
        "FloatVector" => Ok(DataType::Vector),
        "BinaryVector" => Ok(DataType::Vector),
        "JSON" => Ok(DataType::Json),
        _ => Err(RemDbError::TypeMismatch),
    }
}

/// Convert Milvus metric type to remdb DistanceType
pub fn milvus_metric_to_distance(metric: &str) -> Result<DistanceType> {
    match metric {
        "L2" => Ok(DistanceType::L2),
        "IP" => Ok(DistanceType::InnerProduct),
        "COSINE" => Ok(DistanceType::Cosine),
        _ => Err(RemDbError::TypeMismatch),
    }
}

/// Convert Milvus index type string to remdb VectorIndexType
pub fn milvus_index_to_vector_index(index_type: &str) -> Result<VectorIndexType> {
    match index_type {
        "HNSW" => Ok(VectorIndexType::HNSW),
        "IVF_FLAT" => Ok(VectorIndexType::IVF),
        "IVF_PQ" => Ok(VectorIndexType::IVF_PQ),
        _ => Err(RemDbError::TypeMismatch),
    }
}

/// Extract vector dimension from a JSON value (field params)
pub fn parse_vector_dim(params: &Option<crate::milvus::models::FieldParams>) -> Result<u16> {
    match params {
        Some(p) => p.dim.ok_or(RemDbError::TypeMismatch),
        None => Err(RemDbError::TypeMismatch),
    }
}

/// Filter expression parsed from Milvus filter strings
#[derive(Debug, Clone)]
pub enum FilterExpr {
    /// id in [1, 2, 3]
    IdIn(Vec<i64>),
    /// field == value | field != value | field > value | etc.
    Comparison(String, String, String),
    /// field like 'pattern'
    Like(String, String),
    /// Compound: expr && expr
    And(Vec<FilterExpr>),
    /// Empty filter (match all)
    All,
}

/// Parse a simplified Milvus filter expression
pub fn parse_milvus_filter(filter: &str) -> Result<FilterExpr> {
    let filter = filter.trim();
    if filter.is_empty() {
        return Ok(FilterExpr::All);
    }

    // Handle "id in [1, 2, 3]"
    if let Some(pos) = filter.find(" in ") {
        let field = filter[..pos].trim();
        if field == "id" {
            let list_part = filter[pos + 4..].trim();
            let list_part = list_part
                .trim_start_matches('[')
                .trim_end_matches(']');
            let ids: Vec<i64> = list_part
                .split(',')
                .filter_map(|s| s.trim().parse::<i64>().ok())
                .collect();
            if ids.is_empty() {
                return Err(RemDbError::TypeMismatch);
            }
            return Ok(FilterExpr::IdIn(ids));
        }
    }

    // Handle "field like 'pattern'"
    if filter.contains(" like ") {
        let parts: Vec<&str> = filter.splitn(2, " like ").collect();
        if parts.len() == 2 {
            let field = parts[0].trim();
            let pattern = parts[1].trim().trim_matches('\'');
            return Ok(FilterExpr::Like(field.to_string(), pattern.to_string()));
        }
    }

    // Handle comparisons: ==, !=, >, <, >=, <=
    let ops = ["==", "!=", ">=", "<=", ">", "<"];
    for op in &ops {
        if let Some(pos) = filter.find(op) {
            let field = filter[..pos].trim();
            let value = filter[pos + op.len()..].trim();
            return Ok(FilterExpr::Comparison(
                field.to_string(),
                op.to_string(),
                value.to_string(),
            ));
        }
    }

    Err(RemDbError::TypeMismatch)
}

/// Check if a record field matches a filter expression
pub fn matches_filter(
    record: &RecordRef,
    field_indices: &HashMap<String, usize>,
    expr: &FilterExpr,
) -> Result<bool> {
    match expr {
        FilterExpr::All => Ok(true),
        FilterExpr::IdIn(ids) => {
            let id = record.get_i64(0)?; // primary key is always at index 0
            Ok(ids.contains(&id))
        }
        FilterExpr::Comparison(field, op, value) => {
            let col = match field_indices.get(field.as_str()) {
                Some(c) => *c,
                None => return Ok(true), // skip unknown fields
            };
            let record_val = record.get_i64(col)?;
            let cmp_val = value.parse::<i64>().map_err(|_| RemDbError::TypeMismatch)?;
            match op.as_str() {
                "==" => Ok(record_val == cmp_val),
                "!=" => Ok(record_val != cmp_val),
                ">" => Ok(record_val > cmp_val),
                "<" => Ok(record_val < cmp_val),
                ">=" => Ok(record_val >= cmp_val),
                "<=" => Ok(record_val <= cmp_val),
                _ => Ok(true),
            }
        }
        FilterExpr::Like(field, pattern) => {
            let col = match field_indices.get(field.as_str()) {
                Some(c) => *c,
                None => return Ok(true),
            };
            let record_val = record.get_str(col)?;
            // Simple wildcard: % at end = starts_with, % at start = ends_with
            let matched = if pattern.starts_with('%') && pattern.ends_with('%') {
                let inner = &pattern[1..pattern.len() - 1];
                record_val.contains(inner)
            } else if pattern.starts_with('%') {
                record_val.ends_with(&pattern[1..])
            } else if pattern.ends_with('%') {
                record_val.starts_with(&pattern[..pattern.len() - 1])
            } else {
                record_val == pattern
            };
            Ok(matched)
        }
        FilterExpr::And(exprs) => {
            for e in exprs {
                if !matches_filter(record, field_indices, e)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use remdb::types::DataType;

    #[test]
    fn test_milvus_type_to_remdb_int64() {
        let dt = milvus_type_to_remdb("Int64").unwrap();
        assert_eq!(dt, DataType::Int64);
    }

    #[test]
    fn test_milvus_type_to_remdb_float() {
        let dt = milvus_type_to_remdb("Float").unwrap();
        assert_eq!(dt, DataType::Float64);
    }

    #[test]
    fn test_milvus_type_to_remdb_bool() {
        let dt = milvus_type_to_remdb("Bool").unwrap();
        assert_eq!(dt, DataType::Bool);
    }

    #[test]
    fn test_milvus_type_to_remdb_varchar() {
        let dt = milvus_type_to_remdb("VarChar").unwrap();
        assert_eq!(dt, DataType::VarChar);
    }

    #[test]
    fn test_milvus_type_to_remdb_varchar_alt() {
        let dt = milvus_type_to_remdb("Varchar").unwrap();
        assert_eq!(dt, DataType::VarChar);
    }

    #[test]
    fn test_milvus_type_to_remdb_float_vector() {
        let dt = milvus_type_to_remdb("FloatVector").unwrap();
        assert_eq!(dt, DataType::Vector);
    }

    #[test]
    fn test_milvus_type_to_remdb_json() {
        let dt = milvus_type_to_remdb("JSON").unwrap();
        assert_eq!(dt, DataType::Json);
    }

    #[test]
    fn test_milvus_type_to_remdb_unknown() {
        let result = milvus_type_to_remdb("Unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_metric_conversion_l2() {
        let dist = milvus_metric_to_distance("L2").unwrap();
        assert_eq!(dist, DistanceType::L2);
    }

    #[test]
    fn test_metric_conversion_ip() {
        let dist = milvus_metric_to_distance("IP").unwrap();
        assert_eq!(dist, DistanceType::InnerProduct);
    }

    #[test]
    fn test_metric_conversion_cosine() {
        let dist = milvus_metric_to_distance("COSINE").unwrap();
        assert_eq!(dist, DistanceType::Cosine);
    }

    #[test]
    fn test_metric_conversion_unknown() {
        let result = milvus_metric_to_distance("UNKNOWN");
        assert!(result.is_err());
    }

    #[test]
    fn test_index_conversion_hnsw() {
        let idx = milvus_index_to_vector_index("HNSW").unwrap();
        assert_eq!(idx, VectorIndexType::HNSW);
    }

    #[test]
    fn test_index_conversion_ivf_flat() {
        let idx = milvus_index_to_vector_index("IVF_FLAT").unwrap();
        assert_eq!(idx, VectorIndexType::IVF);
    }

    #[test]
    fn test_index_conversion_ivf_pq() {
        let idx = milvus_index_to_vector_index("IVF_PQ").unwrap();
        assert_eq!(idx, VectorIndexType::IVF_PQ);
    }

    #[test]
    fn test_index_conversion_unknown() {
        let result = milvus_index_to_vector_index("UNKNOWN");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_parser_id_in() {
        let expr = parse_milvus_filter("id in [1, 2, 3]").unwrap();
        match expr {
            FilterExpr::IdIn(ids) => assert_eq!(ids, vec![1, 2, 3]),
            _ => panic!("Expected IdIn"),
        }
    }

    #[test]
    fn test_filter_parser_id_in_single() {
        let expr = parse_milvus_filter("id in [42]").unwrap();
        match expr {
            FilterExpr::IdIn(ids) => assert_eq!(ids, vec![42]),
            _ => panic!("Expected IdIn"),
        }
    }

    #[test]
    fn test_filter_parser_comparison_eq() {
        let expr = parse_milvus_filter("id == 42").unwrap();
        match expr {
            FilterExpr::Comparison(field, op, val) => {
                assert_eq!(field, "id");
                assert_eq!(op, "==");
                assert_eq!(val, "42");
            }
            _ => panic!("Expected Comparison"),
        }
    }

    #[test]
    fn test_filter_parser_comparison_ne() {
        let expr = parse_milvus_filter("age != 18").unwrap();
        match expr {
            FilterExpr::Comparison(field, op, val) => {
                assert_eq!(field, "age");
                assert_eq!(op, "!=");
                assert_eq!(val, "18");
            }
            _ => panic!("Expected Comparison"),
        }
    }

    #[test]
    fn test_filter_parser_comparison_gt() {
        let expr = parse_milvus_filter("price > 100").unwrap();
        match expr {
            FilterExpr::Comparison(field, op, val) => {
                assert_eq!(field, "price");
                assert_eq!(op, ">");
                assert_eq!(val, "100");
            }
            _ => panic!("Expected Comparison"),
        }
    }

    #[test]
    fn test_filter_parser_comparison_gte() {
        let expr = parse_milvus_filter("score >= 90").unwrap();
        match expr {
            FilterExpr::Comparison(field, op, val) => {
                assert_eq!(field, "score");
                assert_eq!(op, ">=");
                assert_eq!(val, "90");
            }
            _ => panic!("Expected Comparison"),
        }
    }

    #[test]
    fn test_filter_parser_like() {
        let expr = parse_milvus_filter("name like 'hello%'").unwrap();
        match expr {
            FilterExpr::Like(field, pattern) => {
                assert_eq!(field, "name");
                assert_eq!(pattern, "hello%");
            }
            _ => panic!("Expected Like"),
        }
    }

    #[test]
    fn test_filter_parser_empty() {
        let expr = parse_milvus_filter("").unwrap();
        match expr {
            FilterExpr::All => {}
            _ => panic!("Expected All"),
        }
    }

    #[test]
    fn test_filter_parser_whitespace() {
        let expr = parse_milvus_filter("  ").unwrap();
        match expr {
            FilterExpr::All => {}
            _ => panic!("Expected All"),
        }
    }

    #[test]
    fn test_filter_parser_unknown() {
        let result = parse_milvus_filter("garbage input !!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_vector_dim_some() {
        let params = Some(crate::milvus::models::FieldParams {
            dim: Some(128),
            max_length: None,
        });
        let dim = parse_vector_dim(&params).unwrap();
        assert_eq!(dim, 128);
    }

    #[test]
    fn test_parse_vector_dim_none() {
        let params = None;
        let result = parse_vector_dim(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_vector_dim_missing_dim() {
        let params = Some(crate::milvus::models::FieldParams {
            dim: None,
            max_length: None,
        });
        let result = parse_vector_dim(&params);
        assert!(result.is_err());
    }
}