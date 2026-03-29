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
    println!("🚀

