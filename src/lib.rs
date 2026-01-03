mod cli;
mod ddl_compiler;
pub mod jdbc_server;
mod snapshot_loader;
mod sql_engine;

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
