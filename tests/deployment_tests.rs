//! Tests for production deployment functionality

use gillean::deployment::*;
use gillean::monitoring::*;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::Duration;

#[tokio::test]
async fn test_deployment_config_creation() {
    let config = DeploymentManager::default_config();
    assert_eq!(config.environment, DeploymentEnvironment::Development);
    assert!(config.features.zkp_enabled);
    assert!(config.features.state_channels_enabled);
    assert!(config.features.sharding_enabled);
    assert!(config.features.cross_chain_enabled);
    assert!(config.features.ai_integration_enabled);
    assert!(config.features.mobile_support_enabled);
    assert!(config.features.performance_optimization_enabled);
    assert!(config.features.security_enhancements_enabled);
    assert!(config.features.developer_tools_enabled);
}

#[tokio::test]
async fn test_testnet_config() {
    let config = DeploymentManager::testnet_config();
    assert_eq!(config.environment, DeploymentEnvironment::Testnet);
    assert!(!config.network.bootstrap_nodes.is_empty());
    assert_eq!(config.consensus.block_time, Duration::from_secs(6));
    assert_eq!(config.consensus.validator_count, 7);
    assert_eq!(config.consensus.stake_requirement, 100000);
}

#[tokio::test]
async fn test_mainnet_config() {
    let config = DeploymentManager::mainnet_config();
    assert_eq!(config.environment, DeploymentEnvironment::Mainnet);
    assert!(config.security.tls_enabled);
    assert!(config.monitoring.alerting_enabled);
    assert!(config.database.backup_enabled);
    assert_eq!(config.database.backup_interval, Duration::from_secs(1800));
}

#[tokio::test]
async fn test_deployment_manager_operations() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    let mut manager = DeploymentManager::new(config_path.clone());
    
    // Test configuration update
    manager.update_config(|config| {
        config.environment = DeploymentEnvironment::Testnet;
        config.network.listen_address = "0.0.0.0:30304".to_string();
    });
    
    // Test configuration save and load
    manager.save_config().await.unwrap();
    
    let mut new_manager = DeploymentManager::new(config_path);
    new_manager.load_config().await.unwrap();
    
    assert_eq!(new_manager.config().environment, DeploymentEnvironment::Testnet);
    assert_eq!(new_manager.config().network.listen_address, "0.0.0.0:30304");
}

#[tokio::test]
async fn test_deployment_validation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    let mut manager = DeploymentManager::new(config_path);
    
    // Test valid configuration
    assert!(manager.validate_config().is_ok());
    
    // Test invalid configuration (TLS enabled but no certificates)
    manager.update_config(|config| {
        config.security.tls_enabled = true;
        config.security.tls_cert_path = None;
        config.security.tls_key_path = None;
    });
    
    assert!(manager.validate_config().is_err());
}

#[tokio::test]
async fn test_deployment_environment_initialization() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let data_dir = temp_dir.path().join("data");
    
    let mut manager = DeploymentManager::new(config_path);
    manager.update_config(|config| {
        config.database.data_dir = data_dir.clone();
    });
    
    // Initialize environment
    manager.initialize_environment().await.unwrap();
    
    // Check that data directory was created
    assert!(data_dir.exists());
    assert!(data_dir.is_dir());
}

#[tokio::test]
async fn test_deployment_status() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let data_dir = temp_dir.path().join("data");
    
    let mut manager = DeploymentManager::new(config_path);
    manager.update_config(|config| {
        config.database.data_dir = data_dir.clone();
    });
    
    // Initialize environment
    manager.initialize_environment().await.unwrap();
    
    // Get deployment status
    let status = manager.get_deployment_status().await.unwrap();
    
    assert_eq!(status.environment, DeploymentEnvironment::Development);
    assert!(status.data_directory_exists);
    assert!(status.config_valid);
    assert!(status.is_initialized);
}

#[tokio::test]
async fn test_deployment_scripts_generation() {
    let config = DeploymentManager::default_config();
    let scripts = DeploymentUtils::generate_deployment_scripts(&config).unwrap();
    
    assert!(!scripts.is_empty());
    
    // Check for expected scripts
    let script_names: Vec<&String> = scripts.iter().map(|s| &s.name).collect();
    assert!(script_names.contains(&&"gillean.service".to_string()));
    assert!(script_names.contains(&&"docker-compose.yml".to_string()));
    
    // Check script content
    let systemd_script = scripts.iter().find(|s| s.name == "gillean.service").unwrap();
    assert!(systemd_script.content.contains("[Unit]"));
    assert!(systemd_script.content.contains("Description=Gillean Blockchain Node"));
    
    let docker_script = scripts.iter().find(|s| s.name == "docker-compose.yml").unwrap();
    assert!(docker_script.content.contains("version: '3.8'"));
    assert!(docker_script.content.contains("gillean"));
}

#[tokio::test]
async fn test_metrics_collector() {
    let collector = MetricsCollector::new();
    
    // Test counter metrics
    let mut labels = HashMap::new();
    labels.insert("method".to_string(), "GET".to_string());
    collector.increment_counter("http_requests", labels).await.unwrap();
    
    // Test gauge metrics
    let mut labels = HashMap::new();
    labels.insert("instance".to_string(), "node1".to_string());
    collector.set_gauge("memory_usage", 75.5, labels).await.unwrap();
    
    // Test histogram metrics
    let mut labels = HashMap::new();
    labels.insert("operation".to_string(), "block_processing".to_string());
    collector.record_histogram("operation_duration", 0.5, labels).await.unwrap();
    
    // Get metrics snapshot
    let metrics = collector.get_metrics().await.unwrap();
    assert!(!metrics.counters.is_empty());
    assert!(!metrics.gauges.is_empty());
    assert!(!metrics.histograms.is_empty());
    
    // Test Prometheus format
    let prometheus_metrics = collector.get_prometheus_metrics().await.unwrap();
    assert!(prometheus_metrics.contains("http_requests"));
    assert!(prometheus_metrics.contains("memory_usage"));
    assert!(prometheus_metrics.contains("operation_duration"));
}

#[tokio::test]
async fn test_alert_manager() {
    let metrics_collector = Arc::new(MetricsCollector::new());
    let alert_manager = AlertManager::new(metrics_collector.clone());
    
    // Set a gauge value that will trigger an alert
    metrics_collector.set_gauge("cpu_usage", 90.0, HashMap::new()).await.unwrap();
    
    // Add an alert configuration
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
    
    let high_cpu_alert = fired_alerts.iter().find(|a| a.config.name == "high_cpu").unwrap();
    assert!(high_cpu_alert.is_firing);
    assert_eq!(high_cpu_alert.fire_count, 1);
}

#[tokio::test]
async fn test_health_check_manager() {
    let health_manager = HealthCheckManager::new();
    
    // Add health checks
    health_manager.add_health_check("database", || {
        Ok((HealthStatus::Healthy, "Database connection OK".to_string()))
    }).await.unwrap();
    
    health_manager.add_health_check("network", || {
        Ok((HealthStatus::Degraded, "Network latency high".to_string()))
    }).await.unwrap();
    
    health_manager.add_health_check("storage", || {
        Ok((HealthStatus::Unhealthy, "Storage full".to_string()))
    }).await.unwrap();
    
    // Run health checks
    let health_checks = health_manager.run_health_checks().await.unwrap();
    assert_eq!(health_checks.len(), 3);
    
    // Check individual health statuses
    let database_check = health_checks.iter().find(|h| h.name == "database").unwrap();
    assert_eq!(database_check.status, HealthStatus::Healthy);
    
    let network_check = health_checks.iter().find(|h| h.name == "network").unwrap();
    assert_eq!(network_check.status, HealthStatus::Degraded);
    
    let storage_check = health_checks.iter().find(|h| h.name == "storage").unwrap();
    assert_eq!(storage_check.status, HealthStatus::Unhealthy);
    
    // Check overall health status
    let overall_health = health_manager.get_overall_health().await.unwrap();
    assert_eq!(overall_health, HealthStatus::Unhealthy);
}

#[tokio::test]
async fn test_production_monitor() {
    let monitor = ProductionMonitor::new();
    
    // Add a monitoring event
    let event = MonitoringEvent {
        id: "test_event_1".to_string(),
        event_type: "test".to_string(),
        severity: AlertSeverity::Info,
        message: "Test event for monitoring".to_string(),
        timestamp: std::time::SystemTime::now(),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("source".to_string(), "test".to_string());
            metadata.insert("version".to_string(), "1.0.0".to_string());
            metadata
        },
    };
    
    monitor.add_event(event).await.unwrap();
    
    // Get dashboard data
    let dashboard_data = monitor.get_dashboard_data().await.unwrap();
    
    // Verify dashboard data structure
    assert_eq!(dashboard_data.recent_events.len(), 1);
    assert_eq!(dashboard_data.recent_events[0].id, "test_event_1");
    assert_eq!(dashboard_data.recent_events[0].event_type, "test");
    assert_eq!(dashboard_data.recent_events[0].severity, AlertSeverity::Info);
    
    // Verify system metrics
    assert!(dashboard_data.system_metrics.cpu_usage >= 0.0);
    assert!(dashboard_data.system_metrics.memory_usage >= 0.0);
    assert!(dashboard_data.system_metrics.disk_usage >= 0.0);
    
    // Verify blockchain metrics
    assert!(dashboard_data.blockchain_metrics.block_height >= 0);
    assert!(dashboard_data.blockchain_metrics.transaction_count >= 0);
    assert!(dashboard_data.blockchain_metrics.peer_count >= 0);
}

#[tokio::test]
async fn test_deployment_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let data_dir = temp_dir.path().join("data");
    
    // Create and configure deployment manager
    let mut manager = DeploymentManager::new(config_path.clone());
    manager.update_config(|config| {
        config.environment = DeploymentEnvironment::Testnet;
        config.database.data_dir = data_dir.clone();
        config.network.listen_address = "0.0.0.0:30303".to_string();
        config.monitoring.metrics_enabled = true;
        config.monitoring.metrics_port = 9090;
    });
    
    // Initialize environment
    manager.initialize_environment().await.unwrap();
    
    // Save configuration
    manager.save_config().await.unwrap();
    
    // Validate configuration
    assert!(manager.validate_config().is_ok());
    
    // Get deployment status
    let status = manager.get_deployment_status().await.unwrap();
    assert!(status.is_initialized);
    assert!(status.config_valid);
    
    // Generate deployment scripts
    let scripts = DeploymentUtils::generate_deployment_scripts(manager.config()).unwrap();
    assert!(!scripts.is_empty());
    
    // Test monitoring integration
    let monitor = ProductionMonitor::new();
    let dashboard_data = monitor.get_dashboard_data().await.unwrap();
    assert!(!dashboard_data.system_metrics.uptime.is_zero());
}

#[tokio::test]
async fn test_environment_specific_configs() {
    // Test development environment
    let dev_config = DeploymentManager::default_config();
    assert_eq!(dev_config.environment, DeploymentEnvironment::Development);
    assert!(!dev_config.security.tls_enabled);
    assert!(!dev_config.monitoring.alerting_enabled);
    
    // Test testnet environment
    let testnet_config = DeploymentManager::testnet_config();
    assert_eq!(testnet_config.environment, DeploymentEnvironment::Testnet);
    assert!(!testnet_config.network.bootstrap_nodes.is_empty());
    assert_eq!(testnet_config.consensus.block_time, Duration::from_secs(6));
    
    // Test mainnet environment
    let mainnet_config = DeploymentManager::mainnet_config();
    assert_eq!(mainnet_config.environment, DeploymentEnvironment::Mainnet);
    assert!(mainnet_config.security.tls_enabled);
    assert!(mainnet_config.monitoring.alerting_enabled);
    assert!(mainnet_config.database.backup_enabled);
}

#[tokio::test]
async fn test_feature_flags() {
    let config = DeploymentManager::default_config();
    
    // All features should be enabled by default
    assert!(config.features.zkp_enabled);
    assert!(config.features.state_channels_enabled);
    assert!(config.features.sharding_enabled);
    assert!(config.features.cross_chain_enabled);
    assert!(config.features.ai_integration_enabled);
    assert!(config.features.mobile_support_enabled);
    assert!(config.features.performance_optimization_enabled);
    assert!(config.features.security_enhancements_enabled);
    assert!(config.features.developer_tools_enabled);
}

#[tokio::test]
async fn test_consensus_config() {
    let config = DeploymentManager::default_config();
    
    assert_eq!(config.consensus.algorithm, "dpos");
    assert_eq!(config.consensus.block_time, Duration::from_secs(12));
    assert!(config.consensus.difficulty_adjustment);
    assert_eq!(config.consensus.validator_count, 21);
    assert_eq!(config.consensus.stake_requirement, 1000000);
    assert!(config.consensus.slashing_enabled);
}

#[tokio::test]
async fn test_network_config() {
    let config = NetworkConfig::default();
    
    assert_eq!(config.listen_address, "0.0.0.0:30303");
    assert_eq!(config.max_peers, 50);
    assert_eq!(config.connection_timeout, Duration::from_secs(30));
    assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
    assert!(config.discovery_enabled);
}

#[tokio::test]
async fn test_database_config() {
    let config = DatabaseConfig::default();
    
    assert_eq!(config.data_dir, PathBuf::from("./data"));
    assert_eq!(config.max_db_size, 100 * 1024 * 1024 * 1024); // 100GB
    assert_eq!(config.cache_size, 1024 * 1024 * 1024); // 1GB
    assert!(config.compression_enabled);
    assert!(config.backup_enabled);
    assert_eq!(config.backup_interval, Duration::from_secs(3600)); // 1 hour
}

#[tokio::test]
async fn test_security_config() {
    let config = SecurityConfig::default();
    
    assert!(!config.tls_enabled);
    assert!(config.rate_limiting_enabled);
    assert_eq!(config.max_requests_per_minute, 1000);
    assert!(!config.firewall_enabled);
    assert!(config.allowed_ips.is_empty());
    assert!(config.blocked_ips.is_empty());
}

#[tokio::test]
async fn test_monitoring_config() {
    let config = MonitoringConfig::default();
    
    assert!(config.metrics_enabled);
    assert_eq!(config.metrics_port, 9090);
    assert!(config.health_check_enabled);
    assert_eq!(config.health_check_port, 8080);
    assert_eq!(config.logging_level, "info");
    assert!(!config.alerting_enabled);
}
