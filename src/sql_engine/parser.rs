use crate::sql_engine::{QueryType, SqlError, SqlResult};

pub struct SqlParser;

impl SqlParser {
    pub fn detect_query_type(sql: &str) -> SqlResult<QueryType> {
        let sql_lower = sql.to_lowercase();
        let sql_trimmed = sql_lower.trim();

        if sql_trimmed.starts_with("select") {
            Ok(QueryType::Select)
        } else if sql_trimmed.starts_with("insert") {
            Ok(QueryType::Insert)
        } else if sql_trimmed.starts_with("update") {
            Ok(QueryType::Update)
        } else if sql_trimmed.starts_with("delete") {
            Ok(QueryType::Delete)
        } else if sql_trimmed.starts_with("create table") {
            Ok(QueryType::CreateTable)
        } else if sql_trimmed.starts_with("drop table") {
            Ok(QueryType::DropTable)
        } else if sql_trimmed.starts_with("alter table") {
            Ok(QueryType::AlterTable)
        } else if sql_trimmed.starts_with("create index") {
            Ok(QueryType::CreateIndex)
        } else if sql_trimmed.starts_with("drop index") {
            Ok(QueryType::DropIndex)
        } else if sql_trimmed.starts_with("begin") {
            Ok(QueryType::Begin)
        } else if sql_trimmed.starts_with("commit") {
            Ok(QueryType::Commit)
        } else if sql_trimmed.starts_with("rollback") {
            Ok(QueryType::Rollback)
        } else if sql_trimmed.starts_with("explain") {
            Ok(QueryType::Explain)
        } else if sql_trimmed.starts_with("show tables") {
            Ok(QueryType::ShowTables)
        } else if sql_trimmed.starts_with("describe") || sql_trimmed.starts_with("desc ") {
            Ok(QueryType::Describe)
        } else if sql_trimmed.starts_with("healthcheck") {
            Ok(QueryType::HealthCheck)
        } else if sql_trimmed.starts_with("export") {
            Ok(QueryType::Export)
        } else if sql_trimmed.starts_with("show databases") || sql_trimmed == "databases" {
            Ok(QueryType::Databases)
        } else {
            Err(SqlError::Unsupported)
        }
    }

    pub fn extract_table_name(sql: &str) -> SqlResult<String> {
        let sql_lower = sql.to_lowercase();
        let sql_trimmed = sql_lower.trim();

        if sql_trimmed.starts_with("insert into") {
            Self::extract_table_after_keyword(sql, "into")
        } else if sql_trimmed.starts_with("update") {
            Self::extract_table_after_keyword(sql, "update")
        } else if sql_trimmed.starts_with("delete from") {
            Self::extract_table_after_keyword(sql, "from")
        } else if sql_trimmed.starts_with("create table") {
            Self::extract_table_after_keyword(sql, "table")
        } else if sql_trimmed.starts_with("drop table") {
            Self::extract_table_after_keyword(sql, "table")
        } else if sql_trimmed.starts_with("alter table") {
            Self::extract_table_after_keyword(sql, "table")
        } else if sql_trimmed.starts_with("describe") || sql_trimmed.starts_with("desc ") {
            let parts: Vec<&str> = sql.split_whitespace().collect();
            if parts.len() >= 2 {
                Ok(parts[1].trim_end_matches(';').to_string())
            } else {
                Err(SqlError::InvalidSyntax("Missing table name".to_string()))
            }
        } else {
            Err(SqlError::Unsupported)
        }
    }

    fn extract_table_after_keyword(sql: &str, keyword: &str) -> SqlResult<String> {
        let sql_lower = sql.to_lowercase();
        let keyword_lower = keyword.to_lowercase();

        if let Some(pos) = sql_lower.find(&keyword_lower) {
            let after_keyword = &sql[pos + keyword.len()..];
            let parts: Vec<&str> = after_keyword.split_whitespace().collect();
            if let Some(table_name) = parts.first() {
                Ok(table_name
                    .trim_end_matches(';')
                    .trim_end_matches('(')
                    .to_string())
            } else {
                Err(SqlError::InvalidSyntax(format!(
                    "Missing table name after {}",
                    keyword
                )))
            }
        } else {
            Err(SqlError::InvalidSyntax(format!(
                "Keyword '{}' not found",
                keyword
            )))
        }
    }

    pub fn split_statements(sql: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current = String::new();

        for line in sql.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            current.push_str(trimmed);
            current.push(' ');

            if trimmed.ends_with(';') {
                let stmt = current.trim_end_matches(';').trim().to_string();
                if !stmt.is_empty() {
                    statements.push(stmt);
                }
                current.clear();
            }
        }

        if !current.trim().is_empty() {
            statements.push(current.trim().to_string());
        }

        statements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_query_type_select() {
        assert_eq!(
            SqlParser::detect_query_type("SELECT * FROM users").unwrap(),
            QueryType::Select
        );
    }

    #[test]
    fn test_detect_query_type_insert() {
        assert_eq!(
            SqlParser::detect_query_type("INSERT INTO users VALUES (1, 'test')").unwrap(),
            QueryType::Insert
        );
    }

    #[test]
    fn test_detect_query_type_update() {
        assert_eq!(
            SqlParser::detect_query_type("UPDATE users SET name = 'test'").unwrap(),
            QueryType::Update
        );
    }

    #[test]
    fn test_detect_query_type_delete() {
        assert_eq!(
            SqlParser::detect_query_type("DELETE FROM users WHERE id = 1").unwrap(),
            QueryType::Delete
        );
    }

    #[test]
    fn test_extract_table_name_insert() {
        let table = SqlParser::extract_table_name("INSERT INTO users VALUES (1, 'test')").unwrap();
        assert_eq!(table, "users");
    }

    #[test]
    fn test_split_statements() {
        let sql = "SELECT * FROM users;\nINSERT INTO users VALUES (1, 'test');\n-- comment\nUPDATE users SET name = 'test';";
        let statements = SqlParser::split_statements(sql);
        assert_eq!(statements.len(), 3);
    }
}
