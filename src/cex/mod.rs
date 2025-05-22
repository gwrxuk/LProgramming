use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod binance;
mod bybit;
mod okx;

pub use binance::BinanceClient;
pub use bybit::BybitClient;
pub use okx::OkxClient;

#[derive(Clone)]
pub struct CexClients {
    pub binance: Arc<BinanceClient>,
    pub bybit: Arc<BybitClient>,
    pub okx: Arc<OkxClient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

#[async_trait]
pub trait CexClient {
    async fn get_order_book(&self, symbol: &str) -> Result<OrderBook>;
    async fn get_ticker(&self, symbol: &str) -> Result<f64>;
    async fn place_order(
        &self,
        symbol: &str,
        side: &str,
        price: f64,
        quantity: f64,
    ) -> Result<String>;
    async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()>;
    async fn get_balance(&self, asset: &str) -> Result<f64>;
    async fn get_recent_trades(&self, symbol: &str) -> Result<Vec<Trade>>;
}

pub async fn init_clients(config: &crate::config::Config) -> Result<CexClients> {
    let binance_client = Arc::new(BinanceClient::new(
        config.binance_api_key.clone(),
        config.binance_api_secret.clone(),
    )?);

    let bybit_client = Arc::new(BybitClient::new(
        config.bybit_api_key.clone(),
        config.bybit_api_secret.clone(),
    )?);

    let okx_client = Arc::new(OkxClient::new(
        config.okx_api_key.clone(),
        config.okx_api_secret.clone(),
    )?);

    Ok(CexClients {
        binance: binance_client,
        bybit: bybit_client,
        okx: okx_client,
    })
}

// Helper functions for CEX operations
pub async fn get_best_price_across_exchanges(
    clients: &CexClients,
    symbol: &str,
) -> Result<(f64, String)> {
    let binance_price = clients.binance.get_ticker(symbol).await;
    let bybit_price = clients.bybit.get_ticker(symbol).await;
    let okx_price = clients.okx.get_ticker(symbol).await;

    let prices = vec![
        (binance_price, "Binance"),
        (bybit_price, "Bybit"),
        (okx_price, "OKX"),
    ];

    let mut best_price = None;
    let mut best_exchange = None;

    for (price, exchange) in prices {
        if let Ok(p) = price {
            if best_price.is_none() || p > best_price.unwrap() {
                best_price = Some(p);
                best_exchange = Some(exchange);
            }
        }
    }

    match (best_price, best_exchange) {
        (Some(price), Some(exchange)) => Ok((price, exchange.to_string())),
        _ => Err(anyhow::anyhow!("No valid prices available")),
    }
}

pub async fn execute_arbitrage(
    clients: &CexClients,
    symbol: &str,
    min_profit_threshold: f64,
) -> Result<()> {
    // Get order books from all exchanges
    let binance_ob = clients.binance.get_order_book(symbol).await?;
    let bybit_ob = clients.bybit.get_order_book(symbol).await?;
    let okx_ob = clients.okx.get_order_book(symbol).await?;

    // Find arbitrage opportunities
    // This is a simplified implementation - in production, you'd want to:
    // 1. Consider fees and slippage
    // 2. Check available balances
    // 3. Implement proper risk management
    // 4. Handle order execution failures
    // 5. Consider market impact

    let best_bid = vec![
        binance_ob.bids.first(),
        bybit_ob.bids.first(),
        okx_ob.bids.first(),
    ]
    .into_iter()
    .flatten()
    .max_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
    .ok_or_else(|| anyhow::anyhow!("No bids available"))?;

    let best_ask = vec![
        binance_ob.asks.first(),
        bybit_ob.asks.first(),
        okx_ob.asks.first(),
    ]
    .into_iter()
    .flatten()
    .min_by(|a, b| a.price.partial_cmp(&b.price).unwrap())
    .ok_or_else(|| anyhow::anyhow!("No asks available"))?;

    let profit = best_bid.price - best_ask.price;
    if profit > min_profit_threshold {
        // Execute arbitrage trades
        // This would involve:
        // 1. Placing buy order on exchange with best ask
        // 2. Placing sell order on exchange with best bid
        // 3. Monitoring order execution
        // 4. Handling any failures
        todo!("Implement arbitrage execution");
    }

    Ok(())
}

pub async fn monitor_price_differences(
    clients: &CexClients,
    symbol: &str,
    threshold: f64,
) -> Result<()> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        interval.tick().await;
        let (price, exchange) = get_best_price_across_exchanges(clients, symbol).await?;
        
        // Compare with other exchanges
        let other_prices = vec![
            clients.binance.get_ticker(symbol).await,
            clients.bybit.get_ticker(symbol).await,
            clients.okx.get_ticker(symbol).await,
        ];

        for other_price in other_prices {
            if let Ok(p) = other_price {
                let diff = (price - p).abs() / p;
                if diff > threshold {
                    // Log or alert about significant price difference
                    println!(
                        "Significant price difference detected: {} vs {} ({}%)",
                        price, p, diff * 100.0
                    );
                }
            }
        }
    }
} 