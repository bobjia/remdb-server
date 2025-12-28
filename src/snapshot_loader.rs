use remdb::{RemDb, RemDbError, Result as RemResult};

/// 加载完整快照文件
pub fn load_snapshot(db: &mut RemDb, path: &str) -> RemResult<()> {
    db.restore_snapshot(path)
}

/// 保存增量快照文件
pub fn save_incremental_snapshot(db: &mut RemDb, path: &str) -> RemResult<()> {
    db.save_incremental_snapshot(path)
}

/// 获取快照版本信息
pub fn get_snapshot_version(db: &RemDb) -> u32 {
    db.snapshot_version
}
