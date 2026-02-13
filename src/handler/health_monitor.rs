use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

/// Server health status
#[derive(Debug, Clone, Copy)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

/// Server health monitor
pub struct ServerHealthMonitor {
    status: AtomicBool,
    error_count: AtomicU64,
    last_error_time: AtomicU64,
    consecutive_errors: AtomicU64,
    max_consecutive_errors: u64,
    recovery_mode: AtomicBool,
}

impl ServerHealthMonitor {
    pub fn new(max_consecutive_errors: u64) -> Self {
        Self {
            status: AtomicBool::new(true),
            error_count: AtomicU64::new(0),
            last_error_time: AtomicU64::new(0),
            consecutive_errors: AtomicU64::new(0),
            max_consecutive_errors,
            recovery_mode: AtomicBool::new(false),
        }
    }

    pub fn record_error(&self) -> HealthStatus {
        let error_count = self.error_count.fetch_add(1, Ordering::SeqCst);
        let consecutive = self.consecutive_errors.fetch_add(1, Ordering::SeqCst) + 1;
        let now = Instant::now().elapsed().as_secs() as u64;

        self.last_error_time.store(now, Ordering::SeqCst);

        if consecutive >= self.max_consecutive_errors {
            self.status.store(false, Ordering::SeqCst);
            self.recovery_mode.store(true, Ordering::SeqCst);
            return HealthStatus::Critical;
        } else if consecutive >= self.max_consecutive_errors / 2 {
            self.status.store(false, Ordering::SeqCst);
            return HealthStatus::Degraded;
        }

        HealthStatus::Healthy
    }

    pub fn record_success(&self) {
        self.consecutive_errors.store(0, Ordering::SeqCst);
        self.recovery_mode.store(false, Ordering::SeqCst);
    }

    pub fn get_status(&self) -> HealthStatus {
        if self.recovery_mode.load(Ordering::SeqCst) {
            return HealthStatus::Critical;
        }

        if self.status.load(Ordering::SeqCst) {
            return HealthStatus::Healthy;
        }

        HealthStatus::Degraded
    }

    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::SeqCst)
    }

    pub fn get_consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::SeqCst)
    }

    pub fn is_in_recovery_mode(&self) -> bool {
        self.recovery_mode.load(Ordering::SeqCst)
    }

    pub fn enter_recovery_mode(&self) {
        self.recovery_mode.store(true, Ordering::SeqCst);
    }

    pub fn exit_recovery_mode(&self) {
        self.recovery_mode.store(false, Ordering::SeqCst);
        self.consecutive_errors.store(0, Ordering::SeqCst);
    }
}
