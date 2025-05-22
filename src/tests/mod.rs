use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::Duration;

use crate::{
    config::Config,
    dex::{DexClient, DexClients},
    oracles::{PriceFeed, PriceFeeds},
    cex::{CexClient, CexClients},
    simulation::{SimulationConfig, VolumeSimulator},
    metrics::MetricsManager,
};

#[tokio::test]
async fn test_dex_integration() -> Result<()> {
    let config = Config::load()?;
    let dex_clients = DexClients::init_clients(&config).await?;

    // Test price fetching
    let token_a = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL
    let token_b = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC

    let price = dex_clients.jupiter.get_price(&token_a, &token_b).await?;
    assert!(price > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_oracle_integration() -> Result<()> {
    let config = Config::load()?;
    let price_feeds = PriceFeeds::init_price_feeds(&config).await?;

    // Test price fetching
    let price = price_feeds.pyth.get_price("SOL/USD").await?;
    assert!(price > 0.0);

    let (price, confidence) = price_feeds.pyth.get_price_with_confidence("SOL/USD").await?;
    assert!(price > 0.0);
    assert!(confidence > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_cex_integration() -> Result<()> {
    let config = Config::load()?;
    let cex_clients = CexClients::init_clients(&config).await?;

    // Test order book fetching
    let order_book = cex_clients.binance.get_order_book("BTCUSDT").await?;
    assert!(!order_book.bids.is_empty());
    assert!(!order_book.asks.is_empty());

    // Test ticker fetching
    let price = cex_clients.binance.get_ticker("BTCUSDT").await?;
    assert!(price > 0.0);

    Ok(())
}

#[tokio::test]
async fn test_volume_simulation() -> Result<()> {
    let config = SimulationConfig {
        num_wallets: 10,
        min_trade_size: 0.1,
        max_trade_size: 1.0,
        min_interval: Duration::from_secs(1),
        max_interval: Duration::from_secs(5),
        duration: Duration::from_secs(10),
    };

    let mut simulator = VolumeSimulator::new(config)?;
    let result = simulator.run_simulation().await?;

    assert!(result.total_volume > 0.0);
    assert!(result.num_trades > 0);
    assert!(!result.wallet_volumes.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_metrics() -> Result<()> {
    let metrics = MetricsManager::new()?;

    // Test trade recording
    metrics
        .record_trade(100.0, true, Duration::from_millis(100))
        .await?;

    // Test LP position recording
    metrics
        .record_lp_position("position1".to_string(), 1000.0, 10.0)
        .await?;

    // Test rebalance recording
    metrics.record_rebalance().await?;

    // Test performance metrics
    metrics
        .update_performance_metrics(1024 * 1024, 50.0)
        .await?;

    // Test error recording
    metrics.record_error().await?;

    // Get metrics report
    let report = metrics.get_metrics_report().await?;
    assert!(!report.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_price_monitoring() -> Result<()> {
    let config = Config::load()?;
    let price_feeds = PriceFeeds::init_price_feeds(&config).await?;

    let mut price_updates = Vec::new();
    let callback = Box::new(|price: f64| {
        price_updates.push(price);
    });

    // Start price monitoring
    let monitor_handle = tokio::spawn(async move {
        price_feeds
            .pyth
            .subscribe_price_updates("SOL/USD", callback)
            .await
    });

    // Wait for some updates
    tokio::time::sleep(Duration::from_secs(5)).await;
    monitor_handle.abort();

    assert!(!price_updates.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_arbitrage_detection() -> Result<()> {
    let config = Config::load()?;
    let cex_clients = CexClients::init_clients(&config).await?;

    // Test arbitrage detection
    let (price, exchange) = cex::get_best_price_across_exchanges(&cex_clients, "BTCUSDT").await?;
    assert!(price > 0.0);
    assert!(!exchange.is_empty());

    Ok(())
} 