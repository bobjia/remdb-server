use crate::snapshot_loader::{
    cleanup_old_snapshots, save_full_snapshot_to_dir, save_incremental_snapshot_to_dir,
};
use crate::sql_engine::{execute_extended_sql, format_result_set};
use remdb::RemDb;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Config, Editor};
use std::env;
use std::time::SystemTime;

/// 运行交互式命令行界面
pub fn run_cli(db: &mut RemDb) {
    println!("Interactive console started");
    println!("Type 'help' for available commands");
    println!("Type 'exit' to quit");
    println!("{}", "=".repeat(60));

    // 初始化快照目录（默认使用当前目录下的snapshots文件夹）
    let snapshot_dir = env::var("REMDB_SNAPSHOT_DIR").unwrap_or_else(|_| "snapshots".to_string());
    println!("Snapshot directory: {}", snapshot_dir);

    let config = Config::builder()
        .history_ignore_space(true)
        .auto_add_history(true)
        .build();

    let mut editor: Editor<(), FileHistory> = match Editor::with_config(config) {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("Error: Failed to create editor: {:?}", err);
            return;
        }
    };

    // 尝试加载历史记录
    let _ = editor.load_history("remdb_history.txt");

    loop {
        let prompt = "remdb> ";
        match editor.readline(prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case(":q") {
                    // 保存历史记录
                    let _ = editor.save_history("remdb_history.txt");
                    break;
                }

                if line.eq_ignore_ascii_case("help") {
                    print_help();
                    continue;
                }

                // 处理snapshot命令
                if line.starts_with("snapshot ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        eprintln!(
                            "Error: Invalid snapshot command. Use 'snapshot full' or 'snapshot incremental'."
                        );
                        continue;
                    }

                    match parts[1] {
                        "full" => {
                            // 保存完整快照
                            if let Err(err) = save_full_snapshot_to_dir(db, &snapshot_dir) {
                                eprintln!("Error: Failed to save full snapshot: {:?}", err);
                            } else {
                                println!("Full snapshot saved successfully");
                            }
                        }
                        "incremental" => {
                            // 保存增量快照
                            if let Err(err) = save_incremental_snapshot_to_dir(db, &snapshot_dir) {
                                eprintln!("Error: Failed to save incremental snapshot: {:?}", err);
                            } else {
                                println!("Incremental snapshot saved successfully");
                                // 清理旧快照，保留最新10个
                                let _ = cleanup_old_snapshots(&snapshot_dir, 10);
                            }
                        }
                        _ => {
                            eprintln!(
                                "Error: Invalid snapshot command. Use 'snapshot full' or 'snapshot incremental'."
                            );
                        }
                    }
                    continue;
                }

                // 执行SQL命令
                match execute_extended_sql(db, line) {
                    Ok(result_set) => {
                        let formatted = format_result_set(&result_set);
                        println!("{}", formatted);
                    }
                    Err(err) => {
                        eprintln!("Error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("\nBye");
                // 保存历史记录
                let _ = editor.save_history("remdb_history.txt");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

/// 打印帮助信息
pub fn print_help() {
    println!("Available commands:");
    println!("  exit, :q                        - Exit the console");
    println!("  help                            - Show this help message");
    println!("  tables                          - List all tables");
    println!("  describe <table>                - Show table schema");
    println!("  desc <table>                    - Shortcut for describe");
    println!("  select ...                      - Execute SELECT query");
    println!("  snapshot full                   - Save a full snapshot");
    println!("  snapshot incremental            - Save an incremental snapshot");
    println!("  stat                            - Show database monitoring statistics");
    println!("  healthcheck                     - Check database health status");
    println!("  export ddl <file>               - Export DDL schema to file");
    println!("  export data <table> <file>      - Export table data to CSV file");
    println!("  export all <dir>                - Export both DDL and data to directory");
}
