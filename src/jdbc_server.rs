use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Semaphore};
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt};
use tokio::time::{timeout, Duration};
use remdb::RemDb;
use crate::sql_engine::{execute_extended_sql, ResultSet};
use crate::debug_println;

/// JDBC服务器
pub struct JdbcServer {
    db: Arc<Mutex<&'static mut RemDb>>,
    port: u16,
    max_connections: usize,
    timeout: u64,
}

impl JdbcServer {
    /// 创建新的JDBC服务器
    pub fn new(db: Arc<Mutex<&'static mut RemDb>>, port: u16, max_connections: usize, timeout: u64) -> Self {
        Self {
            db,
            port,
            max_connections,
            timeout,
        }
    }

    /// 启动JDBC服务器
    pub async fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        println!("JDBC server started on port {}", self.port);
        println!("Maximum connections: {}", self.max_connections);
        println!("SQL execution timeout: {} seconds", self.timeout);

        // 创建信号量来限制并发连接数
        let semaphore = Arc::new(Semaphore::new(self.max_connections));
        let timeout = self.timeout;

        loop {
            // 等待连接，获取信号量许可
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let (mut socket, addr) = listener.accept().await?;
            println!("New JDBC connection from {}", addr);

            let db = self.db.clone();
            let conn_timeout = timeout;

            // 处理连接
            tokio::spawn(async move {
                // 连接处理完成后释放许可
                let _permit = permit;
                if let Err(e) = Self::handle_connection(socket, db, conn_timeout).await {
                    println!("JDBC connection error: {:?}", e);
                }
                println!("JDBC connection closed: {}", addr);
            });
        }
    }

    /// 处理JDBC连接
    async fn handle_connection(socket: tokio::net::TcpStream, db: Arc<Mutex<&'static mut RemDb>>, timeout: u64) -> std::io::Result<()> {
        println!("Handling new JDBC connection");
        // 分离TCP流的读写部分，避免可变借用冲突
        let (read_half, mut write_half) = tokio::io::split(socket);
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut line = String::new();

        loop {
            // 读取客户端请求（完整的一行）
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                println!("JDBC connection closed by client");
                return Ok(());
            }

            let request = line.trim().to_string();
            println!("JDBC request: {}", request);

            // 处理请求
            let response = Self::process_request(request, db.clone(), timeout).await;
            println!("JDBC response: {}", response);

            // 发送响应
            write_half.write_all(response.as_bytes()).await?;
            write_half.write_all(b"\n").await?;
            write_half.flush().await?;

            // 清空行缓冲区
            line.clear();
        }
    }

    /// 处理JDBC请求
    async fn process_request(request: String, db: Arc<Mutex<&'static mut RemDb>>, timeout: u64) -> String {
        let parts: Vec<&str> = request.split('|').collect();
        if parts.is_empty() {
            return "ERROR|Invalid request format".to_string();
        }

        let command = parts[0];
        match command {
            "EXECUTE" => {
                if parts.len() < 2 {
                    return "ERROR|Missing SQL statement".to_string();
                }
                let sql = parts[1];
                Self::execute_sql(sql, db, timeout).await
            }
            "CLOSE" => {
                "OK|Connection closed".to_string()
            }
            _ => {
                format!("ERROR|Unknown command: {}", command)
            }
        }
    }

    /// 执行SQL语句
    async fn execute_sql(sql: &str, db: Arc<Mutex<&'static mut RemDb>>, timeout: u64) -> String {
        println!("Executing SQL: {}", sql);
        let start_time = std::time::Instant::now();
        
        // 保存sql字符串的副本，因为它需要在spawn_blocking中使用
        let sql_copy = sql.to_string();
        
        // 创建一个新的Arc副本，以便在spawn_blocking中使用
        let db_copy = db.clone();
        
        // 使用spawn_blocking将同步的SQL执行包装起来，并设置配置的超时时间
        let result = tokio::time::timeout(
            Duration::from_secs(timeout),
            tokio::task::spawn_blocking(move || {
                // 在spawn_blocking内部获取数据库锁
                let mut db_lock = db_copy.blocking_lock();
                execute_extended_sql(&mut *db_lock, &sql_copy)
            })
        ).await;
        
        let duration = start_time.elapsed();
        println!("SQL execution took {:?}", duration);
        
        match result {
            Ok(join_result) => {
                match join_result {
                    Ok(sql_result) => {
                        match sql_result {
                            Ok(result_set) => {
                                let formatted_result = Self::format_result_set(result_set);
                                println!("Formatted result: {}", formatted_result);
                                formatted_result
                            }
                            Err(err) => {
                                let error_result = format!("ERROR|{:?}", err);
                                println!("SQL error: {}", error_result);
                                error_result
                            }
                        }
                    }
                    Err(join_err) => {
                        let error_result = format!("ERROR|Task join error: {:?}", join_err);
                        println!("SQL task error: {}", error_result);
                        error_result
                    }
                }
            }
            Err(timeout_err) => {
                let error_result = format!("ERROR|SQL execution timeout after {} seconds: {:?}", timeout, timeout_err);
                println!("SQL timeout error: {}", error_result);
                error_result
            }
        }
    }

    /// 格式化结果集
    fn format_result_set(result_set: ResultSet) -> String {
        if result_set.columns.is_empty() {
            return format!("OK|{}|0|{}", result_set.affected_rows, "");
        }

        let columns = result_set.columns.join(",");
        let mut rows = Vec::new();
        for row in result_set.rows {
            rows.push(row.join(","));
        }
        let rows_str = rows.join(";");

        format!("OK|{}|{}|{}|{}", 
            result_set.affected_rows, 
            result_set.columns.len(), 
            columns, 
            rows_str)
    }
}
