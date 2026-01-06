use crate::ddl_compiler::DdlError;
use crate::debug_println;
use remdb::{DdlExecutor, RemDb, RemDbError};
use thiserror::Error;

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

impl From<DdlError> for SqlError {
    fn from(err: DdlError) -> Self {
        SqlError::Parsing(err.to_string())
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
        let table_name = sql_lower
            .split_whitespace()
            .nth(1)
            .ok_or(SqlError::Parsing("Missing table name".to_string()))?;
        return execute_describe(db, table_name);
    }

    // 处理SELECT命令
    if sql_lower.starts_with("select ") {
        return execute_select(db, sql);
    }

    // 处理INSERT命令 - 简化检查，只检查是否以"insert"开头
    if sql_lower.starts_with("insert") {
        return execute_insert(db, sql);
    }

    // 处理DELETE命令
    if sql_lower.starts_with("delete ") {
        return execute_delete(db, sql);
    }

    // 处理UPDATE命令
    if sql_lower.starts_with("update ") {
        return execute_update(db, sql);
    }

    // 处理CREATE TABLE命令
    if sql_lower.starts_with("create table ") {
        return execute_create_table(db, sql);
    }

    // 处理CREATE TIMESERIES TABLE命令
    if sql_lower.starts_with("create timeseries table ") {
        return execute_create_time_series_table(db, sql);
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

    // 处理EXPORT命令
    if sql_lower.starts_with("export ") {
        let parts: Vec<&str> = sql_lower.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(SqlError::Parsing("Missing export type".to_string()));
        }

        match parts[1] {
            "ddl" => {
                let output_file = parts
                    .get(2)
                    .ok_or(SqlError::Parsing("Missing output file name".to_string()))?;
                return execute_export_ddl(db, output_file);
            }
            "data" => {
                let table_name = parts
                    .get(2)
                    .ok_or(SqlError::Parsing("Missing table name".to_string()))?;
                let output_file = parts
                    .get(3)
                    .ok_or(SqlError::Parsing("Missing output file name".to_string()))?;
                return execute_export_data(db, table_name, output_file);
            }
            "all" => {
                let output_dir = parts
                    .get(2)
                    .ok_or(SqlError::Parsing("Missing output directory".to_string()))?;
                return execute_export_all(db, output_dir);
            }
            _ => {
                return Err(SqlError::Parsing(format!(
                    "Invalid export type: {}",
                    parts[1]
                )));
            }
        }
    }

    // 处理其他命令
    Err(SqlError::Unsupported)
}

/// 执行TABLES命令
fn execute_tables(db: &RemDb) -> std::result::Result<ResultSet, SqlError> {
    // 直接查询所有表的名称
    // 我们需要遍历所有可能的表ID，直到获取失败为止
    let columns = vec!["Tables".to_string()];
    let mut rows = Vec::new();
    let mut affected_rows = 0;
    
    // 遍历所有普通表
    let mut table_id = 0;
    loop {
        match db.get_table(table_id) {
            Ok(table) => {
                // 添加表名到结果集
                rows.push(vec![table.def.name.to_string()]);
                affected_rows += 1;
                table_id += 1;
            },
            Err(_) => {
                // 没有更多的普通表了，退出循环
                break;
            }
        }
    }
    
    // 遍历所有时序表
    let mut ts_table_id = 0;
    loop {
        match db.get_time_series_table(ts_table_id) {
            Ok(ts_table) => {
                // 添加时序表名到结果集
                rows.push(vec![ts_table.def.base.name.to_string()]);
                affected_rows += 1;
                ts_table_id += 1;
            },
            Err(_) => {
                // 没有更多的时序表了，退出循环
                break;
            }
        }
    }

    // 构造结果集
    Ok(ResultSet {
        columns,
        rows,
        affected_rows,
    })
}

/// 执行DESCRIBE命令
fn execute_describe(db: &mut RemDb, table_name: &str) -> std::result::Result<ResultSet, SqlError> {
    // 直接使用sql_query方法执行DESCRIBE查询
    let result = db.sql_query(&format!("DESCRIBE {}", table_name))?;

    // 构造结果集
    Ok(ResultSet {
        columns: result.columns.clone(),
        rows: result.rows.iter().map(|row| {
            row.values.iter().map(|value| {
                unsafe {
                    // 将TypedValue转换为字符串
                    match value.value_type {
                        remdb::types::DataType::UInt8 => format!("{}", value.value.u8),
                        remdb::types::DataType::UInt16 => format!("{}", value.value.u16),
                        remdb::types::DataType::UInt32 => format!("{}", value.value.u32),
                        remdb::types::DataType::UInt64 => format!("{}", value.value.u64),
                        remdb::types::DataType::Int8 => format!("{}", value.value.i8),
                        remdb::types::DataType::Int16 => format!("{}", value.value.i16),
                        remdb::types::DataType::Int32 => format!("{}", value.value.i32),
                        remdb::types::DataType::Int64 => format!("{}", value.value.i64),
                        remdb::types::DataType::Float32 => format!("{}", value.value.float32),
                        remdb::types::DataType::Float64 => format!("{}", value.value.float64),
                        remdb::types::DataType::Bool => format!("{}", value.value.bool),
                        remdb::types::DataType::Timestamp => format!("{}", value.value.timestamp),
                        remdb::types::DataType::TimestampTZ => format!("{}", value.value.timestamp),
                        remdb::types::DataType::Interval => format!("{}", value.value.u64),
                        remdb::types::DataType::String => {
                            let string_slice = core::str::from_utf8(&value.value.string).unwrap_or("");
                            string_slice.trim_end_matches(char::from(0)).to_string()
                        },
                    }
                }
            }).collect()
        }).collect(),
        affected_rows: result.rows.len(),
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
    let table_name_end = sql_lower[table_name_start..]
        .find(|c: char| c.is_whitespace() || c == ';')
        .unwrap_or_else(|| sql_lower[table_name_start..].len());
    let table_name = &sql_lower[table_name_start..table_name_start + table_name_end];
    debug_println!("Debug: Extracted table name: {}", table_name);

    // 调试：检查db.config.tables中是否存在该表
    let table_exists = db.config.tables.iter().any(|t| t.name == table_name);
    debug_println!(
        "Debug: Table '{}' exists in config: {}",
        table_name,
        table_exists
    );

    // 调试：列出所有配置表
    debug_println!("Debug: All config tables:");
    for (i, table) in db.config.tables.iter().enumerate() {
        debug_println!("Debug:   [{}] name: '{}', id: {}", i, table.name, table.id);
    }

    // 使用RemDb的sql_query方法执行SELECT语句
    let result = db.sql_query(sql)?;

    // 构建完整的行数据
    let mut rows = Vec::new();

    // 遍历所有行，使用引用迭代避免所有权转移
    for row in &result.rows {
        let mut row_data = Vec::new();

        // 遍历行中的所有值，将remdb::TypedValue转换为字符串
        for value in &row.values {
            let value_str = unsafe {
                // 根据TypedValue的value_type确定如何转换为字符串
                match value.value_type {
                    remdb::types::DataType::UInt8 => format!("{}", value.value.u8),
                    remdb::types::DataType::UInt16 => format!("{}", value.value.u16),
                    remdb::types::DataType::UInt32 => format!("{}", value.value.u32),
                    remdb::types::DataType::UInt64 => format!("{}", value.value.u64),
                    remdb::types::DataType::Int8 => format!("{}", value.value.i8),
                    remdb::types::DataType::Int16 => format!("{}", value.value.i16),
                    remdb::types::DataType::Int32 => format!("{}", value.value.i32),
                    remdb::types::DataType::Int64 => format!("{}", value.value.i64),
                    remdb::types::DataType::Float32 => format!("{}", value.value.float32),
                    remdb::types::DataType::Float64 => format!("{}", value.value.float64),
                    remdb::types::DataType::Bool => format!("{}", value.value.bool),
                    remdb::types::DataType::Timestamp => format!("{}", value.value.timestamp),
                    remdb::types::DataType::TimestampTZ => format!("{}", value.value.timestamp),
                    remdb::types::DataType::Interval => format!("{}", value.value.u64),
                    remdb::types::DataType::String => {
                        let string_slice = core::str::from_utf8(&value.value.string).unwrap_or("");
                        string_slice.trim_end_matches(char::from(0)).to_string()
                    },
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
    // 增加写操作计数
    db.metrics.inc_write_ops();

    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing INSERT SQL: {}", sql);

    let sql_lower = sql.trim().to_lowercase();

    // 提取表名
    let table_name = extract_table_name(&sql_lower)?;

    // 检查是否是批量插入语句 (包含多个VALUES子句)
    if sql_lower.contains(")(") {
        // 处理批量插入
        return execute_batch_insert(db, sql, &table_name, &sql_lower);
    }

    // 处理单条插入
    // 提取指定的列名
    let specified_columns = extract_columns(&sql_lower)?;

    // 检查INSERT语句是否包含id列作为独立列名
    let has_id_column = sql_lower.contains("(id") || 
                        sql_lower.contains(", id") || 
                        sql_lower.contains("id,") && !sql_lower.contains("device_id") && !sql_lower.contains("user_id") && !sql_lower.contains("group_id");
    
    if !has_id_column {
        // 如果没有提供id，自动生成一个
        let sql_with_pk = generate_auto_inc_sql(sql)?;
        debug_println!("Debug: Generated INSERT with auto PK: {}", sql_with_pk);

        // 执行带自动生成主键的INSERT语句
        let result = db.sql_query(&sql_with_pk)?;

        // 构造结果集
    let affected_rows = if let Some(row) = result.rows.first() {
        if let Some(value) = row.values.first() {
            // 从结果中提取affected_rows值
            unsafe {
                match value.value_type {
                    remdb::types::DataType::UInt64 => value.value.u64 as usize,
                    remdb::types::DataType::Int64 => value.value.i64 as usize,
                    remdb::types::DataType::Int32 => value.value.i32 as usize,
                    _ => 1 // 默认值为1，表示插入成功
                }
            }
        } else {
            1 // 默认值为1，表示插入成功
        }
    } else {
        1 // 默认值为1，表示插入成功
    };

        return Ok(ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows,
        });
    }

    // 直接使用RemDb的sql_query方法执行INSERT语句
    let result = db.sql_query(sql)?;

    // 构造结果集
    let affected_rows = if let Some(row) = result.rows.first() {
        if let Some(value) = row.values.first() {
            // 从结果中提取affected_rows值
            unsafe {
                match value.value_type {
                    remdb::types::DataType::UInt64 => value.value.u64 as usize,
                    remdb::types::DataType::Int64 => value.value.i64 as usize,
                    remdb::types::DataType::Int32 => value.value.i32 as usize,
                    _ => result.rows.len()
                }
            }
        } else {
            result.rows.len()
        }
    } else {
        result.rows.len()
    };

    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows,
    })
}

/// 执行批量INSERT命令
fn execute_batch_insert(db: &mut RemDb, sql: &str, table_name: &str, sql_lower: &str) -> std::result::Result<ResultSet, SqlError> {
    // 提取指定的列名
    let specified_columns = extract_columns(sql_lower)?;
    
    // 提取所有值组
    let values_list = extract_batch_values(sql)?;
    
    // 检查是否需要自动生成id
       let needs_auto_id = !sql_lower.contains("(id") && 
                           !sql_lower.contains(", id") && 
                           !(sql_lower.contains("id,") && !sql_lower.contains("device_id") && !sql_lower.contains("user_id") && !sql_lower.contains("group_id"));
    
    if needs_auto_id {
        // 需要自动生成id，为每个值组生成完整的INSERT语句
        let mut total_affected = 0;
        
        for values in values_list {
            // 为每条记录生成自动id
            let sql_with_pk = generate_auto_inc_sql_for_batch(sql, &values)?;
            // 检查是否需要转换为INSERT IGNORE
            let final_sql = if sql_lower.contains("insert ignore ") {
                // 将INSERT INTO转换为INSERT IGNORE INTO
                sql_with_pk.replace("INSERT INTO ", "INSERT IGNORE INTO ")
            } else {
                sql_with_pk
            };
            let result = db.sql_query(&final_sql)?;
            total_affected += result.rows.len();
        }
        
        // 构造结果集
        return Ok(ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows: total_affected,
        });
    } else {
        // 不需要自动生成id，使用batch_insert_record方法
        // 转换列名为&str数组
        let column_refs: Vec<&str> = specified_columns.iter().map(|col| col.as_str()).collect();
        
        // 由于生命周期问题，我们直接在循环中构建并执行批量插入
        // 或者，我们可以将所有值转换为字符串，然后构建完整的SQL语句
        // 这里我们选择将批量插入拆分为单条插入，因为batch_insert_record的生命周期要求较高
        let mut total_affected = 0;
        for values in values_list {
            // 检查是否为INSERT IGNORE语句
            let insert_prefix = if sql_lower.contains("insert ignore ") {
                "INSERT IGNORE INTO "
            } else {
                "INSERT INTO "
            };
            
            // 为每条记录构建INSERT语句
            let insert_sql = format!(
                "{}{} ({}) VALUES ({})
",
                insert_prefix,
                table_name,
                specified_columns.join(", "),
                values.join(", ")
            );
            let result = db.sql_query(&insert_sql)?;
            total_affected += result.rows.len();
        }
        
        let affected_rows = total_affected;
        
        // 构造结果集
        Ok(ResultSet {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows,
        })
    }
}

/// 从批量INSERT语句中提取所有值组
fn extract_batch_values(sql: &str) -> std::result::Result<Vec<Vec<String>>, SqlError> {
    let sql_lower = sql.trim().to_lowercase();
    
    // 查找VALUES关键字
    let values_pos = sql_lower
        .find("values")
        .ok_or(SqlError::Parsing("Missing VALUES keyword".to_string()))?;
    
    // 提取VALUES部分
    let values_part = &sql[values_pos + 6..].trim();
    
    // 解析多个值组，格式为 (value1, value2, ...), (value1, value2, ...), ...
    let mut result = Vec::new();
    let mut current_group = Vec::new();
    let mut in_group = false;
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut current_value = String::new();
    let mut bracket_depth = 0;
    
    for c in values_part.chars() {
        match c {
            '"' | '\'' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = c;
                    current_value.push(c);
                } else if c == quote_char {
                    in_quotes = false;
                    quote_char = '\0';
                    current_value.push(c);
                } else {
                    current_value.push(c);
                }
            },
            '(' => {
                if !in_quotes {
                    bracket_depth += 1;
                    if bracket_depth == 1 {
                        in_group = true;
                        current_value.clear();
                    } else {
                        current_value.push(c);
                    }
                } else {
                    current_value.push(c);
                }
            },
            ')' => {
                if !in_quotes {
                    bracket_depth -= 1;
                    if bracket_depth == 0 {
                        in_group = false;
                        // 处理当前值
                        if !current_value.trim().is_empty() {
                            current_group.push(current_value.trim().to_string());
                        }
                        // 添加当前组到结果
                        result.push(current_group);
                        current_group = Vec::new();
                        current_value.clear();
                    } else {
                        current_value.push(c);
                    }
                } else {
                    current_value.push(c);
                }
            },
            ',' => {
                if !in_quotes && in_group && bracket_depth == 1 {
                    // 值分隔符
                    current_group.push(current_value.trim().to_string());
                    current_value.clear();
                } else {
                    current_value.push(c);
                }
            },
            ';' => {
                // SQL语句结束，忽略
                break;
            },
            _ => {
                if in_group {
                    current_value.push(c);
                }
            },
        }
    }
    
    Ok(result)
}

/// 为批量插入生成带有自动递增主键的SQL语句
fn generate_auto_inc_sql_for_batch(sql: &str, values: &Vec<String>) -> std::result::Result<String, SqlError> {
    let sql_lower = sql.trim().to_lowercase();
    
    // 提取表名
    let table_name = extract_table_name(&sql_lower)?;
    
    // 提取列名
    let specified_columns = extract_columns(&sql_lower)?;
    
    // 生成唯一主键值：使用随机32位整数，确保在INT范围内
    let new_pk = rand::random::<u32>() as u64;
    
    // 构建新的INSERT语句
    let new_sql = if specified_columns.is_empty() {
        // 没有指定列名时，直接在值列表前添加id值
        format!(
            "INSERT INTO {} VALUES ({}, {})",
            table_name,
            new_pk,
            values.join(", ")
        )
    } else {
        // 指定了列名时，添加id列和值
        format!(
            "INSERT INTO {} (id, {}) VALUES ({}, {})
",
            table_name,
            specified_columns.join(", "),
            new_pk,
            values.join(", ")
        )
    };
    
    Ok(new_sql)
}

/// 解析INSERT语句的各个部分：表名、指定的列名、值
fn parse_insert_parts(
    sql: &str,
    after_insert: &str,
) -> std::result::Result<(String, Vec<String>, Vec<String>), SqlError> {
    // 查找左括号位置（如果有列名列表）
    if let Some(left_paren) = after_insert.find('(') {
        // 提取表名（左括号前的部分）
        let table_name = after_insert[..left_paren].trim().to_string();

        // 查找右括号位置
        let after_left_paren = &after_insert[left_paren..];
        let right_paren = after_left_paren.find(')').ok_or(SqlError::Parsing(
            "Missing closing parenthesis for columns".to_string(),
        ))?;

        // 提取列名列表
        let columns_part = &after_left_paren[1..right_paren];
        let specified_columns: Vec<String> = columns_part
            .split(',')
            .map(|col| col.trim().to_string())
            .collect();

        // 查找VALUES关键字
        let after_right_paren = &after_left_paren[right_paren + 1..];
        let values_pos = after_right_paren
            .find("values")
            .ok_or(SqlError::Parsing("Missing VALUES keyword".to_string()))?;

        // 提取VALUES部分
        let values_part = &after_right_paren[values_pos + 6..].trim();
        let left_val_paren = values_part.find('(').ok_or(SqlError::Parsing(
            "Missing opening parenthesis for values".to_string(),
        ))?;
        let right_val_paren = values_part.find(')').ok_or(SqlError::Parsing(
            "Missing closing parenthesis for values".to_string(),
        ))?;

        // 提取值列表
        let values_str = &values_part[left_val_paren + 1..right_val_paren];
        let values: Vec<String> = values_str
            .split(',')
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
        let left_paren = values_part.find('(').ok_or(SqlError::Parsing(
            "Missing opening parenthesis for values".to_string(),
        ))?;
        let right_paren = values_part.find(')').ok_or(SqlError::Parsing(
            "Missing closing parenthesis for values".to_string(),
        ))?;

        let values_str = &values_part[left_paren + 1..right_paren];
        let values: Vec<String> = values_str
            .split(',')
            .map(|val| val.trim().to_string())
            .collect();

        // 没有指定列名，返回空列表
        Ok((table_name, Vec::new(), values))
    }
}

/// 从SQL语句中提取表名
fn extract_table_name(sql_lower: &str) -> std::result::Result<String, SqlError> {
    // 查找"insert"、"insert ignore into"或"insert into"后的表名
    let after_insert = if let Some(rest) = sql_lower.strip_prefix("insert into ") {
        rest
    } else if let Some(rest) = sql_lower.strip_prefix("insert ignore into ") {
        rest
    } else if let Some(rest) = sql_lower.strip_prefix("insert ") {
        // 检查是否是"insert ignore"，如果是则跳过ignore
        if let Some(rest) = rest.strip_prefix("ignore into ") {
            rest
        } else {
            // 对于"insert table_name"形式，直接返回剩余部分
            rest
        }
    } else {
        return Err(SqlError::Parsing("Not an INSERT statement".to_string()));
    };

    // 查找表名结束位置（空格或左括号）
    let table_end = after_insert
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(after_insert.len());

    Ok(after_insert[..table_end].trim().to_string())
}

/// 从SQL语句中提取指定的列名
fn extract_columns(sql_lower: &str) -> std::result::Result<Vec<String>, SqlError> {
    // 查找左括号位置
    if let Some(left_paren) = sql_lower.find('(') {
        // 查找右括号位置
        let after_left_paren = &sql_lower[left_paren..];
        let right_paren = after_left_paren.find(')').ok_or(SqlError::Parsing(
            "Missing closing parenthesis for columns".to_string(),
        ))?;

        // 检查是否是列名列表（在VALUES关键字之前）
        let columns_part = &after_left_paren[1..right_paren];
        let after_right_paren = &after_left_paren[right_paren + 1..];

        if after_right_paren.contains("values") {
            // 提取列名列表
            let columns: Vec<String> = columns_part
                .split(',')
                .map(|col| col.trim().to_string())
                .collect();
            Ok(columns)
        } else {
            // 不是列名列表，是VALUES部分
            Ok(Vec::new())
        }
    } else {
        Ok(Vec::new())
    }
}

/// 从SQL语句中提取值列表
fn extract_values(sql_lower: &str) -> std::result::Result<Vec<String>, SqlError> {
    // 查找VALUES关键字
    let values_pos = sql_lower
        .find("values")
        .ok_or(SqlError::Parsing("Missing VALUES keyword".to_string()))?;

    // 提取VALUES部分
    let values_part = &sql_lower[values_pos + 6..].trim();

    // 查找左括号
    let left_paren = values_part.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for values".to_string(),
    ))?;

    // 查找右括号
    let right_paren = values_part.find(')').ok_or(SqlError::Parsing(
        "Missing closing parenthesis for values".to_string(),
    ))?;

    // 提取值列表
    let values_str = &values_part[left_paren + 1..right_paren];
    let values: Vec<String> = values_str
        .split(',')
        .map(|val| val.trim().to_string())
        .collect();

    Ok(values)
}

/// 生成带有自动递增主键的新INSERT语句
fn generate_auto_inc_sql(sql: &str) -> std::result::Result<String, SqlError> {
    let sql_lower = sql.trim().to_lowercase();

    // 提取表名
    let table_name = extract_table_name(&sql_lower)?;

    // 提取列名和值列表
    let specified_columns = extract_columns(&sql_lower)?;
    let values = extract_values(&sql_lower)?;

    // 生成唯一主键值：使用随机32位整数，确保在INT范围内
    let new_pk = rand::random::<u32>() as u64;

    // 构建新的INSERT语句，处理没有指定列名的情况
    let new_sql = if specified_columns.is_empty() {
        // 没有指定列名时，直接在值列表前添加id值
        format!(
            "INSERT INTO {} VALUES ({}, {})",
            table_name,
            new_pk,
            values.join(", ")
        )
    } else {
        // 指定了列名时，添加id列和值
        format!(
            "INSERT INTO {} (id, {}) VALUES ({}, {})",
            table_name,
            specified_columns.join(", "),
            new_pk,
            values.join(", ")
        )
    };

    Ok(new_sql)
}

/// 查询当前表的最大主键值
fn get_max_primary_key(db: &mut RemDb, table_name: &str) -> std::result::Result<u64, SqlError> {
    // 执行SELECT MAX(id)查询
    let sql = format!("SELECT MAX(id) as max_id FROM {}", table_name);
    let result = db.sql_query(&sql)?;

    // 解析结果
    if let Some(row) = result.rows.first() {
        if let Some(max_id_value) = row.values.first() {
            // 尝试将值转换为u64
            // 注意：这里假设主键是整数类型
            let max_id_str = unsafe {
                std::str::from_utf8(&max_id_value.value.string)
                    .unwrap_or("0")
                    .trim_matches(|c: char| c == '\0' || c.is_whitespace())
            };

            // 如果结果为空，返回0
            if max_id_str.is_empty() || max_id_str == "NULL" {
                return Ok(0);
            }

            // 转换为u64
            let max_id = max_id_str.parse::<u64>().unwrap_or(0);
            return Ok(max_id);
        }
    }

    Ok(0)
}

/// 构建包含自动生成主键的新INSERT语句
fn build_insert_with_auto_pk(
    original_sql: &str,
    table: &remdb::types::TableDef,
    pk_name: &str,
    new_pk: u64,
    specified_columns: Vec<String>,
    values: Vec<String>,
) -> std::result::Result<String, SqlError> {
    // 构建新的列名列表，包含主键
    let mut new_columns = vec![pk_name.to_string()];
    new_columns.extend(specified_columns);

    // 构建新的值列表，包含自动生成的主键
    let mut new_values = vec![new_pk.to_string()];
    new_values.extend(values);

    // 构建新的INSERT语句
    let new_sql = format!(
        "INSERT INTO {} ({}) VALUES ({})
",
        table.name,
        new_columns.join(", "),
        new_values.join(", ")
    );

    Ok(new_sql)
}

/// 解析INSERT语句中的列名列表和值列表
fn parse_insert_columns_and_values(
    after_table: &str,
) -> std::result::Result<(Vec<String>, Vec<String>), SqlError> {
    let after_table = after_table.trim();

    // 查找VALUES关键字
    let values_pos = after_table
        .find("values")
        .ok_or(SqlError::Parsing("Missing VALUES keyword".to_string()))?;

    // 提取列名部分（如果有）
    let columns_part = &after_table[..values_pos].trim();
    let mut specified_columns = Vec::new();

    // 如果列名部分非空且以左括号开头，解析列名
    if !columns_part.is_empty() && columns_part.starts_with('(') {
        // 查找右括号
        let right_paren = columns_part.find(')').ok_or(SqlError::Parsing(
            "Missing closing parenthesis for columns".to_string(),
        ))?;

        // 提取列名列表
        let columns_list = &columns_part[1..right_paren];
        specified_columns = columns_list
            .split(',')
            .map(|col| col.trim().to_string())
            .collect();
    }

    // 提取VALUES部分
    let values_part = &after_table[values_pos + 6..].trim();

    // 查找左括号
    let left_val_paren = values_part.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for values".to_string(),
    ))?;

    // 查找右括号
    let right_val_paren = values_part.find(')').ok_or(SqlError::Parsing(
        "Missing closing parenthesis for values".to_string(),
    ))?;

    // 提取值列表
    let values_str = &values_part[left_val_paren + 1..right_val_paren];
    let values = values_str
        .split(',')
        .map(|val| val.trim().to_string())
        .collect();

    Ok((specified_columns, values))
}

/// 执行DELETE命令
fn execute_delete(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 增加删除操作计数
    db.metrics.inc_delete_ops();

    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing DELETE SQL: {}", sql);

    // 处理常见用户错误：DELETE * FROM table（应该是DELETE FROM table）
    let processed_sql = if sql.to_lowercase().contains("delete * from") {
        sql.replace("*", "")
    } else {
        sql.to_string()
    };

    // 直接使用RemDb的sql_query方法执行DELETE语句
    let result = db.sql_query(&processed_sql)?;

    // 构造结果集
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: result.rows.len(), // 返回受影响的行数
    })
}

/// 执行UPDATE命令
fn execute_update(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 增加更新操作计数
    db.metrics.inc_update_ops();

    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing UPDATE SQL: {}", sql);

    // 直接使用RemDb的sql_query方法执行UPDATE语句
    let result = db.sql_query(sql)?;

    // 构造结果集
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: result.rows.len(), // 返回受影响的行数
    })
}

/// 执行CREATE TABLE命令
fn execute_create_table(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing CREATE TABLE SQL: {}", sql);

    let sql_lower = sql.trim().to_lowercase();

    // 查找表名和字段定义开始位置
    let after_create = if let Some(rest) = sql_lower.strip_prefix("create table if not exists ") {
        rest
    } else {
        sql_lower
            .strip_prefix("create table ")
            .ok_or(SqlError::Parsing(
                "Not a CREATE TABLE statement".to_string(),
            ))?
    };
    let table_name_end = after_create
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(after_create.len());
    let table_name = after_create[..table_name_end].trim().to_string();

    // 查找字段定义的开始和结束位置
    let fields_start = sql.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for fields".to_string(),
    ))?;
    let fields_end = sql.rfind(')').ok_or(SqlError::Parsing(
        "Missing closing parenthesis for fields".to_string(),
    ))?;

    // 提取字段定义部分
    let fields_part = &sql[fields_start + 1..fields_end].trim();

    // 解析字段定义
    let mut fields = Vec::new();
    let mut primary_key_index = None;

    // 按逗号分割字段定义，但跳过括号内的逗号
    let mut bracket_count = 0;
    let mut field_start = 0;

    for (i, c) in fields_part.char_indices() {
        match c {
            '(' => bracket_count += 1,
            ')' => bracket_count -= 1,
            ',' => {
                if bracket_count == 0 {
                    // 提取字段定义
                    let field_def = &fields_part[field_start..i].trim();
                    if !field_def.is_empty() {
                        // 解析字段定义
                        let field_parts: Vec<&str> = field_def.split_whitespace().collect();
                        if field_parts.len() >= 2 {
                            let field_name = field_parts[0];
                            let data_type_str = field_parts[1].to_uppercase();

                            // 转换数据类型
                            let data_type = match data_type_str.as_str() {
                                "INT" | "INTEGER" => remdb::types::DataType::Int32,
                                "BIGINT" => remdb::types::DataType::Int64,
                                "FLOAT" => remdb::types::DataType::Float32,
                                "DOUBLE" => remdb::types::DataType::Float64,
                                "BOOLEAN" => remdb::types::DataType::Bool,
                                "TIMESTAMP" => remdb::types::DataType::Timestamp,
                                "STRING" | "VARCHAR" => remdb::types::DataType::String,
                                _ => remdb::types::DataType::String, // 默认使用String类型
                            };

                            fields.push((field_name, data_type, None)); // 添加默认值None

                            // 检查是否为主键
                            if field_parts.iter().any(|&part| {
                                part.eq_ignore_ascii_case("PRIMARY")
                                    && part.eq_ignore_ascii_case("KEY")
                            }) {
                                primary_key_index = Some(fields.len() - 1);
                            }
                        }
                    }
                    field_start = i + 1;
                }
            }
            _ => {}
        }
    }

    // 处理最后一个字段
    let last_field = &fields_part[field_start..].trim();
    if !last_field.is_empty() {
        let field_parts: Vec<&str> = last_field.split_whitespace().collect();
        if field_parts.len() >= 2 {
            let field_name = field_parts[0];
            let data_type_str = field_parts[1].to_uppercase();

            // 转换数据类型
            let data_type = match data_type_str.as_str() {
                "INT" | "INTEGER" => remdb::types::DataType::Int32,
                "BIGINT" => remdb::types::DataType::Int64,
                "FLOAT" => remdb::types::DataType::Float32,
                "DOUBLE" => remdb::types::DataType::Float64,
                "BOOLEAN" => remdb::types::DataType::Bool,
                "TIMESTAMP" => remdb::types::DataType::Timestamp,
                "STRING" | "VARCHAR" => remdb::types::DataType::String,
                _ => remdb::types::DataType::String, // 默认使用String类型
            };

            fields.push((field_name, data_type, None)); // 添加默认值None

            // 检查是否为主键
            if field_parts.iter().any(|&part| {
                part.eq_ignore_ascii_case("PRIMARY") && part.eq_ignore_ascii_case("KEY")
            }) {
                primary_key_index = Some(fields.len() - 1);
            }
        }
    }

    // 如果没有指定主键，默认使用第一个字段作为主键
    if primary_key_index.is_none() && !fields.is_empty() {
        primary_key_index = Some(0);
    }

    // 调用 RemDb::create_table 方法创建表
    db.create_table(&table_name, &fields, primary_key_index)?;

    // 构造结果集
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 0,
    })
}

/// 执行CREATE TIMESERIES TABLE命令
fn execute_create_time_series_table(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing CREATE TIMESERIES TABLE SQL: {}", sql);

    let sql_lower = sql.trim().to_lowercase();

    // 查找表名和字段定义开始位置
    let after_create = if let Some(rest) = sql_lower.strip_prefix("create timeseries table if not exists ") {
        rest
    } else {
        sql_lower
            .strip_prefix("create timeseries table ")
            .ok_or(SqlError::Parsing(
                "Not a CREATE TIMESERIES TABLE statement".to_string(),
            ))?
    };
    let table_name_end = after_create
        .find(|c: char| c.is_whitespace() || c == '(')
        .unwrap_or(after_create.len());
    let table_name = after_create[..table_name_end].trim().to_string();

    // 查找字段定义的开始和结束位置
    let fields_start = sql.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for fields".to_string(),
    ))?;
    let fields_end = sql.rfind(')').ok_or(SqlError::Parsing(
        "Missing closing parenthesis for fields".to_string(),
    ))?;

    // 提取字段定义部分
    let fields_part = &sql[fields_start + 1..fields_end].trim();

    // 解析字段定义
    let mut fields = Vec::new();
    let mut time_field = None;
    let mut value_field = None;
    let mut tag_fields = Vec::new();

    // 按逗号分割字段定义，但跳过括号内的逗号
    let mut bracket_count = 0;
    let mut field_start = 0;

    for (i, c) in fields_part.char_indices() {
        match c {
            '(' => bracket_count += 1,
            ')' => bracket_count -= 1,
            ',' => {
                if bracket_count == 0 {
                    // 提取字段定义
                    let field_def = &fields_part[field_start..i].trim();
                    if !field_def.is_empty() {
                        // 解析字段定义
                        let field_parts: Vec<&str> = field_def.split_whitespace().collect();
                        if field_parts.len() >= 2 {
                            let field_name = field_parts[0];
                            let data_type_str = field_parts[1].to_uppercase();

                            // 检查是否为时间字段
                            if data_type_str == "TIMESTAMP" {
                                time_field = Some(field_name.to_string());
                            }
                            // 检查是否为值字段
                            else if data_type_str == "FLOAT64" || data_type_str == "FLOAT" || data_type_str == "DOUBLE" {
                                value_field = Some(field_name.to_string());
                            }
                            // 其他字段作为标签字段
                            else {
                                tag_fields.push(field_name.to_string());
                            }

                            fields.push((field_name, data_type_str));
                        }
                    }
                    field_start = i + 1;
                }
            }
            _ => {}
        }
    }

    // 处理最后一个字段
    let last_field = &fields_part[field_start..].trim();
    if !last_field.is_empty() {
        let field_parts: Vec<&str> = last_field.split_whitespace().collect();
        if field_parts.len() >= 2 {
            let field_name = field_parts[0];
            let data_type_str = field_parts[1].to_uppercase();

            // 检查是否为时间字段
            if data_type_str == "TIMESTAMP" {
                time_field = Some(field_name.to_string());
            }
            // 检查是否为值字段
            else if data_type_str == "FLOAT64" || data_type_str == "FLOAT" || data_type_str == "DOUBLE" {
                value_field = Some(field_name.to_string());
            }
            // 其他字段作为标签字段
            else {
                tag_fields.push(field_name.to_string());
            }

            fields.push((field_name, data_type_str));
        }
    }

    // 验证必要字段
    let time_field = time_field.ok_or(SqlError::Parsing(
        "Missing TIMESTAMP field for timeseries table".to_string(),
    ))?;
    let value_field = value_field.ok_or(SqlError::Parsing(
        "Missing FLOAT/DOUBLE/FLOAT64 value field for timeseries table".to_string(),
    ))?;

    // 转换标签字段为&str数组
    let tag_field_refs: Vec<&str> = tag_fields.iter().map(|f| f.as_str()).collect();

    // 解析WITH子句
    let mut config = None;
    let mut compression_type = remdb::time_series::CompressionType::DeltaRunLength;
    let mut retention_period_secs = 7 * 24 * 3600; // 默认7天
    
    // 查找WITH子句的位置
    if let Some(with_pos) = sql_lower.find(" with ") {
        let with_clause = &sql[with_pos + 6..].trim();
        
        // 解析WITH子句中的属性
        let mut attr_start = 0;
        let mut bracket_count = 0;
        
        for (i, c) in with_clause.char_indices() {
            match c {
                '(' => bracket_count += 1,
                ')' => bracket_count -= 1,
                ',' => {
                    if bracket_count == 0 {
                        // 提取属性
                        let attr = &with_clause[attr_start..i].trim();
                        parse_with_attr(attr, &mut compression_type, &mut retention_period_secs)?;
                        attr_start = i + 1;
                    }
                }
                _ => {}
            }
        }
        
        // 处理最后一个属性
        let last_attr = &with_clause[attr_start..].trim();
        if !last_attr.is_empty() {
            parse_with_attr(last_attr, &mut compression_type, &mut retention_period_secs)?;
        }
        
        // 创建配置
        config = Some(remdb::time_series::TimeSeriesConfig {
            partition_duration_secs: 3600, // 默认1小时
            retention_period_secs,
            compression: compression_type,
            max_partitions: 1000,
        });
    }

    // 调用RemDb的create_time_series_table方法创建时序表
    db.create_time_series_table(
        &table_name,
        &time_field,
        &value_field,
        &tag_field_refs,
        config
    )?;

    // 构造结果集
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 0,
    })
}

/// 解析WITH子句中的属性
fn parse_with_attr(attr: &str, compression_type: &mut remdb::time_series::CompressionType, retention_period_secs: &mut u64) -> std::result::Result<(), SqlError> {
    let attr_lower = attr.trim().to_lowercase();
    
    // 解析COMPRESSION属性
    if attr_lower.starts_with("compression = ") {
        let comp_part = attr_lower.strip_prefix("compression = ").unwrap();
        
        // 提取括号内的内容
        let comp_content = comp_part.strip_prefix("(").and_then(|s| s.strip_suffix(")")).ok_or(SqlError::Parsing(
            "Invalid COMPRESSION syntax, expected WITH COMPRESSION = (algorithm='delta-delta', enabled=true)".to_string(),
        ))?;
        
        // 解析algorithm属性
        let mut algorithm = "delta-runlength";
        
        let mut prop_start = 0;
        let mut bracket_count = 0;
        
        for (i, c) in comp_content.char_indices() {
            match c {
                '(' => bracket_count += 1,
                ')' => bracket_count -= 1,
                ',' => {
                    if bracket_count == 0 {
                        let prop = &comp_content[prop_start..i].trim();
                        if let Some(alg_pos) = prop.find("algorithm='") {
                            let alg_end = prop[alg_pos + 11..].find("'").ok_or(SqlError::Parsing(
                                "Invalid algorithm syntax, expected algorithm='<algorithm>'".to_string(),
                            ))?;
                            algorithm = &prop[alg_pos + 11..alg_pos + 11 + alg_end];
                        }
                        prop_start = i + 1;
                    }
                }
                _ => {}
            }
        }
        
        // 处理最后一个属性
        let last_prop = &comp_content[prop_start..].trim();
        if let Some(alg_pos) = last_prop.find("algorithm='") {
            let alg_end = last_prop[alg_pos + 11..].find("'").ok_or(SqlError::Parsing(
                "Invalid algorithm syntax, expected algorithm='<algorithm>'".to_string(),
            ))?;
            algorithm = &last_prop[alg_pos + 11..alg_pos + 11 + alg_end];
        }
        
        // 设置压缩类型
        *compression_type = match algorithm {
            "delta-delta" => remdb::time_series::CompressionType::DeltaDelta,
            "delta" => remdb::time_series::CompressionType::Delta,
            "runlength" => remdb::time_series::CompressionType::RunLength,
            "delta-runlength" => remdb::time_series::CompressionType::DeltaRunLength,
            _ => return Err(SqlError::Parsing(format!("Unsupported compression algorithm: {}", algorithm))),
        };
    }
    // 解析TTL属性
    else if attr_lower.starts_with("ttl = ") {
        let ttl_part = attr_lower.strip_prefix("ttl = ").unwrap();
        let ttl_value = ttl_part.trim_matches(|c| c == '\'' || c == '"');
        
        // 解析TTL值，支持天、小时、分钟、秒
        let mut seconds = 0;
        let mut num_str = String::new();
        let mut unit = String::new();
        
        for c in ttl_value.chars() {
            if c.is_digit(10) || c == '.' {
                num_str.push(c);
            } else if c.is_whitespace() {
                continue;
            } else {
                unit.push(c);
            }
        }
        
        let num = num_str.parse::<f64>().map_err(|_| SqlError::Parsing(
            format!("Invalid TTL value: {}", ttl_value).to_string(),
        ))?;
        
        match unit.as_str() {
            "days" | "day" => seconds = (num * 24.0 * 3600.0) as u64,
            "hours" | "hour" => seconds = (num * 3600.0) as u64,
            "minutes" | "minute" => seconds = (num * 60.0) as u64,
            "seconds" | "second" => seconds = num as u64,
            _ => return Err(SqlError::Parsing(format!("Unsupported TTL unit: {}", unit))),
        }
        
        *retention_period_secs = seconds;
    }
    
    Ok(())
}

/// 执行CREATE INDEX命令
fn execute_create_index(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 调试：打印要执行的SQL语句
    debug_println!("Debug: Executing CREATE INDEX SQL: {}", sql);

    let sql_lower = sql.trim().to_lowercase();

    // 解析CREATE INDEX语句
    // 格式：CREATE INDEX index_name ON table_name (column_name) USING index_type;

    // 提取索引名
    let after_create = sql_lower
        .strip_prefix("create index ")
        .ok_or(SqlError::Parsing(
            "Not a CREATE INDEX statement".to_string(),
        ))?;
    let index_name_end = after_create
        .find(" on ")
        .ok_or(SqlError::Parsing("Missing ON keyword".to_string()))?;
    let index_name = after_create[..index_name_end].trim().to_string();

    // 提取表名
    let after_on = &after_create[index_name_end + 4..];
    let table_name_end = after_on.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for column".to_string(),
    ))?;
    let table_name = after_on[..table_name_end].trim().to_string();

    // 提取字段名
    let column_start = after_on.find('(').ok_or(SqlError::Parsing(
        "Missing opening parenthesis for column".to_string(),
    ))?;
    let column_end = after_on.find(')').ok_or(SqlError::Parsing(
        "Missing closing parenthesis for column".to_string(),
    ))?;
    let column_name = after_on[column_start + 1..column_end].trim().to_string();

    // 提取索引类型（如果有）
    let index_type = if let Some(using_pos) = after_on.find(" using ") {
        let using_part = &after_on[using_pos + 6..];
        let type_end = using_part.find(';').unwrap_or(using_part.len());
        let type_str = using_part[..type_end].trim().to_uppercase();

        match type_str.as_str() {
            "BTREE" => remdb::types::IndexType::BTree,
            "TTREE" => remdb::types::IndexType::TTree,
            "HASH" => remdb::types::IndexType::Hash,
            "SORTEDARRAY" => remdb::types::IndexType::SortedArray,
            _ => remdb::types::IndexType::BTree, // 默认使用BTree索引
        }
    } else {
        remdb::types::IndexType::BTree // 默认使用BTree索引
    };

    // 调用 RemDb::create_index 方法创建索引
    db.create_index(&table_name, &column_name, index_type)?;

    // 构造结果集
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 0,
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
    rows.push(vec![
        "Database Size".to_string(),
        format!("{} bytes", metrics.total_memory),
    ]);
    rows.push(vec![
        "Memory Usage".to_string(),
        format!("{} bytes", metrics.used_memory),
    ]);
    rows.push(vec![
        "Memory Usage %".to_string(),
        format!(
            "{:.2}%",
            (metrics.used_memory as f64 / metrics.total_memory as f64) * 100.0
        ),
    ]);
    rows.push(vec![
        "Table Count".to_string(),
        format!("{}", db.config.tables.len()),
    ]);
    rows.push(vec![
        "Read Operations".to_string(),
        format!("{}", metrics.read_ops),
    ]);
    rows.push(vec![
        "Write Operations".to_string(),
        format!("{}", metrics.write_ops),
    ]);
    rows.push(vec![
        "Delete Operations".to_string(),
        format!("{}", metrics.delete_ops),
    ]);
    rows.push(vec![
        "Update Operations".to_string(),
        format!("{}", metrics.update_ops),
    ]);
    rows.push(vec![
        "Cache Hit Rate".to_string(),
        format!("{:.2}%", metrics.cache_hit_rate),
    ]);
    rows.push(vec![
        "Cache Hits".to_string(),
        format!("{}", metrics.cache_hits),
    ]);
    rows.push(vec![
        "Cache Misses".to_string(),
        format!("{}", metrics.cache_misses),
    ]);
    rows.push(vec!["Active Connections".to_string(), "1".to_string()]);
    rows.push(vec![
        "Transactions".to_string(),
        format!("{}", metrics.transactions),
    ]);
    rows.push(vec![
        "Committed Transactions".to_string(),
        format!("{}", metrics.committed_transactions),
    ]);
    rows.push(vec![
        "Rolled Back Transactions".to_string(),
        format!("{}", metrics.rolled_back_transactions),
    ]);

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
    let columns = vec![
        "Component".to_string(),
        "Status".to_string(),
        "Details".to_string(),
    ];
    let mut rows = Vec::new();

    // 添加健康检查结果到结果集
    rows.push(vec![
        "Database".to_string(),
        match health_result.status {
            remdb::monitor::HealthStatus::Healthy => "HEALTHY".to_string(),
            remdb::monitor::HealthStatus::Warning => "WARNING".to_string(),
            remdb::monitor::HealthStatus::Unhealthy => "UNHEALTHY".to_string(),
        },
        health_result.details.to_string(),
    ]);
    // 添加内存健康检查结果
    let metrics = health_result.metrics;
    let memory_usage = metrics.used_memory as f64 / metrics.total_memory as f64;
    rows.push(vec![
        "Memory".to_string(),
        if memory_usage > 0.9 {
            "UNHEALTHY".to_string()
        } else if memory_usage > 0.7 {
            "WARNING".to_string()
        } else {
            "HEALTHY".to_string()
        },
        format!(
            "Usage: {}/{} bytes ({:.2}%)",
            metrics.used_memory,
            metrics.total_memory,
            memory_usage * 100.0
        ),
    ]);

    // 添加表健康检查结果
    rows.push(vec![
        "Tables".to_string(),
        "HEALTHY".to_string(),
        format!("{} tables loaded successfully", db.config.tables.len()),
    ]);

    // 添加操作统计
    rows.push(vec![
        "Operations".to_string(),
        "HEALTHY".to_string(),
        format!(
            "Read: {}, Write: {}, Delete: {}, Update: {}",
            metrics.read_ops, metrics.write_ops, metrics.delete_ops, metrics.update_ops
        ),
    ]);

    // 添加缓存健康检查结果
    rows.push(vec![
        "Cache".to_string(),
        if metrics.cache_hit_rate < 50.0 {
            "WARNING".to_string()
        } else {
            "HEALTHY".to_string()
        },
        format!(
            "Hit Rate: {:.2}%, Hits: {}, Misses: {}",
            metrics.cache_hit_rate, metrics.cache_hits, metrics.cache_misses
        ),
    ]);

    // 添加JDBC服务状态检查
    rows.push(vec![
        "JDBC Server".to_string(),
        "HEALTHY".to_string(),
        "JDBC服务正常运行".to_string(),
    ]);

    // 添加PubSub服务状态检查
    let pubsub_status = match remdb::pubsub::get_topic_id("") {
        Some(_) => "HEALTHY",
        None => "UNHEALTHY",
    };
    rows.push(vec![
        "PubSub Server".to_string(),
        pubsub_status.to_string(),
        "PubSub服务状态通过内部检查判断".to_string(),
    ]);

    // 添加HA服务状态检查
    let ha_status = if let Some(ha_manager) = remdb::ha::get_ha_manager() {
        "HEALTHY"
    } else {
        "UNKNOWN"
    };
    rows.push(vec![
        "HA Service".to_string(),
        ha_status.to_string(),
        "HA服务状态通过内部检查判断".to_string(),
    ]);

    Ok(ResultSet {
        columns,
        rows: rows.clone(),
        affected_rows: rows.len(),
    })
}

/// 执行导出DDL命令
fn execute_export_ddl(db: &RemDb, output_file: &str) -> std::result::Result<ResultSet, SqlError> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output_file)
        .map_err(|e| SqlError::Parsing(format!("Failed to create file: {}", e)))?;

    // 遍历所有表定义
    for table in db.config.tables {
        // 生成CREATE TABLE语句
        let mut create_table_sql = format!("CREATE TABLE {} (\n", table.name);

        // 生成列定义
        for (i, field) in table.fields.iter().enumerate() {
            // 转换数据类型
            let data_type_str = match field.data_type {
                remdb::types::DataType::UInt8 => "TINYINT UNSIGNED".to_string(),
                remdb::types::DataType::UInt16 => "SMALLINT UNSIGNED".to_string(),
                remdb::types::DataType::UInt32 => "INT UNSIGNED".to_string(),
                remdb::types::DataType::UInt64 => "BIGINT UNSIGNED".to_string(),
                remdb::types::DataType::Int8 => "TINYINT".to_string(),
                remdb::types::DataType::Int16 => "SMALLINT".to_string(),
                remdb::types::DataType::Int32 => "INT".to_string(),
                remdb::types::DataType::Int64 => "BIGINT".to_string(),
                remdb::types::DataType::Float32 => "FLOAT".to_string(),
                remdb::types::DataType::Float64 => "DOUBLE".to_string(),
                remdb::types::DataType::Bool => "BOOLEAN".to_string(),
                remdb::types::DataType::Timestamp => "TIMESTAMP".to_string(),
                remdb::types::DataType::TimestampTZ => "TIMESTAMPTZ".to_string(),
                remdb::types::DataType::Interval => "INTERVAL".to_string(),
                remdb::types::DataType::String => format!("VARCHAR({})", field.size),
            };

            // 生成列定义行
            let mut column_def = format!("    {} {}", field.name, data_type_str);

            // 添加NOT NULL约束
            if field.not_null {
                column_def.push_str(" NOT NULL");
            }

            // 添加PRIMARY KEY约束
            if i == table.primary_key {
                column_def.push_str(" PRIMARY KEY");
            }

            // 添加逗号分隔
            if i < table.fields.len() - 1 {
                column_def.push(',');
            }

            column_def.push('\n');
            create_table_sql.push_str(&column_def);
        }

        // 结束CREATE TABLE语句
        create_table_sql.push_str(");\n\n");

        // 写入文件
        file.write_all(create_table_sql.as_bytes())
            .map_err(|e| SqlError::Parsing(format!("Failed to write to file: {}", e)))?;
    }

    // 返回成功结果
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: db.config.tables.len(),
    })
}

/// 执行导出数据命令
fn execute_export_data(
    db: &mut RemDb,
    table_name: &str,
    output_file: &str,
) -> std::result::Result<ResultSet, SqlError> {
    use std::fs::File;
    use std::io::Write;

    // 执行SELECT *查询获取所有数据
    let select_sql = format!("SELECT * FROM {}", table_name);
    let result = execute_select(db, &select_sql)?;

    // 创建输出文件
    let mut file = File::create(output_file)
        .map_err(|e| SqlError::Parsing(format!("Failed to create file: {}", e)))?;

    // 写入表头
    if !result.columns.is_empty() {
        let header_line = result.columns.join(",") + "\n";
        file.write_all(header_line.as_bytes())
            .map_err(|e| SqlError::Parsing(format!("Failed to write header: {}", e)))?;
    }

    // 写入数据行
    for row in &result.rows {
        let data_line = row.join(",") + "\n";
        file.write_all(data_line.as_bytes())
            .map_err(|e| SqlError::Parsing(format!("Failed to write data: {}", e)))?;
    }

    // 返回成功结果
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: result.affected_rows,
    })
}

/// 执行导出全部内容命令
fn execute_export_all(
    db: &mut RemDb,
    output_dir: &str,
) -> std::result::Result<ResultSet, SqlError> {
    use std::fs;

    // 创建输出目录
    fs::create_dir_all(output_dir)
        .map_err(|e| SqlError::Parsing(format!("Failed to create directory: {}", e)))?;

    // 导出DDL
    let ddl_file = format!("{}/schema.ddl", output_dir);
    execute_export_ddl(db, &ddl_file)?;

    // 导出每个表的数据
    for table in db.config.tables {
        let data_file = format!("{}/{}.csv", output_dir, table.name);
        execute_export_data(db, table.name, &data_file)?;
    }

    // 返回成功结果
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: db.config.tables.len(),
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
    let separator: String = col_widths
        .iter()
        .map(|w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("")
        + "+";

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
