// 全局debug模式开关
static DEBUG_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 设置debug模式
pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// 检查是否开启了debug模式
pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(std::sync::atomic::Ordering::Relaxed)
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
