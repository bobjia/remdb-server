use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct WALConfig {
    pub log_path: Option<String>,
    pub log_mode: Option<String>,
    pub checkpoint_interval_ms: Option<u64>,
    pub log_file_size_limit: Option<usize>,
    pub log_prealloc_size: Option<usize>,
    pub log_segment_size: Option<usize>,
    pub retained_checkpoints: Option<usize>,
    pub max_consecutive_invalid: Option<u32>,
    pub skip_threshold: Option<u32>,
    pub skip_block_size: Option<usize>,
    pub max_skip_attempts: Option<u32>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct HaConfig {
    pub enabled: Option<bool>,
    pub node_id: Option<String>,
    pub role: Option<String>,
    pub replication_mode: Option<String>,
    pub heartbeat_interval: Option<u64>,
    pub failure_detection_ms: Option<u64>,
    pub sync_timeout_ms: Option<u64>,
    pub master_address: Option<String>,
    pub master_port: Option<u16>,
    pub replication_port: Option<u16>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PubSubConfig {
    pub enabled: Option<bool>,
    pub udp_bind_address: Option<String>,
    pub heartbeat_interval: Option<u32>,
    pub retransmission_timeout: Option<u32>,
    pub max_retransmissions: Option<u32>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct JdbcConfig {
    pub port: Option<u16>,
    pub enabled: Option<bool>,
    pub max_connections: Option<usize>,
    pub timeout: Option<u64>,
    pub auth_enabled: Option<bool>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct SnapshotConfig {
    pub dir: Option<String>,
    pub interval: Option<u64>,
    pub snapshot_type: Option<String>,
    pub max_incremental_snapshots: Option<usize>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub snapshot_dir: Option<String>,
    pub total_memory: Option<usize>,
    pub default_max_records: Option<usize>,
    pub low_power_mode_supported: Option<bool>,
    pub low_power_max_records: Option<usize>,
    pub log_path: Option<String>,
    pub snapshot_interval: Option<u64>,
    pub snapshot_type: Option<String>,
    pub max_incremental_snapshots: Option<usize>,
    pub debug: Option<bool>,
    pub jdbc_port: Option<u16>,
    pub jdbc_enabled: Option<bool>,
    pub max_connections: Option<usize>,
    pub jdbc_timeout: Option<u64>,
    pub jdbc_auth_enabled: Option<bool>,
    pub jdbc_username: Option<String>,
    pub jdbc_password_hash: Option<String>,
    pub ddl_path: Option<String>,
    pub pubsub: Option<PubSubConfig>,
    pub wal: Option<WALConfig>,
    pub ha: Option<HaConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> crate::error::ServerResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn jdbc_config(&self) -> JdbcConfig {
        JdbcConfig {
            port: self.jdbc_port,
            enabled: self.jdbc_enabled,
            max_connections: self.max_connections,
            timeout: self.jdbc_timeout,
            auth_enabled: self.jdbc_auth_enabled,
            username: self.jdbc_username.clone(),
            password_hash: self.jdbc_password_hash.clone(),
        }
    }

    pub fn snapshot_config(&self) -> SnapshotConfig {
        SnapshotConfig {
            dir: self.snapshot_dir.clone(),
            interval: self.snapshot_interval,
            snapshot_type: self.snapshot_type.clone(),
            max_incremental_snapshots: self.max_incremental_snapshots,
        }
    }

    pub fn wal_log_path(&self) -> Option<&str> {
        self.wal
            .as_ref()
            .and_then(|w| w.log_path.as_deref())
            .or(self.log_path.as_deref())
    }

    pub fn ha_enabled(&self) -> bool {
        self.ha.as_ref().and_then(|h| h.enabled).unwrap_or(false)
    }

    pub fn pubsub_enabled(&self) -> bool {
        self.pubsub
            .as_ref()
            .and_then(|p| p.enabled)
            .unwrap_or(false)
    }
}

pub struct RuntimeConfig {
    pub snapshot_dir: Option<String>,
    pub full_image: Option<String>,
    pub total_memory: usize,
    pub default_max_records: usize,
    pub low_power_mode_supported: bool,
    pub low_power_max_records: Option<usize>,
    pub log_path: Option<String>,
    pub log_file_name: String,
    pub snapshot_interval: Option<u64>,
    pub snapshot_type: Option<String>,
    pub max_incremental_snapshots: Option<usize>,
    pub debug_mode: bool,
    pub jdbc: JdbcConfig,
    pub pubsub: PubSubConfig,
    pub ha: HaConfig,
    pub wal: WALConfig,
    pub ddl_path: Option<String>,
}

impl RuntimeConfig {
    pub fn default_log_file_name(&self) -> String {
        let log_file_path = "./logs";
        let _ = std::fs::create_dir_all(log_file_path);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("{}/remdb-server-{}.log", log_file_path, timestamp)
    }

    pub fn wal_directory(&self) -> String {
        if let Some(ref log_path) = self.log_path {
            let log_file = PathBuf::from(log_path);
            if let Some(parent) = log_file.parent() {
                parent.to_str().unwrap_or("./wal").to_string()
            } else {
                "./wal".to_string()
            }
        } else {
            "./wal".to_string()
        }
    }
}

pub mod loader;

#[cfg(test)]
mod tests;
