use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::snapshot_loader;

pub struct SnapshotScheduler;

impl SnapshotScheduler {
    pub fn start(
        snapshot_dir: Option<String>,
        snapshot_type: Option<String>,
        snapshot_interval: Option<u64>,
        max_snapshots: Option<usize>,
    ) {
        let interval_secs = match snapshot_interval {
            Some(secs) if secs > 0 => secs,
            _ => return,
        };

        let snap_type = match snapshot_type {
            Some(ref t) if t.to_lowercase() == "full" || t.to_lowercase() == "incremental" => {
                t.to_lowercase()
            }
            _ => return,
        };

        let max_snapshots = max_snapshots.unwrap_or(10);

        info!(
            "Starting snapshot timer with interval {} seconds, type: {}",
            interval_secs, snap_type
        );

        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(interval_secs));

            loop {
                timer.tick().await;

                let db_opt = unsafe { remdb::get_global_db() };
                if let Some(mut db_guard) = db_opt {
                    let db = &mut *db_guard;
                    let start = std::time::Instant::now();

                    let result = if let Some(ref dir) = snapshot_dir {
                        if snap_type == "full" {
                            snapshot_loader::save_full_snapshot_to_dir(db, dir)
                        } else {
                            let res = snapshot_loader::save_incremental_snapshot_to_dir(db, dir);
                            if res.is_ok() {
                                let _ = snapshot_loader::cleanup_old_snapshots(dir, max_snapshots);
                            }
                            res
                        }
                    } else {
                        Err(remdb::RemDbError::FileIoError)
                    };

                    match result {
                        Ok(()) => {
                            let duration = start.elapsed();
                            let duration_ms = duration.as_secs_f64() * 1000.0;
                            info!(
                                "[Snapshot Timer] {} snapshot executed successfully in {:.2} ms",
                                snap_type, duration_ms
                            );
                        }
                        Err(e) => {
                            error!(
                                "[Snapshot Timer] Failed to execute {} snapshot: {:?}",
                                snap_type, e
                            );
                        }
                    }
                } else {
                    warn!("[Snapshot Timer] Database not available");
                }
            }
        });
    }
}
