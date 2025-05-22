use anyhow::Result;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

mod raydium;
mod jupiter;

pub use raydium::RaydiumClient;
pub use jupiter::JupiterClient;

#[derive(Clone)]
pub struct DexClients {
    pub raydium: Arc<RaydiumClient>,
    pub jupiter: Arc<JupiterClient>,
}

#[async_trait]
pub trait DexClient {
    async fn get_price(&self, token_a: &Pubkey, token_b: &Pubkey) -> Result<f64>;
    async fn create_lp_position(
        &self,
        token_a: &Pubkey,
        token_b: &Pubkey,
        amount_a: f64,
        amount_b: f64,
        min_price: f64,
        max_price: f64,
    ) -> Result<String>;
    async fn rebalance_position(
        &self,
        position_id: &str,
        new_min_price: f64,
        new_max_price: f64,
    ) -> Result<()>;
    async fn harvest_fees(&self, position_id: &str) -> Result<()>;
    async fn execute_swap(
        &self,
        token_in: &Pubkey,
        token_out: &Pubkey,
        amount_in: f64,
        min_amount_out: f64,
    ) -> Result<String>;
}

pub async fn init_clients(config: &crate::config::Config) -> Result<DexClients> {
    let rpc_client = RpcClient::new(config.solana_rpc_url.clone());

    let raydium_client = Arc::new(RaydiumClient::new(
        rpc_client.clone(),
        config.raydium_program_id.clone(),
    )?);

    let jupiter_client = Arc::new(JupiterClient::new(
        rpc_client,
        config.jupiter_api_url.clone(),
    )?);

    Ok(DexClients {
        raydium: raydium_client,
        jupiter: jupiter_client,
    })
}

// Helper functions for LP management
pub async fn calculate_optimal_range(
    current_price: f64,
    volatility: f64,
    time_horizon: f64,
) -> (f64, f64) {
    // Implement range calculation logic based on volatility and time horizon
    let range = volatility * time_horizon.sqrt();
    (current_price * (1.0 - range), current_price * (1.0 + range))
}

pub async fn calculate_rebalance_threshold(
    current_price: f64,
    min_price: f64,
    max_price: f64,
) -> f64 {
    // Calculate when to rebalance based on price movement
    let mid_price = (min_price + max_price) / 2.0;
    (current_price - mid_price).abs() / mid_price
}

pub async fn calculate_optimal_position_size(
    total_capital: f64,
    risk_per_trade: f64,
    current_price: f64,
) -> f64 {
    // Calculate position size based on risk management rules
    total_capital * risk_per_trade / current_price
} 