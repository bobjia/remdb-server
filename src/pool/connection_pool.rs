use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::Semaphore;

/// 高性能连接池
pub struct HighPerfConnectionPool {
    connections: Mutex<VecDeque<PooledConnection>>,
    semaphore: Semaphore,
    max_size: usize,
    active_count: AtomicUsize,
    // 统计信息
    stats: PoolStats,
}

/// 池化连接
struct PooledConnection {
    id: u64,
    last_used: Instant,
    connection: ConnectionHandle,
    // 零拷贝缓冲区
    zero_copy_buffer: Option<ZeroCopyBuffer>,
}

/// 连接句柄
struct ConnectionHandle {
    // 实际连接的封装，这里可以是TCP连接、数据库连接等
    // 目前是一个简单的占位符
    _inner: (),
}

impl ConnectionHandle {
    /// 创建新的连接句柄
    fn new() -> Self {
        Self { _inner: () }
    }
}

/// 零拷贝缓冲区
struct ZeroCopyBuffer {
    buffer: Vec<u8>,
    last_used: Instant,
}

impl ZeroCopyBuffer {
    /// 创建新的零拷贝缓冲区
    fn new(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            last_used: Instant::now(),
        }
    }
}

/// 连接池统计
struct PoolStats {
    total_requests: AtomicUsize,
    hits: AtomicUsize,
    misses: AtomicUsize,
    avg_wait_time_ns: AtomicU64,
    total_wait_time_ns: AtomicU64,
}

impl PoolStats {
    /// 创建新的统计实例
    fn new() -> Self {
        Self {
            total_requests: AtomicUsize::new(0),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            avg_wait_time_ns: AtomicU64::new(0),
            total_wait_time_ns: AtomicU64::new(0),
        }
    }

    /// 记录请求
    fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// 记录命中
    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::SeqCst);
    }

    /// 记录未命中
    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::SeqCst);
    }

    /// 记录等待时间
    fn record_wait_time(&self, wait_time_ns: u64) {
        let total = self
            .total_wait_time_ns
            .fetch_add(wait_time_ns, Ordering::SeqCst)
            + wait_time_ns;
        let count = self.total_requests.load(Ordering::SeqCst);
        if count > 0 {
            self.avg_wait_time_ns
                .store(total / count as u64, Ordering::SeqCst);
        }
    }

    /// 获取命中率
    fn hit_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::SeqCst);
        if total == 0 {
            return 0.0;
        }
        self.hits.load(Ordering::SeqCst) as f64 / total as f64
    }
}

/// 连接池守卫
pub struct PoolGuard<'a> {
    conn: Option<PooledConnection>,
    permit: Option<tokio::sync::SemaphorePermit<'a>>,
    pool: &'a HighPerfConnectionPool,
}

impl HighPerfConnectionPool {
    /// 创建新的高性能连接池
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Mutex::new(VecDeque::with_capacity(max_size)),
            semaphore: Semaphore::new(max_size),
            max_size,
            active_count: AtomicUsize::new(0),
            stats: PoolStats::new(),
        }
    }

    /// 获取连接（无锁优化）
    pub async fn get_connection(&self) -> PoolGuard<'_> {
        let start_time = Instant::now();
        self.stats.record_request();

        // 尝试快速路径：从池中直接获取
        if let Some(conn) = self.try_get_fast() {
            self.stats.record_hit();
            return conn;
        }

        // 慢速路径：等待信号量
        let permit = self.semaphore.acquire().await.unwrap();

        // 记录等待时间
        let wait_time = start_time.elapsed();
        self.stats.record_wait_time(wait_time.as_nanos() as u64);

        // 再次尝试从池中获取
        let mut conns = self.connections.lock();
        if let Some(mut conn) = conns.pop_front() {
            drop(conns);
            self.stats.record_hit();
            // 更新最后使用时间
            conn.last_used = Instant::now();
            return PoolGuard::new(conn, Some(permit), self);
        }

        drop(conns);

        // 创建新连接
        self.stats.record_miss();
        let new_conn = self.create_new_connection().await;
        PoolGuard::new(new_conn, Some(permit), self)
    }

    /// 快速路径获取连接
    fn try_get_fast(&self) -> Option<PoolGuard<'_>> {
        let mut conns = self.connections.lock();
        if let Some(mut conn) = conns.pop_front() {
            // 更新最后使用时间
            conn.last_used = Instant::now();

            Some(PoolGuard::new(conn, None, self))
        } else {
            None
        }
    }

    /// 创建新连接
    async fn create_new_connection(&self) -> PooledConnection {
        // 实际项目中这里应该是创建真正的连接
        // 现在我们返回一个模拟的连接
        let conn_id = self.active_count.fetch_add(1, Ordering::SeqCst) as u64;

        PooledConnection {
            id: conn_id,
            last_used: Instant::now(),
            connection: ConnectionHandle::new(),
            zero_copy_buffer: Some(ZeroCopyBuffer::new(8192)),
        }
    }

    /// 归还连接
    fn return_connection(&self, mut conn: PooledConnection) {
        conn.last_used = Instant::now();

        let mut conns = self.connections.lock();
        if conns.len() < self.max_size {
            conns.push_back(conn);
        }
        // 如果池已满，连接会被丢弃
    }

    /// 获取池状态信息
    pub fn get_stats(&self) -> PoolStatsSnapshot {
        let conns = self.connections.lock();
        PoolStatsSnapshot {
            max_size: self.max_size,
            current_size: conns.len(),
            active_connections: self.active_count.load(Ordering::SeqCst),
            total_requests: self.stats.total_requests.load(Ordering::SeqCst),
            hits: self.stats.hits.load(Ordering::SeqCst),
            misses: self.stats.misses.load(Ordering::SeqCst),
            avg_wait_time_ns: self.stats.avg_wait_time_ns.load(Ordering::SeqCst),
            hit_rate: self.stats.hit_rate(),
        }
    }
}

impl<'a> PoolGuard<'a> {
    /// 创建新的池守卫
    fn new(
        conn: PooledConnection,
        permit: Option<tokio::sync::SemaphorePermit<'a>>,
        pool: &'a HighPerfConnectionPool,
    ) -> Self {
        Self {
            conn: Some(conn),
            permit,
            pool,
        }
    }

    /// 获取连接引用
    pub fn get_connection(&self) -> Option<&ConnectionHandle> {
        self.conn.as_ref().map(|conn| &conn.connection)
    }

    /// 获取可变连接引用
    pub fn get_connection_mut(&mut self) -> Option<&mut ConnectionHandle> {
        self.conn.as_mut().map(|conn| &mut conn.connection)
    }

    /// 获取零拷贝缓冲区
    pub fn get_zero_copy_buffer(&mut self) -> Option<&mut ZeroCopyBuffer> {
        self.conn
            .as_mut()
            .and_then(|conn| conn.zero_copy_buffer.as_mut())
            .map(|buf| &mut *buf)
    }
}

impl<'a> Drop for PoolGuard<'a> {
    /// 当守卫被丢弃时，归还连接到池中
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.return_connection(conn);
        }
        // permit在退出作用域时自动释放
    }
}

/// 连接池统计快照
pub struct PoolStatsSnapshot {
    pub max_size: usize,
    pub current_size: usize,
    pub active_connections: usize,
    pub total_requests: usize,
    pub hits: usize,
    pub misses: usize,
    pub avg_wait_time_ns: u64,
    pub hit_rate: f64,
}
