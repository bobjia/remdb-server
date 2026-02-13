use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct CheckpointScheduler;

impl CheckpointScheduler {
    pub fn start(checkpoint_interval_ms: u64) {
        if checkpoint_interval_ms == 0 {
            return;
        }

        info!(
            "Starting checkpoint timer with interval {} ms",
            checkpoint_interval_ms
        );

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_millis(checkpoint_interval_ms));

            loop {
                timer.tick().await;

                let log_manager_opt = unsafe { remdb::transaction::get_log_manager() };
                if let Some(log_manager) = log_manager_opt {
                    let start = std::time::Instant::now();

                    match unsafe { log_manager.check_flush_and_checkpoint() } {
                        Ok(()) => {
                            let duration = start.elapsed();
                            let duration_ms = duration.as_secs_f64() * 1000.0;
                            info!(
                                "[Checkpoint Timer] Checkpoint executed successfully in {:.2} ms",
                                duration_ms
                            );
                        }
                        Err(e) => {
                            error!("[Checkpoint Timer] Failed to execute checkpoint: {:?}", e);
                        }
                    }
                } else {
                    warn!("[Checkpoint Timer] LogManager not available");
                }
            }
        });
    }
}
