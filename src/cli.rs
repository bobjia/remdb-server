use rustyline::{Config, Editor};
use rustyline::history::FileHistory;
use rustyline::error::ReadlineError;
use remdb::RemDb;
use crate::sql_engine::{execute_extended_sql, format_result_set};

/// 运行交互式命令行界面
pub fn run_cli(db: &mut RemDb) {
    println!("Interactive console started");
    println!("Type 'help' for available commands");
    println!("Type 'exit' to quit");
    println!("{}", "=".repeat(60));
    
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
    println!("  exit, :q          - Exit the console");
    println!("  help              - Show this help message");
    println!("  tables            - List all tables");
    println!("  describe <table>  - Show table schema");
    println!("  desc <table>      - Shortcut for describe");
    println!("  select ...        - Execute SELECT query");
}
