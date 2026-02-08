use crate::snapshot_loader::{
    cleanup_old_snapshots, save_full_snapshot_to_dir, save_incremental_snapshot_to_dir,
};
use crate::sql_engine::{execute_extended_sql, format_result_set};
use remdb::RemDb;
use remdb::log::{info, error};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{Config, Editor};
use std::env;

/// 运行交互式命令行界面
pub fn run_cli(db: &mut RemDb) {
    info!("Interactive console started");
    info!("Type 'help' for available commands");
    info!("Type 'exit' to quit");
    info!("{}", "=".repeat(60));

    // 初始化快照目录（默认使用当前目录下的snapshots文件夹）
    let snapshot_dir = env::var("REMDB_SNAPSHOT_DIR").unwrap_or_else(|_| "snapshots".to_string());
    info!("Snapshot directory: {}", snapshot_dir);

    let config = Config::builder()
        .history_ignore_space(true)
        .auto_add_history(true)
        .build();

    let mut editor: Editor<(), FileHistory> = match Editor::with_config(config) {
        Ok(editor) => editor,
        Err(err) => {
            error!("Error: Failed to create editor: {:?}", err);
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

                // 处理source命令，用于执行文件中的SQL/DDL语句
                if line.starts_with("source ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() != 2 {
                        error!("Error: Invalid source command. Use 'source <file_path>'.");
                        continue;
                    }

                    let file_path = parts[1];
                    match std::fs::read_to_string(file_path) {
                        Ok(content) => {
                            info!("Executing commands from file: {}", file_path);
                            // 执行文件内容
                            match execute_extended_sql(db, &content) {
                                Ok(result_set) => {
                                    let formatted = format_result_set(&result_set);
                                    info!("{}", formatted);
                                    info!(
                                        "✓ Successfully executed commands from file: {}",
                                        file_path
                                    );
                                }
                                Err(err) => {
                                    error!(
                                        "Error: Failed to execute commands from file {}: {:?}",
                                        file_path, err
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            error!("Error: Failed to read file {}: {:?}", file_path, err);
                        }
                    }
                    continue;
                }

                // 处理snapshot命令
                if line.starts_with("snapshot ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 2 {
                        error!(
                            "Error: Invalid snapshot command. Use 'snapshot full' or 'snapshot incremental'."
                        );
                        continue;
                    }

                    match parts[1] {
                        "full" => {
                            // 保存完整快照
                            if let Err(err) = save_full_snapshot_to_dir(db, &snapshot_dir) {
                                error!("Error: Failed to save full snapshot: {:?}", err);
                            } else {
                                info!("Full snapshot saved successfully");
                            }
                        }
                        "incremental" => {
                            // 保存增量快照
                            if let Err(err) = save_incremental_snapshot_to_dir(db, &snapshot_dir) {
                                error!("Error: Failed to save incremental snapshot: {:?}", err);
                            } else {
                                info!("Incremental snapshot saved successfully");
                                // 清理旧快照，保留最新10个
                                let _ = cleanup_old_snapshots(&snapshot_dir, 10);
                            }
                        }
                        _ => {
                            error!(
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
                        info!("{}", formatted);
                    }
                    Err(err) => {
                        error!("Error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                info!("^C");
            }
            Err(ReadlineError::Eof) => {
                info!("\nBye");
                // 保存历史记录
                let _ = editor.save_history("remdb_history.txt");
                break;
            }
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        }
    }
}

/// 打印帮助信息
pub fn print_help() {
    info!("Available commands:");
    info!("  exit, :q                        - Exit the console");
    info!("  help                            - Show this help message");
    info!("  source <file_path>              - Execute SQL/DDL commands from file");
    info!("  tables                          - List all tables");
    info!("  describe <table>                - Show table schema");
    info!("  desc <table>                    - Shortcut for describe");
    info!("  select ...                      - Execute SELECT query");
    info!("  create database <name>          - Create a new database");
    info!("  drop database [if exists] <name> - Drop an existing database");
    info!("  use database <name>             - Switch to a specified database");
    info!("  close database <name>           - Close a specified database");
    info!("  snapshot full                   - Save a full snapshot");
    info!("  snapshot incremental            - Save an incremental snapshot");
    info!("  stat                            - Show database monitoring statistics");
    info!("  healthcheck                     - Check database health status");
    info!("  export ddl <file>               - Export DDL schema to file");
    info!("  export data <table> <file>      - Export table data to CSV file");
    info!("  export all <dir>                - Export both DDL and data to directory");
}
