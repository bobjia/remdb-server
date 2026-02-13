pub mod fixtures;

pub fn setup_test_config() -> remdb_server::config::RuntimeConfig {
    use remdb_server::config::{HaConfig, JdbcConfig, PubSubConfig, RuntimeConfig, WALConfig};

    RuntimeConfig {
        snapshot_dir: Some("./test_snapshots".to_string()),
        full_image: None,
        total_memory: 1024 * 1024 * 10,
        default_max_records: 1000,
        low_power_mode_supported: true,
        low_power_max_records: None,
        log_path: None,
        log_file_name: "./test_logs/test.log".to_string(),
        snapshot_interval: None,
        snapshot_type: None,
        max_incremental_snapshots: None,
        debug_mode: true,
        jdbc: JdbcConfig {
            port: Some(16666),
            enabled: Some(true),
            max_connections: Some(10),
            timeout: Some(30),
            auth_enabled: Some(false),
            username: None,
            password_hash: None,
        },
        pubsub: PubSubConfig::default(),
        ha: HaConfig::default(),
        wal: WALConfig::default(),
        ddl_path: None,
    }
}
