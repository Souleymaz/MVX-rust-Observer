use anyhow::Result;
use reqwest::Client as HttpClient;
use tokio::task::JoinSet;
use std::sync::Arc;

use crate::api;
use crate::config::AppConfig;
use crate::models::ApiBlock;
use crate::normalizer;
use crate::storage;

pub async fn run(cfg: Arc<AppConfig>, ch_client: Arc<clickhouse::Client>) -> Result<()> {
    tracing::info!("🚀 Démarrage sync parallèle sur {} shards...", cfg.sync.shards.len());

    let mut join_set = JoinSet::new();

    for &shard in &cfg.sync.shards {
        let cfg_clone = Arc::clone(&cfg);
        let ch_clone = Arc::clone(&ch_client);

        join_set.spawn(async move {
            tracing::info!("🔀 Worker shard {} démarré", shard);
            match sync_shard(shard, cfg_clone, ch_clone).await {
                Ok(_) => tracing::info!("✅ Worker shard {} terminé.", shard),
                Err(e) => tracing::error!("❌ Worker shard {} erreur : {}", shard, e),
            }
        });
    }

    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            tracing::error!("❌ Task panic : {}", e);
        }
    }

    tracing::info!("✅ Sync complète terminée.");
    Ok(())
}

async fn sync_shard(
    shard: u32,
    cfg: Arc<AppConfig>,
    ch_client: Arc<clickhouse::Client>,
) -> Result<()> {
    let http_client = HttpClient::new();

    let mut current_nonce = if cfg.sync.from_genesis {
        let checkpoint = storage::get_checkpoint(&ch_client, shard).await?;
        if checkpoint > 0 {
            tracing::info!("📍 Shard {} : reprise depuis checkpoint nonce {}", shard, checkpoint);
            checkpoint
        } else {
            tracing::info!("📍 Shard {} : démarrage depuis la genèse", shard);
            cfg.sync.genesis_nonce
        }
    } else {
        0
    };

    let mut total_blocks = 0u64;
    let mut total_txs = 0u64;

    loop {
        let from_nonce = if cfg.sync.from_genesis && current_nonce > 0 {
            Some(current_nonce)
        } else {
            None
        };

        let blocks = match api::fetch_blocks(&http_client, &cfg.api, from_nonce).await {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("⚠️  Shard {} fetch_blocks erreur : {}. Retry dans 5s...", shard, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if blocks.is_empty() {
            tracing::info!("✅ Shard {} : plus de blocs à synchroniser.", shard);
            break;
        }

        let shard_blocks: Vec<&ApiBlock> = blocks.iter()
            .filter(|b| b.shard == shard)
            .collect();

        if shard_blocks.is_empty() {
            if let Some(last) = blocks.last() {
                current_nonce = last.nonce + 1;
            }
            continue;
        }

        // Insertion des blocs
        let block_rows: Vec<_> = shard_blocks.iter()
            .map(|b| normalizer::normalize_block(b))
            .collect();

        if let Err(e) = storage::insert_blocks(&ch_client, &block_rows).await {
            tracing::error!("❌ Shard {} insert_blocks erreur : {}", shard, e);
        }

        // Récupération et insertion des transactions
        let mut all_tx_rows = Vec::new();

        for block in &shard_blocks {
            if block.tx_count == 0 {
                continue;
            }
            match api::fetch_transactions(&http_client, &cfg.api, &block.hash, shard).await {
                Ok(txs) => {
                    let tx_rows: Vec<_> = txs.iter()
                        .map(|tx| normalizer::normalize_transaction(
                            tx,
                            &block.hash,
                            block.nonce,
                            shard,
                            block.timestamp,
                        ))
                        .collect();
                    all_tx_rows.extend(tx_rows);
                }
                Err(e) => {
                    tracing::warn!("⚠️  Shard {} bloc {} fetch_tx erreur : {}", shard, block.nonce, e);
                }
            }
        }

        if let Err(e) = storage::insert_transactions(&ch_client, &all_tx_rows).await {
            tracing::error!("❌ Shard {} insert_transactions erreur : {}", shard, e);
        }

        // Checkpoint
        if let Some(last_block) = shard_blocks.last() {
            current_nonce = last_block.nonce + 1;
            if let Err(e) = storage::save_checkpoint(&ch_client, shard, last_block.nonce).await {
                tracing::warn!("⚠️  Shard {} save_checkpoint erreur : {}", shard, e);
            }
        }

        total_blocks += shard_blocks.len() as u64;
        total_txs += all_tx_rows.len() as u64;

        tracing::info!(
            "📊 Shard {} | Nonce {} | Blocs: {} | Txs: {}",
            shard, current_nonce, total_blocks, total_txs
        );

        if !cfg.sync.from_genesis {
            tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
        }
    }

    Ok(())
}
