use crate::ddl_compiler::DdlError;
use remdb::RemDbError;
use thiserror::Error;

pub mod ddl;
pub mod delete;
pub mod insert;
pub mod parser;
pub mod select;
pub mod update;

pub use ddl::DdlExecutorHandler;
pub use delete::DeleteExecutor;
pub use insert::InsertExecutor;
pub use parser::SqlParser;
pub use select::SelectExecutor;
pub use update::UpdateExecutor;

#[derive(Error, Debug)]
pub enum SqlError {
    #[error("Database error: {0}")]
    Database(RemDbError),
    #[error("SQL parsing error: {0}")]
    Parsing(String),
    #[error("Unsupported SQL command")]
    Unsupported,
    #[error("Invalid SQL syntax: {0}")]
    InvalidSyntax(String),
    #[error("Table not found: {0}")]
    TableNotFound(String),
    #[error("Column not found: {0}")]
    ColumnNotFound(String),
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
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

pub type SqlResult<T> = Result<T, SqlError>;

#[derive(Debug, Default)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: usize,
}

impl ResultSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_columns(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            affected_rows: 0,
        }
    }

    pub fn with_affected_rows(count: usize) -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            affected_rows: count,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    CreateTable,
    DropTable,
    AlterTable,
    CreateIndex,
    DropIndex,
    Begin,
    Commit,
    Rollback,
    Explain,
    ShowTables,
    Describe,
    HealthCheck,
    Export,
    Databases,
    CreateDatabase,
    DropDatabase,
    UseDatabase,
    CloseDatabase,
}

pub fn execute_extended_sql(db: &mut remdb::RemDb, sql: &str) -> SqlResult<ResultSet> {
    let statements = SqlParser::split_statements(sql);
    let mut total_result = ResultSet::new();

    for stmt in statements {
        let result = execute_single(db, &stmt)?;
        total_result.affected_rows += result.affected_rows;
        if total_result.columns.is_empty() && !result.columns.is_empty() {
            total_result.columns = result.columns;
            total_result.rows = result.rows;
        }
    }

    Ok(total_result)
}

/// Preprocess SQL to handle known incompatibilities with the remdb crate's SQL parser.
/// Currently a pass-through; specific transformations can be added as needed.
fn preprocess_sql(sql: &str, _query_type: QueryType) -> String {
    sql.to_string()
}

fn execute_single(db: &mut remdb::RemDb, sql: &str) -> SqlResult<ResultSet> {
    let query_type = SqlParser::detect_query_type(sql)?;

    // Preprocess SQL to handle known incompatibilities with the remdb crate's SQL parser
    let sql = preprocess_sql(sql, query_type);

    match query_type {
        QueryType::Select
        | QueryType::ShowTables
        | QueryType::Describe
        | QueryType::HealthCheck => SelectExecutor::execute(db, &sql),
        QueryType::Databases => execute_databases_query(db),
        QueryType::Insert => InsertExecutor::execute(db, &sql),
        QueryType::Update => UpdateExecutor::execute(db, &sql),
        QueryType::Delete => DeleteExecutor::execute(db, &sql),
        QueryType::CreateTable => DdlExecutorHandler::execute_create_table(db, &sql),
        QueryType::DropTable => DdlExecutorHandler::execute_drop_table(db, &sql),
        QueryType::AlterTable => DdlExecutorHandler::execute_alter_table(db, &sql),
        QueryType::CreateIndex => DdlExecutorHandler::execute_create_index(db, &sql),
        QueryType::DropIndex => DdlExecutorHandler::execute_drop_index(db, &sql),
        QueryType::Begin | QueryType::Commit | QueryType::Rollback => {
            db.sql_query(&sql)?;
            Ok(ResultSet::new())
        }
        QueryType::CreateDatabase
        | QueryType::DropDatabase
        | QueryType::UseDatabase
        | QueryType::CloseDatabase => {
            db.sql_query(&sql)?;
            Ok(ResultSet::new())
        }
        QueryType::Explain | QueryType::Export => Ok(ResultSet::new()),
    }
}

/// 执行SHOW DATABASES查询
fn execute_databases_query(db: &mut remdb::RemDb) -> SqlResult<ResultSet> {
    let databases = db.databases()?;
    let mut result = ResultSet::with_columns(vec![
        "name".into(),
        "database_type".into(),
        "status".into(),
        "table_count".into(),
        "memory_usage".into(),
    ]);
    for info in databases {
        result.rows.push(vec![
            info.name,
            info.database_type,
            format!("{:?}", info.status),
            info.table_count.to_string(),
            info.memory_usage.to_string(),
        ]);
    }
    Ok(result)
}

pub fn format_result_set(result: &ResultSet) -> String {
    if result.columns.is_empty() && result.rows.is_empty() {
        return format!("Affected rows: {}", result.affected_rows);
    }

    let mut output = String::new();

    if !result.columns.is_empty() {
        output.push('|');
        for col in &result.columns {
            output.push_str(&format!(" {} |", col));
        }
        output.push('\n');

        output.push('|');
        for _ in &result.columns {
            output.push_str("------|");
        }
        output.push('\n');
    }

    for row in &result.rows {
        output.push('|');
        for value in row {
            output.push_str(&format!(" {} |", value));
        }
        output.push('\n');
    }

    if result.affected_rows > 0 {
        output.push_str(&format!("\nAffected rows: {}", result.affected_rows));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_set_new() {
        let rs = ResultSet::new();
        assert!(rs.columns.is_empty());
        assert!(rs.rows.is_empty());
        assert_eq!(rs.affected_rows, 0);
    }

    #[test]
    fn test_result_set_with_columns() {
        let rs = ResultSet::with_columns(vec!["id".to_string(), "name".to_string()]);
        assert_eq!(rs.columns.len(), 2);
        assert!(rs.rows.is_empty());
    }

    #[test]
    fn test_result_set_with_affected_rows() {
        let rs = ResultSet::with_affected_rows(10);
        assert_eq!(rs.affected_rows, 10);
        assert!(rs.columns.is_empty());
    }

    #[test]
    fn test_format_result_set_empty() {
        let rs = ResultSet::with_affected_rows(5);
        let output = format_result_set(&rs);
        assert!(output.contains("Affected rows: 5"));
    }

    #[test]
    fn test_format_result_set_with_data() {
        let mut rs = ResultSet::with_columns(vec!["id".to_string(), "name".to_string()]);
        rs.rows.push(vec!["1".to_string(), "test".to_string()]);
        let output = format_result_set(&rs);
        assert!(output.contains("id"));
        assert!(output.contains("name"));
        assert!(output.contains("test"));
    }
}
