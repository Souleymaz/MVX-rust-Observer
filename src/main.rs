// ============================================================
// MVX Rust Observer - Étape 2 : Architecture modulaire
// Fonction fetch_blocks + fetch_transactions (corrigé)
// ============================================================

use reqwest::Client;
use serde::Deserialize;

// ============================================================
// CONFIGURATION
// ============================================================

const BLOCK_SIZE: u32 = 100;
const FROM_GENESIS: bool = false;

// ============================================================
// STRUCTURES DE DONNÉES — api.multiversx.com
// ============================================================

#[derive(Debug, Deserialize, Clone)]
struct Block {
    nonce: u64,
    hash: String,
    shard: u32,
    #[serde(rename = "txCount", default)]
    tx_count: u32,
    #[serde(default)]
    epoch: u32,
    #[serde(default)]
    round: u64,
    #[serde(default)]
    timestamp: u64,
}

// ============================================================
// STRUCTURES DE DONNÉES — gateway.multiversx.com
// Structure : { data: { block: { miniBlocks: [ { transactions: [...] } ] } } }
// ============================================================

#[derive(Debug, Deserialize)]
struct GatewayResponse {
    data: GatewayData,
}

#[derive(Debug, Deserialize)]
struct GatewayData {
    block: GatewayBlock,
}

#[derive(Debug, Deserialize)]
struct GatewayBlock {
    #[serde(rename = "miniBlocks", default)]
    mini_blocks: Vec<MiniBlock>,
}

#[derive(Debug, Deserialize)]
struct MiniBlock {
    #[serde(default)]
    transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "hash", default)]
    tx_hash: String,
    #[serde(default)]
    sender: String,
    #[serde(default)]
    receiver: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    status: String,
}

// ============================================================
// FONCTION : Récupérer les blocs (api.multiversx.com)
// ============================================================
async fn fetch_blocks(client: &Client, from_genesis: bool, size: u32) -> Vec<Block> {
    println!("─────────────────────────────────────────");
    println!("📦 fetch_blocks() démarré");
    println!("   → Mode : {}", if from_genesis { "DEPUIS LA GENÈSE" } else { "DERNIERS BLOCS" });
    println!("   → Taille demandée : {} blocs", size);

    let url = if from_genesis {
        format!("https://api.multiversx.com/blocks?size={}&nonce=1&order=asc", size)
    } else {
        format!("https://api.multiversx.com/blocks?size={}", size)
    };

    println!("🔗 URL construite : {}", url);
    println!("⏳ Envoi de la requête GET...");

    match client.get(&url).send().await {
        Ok(response) => {
            println!("✅ Réponse reçue ! Status HTTP : {}", response.status());
            println!("⏳ Désérialisation JSON en cours...");

            match response.json::<Vec<Block>>().await {
                Ok(blocks) => {
                    println!("✅ {} blocs récupérés et désérialisés !", blocks.len());
                    blocks
                }
                Err(e) => {
                    eprintln!("❌ Erreur désérialisation JSON blocs : {}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur connexion API fetch_blocks : {}", e);
            vec![]
        }
    }
}

// ============================================================
// FONCTION : Récupérer les transactions d'un bloc
// Endpoint : GET /block/{shard}/by-hash/{hash}?withTxs=true
// ============================================================
async fn fetch_transactions(client: &Client, block_hash: &str, shard: u32) -> Vec<Transaction> {
    println!("─────────────────────────────────────────");
    println!("💸 fetch_transactions() démarré");
    println!("   → Bloc hash : {}...", &block_hash[..8.min(block_hash.len())]);
    println!("   → Shard     : {}", shard);

    let url = format!(
        "https://gateway.multiversx.com/block/{}/by-hash/{}?withTxs=true",
        shard, block_hash
    );

    println!("🔗 URL construite : {}", url);
    println!("⏳ Envoi de la requête GET...");

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            println!("✅ Réponse reçue ! Status HTTP : {}", status);

            if !status.is_success() {
                eprintln!("❌ Erreur HTTP {} pour le bloc {}", status, block_hash);
                return vec![];
            }

            println!("⏳ Désérialisation JSON transactions en cours...");

            match response.json::<GatewayResponse>().await {
                Ok(gateway_resp) => {
                    let txs: Vec<Transaction> = gateway_resp
                        .data
                        .block
                        .mini_blocks
                        .into_iter()
                        .flat_map(|mb| mb.transactions)
                        .collect();

                    println!("✅ {} transactions récupérées pour ce bloc !", txs.len());
                    txs
                }
                Err(e) => {
                    eprintln!("❌ Erreur désérialisation JSON transactions : {}", e);
                    eprintln!("💡 Hash du bloc concerné : {}", block_hash);
                    vec![]
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur connexion Gateway fetch_transactions : {}", e);
            vec![]
        }
    }
}

// ============================================================
// POINT D'ENTRÉE PRINCIPAL
// ============================================================
#[tokio::main]
async fn main() {
    println!("🚀 Démarrage MVX Observer...");
    println!("📡 Connexion au mainnet MultiversX...");

    let client = Client::new();
    println!("✅ Client HTTP créé.");

    // ── Étape 1 : Récupération des blocs ──────────────────
    let blocks = fetch_blocks(&client, FROM_GENESIS, BLOCK_SIZE).await;

    if blocks.is_empty() {
        eprintln!("❌ Aucun bloc récupéré, arrêt du programme.");
        return;
    }

    println!("─────────────────────────────────────────");
    println!("📋 RÉSUMÉ DES BLOCS :");
    for block in &blocks {
        println!(
            "  Bloc #{} | Shard {} | Epoch {} | Txs: {} | Hash: {}...",
            block.nonce,
            block.shard,
            block.epoch,
            block.tx_count,
            &block.hash[..8.min(block.hash.len())]
        );
    }

    // ── Étape 2 : Récupération des transactions ────────────
    println!("─────────────────────────────────────────");
    println!("🔍 Recherche d'un bloc avec des transactions pour tester...");

    // On ignore le shard metachain (4294967295)
    let bloc_avec_txs = blocks
        .iter()
        .find(|b| b.tx_count > 0 && b.shard != 4294967295);

    match bloc_avec_txs {
        Some(block) => {
            println!("✅ Bloc trouvé : #{} avec {} txs (shard {})", block.nonce, block.tx_count, block.shard);
            let txs = fetch_transactions(&client, &block.hash, block.shard).await;

            println!("─────────────────────────────────────────");
            println!("💸 TRANSACTIONS DU BLOC #{} :", block.nonce);

            if txs.is_empty() {
                println!("  ⚠️  Aucune transaction désérialisée (miniBlocks vides ou cross-shard).");
            }

            for tx in &txs {
                let sender_preview   = if tx.sender.len()   >= 8 { &tx.sender[..8]   } else { &tx.sender };
                let receiver_preview = if tx.receiver.len() >= 8 { &tx.receiver[..8] } else { &tx.receiver };
                let hash_preview     = if tx.tx_hash.len()  >= 8 { &tx.tx_hash[..8]  } else { &tx.tx_hash };

                println!(
                    "  TX {} | {} → {} | Valeur: {} | Status: {}",
                    hash_preview,
                    sender_preview,
                    receiver_preview,
                    tx.value,
                    tx.status
                );
            }
        }
        None => {
            println!("⚠️  Aucun bloc avec des transactions dans cet échantillon.");
            println!("💡 Essaie d'augmenter BLOCK_SIZE ou de relancer le programme.");
        }
    }

    println!("─────────────────────────────────────────");
    println!("✅ Programme terminé.");
}
