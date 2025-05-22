use anyhow::Result;
use metrics::{counter, gauge, histogram};
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct MetricsManager {
    registry: Arc<Registry>,
    trade_metrics: Arc<RwLock<TradeMetrics>>,
    lp_metrics: Arc<RwLock<LpMetrics>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Default)]
struct TradeMetrics {
    total_volume: f64,
    num_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    average_trade_size: f64,
    trade_latencies: Vec<Duration>,
}

#[derive(Default)]
struct LpMetrics {
    total_lp_value: f64,
    num_positions: u64,
    total_fees_earned: f64,
    position_values: HashMap<String, f64>,
    rebalance_count: u64,
}

#[derive(Default)]
struct PerformanceMetrics {
    start_time: Option<Instant>,
    uptime: Duration,
    memory_usage: u64,
    cpu_usage: f64,
    error_count: u64,
}

impl MetricsManager {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Register Prometheus metrics
        register_metrics(&registry)?;

        Ok(Self {
            registry: Arc::new(registry),
            trade_metrics: Arc::new(RwLock::new(TradeMetrics::default())),
            lp_metrics: Arc::new(RwLock::new(LpMetrics::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        })
    }

    pub async fn record_trade(
        &self,
        volume: f64,
        success: bool,
        latency: Duration,
    ) -> Result<()> {
        let mut metrics = self.trade_metrics.write().await;
        
        metrics.total_volume += volume;
        metrics.num_trades += 1;
        if success {
            metrics.successful_trades += 1;
        } else {
            metrics.failed_trades += 1;
        }
        metrics.trade_latencies.push(latency);
        metrics.average_trade_size = metrics.total_volume / metrics.num_trades as f64;

        // Update Prometheus metrics
        counter!("trades_total", 1.0);
        counter!("trades_volume", volume);
        histogram!("trade_latency_seconds", latency.as_secs_f64());
        
        Ok(())
    }

    pub async fn record_lp_position(
        &self,
        position_id: String,
        value: f64,
        fees_earned: f64,
    ) -> Result<()> {
        let mut metrics = self.lp_metrics.write().await;
        
        metrics.total_lp_value += value;
        metrics.total_fees_earned += fees_earned;
        metrics.position_values.insert(position_id, value);

        // Update Prometheus metrics
        gauge!("lp_total_value", metrics.total_lp_value);
        gauge!("lp_fees_earned", metrics.total_fees_earned);
        
        Ok(())
    }

    pub async fn record_rebalance(&self) -> Result<()> {
        let mut metrics = self.lp_metrics.write().await;
        metrics.rebalance_count += 1;
        counter!("lp_rebalances_total", 1.0);
        Ok(())
    }

    pub async fn update_performance_metrics(
        &self,
        memory_usage: u64,
        cpu_usage: f64,
    ) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        
        if metrics.start_time.is_none() {
            metrics.start_time = Some(Instant::now());
        }
        
        metrics.memory_usage = memory_usage;
        metrics.cpu_usage = cpu_usage;
        metrics.uptime = metrics.start_time.unwrap().elapsed();

        // Update Prometheus metrics
        gauge!("memory_usage_bytes", memory_usage as f64);
        gauge!("cpu_usage_percent", cpu_usage);
        gauge!("uptime_seconds", metrics.uptime.as_secs_f64());
        
        Ok(())
    }

    pub async fn record_error(&self) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        metrics.error_count += 1;
        counter!("errors_total", 1.0);
        Ok(())
    }

    pub async fn get_metrics_report(&self) -> Result<String> {
        let mut buffer = Vec::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }
}

fn register_metrics(registry: &Registry) -> Result<()> {
    // Register trade metrics
    registry.register(Box::new(counter!(
        "trades_total",
        "Total number of trades executed"
    )))?;
    registry.register(Box::new(counter!(
        "trades_volume",
        "Total trading volume"
    )))?;
    registry.register(Box::new(histogram!(
        "trade_latency_seconds",
        "Trade execution latency in seconds"
    )))?;

    // Register LP metrics
    registry.register(Box::new(gauge!(
        "lp_total_value",
        "Total value of LP positions"
    )))?;
    registry.register(Box::new(gauge!(
        "lp_fees_earned",
        "Total fees earned from LP positions"
    )))?;
    registry.register(Box::new(counter!(
        "lp_rebalances_total",
        "Total number of LP position rebalances"
    )))?;

    // Register performance metrics
    registry.register(Box::new(gauge!(
        "memory_usage_bytes",
        "Memory usage in bytes"
    )))?;
    registry.register(Box::new(gauge!(
        "cpu_usage_percent",
        "CPU usage percentage"
    )))?;
    registry.register(Box::new(gauge!(
        "uptime_seconds",
        "Bot uptime in seconds"
    )))?;
    registry.register(Box::new(counter!(
        "errors_total",
        "Total number of errors encountered"
    )))?;

    Ok(())
}

// Helper functions for metrics analysis
pub fn calculate_success_rate(successful: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        successful as f64 / total as f64
    }
}

pub fn calculate_average_latency(latencies: &[Duration]) -> Duration {
    if latencies.is_empty() {
        Duration::from_secs(0)
    } else {
        let total: Duration = latencies.iter().sum();
        total / latencies.len() as u32
    }
}

pub fn format_metrics_for_dashboard(metrics: &MetricsManager) -> String {
    // This would format the metrics in a way suitable for your dashboard
    // You might want to convert this to JSON or another format depending on your dashboard
    format!(
        "Metrics Summary:\n\
         Total Trades: {}\n\
         Success Rate: {:.2}%\n\
         Average Latency: {:.2}ms\n\
         Total LP Value: ${:.2}\n\
         Total Fees Earned: ${:.2}\n\
         Uptime: {:.2} hours",
        metrics.trade_metrics.blocking_read().num_trades,
        calculate_success_rate(
            metrics.trade_metrics.blocking_read().successful_trades,
            metrics.trade_metrics.blocking_read().num_trades
        ) * 100.0,
        calculate_average_latency(&metrics.trade_metrics.blocking_read().trade_latencies)
            .as_millis(),
        metrics.lp_metrics.blocking_read().total_lp_value,
        metrics.lp_metrics.blocking_read().total_fees_earned,
        metrics.performance_metrics.blocking_read().uptime.as_secs_f64() / 3600.0
    )
} 