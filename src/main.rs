use remdb::{
    RemDb,
    ha::{HARole, ReplicationMode},
};

use clap::Parser;
use core::ptr;
use remdb_server::jdbc_server::JdbcServer;
use remdb_server::{is_debug_mode, set_debug_mode, set_global_log_handle};
use serde::Deserialize;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[macro_use]
mod macros;
mod benchmark;
mod cli;
mod ddl_compiler;
mod snapshot_loader;
mod sql_engine;

// 全局日志文件句柄
static mut LOG_FILE_HANDLE: Option<Arc<Mutex<std::fs::File>>> = None;

/// 设置日志文件
pub fn set_log_file(log_path: &str) -> std::io::Result<()> {
    // 创建日志目录
    std::fs::create_dir_all(log_path)?;

    // 获取当前日期作为日志文件名
    let now = SystemTime::now();
    let timestamp = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let log_file_path = format!("{}/remdb-server-{}.log", log_path, timestamp);

    // 打开日志文件，以追加模式写入
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(log_file_path)?;

    // 将文件句柄包装到Arc<Mutex>中，以便多线程安全访问
    let file_handle = Arc::new(Mutex::new(file));

    // 保存到全局变量
    unsafe {
        LOG_FILE_HANDLE = Some(file_handle.clone());
        set_global_log_handle(file_handle);
    }

    Ok(())
}

/// 写入日志到文件
pub fn write_log_to_file(message: &str) {
    unsafe {
        if let Some(ref file_handle) = LOG_FILE_HANDLE {
            if let Ok(mut file) = file_handle.lock() {
                let _ = writeln!(file, "{}", message);
            }
        }
    }
}

/// 重定义标准println宏，使其同时输出到控制台和日志文件
macro_rules! log_println {
    ($($args:tt)*) => {
        {
            let message = format!($($args)*);
            println!("{}", message);
            write_log_to_file(&message);
        }
    };
}

/// 重定义标准eprintln宏，使其同时输出到控制台和日志文件
macro_rules! log_eprintln {
    ($($args:tt)*) => {
        {
            let message = format!($($args)*);
            eprintln!("{}", message);
            write_log_to_file(&message);
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
                .compare_exchange(
                    0,
                    1,
                    core::sync::atomic::Ordering::Acquire,
                    core::sync::atomic::Ordering::Relaxed,
                )
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

    fn file_open(
        &self,
        path: &str,
        mode: remdb::platform::FileMode,
    ) -> remdb::platform::FileResult<remdb::platform::FileHandle> {
        use std::fs::OpenOptions;

        let mut options = OpenOptions::new();
        match mode {
            remdb::platform::FileMode::Read => {
                options.read(true);
            }
            remdb::platform::FileMode::Write => {
                options.write(true).create(true).truncate(true);
            }
            remdb::platform::FileMode::ReadWrite => {
                options.read(true).write(true).create(true);
            }
            remdb::platform::FileMode::Append => {
                options.write(true).create(true).append(true);
            }
        }

        match options.open(path) {
            Ok(file) => {
                let boxed_file = Box::new(file);
                Ok(Box::into_raw(boxed_file) as remdb::platform::FileHandle)
            }
            Err(_) => Err(()),
        }
    }

    fn file_close(&self, handle: remdb::platform::FileHandle) -> remdb::platform::FileResult<()> {
        unsafe {
            let _ = Box::from_raw(handle as *mut std::fs::File);
        }
        Ok(())
    }

    fn file_write(
        &self,
        handle: remdb::platform::FileHandle,
        buffer: *const u8,
        size: usize,
    ) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts(buffer, size);
            match file.write(slice) {
                Ok(bytes_written) => {
                    file.flush().map_err(|_| ())?;
                    Ok(bytes_written)
                }
                Err(_) => Err(()),
            }
        }
    }

    fn file_read(
        &self,
        handle: remdb::platform::FileHandle,
        buffer: *mut u8,
        size: usize,
    ) -> remdb::platform::FileResult<usize> {
        unsafe {
            let file = &mut *(handle as *mut std::fs::File);
            let slice = core::slice::from_raw_parts_mut(buffer, size);
            match file.read(slice) {
                Ok(bytes_read) => Ok(bytes_read),
                Err(_) => Err(()),
            }
        }
    }

    fn file_seek(
        &self,
        handle: remdb::platform::FileHandle,
        offset: i64,
        whence: remdb::platform::SeekWhence,
    ) -> remdb::platform::FileResult<u64> {
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

/// WAL配置
#[derive(Deserialize, Debug, Default)]
struct WALConfig {
    /// 日志文件路径
    log_path: Option<String>,

    /// 日志模式，可选值：async, sync
    log_mode: Option<String>,

    /// 检查点间隔（毫秒）
    checkpoint_interval_ms: Option<u64>,

    /// 日志文件大小限制（字节）
    log_file_size_limit: Option<usize>,

    /// 日志预分配大小（字节）
    log_prealloc_size: Option<usize>,

    /// 日志段大小（字节）
    log_segment_size: Option<usize>,

    /// 保留的检查点数量
    retained_checkpoints: Option<usize>,
}

/// 高可用配置
#[derive(Deserialize, Debug, Default)]
struct HaConfig {
    /// 是否启用高可用功能
    enabled: Option<bool>,

    /// 节点ID
    node_id: Option<String>,

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

    /// 复制端口（用于WAL日志复制和数据同步）
    replication_port: Option<u16>,
}

/// 配置文件结构体
#[derive(Deserialize, Debug, Default)]
struct Config {
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

    /// 日志文件路径
    log_path: Option<String>,

    /// 增量快照周期（秒）
    snapshot_interval: Option<u64>,

    /// 快照类型：full（全量）或incremental（增量）
    snapshot_type: Option<String>,

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

    /// WAL配置
    wal: Option<WALConfig>,

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
    #[command(subcommand)]
    command: Option<Command>,

    /// 配置文件路径
    #[arg(long, short)]
    config: Option<String>,

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

    /// 日志文件路径
    #[arg(long)]
    log_path: Option<String>,

    /// 增量快照周期（秒）
    #[arg(long)]
    snapshot_interval: Option<u64>,

    /// 快照类型：full（全量）或incremental（增量）
    #[arg(long)]
    snapshot_type: Option<String>,

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

    /// 复制端口（用于WAL日志复制和数据同步）
    #[arg(long)]
    ha_replication_port: Option<u16>,

    /// 心跳端口（用于节点间心跳检测）
    #[arg(long)]
    ha_heartbeat_port: Option<u16>,

    /// 节点ID
    #[arg(long)]
    ha_node_id: Option<String>,
}

/// 子命令定义
#[derive(Parser, Debug)]
enum Command {
    /// 运行基准测试
    Benchmark {
        /// 查询次数
        #[arg(long, default_value = "100000")]
        query_count: usize,

        /// 并发连接数
        #[arg(long, default_value = "16")]
        connections: usize,

        /// 查询模板
        #[arg(long, default_value = "SELECT * FROM test_table WHERE id = {}")]
        query_template: String,

        /// 服务器URL
        #[arg(long, default_value = "jdbc:remdb://localhost:6666")]
        server_url: String,

        /// 测试类型（query、write或mix）
        #[arg(long, default_value = "query")]
        test_type: String,

        /// 写入模板
        #[arg(
            long,
            default_value = "INSERT INTO test_table (id, value) VALUES ({}, {}) ON DUPLICATE KEY UPDATE value = {}"
        )]
        write_template: String,

        /// 读写比例，格式为"8:2"
        #[arg(long, default_value = "8:2")]
        read_write_ratio: String,
    },
}

#[tokio::main]
async fn main() {
    // 在程序最开始就设置默认的 RUST_LOG 环境变量
    unsafe {
        std::env::set_var("RUST_LOG", "error");
    }

    let args = Args::parse();

    let message = "remdb-server v0.1.0";
    println!("{}", message);

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
                eprintln!("Invalid read_write_ratio format. Expected format: \"8:2\".");
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
            Ok(_) => println!("\nBenchmark completed successfully!"),
            Err(e) => {
                eprintln!("\nBenchmark failed: {}", e);
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
    println!("{}", message);
    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(parsed_config) => {
                config = parsed_config;
                let message = "Config file loaded successfully";
                println!("{}", message);
            }
            Err(err) => {
                let message = format!("Warning: Failed to parse config file: {:?}", err);
                eprintln!("{}", message);
                let message = "Using default config values";
                eprintln!("{}", message);
            }
        },
        Err(err) => {
            let message = format!("Warning: Failed to read config file: {:?}", err);
            eprintln!("{}", message);
            let message = "Using default config values";
            eprintln!("{}", message);
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
        let message = "Debug mode enabled";
        println!("{}", message);
    }

    // 初始化日志文件
    let log_file_path = log_path.clone().unwrap_or("./logs".to_string());
    if let Err(err) = set_log_file(&log_file_path) {
        let message = format!("Warning: Failed to initialize log file: {:?}", err);
        eprintln!("{}", message);
    } else {
        let message = format!("Log file initialized at: {}", log_file_path);
        println!("{}", message);
    }

    // 手动初始化平台
    log_println!("Manually initializing platform...");
    remdb::platform::init_platform(&WINDOWS_PLATFORM);
    log_println!("Platform initialized manually");

    // DDL文件现在通过remdbcli的source指令执行，不再在启动时处理
    let (tables, insert_statements): (Vec<remdb::TableDef>, Vec<String>) = (Vec::new(), Vec::new());

    // 创建默认内存分配器
    static mut DEFAULT_ALLOCATOR: remdb::config::DefaultMemoryAllocator =
        remdb::config::DefaultMemoryAllocator;

    // 首先将tables向量泄漏到静态内存，确保TableDef有'static生命周期
    let static_tables = Box::leak(Box::new(tables));

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
            log_path: Box::leak(log_path.unwrap_or("./wal".to_string()).into_boxed_str())
                as &'static str,
            log_mode: match wal_log_mode.as_deref() {
                Some("sync") | Some("Sync") => remdb::config::LogMode::Sync,
                _ => remdb::config::LogMode::Async,
            },
            checkpoint_interval_ms: wal_checkpoint_interval_ms.unwrap_or(30000),
            log_file_size_limit: wal_log_file_size_limit.unwrap_or(16 * 1024 * 1024),
            log_prealloc_size: wal_log_prealloc_size.unwrap_or(4 * 1024 * 1024),
            log_segment_size: wal_log_segment_size.unwrap_or(16 * 1024 * 1024),
            retained_checkpoints: wal_retained_checkpoints.unwrap_or(3),
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
    }));

    // 初始化全局内存分配器，这是关键的一步！
    let total_memory = config.total_memory;

    // 初始化HA manager（在内存分配器和数据库初始化之前，因为HA可能依赖于特定的初始化顺序）
    if ha_enabled {
        log_println!("Initializing HA manager...");
        match remdb::ha::init(config) {
            Ok(_) => log_println!("✓ HA manager initialized successfully"),
            Err(e) => log_eprintln!("Error: Failed to initialize HA manager: {}", e),
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
        log_eprintln!(
            "Error: Failed to initialize global memory allocator: {:?}",
            err
        );
        return;
    }

    // 使用remdb库提供的init_global_db函数初始化数据库，这个函数会从配置中创建表
    let mut db = match unsafe { remdb::init_global_db(config) } {
        Ok(db) => db,
        Err(err) => {
            log_eprintln!("Error: Failed to initialize global database: {:?}", err);
            return;
        }
    };

    log_println!("Database initialized with {} tables", config.tables.len());

    // 加载全量镜像文件（优先级最高）
    if let Some(full_image_path) = &full_image {
        log_println!("Loading full image file: {}", full_image_path);
        if let Err(err) = db.restore_snapshot(full_image_path) {
            log_eprintln!("Error: Failed to load full image: {:?}", err);
        } else {
            log_println!("Full image loaded successfully");
        }
    } else {
        // 从WAL目录恢复数据（如果配置了WAL）
        let wal_dir = &config.wal_config.log_path;
        log_println!("Checking WAL directory: {}", wal_dir);
        if std::path::Path::new(wal_dir).exists() {
            log_println!("Loading and recovering from WAL directory: {}", wal_dir);
            if let Err(err) = snapshot_loader::load_from_wal_dir(&mut db, wal_dir) {
                log_eprintln!("Warning: Failed to recover from WAL: {:?}", err);
                // 如果WAL恢复失败，尝试从快照目录加载
                if let Some(snapshot_dir) = &snapshot_dir {
                    log_println!("Falling back to snapshot directory: {}", snapshot_dir);
                    if let Err(err) = snapshot_loader::load_snapshot_from_dir(&mut db, snapshot_dir)
                    {
                        log_eprintln!("Warning: Failed to load snapshot: {:?}", err);
                    } else {
                        log_println!("Snapshot loaded successfully");
                    }
                }
            } else {
                log_println!("Data recovered successfully from WAL");
            }
        } else if let Some(snapshot_dir) = &snapshot_dir {
            // 如果没有WAL目录，尝试从快照目录加载
            log_println!("Loading snapshot from directory: {}", snapshot_dir);
            if let Err(err) = snapshot_loader::load_snapshot_from_dir(&mut db, snapshot_dir) {
                log_eprintln!("Warning: Failed to load snapshot: {:?}", err);
            } else {
                log_println!("Snapshot loaded successfully");
            }
        }
    }

    // 执行DDL文件中的INSERT语句
    if !insert_statements.is_empty() {
        log_println!(
            "Executing {} INSERT statements from DDL file",
            insert_statements.len()
        );
        for stmt in insert_statements {
            log_println!("Executing: {}", stmt);
            match sql_engine::execute_extended_sql(&mut db, &stmt) {
                Ok(result) => {
                    log_println!(
                        "✓ INSERT executed successfully, affected rows: {}",
                        result.affected_rows
                    );
                }
                Err(err) => {
                    log_eprintln!("Error: Failed to execute INSERT statement: {}", err);
                    log_eprintln!("Statement: {}", stmt);
                }
            }
        }
    }

    // 测试healthcheck命令
    if args.test_export {
        log_println!("\n=== Testing HEALTHCHECK command ===");
        match sql_engine::execute_extended_sql(&mut db, "healthcheck") {
            Ok(result) => {
                log_println!("\nHealthcheck result:");
                log_println!(
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
                        log_println!("{}", line);
                        log_println!(
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
                    log_println!("{}", line);
                }
                log_println!(
                    "+--------------------+----------+------------------------------------------------------------------+"
                );
            }
            Err(err) => {
                log_eprintln!("Error: Failed to execute healthcheck: {}", err);
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

        log_println!(
            "Starting JDBC server on port {} with max connections {} and timeout {} seconds",
            actual_jdbc_port,
            max_conns,
            jdbc_timeout
        );
        log_println!(
            "JDBC authentication: {}",
            if auth_enabled { "enabled" } else { "disabled" }
        );

        // 在后台启动JDBC服务器
        tokio::spawn(async move {
            if let Err(e) = jdbc_server.start().await {
                log_eprintln!("Error: JDBC server failed to start: {:?}", e);
            }
        });
    } else {
        log_println!("JDBC server is disabled");
    }

    // 添加定时器线程，定期检查是否需要创建checkpoint
    let checkpoint_interval = wal_checkpoint_interval_ms.unwrap_or(30000);
    if checkpoint_interval > 0 {
        log_println!(
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
                            println!(
                                "[Checkpoint Timer] Checkpoint executed successfully in {:?}",
                                duration
                            );
                        }
                        Err(e) => {
                            println!("[Checkpoint Timer] Failed to execute checkpoint: {:?}", e);
                        }
                    }
                } else {
                    println!("[Checkpoint Timer] LogManager not available");
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

                log_println!(
                    "Starting snapshot timer with interval {} seconds, type: {}",
                    interval_secs,
                    snap_type
                );

                // 在后台启动快照定时器
                tokio::spawn(async move {
                    let interval = tokio::time::Duration::from_secs(interval_secs);
                    let mut timer = tokio::time::interval(interval);

                    loop {
                        timer.tick().await;

                        // 尝试获取全局数据库实例
                        let db_opt = unsafe { remdb::get_global_db() };
                        if let Some(mut db_guard) = db_opt {
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
                                    println!(
                                        "[Snapshot Timer] {} snapshot executed successfully in {:?}",
                                        snap_type, duration
                                    );
                                }
                                Err(e) => {
                                    println!(
                                        "[Snapshot Timer] Failed to execute {} snapshot: {:?}",
                                        snap_type, e
                                    );
                                }
                            }
                        } else {
                            println!("[Snapshot Timer] Database not available");
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
            log_eprintln!("Warning: Failed to initialize PubSub system: {:?}", err);
        } else {
            log_println!(
                "PubSub system initialized successfully on port {}",
                pubsub_port
            );
        }
    } else {
        log_println!("PubSub server is disabled");
    }

    // 启动交互式控制台（如果启用且不是非交互式模式）
    if !args.non_interactive {
        // 只在非交互式模式下启动CLI，JDBC模式下不启动CLI
        if !should_start_jdbc {
            let mut db_lock = db_arc.lock().unwrap();
            cli::run_cli(&mut db_lock);
        } else {
            log_println!(
                "\n--- JDBC server is running on port {} ---",
                actual_jdbc_port
            );
            log_println!("Interactive CLI is disabled when JDBC server is running.");
            log_println!("Use --non-interactive=false to enable CLI in non-JDBC mode.");
            log_println!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            log_println!("\nStopping JDBC server...");
        }
    } else {
        // 非交互式模式下，如果启用了JDBC服务，等待Ctrl+C
        if should_start_jdbc {
            log_println!(
                "\n--- JDBC server is running on port {} ---",
                actual_jdbc_port
            );
            log_println!("Press Ctrl+C to stop the server");
            tokio::signal::ctrl_c().await.unwrap();
            log_println!("\nStopping JDBC server...");
        }
    }

    // 程序退出前关闭HA manager
    if ha_enabled {
        log_println!("Stopping HA manager...");
        use remdb::ha::shutdown as ha_shutdown;
        if let Err(err) = ha_shutdown() {
            log_eprintln!("Warning: Failed to shutdown HA manager: {:?}", err);
        }
    }

    // 程序退出前关闭PubSub系统
    log_println!("Stopping PubSub server...");
    use remdb::pubsub::shutdown as pubsub_shutdown;
    if let Err(err) = pubsub_shutdown() {
        log_eprintln!("Warning: Failed to shutdown PubSub server: {:?}", err);
    }
}
