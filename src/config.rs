use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub sync: SyncConfig,
    pub clickhouse: ClickhouseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub gateway_url: String,
    pub block_size: u32,
    pub request_delay_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyncConfig {
    pub from_genesis: bool,
    pub genesis_nonce: u64,
    pub workers_per_shard: usize,
    pub shards: Vec<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClickhouseConfig {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
}

pub fn load() -> Result<AppConfig> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let app_config = cfg.try_deserialize::<AppConfig>()?;
    Ok(app_config)
}
