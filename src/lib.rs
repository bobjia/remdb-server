// 全局debug模式开关
static DEBUG_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// 全局日志文件句柄
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
static mut GLOBAL_LOG_HANDLE: Option<Arc<Mutex<File>>> = None;

/// 设置debug模式
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// 检查是否开启了debug模式
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// 设置全局日志文件句柄
pub fn set_global_log_handle(handle: Arc<Mutex<File>>) {
    unsafe {
        GLOBAL_LOG_HANDLE = Some(handle);
    }
}

/// 写入日志到文件
pub fn write_log_to_file(message: &str) {
    unsafe {
        if let Some(ref file_handle) = GLOBAL_LOG_HANDLE {
            if let Ok(mut file) = file_handle.lock() {
                let _ = writeln!(file, "{}", message);
            }
        }
    }
}

#[macro_use]
mod macros;
#[macro_use]
pub mod cli;
mod ddl_compiler;
pub mod handler;
pub mod jdbc_server;
pub mod network;
pub mod pool;
pub mod proto;
mod snapshot_loader;
mod sql_engine;
pub mod tuning;
