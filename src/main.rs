mod ddl_compiler;
mod snapshot_loader;
mod sql_engine;
mod cli;
mod jdbc_server;

use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;
use crate::ddl_compiler::compile_ddl_file;
use crate::snapshot_loader::load_snapshot_from_dir;
use crate::cli::run_cli;
use crate::sql_engine::{execute_extended_sql, format_result_set};
use crate::jdbc_server::JdbcServer;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write, Seek};
use std::time::SystemTime;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

// 全局debug模式开关
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// 设置debug模式
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
}

/// 检查是否开启了debug模式
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

/// 调试日志宏，只有在debug模式下才输出
#[macro_export]
macro_rules! debug_println {
    ($($args:tt)*) => {
        if $crate::is_debug_mode() {
            println!($($args)*);
        }
    };
}

/// 调试错误日志宏，只有在debug模式下才输出
#[macro_export]
macro_rules! debug_eprintln {
    ($($args:tt)*) => {
        if $crate::is_debug_mode() {
            eprintln!($($args)*);
        }
    };
}

// 定义Windows平台实现，用于非POSIX平台
struct WindowsPlatform;

impl remdb::platform::Platform for WindowsPlatform {
    fn get_timestamp(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        now.as_millis() as u64
    }
    
    fn get_timestamp_us(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards");
        now.as_micros() as u64
    }
    
    fn spin_lock(&self, lock: &mut u32) {
        // 使用原子比较交换实现自旋锁
        while unsafe {
            core::sync::atomic::AtomicU32::from_ptr(lock as *mut u32)
                .compare_exchange(0, 1, 
                                 core::sync::atomic::Ordering::Acquire,
                                 core::sync::atomic::Ordering::Relaxed)
                .is_err()
        } {
            // 自旋等待
            core::hint::spin_loop();
        }
    }
    
    fn spin_unlock(&self, lock: &mut u32) {
        unsafe {
            core::sync::atomic::AtomicU32::from_ptr(lock as *mut u32)
                .store(0, core::sync::atomic::Ordering::Release);
        }
    }
    
    fn compiler_barrier(&self) {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
    
    fn full_memory_barrier(&self) {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }
    
    fn memcpy(&self, dest: *mut u8, src: *const u8, size: usize) {
        unsafe {
            ptr::copy_nonoverlapping(src, dest, size);
        }
    }
    
    fn memset(&self, dest: *mut u8, value: u8, size: usize) {
        unsafe {
            ptr::write_bytes(dest, value, size);
        }
    }
    
    fn delay_ms(&self, ms: u32) {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
    
    fn delay_us(&self, us: u32) {
        std::thread::sleep(std::time::Duration::from_micros(us as u64));
    }
    
    fn file_open(&self, path: &str, mode: remdb::platform::FileMode) -> remdb::platform::FileResult<remdb::platform::FileHandle> {
        use std::fs::OpenOptions;
        
        let mut options = OpenOptions::new();
        match mode {
            remdb::platform::FileMode::Read => {
                options.read(true);
            },
            remdb::platform::FileMode::Write => {
                options.write(true).create(true).truncate(true);
            },
            remdb::platform::FileMode::ReadWrite => {
                options.read(true).write(true).create(true);
            },
            remdb::platform::FileMode::Append => {
                options.write(true).create(true).append(true);
            },
        }
        
        match options.open(path) {
            Ok(file) => {
                let boxed_file = Box::new(file);
                Ok(Box::into_raw(boxed_file) as remdb::platform::FileHandle)
            },
            Err(_) => Err(()),
        }
    }
    
    fn file_close(&self, handle: remdb::platform::FileHandle) -> remdb::platform::FileResult<()> {
        unsafe {
            let _ = Box::from_raw(handle as *mut std::fs::File);
        }
        Ok(())
    }
    
    fn file_write(&self, handle: remdb::platform::FileHandle, buffer: *const u8, size: usize) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts(buffer, size);
            match file.write(slice) {
                Ok(bytes_written) => {
                    file.flush().map_err(|_| ())?;
                    Ok(bytes_written)
                },
                Err(_) => Err(()),
            }
        }
    }
    
    fn file_read(&self, handle: remdb::platform::FileHandle, buffer: *mut u8, size: usize) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts_mut(buffer, size);
            match file.read(slice) {
                Ok(bytes_read) => Ok(bytes_read),
                Err(_) => Err(()),
            }
        }
    }
    
    fn file_seek(&self, handle: remdb::platform::FileHandle, offset: i64, whence: remdb::platform::SeekWhence) -> remdb::platform::FileResult<u64> {
        use std::io::Seek;
        
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let seek_from = match whence {
                remdb::platform::SeekWhence::SeekSet => std::io::SeekFrom::Start(offset as u64),
                remdb::platform::SeekWhence::SeekCur => std::io::SeekFrom::Current(offset),
                remdb::platform::SeekWhence::SeekEnd => std::io::SeekFrom::End(offset),
            };
            match file.seek(seek_from) {
                Ok(new_pos) => Ok(new_pos),
                Err(_) => Err(()),
            }
        }
    }
    
    fn file_remove(&self, path: &str) -> remdb::platform::FileResult<()> {
        match std::fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
    
    fn file_size(&self, path: &str) -> remdb::platform::FileResult<usize> {
        use std::fs::metadata;
        match metadata(path) {
            Ok(metadata) => Ok(metadata.len() as usize),
            Err(_) => Err(()),
        }
    }
    
    fn crc32(&self, data: *const u8, size: usize) -> u32 {
        const CRC32_POLY: u32 = 0xEDB88320;
        let mut crc_table = [0u32; 256];
        for i in 0..256 {
            let mut crc = i as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ CRC32_POLY;
                } else {
                    crc >>= 1;
                }
            }
            crc_table[i] = crc;
        }
        let mut crc = 0xFFFFFFFFu32;
        let data_slice = unsafe { core::slice::from_raw_parts(data, size) };
        for &byte in data_slice {
            let index = ((crc ^ byte as u32) & 0xFF) as usize;
            crc = (crc >> 8) ^ crc_table[index];
        }
        crc ^ 0xFFFFFFFFu32
    }
}

// 创建静态平台实例
static WINDOWS_PLATFORM: WindowsPlatform = WindowsPlatform;

/// 高可用配置
#[derive(Deserialize, Debug, Default)]
struct HaConfig {
    /// 是否启用高可用功能
    enabled: Option<bool>,
    
    /// 节点角色（master/slave）
    role: Option<String>,
    
    /// 复制模式（async/sync）
    replication_mode: Option<String>,
    
    /// 心跳间隔（毫秒）
    heartbeat_interval: Option<u64>,
    
    /// 故障检测时间（毫秒）
    failure_detection_ms: Option<u64>,
    
    /// 同步超时时间（毫秒）
    sync_timeout_ms: Option<u64>,
    
    /// 主节点地址（仅slave节点需要）
    master_address: Option<String>,
    
    /// 主节点端口（仅slave节点需要）
    master_port: Option<u16>,
}

/// 配置文件结构体
#[derive(Deserialize, Debug, Default)]
struct Config {
    /// DDL文件路径
    ddl: Option<String>,
    
    /// 快照存储目录
    snapshot_dir: Option<String>,
    
    /// 数据库总内存大小（字节）
    total_memory: Option<usize>,

    /// 默认最大记录数
    default_max_records: Option<usize>, 
    
    /// 是否支持低功耗模式
    low_power_mode_supported: Option<bool>,
    
    /// 低功耗模式下的最大记录数
    low_power_max_records: Option<usize>,
    
    /// 增量快照周期（秒）
    snapshot_interval: Option<u64>,
    
    /// 最大增量快照数量
    max_incremental_snapshots: Option<usize>,
    
    /// 是否开启debug模式
    debug: Option<bool>,
    
    /// JDBC监听端口
    jdbc_port: Option<u16>,
    
    /// 是否启用JDBC服务
    jdbc_enabled: Option<bool>,
    
    /// 最大允许的并发jdbc客户端连接数
    max_connections: Option<usize>,
    
    /// JDBC执行超时时间（秒）
    jdbc_timeout: Option<u64>,
    
    /// JDBC认证配置
    /// 是否启用JDBC认证
    jdbc_auth_enabled: Option<bool>,
    /// JDBC认证用户名
    jdbc_username: Option<String>,
    /// JDBC认证密码哈希值
    jdbc_password_hash: Option<String>,
    
    /// pubsub配置
    pubsub: Option<PubSubConfig>,
    
    /// 高可用配置
    ha: Option<HaConfig>,
}

/// pubsub配置
#[derive(Deserialize, Debug, Default)]
struct PubSubConfig {
    /// 是否启用pubsub功能
    enabled: Option<bool>,
    
    /// UDP绑定地址
    udp_bind_address: Option<String>,
    
    /// 心跳间隔（毫秒）
    heartbeat_interval: Option<u32>,
    
    /// 重传超时（毫秒）
    retransmission_timeout: Option<u32>,
    
    /// 最大重传次数
    max_retransmissions: Option<u32>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(long, short)]
    config: Option<String>,
    
    /// DDL文件路径
    #[arg(long)]
    ddl: Option<String>,
    
    /// 快照存储目录
    #[arg(long)]
    snapshot_dir: Option<String>,
    
    /// 全量镜像文件路径
    #[arg(long)]
    full_image: Option<String>,
    
    /// 数据库总内存大小（字节）
    #[arg(long)]
    total_memory: Option<usize>,

    /// 默认最大记录数
    #[arg(long)]
    default_max_records: Option<usize>, 
    
    /// 是否支持低功耗模式
    #[arg(long)]
    low_power_mode_supported: Option<bool>,
    
    /// 低功耗模式下的最大记录数
    #[arg(long)]
    low_power_max_records: Option<usize>,
    
    /// 增量快照周期（秒）
    #[arg(long)]
    snapshot_interval: Option<u64>,
    
    /// 最大增量快照数量
    #[arg(long)]
    max_incremental_snapshots: Option<usize>,
    
    /// 是否开启debug模式
    #[arg(long, short)]
    debug: bool,
    
    /// 非交互式模式（初始化后退出）
    #[arg(long)]
    non_interactive: bool,
    
    /// 测试导出功能
    #[arg(long)]
    test_export: bool,
    
    /// JDBC监听端口
    #[arg(long)]
    jdbc_port: Option<u16>,
    
    /// 是否启用JDBC服务
    #[arg(long)]
    jdbc_enabled: Option<bool>,
    
    /// 最大允许的并发jdbc客户端连接数
    #[arg(long)]
    max_connections: Option<usize>,
    
    /// JDBC执行超时时间（秒）
    #[arg(long)]
    jdbc_timeout: Option<u64>,
    
    /// 是否启用JDBC认证
    #[arg(long)]
    jdbc_auth_enabled: Option<bool>,
    
    /// JDBC认证用户名
    #[arg(long)]
    jdbc_username: Option<String>,
    
    /// JDBC认证密码哈希值
    #[arg(long)]
    jdbc_password_hash: Option<String>,
    
    /// 是否启用pubsub功能
    #[arg(long)]
    pubsub_enabled: Option<bool>,
    
    /// UDP绑定地址
    #[arg(long)]
    pubsub_udp_bind: Option<String>,
    
    /// 心跳间隔（毫秒）
    #[arg(long)]
    pubsub_heartbeat: Option<u32>,
    
    /// 重传超时（毫秒）
    #[arg(long)]
    pubsub_retrans_timeout: Option<u32>,
    
    /// 最大重传次数
    #[arg(long)]
    pubsub_max_retrans: Option<u32>,
    
    /// 是否启用高可用功能
    #[arg(long)]
    ha_enabled: Option<bool>,
    
    /// 节点角色（master/slave）
    #[arg(long)]
    ha_role: Option<String>,
    
    /// 复制模式（async/sync）
    #[arg(long)]
    ha_replication_mode: Option<String>,
    
    /// 心跳间隔（毫秒）
    #[arg(long)]
    ha_heartbeat_interval: Option<u64>,
    
    /// 故障检测时间（毫秒）
    #[arg(long)]
    ha_failure_detection_ms: Option<u64>,
    
    /// 同步超时时间（毫秒）
    #[arg(long)]
    ha_sync_timeout_ms: Option<u64>,
    
    /// 主节点地址（仅slave节点需要）
    #[arg(long)]
    ha_master_address: Option<String>,
    
    /// 主节点端口（仅slave节点需要）
    #[arg(long)]
    ha_master_port: Option<u16>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    println!("remdb-server v0.1.0");
    
    // 读取配置文件
    let mut config = Config::default();
    if let Some(config_path) = &args.config {
        println!("Reading config file: {}", config_path);
        match fs::read_to_string(config_path) {
            Ok(content) => {
                match toml::from_str(&content) {
                    Ok(parsed_config) => {
                        config = parsed_config;
                        println!("Config file loaded successfully");
                    }
                    Err(err) => {
                        eprintln!("Warning: Failed to parse config file: {:?}", err);
                        eprintln!("Using default config values");
                    }
                }
            }
            Err(err) => {
                eprintln!("Warning: Failed to read config file: {:?}", err);
                eprintln!("Using default config values");
            }
        }
    }
    
    // 合并配置：命令行参数优先级高于配置文件
    let ddl_path = args.ddl.or(config.ddl);
    let snapshot_dir = args.snapshot_dir.or(config.snapshot_dir);
    let full_image = args.full_image.clone();
    let total_memory = args.total_memory.or(config.total_memory);
    let default_max_records = args.default_max_records.or(config.default_max_records);
    let low_power_mode_supported = args.low_power_mode_supported.or(config.low_power_mode_supported);
    let low_power_max_records = args.low_power_max_records.or(config.low_power_max_records);
    let snapshot_interval = args.snapshot_interval.or(config.snapshot_interval);
    let max_incremental_snapshots = args.max_incremental_snapshots.or(config.max_incremental_snapshots);
    let jdbc_port = args.jdbc_port.or(config.jdbc_port);
    let jdbc_enabled = args.jdbc_enabled.or(config.jdbc_enabled);
    let max_connections = args.max_connections.or(config.max_connections);
    let jdbc_timeout = args.jdbc_timeout.or(config.jdbc_timeout);
    
    // 合并JDBC认证配置
    let jdbc_auth_enabled = args.jdbc_auth_enabled.or(config.jdbc_auth_enabled);
    let jdbc_username = args.jdbc_username.or(config.jdbc_username);
    let jdbc_password_hash = args.jdbc_password_hash.or(config.jdbc_password_hash);
    
    // 合并pubsub配置
    let pubsub_enabled = args.pubsub_enabled.or(config.pubsub.as_ref().and_then(|p| p.enabled));
    let pubsub_udp_bind = args.pubsub_udp_bind.or(config.pubsub.as_ref().and_then(|p| p.udp_bind_address.clone()));
    let pubsub_heartbeat = args.pubsub_heartbeat.or(config.pubsub.as_ref().and_then(|p| p.heartbeat_interval));
    let pubsub_retrans_timeout = args.pubsub_retrans_timeout.or(config.pubsub.as_ref().and_then(|p| p.retransmission_timeout));
    let pubsub_max_retrans = args.pubsub_max_retrans.or(config.pubsub.as_ref().and_then(|p| p.max_retransmissions));
    
    // 合并高可用配置
    let ha_enabled = args.ha_enabled.or(config.ha.as_ref().and_then(|h| h.enabled));
    let ha_role = args.ha_role.or(config.ha.as_ref().and_then(|h| h.role.clone()));
    let ha_replication_mode = args.ha_replication_mode.or(config.ha.as_ref().and_then(|h| h.replication_mode.clone()));
    let ha_heartbeat_interval = args.ha_heartbeat_interval.or(config.ha.as_ref().and_then(|h| h.heartbeat_interval));
    let ha_failure_detection_ms = args.ha_failure_detection_ms.or(config.ha.as_ref().and_then(|h| h.failure_detection_ms));
    let ha_sync_timeout_ms = args.ha_sync_timeout_ms.or(config.ha.as_ref().and_then(|h| h.sync_timeout_ms));
    let ha_master_address = args.ha_master_address.or(config.ha.as_ref().and_then(|h| h.master_address.clone()));
    let ha_master_port = args.ha_master_port.or(config.ha.as_ref().and_then(|h| h.master_port));
    
    // 设置debug模式：命令行参数优先级高于配置文件
    let debug_mode = args.debug || config.debug.unwrap_or(false);
    set_debug_mode(debug_mode);
    if debug_mode {
        println!("Debug mode enabled");
    }
    
    // 手动初始化平台
    println!("Manually initializing platform...");
    remdb::platform::init_platform(&WINDOWS_PLATFORM);
    println!("Platform initialized manually");
    
    // 解析DDL文件（如果提供）
    let tables = if let Some(ddl_path) = ddl_path {
        println!("Compiling DDL file: {}", ddl_path);
        match compile_ddl_file(&ddl_path) {
            Ok(tables) => {
                println!("✓ Successfully compiled DDL file");
                debug_println!("Debug: Compiled {} tables:", tables.len());
                for table in &tables {
                    debug_println!("Debug: - Table: {}", table.name);
                }
                tables
            }
            Err(err) => {
                eprintln!("Error: Failed to compile DDL file: {:?}", err);
                return;
            }
        }
    } else {
        Vec::new()
    };
    
    // 创建默认内存分配器
    static mut DEFAULT_ALLOCATOR: remdb::config::DefaultMemoryAllocator = remdb::config::DefaultMemoryAllocator;
    
    // 使用更大的默认最大记录数，允许更多记录
    let small_max_records = 1000; // 允许1000条记录
    
    // 首先将tables向量泄漏到静态内存，确保TableDef有'static生命周期
    let static_tables = Box::leak(Box::new(tables));
    
    // 解析高可用配置
    let ha_enabled = ha_enabled.unwrap_or(false);
    
    // 根据配置设置节点角色
    let ha_role = if !ha_enabled {
        remdb::config::HARole::Master // 未启用高可用时默认为独立master
    } else {
        match ha_role.as_deref() {
            Some("slave") | Some("Slave") => remdb::config::HARole::Slave,
            _ => remdb::config::HARole::Master,
        }
    };
    
    // 根据配置设置复制模式
    let replication_mode = match ha_replication_mode.as_deref() {
        Some("sync") | Some("Sync") => remdb::config::ReplicationMode::Sync,
        _ => remdb::config::ReplicationMode::Async,
    };
    
    // 处理主节点地址，转换为&'static str
    let master_address = ha_master_address.map(|addr| {
        Box::leak(addr.into_boxed_str()) as &'static str
    });
    
    // 创建配置
    let config = Box::leak(Box::new(remdb::config::DbConfig {
        tables: static_tables,
        total_memory: total_memory.unwrap_or(1024 * 1024 * 100), // 默认100MB
        low_power_mode_supported: low_power_mode_supported.unwrap_or(true), // 默认支持低功耗模式
        low_power_max_records: Some(low_power_max_records.unwrap_or(100)), // 默认100条记录
        default_max_records: default_max_records.unwrap_or(small_max_records), // 使用配置文件或命令行参数中的默认值，否则使用1000
        memory_allocator: unsafe {
            &*(&raw const DEFAULT_ALLOCATOR as *const _) as &'static dyn remdb::config::MemoryAllocator
        },
        log_mode: remdb::config::LogMode::Async, // 默认异步日志模式
        checkpoint_interval_ms: 30000, // 默认30秒
        log_file_size_limit: 16 * 1024 * 1024, // 默认16MB
        log_prealloc_size: 4 * 1024 * 1024, // 默认4MB
        log_segment_size: 16 * 1024 * 1024, // 默认16MB
        retained_checkpoints: 3, // 默认保留3个检查点
        ha_role,
        replication_mode,
        heartbeat_interval_ms: ha_heartbeat_interval.unwrap_or(1000), // 默认1秒心跳
        failure_detection_ms: ha_failure_detection_ms.unwrap_or(5000), // 默认5秒故障检测
        sync_timeout_ms: ha_sync_timeout_ms.unwrap_or(2000), // 默认2秒同步超时
        master_address,
        master_port: ha_master_port,
    }));
    
    // 初始化全局内存分配器，这是关键的一步！
    let total_memory = config.total_memory;
    // 使用Vec<u8>在堆上分配内存，避免栈溢出
    let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
    let memory_ptr = memory_vec.as_ptr() as *mut u8;
    // 泄漏Vec，防止被自动释放
    std::mem::forget(memory_vec);
    
    if let Err(err) = unsafe {
        remdb::memory::allocator::init_global_allocator(memory_ptr, total_memory)
    } {
        eprintln!("Error: Failed to initialize global memory allocator: {:?}", err);
        return;
    }
    
    // 使用remdb库提供的init_global_db函数初始化数据库，这个函数会从配置中创建表
    let mut db = match unsafe {
        remdb::init_global_db(config)
    } {
        Ok(db) => db,
        Err(err) => {
            eprintln!("Error: Failed to initialize global database: {:?}", err);
            return;
        }
    };
    
    println!("Database initialized with {} tables", config.tables.len());
    
    // 加载快照
    if let Some(snapshot_dir) = &snapshot_dir {
        println!("Loading snapshot from directory: {}", snapshot_dir);
        if let Err(err) = snapshot_loader::load_snapshot_from_dir(&mut db, snapshot_dir) {
            eprintln!("Warning: Failed to load snapshot: {:?}", err);
        } else {
            println!("Snapshot loaded successfully");
        }
    }
    
    // 加载全量镜像文件
    if let Some(full_image_path) = &full_image {
        println!("Loading full image file: {}", full_image_path);
        if let Err(err) = db.restore_snapshot(full_image_path) {
            eprintln!("Error: Failed to load full image: {:?}", err);
        } else {
            println!("Full image loaded successfully");
        }
    }
        
    // 将数据库实例包装在Arc<Mutex>中，以便在多线程环境中安全访问
    let db_arc = Arc::new(Mutex::new(db));
    
    // 如果配置了端口，则使用配置的端口，否则使用默认端口6666
    let actual_jdbc_port = jdbc_port.unwrap_or(6666);
    
    // 启动JDBC服务器（如果启用了JDBC服务且配置了端口，或者没有明确禁用）
    let should_start_jdbc = jdbc_enabled.unwrap_or(jdbc_port.is_some());
    if should_start_jdbc {
        let max_conns = max_connections.unwrap_or(5); // 默认最大连接数为5
        let jdbc_timeout = jdbc_timeout.unwrap_or(5); // 默认超时时间为5秒
        
        // JDBC认证配置默认值
        let auth_enabled = jdbc_auth_enabled.unwrap_or(false);
        let username = jdbc_username.unwrap_or_else(|| "admin".to_string());
        let password_hash = jdbc_password_hash.unwrap_or_else(|| "8c6976e5b5410415bde908bd4dee15dfb167a9c873fc4bb8a81f6f2ab448a918".to_string());
        
        let jdbc_server = JdbcServer::new(
            db_arc.clone(), 
            actual_jdbc_port, 
            max_conns, 
            jdbc_timeout,
            auth_enabled,
            username,
            password_hash
        );
        
        println!("Starting JDBC server on port {} with max connections {} and timeout {} seconds", actual_jdbc_port, max_conns, jdbc_timeout);
        println!("JDBC authentication: {}", if auth_enabled { "enabled" } else { "disabled" });
        
        // 在后台启动JDBC服务器
        tokio::spawn(async move {
            if let Err(e) = jdbc_server.start().await {
                eprintln!("Error: JDBC server failed to start: {:?}", e);
            }
        });
    } else {
        println!("JDBC server is disabled");
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
            retransmit_timeout: std::time::Duration::from_millis(pubsub_retrans_timeout.unwrap_or(500) as u64),
            max_retransmits: pubsub_max_retrans.unwrap_or(3) as usize,
            heartbeat_interval: std::time::Duration::from_millis(pubsub_heartbeat.unwrap_or(1000) as u64),
            frame_pool_size: 128,
        };
        
        // 初始化PubSub系统
        if let Err(err) = pubsub_init(pubsub_config) {
            eprintln!("Warning: Failed to initialize PubSub system: {:?}", err);
        } else {
            println!("PubSub system initialized successfully on port {}", pubsub_port);
        }
    }
    
    // 启动交互式控制台（如果启用且不是非交互式模式）
    if !args.non_interactive {
        // 只在非交互式模式下启动CLI，JDBC模式下不启动CLI
        if !should_start_jdbc {
            let mut db_lock = db_arc.lock().await;
            cli::run_cli(&mut db_lock);
        } else {
            println!("\n--- JDBC server is running on port {} ---", actual_jdbc_port);
            println!("Interactive CLI is disabled when JDBC server is running.");
            println!("Use --non-interactive=false to enable CLI in non-JDBC mode.");
            println!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            println!("\nStopping JDBC server...");
        }
    } else {
        // 非交互式模式下，如果启用了JDBC服务，等待Ctrl+C
        if should_start_jdbc {
            println!("\n--- JDBC server is running on port {} ---", actual_jdbc_port);
            println!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            println!("\nStopping JDBC server...");
        }
    }
    
    // 程序退出前关闭PubSub系统
    println!("Stopping PubSub server...");
    use remdb::pubsub::shutdown as pubsub_shutdown;
    if let Err(err) = pubsub_shutdown() {
        eprintln!("Warning: Failed to shutdown PubSub server: {:?}", err);
    }
}