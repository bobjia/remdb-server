use clap::Parser;
use ctrlc;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

// 添加protobuf相关导入
use prost::Message;
use remdb_server::proto::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// JDBC server host
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// JDBC server port
    #[arg(short, long, default_value = "6666")]
    port: u16,

    /// Username for authentication
    #[arg(short, long, default_value = "root")]
    username: String,

    /// Password for authentication
    #[arg(short = 'P', long)]
    password: Option<String>,

    /// SQL command to execute
    #[arg(short, long)]
    sql: Option<String>,
}

fn main() {
    // 创建退出标志
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();

    // 设置Ctrl+C处理
    ctrlc::set_handler(move || {
        should_exit_clone.store(true, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    let cli = Cli::parse();

    // 连接到JDBC服务器
    let addr = format!("{}:{}", cli.host, cli.port);
    let socket_addr = addr.parse().expect("Invalid address format");
    let mut stream = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
        Ok(stream) => {
            // 设置读写超时
            if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(10))) {
                eprintln!("Failed to set read timeout: {}", e);
                std::process::exit(1);
            }
            if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(5))) {
                eprintln!("Failed to set write timeout: {}", e);
                std::process::exit(1);
            }

            // 禁用Nagle算法，减少延迟
            stream.set_nodelay(true).unwrap();

            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to JDBC server at {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    // 初始化request_id计数器
    let mut request_id = 1;

    // 处理认证
    let connection_request = ConnectionRequest {
        username: cli.username.clone(),
        password: cli.password.clone().unwrap_or_default(),
        database: "default".to_string(),
        fetch_size: 100,
        auto_commit: true,
    };

    let jdbc_request = JdbcRequest {
        request_id,
        request: Some(jdbc_request::Request::Connection(connection_request)),
    };

    println!("Sending connection request...");
    if let Err(e) = send_jdbc_request(&mut stream, &jdbc_request) {
        eprintln!("Failed to send connection request: {}", e);
        std::process::exit(1);
    }

    println!("Reading connection response...");
    match read_jdbc_response(&mut stream) {
        Ok(response) => {
            println!("Connection response received, status: {}", response.status);
            if response.status != 0 {
                // 0 是 Status::OK 的值
                eprintln!("Authentication failed: {}", response.error_message);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to read connection response: {}", e);
            std::process::exit(1);
        }
    }

    request_id += 1;

    // 如果提供了sql参数，执行单次命令并退出
    if let Some(sql) = cli.sql {
        execute_sql(&mut stream, &sql, &mut request_id);
        return;
    }

    // 否则进入交互式模式
    println!("Connected to JDBC server at {}", addr);
    println!("Type 'exit' or 'quit' to exit");
    println!("Type 'help' for available commands");
    println!("{}", "=".repeat(60));

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        // 检查退出标志
        if should_exit.load(Ordering::SeqCst) {
            println!("\nReceived interrupt signal, exiting...");
            break;
        }

        print!("remdbcli> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        let result = reader.read_line(&mut line);

        match result {
            Ok(0) => {
                // EOF encountered, exit gracefully
                println!("\nEOF encountered, exiting...");
                break;
            }
            Ok(_) => {
                let original_line = line.trim();
                let command = original_line.to_lowercase();
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

                // 处理source命令，用于执行文件中的SQL/DDL语句
                if command.starts_with("source ") {
                    let parts: Vec<&str> = original_line.split_whitespace().collect();
                    if parts.len() != 2 {
                        eprintln!("Error: Invalid source command. Use 'source <file_path>'.");
                        continue;
                    }

                    let file_path = parts[1];
                    match std::fs::read_to_string(file_path) {
                        Ok(content) => {
                            println!("Executing commands from file: {}", file_path);

                            // 按行处理文件内容
                            let mut current_statement = String::new();
                            for line in content.lines() {
                                let trimmed_line = line.trim();

                                // 跳过空行和注释行
                                if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
                                    continue;
                                }

                                current_statement.push_str(trimmed_line);
                                current_statement.push(' ');

                                // 如果语句以分号结束，执行它
                                if trimmed_line.ends_with(';') {
                                    // 移除分号和多余空格
                                    let statement = current_statement.trim_end_matches(';').trim();
                                    if !statement.is_empty() {
                                        // 执行单个语句
                                        execute_sql(&mut stream, statement, &mut request_id);
                                    }
                                    // 重置当前语句
                                    current_statement.clear();
                                }
                            }

                            // 执行最后一个没有分号的语句
                            let statement = current_statement.trim();
                            if !statement.is_empty() {
                                execute_sql(&mut stream, statement, &mut request_id);
                            }

                            println!("✓ Successfully executed commands from file: {}", file_path);
                        }
                        Err(err) => {
                            eprintln!("Error: Failed to read file {}: {:?}", file_path, err);
                        }
                    }
                    continue;
                }

                execute_sql(&mut stream, original_line, &mut request_id);
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // 处理Ctrl+C信号，退出循环
                println!("\nReceived interrupt signal, exiting...");
                break;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    // 关闭连接 - 暂时简化，后续完善
    // if let Err(e) = writeln!(stream, "CLOSE") {
    //     eprintln!("Failed to send close command: {}", e);
    // } else if let Err(e) = stream.flush() {
    //     eprintln!("Failed to flush close command: {}", e);
    // }
}

// 发送JDBC请求的辅助函数
fn send_jdbc_request(stream: &mut TcpStream, request: &JdbcRequest) -> std::io::Result<()> {
    // 序列化请求
    let mut buf = Vec::with_capacity(request.encoded_len());
    request.encode(&mut buf)?;

    // 发送请求长度（4字节大端）
    let len = buf.len() as u32;
    stream.write_all(&len.to_be_bytes())?;

    // 发送请求数据
    stream.write_all(&buf)?;
    stream.flush()?;

    Ok(())
}

// 读取JDBC响应的辅助函数
fn read_jdbc_response(stream: &mut TcpStream) -> std::io::Result<JdbcResponse> {
    // 读取响应长度（4字节大端）
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // 读取响应数据
    let mut data_buf = vec![0u8; len];
    stream.read_exact(&mut data_buf)?;

    // 反序列化响应
    let response = JdbcResponse::decode(&*data_buf)?;

    Ok(response)
}

fn execute_sql(stream: &mut TcpStream, sql: &str, request_id: &mut u64) {
    // 创建查询请求
    let query_request = QueryRequest {
        sql: sql.to_string(),
        parameters: Vec::new(),
        fetch_size: 100,
        use_cursor: false,
    };

    // 创建JDBC请求
    let jdbc_request = JdbcRequest {
        request_id: *request_id,
        request: Some(jdbc_request::Request::Query(query_request)),
    };

    // 发送请求
    if let Err(e) = send_jdbc_request(stream, &jdbc_request) {
        eprintln!("Failed to send SQL command: {}", e);
        return;
    }

    // 读取响应
    match read_jdbc_response(stream) {
        Ok(response) => {
            process_jdbc_response(&response);
        }
        Err(e) => {
            eprintln!("Failed to read SQL response: {}", e);
            return;
        }
    }

    // 更新request_id
    *request_id += 1;
}

fn process_jdbc_response(response: &JdbcResponse) {
    // 0 是 Status::OK 的值
    if response.status == 0 {
        // 处理成功响应
        match &response.response {
            Some(jdbc_response::Response::ResultSet(result_set)) => {
                // 处理结果集
                println!("Debug: ResultSet columns: {:?}", result_set.columns);
                println!("Debug: ResultSet rows: {:?}", result_set.rows);
                println!("Debug: ResultSet row count: {}", result_set.row_count);
                format_result_set_proto(result_set);
            }
            Some(jdbc_response::Response::Update(update)) => {
                // 处理更新响应
                println!("Affected {} row(s)", update.affected_rows);
                if update.last_insert_id > 0 {
                    println!("Last insert ID: {}", update.last_insert_id);
                }
            }
            Some(jdbc_response::Response::Transaction(_)) => {
                // 处理事务响应
                println!("Transaction completed successfully");
            }
            _ => {
                // 其他响应类型
                println!("Command executed successfully");
            }
        }
    } else {
        // 处理错误响应
        eprintln!("Error: {}", response.error_message);
    }
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

// 处理值类型，转换为字符串
fn value_to_string(value: &Value) -> String {
    match &value.value {
        Some(value::Value::BooleanValue(v)) => v.to_string(),
        Some(value::Value::Int32Value(v)) => v.to_string(),
        Some(value::Value::Int64Value(v)) => v.to_string(),
        Some(value::Value::FloatValue(v)) => v.to_string(),
        Some(value::Value::DoubleValue(v)) => v.to_string(),
        Some(value::Value::StringValue(v)) => v.clone(),
        Some(value::Value::BytesValue(v)) => format!("bytes[{:?}]", v),
        Some(value::Value::Uint64Value(v)) => v.to_string(),
        Some(value::Value::Sint64Value(v)) => v.to_string(),
        Some(value::Value::Fixed32Value(v)) => v.to_string(),
        Some(value::Value::Fixed64Value(v)) => v.to_string(),
        Some(value::Value::Sfixed32Value(v)) => v.to_string(),
        Some(value::Value::Sfixed64Value(v)) => v.to_string(),
        Some(value::Value::DateValue(v)) => format!("date[{:?}]", v),
        Some(value::Value::TimeValue(v)) => format!("time[{:?}]", v),
        Some(value::Value::TimestampValue(v)) => format!("timestamp[{:?}]", v),
        Some(value::Value::NullValue(_)) => "NULL".to_string(),
        None => "NULL".to_string(),
    }
}

// 处理protobuf格式的结果集
fn format_result_set_proto(result_set: &ResultSetResponse) {
    if result_set.columns.is_empty() {
        println!("Empty result set");
        return;
    }

    // 获取列名
    let columns: Vec<&str> = result_set.columns.iter().map(|c| &*c.name).collect();

    // 转换行数据为字符串格式
    let rows: Vec<Vec<String>> = result_set
        .rows
        .iter()
        .map(|row| {
            row.values
                .iter()
                .map(|value| value_to_string(value))
                .collect()
        })
        .collect();

    // 如果没有行数据，只显示列名
    if rows.is_empty() {
        format_empty_result_set(columns);
        return;
    }

    // 计算每列的最大宽度
    let mut col_widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            let cell_str: &String = cell;
            if i < col_widths.len() && cell_str.len() > col_widths[i] {
                col_widths[i] = cell_str.len();
            }
        }
    }

    // 构建分隔线
    let separator: String = col_widths
        .iter()
        .map(|w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("")
        + "|";

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
            let cell_str: &String = cell;
            if i < col_widths.len() {
                output.push_str(&format!("| {:<width$} ", cell_str, width = col_widths[i]));
            }
        }
        output.push_str("|");
        output.push_str("\n");
    }

    output.push_str(&separator);
    output.push_str("\n");
    output.push_str(&format!("{} rows in set\n", result_set.row_count));

    println!("{}", output);
}

// 处理空结果集的情况
fn format_empty_result_set(columns: Vec<&str>) {
    if columns.is_empty() {
        println!("Empty result set");
        return;
    }

    // 计算每列的最大宽度
    let col_widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();

    // 构建分隔线
    let separator: String = col_widths
        .iter()
        .map(|w| format!("+{}", "-".repeat(w + 2)))
        .collect::<Vec<_>>()
        .join("")
        + "|";

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
    output.push_str(&format!("0 rows in set\n"));

    println!("{}", output);
}

fn print_help() {
    println!("Available commands:");
    println!("  exit, quit                     - Exit the console");
    println!("  help                           - Show this help message");
    println!("  source <file_path>             - Execute SQL/DDL commands from file");
    println!("  tables                         - List all tables");
    println!("  describe <table>               - Show table schema");
    println!("  desc <table>                   - Shortcut for describe");
    println!("  select ...                     - Execute SELECT query");
    println!("  insert ...                     - Execute INSERT statement");
    println!("  update ...                     - Execute UPDATE statement");
    println!("  delete ...                     - Execute DELETE statement");
    println!("  create table ...               - Create a new table");
    println!("  create index ...               - Create a new index");
    println!("  stat                           - Show database monitoring statistics");
    println!("  healthcheck                    - Check database health status");
    println!("  begin, begin transaction       - Start a new transaction");
    println!("  commit                         - Commit current transaction");
    println!("  rollback                       - Rollback current transaction");
    println!("  flush                          - Flush WAL logs to disk");
}
