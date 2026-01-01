use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, Semaphore};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use remdb::RemDb;
use crate::sql_engine::{execute_extended_sql, ResultSet};
use crate::debug_println;

/// JDBC服务器
pub struct JdbcServer {
    db: Arc<Mutex<&'static mut RemDb>>,
    port: u16,
    max_connections: usize,
}

impl JdbcServer {
    /// 创建新的JDBC服务器
    pub fn new(db: Arc<Mutex<&'static mut RemDb>>, port: u16, max_connections: usize) -> Self {
        Self {
            db,
            port,
            max_connections,
        }
    }

    /// 启动JDBC服务器
    pub async fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        debug_println!("JDBC server started on port {}", self.port);
        debug_println!("Maximum connections: {}", self.max_connections);

        // 创建信号量来限制并发连接数
        let semaphore = Arc::new(Semaphore::new(self.max_connections));

        loop {
            // 等待连接，获取信号量许可
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let (mut socket, addr) = listener.accept().await?;
            debug_println!("New JDBC connection from {}", addr);

            let db = self.db.clone();

            // 处理连接
            tokio::spawn(async move {
                // 连接处理完成后释放许可
                let _permit = permit;
                if let Err(e) = Self::handle_connection(&mut socket, db).await {
                    debug_println!("JDBC connection error: {:?}", e);
                }
                debug_println!("JDBC connection closed: {}", addr);
            });
        }
    }

    /// 处理JDBC连接
    async fn handle_connection(socket: &mut tokio::net::TcpStream, db: Arc<Mutex<&'static mut RemDb>>) -> std::io::Result<()> {
        let mut buf = vec![0; 1024];

        loop {
            // 读取客户端请求
            let n = socket.read(&mut buf).await?;
            if n == 0 {
                return Ok(());
            }

            let request = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            debug_println!("JDBC request: {}", request);

            // 处理请求
            let response = Self::process_request(request, db.clone()).await;

            // 发送响应
            socket.write_all(response.as_bytes()).await?;
            socket.write_all(b"\n").await?;
        }
    }

    /// 处理JDBC请求
    async fn process_request(request: String, db: Arc<Mutex<&'static mut RemDb>>) -> String {
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
                Self::execute_sql(sql, db).await
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
    async fn execute_sql(sql: &str, db: Arc<Mutex<&'static mut RemDb>>) -> String {
        let mut db_lock = db.lock().await;
        match execute_extended_sql(&mut db_lock, sql) {
            Ok(result_set) => {
                Self::format_result_set(result_set)
            }
            Err(err) => {
                format!("ERROR|{:?}", err)
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
