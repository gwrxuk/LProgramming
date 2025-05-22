use anyhow::Result;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    // Solana Configuration
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub solana_keypair_path: PathBuf,

    // DEX Configuration
    pub raydium_program_id: String,
    pub jupiter_api_url: String,

    // Oracle Configuration
    pub pyth_network_program_id: String,
    pub switchboard_program_id: String,

    // CEX API Keys
    pub binance_api_key: String,
    pub binance_api_secret: String,
    pub bybit_api_key: String,
    pub bybit_api_secret: String,
    pub okx_api_key: String,
    pub okx_api_secret: String,

    // Database Configuration
    pub database_url: String,

    // Metrics Configuration
    pub prometheus_port: u16,
    pub metrics_enabled: bool,

    // Trading Configuration
    pub min_trade_size: f64,
    pub max_trade_size: f64,
    pub price_impact_threshold: f64,
    pub slippage_tolerance: f64,
    pub rebalance_threshold: f64,

    // Logging Configuration
    pub log_level: String,
    pub log_file_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file
        dotenv::dotenv().ok();

        Ok(Config {
            solana_rpc_url: env::var("SOLANA_RPC_URL")?,
            solana_ws_url: env::var("SOLANA_WS_URL")?,
            solana_keypair_path: PathBuf::from(env::var("SOLANA_KEYPAIR_PATH")?),

            raydium_program_id: env::var("RAYDIUM_PROGRAM_ID")?,
            jupiter_api_url: env::var("JUPITER_API_URL")?,

            pyth_network_program_id: env::var("PYTH_NETWORK_PROGRAM_ID")?,
            switchboard_program_id: env::var("SWITCHBOARD_PROGRAM_ID")?,

            binance_api_key: env::var("BINANCE_API_KEY")?,
            binance_api_secret: env::var("BINANCE_API_SECRET")?,
            bybit_api_key: env::var("BYBIT_API_KEY")?,
            bybit_api_secret: env::var("BYBIT_API_SECRET")?,
            okx_api_key: env::var("OKX_API_KEY")?,
            okx_api_secret: env::var("OKX_API_SECRET")?,

            database_url: env::var("DATABASE_URL")?,

            prometheus_port: env::var("PROMETHEUS_PORT")?.parse()?,
            metrics_enabled: env::var("METRICS_ENABLED")?.parse()?,

            min_trade_size: env::var("MIN_TRADE_SIZE")?.parse()?,
            max_trade_size: env::var("MAX_TRADE_SIZE")?.parse()?,
            price_impact_threshold: env::var("PRICE_IMPACT_THRESHOLD")?.parse()?,
            slippage_tolerance: env::var("SLIPPAGE_TOLERANCE")?.parse()?,
            rebalance_threshold: env::var("REBALANCE_THRESHOLD")?.parse()?,

            log_level: env::var("LOG_LEVEL")?,
            log_file_path: PathBuf::from(env::var("LOG_FILE_PATH")?),
        })
    }
} 