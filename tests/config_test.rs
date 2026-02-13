use crate::common::setup_test_config;

#[test]
fn test_runtime_config_default_values() {
    let config = setup_test_config();

    assert_eq!(config.total_memory, 1024 * 1024 * 10);
    assert_eq!(config.default_max_records, 1000);
    assert!(config.low_power_mode_supported);
    assert!(config.debug_mode);
}

#[test]
fn test_jdbc_config() {
    let config = setup_test_config();

    assert_eq!(config.jdbc.port, Some(16666));
    assert_eq!(config.jdbc.enabled, Some(true));
    assert_eq!(config.jdbc.max_connections, Some(10));
    assert_eq!(config.jdbc.timeout, Some(30));
    assert_eq!(config.jdbc.auth_enabled, Some(false));
}

#[test]
fn test_wal_directory() {
    let config = setup_test_config();

    assert_eq!(config.wal_directory(), "./wal");
}
