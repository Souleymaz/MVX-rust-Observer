// ============================================================
// MVX Rust Observer - Étape 1 : Récupération des 100 derniers blocs
// API MultiversX Mainnet - sans filtre de champs
// ============================================================

use reqwest::Client;
use serde::Deserialize;

// Structure qui représente un bloc retourné par l'API MultiversX
// Chaque champ correspond exactement à une clé JSON retournée par l'API
// #[serde(default)] évite une erreur si le champ est absent dans la réponse
#[derive(Debug, Deserialize)]
struct Block {
    nonce: u64,      // Numéro séquentiel du bloc (comme un ID)
    hash: String,    // Identifiant unique du bloc en hexadécimal
    shard: u32,      // Numéro du shard (MultiversX est multi-shard)
    #[serde(rename = "txCount", default)]
    tx_count: u32,   // Nombre de transactions — l'API appelle ça "txCount"
    #[serde(default)]
    epoch: u32,      // Epoch (période) dans laquelle ce bloc a été produit
    #[serde(default)]
    round: u64,      // Round du consensus pour ce bloc
    #[serde(default)]
    timestamp: u64,  // Timestamp Unix du bloc
}

#[tokio::main]
async fn main() {
    println!("🚀 Démarrage MVX Observer...");
    println!("📡 Connexion au mainnet MultiversX...");

    // Création du client HTTP reqwest
    // Ce client est réutilisable pour plusieurs requêtes
    let client = Client::new();
    println!("✅ Client HTTP créé.");

    // URL sans filtre de champs pour récupérer toutes les infos du bloc
    // size=100 = on veut les 100 derniers blocs
    let url = "https://api.multiversx.com/blocks?size=100";
    println!("🔗 URL cible : {}", url);

    println!("⏳ Envoi de la requête GET...");

    // Envoi de la requête HTTP GET de manière asynchrone
    match client.get(url).send().await {
        Ok(response) => {
            // La requête a abouti — on vérifie le status HTTP
            println!("✅ Réponse reçue ! Status HTTP : {}", response.status());

            println!("⏳ Désérialisation du JSON en cours...");

            // On tente de convertir le JSON en Vec<Block>
            match response.json::<Vec<Block>>().await {
                Ok(blocks) => {
                    println!("✅ {} blocs désérialisés avec succès !", blocks.len());
                    println!("─────────────────────────────────────────");

                    // Parcours de chaque bloc et affichage des infos clés
                    for block in &blocks {
                        println!(
                            "Bloc #{} | Shard {} | Epoch {} | Txs: {} | Hash: {}...",
                            block.nonce,
                            block.shard,
                            block.epoch,
                            block.tx_count,
                            &block.hash[..8] // 8 premiers caractères du hash pour lisibilité
                        );
                    }

                    println!("─────────────────────────────────────────");
                    println!("✅ Terminé. {} blocs affichés.", blocks.len());
                }
                Err(e) => {
                    // Erreur de désérialisation — souvent un champ manquant ou mal typé
                    eprintln!("❌ Erreur désérialisation JSON : {}", e);
                    eprintln!("💡 Vérifie que les champs de la struct Block correspondent bien à l'API.");
                }
            }
        }
        Err(e) => {
            // Erreur réseau — pas de connexion, DNS, timeout...
            eprintln!("❌ Erreur connexion API : {}", e);
            eprintln!("💡 Vérifie ta connexion internet et que api.multiversx.com est accessible.");
        }
    }
}
