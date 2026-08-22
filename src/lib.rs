//! RemDB Server - 高性能内存数据库服务器
//!
//! 本 crate 提供了 RemDB 数据库的服务器端实现，包括：
//! - JDBC 协议服务器
//! - 连接池管理
//! - SQL 执行引擎
//! - 快照和恢复机制
//! - 定时任务调度

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

pub mod bootstrap;
pub mod cli;
pub mod config;
pub mod context;
mod ddl_compiler;
pub mod error;
pub mod handler;
pub mod jdbc_server;
pub mod milvus;
pub mod network;
pub mod pool;
pub mod proto;
pub mod scheduler;
mod snapshot_loader;
pub mod sql_engine;
pub mod tuning;

pub use context::{AppContext, AppContextBuilder};
pub use error::{ServerError, ServerResult};
