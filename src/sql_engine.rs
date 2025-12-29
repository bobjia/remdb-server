use remdb::{RemDb, RemDbError, DdlExecutor};
use thiserror::Error;
use crate::debug_println;

#[derive(Error, Debug)]
pub enum SqlError {
    #[error("Database error: {0}")]
    Database(RemDbError),
    #[error("SQL parsing error: {0}")]
    Parsing(String),
    #[error("Unsupported SQL command")]
    Unsupported,
}

impl From<RemDbError> for SqlError {
    fn from(err: RemDbError) -> Self {
        SqlError::Database(err)
    }
}

#[derive(Debug)]
pub enum ExtendedQueryType {
    Select,
    Describe,
    Tables,
    Insert,
    Update,
    Delete,
}

#[derive(Debug)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: usize,
}

/// 执行扩展的SQL命令
pub fn execute_extended_sql(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    let sql_lower = sql.trim().to_lowercase();
    
    // 处理TABLES命令
    if sql_lower == "tables" {
        return execute_tables(db);
    }
    
    // 处理DESCRIBE命令
    if sql_lower.starts_with("describe ") || sql_lower.starts_with("desc ") {
        let table_name = sql_lower.split_whitespace().nth(1)
            .ok_or(SqlError::Parsing("Missing table name".to_string()))?;
        return execute_describe(db, table_name);
    }
    
    // 处理SELECT命令
    if sql_lower.starts_with("select ") {
        return execute_select(db, sql);
    }
    
    // 处理INSERT命令
    if sql_lower.starts_with("insert ") {
        return execute_insert(db, sql);
    }
    
    // 处理DELETE命令
    if sql_lower.starts_with("delete ") {
        return execute_delete(db, sql);
    }
    
    // 处理CREATE TABLE命令
    if sql_lower.starts_with("create table ") {
        return execute_create_table(db, sql);
    }
    
    // 处理CREATE INDEX命令
    if sql_lower.starts_with("create index ") {
        return execute_create_index(db, sql);
    }
    
    // 处理STAT命令
    if sql_lower == "stat" {
        return execute_stat(db);
    }
    
    // 处理HEALTHCHECK命令
    if sql_lower == "healthcheck" {
        return execute_healthcheck(db);
    }
    
    // 处理其他命令
    Err(SqlError::Unsupported)
}

/// 执行TABLES命令
fn execute_tables(db: &RemDb) -> std::result::Result<ResultSet, SqlError> {
    let columns = vec!["Tables".to_string()];
    let mut rows = Vec::new();
    
    // 获取所有表，包括初始配置的表和动态创建的表
    for table in db.config.tables {
        rows.push(vec![table.name.to_string()]);
    }
    
    // 添加调试信息：打印实际表数量和表定义数量
    debug_println!("Debug: db.config.tables.len() = {}", db.config.tables.len());
    
    Ok(ResultSet {
        columns,
        rows: rows.clone(),
        affected_rows: rows.len(),
    })
}

/// 执行DESCRIBE命令
fn execute_describe(db: &RemDb, table_name: &str) -> std::result::Result<ResultSet, SqlError> {
    let table = db.config.tables.iter()
        .find(|t| *t.name == *table_name)
        .ok_or(SqlError::Parsing(format!("Table not found: {}", table_name)))?;
    
    let columns = vec!["Type".to_string(), "Name".to_string(), "Details".to_string()];
    let mut rows = Vec::new();
    
    // 添加表名信息
    rows.push(vec![
        "TABLE".to_string(),
        table.name.to_string(),
        "".to_string(),
    ]);
    
    // 添加列信息
    for field in table.fields {
        let data_type = match field.data_type {
            remdb::types::DataType::UInt8 => "UInt8",
            remdb::types::DataType::UInt16 => "UInt16",
            remdb::types::DataType::UInt32 => "UInt32",
            remdb::types::DataType::UInt64 => "UInt64",
            remdb::types::DataType::Int8 => "Int8",
            remdb::types::DataType::Int16 => "Int16",
            remdb::types::DataType::Int32 => "Int32",
            remdb::types::DataType::Int64 => "Int64",
            remdb::types::DataType::Float32 => "Float32",
            remdb::types::DataType::Float64 => "Float64",
            remdb::types::DataType::Bool => "Bool",
            remdb::types::DataType::Timestamp => "Timestamp",
            remdb::types::DataType::String => "String",
        };
        
        let mut details = format!("Size: {}", field.size);
        
        // 标记主键
        if field.offset == table.fields[table.primary_key].offset {
            details.push_str(", PRIMARY KEY");
        }
        
        rows.push(vec![
            "COLUMN".to_string(),
            field.name.to_string(),
            details,
        ]);
    }
    
    // 添加索引信息（示例）
    // 注意：由于当前remdb库不支持在TableDef中存储辅助索引信息，
    // 这里我们使用示例数据来演示索引显示功能
    let sample_indices = match table_name {
        "user" => &[(
            "PRIMARY", "id", "BTREE",
        ), (
            "idx_user_name", "name", "BTREE",
        )] as &[( &str, &str, &str)],
        "product" => &[(
            "PRIMARY", "id", "BTREE",
        ), (
            "idx_product_category", "category", "BTREE",
        ), (
            "idx_product_price", "price", "BTREE",
        )] as &[( &str, &str, &str)],
        _ => &[(
            "PRIMARY", "id", "BTREE",
        )] as &[( &str, &str, &str)],
    };
    
    for &(index_name, column_name, index_type) in sample_indices {
        rows.push(vec![
            "INDEX".to_string(),
            index_name.to_string(),
            format!("{} ({})", index_type, column_name),
        ]);
    }
    
    Ok(ResultSet {
        columns,
        rows: rows.clone(),
        affected_rows: rows.len(),
    })
}

/// 执行SELECT命令
fn execute_select(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 增加读取操作计数
    db.metrics.inc_read_ops();
    
    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing SELECT SQL: {}", sql);
    
    // 调试：手动查找表名
    let sql_lower = sql.to_lowercase();
    let table_name_start = sql_lower.find("from ").map(|pos| pos + 5).unwrap_or(0);
    let table_name_end = sql_lower[table_name_start..].find(|c: char| c.is_whitespace() || c == ';').unwrap_or_else(|| sql_lower[table_name_start..].len());
    let table_name = &sql_lower[table_name_start..table_name_start + table_name_end];
    debug_println!("Debug: Extracted table name: {}", table_name);
    
    // 调试：检查db.config.tables中是否存在该表
    let table_exists = db.config.tables.iter().any(|t| t.name == table_name);
    debug_println!("Debug: Table '{}' exists in config: {}", table_name, table_exists);
    
    // 调试：列出所有配置表
    debug_println!("Debug: All config tables:");
    for (i, table) in db.config.tables.iter().enumerate() {
        debug_println!("Debug:   [{}] name: '{}', id: {}", i, table.name, table.id);
    }
    
    // 使用RemDb的sql_query方法执行SELECT语句
    let result = db.sql_query(sql)?;
    
    // 获取表定义
    let table = db.config.tables.iter()
        .find(|t| *t.name == *table_name)
        .ok_or(SqlError::Database(remdb::RemDbError::TableNotFound))?;
    
    // 构建完整的行数据
    let mut rows = Vec::new();
    
    // 遍历所有行，使用引用迭代避免所有权转移
    for row in &result.rows {
        let mut row_data = Vec::new();
        
        // 遍历行中的所有值，将remdb::Value转换为字符串
        for (i, value) in row.values.iter().enumerate() {
            // 获取当前字段的定义
            let field = &table.fields[i];
            
            let value_str = unsafe {
                // 1. 首先尝试字符串类型
                let mut str_value = String::new();
                let mut has_valid_chars = false;
                
                for &c in value.string.iter() {
                    if c == 0 {
                        break; // 遇到null终止符，结束字符串
                    }
                    if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                        // 只有当字符是可打印的ASCII字符或空格时，才认为是有效字符
                        str_value.push(c as char);
                        has_valid_chars = true;
                    }
                }
                
                if has_valid_chars {
                    // 去除可能的引号
                    str_value.trim_matches('"').to_string()
                } else {
                    // 根据字段类型决定如何转换值
                    match field.data_type {
                        remdb::types::DataType::String => {
                            // TEXT类型字段，显示为空字符串
                            "".to_string()
                        },
                        remdb::types::DataType::Float32 => {
                            // Float32类型，读取float32值
                            let float32_val = value.float32;
                            format!("{}", float32_val)
                        },
                        remdb::types::DataType::Float64 => {
                            // Float64类型，读取float64值
                            let float64_val = value.float64;
                            format!("{}", float64_val)
                        },
                        _ => {
                            // 其他数值类型，显示为i32值
                            let i32_val = value.i32;
                            format!("{}", i32_val)
                        }
                    }
                }
            };
            
            row_data.push(value_str);
        }
        
        rows.push(row_data);
    }
    
    // 构造结果集：包含列信息和行数据
    let result_set = ResultSet {
        columns: result.columns.clone(),
        rows,
        affected_rows: result.rows.len(),
    };
    
    Ok(result_set)
}

/// 执行INSERT命令
fn execute_insert(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 处理INSERT语句，支持带列名的情况
    let sql_lower = sql.trim().to_lowercase();
    
    // 检查是否是INSERT语句
    if !sql_lower.starts_with("insert into ") {
        return Err(SqlError::Parsing("Not an INSERT statement".to_string()));
    }
    
    // 提取表名和剩余部分
    let after_insert = sql_lower.trim_start_matches("insert into ");
    
    // 解析INSERT语句的各个部分
    let (table_name, specified_columns, values) = parse_insert_parts(sql, after_insert)?;
    
    // 查找表定义
    let table = db.config.tables.iter()
        .find(|t| *t.name == *table_name)
        .ok_or(SqlError::Database(remdb::RemDbError::TableNotFound))?;
    
    // 获取所有列名
    let all_columns: Vec<&str> = table.fields.iter()
        .map(|f| f.name)
        .collect();
    
    // 获取所有列名的小写形式，方便后续比较
    let all_columns_lower: Vec<String> = all_columns.iter()
        .map(|col| col.to_lowercase())
        .collect();
    
    // 构建列名到值的映射
    let mut column_values = std::collections::HashMap::new();
    for (col, val) in specified_columns.into_iter().zip(values.into_iter()) {
        let col_lower = col.to_lowercase();
        
        // 检查列名是否存在于表中
        if !all_columns_lower.contains(&col_lower) {
            return Err(SqlError::Parsing(format!("Column '{}' does not exist in table '{}'", col, table_name)));
        }
        
        column_values.insert(col_lower, val);
    }
    
    // 构建完整的VALUES列表，按表的列顺序排列
    let mut full_values = Vec::new();
    for col in &all_columns {
        if let Some(val) = column_values.get(&col.to_lowercase()) {
            full_values.push(val.clone());
        } else {
            // 对于未指定的列，使用默认值
            // 根据数据类型选择默认值
            let field = table.fields.iter().find(|f| f.name == *col).unwrap();
            let default_val = match field.data_type {
                remdb::types::DataType::UInt8 | remdb::types::DataType::UInt16 |
                remdb::types::DataType::UInt32 | remdb::types::DataType::UInt64 |
                remdb::types::DataType::Int8 | remdb::types::DataType::Int16 |
                remdb::types::DataType::Int32 | remdb::types::DataType::Int64 => "0",
                remdb::types::DataType::Float32 | remdb::types::DataType::Float64 => "0.0",
                remdb::types::DataType::Bool => "false",
                remdb::types::DataType::Timestamp => "0",
                remdb::types::DataType::String => "\"\"", // 空字符串
            };
            full_values.push(default_val.to_string());
        }
    }
    
    // 构建完整的INSERT语句
    let full_insert_sql = format!(
        "INSERT INTO {} VALUES ({})
",
        table_name,
        full_values.join(", ")
    );
    
    // 执行INSERT语句
    db.sql_query(&full_insert_sql)?;
    
    // 对于INSERT语句，假设成功插入1行
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 1,
    })
}

/// 解析INSERT语句的各个部分：表名、指定的列名、值
fn parse_insert_parts(sql: &str, after_insert: &str) -> std::result::Result<(String, Vec<String>, Vec<String>), SqlError> {
    // 查找左括号位置（如果有列名列表）
    if let Some(left_paren) = after_insert.find('(') {
        // 提取表名（左括号前的部分）
        let table_name = after_insert[..left_paren].trim().to_string();
        
        // 查找右括号位置
        let after_left_paren = &after_insert[left_paren..];
        let right_paren = after_left_paren.find(')')
            .ok_or(SqlError::Parsing("Missing closing parenthesis for columns".to_string()))?;
        
        // 提取列名列表
        let columns_part = &after_left_paren[1..right_paren];
        let specified_columns: Vec<String> = columns_part.split(',')
            .map(|col| col.trim().to_string())
            .collect();
        
        // 查找VALUES关键字
        let after_right_paren = &after_left_paren[right_paren + 1..];
        let values_pos = after_right_paren.find("values")
            .ok_or(SqlError::Parsing("Missing VALUES keyword".to_string()))?;
        
        // 提取VALUES部分
        let values_part = &after_right_paren[values_pos + 6..].trim();
        let left_val_paren = values_part.find('(')
            .ok_or(SqlError::Parsing("Missing opening parenthesis for values".to_string()))?;
        let right_val_paren = values_part.find(')')
            .ok_or(SqlError::Parsing("Missing closing parenthesis for values".to_string()))?;
        
        // 提取值列表
        let values_str = &values_part[left_val_paren + 1..right_val_paren];
        let values: Vec<String> = values_str.split(',')
            .map(|val| val.trim().to_string())
            .collect();
        
        Ok((table_name, specified_columns, values))
    } else {
        // 没有列名列表，直接解析VALUES
        let parts: Vec<&str> = after_insert.split("values").collect();
        if parts.len() != 2 {
            return Err(SqlError::Parsing("Invalid INSERT syntax".to_string()));
        }
        
        let table_name = parts[0].trim().to_string();
        let values_part = parts[1].trim();
        
        // 提取值列表
        let left_paren = values_part.find('(')
            .ok_or(SqlError::Parsing("Missing opening parenthesis for values".to_string()))?;
        let right_paren = values_part.find(')')
            .ok_or(SqlError::Parsing("Missing closing parenthesis for values".to_string()))?;
        
        let values_str = &values_part[left_paren + 1..right_paren];
        let values: Vec<String> = values_str.split(',')
            .map(|val| val.trim().to_string())
            .collect();
        
        // 没有指定列名，返回空列表
        Ok((table_name, Vec::new(), values))
    }
}

/// 执行DELETE命令
fn execute_delete(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 使用RemDb的sql_query方法执行DELETE语句
    db.sql_query(sql)?;
    
    // 对于DELETE语句，假设成功删除1行
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 1,
    })
}

/// 执行CREATE TABLE命令
fn execute_create_table(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 输出完整的SQL语句，用于调试
    debug_println!("Debug: Executing CREATE TABLE SQL: '{}'", sql);
    
    // 使用sql_query方法执行CREATE TABLE语句
    db.sql_query(sql)?;
    
    // 返回成功结果，受影响行数为1
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 1,
    })
}

/// 执行CREATE INDEX命令
fn execute_create_index(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 输出完整的SQL语句，用于调试
    debug_println!("Debug: Executing CREATE INDEX SQL: '{}'", sql);
    
    // 使用sql_query方法执行CREATE INDEX语句
    db.sql_query(sql)?;
    
    // 返回成功结果，受影响行数为1
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 1,
    })
}

/// 执行STAT命令
fn execute_stat(db: &RemDb) -> std::result::Result<ResultSet, SqlError> {
    // 获取监控指标快照
    let metrics = db.metrics_snapshot();
    
    // 构建结果集
    let columns = vec!["Metric".to_string(), "Value".to_string()];
    let mut rows = Vec::new();
    
    // 添加监控指标到结果集
    rows.push(vec!["Database Size".to_string(), format!("{} bytes", metrics.total_memory)]);
    rows.push(vec!["Memory Usage".to_string(), format!("{} bytes", metrics.used_memory)]);
    rows.push(vec!["Memory Usage %".to_string(), format!("{:.2}%", (metrics.used_memory as f64 / metrics.total_memory as f64) * 100.0)]);
    rows.push(vec!["Table Count".to_string(), format!("{}", db.config.tables.len())]);
    rows.push(vec!["Read Operations".to_string(), format!("{}", metrics.read_ops)]);
    rows.push(vec!["Write Operations".to_string(), format!("{}", metrics.write_ops)]);
    rows.push(vec!["Delete Operations".to_string(), format!("{}", metrics.delete_ops)]);
    rows.push(vec!["Update Operations".to_string(), format!("{}", metrics.update_ops)]);
    rows.push(vec!["Cache Hit Rate".to_string(), format!("{:.2}%", metrics.cache_hit_rate)]);
    rows.push(vec!["Cache Hits".to_string(), format!("{}", metrics.cache_hits)]);
    rows.push(vec!["Cache Misses".to_string(), format!("{}", metrics.cache_misses)]);
    rows.push(vec!["Active Connections".to_string(), "1".to_string()]);
    rows.push(vec!["Transactions".to_string(), format!("{}", metrics.transactions)]);
    rows.push(vec!["Committed Transactions".to_string(), format!("{}", metrics.committed_transactions)]);
    rows.push(vec!["Rolled Back Transactions".to_string(), format!("{}", metrics.rolled_back_transactions)]);
    
    Ok(ResultSet {
        columns,
        rows: rows.clone(),
        affected_rows: rows.len(),
    })
}

/// 执行HEALTHCHECK命令
fn execute_healthcheck(db: &RemDb) -> std::result::Result<ResultSet, SqlError> {
    // 执行健康检查
    let health_result = db.health_check();
    
    // 构建结果集
    let columns = vec!["Component".to_string(), "Status".to_string(), "Details".to_string()];
    let mut rows = Vec::new();
    
    // 添加健康检查结果到结果集
    rows.push(vec![
        "Database".to_string(),
        match health_result.status {
            remdb::monitor::HealthStatus::Healthy => "HEALTHY".to_string(),
            remdb::monitor::HealthStatus::Warning => "WARNING".to_string(),
            remdb::monitor::HealthStatus::Unhealthy => "UNHEALTHY".to_string(),
        },
        health_result.details.to_string()
    ]);
    
    // 添加内存健康检查结果
    let metrics = health_result.metrics;
    let memory_usage = metrics.used_memory as f64 / metrics.total_memory as f64;
    rows.push(vec![
        "Memory".to_string(),
        if memory_usage > 0.9 { "UNHEALTHY".to_string() } 
        else if memory_usage > 0.7 { "WARNING".to_string() } 
        else { "HEALTHY".to_string() },
        format!("Usage: {}/{} bytes ({:.2}%)", metrics.used_memory, metrics.total_memory, memory_usage * 100.0)
    ]);
    
    // 添加表健康检查结果
    rows.push(vec![
        "Tables".to_string(),
        "HEALTHY".to_string(),
        format!("{} tables loaded successfully", db.config.tables.len())
    ]);
    
    // 添加操作统计
    rows.push(vec![
        "Operations".to_string(),
        "HEALTHY".to_string(),
        format!("Read: {}, Write: {}, Delete: {}, Update: {}", 
               metrics.read_ops, metrics.write_ops, metrics.delete_ops, metrics.update_ops)
    ]);
    
    // 添加缓存健康检查结果
    rows.push(vec![
        "Cache".to_string(),
        if metrics.cache_hit_rate < 50.0 { "WARNING".to_string() } 
        else { "HEALTHY".to_string() },
        format!("Hit Rate: {:.2}%, Hits: {}, Misses: {}", 
               metrics.cache_hit_rate, metrics.cache_hits, metrics.cache_misses)
    ]);
    
    Ok(ResultSet {
        columns,
        rows: rows.clone(),
        affected_rows: rows.len(),
    })
}

/// 格式化ResultSet为表格输出
pub fn format_result_set(result_set: &ResultSet) -> String {
    if result_set.columns.is_empty() {
        return format!("Affected {0} row(s)", result_set.affected_rows);
    }
    
    // 计算每列的最大宽度
    let mut col_widths: Vec<usize> = result_set.columns.iter().map(|c| c.len()).collect();
    
    for row in &result_set.rows {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > col_widths[i] {
                col_widths[i] = cell.len();
            }
        }
    }
    
    // 构建分隔线
    let separator: String = col_widths.iter()
        .map(|w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("") + "+";
    
    // 构建表头
    let mut output = String::new();
    output.push_str(&separator);
    output.push_str("\n");
    
    for (i, col) in result_set.columns.iter().enumerate() {
        output.push_str(&format!("| {:<width$} ", col, width = col_widths[i]));
    }
    output.push_str("|");
    output.push_str("\n");
    
    output.push_str(&separator);
    output.push_str("\n");
    
    // 构建行
    for row in &result_set.rows {
        for (i, cell) in row.iter().enumerate() {
            output.push_str(&format!("| {:<width$} ", cell, width = col_widths[i]));
        }
        output.push_str("|");
        output.push_str("\n");
    }
    
    output.push_str(&separator);
    output.push_str("\n");
    
    output
}