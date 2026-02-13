use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("SQL执行错误: {0}")]
    SqlExecution(String),

    #[error("DDL编译错误: {0}")]
    DdlCompilation(String),

    #[error("快照加载错误: {0}")]
    SnapshotLoad(String),

    #[error("快照保存错误: {0}")]
    SnapshotSave(String),

    #[error("WAL恢复错误: {0}")]
    WalRecovery(String),

    #[error("JDBC服务器错误: {0}")]
    JdbcServer(String),

    #[error("认证失败: {0}")]
    Authentication(String),

    #[error("连接池错误: {0}")]
    ConnectionPool(String),

    #[error("平台初始化错误: {0}")]
    PlatformInit(String),

    #[error("服务启动错误: {0}")]
    ServiceStart(String),

    #[error("解析错误: {0}")]
    Parse(String),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML解析错误: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("内存分配错误: {0}")]
    MemoryAllocation(String),

    #[error("HA管理器错误: {0}")]
    HaManager(String),

    #[error("PubSub系统错误: {0}")]
    PubSub(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("无效参数: {0}")]
    InvalidArgument(String),
}

pub type ServerResult<T> = Result<T, ServerError>;

impl From<remdb::RemDbError> for ServerError {
    fn from(err: remdb::RemDbError) -> Self {
        ServerError::Database(format!("{:?}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_error_config() {
        let err = ServerError::Config("test config error".to_string());
        assert_eq!(err.to_string(), "配置错误: test config error");
    }

    #[test]
    fn test_server_error_database() {
        let err = ServerError::Database("test database error".to_string());
        assert_eq!(err.to_string(), "数据库错误: test database error");
    }

    #[test]
    fn test_server_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let server_err: ServerError = io_err.into();
        assert!(matches!(server_err, ServerError::Io(_)));
    }

    #[test]
    fn test_server_result_ok() {
        let result: ServerResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_server_result_err() {
        let result: ServerResult<i32> = Err(ServerError::Config("test".to_string()));
        assert!(result.is_err());
    }
}
