use anyhow::Result;
use clickhouse::Client;
use crate::config::ClickhouseConfig;
use crate::models::{BlockRow, TransactionRow};

pub fn create_client(cfg: &ClickhouseConfig) -> Client {
    Client::default()
        .with_url(&cfg.url)
        .with_database(&cfg.database)
        .with_user(&cfg.username)
        .with_password(&cfg.password)
}

pub async fn init_schema(client: &Client, database: &str) -> Result<()> {
    tracing::info!("️ Initialisation du schema ClickHouse...");

    // Création de la base
    client
        .query(&format!("CREATE DATABASE IF NOT EXISTS {}", database))
        .execute()
        .await?;

    // Table blocks
    client
        .query(
            "CREATE TABLE IF NOT EXISTS blocks (
                nonce UInt64,
                hash String,
                shard UInt32,
                tx_count UInt32,
                epoch UInt32,
                round UInt64,
                timestamp UInt64
            )
            ENGINE = ReplacingMergeTree()
            PARTITION BY toYYYYMM(toDateTime(timestamp))
            ORDER BY (shard, nonce)"
        )
        .execute()
        .await?;

    // Table transactions
    client
        .query(
            "CREATE TABLE IF NOT EXISTS transactions (
                tx_hash String,
                block_hash String,
                block_nonce UInt64,
                shard UInt32,
                sender String,
                receiver String,
                value_egld Float64,
                value_raw String,
                status String,
                gas_used UInt64,
                gas_price UInt64,
                gas_limit UInt64,
                nonce UInt64,
                tx_type String,
                timestamp UInt64
            )
            ENGINE = ReplacingMergeTree()
            PARTITION BY toYYYYMM(toDateTime(timestamp))
            ORDER BY (shard, block_nonce, tx_hash)"
        )
        .execute()
        .await?;

    // Table sync_checkpoint
    client
        .query(
            "CREATE TABLE IF NOT EXISTS sync_checkpoint (
                shard UInt32,
                last_nonce UInt64,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            ORDER BY shard"
        )
        .execute()
        .await?;

    tracing::info!("✅ Schema ClickHouse initialisé.");
    Ok(())
}

pub async fn insert_blocks(client: &Client, blocks: &[BlockRow]) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }
    let mut insert = client.insert("blocks")?;
    for block in blocks {
        insert.write(block).await?;
    }
    insert.end().await?;
    tracing::debug!("✅ {} blocs insérés.", blocks.len());
    Ok(())
}

pub async fn insert_transactions(client: &Client, txs: &[TransactionRow]) -> Result<()> {
    if txs.is_empty() {
        return Ok(());
    }
    let mut insert = client.insert("transactions")?;
    for tx in txs {
        insert.write(tx).await?;
    }
    insert.end().await?;
    tracing::debug!("✅ {} transactions insérées.", txs.len());
    Ok(())
}

pub async fn get_checkpoint(client: &Client, shard: u32) -> Result<u64> {
    let result = client
        .query("SELECT last_nonce FROM sync_checkpoint WHERE shard = ? ORDER BY updated_at DESC LIMIT 1")
        .bind(shard)
        .fetch_optional::<u64>()
        .await?;
    Ok(result.unwrap_or(0))
}

pub async fn save_checkpoint(client: &Client, shard: u32, last_nonce: u64) -> Result<()> {
    client
        .query("INSERT INTO sync_checkpoint (shard, last_nonce) VALUES (?, ?)")
        .bind(shard)
        .bind(last_nonce)
        .execute()
        .await?;
    tracing::debug!("Checkpoint shard {} → nonce {}", shard, last_nonce);
    Ok(())
}