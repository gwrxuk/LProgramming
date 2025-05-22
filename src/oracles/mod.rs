use anyhow::Result;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

mod pyth;
mod switchboard;

pub use pyth::PythClient;
pub use switchboard::SwitchboardClient;

#[derive(Clone)]
pub struct PriceFeeds {
    pub pyth: Arc<PythClient>,
    pub switchboard: Arc<SwitchboardClient>,
}

#[async_trait]
pub trait PriceFeed {
    async fn get_price(&self, symbol: &str) -> Result<f64>;
    async fn get_price_with_confidence(&self, symbol: &str) -> Result<(f64, f64)>;
    async fn subscribe_price_updates(
        &self,
        symbol: &str,
        callback: Box<dyn Fn(f64) + Send + Sync>,
    ) -> Result<()>;
}

pub async fn init_price_feeds(config: &crate::config::Config) -> Result<PriceFeeds> {
    let rpc_client = RpcClient::new(config.solana_rpc_url.clone());

    let pyth_client = Arc::new(PythClient::new(
        rpc_client.clone(),
        config.pyth_network_program_id.clone(),
    )?);

    let switchboard_client = Arc::new(SwitchboardClient::new(
        rpc_client,
        config.switchboard_program_id.clone(),
    )?);

    Ok(PriceFeeds {
        pyth: pyth_client,
        switchboard: switchboard_client,
    })
}

// Helper functions for price feed management
pub async fn get_best_price(
    pyth_price: Result<f64>,
    switchboard_price: Result<f64>,
) -> Result<f64> {
    match (pyth_price, switchboard_price) {
        (Ok(pyth), Ok(switchboard)) => {
            // Compare prices and return the one with better confidence
            // This is a simple implementation - you might want to add more sophisticated logic
            Ok((pyth + switchboard) / 2.0)
        }
        (Ok(price), Err(_)) => Ok(price),
        (Err(_), Ok(price)) => Ok(price),
        (Err(e1), Err(e2)) => Err(anyhow::anyhow!(
            "Both price feeds failed: {:?}, {:?}",
            e1,
            e2
        )),
    }
}

pub async fn monitor_price_changes(
    price_feed: Arc<dyn PriceFeed + Send + Sync>,
    symbol: &str,
    threshold: f64,
    callback: Box<dyn Fn(f64) + Send + Sync>,
) -> Result<()> {
    let mut last_price = price_feed.get_price(symbol).await?;
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let current_price = price_feed.get_price(symbol).await?;
        let price_change = (current_price - last_price).abs() / last_price;

        if price_change >= threshold {
            callback(current_price);
            last_price = current_price;
        }
    }
} 