//! Production monitoring and observability
//! 
//! This module provides comprehensive monitoring, metrics collection,
//! alerting, and observability features for production deployments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tokio::time::interval;
use crate::error::Result;

/// Metrics collector for production monitoring
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, CounterMetric>>>,
    gauges: Arc<RwLock<HashMap<String, GaugeMetric>>>,
    histograms: Arc<RwLock<HashMap<String, HistogramMetric>>>,
    start_time: Instant,
}

/// Counter metric for counting events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterMetric {
    pub name: String,
    pub value: u64,
    pub labels: HashMap<String, String>,
    pub last_updated: SystemTime,
}

/// Gauge metric for measuring current values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeMetric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub last_updated: SystemTime,
}

/// Histogram metric for measuring distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramMetric {
    pub name: String,
    pub buckets: Vec<HistogramBucket>,
    pub count: u64,
    pub sum: f64,
    pub labels: HashMap<String, String>,
    pub last_updated: SystemTime,
}

/// Histogram bucket for distribution measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub email_recipients: Vec<String>,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    RateIncrease,
    RateDecrease,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// Alert state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {
    pub config: AlertConfig,
    pub is_firing: bool,
    pub last_fired: Option<SystemTime>,
    pub last_resolved: Option<SystemTime>,
    pub fire_count: u64,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub duration: Duration,
    pub last_check: SystemTime,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkIO,
    pub uptime: Duration,
    pub timestamp: SystemTime,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Blockchain-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetrics {
    pub block_height: u64,
    pub transaction_count: u64,
    pub pending_transactions: u64,
    pub peer_count: u32,
    pub sync_status: String,
    pub consensus_status: String,
    pub shard_count: u32,
    pub active_channels: u32,
    pub zkp_proofs_generated: u64,
    pub ai_analyses_completed: u64,
}

/// Monitoring dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub system_metrics: SystemMetrics,
    pub blockchain_metrics: BlockchainMetrics,
    pub health_checks: Vec<HealthCheck>,
    pub active_alerts: Vec<AlertState>,
    pub recent_events: Vec<MonitoringEvent>,
    pub timestamp: SystemTime,
}

/// Monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEvent {
    pub id: String,
    pub event_type: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: SystemTime,
    pub metadata: HashMap<String, String>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }
    
    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str, labels: HashMap<String, String>) -> Result<()> {
        let mut counters = self.counters.write().await;
        let key = format!("{}_{}", name, Self::labels_to_key(&labels));
        
        let counter = counters.entry(key.clone()).or_insert_with(|| CounterMetric {
            name: name.to_string(),
            value: 0,
            labels,
            last_updated: SystemTime::now(),
        });
        
        counter.value += 1;
        counter.last_updated = SystemTime::now();
        
        Ok(())
    }
    
    /// Set a gauge metric value
    pub async fn set_gauge(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<()> {
        let mut gauges = self.gauges.write().await;
        let key = format!("{}_{}", name, Self::labels_to_key(&labels));
        
        let gauge = gauges.entry(key.clone()).or_insert_with(|| GaugeMetric {
            name: name.to_string(),
            value: 0.0,
            labels,
            last_updated: SystemTime::now(),
        });
        
        gauge.value = value;
        gauge.last_updated = SystemTime::now();
        
        Ok(())
    }
    
    /// Record a histogram value
    pub async fn record_histogram(&self, name: &str, value: f64, labels: HashMap<String, String>) -> Result<()> {
        let mut histograms = self.histograms.write().await;
        let key = format!("{}_{}", name, Self::labels_to_key(&labels));
        
        let histogram = histograms.entry(key.clone()).or_insert_with(|| HistogramMetric {
            name: name.to_string(),
            buckets: vec![
                HistogramBucket { upper_bound: 0.1, count: 0 },
                HistogramBucket { upper_bound: 0.5, count: 0 },
                HistogramBucket { upper_bound: 1.0, count: 0 },
                HistogramBucket { upper_bound: 5.0, count: 0 },
                HistogramBucket { upper_bound: 10.0, count: 0 },
                HistogramBucket { upper_bound: f64::INFINITY, count: 0 },
            ],
            count: 0,
            sum: 0.0,
            labels,
            last_updated: SystemTime::now(),
        });
        
        histogram.count += 1;
        histogram.sum += value;
        
        // Update buckets
        for bucket in &mut histogram.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
            }
        }
        
        histogram.last_updated = SystemTime::now();
        
        Ok(())
    }
    
    /// Get all metrics
    pub async fn get_metrics(&self) -> Result<MetricsSnapshot> {
        let counters = self.counters.read().await;
        let gauges = self.gauges.read().await;
        let histograms = self.histograms.read().await;
        
        Ok(MetricsSnapshot {
            counters: counters.values().cloned().collect(),
            gauges: gauges.values().cloned().collect(),
            histograms: histograms.values().cloned().collect(),
            timestamp: SystemTime::now(),
            uptime: self.start_time.elapsed(),
        })
    }
    
    /// Get metrics in Prometheus format
    pub async fn get_prometheus_metrics(&self) -> Result<String> {
        let metrics = self.get_metrics().await?;
        let mut output = String::new();
        
        // Add counters
        for counter in &metrics.counters {
            output.push_str(&format!(
                "# HELP {} Counter metric\n# TYPE {} counter\n{}_total{{}} {}\n",
                counter.name, counter.name, counter.name, counter.value
            ));
        }
        
        // Add gauges
        for gauge in &metrics.gauges {
            output.push_str(&format!(
                "# HELP {} Gauge metric\n# TYPE {} gauge\n{} {{}} {}\n",
                gauge.name, gauge.name, gauge.name, gauge.value
            ));
        }
        
        // Add histograms
        for histogram in &metrics.histograms {
            output.push_str(&format!(
                "# HELP {} Histogram metric\n# TYPE {} histogram\n",
                histogram.name, histogram.name
            ));
            
            for bucket in &histogram.buckets {
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"}} {}\n",
                    histogram.name, bucket.upper_bound, bucket.count
                ));
            }
            
            output.push_str(&format!("{}_count {{}} {}\n", histogram.name, histogram.count));
            output.push_str(&format!("{}_sum {{}} {}\n", histogram.name, histogram.sum));
        }
        
        Ok(output)
    }
    
    fn labels_to_key(labels: &HashMap<String, String>) -> String {
        let mut sorted_labels: Vec<_> = labels.iter().collect();
        sorted_labels.sort_by_key(|(k, _)| *k);
        
        sorted_labels
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub counters: Vec<CounterMetric>,
    pub gauges: Vec<GaugeMetric>,
    pub histograms: Vec<HistogramMetric>,
    pub timestamp: SystemTime,
    pub uptime: Duration,
}

/// Alert manager for handling alerts
pub struct AlertManager {
    alerts: Arc<RwLock<HashMap<String, AlertState>>>,
    metrics_collector: Arc<MetricsCollector>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector,
        }
    }
    
    /// Add an alert configuration
    pub async fn add_alert(&self, config: AlertConfig) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        let alert_state = AlertState {
            config: config.clone(),
            is_firing: false,
            last_fired: None,
            last_resolved: None,
            fire_count: 0,
        };
        
        alerts.insert(config.name.clone(), alert_state);
        Ok(())
    }
    
    /// Check all alerts
    pub async fn check_alerts(&self) -> Result<Vec<AlertState>> {
        let mut alerts = self.alerts.write().await;
        let mut fired_alerts = Vec::new();
        
        for (_, alert_state) in alerts.iter_mut() {
            if !alert_state.config.enabled {
                continue;
            }
            
            let should_fire = self.evaluate_alert_condition(&alert_state.config).await?;
            
            if should_fire && !alert_state.is_firing {
                // Alert just started firing
                alert_state.is_firing = true;
                alert_state.last_fired = Some(SystemTime::now());
                alert_state.fire_count += 1;
                fired_alerts.push(alert_state.clone());
            } else if !should_fire && alert_state.is_firing {
                // Alert resolved
                alert_state.is_firing = false;
                alert_state.last_resolved = Some(SystemTime::now());
            }
        }
        
        Ok(fired_alerts)
    }
    
    /// Get all alert states
    pub async fn get_alerts(&self) -> Result<Vec<AlertState>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.values().cloned().collect())
    }
    
    async fn evaluate_alert_condition(&self, config: &AlertConfig) -> Result<bool> {
        let metrics = self.metrics_collector.get_metrics().await?;
        
        // Find the metric value
        let metric_value = match config.condition {
            AlertCondition::GreaterThan | AlertCondition::LessThan | 
            AlertCondition::Equal | AlertCondition::NotEqual => {
                // Look for gauge metric
                metrics.gauges
                    .iter()
                    .find(|g| g.name == config.metric_name)
                    .map(|g| g.value)
                    .unwrap_or(0.0)
            },
            AlertCondition::RateIncrease | AlertCondition::RateDecrease => {
                // Look for counter metric and calculate rate
                metrics.counters
                    .iter()
                    .find(|c| c.name == config.metric_name)
                    .map(|c| c.value as f64)
                    .unwrap_or(0.0)
            },
        };
        
        let result = match config.condition {
            AlertCondition::GreaterThan => metric_value > config.threshold,
            AlertCondition::LessThan => metric_value < config.threshold,
            AlertCondition::Equal => (metric_value - config.threshold).abs() < f64::EPSILON,
            AlertCondition::NotEqual => (metric_value - config.threshold).abs() >= f64::EPSILON,
            AlertCondition::RateIncrease => metric_value > config.threshold,
            AlertCondition::RateDecrease => metric_value < config.threshold,
        };
        
        Ok(result)
    }
}

/// Health check manager
pub struct HealthCheckManager {
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
}

impl HealthCheckManager {
    /// Create a new health check manager
    pub fn new() -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add a health check
    pub async fn add_health_check<F>(&self, name: &str, check_fn: F) -> Result<()>
    where
        F: Fn() -> Result<(HealthStatus, String)> + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let (status, message) = check_fn()?;
        let duration = start_time.elapsed();
        
        let health_check = HealthCheck {
            name: name.to_string(),
            status,
            message,
            duration,
            last_check: SystemTime::now(),
        };
        
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(name.to_string(), health_check);
        
        Ok(())
    }
    
    /// Run all health checks
    pub async fn run_health_checks(&self) -> Result<Vec<HealthCheck>> {
        let health_checks = self.health_checks.read().await;
        Ok(health_checks.values().cloned().collect())
    }
    
    /// Get overall health status
    pub async fn get_overall_health(&self) -> Result<HealthStatus> {
        let health_checks = self.health_checks.read().await;
        
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        
        for health_check in health_checks.values() {
            match health_check.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }
        
        if has_unhealthy {
            Ok(HealthStatus::Unhealthy)
        } else if has_degraded {
            Ok(HealthStatus::Degraded)
        } else {
            Ok(HealthStatus::Healthy)
        }
    }
}

/// Production monitoring manager
pub struct ProductionMonitor {
    metrics_collector: Arc<MetricsCollector>,
    alert_manager: Arc<AlertManager>,
    health_check_manager: Arc<HealthCheckManager>,
    events: Arc<RwLock<Vec<MonitoringEvent>>>,
}

impl ProductionMonitor {
    /// Create a new production monitor
    pub fn new() -> Self {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new(metrics_collector.clone()));
        let health_check_manager = Arc::new(HealthCheckManager::new());
        
        Self {
            metrics_collector,
            alert_manager,
            health_check_manager,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        // Start alert checking
        self.start_alert_checking().await?;
        
        // Start health checks
        self.start_health_checks().await?;
        
        Ok(())
    }
    
    /// Get dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let system_metrics = self.get_system_metrics().await?;
        let blockchain_metrics = self.get_blockchain_metrics().await?;
        let health_checks = self.health_check_manager.run_health_checks().await?;
        let active_alerts = self.alert_manager.get_alerts().await?
            .into_iter()
            .filter(|a| a.is_firing)
            .collect();
        let recent_events = self.get_recent_events().await?;
        
        Ok(DashboardData {
            system_metrics,
            blockchain_metrics,
            health_checks,
            active_alerts,
            recent_events,
            timestamp: SystemTime::now(),
        })
    }
    
    /// Add a monitoring event
    pub async fn add_event(&self, event: MonitoringEvent) -> Result<()> {
        let mut events = self.events.write().await;
        events.push(event);
        
        // Keep only last 1000 events
        if events.len() > 1000 {
            let to_remove = events.len() - 1000;
            events.drain(0..to_remove);
        }
        
        Ok(())
    }
    
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics_collector = self.metrics_collector.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Collect system metrics
                if let Ok(system_metrics) = Self::collect_system_metrics().await {
                    let _ = metrics_collector.set_gauge(
                        "system_cpu_usage",
                        system_metrics.cpu_usage,
                        HashMap::new(),
                    ).await;
                    
                    let _ = metrics_collector.set_gauge(
                        "system_memory_usage",
                        system_metrics.memory_usage,
                        HashMap::new(),
                    ).await;
                    
                    let _ = metrics_collector.set_gauge(
                        "system_disk_usage",
                        system_metrics.disk_usage,
                        HashMap::new(),
                    ).await;
                }
            }
        });
        
        Ok(())
    }
    
    async fn start_alert_checking(&self) -> Result<()> {
        let alert_manager = self.alert_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if let Ok(fired_alerts) = alert_manager.check_alerts().await {
                    for alert in fired_alerts {
                        // Handle fired alert (send notifications, etc.)
                        println!("Alert fired: {}", alert.config.name);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn start_health_checks(&self) -> Result<()> {
        let health_check_manager = self.health_check_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let _ = health_check_manager.run_health_checks().await;
            }
        });
        
        Ok(())
    }
    
    async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        // In a real implementation, this would collect actual system metrics
        Ok(SystemMetrics {
            cpu_usage: 25.5,
            memory_usage: 60.2,
            disk_usage: 45.8,
            network_io: NetworkIO {
                bytes_sent: 1024000,
                bytes_received: 2048000,
                packets_sent: 1500,
                packets_received: 2000,
            },
            uptime: Duration::from_secs(86400), // 24 hours
            timestamp: SystemTime::now(),
        })
    }
    
    async fn get_blockchain_metrics(&self) -> Result<BlockchainMetrics> {
        // In a real implementation, this would collect actual blockchain metrics
        Ok(BlockchainMetrics {
            block_height: 12345,
            transaction_count: 98765,
            pending_transactions: 150,
            peer_count: 25,
            sync_status: "synced".to_string(),
            consensus_status: "active".to_string(),
            shard_count: 8,
            active_channels: 12,
            zkp_proofs_generated: 5432,
            ai_analyses_completed: 8765,
        })
    }
    
    async fn get_recent_events(&self) -> Result<Vec<MonitoringEvent>> {
        let events = self.events.read().await;
        Ok(events.clone())
    }
    
    async fn collect_system_metrics() -> Result<SystemMetrics> {
        // Placeholder for actual system metrics collection
        Ok(SystemMetrics {
            cpu_usage: 25.5,
            memory_usage: 60.2,
            disk_usage: 45.8,
            network_io: NetworkIO {
                bytes_sent: 1024000,
                bytes_received: 2048000,
                packets_sent: 1500,
                packets_received: 2000,
            },
            uptime: Duration::from_secs(86400),
            timestamp: SystemTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // Test counter
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        collector.increment_counter("http_requests", labels).await.unwrap();
        
        // Test gauge
        let mut labels = HashMap::new();
        labels.insert("instance".to_string(), "node1".to_string());
        collector.set_gauge("memory_usage", 75.5, labels).await.unwrap();
        
        // Test histogram
        let mut labels = HashMap::new();
        labels.insert("operation".to_string(), "block_processing".to_string());
        collector.record_histogram("operation_duration", 0.5, labels).await.unwrap();
        
        let metrics = collector.get_metrics().await.unwrap();
        assert!(!metrics.counters.is_empty());
        assert!(!metrics.gauges.is_empty());
        assert!(!metrics.histograms.is_empty());
    }
    
    #[tokio::test]
    async fn test_alert_manager() {
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = AlertManager::new(metrics_collector.clone());
        
        // Set a gauge value
        metrics_collector.set_gauge("cpu_usage", 90.0, HashMap::new()).await.unwrap();
        
        // Add an alert
        let alert_config = AlertConfig {
            name: "high_cpu".to_string(),
            metric_name: "cpu_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 80.0,
            duration: Duration::from_secs(60),
            severity: AlertSeverity::Warning,
            enabled: true,
            webhook_url: None,
            email_recipients: vec![],
        };
        
        alert_manager.add_alert(alert_config).await.unwrap();
        
        // Check alerts
        let fired_alerts = alert_manager.check_alerts().await.unwrap();
        assert!(!fired_alerts.is_empty());
    }
    
    #[tokio::test]
    async fn test_health_check_manager() {
        let health_manager = HealthCheckManager::new();
        
        health_manager.add_health_check("database", || {
            Ok((HealthStatus::Healthy, "Database connection OK".to_string()))
        }).await.unwrap();
        
        health_manager.add_health_check("network", || {
            Ok((HealthStatus::Degraded, "Network latency high".to_string()))
        }).await.unwrap();
        
        let health_checks = health_manager.run_health_checks().await.unwrap();
        assert_eq!(health_checks.len(), 2);
        
        let overall_health = health_manager.get_overall_health().await.unwrap();
        assert_eq!(overall_health, HealthStatus::Degraded);
    }
    
    #[tokio::test]
    async fn test_production_monitor() {
        let monitor = ProductionMonitor::new();
        
        let event = MonitoringEvent {
            id: "test_event".to_string(),
            event_type: "test".to_string(),
            severity: AlertSeverity::Info,
            message: "Test event".to_string(),
            timestamp: SystemTime::now(),
            metadata: HashMap::new(),
        };
        
        monitor.add_event(event).await.unwrap();
        
        let dashboard_data = monitor.get_dashboard_data().await.unwrap();
        assert_eq!(dashboard_data.recent_events.len(), 1);
    }
}
