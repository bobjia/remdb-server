use crate::handler::JdbcProtocolHandler;
use crate::handler::health_monitor::{HealthStatus, ServerHealthMonitor};
use crate::pool::HighPerfConnectionPool;
use crate::sql_engine::{ResultSet, execute_extended_sql};
use crate::tuning::SystemTuner;
use hex;
use remdb::RemDb;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio::time::Duration;
use tracing::{error, info, warn};

/// JDBC服务器
pub struct JdbcServer {
    db: Arc<Mutex<&'static mut RemDb>>,
    port: u16,
    max_connections: usize,
    timeout: u64,
    /// 认证配置
    auth_enabled: bool,
    username: String,
    password_hash: String,
    /// 高性能协议处理器
    protocol_handler: Arc<JdbcProtocolHandler>,
    /// 高性能连接池
    connection_pool: Arc<HighPerfConnectionPool>,
    /// 系统调优器
    system_tuner: Arc<SystemTuner>,
    /// 健康监控器
    health_monitor: Arc<ServerHealthMonitor>,
}

impl JdbcServer {
    /// 创建新的JDBC服务器
    pub fn new(
        db: Arc<Mutex<&'static mut RemDb>>,
        port: u16,
        max_connections: usize,
        timeout: u64,
        auth_enabled: bool,
        username: String,
        password_hash: String,
    ) -> Self {
        // 获取CPU核心数作为工作线程数
        let worker_count = num_cpus::get();

        // 创建高性能组件
        let protocol_handler = Arc::new(JdbcProtocolHandler::new(
            worker_count,
            db.clone(),
            auth_enabled,
            username.clone(),
            password_hash.clone(),
        ));
        let connection_pool = Arc::new(HighPerfConnectionPool::new(max_connections));
        let system_tuner = Arc::new(SystemTuner::new());
        let health_monitor = Arc::new(ServerHealthMonitor::new(5));

        Self {
            db,
            port,
            max_connections,
            timeout,
            auth_enabled,
            username,
            password_hash,
            protocol_handler,
            connection_pool,
            system_tuner,
            health_monitor,
        }
    }

    /// 启动JDBC服务器
    pub async fn start(&self) -> std::io::Result<()> {
        // 暂时注释掉系统调优功能，待修复后重新启用
        // let tuner_clone = self.system_tuner.clone();
        // tokio::spawn(async move {
        //     tuner_clone.start_auto_tuning().await;
        // });

        // 绑定TCP监听器
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        info!("High-performance JDBC server started on port {}", self.port);
        info!("Maximum connections: {}", self.max_connections);
        info!("SQL execution timeout: {} seconds", self.timeout);
        info!("Worker threads: {}", num_cpus::get());

        // 创建信号量来限制并发连接数
        let semaphore = Arc::new(Semaphore::new(self.max_connections));

        loop {
            tokio::select! {
                // 接受新连接
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((socket, addr)) => {
                            info!("New JDBC connection from: {}", addr);

                            // 获取信号量许可
                            let permit = semaphore.clone().acquire_owned().await.unwrap();
                            let handler = self.protocol_handler.clone();
                            let health_monitor = self.health_monitor.clone();

                            // 处理连接
                            tokio::spawn(async move {
                                // 连接处理完成后释放许可
                                let _permit = permit;

                                // 使用高性能协议处理器处理连接
                                match handler.handle_connection(socket).await {
                                    Ok(_) => {
                                        health_monitor.record_success();
                                    }
                                    Err(e) => {
                                        error!("JDBC connection error: {:?}", e);
                                        health_monitor.record_error();
                                    }
                                }

                                info!("JDBC connection closed: {}", addr);
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                            self.health_monitor.record_error();
                        }
                    }
                }
            }
        }
    }

    /// 获取系统调优器
    pub fn get_system_tuner(&self) -> &SystemTuner {
        &self.system_tuner
    }

    /// 获取连接池统计信息
    pub fn get_pool_stats(&self) -> crate::pool::PoolStatsSnapshot {
        self.connection_pool.get_stats()
    }

    /// 获取系统健康状态
    pub fn get_health_status(&self) -> crate::handler::health_monitor::HealthStatus {
        self.health_monitor.get_status()
    }

    /// 获取错误计数
    pub fn get_error_count(&self) -> u64 {
        self.health_monitor.get_error_count()
    }

    /// 验证用户名和密码
    pub fn verify_credentials(
        provided_username: &str,
        provided_password: &str,
        expected_username: &str,
        expected_password_hash: &str,
    ) -> bool {
        // 首先验证用户名
        if provided_username != expected_username {
            return false;
        }

        // 计算提供的密码的SHA-256哈希值
        let mut hasher = Sha256::new();
        hasher.update(provided_password);
        let provided_hash = hasher.finalize();

        // 将计算得到的哈希值转换为十六进制字符串
        let provided_hash_str = hex::encode(provided_hash);

        // 比较哈希值
        provided_hash_str == expected_password_hash
    }
}
