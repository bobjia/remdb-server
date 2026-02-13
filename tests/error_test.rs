use remdb_server::ServerError;

#[test]
fn test_server_error_display() {
    let err = ServerError::Config("test error".to_string());
    assert_eq!(format!("{}", err), "配置错误: test error");

    let err = ServerError::Database("db error".to_string());
    assert_eq!(format!("{}", err), "数据库错误: db error");

    let err = ServerError::InvalidArgument("invalid".to_string());
    assert_eq!(format!("{}", err), "无效参数: invalid");
}

#[test]
fn test_server_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let server_err: ServerError = io_err.into();

    assert!(matches!(server_err, ServerError::Io(_)));
}

#[test]
fn test_server_result_ok() {
    let result: remdb_server::ServerResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_server_result_err() {
    let result: remdb_server::ServerResult<i32> = Err(ServerError::Config("test".to_string()));
    assert!(result.is_err());
}
