use remdb::RemDb;
use std::sync::{Arc, Mutex};

use crate::config::RuntimeConfig;
use crate::error::{ServerError, ServerResult};

pub struct AppContext {
    pub db: Arc<Mutex<&'static mut RemDb>>,
    pub config: RuntimeConfig,
}

impl AppContext {
    pub fn new(db: &'static mut RemDb, config: RuntimeConfig) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            config,
        }
    }

    pub fn db_clone(&self) -> Arc<Mutex<&'static mut RemDb>> {
        Arc::clone(&self.db)
    }
}

pub struct AppContextBuilder {
    config: Option<RuntimeConfig>,
    tables: Vec<remdb::TableDef>,
}

impl AppContextBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            tables: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: RuntimeConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_tables(mut self, tables: Vec<remdb::TableDef>) -> Self {
        self.tables = tables;
        self
    }

    pub fn build(self) -> ServerResult<AppContext> {
        let config = self
            .config
            .ok_or_else(|| ServerError::InvalidArgument("Config is required".to_string()))?;

        let db = Self::init_database(&config, self.tables)?;

        Ok(AppContext::new(db, config))
    }

    fn init_database(
        config: &RuntimeConfig,
        tables: Vec<remdb::TableDef>,
    ) -> ServerResult<&'static mut RemDb> {
        use remdb::config::{
            DbConfig, DefaultMemoryAllocator, LogMode, WALCompressionType,
            WALConfig as RemdbWalConfig,
        };
        use remdb::ha::{HAConfig, HARole, ReplicationMode};

        static mut DEFAULT_ALLOCATOR: DefaultMemoryAllocator = DefaultMemoryAllocator;

        let wal_log_path = config.wal_directory();
        let wal_log_path_static: &'static str = Box::leak(wal_log_path.into_boxed_str());

        let ha_role = if config.ha.enabled.unwrap_or(false) {
            match config.ha.role.as_deref() {
                Some("slave") | Some("Slave") => HARole::Slave,
                _ => HARole::Master,
            }
        } else {
            HARole::Master
        };

        let replication_mode = match config.ha.replication_mode.as_deref() {
            Some("sync") | Some("Sync") => ReplicationMode::Sync,
            _ => ReplicationMode::Async,
        };

        let master_address = config
            .ha
            .master_address
            .as_ref()
            .map(|addr| Box::leak(addr.clone().into_boxed_str()) as &'static str);

        let node_id = config
            .ha
            .node_id
            .as_ref()
            .and_then(|id| id.parse::<u32>().ok())
            .unwrap_or(1);

        let db_config = Box::leak(Box::new(DbConfig {
            tables,
            total_memory: config.total_memory,
            low_power_mode_supported: config.low_power_mode_supported,
            low_power_max_records: config.low_power_max_records,
            default_max_records: config.default_max_records,
            memory_allocator: unsafe {
                &*(&raw const DEFAULT_ALLOCATOR as *const _)
                    as &'static dyn remdb::config::MemoryAllocator
            },
            wal_config: RemdbWalConfig {
                log_path: wal_log_path_static,
                log_mode: match config.wal.log_mode.as_deref() {
                    Some("sync") | Some("Sync") => LogMode::Sync,
                    _ => LogMode::Async,
                },
                checkpoint_interval_ms: config.wal.checkpoint_interval_ms.unwrap_or(30000),
                log_file_size_limit: config.wal.log_file_size_limit.unwrap_or(16 * 1024 * 1024),
                log_prealloc_size: config.wal.log_prealloc_size.unwrap_or(4 * 1024 * 1024),
                log_segment_size: config.wal.log_segment_size.unwrap_or(16 * 1024 * 1024),
                retained_checkpoints: config.wal.retained_checkpoints.unwrap_or(3),
                max_consecutive_invalid: config.wal.max_consecutive_invalid.unwrap_or(100),
                skip_threshold: config.wal.skip_threshold.unwrap_or(20),
                skip_block_size: config.wal.skip_block_size.unwrap_or(4096),
                max_skip_attempts: config.wal.max_skip_attempts.unwrap_or(10),
                compression_type: WALCompressionType::None,
                compression_level: 1,
            },
            time_series_defaults: remdb::TimeSeriesConfig::DEFAULT,
            pubsub_config: None,
            ha_config: Some(HAConfig {
                node_id,
                ha_role,
                replication_mode,
                heartbeat_interval_ms: config.ha.heartbeat_interval.unwrap_or(1000),
                failure_detection_ms: config.ha.failure_detection_ms.unwrap_or(5000),
                sync_timeout_ms: config.ha.sync_timeout_ms.unwrap_or(2000),
                master_address,
                master_port: config.ha.master_port,
                replication_port: config.ha.replication_port.unwrap_or(6668),
            }),
        }));

        unsafe {
            remdb::init_global_db(db_config).map_err(|e| ServerError::Database(format!("{:?}", e)))
        }
    }
}

impl Default for AppContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_context_builder_new() {
        let builder = AppContextBuilder::new();
        assert!(builder.config.is_none());
        assert!(builder.tables.is_empty());
    }
}
