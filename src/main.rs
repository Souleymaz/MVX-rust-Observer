use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[tokio::main]
async fn main() {
    println!("🚀 Démarrage MVX Observer...");
    println!("📡 Connexion au mainnet MultiversX...");

    let client = Client::new();

    // On enlève le filtre fields pour récupérer tous les champs
    let url = "https://api.multiversx.com/blocks?size=100";

    println!("🔗 Appel API : {}", url);

    match client.get(url).send().await {
        Ok(response) => {
            println!("✅ Réponse reçue, status : {}", response.status());

            match response.json::<Vec<Block>>().await {
                Ok(blocks) => {
                    println!("✅ {} blocs récupérés !", blocks.len());
                    println!("─────────────────────────────────────────");

                    for block in &blocks {
                        println!(
                            "Bloc #{} | Shard {} | Epoch {} | Txs: {} | Hash: {}",
                            block.nonce,
                            block.shard,
                            block.epoch,
                            block.tx_count,
                            &block.hash[..8]
                        );
                    }

                    println!("─────────────────────────────────────────");
                    println!("✅ Terminé. {} blocs affichés.", blocks.len());
                }
                Err(e) => {
                    eprintln!("❌ Erreur désérialisation JSON : {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Erreur connexion API : {}", e);
        }
    }
}
