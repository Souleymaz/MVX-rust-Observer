// ============================================================
// MVX Rust Observer - Étape 2 : Architecture modulaire
// Fonction fetch_blocks + fetch_transactions (squelette)
// ============================================================

use reqwest::Client;
use serde::Deserialize;

// ============================================================
// CONFIGURATION
// Change ces variables pour contrôler le comportement
// ============================================================

// Nombre de blocs à récupérer (100 pour tester, plus grand pour historique)
const BLOCK_SIZE: u32 = 100;

// Si true : récupère depuis le début de la blockchain (nonce 1)
// Si false : récupère les derniers blocs
const FROM_GENESIS: bool = false;

// ============================================================
// STRUCTURES DE DONNÉES
// ============================================================

// Représente un bloc retourné par l'API MultiversX
#[derive(Debug, Deserialize, Clone)]
struct Block {
    nonce: u64,     // Numéro séquentiel du bloc
    hash: String,   // Hash unique du bloc
    shard: u32,     // Shard d'appartenance
    #[serde(rename = "txCount", default)]
    tx_count: u32,  // Nombre de transactions dans ce bloc
    #[serde(default)]
    epoch: u32,     // Epoch de production du bloc
    #[serde(default)]
    round: u64,     // Round du consensus
    #[serde(default)]
    timestamp: u64, // Timestamp Unix
}

// Représente une transaction retournée par l'API MultiversX
// On va remplir les champs au fur et à mesure qu'on découvre l'API
#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "txHash")]
    tx_hash: String,   // Hash unique de la transaction
    #[serde(default)]
    sender: String,    // Adresse de l'expéditeur
    #[serde(default)]
    receiver: String,  // Adresse du destinataire
    #[serde(default)]
    value: String,     // Montant en EGLD (en denomination minimale)
    #[serde(default)]
    status: String,    // Statut : success, fail, pending...
}

// ============================================================
// FONCTION : Récupérer les blocs
// ============================================================
// Si from_genesis = true  → récupère depuis le nonce 1
// Si from_genesis = false → récupère les derniers blocs
async fn fetch_blocks(client: &Client, from_genesis: bool, size: u32) -> Vec<Block> {
    println!("─────────────────────────────────────────");
    println!("📦 fetch_blocks() démarré");
    println!("   → Mode : {}", if from_genesis { "DEPUIS LA GENÈSE" } else { "DERNIERS BLOCS" });
    println!("   → Taille demandée : {} blocs", size);

    // Construction de l'URL selon le mode choisi
    let url = if from_genesis {
        // On commence depuis le nonce 1 (début de la blockchain)
        format!(
            "https://api.multiversx.com/blocks?size={}&nonce=1&order=asc",
            size
        )
    } else {
        // On récupère les derniers blocs (ordre décroissant par défaut)
        format!(
            "https://api.multiversx.com/blocks?size={}",
            size
        )
    };

    println!("🔗 URL construite : {}", url);
    println!("⏳ Envoi de la requête GET...");

    // Envoi de la requête HTTP
    match client.get(&url).send().await {
        Ok(response) => {
            println!("✅ Réponse reçue ! Status HTTP : {}", response.status());
            println!("⏳ Désérialisation JSON en cours...");

            match response.json::<Vec<Block>>().await {
                Ok(blocks) => {
                    println!("✅ {} blocs récupérés et désérialisés !", blocks.len());
                    blocks // On retourne les blocs
                }
                Err(e) => {
                    // Erreur de parsing JSON — champ manquant ou mal typé
                    eprintln!("❌ Erreur désérialisation JSON blocs : {}", e);
                    eprintln!("💡 Vérifie que la struct Block correspond bien à l'API.");
                    vec![] // On retourne un vecteur vide pour ne pas planter
                }
            }
        }
        Err(e) => {
            // Erreur réseau
            eprintln!("❌ Erreur connexion API fetch_blocks : {}", e);
            eprintln!("💡 Vérifie ta connexion et que api.multiversx.com est accessible.");
            vec![]
        }
    }
}

// ============================================================
// FONCTION : Récupérer les transactions d'un bloc
// ============================================================
// Prend le hash d'un bloc et retourne ses transactions
async fn fetch_transactions(client: &Client, block_hash: &str, shard: u32) -> Vec<Transaction> {
    println!("─────────────────────────────────────────");
    println!("💸 fetch_transactions() démarré");
    println!("   → Bloc hash : {}...", &block_hash[..8]);
    println!("   → Shard     : {}", shard);

    // URL pour récupérer les transactions d'un bloc via son hash
    let url = format!(
        "https://api.multiversx.com/blocks/{}/transactions",
        block_hash
    );

    println!("🔗 URL construite : {}", url);
    println!("⏳ Envoi de la requête GET...");

    match client.get(&url).send().await {
        Ok(response) => {
            println!("✅ Réponse reçue ! Status HTTP : {}", response.status());
            println!("⏳ Désérialisation JSON transactions en cours...");

            match response.json::<Vec<Transaction>>().await {
                Ok(txs) => {
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
            eprintln!("❌ Erreur connexion API fetch_transactions : {}", e);
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

    // Création du client HTTP réutilisable
    let client = Client::new();
    println!("✅ Client HTTP créé.");

    // ── Étape 1 : Récupération des blocs ──────────────────
    let blocks = fetch_blocks(&client, FROM_GENESIS, BLOCK_SIZE).await;

    if blocks.is_empty() {
        eprintln!("❌ Aucun bloc récupéré, arrêt du programme.");
        return;
    }

    // Affichage du résumé des blocs
    println!("─────────────────────────────────────────");
    println!("📋 RÉSUMÉ DES BLOCS :");
    for block in &blocks {
        println!(
            "  Bloc #{} | Shard {} | Epoch {} | Txs: {} | Hash: {}...",
            block.nonce,
            block.shard,
            block.epoch,
            block.tx_count,
            &block.hash[..8]
        );
    }

    // ── Étape 2 : Récupération des transactions ────────────
    // Pour l'instant on teste sur le PREMIER bloc qui a des transactions
    println!("─────────────────────────────────────────");
    println!("🔍 Recherche d'un bloc avec des transactions pour tester...");

    let bloc_avec_txs = blocks.iter().find(|b| b.tx_count > 0);

    match bloc_avec_txs {
        Some(block) => {
            println!("✅ Bloc trouvé : #{} avec {} txs", block.nonce, block.tx_count);
            let txs = fetch_transactions(&client, &block.hash, block.shard).await;

            println!("─────────────────────────────────────────");
            println!("💸 TRANSACTIONS DU BLOC #{} :", block.nonce);
            for tx in &txs {
                println!(
                    "  TX {} | {} → {} | Valeur: {} | Status: {}",
                    &tx.tx_hash[..8],
                    &tx.sender[..8],
                    &tx.receiver[..8],
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
