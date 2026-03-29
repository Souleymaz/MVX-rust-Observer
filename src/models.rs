use serde::{Deserialize, Serialize};
use clickhouse::Row;

// ── API structs ──────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct ApiBlock {
    pub nonce: u64,
    pub hash: String,
    pub shard: u32,
    #[serde(rename = "txCount", default)]
    pub tx_count: u32,
    #[serde(default)]
    pub epoch: u32,
    #[serde(default)]
    pub round: u64,
    #[serde(default)]
    pub timestamp: u64,
}

#[derive(Debug, Deserialize)]
pub struct GatewayResponse {
    pub data: GatewayData,
}

#[derive(Debug, Deserialize)]
pub struct GatewayData {
    pub block: GatewayBlock,
}

#[derive(Debug, Deserialize)]
pub struct GatewayBlock {
    #[serde(rename = "miniBlocks", default)]
    pub mini_blocks: Vec<MiniBlock>,
}

#[derive(Debug, Deserialize)]
pub struct MiniBlock {
    #[serde(default)]
    pub transactions: Vec<ApiTransaction>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiTransaction {
    #[serde(rename = "hash", default)]
    pub tx_hash: String,
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub receiver: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub status: String,
    #[serde(rename = "gasUsed", default)]
    pub gas_used: u64,
    #[serde(rename = "gasPrice", default)]
    pub gas_price: u64,
    #[serde(rename = "gasLimit", default)]
    pub gas_limit: u64,
    #[serde(default)]
    pub nonce: u64,
    #[serde(default)]
    pub data: Option<String>,
    #[serde(rename = "type", default)]
    pub tx_type: String,
}

// ── ClickHouse structs ───────────────────────────────────

#[derive(Debug, Serialize, Row, Clone)]
pub struct BlockRow {
    pub nonce: u64,
    pub hash: String,
    pub shard: u32,
    pub tx_count: u32,
    pub epoch: u32,
    pub round: u64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Row, Clone)]
pub struct TransactionRow {
    pub tx_hash: String,
    pub block_hash: String,
    pub block_nonce: u64,
    pub shard: u32,
    pub sender: String,
    pub receiver: String,
    pub value_egld: f64,
    pub value_raw: String,
    pub status: String,
    pub gas_used: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub nonce: u64,
    pub tx_type: String,
    pub timestamp: u64,
}
