use anyhow::Result;
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::config::ApiConfig;
use crate::models::{ApiBlock, ApiTransaction, GatewayResponse};

pub async fn fetch_blocks(
    client: &Client,
    cfg: &ApiConfig,
    from_nonce: Option<u64>,
) -> Result<Vec<ApiBlock>> {
    let url = match from_nonce {
        Some(nonce) => format!(
            "{}/blocks?size={}&nonce={}&order=asc",
            cfg.base_url, cfg.block_size, nonce
        ),
        None => format!("{}/blocks?size={}", cfg.base_url, cfg.block_size),
    };

    tracing::debug!("🔗 fetch_blocks URL : {}", url);

    let response = client.get(&url).send().await?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("fetch_blocks HTTP {} pour URL {}", status, url);
    }

    let blocks = response.json::<Vec<ApiBlock>>().await?;
    tracing::debug!("📦 {} blocs récupérés depuis nonce {:?}", blocks.len(), from_nonce);

    sleep(Duration::from_millis(cfg.request_delay_ms)).await;

    Ok(blocks)
}

pub async fn fetch_transactions(
    client: &Client,
    cfg: &ApiConfig,
    block_hash: &str,
    shard: u32,
) -> Result<Vec<ApiTransaction>> {
    let url = format!(
        "{}/block/{}/by-hash/{}?withTxs=true",
        cfg.gateway_url, shard, block_hash
    );

    tracing::debug!("🔗 fetch_transactions URL : {}", url);

    let response = client.get(&url).send().await?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("fetch_transactions HTTP {} pour bloc {}", status, block_hash);
    }

    let gateway_resp = response.json::<GatewayResponse>().await?;

    let txs: Vec<ApiTransaction> = gateway_resp
        .data
        .block
        .mini_blocks
        .into_iter()
        .flat_map(|mb| mb.transactions)
        .collect();

    tracing::debug!("💸 {} transactions pour bloc {}", txs.len(), &block_hash[..8.min(block_hash.len())]);

    sleep(Duration::from_millis(cfg.request_delay_ms)).await;

    Ok(txs)
}
