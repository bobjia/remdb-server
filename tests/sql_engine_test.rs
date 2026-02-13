use remdb_server::sql_engine::{QueryType, ResultSet, SqlParser};

#[test]
fn test_query_type_detection() {
    assert_eq!(
        SqlParser::detect_query_type("SELECT * FROM users").unwrap(),
        QueryType::Select
    );
    assert_eq!(
        SqlParser::detect_query_type("INSERT INTO users VALUES (1)").unwrap(),
        QueryType::Insert
    );
    assert_eq!(
        SqlParser::detect_query_type("UPDATE users SET name = 'test'").unwrap(),
        QueryType::Update
    );
    assert_eq!(
        SqlParser::detect_query_type("DELETE FROM users WHERE id = 1").unwrap(),
        QueryType::Delete
    );
    assert_eq!(
        SqlParser::detect_query_type("CREATE TABLE test (id INT)").unwrap(),
        QueryType::CreateTable
    );
    assert_eq!(
        SqlParser::detect_query_type("DROP TABLE test").unwrap(),
        QueryType::DropTable
    );
    assert_eq!(
        SqlParser::detect_query_type("SHOW TABLES").unwrap(),
        QueryType::ShowTables
    );
    assert_eq!(
        SqlParser::detect_query_type("DESCRIBE users").unwrap(),
        QueryType::Describe
    );
    assert_eq!(
        SqlParser::detect_query_type("BEGIN").unwrap(),
        QueryType::Begin
    );
    assert_eq!(
        SqlParser::detect_query_type("COMMIT").unwrap(),
        QueryType::Commit
    );
    assert_eq!(
        SqlParser::detect_query_type("ROLLBACK").unwrap(),
        QueryType::Rollback
    );
}

#[test]
fn test_query_type_case_insensitive() {
    assert_eq!(
        SqlParser::detect_query_type("select * from users").unwrap(),
        QueryType::Select
    );
    assert_eq!(
        SqlParser::detect_query_type("Select * From Users").unwrap(),
        QueryType::Select
    );
    assert_eq!(
        SqlParser::detect_query_type("  SELECT  *  FROM  users  ").unwrap(),
        QueryType::Select
    );
}

#[test]
fn test_split_statements() {
    let sql = "SELECT * FROM users;\nINSERT INTO users VALUES (1);\n-- comment\nUPDATE users SET name = 'test';";
    let statements = SqlParser::split_statements(sql);

    assert_eq!(statements.len(), 3);
    assert!(statements[0].contains("SELECT"));
    assert!(statements[1].contains("INSERT"));
    assert!(statements[2].contains("UPDATE"));
}

#[test]
fn test_split_statements_with_comments() {
    let sql = "-- This is a comment\nSELECT * FROM users;\n-- Another comment\nINSERT INTO users VALUES (1);";
    let statements = SqlParser::split_statements(sql);

    assert_eq!(statements.len(), 2);
}

#[test]
fn test_result_set_new() {
    let rs = ResultSet::new();
    assert!(rs.columns.is_empty());
    assert!(rs.rows.is_empty());
    assert_eq!(rs.affected_rows, 0);
}

#[test]
fn test_result_set_with_affected_rows() {
    let rs = ResultSet::with_affected_rows(10);
    assert_eq!(rs.affected_rows, 10);
    assert!(rs.columns.is_empty());
}
