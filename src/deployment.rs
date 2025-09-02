//! Production deployment and configuration management
//! 
//! This module provides infrastructure for deploying Gillean blockchain
//! to testnet and mainnet environments with proper configuration management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use crate::error::{BlockchainError, Result};

/// Deployment environment configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentEnvironment {
    Development,
    Testnet,
    Mainnet,
    Staging,
}

impl DeploymentEnvironment {
    pub fn is_production(&self) -> bool {
        matches!(self, DeploymentEnvironment::Mainnet)
    }
    
    pub fn is_testing(&self) -> bool {
        matches!(self, DeploymentEnvironment::Development | DeploymentEnvironment::Testnet | DeploymentEnvironment::Staging)
    }
}

/// Network configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bootstrap_nodes: Vec<String>,
    pub listen_address: String,
    pub external_address: Option<String>,
    pub max_peers: u32,
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub discovery_enabled: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![],
            listen_address: "0.0.0.0:30303".to_string(),
            external_address: None,
            max_peers: 50,
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            discovery_enabled: true,
        }
    }
}

/// Database configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub data_dir: PathBuf,
    pub max_db_size: u64,
    pub cache_size: usize,
    pub compression_enabled: bool,
    pub backup_enabled: bool,
    pub backup_interval: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            max_db_size: 100 * 1024 * 1024 * 1024, // 100GB
            cache_size: 1024 * 1024 * 1024, // 1GB
            compression_enabled: true,
            backup_enabled: true,
            backup_interval: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Security configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
    pub rate_limiting_enabled: bool,
    pub max_requests_per_minute: u32,
    pub firewall_enabled: bool,
    pub allowed_ips: Vec<String>,
    pub blocked_ips: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            rate_limiting_enabled: true,
            max_requests_per_minute: 1000,
            firewall_enabled: false,
            allowed_ips: vec![],
            blocked_ips: vec![],
        }
    }
}

/// Monitoring configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_enabled: bool,
    pub health_check_port: u16,
    pub logging_level: String,
    pub log_file_path: Option<PathBuf>,
    pub alerting_enabled: bool,
    pub alert_webhook_url: Option<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_port: 9090,
            health_check_enabled: true,
            health_check_port: 8080,
            logging_level: "info".to_string(),
            log_file_path: None,
            alerting_enabled: false,
            alert_webhook_url: None,
        }
    }
}

/// Complete deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub environment: DeploymentEnvironment,
    pub network: NetworkConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub consensus: ConsensusConfig,
    pub features: FeatureFlags,
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Consensus configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: String,
    pub block_time: Duration,
    pub difficulty_adjustment: bool,
    pub validator_count: u32,
    pub stake_requirement: u64,
    pub slashing_enabled: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            algorithm: "dpos".to_string(),
            block_time: Duration::from_secs(12),
            difficulty_adjustment: true,
            validator_count: 21,
            stake_requirement: 1000000, // 1M tokens
            slashing_enabled: true,
        }
    }
}

/// Feature flags for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub zkp_enabled: bool,
    pub state_channels_enabled: bool,
    pub sharding_enabled: bool,
    pub cross_chain_enabled: bool,
    pub ai_integration_enabled: bool,
    pub mobile_support_enabled: bool,
    pub performance_optimization_enabled: bool,
    pub security_enhancements_enabled: bool,
    pub developer_tools_enabled: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            zkp_enabled: true,
            state_channels_enabled: true,
            sharding_enabled: true,
            cross_chain_enabled: true,
            ai_integration_enabled: true,
            mobile_support_enabled: true,
            performance_optimization_enabled: true,
            security_enhancements_enabled: true,
            developer_tools_enabled: true,
        }
    }
}

/// Deployment manager for handling production deployments
pub struct DeploymentManager {
    config: DeploymentConfig,
    config_path: PathBuf,
}

impl DeploymentManager {
    /// Create a new deployment manager
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config: Self::default_config(),
            config_path,
        }
    }
    
    /// Load configuration from file
    pub async fn load_config(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).await
                .map_err(|e| BlockchainError::InvalidInput(format!("Failed to read config: {}", e)))?;
            
            self.config = toml::from_str(&content)
                .map_err(|e| BlockchainError::InvalidInput(format!("Invalid config format: {}", e)))?;
        }
        
        Ok(())
    }
    
    /// Save configuration to file
    pub async fn save_config(&self) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| BlockchainError::InvalidInput(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&self.config_path, content).await
            .map_err(|e| BlockchainError::InvalidInput(format!("Failed to write config: {}", e)))?;
        
        Ok(())
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DeploymentConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn update_config<F>(&mut self, updater: F) 
    where 
        F: FnOnce(&mut DeploymentConfig),
    {
        updater(&mut self.config);
    }
    
    /// Validate configuration for deployment
    pub fn validate_config(&self) -> Result<()> {
        // Validate network configuration
        if self.config.network.bootstrap_nodes.is_empty() && self.config.environment.is_production() {
            return Err(BlockchainError::InvalidInput(
                "Bootstrap nodes required for production deployment".to_string()
            ));
        }
        
        // Validate security configuration
        if self.config.security.tls_enabled
            && (self.config.security.tls_cert_path.is_none() || self.config.security.tls_key_path.is_none()) {
            return Err(BlockchainError::InvalidInput(
                "TLS certificate and key paths required when TLS is enabled".to_string()
            ));
        }
        
        // Validate database configuration
        if self.config.database.data_dir.exists() && !self.config.database.data_dir.is_dir() {
            return Err(BlockchainError::InvalidInput(
                "Database data directory must be a directory".to_string()
            ));
        }
        
        // Validate monitoring configuration
        if self.config.monitoring.metrics_enabled && self.config.monitoring.metrics_port == 0 {
            return Err(BlockchainError::InvalidInput(
                "Valid metrics port required when metrics are enabled".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Generate default configuration based on environment
    pub fn default_config() -> DeploymentConfig {
        DeploymentConfig {
            environment: DeploymentEnvironment::Development,
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
            consensus: ConsensusConfig::default(),
            features: FeatureFlags::default(),
            custom_settings: HashMap::new(),
        }
    }
    
    /// Generate testnet configuration
    pub fn testnet_config() -> DeploymentConfig {
        let mut config = Self::default_config();
        config.environment = DeploymentEnvironment::Testnet;
        config.network.bootstrap_nodes = vec![
            "testnet-node-1.gillean.org:30303".to_string(),
            "testnet-node-2.gillean.org:30303".to_string(),
            "testnet-node-3.gillean.org:30303".to_string(),
        ];
        config.consensus.block_time = Duration::from_secs(6); // Faster blocks for testing
        config.consensus.validator_count = 7; // Fewer validators for testnet
        config.consensus.stake_requirement = 100000; // Lower stake requirement
        config.monitoring.logging_level = "debug".to_string();
        config
    }
    
    /// Generate mainnet configuration
    pub fn mainnet_config() -> DeploymentConfig {
        let mut config = Self::default_config();
        config.environment = DeploymentEnvironment::Mainnet;
        config.network.bootstrap_nodes = vec![
            "mainnet-node-1.gillean.org:30303".to_string(),
            "mainnet-node-2.gillean.org:30303".to_string(),
            "mainnet-node-3.gillean.org:30303".to_string(),
            "mainnet-node-4.gillean.org:30303".to_string(),
            "mainnet-node-5.gillean.org:30303".to_string(),
        ];
        config.security.tls_enabled = true;
        config.security.rate_limiting_enabled = true;
        config.security.max_requests_per_minute = 5000;
        config.monitoring.alerting_enabled = true;
        config.monitoring.logging_level = "warn".to_string();
        config.database.backup_enabled = true;
        config.database.backup_interval = Duration::from_secs(1800); // 30 minutes
        config
    }
    
    /// Initialize deployment environment
    pub async fn initialize_environment(&self) -> Result<()> {
        // Create data directory
        if !self.config.database.data_dir.exists() {
            fs::create_dir_all(&self.config.database.data_dir).await
                .map_err(|e| BlockchainError::InvalidInput(format!("Failed to create data directory: {}", e)))?;
        }
        
        // Create log directory if specified
        if let Some(log_path) = &self.config.monitoring.log_file_path {
            if let Some(log_dir) = log_path.parent() {
                if !log_dir.exists() {
                    fs::create_dir_all(log_dir).await
                        .map_err(|e| BlockchainError::InvalidInput(format!("Failed to create log directory: {}", e)))?;
                }
            }
        }
        
        // Validate TLS certificates if enabled
        if self.config.security.tls_enabled {
            if let Some(cert_path) = &self.config.security.tls_cert_path {
                if !cert_path.exists() {
                    return Err(BlockchainError::InvalidInput(
                        format!("TLS certificate file not found: {:?}", cert_path)
                    ));
                }
            }
            
            if let Some(key_path) = &self.config.security.tls_key_path {
                if !key_path.exists() {
                    return Err(BlockchainError::InvalidInput(
                        format!("TLS key file not found: {:?}", key_path)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Get deployment status
    pub async fn get_deployment_status(&self) -> Result<DeploymentStatus> {
        let mut status = DeploymentStatus {
            environment: self.config.environment.clone(),
            is_initialized: false,
            data_directory_exists: false,
            config_valid: false,
            last_backup: None,
            uptime: Duration::from_secs(0),
            node_count: 0,
            block_height: 0,
            sync_status: SyncStatus::Unknown,
        };
        
        // Check if data directory exists
        status.data_directory_exists = self.config.database.data_dir.exists();
        
        // Validate configuration
        status.config_valid = self.validate_config().is_ok();
        
        // Check if environment is initialized
        status.is_initialized = status.data_directory_exists && status.config_valid;
        
        Ok(status)
    }
}

/// Deployment status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub environment: DeploymentEnvironment,
    pub is_initialized: bool,
    pub data_directory_exists: bool,
    pub config_valid: bool,
    pub last_backup: Option<std::time::SystemTime>,
    pub uptime: Duration,
    pub node_count: u32,
    pub block_height: u64,
    pub sync_status: SyncStatus,
}

/// Network synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Unknown,
    Syncing,
    Synced,
    Behind,
    Error(String),
}

/// Deployment utilities
pub struct DeploymentUtils;

impl DeploymentUtils {
    /// Generate deployment scripts
    pub fn generate_deployment_scripts(config: &DeploymentConfig) -> Result<Vec<DeploymentScript>> {
        let mut scripts = Vec::new();
        
        // Systemd service script
        scripts.push(DeploymentScript {
            name: "gillean.service".to_string(),
            content: Self::generate_systemd_service(config),
            target_path: "/etc/systemd/system/gillean.service".to_string(),
        });
        
        // Docker compose file
        scripts.push(DeploymentScript {
            name: "docker-compose.yml".to_string(),
            content: Self::generate_docker_compose(config),
            target_path: "docker-compose.yml".to_string(),
        });
        
        // Nginx configuration
        if config.security.tls_enabled {
            scripts.push(DeploymentScript {
                name: "nginx.conf".to_string(),
                content: Self::generate_nginx_config(config),
                target_path: "/etc/nginx/sites-available/gillean".to_string(),
            });
        }
        
        // Backup script
        if config.database.backup_enabled {
            scripts.push(DeploymentScript {
                name: "backup.sh".to_string(),
                content: Self::generate_backup_script(config),
                target_path: "/usr/local/bin/gillean-backup.sh".to_string(),
            });
        }
        
        Ok(scripts)
    }
    
    fn generate_systemd_service(_config: &DeploymentConfig) -> String {
        r#"[Unit]
Description=Gillean Blockchain Node
After=network.target

[Service]
Type=simple
User=gillean
Group=gillean
WorkingDirectory=/opt/gillean
ExecStart=/opt/gillean/gillean --config /etc/gillean/config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=gillean

[Install]
WantedBy=multi-user.target"#.to_string()
    }
    
    fn generate_docker_compose(config: &DeploymentConfig) -> String {
        format!(
            r#"version: '3.8'

services:
  gillean:
    image: gillean/blockchain:latest
    container_name: gillean-node
    restart: unless-stopped
    ports:
      - "{}:30303"
      - "{}:9090"
      - "{}:8080"
    volumes:
      - ./data:/opt/gillean/data
      - ./config.toml:/etc/gillean/config.toml
    environment:
      - RUST_LOG={}
    networks:
      - gillean-network

networks:
  gillean-network:
    driver: bridge"#,
            config.network.listen_address.split(':').nth(1).unwrap_or("30303"),
            config.monitoring.metrics_port,
            config.monitoring.health_check_port,
            config.monitoring.logging_level
        )
    }
    
    fn generate_nginx_config(config: &DeploymentConfig) -> String {
        format!(
            r#"server {{
    listen 80;
    listen 443 ssl http2;
    server_name gillean-node.example.com;
    
    ssl_certificate /etc/ssl/certs/gillean.crt;
    ssl_certificate_key /etc/ssl/private/gillean.key;
    
    location / {{
        proxy_pass http://127.0.0.1:{};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
    
    location /metrics {{
        proxy_pass http://127.0.0.1:{};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }}
}}"#,
            config.monitoring.health_check_port,
            config.monitoring.metrics_port
        )
    }
    
    fn generate_backup_script(config: &DeploymentConfig) -> String {
        format!(
            r#"#!/bin/bash
# Gillean Blockchain Backup Script

BACKUP_DIR="/opt/gillean/backups"
DATA_DIR="{}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="gillean_backup_$TIMESTAMP.tar.gz"

mkdir -p $BACKUP_DIR

# Create backup
tar -czf $BACKUP_DIR/$BACKUP_FILE -C $DATA_DIR .

# Keep only last 7 days of backups
find $BACKUP_DIR -name "gillean_backup_*.tar.gz" -mtime +7 -delete

echo "Backup completed: $BACKUP_FILE"
"#,
            config.database.data_dir.to_string_lossy()
        )
    }
}

/// Deployment script information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentScript {
    pub name: String,
    pub content: String,
    pub target_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_deployment_config_creation() {
        let config = DeploymentManager::default_config();
        assert_eq!(config.environment, DeploymentEnvironment::Development);
        assert!(config.features.zkp_enabled);
        assert!(config.features.state_channels_enabled);
    }
    
    #[tokio::test]
    async fn test_testnet_config() {
        let config = DeploymentManager::testnet_config();
        assert_eq!(config.environment, DeploymentEnvironment::Testnet);
        assert!(!config.network.bootstrap_nodes.is_empty());
        assert_eq!(config.consensus.block_time, Duration::from_secs(6));
    }
    
    #[tokio::test]
    async fn test_mainnet_config() {
        let config = DeploymentManager::mainnet_config();
        assert_eq!(config.environment, DeploymentEnvironment::Mainnet);
        assert!(config.security.tls_enabled);
        assert!(config.monitoring.alerting_enabled);
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let mut manager = DeploymentManager::new(PathBuf::from("test.toml"));
        manager.update_config(|config| {
            config.security.tls_enabled = true;
            // TLS paths are None by default, which should cause validation to fail
        });
        // Missing TLS paths should cause validation to fail
        assert!(manager.validate_config().is_err());
    }
    
    #[tokio::test]
    async fn test_deployment_scripts_generation() {
        let config = DeploymentManager::default_config();
        let scripts = DeploymentUtils::generate_deployment_scripts(&config).unwrap();
        
        assert!(!scripts.is_empty());
        assert!(scripts.iter().any(|s| s.name == "gillean.service"));
        assert!(scripts.iter().any(|s| s.name == "docker-compose.yml"));
    }
    
    #[tokio::test]
    async fn test_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let mut manager = DeploymentManager::new(config_path.clone());
        manager.update_config(|config| {
            config.environment = DeploymentEnvironment::Testnet;
        });
        
        manager.save_config().await.unwrap();
        
        let mut new_manager = DeploymentManager::new(config_path);
        new_manager.load_config().await.unwrap();
        
        assert_eq!(new_manager.config().environment, DeploymentEnvironment::Testnet);
    }
}
