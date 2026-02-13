use clap::Parser;
use ctrlc;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use remdb::log::info;

use prost::Message;
use remdb_server::proto::*;

struct ConnectionManager {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    stream: Option<TcpStream>,
    max_retries: u32,
    retry_delay: Duration,
}

impl ConnectionManager {
    fn new(host: String, port: u16, username: String, password: Option<String>) -> Self {
        ConnectionManager {
            host,
            port,
            username,
            password,
            stream: None,
            max_retries: 5,
            retry_delay: Duration::from_secs(2),
        }
    }

    fn connect(&mut self) -> std::io::Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let socket_addr = addr.parse().expect("Invalid address format");

        let mut retry_count = 0;
        loop {
            match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5)) {
                Ok(mut stream) => {
                    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(10))) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to set read timeout: {}", e),
                        ));
                    }
                    if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(5))) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to set write timeout: {}", e),
                        ));
                    }
                    stream.set_nodelay(true).unwrap();

                    self.stream = Some(stream);
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= self.max_retries {
                        return Err(e);
                    }
                    info!(
                        "Connection failed (attempt {}/{}), retrying in {:?}...",
                        retry_count, self.max_retries, self.retry_delay
                    );
                    std::thread::sleep(self.retry_delay);
                }
            }
        }
    }

    fn authenticate(&mut self) -> std::io::Result<()> {
        let connection_request = ConnectionRequest {
            username: self.username.clone(),
            password: self.password.clone().unwrap_or_default(),
            database: "default".to_string(),
            fetch_size: 100,
            auto_commit: true,
        };

        let jdbc_request = JdbcRequest {
            request_id: 1,
            request: Some(jdbc_request::Request::Connection(connection_request)),
        };

        self.send_jdbc_request(&jdbc_request)?;

        let response = self.read_jdbc_response()?;
        if response.status != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Authentication failed: {}", response.error_message),
            ));
        }

        Ok(())
    }

    fn send_jdbc_request(&mut self, request: &JdbcRequest) -> std::io::Result<()> {
        let mut retry_count = 0;
        loop {
            match self.try_send_jdbc_request(request) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if self.is_connection_error(&e) {
                        retry_count += 1;
                        if retry_count >= self.max_retries {
                            return Err(e);
                        }
                        info!(
                            "Connection lost, attempting to reconnect (attempt {}/{})...",
                            retry_count, self.max_retries
                        );
                        self.reconnect()?;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    fn try_send_jdbc_request(&mut self, request: &JdbcRequest) -> std::io::Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "No active connection")
        })?;

        let mut buf = Vec::with_capacity(request.encoded_len());
        request.encode(&mut buf)?;

        let len = buf.len() as u32;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&buf)?;
        stream.flush()?;

        Ok(())
    }

    fn read_jdbc_response(&mut self) -> std::io::Result<JdbcResponse> {
        let mut retry_count = 0;
        loop {
            match self.try_read_jdbc_response() {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if self.is_connection_error(&e) {
                        retry_count += 1;
                        if retry_count >= self.max_retries {
                            return Err(e);
                        }
                        info!(
                            "Connection lost, attempting to reconnect (attempt {}/{})...",
                            retry_count, self.max_retries
                        );
                        self.reconnect()?;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    fn try_read_jdbc_response(&mut self) -> std::io::Result<JdbcResponse> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "No active connection")
        })?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut data_buf = vec![0u8; len];
        stream.read_exact(&mut data_buf)?;

        let response = JdbcResponse::decode(&*data_buf)?;
        Ok(response)
    }

    fn reconnect(&mut self) -> std::io::Result<()> {
        info!("Reconnecting to server {}:{}...", self.host, self.port);
        self.stream = None;
        self.connect()?;
        self.authenticate()?;
        info!("Successfully reconnected to server");
        Ok(())
    }

    fn is_connection_error(&self, e: &std::io::Error) -> bool {
        matches!(
            e.kind(),
            std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::NotConnected
        )
    }
}

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
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();

    ctrlc::set_handler(move || {
        should_exit_clone.store(true, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    let cli = Cli::parse();

    let addr = format!("{}:{}", cli.host, cli.port);

    let mut conn_manager = ConnectionManager::new(
        cli.host.clone(),
        cli.port,
        cli.username.clone(),
        cli.password.clone(),
    );

    info!("Connecting to JDBC server at {}...", addr);
    if let Err(e) = conn_manager.connect() {
        eprintln!("Failed to connect to JDBC server at {}: {}", addr, e);
        std::process::exit(1);
    }

    info!("Authenticating...");
    if let Err(e) = conn_manager.authenticate() {
        eprintln!("Authentication failed: {}", e);
        std::process::exit(1);
    }

    let mut request_id = 2;

    let mut current_database: Option<String> = None;

    if let Some(sql) = cli.sql {
        execute_sql(&mut conn_manager, &sql, &mut request_id, &current_database);
        return;
    }

    println!("Connected to JDBC server at {}", addr);
    println!("Type 'exit' or 'quit' to exit");
    println!("Type 'help' for available commands");
    println!("{}", "=".repeat(60));

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    loop {
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

                if command.starts_with("use database ") {
                    let parts: Vec<&str> = original_line.split_whitespace().collect();
                    if parts.len() != 3 {
                        eprintln!(
                            "Error: Invalid use database command. Use 'use database <database_name>'."
                        );
                        continue;
                    }

                    let database_name = parts[2];
                    current_database = Some(database_name.to_string());
                    println!("✓ Database changed to: {}", database_name);
                    continue;
                }

                if command.starts_with("close database ") {
                    let parts: Vec<&str> = original_line.split_whitespace().collect();
                    if parts.len() != 3 {
                        eprintln!(
                            "Error: Invalid close database command. Use 'close database <database_name>'."
                        );
                        continue;
                    }

                    let database_name = parts[2];
                    execute_sql(
                        &mut conn_manager,
                        original_line,
                        &mut request_id,
                        &current_database,
                    );
                    if let Some(current_db) = &current_database {
                        if current_db == database_name {
                            current_database = None;
                            println!("✓ Database context cleared for: {}", database_name);
                        }
                    }
                    continue;
                }

                if command == "databases" {
                    execute_sql(
                        &mut conn_manager,
                        "show databases",
                        &mut request_id,
                        &current_database,
                    );
                    continue;
                }

                if command == "tables" {
                    execute_sql(
                        &mut conn_manager,
                        "show tables",
                        &mut request_id,
                        &current_database,
                    );
                    continue;
                }

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

                            let mut current_statement = String::new();
                            for line in content.lines() {
                                let trimmed_line = line.trim();

                                if trimmed_line.is_empty() || trimmed_line.starts_with("--") {
                                    continue;
                                }

                                current_statement.push_str(trimmed_line);
                                current_statement.push(' ');

                                if trimmed_line.ends_with(';') {
                                    let statement = current_statement.trim_end_matches(';').trim();
                                    if !statement.is_empty() {
                                        execute_sql(
                                            &mut conn_manager,
                                            statement,
                                            &mut request_id,
                                            &current_database,
                                        );
                                    }
                                    current_statement.clear();
                                }
                            }

                            let statement = current_statement.trim();
                            if !statement.is_empty() {
                                execute_sql(
                                    &mut conn_manager,
                                    statement,
                                    &mut request_id,
                                    &current_database,
                                );
                            }

                            println!("✓ Successfully executed commands from file: {}", file_path);
                        }
                        Err(err) => {
                            eprintln!("Error: Failed to read file {}: {:?}", file_path, err);
                        }
                    }
                    continue;
                }

                execute_sql(
                    &mut conn_manager,
                    original_line,
                    &mut request_id,
                    &current_database,
                );
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                println!("\nReceived interrupt signal, exiting...");
                break;
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute_sql(
    conn_manager: &mut ConnectionManager,
    sql: &str,
    request_id: &mut u64,
    current_database: &Option<String>,
) {
    let sql_lower = sql.trim().to_lowercase();
    let needs_database = !sql_lower.starts_with("create database ")
        && !sql_lower.starts_with("drop database ")
        && !sql_lower.starts_with("use database ")
        && !sql_lower.starts_with("show databases")
        && !sql_lower.starts_with("stat")
        && !sql_lower.starts_with("healthcheck")
        && !sql_lower.starts_with("flush");

    if needs_database && current_database.is_none() {
        eprintln!(
            "Error: No database selected. Please use 'use database <database_name>' to select a database first."
        );
        return;
    }

    let query_request = QueryRequest {
        sql: sql.to_string(),
        parameters: Vec::new(),
        fetch_size: 100,
        use_cursor: false,
    };

    let jdbc_request = JdbcRequest {
        request_id: *request_id,
        request: Some(jdbc_request::Request::Query(query_request)),
    };

    if let Err(e) = conn_manager.send_jdbc_request(&jdbc_request) {
        eprintln!("Failed to send SQL command: {}", e);
        return;
    }

    match conn_manager.read_jdbc_response() {
        Ok(response) => {
            process_jdbc_response(&response);
        }
        Err(e) => {
            eprintln!("Failed to read SQL response: {}", e);
            return;
        }
    }

    *request_id += 1;
}

fn process_jdbc_response(response: &JdbcResponse) {
    // 0 是 Status::OK 的值
    if response.status == 0 {
        // 处理成功响应
        match &response.response {
            Some(jdbc_response::Response::ResultSet(result_set)) => {
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
        Some(value::Value::VectorData(v)) => format_vector_data(v),
        Some(value::Value::NullValue(_)) => "NULL".to_string(),
        None => "NULL".to_string(),
    }
}

// 处理向量数据，转换为字符串
fn format_vector_data(vector_data: &VectorData) -> String {
    if !vector_data.values.is_empty() {
        // 使用float值
        let values: Vec<String> = vector_data
            .values
            .iter()
            .map(|v| format!("{:.4}", v))
            .collect();
        format!("vector[{}]", values.join(", "))
    } else if !vector_data.double_values.is_empty() {
        // 使用double值
        let values: Vec<String> = vector_data
            .double_values
            .iter()
            .map(|v| format!("{:.4}", v))
            .collect();
        format!("vector[{}]", values.join(", "))
    } else {
        "vector[]".to_string()
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
    println!("  databases                      - List all databases");
    println!("  tables                         - List all tables");
    println!("  describe <table>               - Show table schema");
    println!("  desc <table>                   - Shortcut for describe");
    println!("  select ...                     - Execute SELECT query");
    println!("  insert ...                     - Execute INSERT statement");
    println!("  update ...                     - Execute UPDATE statement");
    println!("  delete ...                     - Execute DELETE statement");
    println!("  create table ...               - Create a new table");
    println!("  create index ...               - Create a new index");
    println!("  create database ...            - Create a new database");
    println!("  drop database ...              - Drop an existing database");
    println!("  use database ...               - Switch to a specified database");
    println!("  close database ...             - Close a specified database");
    println!("  stat                           - Show database monitoring statistics");
    println!("  healthcheck                    - Check database health status");
    println!("  begin, begin transaction       - Start a new transaction");
    println!("  commit                         - Commit current transaction");
    println!("  rollback                       - Rollback current transaction");
    println!("  flush                          - Flush WAL logs to disk");
}
