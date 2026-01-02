use remdb::{types::{TableDef, FieldDef, DataType}, RemDb, Result as RemResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DdlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("DDL parsing error: {0}")]
    Parsing(String),
}

/// DDL列定义
struct DdlColumn {
    name: &'static str,
    data_type: DataType,
    size: usize,
    nullable: bool,
    primary_key: bool,
    auto_increment: bool,
}

/// DDL索引定义
struct DdlIndex {
    name: &'static str,
    table_name: &'static str,
    column_name: &'static str,
    index_type: remdb::types::IndexType,
}

/// 编译DDL文件，生成表定义
pub fn compile_ddl_file(file_path: &str) -> std::result::Result<Vec<TableDef>, DdlError> {
    // 读取DDL文件内容
    let content = std::fs::read_to_string(file_path)?;
    
    // 解析DDL内容
    parse_ddl_content(&content)
}

/// 解析DDL内容，生成表定义
pub fn parse_ddl_content(content: &str) -> std::result::Result<Vec<TableDef>, DdlError> {
    let mut tables = Vec::new();
    let mut table_id = 0;
    
    // 预处理：移除注释和空行，合并多行语句
    let mut processed_content = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        processed_content.push_str(line);
        processed_content.push(' ');
    }
    
    // 按分号分割语句
    let statements: Vec<&str> = processed_content.split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    println!("Debug: Found {} statements in DDL", statements.len());
    
    // 处理每个语句
    for (i, statement) in statements.iter().enumerate() {
        println!("Debug: Statement {}: {}", i, statement);
        let words: Vec<&str> = statement.split_whitespace().collect();
        if words.len() >= 3 && 
           (words[0].eq_ignore_ascii_case("CREATE") || words[0].eq_ignore_ascii_case("create")) && 
           (words[1].eq_ignore_ascii_case("TABLE") || words[1].eq_ignore_ascii_case("table")) {
            // 处理CREATE TABLE语句
            let table_line = statement;
            
            // 查找左括号，用于分离表名和列定义
            let left_paren = table_line.find('(')
                .ok_or(DdlError::Parsing("Invalid CREATE TABLE syntax: missing '('".to_string()))?;
            
            // 提取表名部分（CREATE TABLE和左括号之间的内容）
            let table_name_part = &table_line[12..left_paren]; // 跳过"CREATE TABLE "
            let table_name_str = table_name_part.trim().to_string();
            let table_name = Box::leak(Box::new(table_name_str)) as &'static str;
            
            // 提取列定义部分（左括号到右括号之间的内容）
            let columns_part = &table_line[left_paren..];
            
            // 查找右括号（使用rfind找到最后一个右括号，避免被VARCHAR(50)中的括号干扰）
            let right_paren = columns_part.rfind(')')
                .ok_or(DdlError::Parsing(format!("Invalid CREATE TABLE syntax: missing ')'")))?;
            
            // 提取括号内的列定义
            let columns_content = &columns_part[1..right_paren];
            
            // 解析列定义
            let mut columns = Vec::new();
            for column_str in columns_content.split(',') {
                let column_str = column_str.trim();
                if !column_str.is_empty() {
                    let column = parse_column_def(column_str)?;
                    columns.push(column);
                }
            }
            
            // 生成表定义
            let table = create_table_def(table_id, table_name, &columns)?;
            tables.push(table);
            table_id += 1;
        }
    }
    
    Ok(tables)
}

/// 解析索引定义
fn parse_index_def(line: &str) -> std::result::Result<DdlIndex, DdlError> {
    // 移除分号
    let line = line.trim_end_matches(';');
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 6 {
        return Err(DdlError::Parsing(format!("Invalid CREATE INDEX syntax: {}", line)));
    }
    
    // 提取索引名、表名和列名
    let index_name = parts[2];
    let table_name = parts[4];
    
    // 提取列名，处理括号
    let columns_part = line.split('(').nth(1).ok_or(DdlError::Parsing(format!("Invalid CREATE INDEX syntax: {}", line)))?;
    let column_name = columns_part.trim_end_matches(')').trim();
    
    Ok(DdlIndex {
        name: Box::leak(Box::new(index_name.to_string())) as &'static str,
        table_name: Box::leak(Box::new(table_name.to_string())) as &'static str,
        column_name: Box::leak(Box::new(column_name.to_string())) as &'static str,
        index_type: remdb::types::IndexType::BTree,
    })
}

/// 解析列定义
fn parse_column_def(line: &str) -> std::result::Result<DdlColumn, DdlError> {
    // 移除逗号和分号
    let line = line.trim_end_matches([',', ';'].as_slice());
    
    // 查找第一个空格，用于分离列名和数据类型
    let first_space = line.find(|c: char| c.is_whitespace()).ok_or(
        DdlError::Parsing(format!("Invalid column definition: missing data type for '{}'", line))
    )?;
    
    // 提取列名
    let name = &line[..first_space].trim();
    
    // 查找数据类型结束位置（处理带括号的类型如 VARCHAR(50)）
    let remaining = &line[first_space..].trim();
    
    // 检查数据类型是否包含左括号
    let typ: &str;
    let constraints_part: &str;
    
    if let Some(left_paren) = remaining.find('(') {
        // 查找匹配的右括号
        let right_paren = remaining[left_paren..].find(')')
            .ok_or(DdlError::Parsing(format!("Invalid column definition: missing closing parenthesis in data type: {}", line)))? + left_paren + 1;
        
        // 提取数据类型和约束部分
        typ = &remaining[..right_paren].trim();
        constraints_part = &remaining[right_paren..].trim();
    } else {
        // 数据类型没有括号，查找下一个空格来分离数据类型和约束
        if let Some(next_space) = remaining.find(|c: char| c.is_whitespace()) {
            typ = &remaining[..next_space].trim();
            constraints_part = &remaining[next_space..].trim();
        } else {
            // 只有数据类型，没有约束
            typ = remaining;
            constraints_part = "";
        }
    }
    
    // 解析数据类型
    let (data_type, size) = parse_data_type(typ)?;
    
    // 解析约束
    let mut nullable = true;
    let mut primary_key = false;
    let mut auto_increment = false;
    
    // 将约束部分转换为小写，便于比较
    let constraints_lower = constraints_part.to_lowercase();
    
    // 检查NOT NULL约束
    if constraints_lower.contains("not null") {
        nullable = false;
    }
    
    // 检查PRIMARY KEY约束
    if constraints_lower.contains("primary key") {
        primary_key = true;
    }
    
    // 检查AUTO_INCREMENT约束（支持多种写法）
    if constraints_lower.contains("auto_increment") || constraints_lower.contains("autoincrement") {
        auto_increment = true;
    }
    
    Ok(DdlColumn {
        name: Box::leak(Box::new(name.to_string())) as &'static str,
        data_type,
        size,
        nullable,
        primary_key,
        auto_increment,
    })
}

/// 解析数据类型
fn parse_data_type(typ: &str) -> std::result::Result<(DataType, usize), DdlError> {
    let typ_lower = typ.to_lowercase();
    
    // 处理INT类型
    if typ_lower.starts_with("int") {
        // 检查是否有括号（如INT(10)）
        if let Some(left_paren) = typ_lower.find('(') {
            if let Some(right_paren) = typ_lower[left_paren..].find(')') {
                let size_str = &typ_lower[left_paren + 1..left_paren + right_paren];
                let size = size_str.parse::<usize>().map_err(|_| {
                    DdlError::Parsing(format!("Invalid size for INT type: {}", typ))
                })?;
                return Ok((DataType::Int32, size));
            }
        }
        return Ok((DataType::Int32, 4)); // 默认大小为4字节
    }
    
    // 处理BIGINT类型
    if typ_lower.starts_with("bigint") {
        return Ok((DataType::Int64, 8)); // BIGINT固定8字节
    }
    
    // 处理DOUBLE类型
    if typ_lower.starts_with("double") {
        return Ok((DataType::Float64, 8)); // DOUBLE固定8字节
    }
    
    // 处理FLOAT类型
    if typ_lower.starts_with("float") {
        return Ok((DataType::Float32, 4)); // FLOAT固定4字节
    }
    
    // 处理TEXT类型
    if typ_lower.starts_with("text") {
        // 检查是否有括号（如TEXT(255)）
        if let Some(left_paren) = typ_lower.find('(') {
            if let Some(right_paren) = typ_lower[left_paren..].find(')') {
                let size_str = &typ_lower[left_paren + 1..left_paren + right_paren];
                let size = size_str.parse::<usize>().map_err(|_| {
                    DdlError::Parsing(format!("Invalid size for TEXT type: {}", typ))
                })?;
                return Ok((DataType::String, size));
            }
        }
        return Ok((DataType::String, 255)); // 默认大小为255字节
    }
    
    // 处理VARCHAR类型
    if typ_lower.starts_with("varchar") {
        // 查找括号
        let left_paren = typ_lower.find('(').ok_or(
            DdlError::Parsing(format!("Invalid VARCHAR syntax: missing size in '{}'", typ))
        )?;
        let right_paren = typ_lower[left_paren..].find(')').ok_or(
            DdlError::Parsing(format!("Invalid VARCHAR syntax: missing closing parenthesis in '{}'", typ))
        )? + left_paren + 1;
        
        let size_str = &typ_lower[left_paren + 1..right_paren - 1];
        let size = size_str.parse::<usize>().map_err(|_| {
            DdlError::Parsing(format!("Invalid size for VARCHAR type: {}", typ))
        })?;
        
        return Ok((DataType::String, size));
    }
    
    // 处理BOOLEAN类型
    if typ_lower.starts_with("boolean") || typ_lower.starts_with("bool") {
        return Ok((DataType::Bool, 1)); // BOOLEAN固定1字节
    }
    
    // 处理DATE类型
    if typ_lower.starts_with("date") {
        return Ok((DataType::String, 10)); // DATE格式：YYYY-MM-DD，固定10字节
    }
    
    // 处理DATETIME类型
    if typ_lower.starts_with("datetime") {
        return Ok((DataType::String, 19)); // DATETIME格式：YYYY-MM-DD HH:MM:SS，固定19字节
    }
    
    // 不支持的数据类型
    Err(DdlError::Parsing(format!("Unsupported data type: {}", typ)))
}

/// 创建表定义
fn create_table_def(table_id: usize, name: &'static str, columns: &[DdlColumn]) -> std::result::Result<TableDef, DdlError> {
    // 找到主键
    let primary_key = columns.iter()
        .position(|col| col.primary_key)
        .ok_or(DdlError::Parsing(format!("No primary key found for table: {}", name)))?;
    
    // 计算字段偏移量
    let mut offset = 0;
    let mut field_defs = Vec::new();
    
    for col in columns {
        let field_def = FieldDef {
            name: col.name,
            data_type: col.data_type,
            size: col.size,
            offset,
            not_null: !col.nullable,
            primary_key: col.primary_key,
            unique: false,
            auto_increment: col.auto_increment,
        };
        
        field_defs.push(field_def);
        offset += col.size;
    }
    
    // 转换为静态字段定义数组
    let field_defs_static = Box::leak(Box::new(field_defs));
    
    Ok(TableDef {
        id: table_id as u8,
        name,
        fields: field_defs_static,
        primary_key,
        secondary_index: None,
        secondary_index_type: remdb::types::IndexType::BTree,
        record_size: offset,
        max_records: 1000, // 允许1000条记录，支持多个记录
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_table_with_auto_increment() {
        // Test CREATE TABLE statement with AUTO_INCREMENT and VARCHAR(n)
        let create_table_sql = "CREATE TABLE iot_devices (id INT AUTO_INCREMENT PRIMARY KEY,  device_id VARCHAR(50),  timestamp BIGINT,  temperature DOUBLE,  humidity DOUBLE,  pressure DOUBLE,  battery_level INT);";
        
        // Parse the CREATE TABLE statement
        match parse_ddl_content(create_table_sql) {
            Ok(tables) => {
                // Verify that one table was created
                assert_eq!(tables.len(), 1, "Expected 1 table, got {}", tables.len());
                
                let table = &tables[0];
                assert_eq!(table.name, "iot_devices", "Expected table name 'iot_devices', got '{}'", table.name);
                
                // Verify that 7 fields were created
                assert_eq!(table.fields.len(), 7, "Expected 7 fields, got {}", table.fields.len());
                
                // Verify field properties
                let fields = &table.fields;
                
                // Check id field (AUTO_INCREMENT PRIMARY KEY)
                assert_eq!(fields[0].name, "id", "Expected field name 'id', got '{}'", fields[0].name);
                assert_eq!(fields[0].size, 4, "Expected size 4 for INT, got {}", fields[0].size);
                assert_eq!(fields[0].primary_key, true, "Expected id to be primary key, got false");
                assert_eq!(fields[0].auto_increment, true, "Expected id to be AUTO_INCREMENT, got false");
                
                // Check device_id field (VARCHAR(50))
                assert_eq!(fields[1].name, "device_id", "Expected field name 'device_id', got '{}'", fields[1].name);
                assert_eq!(fields[1].size, 50, "Expected size 50 for VARCHAR(50), got {}", fields[1].size);
                
                println!("✓ CREATE TABLE with AUTO_INCREMENT and VARCHAR(n) parsed successfully!");
            },
            Err(err) => {
                panic!("CREATE TABLE parsing failed: {:?}", err);
            }
        }
    }
}