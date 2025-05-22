use anyhow::Result;
use rand::Rng;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time;

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub num_wallets: usize,
    pub min_trade_size: f64,
    pub max_trade_size: f64,
    pub min_interval: Duration,
    pub max_interval: Duration,
    pub duration: Duration,
}

#[derive(Debug)]
pub struct SimulationResult {
    pub total_volume: f64,
    pub num_trades: usize,
    pub average_trade_size: f64,
    pub wallet_volumes: HashMap<Pubkey, f64>,
    pub execution_times: Vec<Duration>,
}

pub struct VolumeSimulator {
    config: SimulationConfig,
    wallets: Vec<Pubkey>,
    rng: rand::rngs::ThreadRng,
}

impl VolumeSimulator {
    pub fn new(config: SimulationConfig) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let wallets = (0..config.num_wallets)
            .map(|_| {
                let mut keypair = solana_sdk::signature::Keypair::new();
                keypair.pubkey()
            })
            .collect();

        Ok(Self {
            config,
            wallets,
            rng,
        })
    }

    pub async fn run_simulation(&mut self) -> Result<SimulationResult> {
        let start_time = Instant::now();
        let mut result = SimulationResult {
            total_volume: 0.0,
            num_trades: 0,
            average_trade_size: 0.0,
            wallet_volumes: HashMap::new(),
            execution_times: Vec::new(),
        };

        while start_time.elapsed() < self.config.duration {
            let wallet = self.select_random_wallet();
            let trade_size = self.generate_trade_size();
            let execution_time = self.simulate_trade_execution().await?;

            // Update results
            result.total_volume += trade_size;
            result.num_trades += 1;
            result.execution_times.push(execution_time);
            *result.wallet_volumes.entry(wallet).or_insert(0.0) += trade_size;

            // Wait for next trade
            let delay = self.generate_delay();
            time::sleep(delay).await;
        }

        result.average_trade_size = result.total_volume / result.num_trades as f64;
        Ok(result)
    }

    fn select_random_wallet(&mut self) -> Pubkey {
        let index = self.rng.gen_range(0..self.wallets.len());
        self.wallets[index]
    }

    fn generate_trade_size(&mut self) -> f64 {
        self.rng.gen_range(self.config.min_trade_size..self.config.max_trade_size)
    }

    fn generate_delay(&mut self) -> Duration {
        let millis = self.rng.gen_range(
            self.config.min_interval.as_millis()..self.config.max_interval.as_millis(),
        );
        Duration::from_millis(millis as u64)
    }

    async fn simulate_trade_execution(&mut self) -> Result<Duration> {
        // Simulate network latency and execution time
        let latency = self.rng.gen_range(50..200);
        time::sleep(Duration::from_millis(latency)).await;
        Ok(Duration::from_millis(latency))
    }
}

// Helper functions for simulation analysis
pub fn analyze_simulation_results(result: &SimulationResult) -> HashMap<String, f64> {
    let mut analysis = HashMap::new();
    
    // Calculate statistics
    analysis.insert("total_volume".to_string(), result.total_volume);
    analysis.insert("num_trades".to_string(), result.num_trades as f64);
    analysis.insert("average_trade_size".to_string(), result.average_trade_size);
    
    // Calculate execution time statistics
    let avg_execution_time: f64 = result.execution_times.iter()
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / result.execution_times.len() as f64;
    analysis.insert("average_execution_time_ms".to_string(), avg_execution_time);
    
    // Calculate volume distribution statistics
    let volumes: Vec<f64> = result.wallet_volumes.values().copied().collect();
    let avg_volume = volumes.iter().sum::<f64>() / volumes.len() as f64;
    analysis.insert("average_wallet_volume".to_string(), avg_volume);
    
    analysis
}

pub fn generate_simulation_report(result: &SimulationResult) -> String {
    let analysis = analyze_simulation_results(result);
    
    format!(
        "Simulation Report:\n\
         Total Volume: ${:.2}\n\
         Number of Trades: {}\n\
         Average Trade Size: ${:.2}\n\
         Average Execution Time: {:.2}ms\n\
         Average Wallet Volume: ${:.2}\n\
         Number of Wallets: {}",
        analysis["total_volume"],
        analysis["num_trades"],
        analysis["average_trade_size"],
        analysis["average_execution_time_ms"],
        analysis["average_wallet_volume"],
        result.wallet_volumes.len()
    )
} 