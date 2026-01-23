use crate::network::ZeroCopyTransport;
use crate::proto::*;
use crate::sql_engine::{ResultSet, execute_extended_sql};
use bytes::Bytes;
use crossbeam::queue::SegQueue;
use hex;
use prost::Message;
use rayon::prelude::*;
use remdb::RemDb;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};

/// JDBC协议处理器
pub struct JdbcProtocolHandler {
    // 无锁请求队列
    request_queue: Arc<SegQueue<(JdbcRequest, mpsc::UnboundedSender<JdbcResponse>)>>,
    // 工作线程池
    workers: Vec<WorkerThread>,
    // 统计信息
    metrics: HandlerMetrics,
    // 数据库引用
    db: Arc<std::sync::Mutex<&'static mut RemDb>>,
    // 认证配置
    auth_enabled: bool,
    username: String,
    password_hash: String,
}

/// 工作线程
struct WorkerThread {
    id: u32,
    handle: Option<std::thread::JoinHandle<()>>,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    // 认证配置
    auth_enabled: bool,
    username: String,
    password_hash: String,
}

/// 处理器指标
#[derive(Clone)]
struct HandlerMetrics {
    requests_processed: Arc<AtomicU64>,
    avg_latency_ns: Arc<AtomicU64>,
    active_connections: Arc<AtomicU32>,
    total_latency_ns: Arc<AtomicU64>,
}

impl HandlerMetrics {
    /// 创建新的指标实例
    fn new() -> Self {
        Self {
            requests_processed: Arc::new(AtomicU64::new(0)),
            avg_latency_ns: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU32::new(0)),
            total_latency_ns: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 记录新连接
    fn new_connection(&self) -> u32 {
        self.active_connections.fetch_add(1, Ordering::SeqCst)
    }

    /// 记录连接关闭
    fn connection_closed(&self, _conn_id: u32) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }

    /// 记录请求
    fn record_request(&self) {
        self.requests_processed.fetch_add(1, Ordering::SeqCst);
    }

    /// 记录延迟
    fn record_latency(&self, latency_ns: u64) {
        let total = self
            .total_latency_ns
            .fetch_add(latency_ns, Ordering::SeqCst)
            + latency_ns;
        let count = self.requests_processed.load(Ordering::SeqCst);
        if count > 0 {
            self.avg_latency_ns.store(total / count, Ordering::SeqCst);
        }
    }
}

impl WorkerThread {
    /// 创建新的工作线程
    fn new(
        id: u32,
        request_queue: Arc<SegQueue<(JdbcRequest, mpsc::UnboundedSender<JdbcResponse>)>>,
        db: Arc<std::sync::Mutex<&'static mut RemDb>>,
        auth_enabled: bool,
        username: String,
        password_hash: String,
    ) -> Self {
        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_clone = stop_flag.clone();
        let queue_clone = request_queue.clone();
        let db_clone = db.clone();
        let auth_enabled_clone = auth_enabled;
        let username_clone = username.clone();
        let password_hash_clone = password_hash.clone();

        let handle = std::thread::spawn(move || {
            while !flag_clone.load(Ordering::SeqCst) {
                // 从无锁队列获取请求
                if let Some((request, response_tx)) = queue_clone.pop() {
                    // 处理请求
                    let response = JdbcProtocolHandler::process_single_request(
                        &mut *db_clone.lock().unwrap(),
                        request,
                        auth_enabled_clone,
                        &username_clone,
                        &password_hash_clone,
                    );
                    // 发送响应
                    if response_tx.send(response).is_err() {
                        warn!("Failed to send response: channel closed");
                    }
                } else {
                    // 短暂休眠，避免CPU忙等
                    std::thread::sleep(std::time::Duration::from_nanos(100));
                }
            }
        });

        Self {
            id,
            handle: Some(handle),
            stop_flag,
            auth_enabled,
            username,
            password_hash,
        }
    }
}

impl JdbcProtocolHandler {
    /// 创建新的JDBC协议处理器
    pub fn new(
        worker_count: usize, 
        db: Arc<std::sync::Mutex<&'static mut RemDb>>,
        auth_enabled: bool,
        username: String,
        password_hash: String,
    ) -> Self {
        let request_queue = Arc::new(SegQueue::new());

        let mut workers = Vec::with_capacity(worker_count);
        for i in 0..worker_count {
            let worker = WorkerThread::new(
                i as u32, 
                request_queue.clone(), 
                db.clone(),
                auth_enabled,
                username.clone(),
                password_hash.clone(),
            );
            workers.push(worker);
        }

        Self {
            request_queue,
            workers,
            metrics: HandlerMetrics::new(),
            db,
            auth_enabled,
            username,
            password_hash,
        }
    }

    /// 处理JDBC连接
    pub async fn handle_connection(
        &self,
        socket: TcpStream,
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn_id = self.metrics.new_connection();
        info!("New JDBC connection established: {}", conn_id);

        // 创建响应通道
        let (response_tx, response_rx) = mpsc::unbounded_channel::<JdbcResponse>();

        // 检查是否启用了zerocopy特性
        #[cfg(feature = "zerocopy")]
        {
            info!("Using zero-copy transport for JDBC connection: {}", conn_id);
            // 使用零拷贝传输
            let mut transport = ZeroCopyTransport::new(socket);
            
            // 设置TCP选项
            if let Err(e) = transport.set_tcp_options() {
                error!("Failed to set TCP options for zero-copy transport: {:?}", e);
            }

            // 使用Arc<tokio::sync::Mutex>来共享ZeroCopyTransport
            let shared_transport = Arc::new(Mutex::new(transport));
            let shared_transport_writer = shared_transport.clone();
            
            // 启动零拷贝响应发送任务
            let (tx_clone, rx_clone) = (response_tx.clone(), response_rx);
            tokio::spawn(async move {
                let mut rx = rx_clone;
                while let Some(response) = rx.recv().await {
                    let mut buf = Vec::new();
                    if let Err(e) = response.encode(&mut buf) {
                        error!("Failed to encode response: {:?}", e);
                        break;
                    }

                    let len = buf.len() as u32;
                    let mut full_buf = Vec::with_capacity(4 + buf.len());
                    full_buf.extend_from_slice(&len.to_be_bytes());
                    full_buf.extend_from_slice(&buf);

                    let mut transport = shared_transport_writer.lock().await;
                    if let Err(e) = transport.send_zero_copy(Bytes::from(full_buf)).await {
                        error!("Failed to send zero-copy response: {:?}", e);
                        break;
                    }
                }
            });

            // 主循环：零拷贝读取请求
            loop {
                // 读取4字节的请求长度（大端）
                let len_data = {
                    let mut transport = shared_transport.lock().await;
                    transport.read_zero_copy().await?
                };
                if len_data.len() != 4 {
                    break;
                }

                let len_buf: [u8; 4] = len_data.as_ref().try_into()?;
                let len = u32::from_be_bytes(len_buf) as usize;

                // 读取完整的请求数据
                let data_buf = {
                    let mut transport = shared_transport.lock().await;
                    transport.read_zero_copy().await?
                };
                if data_buf.len() != len {
                    break;
                }

                // 解析JDBC请求
                if let Ok(request) = JdbcRequest::decode(data_buf.as_ref()) {
                    self.request_queue.push((request, tx_clone.clone()));
                    self.metrics.record_request();
                } else {
                    error!("Failed to decode JDBC request");
                }
            }
        }

        // 默认使用普通TCP流
        #[cfg(not(feature = "zerocopy"))]
        {
            // 设置TCP选项
            if let Err(e) = socket.set_nodelay(true) {
                error!("Failed to set TCP options: {:?}", e);
            }

            let (mut reader, mut writer) = socket.into_split();

            // 启动响应发送任务（独占writer，避免读写互斥）
            tokio::spawn(async move {
                let mut rx = response_rx;
                while let Some(response) = rx.recv().await {
                    let mut buf = Vec::new();
                    if let Err(e) = response.encode(&mut buf) {
                        error!("Failed to encode response: {:?}", e);
                        break;
                    }

                    let len = buf.len() as u32;
                    let mut full_buf = Vec::with_capacity(4 + buf.len());
                    full_buf.extend_from_slice(&len.to_be_bytes());
                    full_buf.extend_from_slice(&buf);

                    if let Err(e) = writer.write_all(&full_buf).await {
                        error!("Failed to send response: {:?}", e);
                        break;
                    }
                    if let Err(e) = writer.flush().await {
                        error!("Failed to flush response: {:?}", e);
                        break;
                    }
                }
            });

            // 主循环处理请求
            loop {
                // 读取4字节的请求长度（大端）
                let mut len_buf = [0u8; 4];
                let mut bytes_read = 0;

                while bytes_read < 4 {
                    let result = reader.read(&mut len_buf[bytes_read..]).await;
                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            bytes_read += n;
                        }
                        Err(e) => {
                            error!(
                                "Connection {} error reading request length: {:?}",
                                conn_id, e
                            );
                            break;
                        }
                    }
                }

                if bytes_read != 4 {
                    break;
                }

                let len = u32::from_be_bytes(len_buf) as usize;

                // 然后读取完整的请求数据
                let mut data_buf = vec![0u8; len];
                let mut bytes_read = 0;

                while bytes_read < len {
                    let result = reader.read(&mut data_buf[bytes_read..]).await;
                    match result {
                        Ok(0) => break,
                        Ok(n) => {
                            bytes_read += n;
                        }
                        Err(e) => {
                            error!("Connection {} error reading request data: {:?}", conn_id, e);
                            break;
                        }
                    }
                }

                if bytes_read != len {
                    break;
                }

                // 解析JDBC请求
                if let Ok(request) = JdbcRequest::decode(&data_buf[..]) {
                    self.request_queue.push((request, response_tx.clone()));
                    self.metrics.record_request();
                } else {
                    error!("Failed to decode JDBC request");
                }
            }
        }

        self.metrics.connection_closed(conn_id);

        Ok(())
    }

    /// 处理单个请求
    fn process_single_request(
        db: &mut RemDb, 
        request: JdbcRequest, 
        auth_enabled: bool,
        expected_username: &str,
        expected_password_hash: &str,
    ) -> JdbcResponse {
        let start_time = Instant::now();

        // 创建默认响应
        let mut response = JdbcResponse {
            request_id: request.request_id,
            status: Status::Ok.into(),
            error_message: String::new(),
            response: None,
        };

        // 根据请求类型处理
        match request.request {
            Some(jdbc_request::Request::Query(query)) => {
                let sql = query.sql;
                // 执行SQL查询
                match execute_extended_sql(db, &sql) {
                    Ok(result_set) => {
                        // 转换为响应格式
                        let formatted_result = Self::convert_to_result_set_response(result_set);
                        response.response =
                            Some(jdbc_response::Response::ResultSet(formatted_result));
                    }
                    Err(err) => {
                        response.status = Status::Error.into();
                        response.error_message = format!("{:?}", err);
                    }
                }
            }
            Some(jdbc_request::Request::Batch(batch)) => {
                // 串行执行批处理请求
                let mut affected_rows = 0;
                let mut batch_success = true;

                for sql in &batch.sql_statements {
                    match execute_extended_sql(db, sql) {
                        Ok(result_set) => {
                            affected_rows += result_set.affected_rows;
                        }
                        _ => {
                            response.status = Status::Error.into();
                            response.error_message = "Batch execution failed".to_string();
                            batch_success = false;
                            break;
                        }
                    }
                }

                // 设置更新响应
                let update_response = UpdateResponse {
                    affected_rows: affected_rows as u64,
                    last_insert_id: 0,
                };
                response.response = Some(jdbc_response::Response::Update(update_response));
            }
            Some(jdbc_request::Request::BeginTransaction(begin)) => {
                // 开始事务
                let tx_type = match begin.r#type() {
                    TransactionType::ReadOnly => remdb::transaction::TransactionType::ReadOnly,
                    TransactionType::ReadWrite => remdb::transaction::TransactionType::ReadWrite,
                    _ => remdb::transaction::TransactionType::ReadWrite,
                };

                let isolation_level = match begin.isolation_level() {
                    IsolationLevel::ReadUncommitted => {
                        remdb::transaction::IsolationLevel::ReadUncommitted
                    }
                    IsolationLevel::ReadCommitted => {
                        remdb::transaction::IsolationLevel::ReadCommitted
                    }
                    IsolationLevel::RepeatableRead => {
                        remdb::transaction::IsolationLevel::RepeatableRead
                    }
                    IsolationLevel::Serializable => {
                        remdb::transaction::IsolationLevel::Serializable
                    }
                    _ => remdb::transaction::IsolationLevel::ReadCommitted,
                };

                unsafe {
                    match db.begin_transaction(
                        tx_type,
                        isolation_level,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        0,
                    ) {
                        Ok(_) => {
                            let tx_response = TransactionResponse {
                                transaction_id: 0, // TODO: 实现事务ID生成
                            };
                            response.response =
                                Some(jdbc_response::Response::Transaction(tx_response));
                        }
                        Err(err) => {
                            response.status = Status::Error.into();
                            response.error_message = format!("{:?}", err);
                        }
                    }
                }
            }
            Some(jdbc_request::Request::CommitTransaction(_)) => {
                // 提交事务
                unsafe {
                    match db.commit_transaction() {
                        Ok(_) => {
                            let tx_response = TransactionResponse { transaction_id: 0 };
                            response.response =
                                Some(jdbc_response::Response::Transaction(tx_response));
                        }
                        Err(err) => {
                            response.status = Status::Error.into();
                            response.error_message = format!("{:?}", err);
                        }
                    }
                }
            }
            Some(jdbc_request::Request::RollbackTransaction(_)) => {
                // 回滚事务
                unsafe {
                    match db.rollback_transaction() {
                        Ok(_) => {
                            let tx_response = TransactionResponse { transaction_id: 0 };
                            response.response =
                                Some(jdbc_response::Response::Transaction(tx_response));
                        }
                        Err(err) => {
                            response.status = Status::Error.into();
                            response.error_message = format!("{:?}", err);
                        }
                    }
                }
            }
            Some(jdbc_request::Request::Connection(conn_req)) => {
                // 处理连接请求
                if auth_enabled {
                    // 验证用户名和密码
                    let provided_username = conn_req.username;
                    let provided_password = conn_req.password;
                    
                    // 计算提供的密码的SHA-256哈希值
                    let mut hasher = Sha256::new();
                    hasher.update(provided_password);
                    let provided_hash = hasher.finalize();
                    let provided_hash_str = hex::encode(provided_hash);
                    
                    // 比较用户名和哈希值
                    if provided_username != expected_username || provided_hash_str != expected_password_hash {
                        response.status = Status::Unauthorized.into();
                        response.error_message = "Invalid username or password".to_string();
                        return response;
                    }
                }
                
                // 认证成功，返回连接响应
                let conn_response = ConnectionResponse {
                    connection_id: 1,
                    server_version: "0.1.0".to_string(),
                    protocol_version: 1,
                };
                response.response = Some(jdbc_response::Response::Connection(conn_response));
            }
            _ => {
                // 未实现的请求类型
                response.status = Status::Error.into();
                response.error_message = "Request type not implemented".to_string();
            }
        }

        // 记录延迟
        let duration = start_time.elapsed();
        // 注意：这里无法直接访问metrics，因为worker线程没有引用
        // 后续可以考虑使用全局metrics或其他方式

        response
    }

    /// 转换为结果集响应
    fn convert_to_result_set_response(result_set: ResultSet) -> ResultSetResponse {
        let mut columns = Vec::new();
        let mut rows = Vec::new();

        // 转换列元数据
        for (i, col_name) in result_set.columns.iter().enumerate() {
            // TODO: 获取实际数据类型
            let col = ColumnMetadata {
                name: col_name.clone(),
                r#type: DataType::Varchar.into(),
                precision: 0,
                scale: 0,
                nullable: true,
                primary_key: i == 0, // 假设第一列为主键
            };
            columns.push(col);
        }

        // 转换行数据
        for row in result_set.rows {
            let mut values = Vec::new();
            for value in row {
                // 检查是否是向量值
                if value.starts_with('[') && value.ends_with(']') {
                    // 这是一个向量值，尝试解析为float数组
                    let vector_str = &value[1..value.len()-1];
                    let elements: Vec<&str> = vector_str.split(',').map(|s| s.trim()).collect();
                    let mut float_values = Vec::new();
                    let mut double_values = Vec::new();
                    let mut is_double = false;
                    
                    // 尝试解析元素
                    for elem in elements {
                        if let Ok(f) = elem.parse::<f32>() {
                            float_values.push(f);
                        } else if let Ok(d) = elem.parse::<f64>() {
                            double_values.push(d);
                            is_double = true;
                        }
                    }
                    
                    // 根据解析结果创建向量数据
                    let vector_data = VectorData {
                        values: if is_double { Vec::new() } else { float_values },
                        double_values: if is_double { double_values } else { Vec::new() },
                    };
                    
                    let val = Value {
                        value: Some(value::Value::VectorData(vector_data)),
                    };
                    values.push(val);
                } else {
                    // 普通字符串值
                    let val = Value {
                        value: Some(value::Value::StringValue(value)),
                    };
                    values.push(val);
                }
            }
            let row_data = RowData { values };
            rows.push(row_data);
        }

        let row_count = rows.len() as u64;

        ResultSetResponse {
            columns,
            rows,
            row_count,
            has_more_rows: false,
        }
    }

    /// 批量请求处理
    pub async fn handle_batch(&self, batch: Vec<JdbcRequest>) -> Vec<JdbcResponse> {
        // 串行处理请求，避免并行带来的锁问题
        let mut responses = Vec::with_capacity(batch.len());
        let mut db_lock = self.db.lock().unwrap();
        
        let auth_enabled = self.auth_enabled;
        let username = self.username.clone();
        let password_hash = self.password_hash.clone();

        for req in batch {
            responses.push(JdbcProtocolHandler::process_single_request(
                &mut *db_lock,
                req,
                auth_enabled,
                &username,
                &password_hash,
            ));
        }

        responses
    }
}
