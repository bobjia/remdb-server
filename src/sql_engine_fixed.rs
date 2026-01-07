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

    // 调试：打印结果基本信息
    debug_println!("Debug: Query result - rows: {}, columns: {}", result.rows.len(), result.columns.len());
    
    // 调试：打印列信息
    debug_println!("Debug: Columns:");
    for (i, col) in result.columns.iter().enumerate() {
        debug_println!("Debug:   [{}] name: '{}'", i, col);
    }

    // 构建完整的行数据
    let mut rows = Vec::new();

    // 遍历所有行，使用引用迭代避免所有权转移
    for (row_idx, row) in result.rows.iter().enumerate() {
        debug_println!("Debug: Row [{}] - values: {}", row_idx, row.values.len());
        
        let mut row_data = Vec::new();

        // 遍历行中的所有值，将remdb::TypedValue转换为字符串
        for (val_idx, value) in row.values.iter().enumerate() {
            debug_println!("Debug:   Value [{}] - type: {:?}", val_idx, value.value_type);
            
            let value_str = unsafe {
                // 调试：打印value的各个字段
                debug_println!("Debug:     Value fields - u8: {}, u16: {}, u32: {}, u64: {}, i8: {}, i16: {}, i32: {}, i64: {}, float32: {}, float64: {}, string: '{:?}'", 
                    value.value.u8, 
                    value.value.u16, 
                    value.value.u32, 
                    value.value.u64, 
                    value.value.i8, 
                    value.value.i16, 
                    value.value.i32, 
                    value.value.i64, 
                    value.value.float32, 
                    value.value.float64, 
                    core::str::from_utf8(&value.value.string).unwrap_or(""));
                
                // 对于聚合函数结果，我们需要特殊处理
                // 检查当前SQL是否包含聚合函数
                let sql_lower = sql.to_lowercase();
                let is_aggregation = sql_lower.contains("max(") || sql_lower.contains("min(") || 
                                    sql_lower.contains("count(") || sql_lower.contains("sum(") || 
                                    sql_lower.contains("avg(");
                
                if is_aggregation {
                    // 对于聚合函数，我们需要执行实际的聚合计算
                    // 但由于我们无法访问表的原始数据，我们需要采用另一种方法
                    // 让我们直接执行一个简单的查询，获取表中的所有数据，然后手动计算聚合结果
                    debug_println!("Debug:     Detected aggregation query, trying to get raw data");
                    
                    // 提取表名
                    let table_name_start = sql_lower.find("from ").map(|pos| pos + 5).unwrap_or(0);
                    let table_name_end = sql_lower[table_name_start..]
                        .find(|c: char| c.is_whitespace() || c == ';' || c == ')' || c == ',')
                        .unwrap_or_else(|| sql_lower[table_name_start..].len());
                    let table_name = &sql_lower[table_name_start..table_name_start + table_name_end];
                    
                    // 执行查询获取所有数据
                    let raw_sql = format!("SELECT * FROM {}", table_name);
                    debug_println!("Debug:     Executing raw query: {}", raw_sql);
                    let raw_result = db.sql_query(&raw_sql)?;
                    
                    // 获取当前列名，用于确定聚合类型
                    let current_col_name = &result.columns[val_idx];
                    debug_println!("Debug:     Current column: {}", current_col_name);
                    
                    // 提取聚合列名
                    let agg_col = {
                        // 查找第一个左括号和对应的右括号
                        let left_paren = sql_lower.find('(').unwrap();
                        let right_paren = sql_lower[left_paren..].find(')').unwrap() + left_paren;
                        &sql_lower[left_paren + 1..right_paren]
                    };
                    debug_println!("Debug:     Aggregation column: {}", agg_col);
                    
                    // 收集所有数值
                    let mut numeric_values = Vec::new();
                    for row in &raw_result.rows {
                        for (col_idx, col_name) in raw_result.columns.iter().enumerate() {
                            if col_name.to_lowercase() == agg_col {
                                let val = &row.values[col_idx];
                                // 收集所有数值字段
                                let vals = vec![
                                    val.value.u8 as i64,
                                    val.value.u16 as i64,
                                    val.value.u32 as i64,
                                    val.value.u64 as i64,
                                    val.value.i8 as i64,
                                    val.value.i16 as i64,
                                    val.value.i32 as i64,
                                    val.value.i64
                                ];
                                // 添加第一个非零值
                                if let Some(v) = vals.into_iter().find(|&x| x != 0) {
                                    numeric_values.push(v);
                                }
                            }
                        }
                    }
                    
                    // 根据列名执行相应的聚合计算
                    match current_col_name.as_str() {
                        "max" => {
                            let max_val = numeric_values.iter().max().unwrap_or(&0);
                            debug_println!("Debug:     Calculated max: {}", max_val);
                            format!("{}", max_val)
                        },
                        "min" => {
                            let min_val = numeric_values.iter().min().unwrap_or(&0);
                            debug_println!("Debug:     Calculated min: {}", min_val);
                            format!("{}", min_val)
                        },
                        "avg" => {
                            let sum: i64 = numeric_values.iter().sum();
                            let avg = if numeric_values.is_empty() {
                                0.0
                            } else {
                                sum as f64 / numeric_values.len() as f64
                            };
                            debug_println!("Debug:     Calculated avg: {}", avg);
                            format!("{}", avg)
                        },
                        _ => {
                            // 其他情况，使用默认处理
                            format!("{}", value.value.u64)
                        }
                    }
                } else {
                    // 非聚合查询，使用默认处理
                    match value.value_type {
                        remdb::types::DataType::Int8 => format!("{}", value.value.i8),
                        remdb::types::DataType::Int16 => format!("{}", value.value.i16),
                        remdb::types::DataType::Int32 => format!("{}", value.value.i32),
                        remdb::types::DataType::Int64 => format!("{}", value.value.i64),
                        remdb::types::DataType::UInt8 => format!("{}", value.value.u8),
                        remdb::types::DataType::UInt16 => format!("{}", value.value.u16),
                        remdb::types::DataType::UInt32 => format!("{}", value.value.u32),
                        remdb::types::DataType::UInt64 => format!("{}", value.value.u64),
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
            };
            
            debug_println!("Debug:     Converted to: '{}'", value_str);

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