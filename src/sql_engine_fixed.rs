/// 执行扩展的SQL命令，支持多行、注释和分号分隔
pub fn execute_extended_sql(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
    // 处理多行SQL语句，支持分号分隔和注释
    let lines: Vec<&str> = sql.lines().collect();
    let mut current_statement = String::new();
    
    for line in lines {
        let trimmed_line = line.trim();
        
        // 跳过空行和注释行
        if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
            continue;
        }
        
        // 添加当前行到语句
        current_statement.push_str(trimmed_line);
        current_statement.push(' ');
        
        // 如果语句以分号结束，执行它
        if trimmed_line.ends_with(';') {
            // 移除分号和多余空格
            let statement = current_statement.trim_end_matches(';').trim();
            if !statement.is_empty() {
                // 执行单个语句
                let result = execute_single_statement(db, statement);
                if result.is_err() {
                    return result;
                }
            }
            // 重置当前语句
            current_statement.clear();
        }
    }
    
    // 执行最后一个没有分号的语句
    let statement = current_statement.trim();
    if !statement.is_empty() {
        return execute_single_statement(db, statement);
    }
    
    // 如果没有执行任何语句，返回空结果
    Ok(ResultSet {
        columns: Vec::new(),
        rows: Vec::new(),
        affected_rows: 0,
    })
}

/// 执行单个SQL语句
fn execute_single_statement(db: &mut RemDb, sql: &str) -> std::result::Result<ResultSet, SqlError> {
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