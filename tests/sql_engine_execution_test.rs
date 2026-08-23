//! Server-level integration tests for the SQL engine routing layer.
//!
//! Every remdbcli command reaches the database through
//! `execute_extended_sql` (CLI → JDBC handler → `execute_extended_sql` →
//! per-command executor). The core `remdb` crate unit-tests the SQL
//! *semantics*; this file tests the server-side *routing* that those tests
//! don't cover, against a live `RemDb` built the same way the server builds
//! one (`init_global_db`). It guards against regressions like database
//! commands falling through to `SqlError::Unsupported`.

use std::sync::Mutex;

use remdb::config::{DbConfig, DefaultMemoryAllocator, LogMode, WALConfig};
use remdb::RemDb;
use remdb_server::sql_engine::{SqlError, execute_extended_sql};

/// 全局互斥锁，确保测试串行执行（RemDb 使用全局内存分配器和全局数据库实例）
static TEST_MUTEX: Mutex<()> = Mutex::new(());

/// 测试用内存池，在测试期间保持有效
static TEST_DB_MEMORY: Mutex<Option<Box<[u8]>>> = Mutex::new(None);

const TEST_MEMORY_SIZE: usize = 16 * 1024 * 1024;

/// 与 remdb-server 启动时一致的数据库配置（无预置表）
static DB_CONFIG: DbConfig = DbConfig {
    tables: Vec::new(),
    total_memory: TEST_MEMORY_SIZE,
    default_max_records: 1000,
    low_power_mode_supported: false,
    low_power_max_records: None,
    memory_allocator: &DefaultMemoryAllocator,
    wal_config: WALConfig {
        log_path: "./test_logs/sql_engine_execution.wal",
        log_mode: LogMode::Sync,
        checkpoint_interval_ms: 60000,
        log_file_size_limit: 16 * 1024 * 1024,
        log_prealloc_size: 0,
        log_segment_size: 16 * 1024 * 1024,
        max_consecutive_invalid: 100,
        retained_checkpoints: 1,
        skip_threshold: 1000,
        skip_block_size: 1024 * 1024,
        max_skip_attempts: 3,
        compression_type: remdb::config::WALCompressionType::None,
        compression_level: 3,
    },
    time_series_defaults: remdb::time_series::TimeSeriesConfig {
        partition_duration_secs: 3600,
        retention_period_secs: 86400,
        max_partitions: 100,
        compression: remdb::time_series::CompressionType::None,
    },
    pubsub_config: None,
    ha_config: None,
    model_worker_config: remdb::config::ModelWorkerConfig::DEFAULT,
};

/// 初始化平台、全局内存分配器并创建全局数据库实例。
/// 复刻 remdb-server 的启动路径（src/main.rs / src/bootstrap/service.rs）。
fn setup() -> Result<&'static mut RemDb, SqlError> {
    // 取出旧内存池并保持存活，确保旧数据库 drop 时旧分配器仍然有效
    let old_pool = TEST_DB_MEMORY
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();

    remdb::platform::init_platform(remdb::platform::posix::get_posix_platform());
    remdb::reset_global_db();
    drop(old_pool);

    // 分配新的内存池并初始化全局分配器
    let mut pool = vec![0u8; TEST_MEMORY_SIZE].into_boxed_slice();
    let ptr = pool.as_mut_ptr();
    remdb::memory::allocator::init_global_allocator(ptr, pool.len()).unwrap();
    *TEST_DB_MEMORY
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = Some(pool);

    // 创建全局数据库实例（内部会注册 "default" 数据库）
    remdb::init_global_db(&DB_CONFIG).map_err(SqlError::from)
}

fn database_names(result: &remdb_server::sql_engine::ResultSet) -> Vec<&str> {
    result.rows.iter().map(|r| r[0].as_str()).collect()
}

#[test]
fn test_show_databases_lists_default() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    let result = execute_extended_sql(db, "show databases").unwrap();

    assert_eq!(
        result.columns,
        vec![
            "name".to_string(),
            "database_type".to_string(),
            "status".to_string(),
            "table_count".to_string(),
            "memory_usage".to_string(),
        ]
    );
    let names = database_names(&result);
    assert!(
        names.contains(&"default"),
        "expected 'default' in database list, got {:?}",
        names
    );
}

#[test]
fn test_create_database_succeeds_and_lists() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    // 创建数据库
    execute_extended_sql(db, "create database demo").unwrap();

    // 列表中应包含 demo
    let result = execute_extended_sql(db, "show databases").unwrap();
    let names = database_names(&result);
    assert!(
        names.contains(&"demo"),
        "expected 'demo' in database list, got {:?}",
        names
    );
}

#[test]
fn test_duplicate_create_database_errors() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    execute_extended_sql(db, "create database demo").unwrap();
    // 重复创建应返回 DatabaseExists，而不是 Unsupported 或 InternalError
    let err = execute_extended_sql(db, "create database demo").unwrap_err();
    assert!(
        matches!(err, SqlError::Database(remdb::RemDbError::DatabaseExists)),
        "expected DatabaseExists, got {:?}",
        err
    );
}

#[test]
fn test_drop_database() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    execute_extended_sql(db, "create database demo").unwrap();
    execute_extended_sql(db, "drop database demo").unwrap();

    let result = execute_extended_sql(db, "show databases").unwrap();
    let names = database_names(&result);
    assert!(
        !names.contains(&"demo"),
        "expected 'demo' dropped, got {:?}",
        names
    );
}

#[test]
fn test_close_database() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    execute_extended_sql(db, "create database demo").unwrap();
    execute_extended_sql(db, "close database demo").unwrap();
}

#[test]
fn test_use_database_succeeds() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    execute_extended_sql(db, "create database demo").unwrap();

    // 新建的数据库（状态为 Created）应可直接切换使用
    execute_extended_sql(db, "use database demo").unwrap();
}

#[test]
fn test_crud_commands_dispatch() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    // CREATE TABLE
    execute_extended_sql(
        db,
        "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR, age INT)",
    )
    .unwrap();

    // INSERT
    execute_extended_sql(db, "INSERT INTO users (id, name, age) VALUES (1, 'alice', 30)").unwrap();

    // SELECT
    let result = execute_extended_sql(db, "SELECT * FROM users").unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0], vec!["1", "alice", "30"]);

    // UPDATE
    execute_extended_sql(db, "UPDATE users SET age = 31 WHERE id = 1").unwrap();
    let result = execute_extended_sql(db, "SELECT age FROM users WHERE id = 1").unwrap();
    assert_eq!(result.rows[0], vec!["31"]);

    // DELETE
    execute_extended_sql(db, "DELETE FROM users WHERE id = 1").unwrap();
    let result = execute_extended_sql(db, "SELECT * FROM users").unwrap();
    assert!(result.rows.is_empty(), "expected no rows after delete");

    // SHOW TABLES
    let result = execute_extended_sql(db, "show tables").unwrap();
    let names: Vec<&str> = result.rows.iter().map(|r| r[0].as_str()).collect();
    assert!(
        names.contains(&"users"),
        "expected 'users' in table list, got {:?}",
        names
    );
}

#[test]
fn test_unknown_command_still_unsupported() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let db = setup().unwrap();

    // 保证新加的数据库命令分支不会误吞未知命令
    let result = execute_extended_sql(db, "frobnicate the widget");
    assert!(matches!(result, Err(SqlError::Unsupported)));
}
