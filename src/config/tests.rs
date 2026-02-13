#[cfg(test)]
mod tests {
    use crate::config::{Config, HaConfig, JdbcConfig, PubSubConfig, RuntimeConfig, WALConfig};

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.snapshot_dir.is_none());
        assert!(config.total_memory.is_none());
        assert!(config.debug.is_none());
    }

    #[test]
    fn test_config_jdbc_config() {
        let mut config = Config::default();
        config.jdbc_port = Some(6666);
        config.jdbc_enabled = Some(true);
        config.max_connections = Some(10);

        let jdbc = config.jdbc_config();
        assert_eq!(jdbc.port, Some(6666));
        assert_eq!(jdbc.enabled, Some(true));
        assert_eq!(jdbc.max_connections, Some(10));
    }

    #[test]
    fn test_config_ha_enabled() {
        let mut config = Config::default();
        assert!(!config.ha_enabled());

        config.ha = Some(HaConfig {
            enabled: Some(true),
            ..Default::default()
        });
        assert!(config.ha_enabled());
    }

    #[test]
    fn test_config_pubsub_enabled() {
        let mut config = Config::default();
        assert!(!config.pubsub_enabled());

        config.pubsub = Some(PubSubConfig {
            enabled: Some(true),
            ..Default::default()
        });
        assert!(config.pubsub_enabled());
    }

    #[test]
    fn test_runtime_config_wal_directory() {
        let config = RuntimeConfig {
            snapshot_dir: None,
            full_image: None,
            total_memory: 1024 * 1024 * 100,
            default_max_records: 10000,
            low_power_mode_supported: true,
            low_power_max_records: None,
            log_path: None,
            log_file_name: "./logs/test.log".to_string(),
            snapshot_interval: None,
            snapshot_type: None,
            max_incremental_snapshots: None,
            debug_mode: false,
            jdbc: JdbcConfig::default(),
            pubsub: PubSubConfig::default(),
            ha: HaConfig::default(),
            wal: WALConfig::default(),
            ddl_path: None,
        };

        assert_eq!(config.wal_directory(), "./wal");
    }

    #[test]
    fn test_runtime_config_wal_directory_with_log_path() {
        let config = RuntimeConfig {
            snapshot_dir: None,
            full_image: None,
            total_memory: 1024 * 1024 * 100,
            default_max_records: 10000,
            low_power_mode_supported: true,
            low_power_max_records: None,
            log_path: Some("./data/wal/redo.log".to_string()),
            log_file_name: "./logs/test.log".to_string(),
            snapshot_interval: None,
            snapshot_type: None,
            max_incremental_snapshots: None,
            debug_mode: false,
            jdbc: JdbcConfig::default(),
            pubsub: PubSubConfig::default(),
            ha: HaConfig::default(),
            wal: WALConfig::default(),
            ddl_path: None,
        };

        assert_eq!(config.wal_directory(), "./data/wal");
    }
}
