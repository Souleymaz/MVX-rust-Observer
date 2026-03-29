// ============================================================
// MVX Rust Observer - Étape 1 : Récupération des 100 derniers blocs
// API MultiversX Mainnet
// ============================================================

use reqwest::Client;
use serde::Deserialize;

// Structure qui représente un bloc retourné par l'API MultiversX
#[derive(Debug, Deserialize)]
struct Block {
    nonce: u64,          // Numéro du bloc
    hash: String,        // Hash unique du bloc
    numTxs: u32,         // Nombre de transactions dans ce bloc
    shard: u32,          // Shard d'appartenance du bloc
}

#[tokio::main]
async fn main() {
    println!("🚀 Démarrage MVX Observer...");
    println!("📡 Connexion au mainnet MultiversX...");

    // Création du client HTTP
    let client = Client::new();

    // URL de l'API pour récupérer les 100 derniers blocs
    let url = "https://api.multiversx.com/blocks?size=100&fields=nonce,hash,numTxs,shard";

    println!("🔗 Appel API : {}", url);

    // Appel à l'API
    match client.get(url).send().await {
        Ok(response) => {
            println!("✅ Réponse reçue, status : {}", response.status());

            // Désérialisation du JSON
            match response.json::<Vec<Block>>().await {
                Ok(blocks) => {
                    println!("✅ {} blocs récupérés !", blocks.len());
                    println!("─────────────────────────────────────────");

                    // Affichage de chaque bloc
                    for block in &blocks {
                        println!(
                            "Bloc #{} | Shard {} | Txs: {} | Hash: {}",
                            block.nonce,
                            block.shard,
                            block.numTxs,
                            &block.hash[..8] // On affiche juste les 8 premiers caractères du hash
                        );
                    }

                    println!("─────────────────────────────────────────");
                    println!("✅ Terminé. {} blocs affichés.", blocks.len());
                }
                Err(e) => {
                    // Erreur de désérialisation JSON
                    eprintln!("❌ Erreur désérialisation JSON : {}", e);
                }
            }
        }
        Err(e) => {
            // Erreur de connexion réseau
            eprintln!("❌ Erreur connexion API : {}", e);
        }
    }
}
