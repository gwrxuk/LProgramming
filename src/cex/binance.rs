use anyhow::Result;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{CexClient, OrderBook, PriceLevel, Trade};

pub struct BinanceClient {
    client: Client,
    api_key: String,
    api_secret: String,
    base_url: String,
}

impl BinanceClient {
    pub fn new(api_key: String, api_secret: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            api_key,
            api_secret,
            base_url: "https://api.binance.com".to_string(),
        })
    }

    fn generate_signature(&self, query_string: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: Option<&[(&str, &str)]>,
        signed: bool,
    ) -> Result<T> {
        let mut url = format!("{}{}", self.base_url, endpoint);
        
        if let Some(params) = params {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            
            if signed {
                let signature = self.generate_signature(&query_string);
                url = format!("{}?{}&signature={}", url, query_string, signature);
            } else {
                url = format!("{}?{}", url, query_string);
            }
        }

        let mut request = self.client.get(&url);
        if signed {
            request = request.header("X-MBX-APIKEY", &self.api_key);
        }

        let response = request.send().await?;
        let data = response.json::<T>().await?;
        Ok(data)
    }
}

#[async_trait]
impl CexClient for BinanceClient {
    async fn get_order_book(&self, symbol: &str) -> Result<OrderBook> {
        #[derive(Deserialize)]
        struct BinanceOrderBook {
            bids: Vec<[String; 2]>,
            asks: Vec<[String; 2]>,
            lastUpdateId: u64,
        }

        let endpoint = format!("/api/v3/depth?symbol={}&limit=100", symbol);
        let binance_ob: BinanceOrderBook = self.make_request(&endpoint, None, false).await?;

        let bids = binance_ob
            .bids
            .into_iter()
            .map(|[price, qty]| PriceLevel {
                price: price.parse().unwrap(),
                quantity: qty.parse().unwrap(),
            })
            .collect();

        let asks = binance_ob
            .asks
            .into_iter()
            .map(|[price, qty]| PriceLevel {
                price: price.parse().unwrap(),
                quantity: qty.parse().unwrap(),
            })
            .collect();

        Ok(OrderBook {
            bids,
            asks,
            timestamp: binance_ob.lastUpdateId,
        })
    }

    async fn get_ticker(&self, symbol: &str) -> Result<f64> {
        #[derive(Deserialize)]
        struct BinanceTicker {
            price: String,
        }

        let endpoint = format!("/api/v3/ticker/price?symbol={}", symbol);
        let ticker: BinanceTicker = self.make_request(&endpoint, None, false).await?;
        Ok(ticker.price.parse()?)
    }

    async fn place_order(
        &self,
        symbol: &str,
        side: &str,
        price: f64,
        quantity: f64,
    ) -> Result<String> {
        let timestamp = Self::get_timestamp();
        let params = &[
            ("symbol", symbol),
            ("side", side),
            ("type", "LIMIT"),
            ("timeInForce", "GTC"),
            ("price", &price.to_string()),
            ("quantity", &quantity.to_string()),
            ("timestamp", &timestamp.to_string()),
        ];

        #[derive(Deserialize)]
        struct OrderResponse {
            orderId: String,
        }

        let endpoint = "/api/v3/order";
        let response: OrderResponse = self.make_request(endpoint, Some(params), true).await?;
        Ok(response.orderId)
    }

    async fn cancel_order(&self, symbol: &str, order_id: &str) -> Result<()> {
        let timestamp = Self::get_timestamp();
        let params = &[
            ("symbol", symbol),
            ("orderId", order_id),
            ("timestamp", &timestamp.to_string()),
        ];

        let endpoint = "/api/v3/order";
        self.make_request::<()>(endpoint, Some(params), true).await?;
        Ok(())
    }

    async fn get_balance(&self, asset: &str) -> Result<f64> {
        let timestamp = Self::get_timestamp();
        let params = &[("timestamp", &timestamp.to_string())];

        #[derive(Deserialize)]
        struct Balance {
            asset: String,
            free: String,
            locked: String,
        }

        #[derive(Deserialize)]
        struct AccountInfo {
            balances: Vec<Balance>,
        }

        let endpoint = "/api/v3/account";
        let account: AccountInfo = self.make_request(endpoint, Some(params), true).await?;

        let balance = account
            .balances
            .into_iter()
            .find(|b| b.asset == asset)
            .ok_or_else(|| anyhow::anyhow!("Asset not found"))?;

        Ok(balance.free.parse()?)
    }

    async fn get_recent_trades(&self, symbol: &str) -> Result<Vec<Trade>> {
        #[derive(Deserialize)]
        struct BinanceTrade {
            id: u64,
            price: String,
            qty: String,
            time: u64,
            isBuyerMaker: bool,
        }

        let endpoint = format!("/api/v3/trades?symbol={}&limit=100", symbol);
        let trades: Vec<BinanceTrade> = self.make_request(&endpoint, None, false).await?;

        Ok(trades
            .into_iter()
            .map(|t| Trade {
                id: t.id.to_string(),
                symbol: symbol.to_string(),
                side: if t.isBuyerMaker { "SELL" } else { "BUY" }.to_string(),
                price: t.price.parse().unwrap(),
                quantity: t.qty.parse().unwrap(),
                timestamp: t.time,
            })
            .collect())
    }
} 