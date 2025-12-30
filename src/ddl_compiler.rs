// 添加extern crate alloc语句
extern crate alloc;

use std::fs::File;
use std::io::Read;
use alloc::sync::Arc;
use remdb::{types::{TableDef, FieldDef, DataType, RecordHeader}, RemDb, Result as RemResult, DdlExecutor};
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
}

/// DDL索引定义
struct DdlIndex {
    name: &'static str,
    table_name: &'static str,
    column_name: &'static str,
    index_type: remdb::types::IndexType,
}

/// 编译DDL文件，生成表定义
pub fn compile_ddl_file(path: &str) -> std::result::Result<Vec<TableDef>, DdlError> {
    // 读取DDL文件内容
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // 解析DDL内容
    let tables = parse_ddl_content(&content)?;
    
    Ok(tables)
}

/// 解析DDL内容，生成表定义
fn parse_ddl_content(content: &str) -> std::result::Result<Vec<TableDef>, DdlError> {
    let mut tables = Vec::new();
    let mut indices = Vec::new();
    let mut current_table = None;
    let mut current_columns = Vec::new();
    let mut in_table = false;
    let mut table_id = 0;
    
    // 按行处理DDL内容
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") {
            continue;
        }
        
        // 开始解析CREATE TABLE语句
        if line.starts_with("CREATE TABLE") {
            // 提取表名
            let table_name = line
                .trim_start_matches("CREATE TABLE")
                .trim()
                .split_whitespace()
                .next()
                .ok_or(DdlError::Parsing("Invalid CREATE TABLE syntax".to_string()))?;
            
            // 存储表名
            current_table = Some(Box::leak(Box::new(table_name.to_string())) as &'static str);
            in_table = true;
            continue;
        }
        
        // 解析CREATE INDEX语句
        if line.starts_with("CREATE INDEX") {
            let index = parse_index_def(line)?;
            indices.push(index);
            continue;
        }
        
        // 结束解析表定义
        if in_table && line.ends_with(';') {
            if let Some(table_name) = current_table {
                // 生成表定义
                let table = create_table_def(table_id, table_name, &current_columns)?;
                tables.push(table);
                table_id += 1;
                
                // 重置状态
                current_table = None;
                current_columns.clear();
                in_table = false;
            }
            continue;
        }
        
        // 解析列定义
        if in_table && !line.starts_with('(') && !line.starts_with(')') {
            let column = parse_column_def(line)?;
            current_columns.push(column);
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
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return Err(DdlError::Parsing(format!("Invalid column definition: {}", line)));
    }
    
    let name = parts[0];
    let typ = parts[1];
    
    // 解析数据类型
    let (data_type, size) = parse_data_type(typ)?;
    
    // 检查是否有NOT NULL约束
    let nullable = !parts.iter().any(|&p| p.eq_ignore_ascii_case("NOT")) || 
                   parts.iter().any(|&p| p.eq_ignore_ascii_case("NULL"));
    
    // 检查是否为主键
    let primary_key = parts.iter().any(|&p| p.eq_ignore_ascii_case("PRIMARY")) && 
                      parts.iter().any(|&p| p.eq_ignore_ascii_case("KEY"));
    
    Ok(DdlColumn {
        name: Box::leak(Box::new(name.to_string())) as &'static str,
        data_type,
        size,
        nullable,
        primary_key,
    })
}

/// 解析数据类型
fn parse_data_type(typ: &str) -> std::result::Result<(DataType, usize), DdlError> {
    match typ.to_uppercase().as_str() {
        "INTEGER" | "INT" => Ok((DataType::Int32, 4)),
        "BIGINT" => Ok((DataType::Int64, 8)),
        "SMALLINT" => Ok((DataType::Int16, 2)),
        "TINYINT" => Ok((DataType::Int8, 1)),
        "BOOLEAN" | "BOOL" => Ok((DataType::Bool, 1)),
        "REAL" | "FLOAT" => Ok((DataType::Float32, 4)),
        "DOUBLE" => Ok((DataType::Float64, 8)),
        "TEXT" => Ok((DataType::String, 64)), // 固定大小64字节
        _ => Err(DdlError::Parsing(format!("Unsupported data type: {}", typ))),
    }
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
            auto_increment: false,
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

/// 初始化数据库实例
pub fn init_database(tables: Vec<TableDef>, total_memory: Option<usize>, default_max_records: Option<usize>, low_power_mode_supported: Option<bool>, low_power_max_records: Option<usize>) -> RemResult<RemDb> {
    // 首先将tables向量泄漏到静态内存，确保TableDef有'static生命周期
    let static_tables = Box::leak(Box::new(tables));
    
    // 创建默认内存分配器
    static mut DEFAULT_ALLOCATOR: remdb::config::DefaultMemoryAllocator = remdb::config::DefaultMemoryAllocator;
    
    // 使用非常小的默认最大记录数，避免内存不足
    let small_max_records = 1; // 仅使用1条记录，最小化内存使用
    
    // 创建配置
    let config = Box::leak(Box::new(remdb::config::DbConfig {
        tables: static_tables,
        total_memory: total_memory.unwrap_or(1024 * 1024 * 100), // 默认100MB
        default_max_records: small_max_records, // 使用非常小的默认值，避免内存不足
        low_power_mode_supported: low_power_mode_supported.unwrap_or(true), // 默认支持低功耗模式
        low_power_max_records: Some(small_max_records), // 使用非常小的默认值
        memory_allocator: unsafe { &*(&raw const DEFAULT_ALLOCATOR as *const _) as &'static dyn remdb::config::MemoryAllocator },
    }));
    
    // 创建数据库实例
    let mut db = RemDb::new(config);
    
    // 初始化数据库
    db.init()?;
    
    // 注意：我们不需要手动创建表，因为RemDb的sql_query方法会在执行查询时自动创建表
    // 这是因为sql_query方法会解析SQL查询，提取表名，然后查找表，如果表不存在，会自动创建表
    
    Ok(db)
}