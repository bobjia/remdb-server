use remdb::types::{DataType, DistanceType, RemDbError, Result, VectorIndexType};

pub fn milvus_type_to_remdb(type_str: &str) -> Result<DataType> {
    match type_str {
        "Int64" => Ok(DataType::Integer),
        "Float" => Ok(DataType::Real),
        "Bool" => Ok(DataType::Boolean),
        "VarChar" | "Varchar" => Ok(DataType::Text),
        "FloatVector" => Ok(DataType::Vector),
        "JSON" => Ok(DataType::JSON),
        _ => Err(RemDbError::TypeMismatch),
    }
}

pub fn milvus_metric_to_distance(metric: &str) -> Result<DistanceType> {
    match metric {
        "L2" => Ok(DistanceType::L2),
        "IP" => Ok(DistanceType::InnerProduct),
        "COSINE" => Ok(DistanceType::Cosine),
        _ => Err(RemDbError::TypeMismatch),
    }
}

pub fn milvus_index_to_vector_index(index_type: &str) -> Result<VectorIndexType> {
    match index_type {
        "HNSW" => Ok(VectorIndexType::HNSW),
        "IVF_FLAT" => Ok(VectorIndexType::IVF),
        "IVF_PQ" => Ok(VectorIndexType::IVF_PQ),
        _ => Err(RemDbError::TypeMismatch),
    }
}

pub fn parse_vector_dim(params: &Option<crate::milvus::models::FieldParams>) -> Result<u16> {
    match params {
        Some(p) => p.dim.ok_or(RemDbError::TypeMismatch),
        None => Err(RemDbError::TypeMismatch),
    }
}

#[derive(Debug, Clone)]
pub enum FilterExpr {
    IdIn(Vec<i64>),
    Comparison(String, String, String),
    Like(String, String),
    And(Vec<FilterExpr>),
    All,
}

pub fn parse_milvus_filter(filter: &str) -> Result<FilterExpr> {
    let filter = filter.trim();
    if filter.is_empty() {
        return Ok(FilterExpr::All);
    }

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

    if filter.contains(" like ") {
        let parts: Vec<&str> = filter.splitn(2, " like ").collect();
        if parts.len() == 2 {
            let field = parts[0].trim();
            let pattern = parts[1].trim().trim_matches('\'');
            return Ok(FilterExpr::Like(field.to_string(), pattern.to_string()));
        }
    }

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

pub fn matches_filter(
    record: &remdb::table::RecordRef,
    field_indices: &std::collections::HashMap<String, usize>,
    expr: &FilterExpr,
) -> Result<bool> {
    match expr {
        FilterExpr::All => Ok(true),
        FilterExpr::IdIn(ids) => {
            let id = record.get_i64(0)?;
            Ok(ids.contains(&id))
        }
        FilterExpr::Comparison(field, op, value) => {
            let col = match field_indices.get(field.as_str()) {
                Some(c) => *c,
                None => return Ok(true),
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

    #[test]
    fn test_milvus_type_to_remdb() {
        assert_eq!(milvus_type_to_remdb("Int64").unwrap(), DataType::Integer);
        assert_eq!(milvus_type_to_remdb("Float").unwrap(), DataType::Real);
        assert_eq!(milvus_type_to_remdb("Bool").unwrap(), DataType::Boolean);
        assert_eq!(milvus_type_to_remdb("VarChar").unwrap(), DataType::Text);
        assert_eq!(milvus_type_to_remdb("FloatVector").unwrap(), DataType::Vector);
        assert!(milvus_type_to_remdb("Unknown").is_err());
    }

    #[test]
    fn test_metric_conversion() {
        assert_eq!(milvus_metric_to_distance("L2").unwrap(), DistanceType::L2);
        assert_eq!(milvus_metric_to_distance("IP").unwrap(), DistanceType::InnerProduct);
        assert_eq!(milvus_metric_to_distance("COSINE").unwrap(), DistanceType::Cosine);
        assert!(milvus_metric_to_distance("UNKNOWN").is_err());
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
    fn test_filter_parser_comparison() {
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
    fn test_filter_parser_empty() {
        let expr = parse_milvus_filter("").unwrap();
        assert!(matches!(expr, FilterExpr::All));
    }

    #[test]
    fn test_filter_parser_like() {
        let expr = parse_milvus_filter("name like 'test%'").unwrap();
        match expr {
            FilterExpr::Like(field, pattern) => {
                assert_eq!(field, "name");
                assert_eq!(pattern, "test%");
            }
            _ => panic!("Expected Like"),
        }
    }

    #[test]
    fn test_index_type_conversion() {
        assert_eq!(milvus_index_to_vector_index("HNSW").unwrap(), VectorIndexType::HNSW);
        assert_eq!(milvus_index_to_vector_index("IVF_FLAT").unwrap(), VectorIndexType::IVF);
        assert!(milvus_index_to_vector_index("UNKNOWN").is_err());
    }
}
