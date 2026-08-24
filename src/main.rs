use remdb::{
    RemDb,
    ha::{HARole, ReplicationMode},
};

use clap::Parser;
use remdb::log::{error, info, warn};
use remdb_server::bootstrap::init_platform;
use remdb_server::config::loader::Args;
use remdb_server::config::{self, Config, HaConfig, PubSubConfig, WALConfig};
use remdb_server::jdbc_server::JdbcServer;
use remdb_server::{is_debug_mode, set_debug_mode};
use std::fs;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

mod benchmark;
mod cli;
mod ddl_compiler;
mod handler;
mod macros;
mod pool;
mod proto;
mod snapshot_loader;
mod sql_engine;
mod tuning;

use remdb_server::config::loader::Command;

#[tokio::main]
async fn main() {
    // 在程序最开始就设置默认的 RUST_LOG 环境变量
    unsafe {
        std::env::set_var("RUST_LOG", "error");
    }

    let args = Args::parse();

    let message = "remdb-server v0.3.2";
    info!("{}", message);

    // 处理子命令
    if let Some(Command::Benchmark {
        query_count,
        connections,
        query_template,
        server_url,
        test_type,
        write_template,
        read_write_ratio,
    }) = args.command
    {
        // 导入基准测试模块
        use benchmark::{BenchmarkConfig, run_benchmark};

        // 解析读写比例
        let read_write_ratio = match read_write_ratio
            .split(":")
            .map(|s| s.parse::<usize>())
            .collect::<Vec<_>>()
            .as_slice()
        {
            [Ok(read), Ok(write)] => (*read, *write),
            _ => {
                error!("Invalid read_write_ratio format. Expected format: \"8:2\".");
                std::process::exit(1);
            }
        };

        let config = BenchmarkConfig {
            server_url,
            connection_count: connections,
            query_count,
            query_template,
            run_duration_secs: None,
            test_type,
            write_template,
            read_write_ratio,
        };

        match run_benchmark(config).await {
            Ok(_) => info!("\nBenchmark completed successfully!"),
            Err(e) => {
                error!("\nBenchmark failed: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // 确定要使用的配置文件路径
    let config_path = if let Some(path) = &args.config {
        path.clone()
    } else {
        // 默认使用 remdb-master.toml
        "./remdb-master.toml".to_string()
    };

    // 提前解析配置文件，获取完整的 debug 模式设置
    let mut config = Config::default();
    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(parsed_config) => {
                config = parsed_config;
            }
            Err(_) => {}
        },
        Err(_) => {}
    }

    // 计算最终的 debug 模式
    let debug_mode = args.debug || config.debug.unwrap_or(false);

    // 如果是 debug 模式，更新 RUST_LOG 环境变量
    if debug_mode {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    // 重新初始化配置，因为上面的代码只是为了获取 debug 模式
    let mut config = Config::default();
    let message = format!("Reading config file: {}", config_path);
    info!("{}", message);
    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(parsed_config) => {
                config = parsed_config;
                let message = "Config file loaded successfully";
                info!("{}", message);
            }
            Err(err) => {
                let message = format!("Warning: Failed to parse config file: {:?}", err);
                warn!("{}", message);
                let message = "Using default config values";
                warn!("{}", message);
            }
        },
        Err(err) => {
            let message = format!("Warning: Failed to read config file: {:?}", err);
            warn!("{}", message);
            let message = "Using default config values";
            warn!("{}", message);
        }
    }

    // 合并配置：命令行参数优先级高于配置文件

    let snapshot_dir = args.snapshot_dir.or(config.snapshot_dir);
    let full_image = args.full_image.clone();
    let total_memory = args.total_memory.or(config.total_memory);
    let default_max_records = args.default_max_records.or(config.default_max_records);
    // 合并WAL配置
    let wal_log_path = config.wal.as_ref().and_then(|w| w.log_path.clone());
    let wal_log_mode = config.wal.as_ref().and_then(|w| w.log_mode.clone());
    let wal_checkpoint_interval_ms = config.wal.as_ref().and_then(|w| w.checkpoint_interval_ms);
    let wal_log_file_size_limit = config.wal.as_ref().and_then(|w| w.log_file_size_limit);
    let wal_log_prealloc_size = config.wal.as_ref().and_then(|w| w.log_prealloc_size);
    let wal_log_segment_size = config.wal.as_ref().and_then(|w| w.log_segment_size);
    let wal_retained_checkpoints = config.wal.as_ref().and_then(|w| w.retained_checkpoints);
    let wal_max_consecutive_invalid = config.wal.as_ref().and_then(|w| w.max_consecutive_invalid);
    let wal_skip_threshold = config.wal.as_ref().and_then(|w| w.skip_threshold);
    let wal_skip_block_size = config.wal.as_ref().and_then(|w| w.skip_block_size);
    let wal_max_skip_attempts = config.wal.as_ref().and_then(|w| w.max_skip_attempts);

    let low_power_mode_supported = args
        .low_power_mode_supported
        .or(config.low_power_mode_supported);
    let low_power_max_records = args.low_power_max_records.or(config.low_power_max_records);
    let log_path = args.log_path.or(wal_log_path).or(config.log_path);
    let snapshot_interval = args.snapshot_interval.or(config.snapshot_interval);
    let snapshot_type = args.snapshot_type.or(config.snapshot_type);
    let max_incremental_snapshots = args
        .max_incremental_snapshots
        .or(config.max_incremental_snapshots);
    let jdbc_port = args.jdbc_port.or(config.jdbc_port);
    let jdbc_enabled = args.jdbc_enabled.or(config.jdbc_enabled);
    let max_connections = args.max_connections.or(config.max_connections);
    let jdbc_timeout = args.jdbc_timeout.or(config.jdbc_timeout);

    // 合并JDBC认证配置
    let jdbc_auth_enabled = args.jdbc_auth_enabled.or(config.jdbc_auth_enabled);
    let jdbc_username = args.jdbc_username.or(config.jdbc_username);
    let jdbc_password_hash = args.jdbc_password_hash.or(config.jdbc_password_hash);

    // 合并pubsub配置
    let pubsub_enabled = args
        .pubsub_enabled
        .or(config.pubsub.as_ref().and_then(|p| p.enabled));
    let pubsub_udp_bind = args.pubsub_udp_bind.or(config
        .pubsub
        .as_ref()
        .and_then(|p| p.udp_bind_address.clone()));
    let pubsub_heartbeat = args
        .pubsub_heartbeat
        .or(config.pubsub.as_ref().and_then(|p| p.heartbeat_interval));
    let pubsub_retrans_timeout = args.pubsub_retrans_timeout.or(config
        .pubsub
        .as_ref()
        .and_then(|p| p.retransmission_timeout));
    let pubsub_max_retrans = args
        .pubsub_max_retrans
        .or(config.pubsub.as_ref().and_then(|p| p.max_retransmissions));

    // 合并高可用配置
    let ha_enabled = args
        .ha_enabled
        .or(config.ha.as_ref().and_then(|h| h.enabled));
    let ha_node_id = args
        .ha_node_id
        .or(config.ha.as_ref().and_then(|h| h.node_id.clone()));
    let ha_role = args
        .ha_role
        .or(config.ha.as_ref().and_then(|h| h.role.clone()));
    let ha_replication_mode = args
        .ha_replication_mode
        .or(config.ha.as_ref().and_then(|h| h.replication_mode.clone()));
    let ha_heartbeat_interval = args
        .ha_heartbeat_interval
        .or(config.ha.as_ref().and_then(|h| h.heartbeat_interval));
    let ha_failure_detection_ms = args
        .ha_failure_detection_ms
        .or(config.ha.as_ref().and_then(|h| h.failure_detection_ms));
    let ha_sync_timeout_ms = args
        .ha_sync_timeout_ms
        .or(config.ha.as_ref().and_then(|h| h.sync_timeout_ms));
    let ha_master_address = args
        .ha_master_address
        .or(config.ha.as_ref().and_then(|h| h.master_address.clone()));
    let ha_master_port = args
        .ha_master_port
        .or(config.ha.as_ref().and_then(|h| h.master_port));
    let ha_replication_port = args
        .ha_replication_port
        .or(config.ha.as_ref().and_then(|h| h.replication_port));

    // 设置debug模式：命令行参数优先级高于配置文件
    let debug_mode = args.debug || config.debug.unwrap_or(false);
    set_debug_mode(debug_mode);

    if debug_mode {
        info!("Debug mode enabled");
    }

    // 初始化日志文件
    let log_file_name = if let Some(log_path_val) = log_path.as_ref() {
        // 用户指定了完整的日志文件路径
        let log_file = std::path::Path::new(log_path_val);
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        log_path_val.clone()
    } else {
        // 使用默认的日志目录和带有时间戳的文件名
        let log_file_path = "./logs";
        std::fs::create_dir_all(log_file_path).ok();
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}/remdb-server-{}.log", log_file_path, timestamp)
    };

    match remdb::init_logger_with_file(&log_file_name, debug_mode) {
        Ok(_) => {
            info!("Log file initialized at: {}", log_file_name);
        }
        Err(err) => {
            warn!("Failed to initialize log file: {:?}", err);
        }
    }

    // 手动初始化平台
    info!("Manually initializing platform...");
    init_platform();
    info!("Platform initialized manually");

    // 尝试从config中获取ddl_path，如果存在则加载
    let ddl_path = config.ddl_path.as_deref().unwrap_or("");
    let (tables, insert_statements): (Vec<remdb::TableDef>, Vec<String>) = if !ddl_path.is_empty() {
        info!("Loading DDL file: {}", ddl_path);
        match ddl_compiler::compile_ddl_file(ddl_path) {
            Ok(result) => {
                info!(
                    "DDL file loaded successfully, {} tables created",
                    result.0.len()
                );
                result
            }
            Err(err) => {
                warn!("Failed to load DDL file: {}, using empty tables", err);
                (Vec::new(), Vec::new())
            }
        }
    } else {
        // 没有配置DDL文件，尝试从快照目录加载表定义
        let mut tables_from_snapshot = Vec::new();

        // 优先从快照目录加载表定义
        if let Some(dir) = &snapshot_dir {
            info!(
                "No DDL file configured, attempting to load table definitions from snapshot directory: {}",
                dir
            );
            match snapshot_loader::load_table_defs_from_dir(dir) {
                Ok(loaded_tables) => {
                    info!(
                        "Loaded {} table definitions from snapshot directory",
                        loaded_tables.len()
                    );
                    tables_from_snapshot = loaded_tables;
                }
                Err(err) => {
                    warn!(
                        "Failed to load table definitions from snapshot directory: {:?}",
                        err
                    );
                }
            }
        }

        // 如果快照目录没有，尝试从WAL目录加载
        if tables_from_snapshot.is_empty() {
            let wal_dir = &config
                .wal
                .as_ref()
                .and_then(|w| w.log_path.clone())
                .unwrap_or_else(|| "./wal".to_string());
            info!(
                "Attempting to load table definitions from WAL directory: {}",
                wal_dir
            );
            if std::path::Path::new(wal_dir).exists() {
                // 从WAL目录查找快照文件
                match snapshot_loader::load_table_defs_from_dir(wal_dir) {
                    Ok(loaded_tables) => {
                        info!(
                            "Loaded {} table definitions from WAL directory",
                            loaded_tables.len()
                        );
                        tables_from_snapshot = loaded_tables;
                    }
                    Err(err) => {
                        warn!(
                            "Failed to load table definitions from WAL directory: {:?}",
                            err
                        );
                    }
                }
            }
        }

        if !tables_from_snapshot.is_empty() {
            info!(
                "Using table definitions loaded from snapshot: {} tables",
                tables_from_snapshot.len()
            );
            (tables_from_snapshot, Vec::new())
        } else {
            info!("No DDL file and no snapshot found, using empty tables");
            (Vec::new(), Vec::new())
        }
    };

    // 创建默认内存分配器
    static mut DEFAULT_ALLOCATOR: remdb::config::DefaultMemoryAllocator =
        remdb::config::DefaultMemoryAllocator;

    // 直接使用tables向量，不需要泄漏到静态内存
    let static_tables = tables;

    // 解析高可用配置
    let ha_enabled = ha_enabled.unwrap_or(false);

    // 根据配置设置节点角色
    let ha_role = if !ha_enabled {
        HARole::Master // 未启用高可用时默认为独立master
    } else {
        match ha_role.as_deref() {
            Some("slave") | Some("Slave") => HARole::Slave,
            _ => HARole::Master,
        }
    };

    // 根据配置设置复制模式
    let replication_mode = match ha_replication_mode.as_deref() {
        Some("sync") | Some("Sync") => ReplicationMode::Sync,
        _ => ReplicationMode::Async,
    };

    // 处理主节点地址，转换为&'static str
    let master_address =
        ha_master_address.map(|addr| Box::leak(addr.into_boxed_str()) as &'static str);

    // 处理节点ID，转换为u32类型
    let node_id = ha_node_id
        .as_deref()
        .and_then(|id| id.parse::<u32>().ok())
        .unwrap_or(1); // 默认节点ID为1

    // 修复 WAL 配置的 log_path 设置
    let wal_log_path = if let Some(log_path_val) = log_path.as_ref() {
        // 如果用户指定了日志文件路径，使用其目录作为 WAL 路径
        let log_file = std::path::Path::new(log_path_val);
        if let Some(parent) = log_file.parent() {
            parent.to_str().unwrap_or("./wal")
        } else {
            "./wal"
        }
    } else {
        // 使用默认的 WAL 路径
        "./wal"
    };

    // Save Milvus config before config variable is shadowed
    let milvus_config_saved = config.milvus.clone();

    // 创建配置
    let config = Box::leak(Box::new(remdb::config::DbConfig {
        tables: static_tables,
        total_memory: total_memory.unwrap_or(1024 * 1024 * 100), // 默认100MB
        low_power_mode_supported: low_power_mode_supported.unwrap_or(true), // 默认支持低功耗模式
        low_power_max_records: low_power_max_records,            // 使用配置文件或命令行参数中的值
        default_max_records: default_max_records.unwrap_or(10000), // 使用配置文件或命令行参数中的默认值，否则使用10000
        memory_allocator: unsafe {
            &*(&raw const DEFAULT_ALLOCATOR as *const _)
                as &'static dyn remdb::config::MemoryAllocator
        },
        wal_config: remdb::config::WALConfig {
            log_path: Box::leak(wal_log_path.to_string().into_boxed_str()) as &'static str,
            log_mode: match wal_log_mode.as_deref() {
                Some("sync") | Some("Sync") => remdb::config::LogMode::Sync,
                _ => remdb::config::LogMode::Async,
            },
            checkpoint_interval_ms: wal_checkpoint_interval_ms.unwrap_or(30000),
            log_file_size_limit: wal_log_file_size_limit.unwrap_or(16 * 1024 * 1024),
            log_prealloc_size: wal_log_prealloc_size.unwrap_or(4 * 1024 * 1024),
            log_segment_size: wal_log_segment_size.unwrap_or(16 * 1024 * 1024),
            retained_checkpoints: wal_retained_checkpoints.unwrap_or(3),
            max_consecutive_invalid: wal_max_consecutive_invalid.unwrap_or(100),
            skip_threshold: wal_skip_threshold.unwrap_or(20),
            skip_block_size: wal_skip_block_size.unwrap_or(4096),
            max_skip_attempts: wal_max_skip_attempts.unwrap_or(10),
            compression_type: remdb::config::WALCompressionType::None,
            compression_level: 1,
        },
        time_series_defaults: remdb::TimeSeriesConfig::DEFAULT, // 时序数据默认配置
        pubsub_config: None,                                    // PubSub配置，默认不使用
        ha_config: Some(remdb::ha::HAConfig {
            node_id,
            ha_role,
            replication_mode,
            heartbeat_interval_ms: ha_heartbeat_interval.unwrap_or(1000), // 默认1秒心跳
            failure_detection_ms: ha_failure_detection_ms.unwrap_or(5000), // 默认5秒故障检测
            sync_timeout_ms: ha_sync_timeout_ms.unwrap_or(2000),          // 默认2秒同步超时
            master_address,
            master_port: ha_master_port,
            replication_port: ha_replication_port.unwrap_or(6668), // 默认复制端口
                                                                   // 默认心跳端口
            }),
            model_worker_config: remdb::config::ModelWorkerConfig::default(),
        }));

    // 初始化全局内存分配器，这是关键的一步！
    let total_memory = config.total_memory;

    // 初始化HA manager（在内存分配器和数据库初始化之前，因为HA可能依赖于特定的初始化顺序）
    if ha_enabled {
        info!("Initializing HA manager...");
        match remdb::ha::init(config) {
            Ok(_) => info!("HA manager initialized successfully"),
            Err(e) => error!("Failed to initialize HA manager: {}", e),
        }
    }

    // 使用Vec<u8>在堆上分配内存，避免栈溢出
    let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
    let memory_ptr = memory_vec.as_ptr() as *mut u8;
    // 泄漏Vec，防止被自动释放
    std::mem::forget(memory_vec);

    if let Err(err) =
        unsafe { remdb::memory::allocator::init_global_allocator(memory_ptr, total_memory) }
    {
        error!("Failed to initialize global memory allocator: {:?}", err);
        return;
    }

    // 使用remdb库提供的init_global_db函数初始化数据库，这个函数会从配置中创建表
    let mut db = match unsafe { remdb::init_global_db(config) } {
        Ok(db) => db,
        Err(err) => {
            error!("Failed to initialize global database: {:?}", err);
            return;
        }
    };

    // 初始化索引构建线程池
    remdb::init_index_build_thread_pool(4);
    info!("Index build thread pool initialized with 4 threads");

    info!("Database initialized with {} tables", config.tables.len());

    // 标记是否从快照或WAL恢复了数据
    let mut data_restored = false;

    // 加载全量镜像文件（优先级最高）
    if let Some(full_image_path) = &full_image {
        info!("Loading full image file: {}", full_image_path);
        if let Err(err) = db.restore_snapshot(full_image_path) {
            error!("Failed to load full image: {:?}", err);
        } else {
            info!("Full image loaded successfully");
            data_restored = true;
        }
    } else {
        // 从WAL目录恢复数据（如果配置了WAL）
        let wal_dir = &config.wal_config.log_path;
        info!("Checking WAL directory: {}", wal_dir);
        if std::path::Path::new(wal_dir).exists() {
            info!("Loading and recovering from WAL directory: {}", wal_dir);
            if let Err(err) = snapshot_loader::load_from_wal_dir(&mut db, wal_dir) {
                warn!("Failed to recover from WAL: {:?}", err);
                // 如果WAL恢复失败，尝试从快照目录加载
                if let Some(snapshot_dir) = &snapshot_dir {
                    info!("Falling back to snapshot directory: {}", snapshot_dir);
                    if let Err(err) = snapshot_loader::load_snapshot_from_dir(&mut db, snapshot_dir)
                    {
                        warn!("Failed to load snapshot: {:?}", err);
                    } else {
                        info!("Snapshot loaded successfully");
                        data_restored = true;
                    }
                }
            } else {
                info!("Data recovered successfully from WAL");
                data_restored = true;
            }
        } else if let Some(snapshot_dir) = &snapshot_dir {
            // 如果没有WAL目录，尝试从快照目录加载
            info!("Loading snapshot from directory: {}", snapshot_dir);
            if let Err(err) = snapshot_loader::load_snapshot_from_dir(&mut db, snapshot_dir) {
                warn!("Failed to load snapshot: {:?}", err);
            } else {
                info!("Snapshot loaded successfully");
                data_restored = true;
            }
        }
    }

    // 执行DDL文件中的INSERT语句（仅当没有从快照或WAL恢复数据时）
    if !insert_statements.is_empty() && !data_restored {
        info!(
            "Executing {} INSERT statements from DDL file",
            insert_statements.len()
        );
        for stmt in insert_statements {
            info!("Executing: {}", stmt);
            match sql_engine::execute_extended_sql(&mut db, &stmt) {
                Ok(result) => {
                    info!(
                        "INSERT executed successfully, affected rows: {}",
                        result.affected_rows
                    );
                }
                Err(err) => {
                    error!("Failed to execute INSERT statement: {}", err);
                    error!("Statement: {}", stmt);
                }
            }
        }
    } else if !insert_statements.is_empty() && data_restored {
        info!("Skipping INSERT statements from DDL file (data already restored from snapshot/WAL)");
    } else if data_restored {
        info!("Data restored from snapshot/WAL, no DDL INSERT statements needed");
    }

    // 测试healthcheck命令
    if args.test_export {
        info!("\n=== Testing HEALTHCHECK command ===");
        match sql_engine::execute_extended_sql(&mut db, "healthcheck") {
            Ok(result) => {
                info!("\nHealthcheck result:");
                info!(
                    "+--------------------+----------+------------------------------------------------------------------+"
                );
                for (i, row) in result.rows.iter().enumerate() {
                    if i == 0 {
                        // 打印列名
                        let mut line = String::from("|");
                        for col in &result.columns {
                            line.push_str(&format!(" {:<18} ", col));
                            line.push('|');
                        }
                        info!("{}", line);
                        info!(
                            "+--------------------+----------+------------------------------------------------------------------+"
                        );
                    }
                    let mut line = String::from("|");
                    for (j, value) in row.iter().enumerate() {
                        let width = match j {
                            0 => 18,
                            1 => 8,
                            _ => 50,
                        };
                        line.push_str(&format!(" {:width$} ", value, width = width));
                        line.push('|');
                    }
                    info!("{}", line);
                }
                info!(
                    "+--------------------+----------+------------------------------------------------------------------+"
                );
            }
            Err(err) => {
                error!("Failed to execute healthcheck: {}", err);
            }
        }
    }

    // 将数据库实例泄漏到静态内存，以获取'static生命周期
    let db_static = Box::leak(Box::new(db));
    let db_mut_ref: &'static mut RemDb = db_static;

    // 将数据库实例包装在Arc<Mutex>中，以便在多线程环境中安全访问
    let db_arc = Arc::new(Mutex::new(db_mut_ref));

    // 如果配置了端口，则使用配置的端口，否则使用默认端口6666
    let actual_jdbc_port = jdbc_port.unwrap_or(6666);

    // 启动JDBC服务器（默认启用）
    let should_start_jdbc = jdbc_enabled.unwrap_or(true);
    if should_start_jdbc {
        let max_conns = max_connections.unwrap_or(5); // 默认最大连接数为5
        let jdbc_timeout = jdbc_timeout.unwrap_or(5); // 默认超时时间为5秒

        // JDBC认证配置默认值
        let auth_enabled = jdbc_auth_enabled.unwrap_or(false);
        let username = jdbc_username.unwrap_or_else(|| "admin".to_string());
        let password_hash = jdbc_password_hash.unwrap_or_else(|| {
            "8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918".to_string()
        });

        let jdbc_server = JdbcServer::new(
            db_arc.clone(),
            actual_jdbc_port,
            max_conns,
            jdbc_timeout,
            auth_enabled,
            username,
            password_hash,
        );

        info!(
            "Starting JDBC server on port {} with max connections {} and timeout {} seconds",
            actual_jdbc_port, max_conns, jdbc_timeout
        );
        info!(
            "JDBC authentication: {}",
            if auth_enabled { "enabled" } else { "disabled" }
        );

        // 在后台启动JDBC服务器
        tokio::spawn(async move {
            if let Err(e) = jdbc_server.start().await {
                error!("JDBC server failed to start: {:?}", e);
            }
        });
    } else {
        info!("JDBC server is disabled");
    }

    // Start Milvus RESTful API server if enabled
    let milvus_enabled = args.milvus_enabled.unwrap_or(milvus_config_saved.enabled);
    if milvus_enabled {
        let milvus_db = db_arc.clone();
        let milvus_port = args.milvus_port;
        let milvus_api_key = args.milvus_api_key.clone().or(milvus_config_saved.api_key.clone());
        tokio::spawn(async move {
            let server = remdb_server::milvus::MilvusServer::new(
                milvus_db,
                milvus_port,
                milvus_api_key,
            );
            server.start().await;
        });
        info!("Milvus RESTful API server enabled on port {}", milvus_port);
    }

    // 添加定时器线程，定期检查是否需要创建checkpoint
    let checkpoint_interval = wal_checkpoint_interval_ms.unwrap_or(30000);
    if checkpoint_interval > 0 {
        info!(
            "Starting checkpoint timer with interval {} ms",
            checkpoint_interval
        );

        // 在后台启动checkpoint定时器
        tokio::spawn(async move {
            let interval = tokio::time::Duration::from_millis(checkpoint_interval as u64);
            let mut timer = tokio::time::interval(interval);

            loop {
                timer.tick().await;

                // 尝试获取LogManager并检查是否需要创建checkpoint
                let log_manager_opt = unsafe { remdb::transaction::get_log_manager() };
                if let Some(log_manager) = log_manager_opt {
                    // 记录开始时间
                    let start = std::time::Instant::now();

                    // 调用检查函数，记录结果
                    match unsafe { log_manager.check_flush_and_checkpoint() } {
                        Ok(()) => {
                            let duration = start.elapsed();
                            let duration_ms = duration.as_secs_f64() * 1000.0;
                            info!(
                                "[Checkpoint Timer] Checkpoint executed successfully in {:.2} ms",
                                duration_ms
                            );
                        }
                        Err(e) => {
                            error!("[Checkpoint Timer] Failed to execute checkpoint: {:?}", e);
                        }
                    }
                } else {
                    warn!("[Checkpoint Timer] LogManager not available");
                }
            }
        });
    }

    // 添加定时器线程，定期创建快照
    if let Some(interval_secs) = snapshot_interval {
        if let Some(snap_type) = &snapshot_type {
            let snap_type = snap_type.to_lowercase();
            if snap_type == "full" || snap_type == "incremental" {
                // 克隆快照目录和其他配置，确保它们的生命周期足够长
                let snapshot_dir_clone = snapshot_dir.clone();
                let max_snapshots = max_incremental_snapshots.unwrap_or(10);

                info!(
                    "Starting snapshot timer with interval {} seconds, type: {}",
                    interval_secs, snap_type
                );

                // 在后台启动快照定时器
                tokio::spawn(async move {
                    let interval = tokio::time::Duration::from_secs(interval_secs);
                    let mut timer = tokio::time::interval(interval);

                    loop {
                        timer.tick().await;

                        // 尝试获取全局数据库实例
                        let db_opt = remdb::get_global_db();
                        if let Some(db_guard) = db_opt {
                            let db = &mut *db_guard;

                            // 记录开始时间
                            let start = std::time::Instant::now();

                            // 根据配置的快照类型创建快照
                            let result = if let Some(dir) = &snapshot_dir_clone {
                                if snap_type == "full" {
                                    snapshot_loader::save_full_snapshot_to_dir(db, dir)
                                } else {
                                    let res =
                                        snapshot_loader::save_incremental_snapshot_to_dir(db, dir);
                                    // 如果是增量快照，清理旧的快照
                                    if res.is_ok() {
                                        let _ = snapshot_loader::cleanup_old_snapshots(
                                            dir,
                                            max_snapshots,
                                        );
                                    }
                                    res
                                }
                            } else {
                                Err(remdb::RemDbError::FileIoError)
                            };

                            // 记录结果
                            match result {
                                Ok(()) => {
                                    let duration = start.elapsed();
                                    let duration_ms = duration.as_secs_f64() * 1000.0;
                                    info!(
                                        "[Snapshot Timer] {} snapshot executed successfully in {:.2} ms",
                                        snap_type, duration_ms
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "[Snapshot Timer] Failed to execute {} snapshot: {:?}",
                                        snap_type, e
                                    );
                                }
                            }
                        } else {
                            warn!("[Snapshot Timer] Database not available");
                        }
                    }
                });
            }
        }
    }

    // 初始化并启动PubSub系统（如果启用）
    if pubsub_enabled.unwrap_or(false) {
        use remdb::pubsub::{PubSubConfig, UdpMode, init as pubsub_init};

        // 从UDP绑定地址中提取端口号，默认使用6667
        let mut pubsub_port = 6667;
        if let Some(bind_addr) = &pubsub_udp_bind {
            if let Some(addr) = bind_addr.split(':').last() {
                if let Ok(port) = addr.parse::<u16>() {
                    pubsub_port = port;
                }
            }
        }

        // 创建PubSub配置
        let pubsub_config = PubSubConfig {
            udp_mode: UdpMode::Unicast,
            multicast_addr: None,
            port: pubsub_port,
            max_topics: 32,
            max_subscribers_per_topic: 16,
            buffer_size: 4096,
            enable_nack: true,
            retransmit_timeout: std::time::Duration::from_millis(
                pubsub_retrans_timeout.unwrap_or(500) as u64,
            ),
            max_retransmits: pubsub_max_retrans.unwrap_or(3) as usize,
            heartbeat_interval: std::time::Duration::from_millis(
                pubsub_heartbeat.unwrap_or(1000) as u64
            ),
            frame_pool_size: 128,
        };

        // 初始化PubSub系统
        if let Err(err) = pubsub_init(pubsub_config) {
            warn!("Failed to initialize PubSub system: {:?}", err);
        } else {
            info!(
                "PubSub system initialized successfully on port {}",
                pubsub_port
            );
        }
    } else {
        info!("PubSub server is disabled");
    }

    // 启动交互式控制台（如果启用且不是非交互式模式）
    if !args.non_interactive {
        // 只在非交互式模式下启动CLI，JDBC模式下不启动CLI
        if !should_start_jdbc {
            let mut db_lock = db_arc.lock().unwrap();
            cli::run_cli(&mut db_lock);
        } else {
            info!(
                "\n--- JDBC server is running on port {} ---",
                actual_jdbc_port
            );
            info!("Interactive CLI is disabled when JDBC server is running.");
            info!("Use --non-interactive=false to enable CLI in non-JDBC mode.");
            info!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            info!("\nStopping JDBC server...");
        }
    } else {
        // 非交互式模式下，如果启用了JDBC服务，等待Ctrl+C
        if should_start_jdbc {
            info!(
                "\n--- JDBC server is running on port {} ---",
                actual_jdbc_port
            );
            info!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            info!("\nStopping JDBC server...");
        }
    }

    // 程序退出前关闭HA manager
    if ha_enabled {
        info!("Stopping HA manager...");
        use remdb::ha::shutdown as ha_shutdown;
        if let Err(err) = ha_shutdown() {
            warn!("Failed to shutdown HA manager: {:?}", err);
        }
    }

    // 程序退出前关闭PubSub系统
    info!("Stopping PubSub server...");
    use remdb::pubsub::shutdown as pubsub_shutdown;
    if let Err(err) = pubsub_shutdown() {
        warn!("Failed to shutdown PubSub server: {:?}", err);
    }
}
