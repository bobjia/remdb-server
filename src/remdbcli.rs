use clap::Parser;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// JDBC server host
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// JDBC server port
    #[arg(short, long, default_value = "54321")]
    port: u16,

    /// Username for authentication
    #[arg(short, long)]
    username: Option<String>,

    /// Password for authentication
    #[arg(short = 'P', long)]
    password: Option<String>,

    /// SQL command to execute
    #[arg(short, long)]
    sql: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // 连接到JDBC服务器
    let addr = format!("{}:{}", cli.host, cli.port);
    let mut stream = match TcpStream::connect(&addr) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to JDBC server at {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    // 处理认证
    if let Some(username) = &cli.username {
        if let Some(password) = &cli.password {
            let auth_command = format!("AUTH|{}|{}", username, password);
            if let Err(e) = writeln!(stream, "{}", auth_command) {
                eprintln!("Failed to send auth command: {}", e);
                std::process::exit(1);
            }
            stream.flush().unwrap();

            let mut response = String::new();
            let mut reader = BufReader::new(&mut stream);
            if let Err(e) = reader.read_line(&mut response) {
                eprintln!("Failed to read auth response: {}", e);
                std::process::exit(1);
            }

            if response.starts_with("ERROR|") {
                eprintln!("Authentication failed: {}", &response[6..].trim());
                std::process::exit(1);
            }
        }
    }

    // 如果提供了sql参数，执行单次命令并退出
    if let Some(sql) = cli.sql {
        execute_sql(&mut stream, &sql);
        return;
    }

    // 否则进入交互式模式
    println!("Connected to JDBC server at {}", addr);
    println!("Type 'exit' or 'quit' to exit");
    println!("Type 'help' for available commands");
    println!("{}", "=".repeat(60));

    let stdin = io::stdin();
    loop {
        print!("remdbcli> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let command = line.trim().to_lowercase();
        if command.is_empty() {
            continue;
        }

        if command == "exit" || command == "quit" {
            break;
        }

        if command == "help" {
            print_help();
            continue;
        }

        execute_sql(&mut stream, &line.trim());
    }

    // 关闭连接
    writeln!(stream, "CLOSE").unwrap();
    stream.flush().unwrap();
}

fn execute_sql(stream: &mut TcpStream, sql: &str) {
    // 构建EXECUTE命令
    let execute_command = format!("EXECUTE|{}", sql);

    // 发送命令
    if let Err(e) = writeln!(stream, "{}", execute_command) {
        eprintln!("Failed to send SQL command: {}", e);
        return;
    }
    stream.flush().unwrap();

    // 读取响应
    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    if let Err(e) = reader.read_line(&mut response) {
        eprintln!("Failed to read SQL response: {}", e);
        return;
    }

    // 处理响应
    process_response(&response);
}

fn process_response(response: &str) {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return;
    }

    if trimmed.starts_with("ERROR|") {
        eprintln!("Error: {}", &trimmed[6..]);
        return;
    }

    if trimmed.starts_with("OK|") {
        let parts: Vec<&str> = trimmed[3..].split('|').collect();
        if parts.len() < 1 {
            eprintln!("Invalid OK response format");
            return;
        }

        let affected_rows = parts[0].parse::<usize>().unwrap_or(0);

        if parts.len() == 1 || parts[1] == "0" {
            // 没有结果集，只显示受影响的行数
            println!("Affected {} row(s)", affected_rows);
            return;
        }

        // 有结果集
        if parts.len() >= 4 {
            let columns_count = parts[1].parse::<usize>().unwrap_or(0);
            let columns = parts[2].split(',').collect::<Vec<&str>>();
            let rows_str = parts[3];

            // 检查列数是否匹配
            if columns.len() != columns_count {
                eprintln!(
                    "Column count mismatch: expected {}, got {}",
                    columns_count,
                    columns.len()
                );
                return;
            }

            let rows = if rows_str.is_empty() {
                Vec::new()
            } else {
                rows_str
                    .split(';')
                    .map(|row| row.split(',').collect::<Vec<&str>>())
                    .collect::<Vec<Vec<&str>>>()
            };

            // 格式化输出结果集
            format_result_set(columns, rows);
            return;
        }
    }

    eprintln!("Unknown response format: {}", trimmed);
}

fn format_result_set(columns: Vec<&str>, rows: Vec<Vec<&str>>) {
    if columns.is_empty() {
        return;
    }

    // 计算每列的最大宽度
    let mut col_widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() && cell.len() > col_widths[i] {
                col_widths[i] = cell.len();
            }
        }
    }

    // 构建分隔线
    let separator: String = col_widths
        .iter()
        .map(|w| format!("+{}+", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("")
        .trim_end()
        .to_string();

    // 构建表头
    let mut output = String::new();
    output.push_str(&separator);
    output.push_str("\n");

    for (i, col) in columns.iter().enumerate() {
        if i < col_widths.len() {
            output.push_str(&format!("| {:<width$} ", col, width = col_widths[i]));
        }
    }
    output.push_str("|");
    output.push_str("\n");

    output.push_str(&separator);
    output.push_str("\n");

    // 构建行
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                output.push_str(&format!("| {:<width$} ", cell, width = col_widths[i]));
            }
        }
        output.push_str("|");
        output.push_str("\n");
    }

    output.push_str(&separator);
    output.push_str("\n");

    println!("{}", output);
}

fn print_help() {
    println!("Available commands:");
    println!("  exit, quit                     - Exit the console");
    println!("  help                           - Show this help message");
    println!("  tables                         - List all tables");
    println!("  describe <table>               - Show table schema");
    println!("  desc <table>                   - Shortcut for describe");
    println!("  select ...                     - Execute SELECT query");
    println!("  insert ...                     - Execute INSERT statement");
    println!("  update ...                     - Execute UPDATE statement");
    println!("  delete ...                     - Execute DELETE statement");
    println!("  create table ...               - Create a new table");
    println!("  create index ...               - Create a new index");
}
