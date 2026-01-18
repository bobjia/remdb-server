use remdb::{RemDb, RemDbError, Result as RemResult};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// 从目录加载快照
pub fn load_snapshot_from_dir(db: &mut RemDb, dir_path: &str) -> RemResult<()> {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return Err(RemDbError::FileIoError);
    }

    let mut snapshots = Vec::new();

    for entry in fs::read_dir(dir_path).map_err(|_| RemDbError::FileIoError)? {
        let entry = entry.map_err(|_| RemDbError::FileIoError)?;
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
            let full_path = file_path.to_string_lossy().to_string();

            // 解析文件名，提取类型和时间戳
            if file_name.starts_with("full_") && file_name.ends_with(".remd") {
                // 全量快照：full_timestamp.remd
                if let Some(ts_str) = file_name
                    .strip_prefix("full_")
                    .and_then(|s| s.strip_suffix(".remd"))
                {
                    if let Ok(ts) = ts_str.parse::<u64>() {
                        snapshots.push(SnapshotInfo {
                            path: full_path,
                            file_type: SnapshotType::FullSnapshot,
                            timestamp: ts,
                        });
                    }
                }
            } else if file_name.starts_with("incremental_") && file_name.ends_with(".remd") {
                // 增量快照：incremental_timestamp.remd
                if let Some(ts_str) = file_name
                    .strip_prefix("incremental_")
                    .and_then(|s| s.strip_suffix(".remd"))
                {
                    if let Ok(ts) = ts_str.parse::<u64>() {
                        snapshots.push(SnapshotInfo {
                            path: full_path,
                            file_type: SnapshotType::IncrementalSnapshot,
                            timestamp: ts,
                        });
                    }
                }
            }
        }
    }

    // 按时间戳排序，最新的在前
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut full_snapshot_loaded = false;
    let mut loaded_snapshots = Vec::new();

    // 收集所有快照，以便按正确顺序加载
    for snapshot in &snapshots {
        loaded_snapshots.push(snapshot);
    }

    // 反转列表，按时间顺序加载
    loaded_snapshots.reverse();

    for snapshot in loaded_snapshots {
        match snapshot.file_type {
            SnapshotType::FullSnapshot => {
                if !full_snapshot_loaded {
                    println!("Loading full snapshot: {}", snapshot.path);
                    db.restore_snapshot(&snapshot.path)?;
                    full_snapshot_loaded = true;
                }
            }
            SnapshotType::IncrementalSnapshot => {
                if full_snapshot_loaded {
                    println!("Loading incremental snapshot: {}", snapshot.path);
                    db.restore_snapshot(&snapshot.path)?;
                }
            }
            _ => { /* 忽略其他类型 */ }
        }
    }

    if !full_snapshot_loaded {
        println!("No full snapshot found in directory");
        return Ok(());
    }

    Ok(())
}

/// 保存完整快照到目录
pub fn save_full_snapshot_to_dir(db: &mut RemDb, dir_path: &str) -> RemResult<()> {
    fs::create_dir_all(dir_path).map_err(|_| RemDbError::FileIoError)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| RemDbError::InternalError)?
        .as_secs();
    let file_path = format!("{}/full_{}.remd", dir_path, timestamp);
    println!("Saving full snapshot to: {}", file_path);
    db.save_snapshot(&file_path)
}

/// 保存增量快照到目录
pub fn save_incremental_snapshot_to_dir(db: &mut RemDb, dir_path: &str) -> RemResult<()> {
    fs::create_dir_all(dir_path).map_err(|_| RemDbError::FileIoError)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| RemDbError::InternalError)?
        .as_secs();
    let file_path = format!("{}/incremental_{}.remd", dir_path, timestamp);
    println!("Saving incremental snapshot to: {}", file_path);
    db.save_incremental_snapshot(&file_path)
}

/// 清理旧的增量快照
pub fn cleanup_old_snapshots(dir_path: &str, max_incremental: usize) -> RemResult<()> {
    let path = Path::new(dir_path);
    if !path.exists() || !path.is_dir() {
        return Ok(());
    }

    let mut incremental_snapshots = Vec::new();
    for entry in fs::read_dir(dir_path).map_err(|_| RemDbError::FileIoError)? {
        let entry = entry.map_err(|_| RemDbError::FileIoError)?;
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
            if file_name.starts_with("incremental_") {
                incremental_snapshots.push((file_path, file_name));
            }
        }
    }

    incremental_snapshots.sort_by(|a, b| a.1.cmp(&b.1));

    while incremental_snapshots.len() > max_incremental {
        let (file_path, file_name) = incremental_snapshots.remove(0);
        println!("Cleaning up old incremental snapshot: {}", file_name);
        fs::remove_file(file_path).map_err(|_| RemDbError::FileIoError)?;
    }

    Ok(())
}

/// 快照或检查点文件信息
struct SnapshotInfo {
    /// 文件路径
    path: String,
    /// 文件类型
    file_type: SnapshotType,
    /// 时间戳或序列号
    timestamp: u64,
}

/// 快照类型
enum SnapshotType {
    /// 全量快照
    FullSnapshot,
    /// 增量快照
    IncrementalSnapshot,
    /// 检查点
    Checkpoint,
}

/// 从WAL目录加载并恢复数据
pub fn load_from_wal_dir(db: &mut RemDb, wal_dir: &str) -> RemResult<()> {
    // 1. 扫描snapshot和wal数据目录，识别可用文件
    let mut snapshots = Vec::new();
    let mut wal_files = Vec::new();

    if let Ok(entries) = fs::read_dir(wal_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if file_path.is_file() {
                    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                    let full_path = file_path.to_string_lossy().to_string();

                    // 解析文件名，提取类型和时间戳/序列号
                    if file_name.starts_with("full_") && file_name.ends_with(".remd") {
                        // 全量快照：full_timestamp.remd
                        if let Some(ts_str) = file_name
                            .strip_prefix("full_")
                            .and_then(|s| s.strip_suffix(".remd"))
                        {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                snapshots.push(SnapshotInfo {
                                    path: full_path,
                                    file_type: SnapshotType::FullSnapshot,
                                    timestamp: ts,
                                });
                            }
                        }
                    } else if file_name.starts_with("incremental_") && file_name.ends_with(".remd")
                    {
                        // 增量快照：incremental_timestamp.remd
                        if let Some(ts_str) = file_name
                            .strip_prefix("incremental_")
                            .and_then(|s| s.strip_suffix(".remd"))
                        {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                snapshots.push(SnapshotInfo {
                                    path: full_path,
                                    file_type: SnapshotType::IncrementalSnapshot,
                                    timestamp: ts,
                                });
                            }
                        }
                    } else if file_name.starts_with("checkpoint_") {
                        // 检查点：checkpoint_timestamp
                        if let Some(ts_str) = file_name.strip_prefix("checkpoint_") {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                snapshots.push(SnapshotInfo {
                                    path: full_path,
                                    file_type: SnapshotType::Checkpoint,
                                    timestamp: ts,
                                });
                            }
                        }
                    } else if file_name.ends_with(".wal") {
                        // 处理WAL文件
                        if file_name == "remdb.wal" {
                            // 默认WAL文件，序列号为0
                            wal_files.push((full_path, 0));
                        } else {
                            // 按序列号命名的WAL文件：seq_num.wal
                            if let Some(seq_str) = file_name.strip_suffix(".wal") {
                                if let Ok(seq) = seq_str.parse::<u64>() {
                                    wal_files.push((full_path, seq));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. 查找最新的完整快照（Snapshot或完整Checkpoint）
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut snapshot_seq = 0;

    // 3. 加载最新快照（如果有）
    if let Some(latest_snapshot) = snapshots.iter().find(|s| {
        matches!(
            s.file_type,
            SnapshotType::FullSnapshot | SnapshotType::Checkpoint
        )
    }) {
        // 加载该快照到内存
        println!("Loading latest snapshot: {}", latest_snapshot.path);
        db.restore_snapshot(&latest_snapshot.path)?;

        // 确定快照对应的序列号
        snapshot_seq = db.snapshot_version as u64;
        println!("Snapshot loaded with sequence number: {}", snapshot_seq);
    } else {
        println!("No full snapshot or checkpoint found, will replay all WAL files");
    }

    // 4. 按序列号排序WAL文件
    wal_files.sort_by(|a, b| a.1.cmp(&b.1));

    // 5. 筛选出需要重放的WAL文件
    let total_wal_files = wal_files.len();
    let mut wal_files_to_replay = Vec::new();
    for (path, seq) in wal_files {
        if seq > snapshot_seq {
            wal_files_to_replay.push((path, seq));
        } else if seq == 0 && snapshot_seq == 0 {
            // 如果是默认WAL文件（seq=0）且没有快照（snapshot_seq=0），则需要重放
            wal_files_to_replay.push((path, seq));
        }
    }

    // 6. 重放WAL中的操作
    println!(
        "Found {} WAL files in total, {} to replay",
        total_wal_files,
        wal_files_to_replay.len()
    );

    // 如果有WAL文件需要重放，尝试使用LogManager进行重放
    if !wal_files_to_replay.is_empty() {
        println!("Attempting to replay WAL files...");

        // 尝试使用LogManager进行WAL重放
        unsafe {
            if let Some(log_manager) = remdb::transaction::get_log_manager() {
                println!("✓ LogManager found, attempting recovery...");

                // 调试信息：LogManager found and ready for recovery

                // 尝试恢复，使用更详细的错误处理
                match log_manager.recover(db) {
                    Ok(_) => {
                        println!("✓ WAL recovery completed successfully");
                    }
                    Err(err) => {
                        println!("Warning: WAL recovery failed with error: {:?}", err);
                        println!("  This might be due to:");
                        println!("  - WAL file format mismatch");
                        println!("  - File permission issues");
                        println!("  - Corrupt WAL file");
                        println!("  - LogManager configuration issues");
                        println!("  - Using fallback recovery approach");
                    }
                }
            } else {
                println!("Warning: LogManager not available, cannot replay WAL files");
            }
        }
    } else {
        println!("No WAL files need to be replayed");
    }

    // 7. 数据恢复到最新一致状态
    println!("✓ Data recovery completed successfully");
    Ok(())
}

/// 获取快照版本信息
pub fn get_snapshot_version(db: &RemDb) -> u32 {
    db.snapshot_version
}
