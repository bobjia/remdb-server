mod ddl_compiler;
mod snapshot_loader;
mod sql_engine;
mod cli;

use clap::Parser;
use crate::ddl_compiler::{compile_ddl_file, init_database};
use crate::snapshot_loader::load_snapshot_from_dir;
use crate::cli::run_cli;
use crate::sql_engine::{execute_extended_sql, format_result_set};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write, Seek};
use std::time::SystemTime;
use core::ptr;

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
    
    /// 非交互式模式（初始化后退出）
    #[arg(long)]
    non_interactive: bool,
}

fn main() {
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
    let total_memory = args.total_memory.or(config.total_memory);
    let default_max_records = args.default_max_records.or(config.default_max_records);
    let low_power_mode_supported = args.low_power_mode_supported.or(config.low_power_mode_supported);
    let low_power_max_records = args.low_power_max_records.or(config.low_power_max_records);
    let snapshot_interval = args.snapshot_interval.or(config.snapshot_interval);
    let max_incremental_snapshots = args.max_incremental_snapshots.or(config.max_incremental_snapshots);
    
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
                println!("Debug: Compiled {} tables:", tables.len());
                for table in &tables {
                    println!("Debug: - Table: {}", table.name);
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
    
    // 使用非常小的默认最大记录数，避免内存不足
    let small_max_records = 1; // 仅使用1条记录，最小化内存使用
    
    // 首先将tables向量泄漏到静态内存，确保TableDef有'static生命周期
    let static_tables = Box::leak(Box::new(tables));
    
    // 创建配置
    let config = Box::leak(Box::new(remdb::config::DbConfig {
        tables: static_tables,
        total_memory: total_memory.unwrap_or(1024 * 1024 * 100), // 默认100MB
        default_max_records: small_max_records, // 使用非常小的默认值，避免内存不足
        low_power_mode_supported: low_power_mode_supported.unwrap_or(true), // 默认支持低功耗模式
        low_power_max_records: Some(low_power_max_records.unwrap_or(100)), // 默认100条记录
        memory_allocator: unsafe {
            &*(&raw const DEFAULT_ALLOCATOR as *const _) as &'static dyn remdb::config::MemoryAllocator
        },
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
    let db = match unsafe {
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
        if let Err(err) = snapshot_loader::load_snapshot_from_dir(db, snapshot_dir) {
            eprintln!("Warning: Failed to load snapshot: {:?}", err);
        } else {
            println!("Snapshot loaded successfully");
        }
    }
    
    // 启动交互式控制台
    if !args.non_interactive {
        cli::run_cli(db);
    } else {
        // 测试：直接执行tables命令查看表列表
        println!("\n--- Testing database tables ---");
        let tables_result = sql_engine::execute_extended_sql(db, "tables");
        match tables_result {
            Ok(result) => {
                println!("Tables command output:");
                println!("{}", sql_engine::format_result_set(&result));
            },
            Err(err) => {
                eprintln!("Error executing tables command: {:?}", err);
            }
        }
        
        // 测试：尝试执行简单的SELECT查询
        println!("\n--- Testing SELECT query ---");
        let select_result = sql_engine::execute_extended_sql(db, "SELECT * FROM users");
        match select_result {
            Ok(result) => {
                println!("SELECT command output:");
                println!("{}", sql_engine::format_result_set(&result));
            },
            Err(err) => {
                eprintln!("Error executing SELECT query: {:?}", err);
            }
        }
        
        println!("\n✓ Database initialized successfully in non-interactive mode");
    }
}