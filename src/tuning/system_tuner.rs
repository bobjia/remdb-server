use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::time;
use tracing::{error, info, warn};

/// 系统调优器
pub struct SystemTuner {
    system: System,
    tuning_active: AtomicBool,
    // 动态调整参数
    thread_pool_size: AtomicUsize,
    buffer_pool_size: AtomicUsize,
    connection_limit: AtomicUsize,
    // 上一次调整时间
    last_tuning_time: std::sync::atomic::AtomicU64,
}

/// 调优错误
#[derive(Debug, thiserror::Error)]
pub enum TunerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Unsupported operation")]
    Unsupported,
}

impl SystemTuner {
    /// 创建新的系统调优器
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            tuning_active: AtomicBool::new(false),
            thread_pool_size: AtomicUsize::new(num_cpus::get()),
            buffer_pool_size: AtomicUsize::new(16),
            connection_limit: AtomicUsize::new(10000),
            last_tuning_time: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 启动自动调优
    pub async fn start_auto_tuning(&self) {
        self.tuning_active.store(true, Ordering::SeqCst);
        info!("Starting system auto-tuning");

        let mut interval = time::interval(Duration::from_secs(10));

        while self.tuning_active.load(Ordering::SeqCst) {
            interval.tick().await;
            // 暂时注释掉调优逻辑，待修复后重新启用
            // self.adjust_parameters();
        }

        info!("System auto-tuning stopped");
    }

    /// 停止自动调优
    pub fn stop_auto_tuning(&self) {
        self.tuning_active.store(false, Ordering::SeqCst);
    }

    /// 调整参数
    fn adjust_parameters(&mut self) {
        self.system.refresh_all();

        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        let memory_usage = self.system.used_memory() as f64 / self.system.total_memory() as f64;

        info!(
            "System tuning: CPU usage = {:.1}%, Memory usage = {:.1}%",
            cpu_usage,
            memory_usage * 100.0
        );

        // 根据系统负载动态调整
        self.adjust_thread_pool(cpu_usage);
        self.adjust_buffer_pool(memory_usage);
        self.adjust_connection_limit(cpu_usage, memory_usage);

        // 更新最后调整时间
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_tuning_time.store(now, Ordering::SeqCst);
    }

    /// 调整线程池大小
    fn adjust_thread_pool(&self, cpu_usage: f32) {
        let current = self.thread_pool_size.load(Ordering::Relaxed);
        let max_threads = num_cpus::get() * 2;

        if cpu_usage > 80.0 {
            // CPU高负载，减少线程数
            if current > 2 {
                let new_size = current / 2;
                self.thread_pool_size.store(new_size, Ordering::Relaxed);
                info!(
                    "Decreased thread pool size from {} to {} due to high CPU usage ({:.1}%)",
                    current, new_size, cpu_usage
                );
            }
        } else if cpu_usage < 30.0 {
            // CPU低负载，增加线程数
            if current < max_threads {
                let new_size = current * 2;
                let new_size = std::cmp::min(new_size, max_threads);
                self.thread_pool_size.store(new_size, Ordering::Relaxed);
                info!(
                    "Increased thread pool size from {} to {} due to low CPU usage ({:.1}%)",
                    current, new_size, cpu_usage
                );
            }
        }
    }

    /// 调整缓冲区池大小
    fn adjust_buffer_pool(&self, memory_usage: f64) {
        let current = self.buffer_pool_size.load(Ordering::Relaxed);

        if memory_usage > 0.8 {
            // 内存压力大，减少缓冲区
            if current > 4 {
                let new_size = current / 2;
                self.buffer_pool_size.store(new_size, Ordering::Relaxed);
                info!(
                    "Decreased buffer pool size from {} to {} due to high memory usage ({:.1}%)",
                    current,
                    new_size,
                    memory_usage * 100.0
                );
            }
        } else if memory_usage < 0.4 {
            // 内存充足，增加缓冲区
            let new_size = current * 2;
            let new_size = std::cmp::min(new_size, 256); // 上限256
            self.buffer_pool_size.store(new_size, Ordering::Relaxed);
            info!(
                "Increased buffer pool size from {} to {} due to low memory usage ({:.1}%)",
                current,
                new_size,
                memory_usage * 100.0
            );
        }
    }

    /// 调整连接限制
    fn adjust_connection_limit(&self, cpu_usage: f32, memory_usage: f64) {
        let current = self.connection_limit.load(Ordering::Relaxed);
        let max_connections = 50000; // 最大连接数上限

        if cpu_usage > 85.0 || memory_usage > 0.85 {
            // 高负载，减少连接数
            if current > 1000 {
                let new_size = (current as f64 * 0.8) as usize;
                self.connection_limit.store(new_size, Ordering::Relaxed);
                info!(
                    "Decreased connection limit from {} to {} due to high system load (CPU: {:.1}%, Memory: {:.1}%)",
                    current,
                    new_size,
                    cpu_usage,
                    memory_usage * 100.0
                );
            }
        } else if cpu_usage < 40.0 && memory_usage < 0.5 {
            // 低负载，增加连接数
            if current < max_connections {
                let new_size = (current as f64 * 1.2) as usize;
                let new_size = std::cmp::min(new_size, max_connections);
                self.connection_limit.store(new_size, Ordering::Relaxed);
                info!(
                    "Increased connection limit from {} to {} due to low system load (CPU: {:.1}%, Memory: {:.1}%)",
                    current,
                    new_size,
                    cpu_usage,
                    memory_usage * 100.0
                );
            }
        }
    }

    /// 应用内核调优
    pub fn apply_kernel_tuning(&self) -> Result<(), TunerError> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

            // 检查是否有root权限
            if !nix::unistd::getuid().is_root() {
                warn!("Skipping kernel tuning: root privileges required");
                return Err(TunerError::PermissionDenied);
            }

            info!("Applying kernel tuning parameters");

            // 调整TCP参数
            fs::write("/proc/sys/net/core/rmem_max", "134217728")?;
            fs::write("/proc/sys/net/core/wmem_max", "134217728")?;
            fs::write("/proc/sys/net/ipv4/tcp_rmem", "4096 87380 134217728")?;
            fs::write("/proc/sys/net/ipv4/tcp_wmem", "4096 65536 134217728")?;

            // 启用TCP快速打开
            fs::write("/proc/sys/net/ipv4/tcp_fastopen", "3")?;

            // 增加TCP最大连接数
            fs::write("/proc/sys/net/core/somaxconn", "65535")?;
            fs::write("/proc/sys/net/ipv4/tcp_max_syn_backlog", "65535")?;

            // 减少TCP超时时间
            fs::write("/proc/sys/net/ipv4/tcp_fin_timeout", "15")?;
            fs::write("/proc/sys/net/ipv4/tcp_keepalive_time", "300")?;
            fs::write("/proc/sys/net/ipv4/tcp_keepalive_probes", "3")?;
            fs::write("/proc/sys/net/ipv4/tcp_keepalive_intvl", "15")?;

            // 禁用透明大页（对内存数据库更好）
            fs::write("/sys/kernel/mm/transparent_hugepage/enabled", "never")?;
            fs::write("/sys/kernel/mm/transparent_hugepage/defrag", "never")?;

            // 增加文件描述符限制
            fs::write("/proc/sys/fs/file-max", "1000000")?;

            info!("Kernel tuning applied successfully");
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Kernel tuning is only supported on Linux");
            return Err(TunerError::Unsupported);
        }

        Ok(())
    }

    /// 获取当前调优参数
    pub fn get_current_parameters(&self) -> TuningParameters {
        TuningParameters {
            thread_pool_size: self.thread_pool_size.load(Ordering::Relaxed),
            buffer_pool_size: self.buffer_pool_size.load(Ordering::Relaxed),
            connection_limit: self.connection_limit.load(Ordering::Relaxed),
            tuning_active: self.tuning_active.load(Ordering::Relaxed),
            last_tuning_time: self.last_tuning_time.load(Ordering::Relaxed),
        }
    }

    /// 手动设置线程池大小
    pub fn set_thread_pool_size(&self, size: usize) {
        self.thread_pool_size.store(size, Ordering::Relaxed);
        info!("Manually set thread pool size to {}", size);
    }

    /// 手动设置缓冲区池大小
    pub fn set_buffer_pool_size(&self, size: usize) {
        self.buffer_pool_size.store(size, Ordering::Relaxed);
        info!("Manually set buffer pool size to {}", size);
    }

    /// 手动设置连接限制
    pub fn set_connection_limit(&self, limit: usize) {
        self.connection_limit.store(limit, Ordering::Relaxed);
        info!("Manually set connection limit to {}", limit);
    }
}

/// 调优参数
pub struct TuningParameters {
    pub thread_pool_size: usize,
    pub buffer_pool_size: usize,
    pub connection_limit: usize,
    pub tuning_active: bool,
    pub last_tuning_time: u64,
}
