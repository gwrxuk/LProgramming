use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod dex;
mod oracles;
mod cex;
mod models;
mod utils;
mod metrics;
mod simulation;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(true)
        .pretty()
        .init();

    info!("Starting Solana DEX Bot...");

    // Load configuration
    let config = config::Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize metrics
    metrics::init(&config)?;
    info!("Metrics initialized");

    // Initialize DEX clients
    let dex_clients = dex::init_clients(&config).await?;
    info!("DEX clients initialized");

    // Initialize price feeds
    let price_feeds = oracles::init_price_feeds(&config).await?;
    info!("Price feeds initialized");

    // Initialize CEX clients
    let cex_clients = cex::init_clients(&config).await?;
    info!("CEX clients initialized");

    // Start the main trading loop
    run_trading_loop(config, dex_clients, price_feeds, cex_clients).await?;

    Ok(())
}

async fn run_trading_loop(
    config: config::Config,
    dex_clients: dex::DexClients,
    price_feeds: oracles::PriceFeeds,
    cex_clients: cex::CexClients,
) -> Result<()> {
    info!("Starting trading loop...");
    
    // Main trading loop implementation will go here
    // This is where we'll implement the core trading logic
    
    Ok(())
} 