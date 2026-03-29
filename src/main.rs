mod api;
mod config;
mod models;
mod normalizer;
mod storage;
mod sync;

use std::sync::Arc;
use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    tracing::info!("🚀 MVX Rust Observer v0.2 démarré");

    let cfg = config::load().map_err(|e| {
        tracing::error!("❌ Impossible de charger config.toml : {}", e);
        e
    })?;
    tracing::info!("✅ Configuration chargée.");
    tracing::info!("   → Mode : {}", if cfg.sync.from_genesis { "GENÈSE" } else { "TEMPS RÉEL" });
    tracing::info!("   → Shards : {:?}", cfg.sync.shards);

    let cfg = Arc::new(cfg);

    tracing::info!("🗄️  Connexion à ClickHouse sur {}...", cfg.clickhouse.url);
    let ch_client = storage::create_client(&cfg.clickhouse);

    storage::init_schema(&ch_client, &cfg.clickhouse.database).await?;

    let ch_client = Arc::new(ch_client);
    tracing::info!("✅ ClickHouse prêt.");

    tracing::info!("🔀 Lancement des workers de synchronisation...");
    sync::run(Arc::clone(&cfg), Arc::clone(&ch_client)).await?;

    tracing::info!("✅ Programme terminé.");
    Ok(())
}
