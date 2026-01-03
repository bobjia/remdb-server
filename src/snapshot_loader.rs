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

    let mut full_snapshots = Vec::new();
    let mut incremental_snapshots = Vec::new();

    for entry in fs::read_dir(dir_path).map_err(|_| RemDbError::FileIoError)? {
        let entry = entry.map_err(|_| RemDbError::FileIoError)?;
        let file_path = entry.path();
        if file_path.is_file() {
            let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
            if file_name.starts_with("full_") {
                full_snapshots.push(file_name);
            } else if file_name.starts_with("incremental_") {
                incremental_snapshots.push(file_name);
            }
        }
    }

    full_snapshots.sort();
    incremental_snapshots.sort();

    if let Some(latest_full) = full_snapshots.last() {
        let full_path = format!("{}/{}", dir_path, latest_full);
        println!("Loading latest full snapshot: {}", full_path);
        db.restore_snapshot(&full_path)?;
    } else {
        println!("No full snapshot found in directory");
        return Ok(());
    }

    for incremental in &incremental_snapshots {
        let incremental_path = format!("{}/{}", dir_path, incremental);
        println!("Loading incremental snapshot: {}", incremental_path);
        db.restore_snapshot(&incremental_path)?;
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

/// 获取快照版本信息
pub fn get_snapshot_version(db: &RemDb) -> u32 {
    db.snapshot_version
}
