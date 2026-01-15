/// 调试日志宏，只有在debug模式下才输出
macro_rules! debug_println {
    ($($args:tt)*) => {
        if crate::is_debug_mode() {
            let message = format!($($args)*);
            println!("{}", message);
            crate::write_log_to_file(&message);
        }
    };
}

/// 调试错误日志宏，只有在debug模式下才输出
macro_rules! debug_eprintln {
    ($($args:tt)*) => {
        if crate::is_debug_mode() {
            let message = format!($($args)*);
            eprintln!("{}", message);
            crate::write_log_to_file(&message);
        }
    };
}