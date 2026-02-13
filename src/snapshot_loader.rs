use remdb::log::{error, info, warn};
use remdb::platform;
use remdb::{RemDb, RemDbError, Result as RemResult, TableDef};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

struct Defer<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Defer<F> {
    fn new(f: F) -> Self {
        Defer(Some(f))
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}

/// Load table definitions from snapshot file (without record data)
/// This function is used to restore table structure before database initialization
pub fn load_table_defs_from_snapshot(snapshot_path: &str) -> RemResult<Vec<TableDef>> {
    let path = Path::new(snapshot_path);
    if !path.exists() || !path.is_file() {
        return Err(RemDbError::FileIoError);
    }

    let handle = platform::file_open(snapshot_path, platform::FileMode::Read)
        .map_err(|_| RemDbError::FileIoError)?;

    let _defer = Defer::new(|| {
        let _ = platform::file_close(handle);
    });

    // Read magic number
    let mut magic_bytes = [0u8; 4];
    let read = platform::file_read(handle, magic_bytes.as_mut_ptr(), magic_bytes.len())
        .map_err(|_| RemDbError::FileIoError)?;
    if read != magic_bytes.len() {
        return Err(RemDbError::FileIoError);
    }
    let magic = u32::from_le_bytes(magic_bytes);
    if magic != 0x52454D44 {
        return Err(RemDbError::SnapshotFormatError);
    }

    // Read version number
    let mut version_bytes = [0u8; 4];
    let read = platform::file_read(handle, version_bytes.as_mut_ptr(), version_bytes.len())
        .map_err(|_| RemDbError::FileIoError)?;
    if read != version_bytes.len() {
        return Err(RemDbError::FileIoError);
    }
    let version = u32::from_le_bytes(version_bytes);
    if version != 1 {
        return Err(RemDbError::SnapshotFormatError);
    }

    // Read snapshot type
    let mut snapshot_type_bytes = [0u8; 1];
    let read = platform::file_read(
        handle,
        snapshot_type_bytes.as_mut_ptr(),
        snapshot_type_bytes.len(),
    )
    .map_err(|_| RemDbError::FileIoError)?;
    if read != snapshot_type_bytes.len() {
        return Err(RemDbError::FileIoError);
    }

    // Skip base version number
    let mut base_version_bytes = [0u8; 4];
    let read = platform::file_read(
        handle,
        base_version_bytes.as_mut_ptr(),
        base_version_bytes.len(),
    )
    .map_err(|_| RemDbError::FileIoError)?;
    if read != base_version_bytes.len() {
        return Err(RemDbError::FileIoError);
    }

    // Read table count
    let mut table_count_bytes = [0u8; 4];
    let read = platform::file_read(
        handle,
        table_count_bytes.as_mut_ptr(),
        table_count_bytes.len(),
    )
    .map_err(|_| RemDbError::FileIoError)?;
    if read != table_count_bytes.len() {
        return Err(RemDbError::FileIoError);
    }
    let table_count = u32::from_le_bytes(table_count_bytes) as usize;

    let mut table_defs = Vec::new();

    // Read each table definition
    for _ in 0..table_count {
        // Read table ID
        let mut table_id_bytes = [0u8; 4];
        let read = platform::file_read(handle, table_id_bytes.as_mut_ptr(), table_id_bytes.len())
            .map_err(|_| RemDbError::FileIoError)?;
        if read != table_id_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let table_id = u32::from_le_bytes(table_id_bytes) as u8;

        // Read table name
        let mut table_name_len_bytes = [0u8; 1];
        let read = platform::file_read(
            handle,
            table_name_len_bytes.as_mut_ptr(),
            table_name_len_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != table_name_len_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let table_name_len = table_name_len_bytes[0] as usize;

        let mut table_name_bytes = vec![0u8; table_name_len];
        let read = platform::file_read(handle, table_name_bytes.as_mut_ptr(), table_name_len)
            .map_err(|_| RemDbError::FileIoError)?;
        if read != table_name_len {
            return Err(RemDbError::FileIoError);
        }
        let table_name = String::from_utf8_lossy(&table_name_bytes).to_string();

        // Read field count
        let mut field_count_bytes = [0u8; 1];
        let read = platform::file_read(
            handle,
            field_count_bytes.as_mut_ptr(),
            field_count_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != field_count_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let field_count = field_count_bytes[0] as usize;

        // Read primary key field count
        let mut primary_key_count_bytes = [0u8; 1];
        let read = platform::file_read(
            handle,
            primary_key_count_bytes.as_mut_ptr(),
            primary_key_count_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != primary_key_count_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let primary_key_count = primary_key_count_bytes[0] as usize;

        // Read secondary index field count
        let mut secondary_index_count_bytes = [0u8; 1];
        let read = platform::file_read(
            handle,
            secondary_index_count_bytes.as_mut_ptr(),
            secondary_index_count_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != secondary_index_count_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let secondary_index_count = secondary_index_count_bytes[0] as usize;

        // Read secondary index type
        let mut secondary_index_type_bytes = [0u8; 1];
        let read = platform::file_read(
            handle,
            secondary_index_type_bytes.as_mut_ptr(),
            secondary_index_type_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != secondary_index_type_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let secondary_index_type = secondary_index_type_bytes[0];

        // Read max records
        let mut max_records_bytes = [0u8; 4];
        let read = platform::file_read(
            handle,
            max_records_bytes.as_mut_ptr(),
            max_records_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != max_records_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let max_records = u32::from_le_bytes(max_records_bytes) as usize;

        // Read field definitions
        let mut fields = Vec::new();
        for _ in 0..field_count {
            // Read field name
            let mut field_name_len_bytes = [0u8; 1];
            let read = platform::file_read(
                handle,
                field_name_len_bytes.as_mut_ptr(),
                field_name_len_bytes.len(),
            )
            .map_err(|_| RemDbError::FileIoError)?;
            if read != field_name_len_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            let field_name_len = field_name_len_bytes[0] as usize;

            let mut field_name_bytes = vec![0u8; field_name_len];
            let read = platform::file_read(handle, field_name_bytes.as_mut_ptr(), field_name_len)
                .map_err(|_| RemDbError::FileIoError)?;
            if read != field_name_len {
                return Err(RemDbError::FileIoError);
            }
            let field_name = String::from_utf8_lossy(&field_name_bytes).to_string();

            // Read data type
            let mut data_type_bytes = [0u8; 1];
            let read =
                platform::file_read(handle, data_type_bytes.as_mut_ptr(), data_type_bytes.len())
                    .map_err(|_| RemDbError::FileIoError)?;
            if read != data_type_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            let data_type = remdb::types::DataType::from(data_type_bytes[0]);

            // Read field size
            let mut field_size_bytes = [0u8; 4];
            let read = platform::file_read(
                handle,
                field_size_bytes.as_mut_ptr(),
                field_size_bytes.len(),
            )
            .map_err(|_| RemDbError::FileIoError)?;
            if read != field_size_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            let field_size = u32::from_le_bytes(field_size_bytes) as usize;

            // Read string length limit
            let mut has_string_length_bytes = [0u8; 1];
            let read = platform::file_read(
                handle,
                has_string_length_bytes.as_mut_ptr(),
                has_string_length_bytes.len(),
            )
            .map_err(|_| RemDbError::FileIoError)?;
            if read != has_string_length_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            let string_length = if has_string_length_bytes[0] == 1 {
                let mut string_len_bytes = [0u8; 4];
                let read = platform::file_read(
                    handle,
                    string_len_bytes.as_mut_ptr(),
                    string_len_bytes.len(),
                )
                .map_err(|_| RemDbError::FileIoError)?;
                if read != string_len_bytes.len() {
                    return Err(RemDbError::FileIoError);
                }
                Some(u32::from_le_bytes(string_len_bytes) as usize)
            } else {
                None
            };

            // Read field flags
            let mut flags_bytes = [0u8; 1];
            let read = platform::file_read(handle, flags_bytes.as_mut_ptr(), flags_bytes.len())
                .map_err(|_| RemDbError::FileIoError)?;
            if read != flags_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            let flags = flags_bytes[0];
            let primary_key = (flags & 0x01) != 0;
            let not_null = (flags & 0x02) != 0;
            let unique = (flags & 0x04) != 0;
            let auto_increment = (flags & 0x08) != 0;

            // Read default value flag (default values not supported yet)
            let mut has_default_bytes = [0u8; 1];
            let read = platform::file_read(
                handle,
                has_default_bytes.as_mut_ptr(),
                has_default_bytes.len(),
            )
            .map_err(|_| RemDbError::FileIoError)?;
            if read != has_default_bytes.len() {
                return Err(RemDbError::FileIoError);
            }

            fields.push(remdb::types::FieldDef {
                name: field_name,
                data_type,
                size: field_size,
                string_length,
                offset: 0,
                primary_key,
                not_null,
                unique,
                auto_increment,
                default_value: None,
                vector_metadata: None,
                json_metadata: None,
            });
        }

        // Read primary key field indices
        let mut primary_key_indices = Vec::new();
        for _ in 0..primary_key_count {
            let mut pk_idx_bytes = [0u8; 1];
            let read = platform::file_read(handle, pk_idx_bytes.as_mut_ptr(), pk_idx_bytes.len())
                .map_err(|_| RemDbError::FileIoError)?;
            if read != pk_idx_bytes.len() {
                return Err(RemDbError::FileIoError);
            }
            primary_key_indices.push(pk_idx_bytes[0] as usize);
        }

        // Read secondary index field indices
        let mut secondary_index_indices = if secondary_index_count > 0 {
            let mut indices = Vec::new();
            for _ in 0..secondary_index_count {
                let mut idx_bytes = [0u8; 1];
                let read = platform::file_read(handle, idx_bytes.as_mut_ptr(), idx_bytes.len())
                    .map_err(|_| RemDbError::FileIoError)?;
                if read != idx_bytes.len() {
                    return Err(RemDbError::FileIoError);
                }
                indices.push(idx_bytes[0] as usize);
            }
            Some(indices)
        } else {
            None
        };

        // Create table definition
        table_defs.push(TableDef {
            id: table_id,
            name: table_name,
            fields,
            primary_key: primary_key_indices,
            secondary_index: secondary_index_indices,
            secondary_index_type: remdb::types::IndexType::from(secondary_index_type),
            record_size: 0,
            max_records,
            version: 0,
            created_at: 0,
            updated_at: 0,
        });

        // Skip record data (only read table definitions)
        let mut record_count_bytes = [0u8; 4];
        let read = platform::file_read(
            handle,
            record_count_bytes.as_mut_ptr(),
            record_count_bytes.len(),
        )
        .map_err(|_| RemDbError::FileIoError)?;
        if read != record_count_bytes.len() {
            return Err(RemDbError::FileIoError);
        }
        let record_count = u32::from_le_bytes(record_count_bytes) as usize;

        // Calculate record size
        let mut record_size = 0;
        for field in &table_defs.last().unwrap().fields {
            record_size += field.size;
        }

        // Skip all record data
        for _ in 0..record_count {
            // Skip record index
            let mut index_bytes = [0u8; 4];
            let _ = platform::file_read(handle, index_bytes.as_mut_ptr(), index_bytes.len());
            // Skip record data
            let mut dummy_record = vec![0u8; record_size];
            let _ = platform::file_read(handle, dummy_record.as_mut_ptr(), record_size);
        }
    }

    Ok(table_defs)
}

/// Load table definitions from directory by finding the latest snapshot
pub fn load_table_defs_from_dir(dir_path: &str) -> RemResult<Vec<TableDef>> {
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

            // Parse file name to extract type and timestamp
            if file_name.starts_with("full_") && file_name.ends_with(".remd") {
                // Full snapshot: full_timestamp.remd
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
            }
        }
    }

    // Sort by timestamp, newest first
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Find the latest full snapshot
    if let Some(latest_snapshot) = snapshots
        .iter()
        .find(|s| matches!(s.file_type, SnapshotType::FullSnapshot))
    {
        info!(
            "Loading table definitions from latest snapshot: {}",
            latest_snapshot.path
        );
        load_table_defs_from_snapshot(&latest_snapshot.path)
    } else {
        warn!("No full snapshot found in directory");
        Err(RemDbError::FileIoError)
    }
}

/// Load snapshot from directory
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

            // Parse file name to extract type and timestamp
            if file_name.starts_with("full_") && file_name.ends_with(".remd") {
                // Full snapshot: full_timestamp.remd
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
                // Incremental snapshot: incremental_timestamp.remd
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

    // Sort by timestamp, newest first
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut full_snapshot_loaded = false;
    let mut loaded_snapshots = Vec::new();

    // Collect all snapshots to load in correct order
    for snapshot in &snapshots {
        loaded_snapshots.push(snapshot);
    }

    // Reverse list to load in chronological order
    loaded_snapshots.reverse();

    for snapshot in loaded_snapshots {
        match snapshot.file_type {
            SnapshotType::FullSnapshot => {
                if !full_snapshot_loaded {
                    info!("Loading full snapshot: {}", snapshot.path);
                    db.restore_snapshot(&snapshot.path)?;
                    full_snapshot_loaded = true;
                }
            }
            SnapshotType::IncrementalSnapshot => {
                if full_snapshot_loaded {
                    info!("Loading incremental snapshot: {}", snapshot.path);
                    db.restore_snapshot(&snapshot.path)?;
                }
            }
            _ => { /* Ignore other types */ }
        }
    }

    if !full_snapshot_loaded {
        warn!("No full snapshot found in directory");
        return Ok(());
    }

    Ok(())
}

/// Save full snapshot to directory
pub fn save_full_snapshot_to_dir(db: &mut RemDb, dir_path: &str) -> RemResult<()> {
    fs::create_dir_all(dir_path).map_err(|_| RemDbError::FileIoError)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| RemDbError::InternalError)?
        .as_secs();
    let file_path = format!("{}/full_{}.remd", dir_path, timestamp);
    info!("Saving full snapshot to: {}", file_path);
    db.save_snapshot(&file_path)
}

/// Save incremental snapshot to directory
pub fn save_incremental_snapshot_to_dir(db: &mut RemDb, dir_path: &str) -> RemResult<()> {
    fs::create_dir_all(dir_path).map_err(|_| RemDbError::FileIoError)?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| RemDbError::InternalError)?
        .as_secs();
    let file_path = format!("{}/incremental_{}.remd", dir_path, timestamp);
    info!("Saving incremental snapshot to: {}", file_path);
    db.save_incremental_snapshot(&file_path)
}

/// Clean up old incremental snapshots
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
        info!("Cleaning up old incremental snapshot: {}", file_name);
        fs::remove_file(file_path).map_err(|_| RemDbError::FileIoError)?;
    }

    Ok(())
}

/// Snapshot or checkpoint file info
struct SnapshotInfo {
    /// File path
    path: String,
    /// File type
    file_type: SnapshotType,
    /// Timestamp or sequence number
    timestamp: u64,
}

/// Snapshot type
enum SnapshotType {
    /// Full snapshot
    FullSnapshot,
    /// Incremental snapshot
    IncrementalSnapshot,
    /// Checkpoint
    Checkpoint,
}

/// Load and recover data from WAL directory
pub fn load_from_wal_dir(db: &mut RemDb, wal_dir: &str) -> RemResult<()> {
    // 1. Scan snapshot and wal data directories to identify available files
    let mut snapshots = Vec::new();
    let mut wal_files = Vec::new();

    if let Ok(entries) = fs::read_dir(wal_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if file_path.is_file() {
                    let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
                    let full_path = file_path.to_string_lossy().to_string();

                    // Parse file name to extract type and timestamp/sequence number
                    if file_name.starts_with("full_") && file_name.ends_with(".remd") {
                        // Full snapshot: full_timestamp.remd
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
                        // Incremental snapshot: incremental_timestamp.remd
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
                        // Checkpoint: checkpoint_timestamp
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
                        // Handle WAL files
                        if file_name == "remdb.wal" {
                            // Default WAL file, sequence number is 0
                            wal_files.push((full_path, 0));
                        } else {
                            // WAL files named by sequence number: seq_num.wal
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

    // 2. Find the latest complete snapshot (Snapshot or complete Checkpoint)
    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let mut snapshot_seq = 0;

    // 3. Load the latest snapshot (if available)
    if let Some(latest_snapshot) = snapshots.iter().find(|s| {
        matches!(
            s.file_type,
            SnapshotType::FullSnapshot | SnapshotType::Checkpoint
        )
    }) {
        // Load that snapshot into memory
        info!("Loading latest snapshot: {}", latest_snapshot.path);
        db.restore_snapshot(&latest_snapshot.path)?;

        // Determine the snapshot's corresponding sequence number
        snapshot_seq = db.snapshot_version as u64;
        info!("Snapshot loaded with sequence number: {}", snapshot_seq);
    } else {
        warn!("No full snapshot or checkpoint found, will replay all WAL files");
    }

    // 4. Sort WAL files by sequence number
    wal_files.sort_by(|a, b| a.1.cmp(&b.1));

    // 5. Filter WAL files that need to be replayed
    let total_wal_files = wal_files.len();
    let mut wal_files_to_replay = Vec::new();
    for (path, seq) in wal_files {
        if seq > snapshot_seq {
            wal_files_to_replay.push((path, seq));
        } else if seq == 0 && snapshot_seq == 0 {
            // If it's the default WAL file (seq=0) and no snapshot (snapshot_seq=0), need to replay
            wal_files_to_replay.push((path, seq));
        }
    }

    // 6. Replay WAL operations
    info!(
        "Found {} WAL files in total, {} to replay",
        total_wal_files,
        wal_files_to_replay.len()
    );

    // If there are WAL files to replay, try using LogManager for replay
    if !wal_files_to_replay.is_empty() {
        info!("Attempting to replay WAL files...");

        // Validate WAL files before replay
        let mut valid_wal_files = Vec::new();
        for (path, seq) in &wal_files_to_replay {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > 0 {
                    info!(
                        "Validating WAL file: {} (size: {} bytes)",
                        path,
                        metadata.len()
                    );
                    valid_wal_files.push((path.clone(), *seq));
                } else {
                    warn!("Skipping empty WAL file: {}", path);
                }
            } else {
                warn!("Failed to read WAL file metadata: {}, skipping", path);
            }
        }

        if valid_wal_files.is_empty() {
            warn!("No valid WAL files found, skipping WAL replay");
            return Ok(());
        }

        // Try using LogManager for WAL replay with validated files
        unsafe {
            if let Some(log_manager) = remdb::transaction::get_log_manager() {
                info!("LogManager found, attempting recovery...");

                // Try recovery with more detailed error handling
                match log_manager.recover(db) {
                    Ok(_) => {
                        info!("WAL recovery completed successfully");
                        // WAL恢复成功，不需要检查表数量
                        // 因为表可能是从DDL文件创建的，不是从WAL恢复的
                    }
                    Err(err) => {
                        error!("Critical: WAL recovery failed with error: {:?}", err);
                        error!("This may indicate data corruption or incomplete transactions");
                        error!("Please check WAL files in directory: {}", wal_dir);

                        // Try to continue with partial recovery
                        warn!("Attempting to continue with partial data recovery...");

                        // Return RemDbError directly
                        return Err(err);
                    }
                }
            } else {
                warn!("Warning: LogManager not available, cannot replay WAL files");
            }
        }
    } else {
        info!("No WAL files need to be replayed");
    }

    // 7. Data recovered to latest consistent state
    info!("Data recovery completed successfully");
    Ok(())
}

/// Get snapshot version info
pub fn get_snapshot_version(db: &RemDb) -> u32 {
    db.snapshot_version
}
