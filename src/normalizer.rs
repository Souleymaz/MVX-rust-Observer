use crate::models::{ApiBlock, ApiTransaction, BlockRow, TransactionRow};

const EGLD_DENOMINATION: f64 = 1_000_000_000_000_000_000.0;

pub fn normalize_block(block: &ApiBlock) -> BlockRow {
    BlockRow {
        nonce: block.nonce,
        hash: block.hash.clone(),
        shard: block.shard,
        tx_count: block.tx_count,
        epoch: block.epoch,
        round: block.round,
        timestamp: block.timestamp,
    }
}

pub fn normalize_transaction(
    tx: &ApiTransaction,
    block_hash: &str,
    block_nonce: u64,
    shard: u32,
    timestamp: u64,
) -> TransactionRow {
    TransactionRow {
        tx_hash: tx.tx_hash.clone(),
        block_hash: block_hash.to_string(),
        block_nonce,
        shard,
        sender: tx.sender.clone(),
        receiver: tx.receiver.clone(),
        value_egld: parse_egld_value(&tx.value),
        value_raw: tx.value.clone(),
        status: tx.status.clone(),
        gas_used: tx.gas_used,
        gas_price: tx.gas_price,
        gas_limit: tx.gas_limit,
        nonce: tx.nonce,
        tx_type: classify_transaction(tx),
        timestamp,
    }
}

fn parse_egld_value(raw: &str) -> f64 {
    if raw.is_empty() || raw == "0" {
        return 0.0;
    }
    match raw.parse::<u128>() {
        Ok(val) => val as f64 / EGLD_DENOMINATION,
        Err(_) => {
            tracing::warn!("⚠️  Valeur non parseable : '{}'", raw);
            0.0
        }
    }
}

fn classify_transaction(tx: &ApiTransaction) -> String {
    if !tx.tx_type.is_empty() && tx.tx_type != "normal" {
        return tx.tx_type.clone();
    }
    match &tx.data {
        Some(data) if data.starts_with("delegate") => "delegate".to_string(),
        Some(data) if data.starts_with("unDelegate") => "undelegate".to_string(),
        Some(data) if data.starts_with("claimRewards") => "claim_rewards".to_string(),
        Some(data) if data.starts_with("ESDTTransfer") => "esdt_transfer".to_string(),
        Some(data) if data.starts_with("MultiESDTNFTTransfer") => "nft_transfer".to_string(),
        Some(data) if !data.is_empty() => "smart_contract".to_string(),
        _ => {
            if tx.value != "0" && !tx.value.is_empty() {
                "egld_transfer".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }
}
