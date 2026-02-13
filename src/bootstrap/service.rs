use crate::config::RuntimeConfig;
use crate::error::{ServerError, ServerResult};
use remdb::{
    RemDb, TableDef,
    config::{
        DbConfig, DefaultMemoryAllocator, LogMode, WALCompressionType, WALConfig as RemdbWalConfig,
    },
    ha::{HAConfig, HARole, ReplicationMode},
    init_global_db,
    memory::allocator::init_global_allocator,
};
use std::sync::{Arc, Mutex};

static mut DEFAULT_ALLOCATOR: DefaultMemoryAllocator = DefaultMemoryAllocator;

pub struct DatabaseContext {
    pub db: Arc<Mutex<&'static mut RemDb>>,
    pub tables: Vec<TableDef>,
}

pub struct ServiceStarter;

impl ServiceStarter {
    pub fn init_memory_allocator(total_memory: usize) -> ServerResult<()> {
        let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
        let memory_ptr = memory_vec.as_ptr() as *mut u8;
        std::mem::forget(memory_vec);

        unsafe {
            init_global_allocator(memory_ptr, total_memory)
                .map_err(|e| ServerError::MemoryAllocation(format!("{:?}", e)))?;
        }

        Ok(())
    }

    pub fn create_db_config(
        runtime_config: &RuntimeConfig,
        tables: Vec<TableDef>,
    ) -> &'static DbConfig {
        let wal_log_path = runtime_config.wal_directory();
        let wal_log_path_static: &'static str = Box::leak(wal_log_path.into_boxed_str());

        let ha_role = if runtime_config.ha.enabled.unwrap_or(false) {
            match runtime_config.ha.role.as_deref() {
                Some("slave") | Some("Slave") => HARole::Slave,
                _ => HARole::Master,
            }
        } else {
            HARole::Master
        };

        let replication_mode = match runtime_config.ha.replication_mode.as_deref() {
            Some("sync") | Some("Sync") => ReplicationMode::Sync,
            _ => ReplicationMode::Async,
        };

        let master_address = runtime_config
            .ha
            .master_address
            .as_ref()
            .map(|addr| Box::leak(addr.clone().into_boxed_str()) as &'static str);

        let node_id = runtime_config
            .ha
            .node_id
            .as_ref()
            .and_then(|id| id.parse::<u32>().ok())
            .unwrap_or(1);

        Box::leak(Box::new(DbConfig {
            tables,
            total_memory: runtime_config.total_memory,
            low_power_mode_supported: runtime_config.low_power_mode_supported,
            low_power_max_records: runtime_config.low_power_max_records,
            default_max_records: runtime_config.default_max_records,
            memory_allocator: unsafe {
                &*(&raw const DEFAULT_ALLOCATOR as *const _)
                    as &'static dyn remdb::config::MemoryAllocator
            },
            wal_config: RemdbWalConfig {
                log_path: wal_log_path_static,
                log_mode: match runtime_config.wal.log_mode.as_deref() {
                    Some("sync") | Some("Sync") => LogMode::Sync,
                    _ => LogMode::Async,
                },
                checkpoint_interval_ms: runtime_config.wal.checkpoint_interval_ms.unwrap_or(30000),
                log_file_size_limit: runtime_config
                    .wal
                    .log_file_size_limit
                    .unwrap_or(16 * 1024 * 1024),
                log_prealloc_size: runtime_config
                    .wal
                    .log_prealloc_size
                    .unwrap_or(4 * 1024 * 1024),
                log_segment_size: runtime_config
                    .wal
                    .log_segment_size
                    .unwrap_or(16 * 1024 * 1024),
                retained_checkpoints: runtime_config.wal.retained_checkpoints.unwrap_or(3),
                max_consecutive_invalid: runtime_config.wal.max_consecutive_invalid.unwrap_or(100),
                skip_threshold: runtime_config.wal.skip_threshold.unwrap_or(20),
                skip_block_size: runtime_config.wal.skip_block_size.unwrap_or(4096),
                max_skip_attempts: runtime_config.wal.max_skip_attempts.unwrap_or(10),
                compression_type: WALCompressionType::None,
                compression_level: 1,
            },
            time_series_defaults: remdb::TimeSeriesConfig::DEFAULT,
            pubsub_config: None,
            ha_config: Some(HAConfig {
                node_id,
                ha_role,
                replication_mode,
                heartbeat_interval_ms: runtime_config.ha.heartbeat_interval.unwrap_or(1000),
                failure_detection_ms: runtime_config.ha.failure_detection_ms.unwrap_or(5000),
                sync_timeout_ms: runtime_config.ha.sync_timeout_ms.unwrap_or(2000),
                master_address,
                master_port: runtime_config.ha.master_port,
                replication_port: runtime_config.ha.replication_port.unwrap_or(6668),
            }),
        }))
    }

    pub fn init_database(db_config: &'static DbConfig) -> ServerResult<&'static mut RemDb> {
        unsafe { init_global_db(db_config).map_err(|e| ServerError::Database(format!("{:?}", e))) }
    }

    pub fn init_ha_manager(db_config: &'static DbConfig) -> ServerResult<()> {
        if db_config.ha_config.is_some() {
            remdb::ha::init(db_config).map_err(|e| ServerError::HaManager(format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn create_db_context(db: &'static mut RemDb) -> DatabaseContext {
        let db_arc = Arc::new(Mutex::new(db));

        DatabaseContext {
            db: db_arc,
            tables: Vec::new(),
        }
    }
}
