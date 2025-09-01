use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log::{info, debug, warn};
use metrics::{counter, gauge, histogram};
use crate::{Result, BlockchainError, Blockchain};

/// Blockchain metrics and monitoring data
/// 
/// Tracks various performance and operational metrics
/// for monitoring the blockchain's health and performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetrics {
    /// Total number of blocks in the chain
    pub total_blocks: u64,
    /// Total number of transactions processed
    pub total_transactions: u64,
    /// Current pending transactions
    pub pending_transactions: u64,
    /// Average block mining time in milliseconds
    pub avg_mining_time_ms: f64,
    /// Total mining time in milliseconds
    pub total_mining_time_ms: u64,
    /// Number of successful mines
    pub successful_mines: u64,
    /// Number of failed mining attempts
    pub failed_mines: u64,
    /// Current mining difficulty
    pub current_difficulty: u32,
    /// Total blockchain size in bytes
    pub blockchain_size_bytes: u64,
    /// Average transactions per block
    pub avg_transactions_per_block: f64,
    /// Last block timestamp
    pub last_block_timestamp: i64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Network latency metrics
    pub network_metrics: NetworkMetrics,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Network-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Number of connected peers
    pub connected_peers: u32,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Average message latency in milliseconds
    pub avg_message_latency_ms: f64,
    /// Failed connection attempts
    pub failed_connections: u64,
    /// Successful connections
    pub successful_connections: u64,
}

/// Performance-related metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average transaction processing time in milliseconds
    pub avg_transaction_time_ms: f64,
    /// Average block validation time in milliseconds
    pub avg_validation_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk usage in bytes
    pub disk_usage_bytes: u64,
}

/// Blockchain monitor for tracking metrics and performance
/// 
/// Provides real-time monitoring capabilities for the blockchain
/// and exposes metrics for external monitoring systems.
#[derive(Debug)]
pub struct BlockchainMonitor {
    /// Start time of the monitor
    start_time: Instant,
    /// Current metrics
    metrics: BlockchainMetrics,
    /// Mining time history for averaging
    mining_times: Vec<Duration>,
    /// Transaction processing times
    transaction_times: Vec<Duration>,
    /// Block validation times
    validation_times: Vec<Duration>,
    /// Message latencies
    message_latencies: Vec<Duration>,
}

impl BlockchainMonitor {
    /// Create a new blockchain monitor
    /// 
    /// # Returns
    /// * `BlockchainMonitor` - The new monitor instance
    /// 
    /// # Example
    /// ```
    /// use gillean::monitor::BlockchainMonitor;
    /// 
    /// let monitor = BlockchainMonitor::new();
    /// let metrics = monitor.get_metrics();
    /// assert_eq!(metrics.total_blocks, 0);
    /// ```
    pub fn new() -> Self {
        let monitor = BlockchainMonitor {
            start_time: Instant::now(),
            metrics: BlockchainMetrics {
                total_blocks: 0,
                total_transactions: 0,
                pending_transactions: 0,
                avg_mining_time_ms: 0.0,
                total_mining_time_ms: 0,
                successful_mines: 0,
                failed_mines: 0,
                current_difficulty: 4,
                blockchain_size_bytes: 0,
                avg_transactions_per_block: 0.0,
                last_block_timestamp: 0,
                uptime_seconds: 0,
                network_metrics: NetworkMetrics {
                    connected_peers: 0,
                    messages_sent: 0,
                    messages_received: 0,
                    avg_message_latency_ms: 0.0,
                    failed_connections: 0,
                    successful_connections: 0,
                },
                performance_metrics: PerformanceMetrics {
                    avg_transaction_time_ms: 0.0,
                    avg_validation_time_ms: 0.0,
                    memory_usage_bytes: 0,
                    cpu_usage_percent: 0.0,
                    disk_usage_bytes: 0,
                },
            },
            mining_times: Vec::new(),
            transaction_times: Vec::new(),
            validation_times: Vec::new(),
            message_latencies: Vec::new(),
        };

        info!("Blockchain monitor initialized");
        monitor
    }

    /// Update metrics from a blockchain instance
    /// 
    /// # Arguments
    /// * `blockchain` - The blockchain to get metrics from
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn update_from_blockchain(&mut self, blockchain: &Blockchain) -> Result<()> {
        let stats = blockchain.get_stats();
        
        self.metrics.total_blocks = stats.block_count as u64;
        self.metrics.total_transactions = stats.total_transactions as u64;
        self.metrics.pending_transactions = stats.pending_transactions as u64;
        self.metrics.current_difficulty = stats.difficulty;
        self.metrics.blockchain_size_bytes = stats.chain_size as u64;
        self.metrics.avg_transactions_per_block = if stats.block_count > 0 {
            stats.total_transactions as f64 / stats.block_count as f64
        } else {
            0.0
        };

        if let Ok(latest_block) = blockchain.get_latest_block() {
            self.metrics.last_block_timestamp = latest_block.timestamp;
        }

        self.metrics.uptime_seconds = self.start_time.elapsed().as_secs();

        // Update Prometheus metrics
        counter!("blockchain.blocks.total", self.metrics.total_blocks);
        counter!("blockchain.transactions.total", self.metrics.total_transactions);
        gauge!("blockchain.pending_transactions", self.metrics.pending_transactions as f64);
        gauge!("blockchain.difficulty", self.metrics.current_difficulty as f64);
        gauge!("blockchain.size_bytes", self.metrics.blockchain_size_bytes as f64);
        gauge!("blockchain.uptime_seconds", self.metrics.uptime_seconds as f64);

        debug!("Updated blockchain metrics: {} blocks, {} transactions", 
               self.metrics.total_blocks, self.metrics.total_transactions);
        
        Ok(())
    }

    /// Record a successful mining operation
    /// 
    /// # Arguments
    /// * `mining_time` - Time taken to mine the block
    /// * `difficulty` - Mining difficulty used
    /// 
    /// # Example
    /// ```
    /// use gillean::monitor::BlockchainMonitor;
    /// use std::time::Duration;
    /// 
    /// let mut monitor = BlockchainMonitor::new();
    /// monitor.record_successful_mine(Duration::from_millis(1000), 4);
    /// let metrics = monitor.get_metrics();
    /// assert_eq!(metrics.successful_mines, 1);
    /// ```
    pub fn record_successful_mine(&mut self, mining_time: Duration, difficulty: u32) {
        self.metrics.successful_mines += 1;
        self.metrics.total_mining_time_ms += mining_time.as_millis() as u64;
        self.mining_times.push(mining_time);

        // Keep only last 100 mining times for averaging
        if self.mining_times.len() > 100 {
            self.mining_times.remove(0);
        }

        self.metrics.avg_mining_time_ms = self.mining_times.iter()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / self.mining_times.len() as f64;

        // Update Prometheus metrics
        counter!("blockchain.mines.successful", 1);
        histogram!("blockchain.mining_time_ms", mining_time.as_millis() as f64);
        gauge!("blockchain.avg_mining_time_ms", self.metrics.avg_mining_time_ms);

        info!("Recorded successful mine: {}ms (difficulty: {})", 
              mining_time.as_millis(), difficulty);
    }

    /// Record a failed mining attempt
    /// 
    /// # Arguments
    /// * `reason` - Reason for the failure
    pub fn record_failed_mine(&mut self, reason: &str) {
        self.metrics.failed_mines += 1;
        counter!("blockchain.mines.failed", 1);
        warn!("Recorded failed mine: {}", reason);
    }

    /// Record transaction processing time
    /// 
    /// # Arguments
    /// * `processing_time` - Time taken to process the transaction
    pub fn record_transaction_time(&mut self, processing_time: Duration) {
        self.transaction_times.push(processing_time);

        // Keep only last 1000 transaction times
        if self.transaction_times.len() > 1000 {
            self.transaction_times.remove(0);
        }

        self.metrics.performance_metrics.avg_transaction_time_ms = self.transaction_times.iter()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / self.transaction_times.len() as f64;

        histogram!("blockchain.transaction_time_ms", processing_time.as_millis() as f64);
    }

    /// Record block validation time
    /// 
    /// # Arguments
    /// * `validation_time` - Time taken to validate the block
    pub fn record_validation_time(&mut self, validation_time: Duration) {
        self.validation_times.push(validation_time);

        // Keep only last 100 validation times
        if self.validation_times.len() > 100 {
            self.validation_times.remove(0);
        }

        self.metrics.performance_metrics.avg_validation_time_ms = self.validation_times.iter()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / self.validation_times.len() as f64;

        histogram!("blockchain.validation_time_ms", validation_time.as_millis() as f64);
    }

    /// Record network message latency
    /// 
    /// # Arguments
    /// * `latency` - Message latency
    pub fn record_message_latency(&mut self, latency: Duration) {
        self.message_latencies.push(latency);

        // Keep only last 1000 latencies
        if self.message_latencies.len() > 1000 {
            self.message_latencies.remove(0);
        }

        self.metrics.network_metrics.avg_message_latency_ms = self.message_latencies.iter()
            .map(|d| d.as_millis() as f64)
            .sum::<f64>() / self.message_latencies.len() as f64;

        histogram!("blockchain.message_latency_ms", latency.as_millis() as f64);
    }

    /// Record a successful network connection
    pub fn record_successful_connection(&mut self) {
        self.metrics.network_metrics.successful_connections += 1;
        counter!("blockchain.network.connections.successful", 1);
    }

    /// Record a failed network connection
    pub fn record_failed_connection(&mut self) {
        self.metrics.network_metrics.failed_connections += 1;
        counter!("blockchain.network.connections.failed", 1);
    }

    /// Update connected peers count
    /// 
    /// # Arguments
    /// * `peer_count` - Number of connected peers
    pub fn update_peer_count(&mut self, peer_count: u32) {
        self.metrics.network_metrics.connected_peers = peer_count;
        gauge!("blockchain.network.peers.connected", peer_count as f64);
    }

    /// Record a sent message
    pub fn record_message_sent(&mut self) {
        self.metrics.network_metrics.messages_sent += 1;
        counter!("blockchain.network.messages.sent", 1);
    }

    /// Record a received message
    pub fn record_message_received(&mut self) {
        self.metrics.network_metrics.messages_received += 1;
        counter!("blockchain.network.messages.received", 1);
    }

    /// Get current metrics
    /// 
    /// # Returns
    /// * `BlockchainMetrics` - Current metrics snapshot
    pub fn get_metrics(&self) -> BlockchainMetrics {
        self.metrics.clone()
    }

    /// Get metrics as JSON string
    /// 
    /// # Returns
    /// * `Result<String>` - JSON representation of metrics
    pub fn get_metrics_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.metrics).map_err(BlockchainError::from)
    }

    /// Get a summary of current metrics
    /// 
    /// # Returns
    /// * `String` - Human-readable metrics summary
    pub fn get_summary(&self) -> String {
        format!(
            "Blockchain Monitor Summary:\n\
             ├── Blocks: {}\n\
             ├── Transactions: {} ({} pending)\n\
             ├── Mining: {} successful, {} failed (avg: {:.2}ms)\n\
             ├── Difficulty: {}\n\
             ├── Chain Size: {} bytes\n\
             ├── Uptime: {}s\n\
             ├── Network: {} peers, {:.2}ms avg latency\n\
             └── Performance: {:.2}ms avg tx, {:.2}ms avg validation",
            self.metrics.total_blocks,
            self.metrics.total_transactions,
            self.metrics.pending_transactions,
            self.metrics.successful_mines,
            self.metrics.failed_mines,
            self.metrics.avg_mining_time_ms,
            self.metrics.current_difficulty,
            self.metrics.blockchain_size_bytes,
            self.metrics.uptime_seconds,
            self.metrics.network_metrics.connected_peers,
            self.metrics.network_metrics.avg_message_latency_ms,
            self.metrics.performance_metrics.avg_transaction_time_ms,
            self.metrics.performance_metrics.avg_validation_time_ms,
        )
    }

    /// Check if the blockchain is healthy
    /// 
    /// # Returns
    /// * `bool` - True if healthy, false otherwise
    pub fn is_healthy(&self) -> bool {
        // Basic health checks
        self.metrics.failed_mines < self.metrics.successful_mines * 10 && // Not too many failed mines
        self.metrics.avg_mining_time_ms < 30000.0 && // Average mining time under 30 seconds
        self.metrics.network_metrics.failed_connections < self.metrics.network_metrics.successful_connections * 5 // Not too many failed connections
    }

    /// Get health status with details
    /// 
    /// # Returns
    /// * `HealthStatus` - Detailed health status
    pub fn get_health_status(&self) -> HealthStatus {
        let mut issues = Vec::new();

        if self.metrics.failed_mines > self.metrics.successful_mines * 10 {
            issues.push("High mining failure rate".to_string());
        }

        if self.metrics.avg_mining_time_ms > 30000.0 {
            issues.push("Slow mining performance".to_string());
        }

        if self.metrics.network_metrics.failed_connections > self.metrics.network_metrics.successful_connections * 5 {
            issues.push("Network connectivity issues".to_string());
        }

        if self.metrics.performance_metrics.avg_transaction_time_ms > 1000.0 {
            issues.push("Slow transaction processing".to_string());
        }

        let status = if issues.is_empty() { "healthy" } else { "unhealthy" };

        HealthStatus {
            status: status.to_string(),
            issues,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}

impl Default for BlockchainMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Health status of the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: String,
    /// List of health issues
    pub issues: Vec<String>,
    /// Timestamp of the health check
    pub timestamp: u64,
}

/// Initialize metrics for the application
pub fn init_metrics() {
    // Initialize Prometheus metrics
    metrics::describe_counter!("blockchain.blocks.total", "Total number of blocks");
    metrics::describe_counter!("blockchain.transactions.total", "Total number of transactions");
    metrics::describe_gauge!("blockchain.pending_transactions", "Number of pending transactions");
    metrics::describe_gauge!("blockchain.difficulty", "Current mining difficulty");
    metrics::describe_gauge!("blockchain.size_bytes", "Blockchain size in bytes");
    metrics::describe_gauge!("blockchain.uptime_seconds", "Uptime in seconds");
    metrics::describe_counter!("blockchain.mines.successful", "Successful mining attempts");
    metrics::describe_counter!("blockchain.mines.failed", "Failed mining attempts");
    metrics::describe_histogram!("blockchain.mining_time_ms", "Mining time in milliseconds");
    metrics::describe_histogram!("blockchain.transaction_time_ms", "Transaction processing time");
    metrics::describe_histogram!("blockchain.validation_time_ms", "Block validation time");
    metrics::describe_histogram!("blockchain.message_latency_ms", "Network message latency");
    metrics::describe_counter!("blockchain.network.connections.successful", "Successful network connections");
    metrics::describe_counter!("blockchain.network.connections.failed", "Failed network connections");
    metrics::describe_gauge!("blockchain.network.peers.connected", "Number of connected peers");
    metrics::describe_counter!("blockchain.network.messages.sent", "Messages sent");
    metrics::describe_counter!("blockchain.network.messages.received", "Messages received");

    info!("Metrics initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_monitor_creation() {
        let monitor = BlockchainMonitor::new();
        let metrics = monitor.get_metrics();
        
        assert_eq!(metrics.total_blocks, 0);
        assert_eq!(metrics.total_transactions, 0);
        assert_eq!(metrics.successful_mines, 0);
        assert_eq!(metrics.failed_mines, 0);
    }

    #[test]
    fn test_successful_mine_recording() {
        let mut monitor = BlockchainMonitor::new();
        monitor.record_successful_mine(Duration::from_millis(1000), 4);
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.successful_mines, 1);
        assert_eq!(metrics.total_mining_time_ms, 1000);
        assert_eq!(metrics.avg_mining_time_ms, 1000.0);
    }

    #[test]
    fn test_failed_mine_recording() {
        let mut monitor = BlockchainMonitor::new();
        monitor.record_failed_mine("Timeout");
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.failed_mines, 1);
    }

    #[test]
    fn test_transaction_time_recording() {
        let mut monitor = BlockchainMonitor::new();
        monitor.record_transaction_time(Duration::from_millis(100));
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.performance_metrics.avg_transaction_time_ms, 100.0);
    }

    #[test]
    fn test_validation_time_recording() {
        let mut monitor = BlockchainMonitor::new();
        monitor.record_validation_time(Duration::from_millis(50));
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.performance_metrics.avg_validation_time_ms, 50.0);
    }

    #[test]
    fn test_network_metrics() {
        let mut monitor = BlockchainMonitor::new();
        
        monitor.record_successful_connection();
        monitor.record_failed_connection();
        monitor.update_peer_count(5);
        monitor.record_message_sent();
        monitor.record_message_received();
        monitor.record_message_latency(Duration::from_millis(10));
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.network_metrics.successful_connections, 1);
        assert_eq!(metrics.network_metrics.failed_connections, 1);
        assert_eq!(metrics.network_metrics.connected_peers, 5);
        assert_eq!(metrics.network_metrics.messages_sent, 1);
        assert_eq!(metrics.network_metrics.messages_received, 1);
        assert_eq!(metrics.network_metrics.avg_message_latency_ms, 10.0);
    }

    #[test]
    fn test_health_status() {
        let monitor = BlockchainMonitor::new();
        let health = monitor.get_health_status();
        
        assert_eq!(health.status, "healthy");
        assert!(health.issues.is_empty());
    }

    #[test]
    fn test_metrics_json() {
        let monitor = BlockchainMonitor::new();
        let json = monitor.get_metrics_json().unwrap();
        
        assert!(json.contains("total_blocks"));
        assert!(json.contains("total_transactions"));
    }

    #[test]
    fn test_summary() {
        let monitor = BlockchainMonitor::new();
        let summary = monitor.get_summary();
        
        assert!(summary.contains("Blockchain Monitor Summary"));
        assert!(summary.contains("Blocks: 0"));
    }
}
