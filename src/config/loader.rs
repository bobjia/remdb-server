use crate::config::{Config, HaConfig, JdbcConfig, PubSubConfig, RuntimeConfig, WALConfig};
use crate::error::{ServerError, ServerResult};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long, short)]
    pub config: Option<String>,

    #[arg(long)]
    pub snapshot_dir: Option<String>,

    #[arg(long)]
    pub full_image: Option<String>,

    #[arg(long)]
    pub total_memory: Option<usize>,

    #[arg(long)]
    pub default_max_records: Option<usize>,

    #[arg(long)]
    pub low_power_mode_supported: Option<bool>,

    #[arg(long)]
    pub low_power_max_records: Option<usize>,

    #[arg(long)]
    pub log_path: Option<String>,

    #[arg(long)]
    pub snapshot_interval: Option<u64>,

    #[arg(long)]
    pub snapshot_type: Option<String>,

    #[arg(long)]
    pub max_incremental_snapshots: Option<usize>,

    #[arg(long, short)]
    pub debug: bool,

    #[arg(long)]
    pub non_interactive: bool,

    #[arg(long)]
    pub test_export: bool,

    #[arg(long)]
    pub jdbc_port: Option<u16>,

    #[arg(long)]
    pub jdbc_enabled: Option<bool>,

    #[arg(long)]
    pub max_connections: Option<usize>,

    #[arg(long)]
    pub jdbc_timeout: Option<u64>,

    #[arg(long)]
    pub jdbc_auth_enabled: Option<bool>,

    #[arg(long)]
    pub jdbc_username: Option<String>,

    #[arg(long)]
    pub jdbc_password_hash: Option<String>,

    #[arg(long)]
    pub pubsub_enabled: Option<bool>,

    #[arg(long)]
    pub pubsub_udp_bind: Option<String>,

    #[arg(long)]
    pub pubsub_heartbeat: Option<u32>,

    #[arg(long)]
    pub pubsub_retrans_timeout: Option<u32>,

    #[arg(long)]
    pub pubsub_max_retrans: Option<u32>,

    #[arg(long)]
    pub ha_enabled: Option<bool>,

    #[arg(long)]
    pub ha_role: Option<String>,

    #[arg(long)]
    pub ha_replication_mode: Option<String>,

    #[arg(long)]
    pub ha_heartbeat_interval: Option<u64>,

    #[arg(long)]
    pub ha_failure_detection_ms: Option<u64>,

    #[arg(long)]
    pub ha_sync_timeout_ms: Option<u64>,

    #[arg(long)]
    pub ha_master_address: Option<String>,

    #[arg(long)]
    pub ha_master_port: Option<u16>,

    #[arg(long)]
    pub ha_replication_port: Option<u16>,

    #[arg(long)]
    pub ha_heartbeat_port: Option<u16>,

    #[arg(long)]
    pub ha_node_id: Option<String>,

    #[arg(long, default_value_t = 19530)]
    pub milvus_port: u16,

    #[arg(long)]
    pub milvus_api_key: Option<String>,

    #[arg(long)]
    pub milvus_enabled: Option<bool>,
}

#[derive(Parser, Debug)]
pub enum Command {
    Benchmark {
        #[arg(long, default_value = "100000")]
        query_count: usize,

        #[arg(long, default_value = "16")]
        connections: usize,

        #[arg(long, default_value = "SELECT * FROM test_table WHERE id = {}")]
        query_template: String,

        #[arg(long, default_value = "jdbc:remdb://localhost:6666")]
        server_url: String,

        #[arg(long, default_value = "query")]
        test_type: String,

        #[arg(
            long,
            default_value = "INSERT INTO test_table (id, value) VALUES ({}, {}) ON DUPLICATE KEY UPDATE value = {}"
        )]
        write_template: String,

        #[arg(long, default_value = "8:2")]
        read_write_ratio: String,
    },
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load() -> ServerResult<(Args, Config)> {
        let args = Args::parse();
        let config_path = args
            .config
            .clone()
            .unwrap_or_else(|| "./remdb-master.toml".to_string());

        let config = match Config::from_file(&config_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to load config file {}: {:?}", config_path, e);
                Config::default()
            }
        };

        Ok((args, config))
    }

    pub fn merge_args_and_config(args: Args, config: Config) -> RuntimeConfig {
        let debug_mode = args.debug || config.debug.unwrap_or(false);

        let jdbc = JdbcConfig {
            port: args.jdbc_port.or(config.jdbc_port),
            enabled: args.jdbc_enabled.or(config.jdbc_enabled),
            max_connections: args.max_connections.or(config.max_connections),
            timeout: args.jdbc_timeout.or(config.jdbc_timeout),
            auth_enabled: args.jdbc_auth_enabled.or(config.jdbc_auth_enabled),
            username: args.jdbc_username.or(config.jdbc_username),
            password_hash: args.jdbc_password_hash.or(config.jdbc_password_hash),
        };

        let pubsub = PubSubConfig {
            enabled: args
                .pubsub_enabled
                .or(config.pubsub.as_ref().and_then(|p| p.enabled)),
            udp_bind_address: args.pubsub_udp_bind.or(config
                .pubsub
                .as_ref()
                .and_then(|p| p.udp_bind_address.clone())),
            heartbeat_interval: args
                .pubsub_heartbeat
                .or(config.pubsub.as_ref().and_then(|p| p.heartbeat_interval)),
            retransmission_timeout: args.pubsub_retrans_timeout.or(config
                .pubsub
                .as_ref()
                .and_then(|p| p.retransmission_timeout)),
            max_retransmissions: args
                .pubsub_max_retrans
                .or(config.pubsub.as_ref().and_then(|p| p.max_retransmissions)),
        };

        let ha = HaConfig {
            enabled: args
                .ha_enabled
                .or(config.ha.as_ref().and_then(|h| h.enabled)),
            node_id: args
                .ha_node_id
                .or(config.ha.as_ref().and_then(|h| h.node_id.clone())),
            role: args
                .ha_role
                .or(config.ha.as_ref().and_then(|h| h.role.clone())),
            replication_mode: args
                .ha_replication_mode
                .or(config.ha.as_ref().and_then(|h| h.replication_mode.clone())),
            heartbeat_interval: args
                .ha_heartbeat_interval
                .or(config.ha.as_ref().and_then(|h| h.heartbeat_interval)),
            failure_detection_ms: args
                .ha_failure_detection_ms
                .or(config.ha.as_ref().and_then(|h| h.failure_detection_ms)),
            sync_timeout_ms: args
                .ha_sync_timeout_ms
                .or(config.ha.as_ref().and_then(|h| h.sync_timeout_ms)),
            master_address: args
                .ha_master_address
                .or(config.ha.as_ref().and_then(|h| h.master_address.clone())),
            master_port: args
                .ha_master_port
                .or(config.ha.as_ref().and_then(|h| h.master_port)),
            replication_port: args
                .ha_replication_port
                .or(config.ha.as_ref().and_then(|h| h.replication_port)),
        };

        let wal = config.wal.clone().unwrap_or_default();

        let log_path_value = args.log_path.clone().or(config.log_path.clone());
        let log_file_name = if let Some(ref log_path) = log_path_value {
            let log_file = std::path::Path::new(log_path);
            if let Some(parent) = log_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            log_path.clone()
        } else {
            let log_file_path = "./logs";
            let _ = std::fs::create_dir_all(log_file_path);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("{}/remdb-server-{}.log", log_file_path, timestamp)
        };

        RuntimeConfig {
            snapshot_dir: args.snapshot_dir.or(config.snapshot_dir),
            full_image: args.full_image,
            total_memory: args
                .total_memory
                .or(config.total_memory)
                .unwrap_or(1024 * 1024 * 100),
            default_max_records: args
                .default_max_records
                .or(config.default_max_records)
                .unwrap_or(10000),
            low_power_mode_supported: args
                .low_power_mode_supported
                .or(config.low_power_mode_supported)
                .unwrap_or(true),
            low_power_max_records: args.low_power_max_records.or(config.low_power_max_records),
            log_path: log_path_value,
            log_file_name,
            snapshot_interval: args.snapshot_interval.or(config.snapshot_interval),
            snapshot_type: args.snapshot_type.or(config.snapshot_type),
            max_incremental_snapshots: args
                .max_incremental_snapshots
                .or(config.max_incremental_snapshots),
            debug_mode,
            jdbc,
            pubsub,
            ha,
            wal,
            ddl_path: config.ddl_path,
        }
    }
}
