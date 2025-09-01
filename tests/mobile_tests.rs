use gillean::{Blockchain, Transaction, BlockchainError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Mobile Support Types
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    IOs,
    Android,
    Flutter,
    ReactNative,
    Xamarin,
}

#[derive(Debug, Clone)]
pub struct MobileDevice {
    pub device_id: String,
    pub platform: Platform,
    pub os_version: String,
    pub app_version: String,
    pub screen_resolution: (u32, u32),
    pub network_type: NetworkType,
    pub battery_level: f64,
    pub is_online: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkType {
    WiFi,
    Cellular4G,
    Cellular5G,
    Cellular3G,
    Offline,
}

#[derive(Debug, Clone)]
pub struct MobileWallet {
    pub wallet_id: String,
    pub device_id: String,
    pub encrypted_private_key: String,
    pub public_address: String,
    pub balance: f64,
    pub transaction_history: Vec<Transaction>,
    pub security_level: SecurityLevel,
    pub biometric_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    Basic,
    Enhanced,
    Maximum,
}

#[derive(Debug, Clone)]
pub struct MobileTransaction {
    pub transaction_id: String,
    pub device_id: String,
    pub amount: f64,
    pub recipient: String,
    pub gas_price: f64,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub network_fee: f64,
    pub confirmation_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct PushNotification {
    pub notification_id: String,
    pub device_id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: u64,
    pub is_read: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    TransactionReceived,
    TransactionConfirmed,
    SecurityAlert,
    PriceAlert,
    NetworkUpdate,
}

#[derive(Debug, Clone)]
pub struct MobileAppConfig {
    pub app_id: String,
    pub api_endpoint: String,
    pub websocket_url: String,
    pub push_notification_key: String,
    pub analytics_enabled: bool,
    pub crash_reporting_enabled: bool,
    pub debug_mode: bool,
}

// Mobile Support Manager
pub struct MobileManager {
    pub devices: Arc<Mutex<HashMap<String, MobileDevice>>>,
    pub wallets: Arc<Mutex<HashMap<String, MobileWallet>>>,
    pub transactions: Arc<Mutex<HashMap<String, MobileTransaction>>>,
    pub notifications: Arc<Mutex<HashMap<String, PushNotification>>>,
    pub app_config: MobileAppConfig,
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl MobileManager {
    pub fn new(blockchain: Blockchain) -> Self {
        let app_config = MobileAppConfig {
            app_id: "com.gillean.mobile".to_string(),
            api_endpoint: "https://api.gillean.com".to_string(),
            websocket_url: "wss://ws.gillean.com".to_string(),
            push_notification_key: "mobile_push_key_123".to_string(),
            analytics_enabled: true,
            crash_reporting_enabled: true,
            debug_mode: false,
        };

        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            wallets: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            notifications: Arc::new(Mutex::new(HashMap::new())),
            app_config,
            blockchain: Arc::new(Mutex::new(blockchain)),
        }
    }

    pub fn register_device(&self, device: MobileDevice) -> Result<(), BlockchainError> {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.device_id.clone(), device);
        Ok(())
    }

    pub fn create_wallet(&self, device_id: &str, security_level: SecurityLevel) -> Result<MobileWallet, BlockchainError> {
        let wallet_id = Uuid::new_v4().to_string();
        let public_address = format!("0x{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let wallet = MobileWallet {
            wallet_id: wallet_id.clone(),
            device_id: device_id.to_string(),
            encrypted_private_key: format!("encrypted_key_{}", wallet_id),
            public_address,
            balance: 0.0,
            transaction_history: Vec::new(),
            security_level: security_level.clone(),
            biometric_enabled: security_level == SecurityLevel::Maximum,
        };

        let mut wallets = self.wallets.lock().unwrap();
        wallets.insert(wallet_id.clone(), wallet.clone());
        Ok(wallet)
    }

    pub fn send_transaction(&self, wallet_id: &str, recipient: &str, amount: f64) -> Result<MobileTransaction, BlockchainError> {
        let mut wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get_mut(wallet_id)
            .ok_or(BlockchainError::InvalidInput("Wallet not found".to_string()))?;

        if amount > wallet.balance {
            return Err(BlockchainError::InvalidInput("Insufficient balance".to_string()));
        }

        let transaction_id = Uuid::new_v4().to_string();
        let gas_price = 20.0; // Standard gas price
        let network_fee = gas_price * 21000.0; // Standard gas limit

        let mobile_tx = MobileTransaction {
            transaction_id: transaction_id.clone(),
            device_id: wallet.device_id.clone(),
            amount,
            recipient: recipient.to_string(),
            gas_price,
            status: TransactionStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            network_fee,
            confirmation_count: 0,
        };

        // Create blockchain transaction
        let blockchain_tx = Transaction::new_transfer(
            wallet.public_address.clone(),
            recipient.to_string(),
            amount,
            Some(format!("Mobile transaction {}", transaction_id)),
        )?;

        // Add to blockchain
        let mut blockchain = self.blockchain.lock().unwrap();
        blockchain.add_transaction(
            blockchain_tx.sender.clone(),
            blockchain_tx.receiver.clone(),
            blockchain_tx.amount,
            blockchain_tx.message.clone(),
        )?;

        // Update wallet
        wallet.balance -= amount + network_fee;
        wallet.transaction_history.push(blockchain_tx);

        // Store mobile transaction
        let mut transactions = self.transactions.lock().unwrap();
        transactions.insert(transaction_id.clone(), mobile_tx.clone());

        // Send notification
        self.send_notification(
            &wallet.device_id,
            "Transaction Sent",
            &format!("Sent {} GIL to {}", amount, recipient),
            NotificationType::TransactionConfirmed,
        )?;

        Ok(mobile_tx)
    }

    pub fn receive_transaction(&self, wallet_id: &str, amount: f64, sender: &str) -> Result<(), BlockchainError> {
        let mut wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get_mut(wallet_id)
            .ok_or(BlockchainError::InvalidInput("Wallet not found".to_string()))?;

        wallet.balance += amount;

        // Send notification
        self.send_notification(
            &wallet.device_id,
            "Transaction Received",
            &format!("Received {} GIL from {}", amount, sender),
            NotificationType::TransactionReceived,
        )?;

        Ok(())
    }

    pub fn send_notification(&self, device_id: &str, title: &str, message: &str, notification_type: NotificationType) -> Result<(), BlockchainError> {
        let notification_id = Uuid::new_v4().to_string();
        
        let notification = PushNotification {
            notification_id: notification_id.clone(),
            device_id: device_id.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            notification_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_read: false,
        };

        let mut notifications = self.notifications.lock().unwrap();
        notifications.insert(notification_id, notification);
        Ok(())
    }

    pub fn get_device_notifications(&self, device_id: &str) -> Result<Vec<PushNotification>, BlockchainError> {
        let notifications = self.notifications.lock().unwrap();
        let device_notifications: Vec<PushNotification> = notifications.values()
            .filter(|n| n.device_id == device_id)
            .cloned()
            .collect();
        Ok(device_notifications)
    }

    pub fn mark_notification_read(&self, notification_id: &str) -> Result<(), BlockchainError> {
        let mut notifications = self.notifications.lock().unwrap();
        if let Some(notification) = notifications.get_mut(notification_id) {
            notification.is_read = true;
        }
        Ok(())
    }

    pub fn sync_wallet(&self, wallet_id: &str) -> Result<(), BlockchainError> {
        let mut wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get_mut(wallet_id)
            .ok_or(BlockchainError::InvalidInput("Wallet not found".to_string()))?;

        // Simulate blockchain sync
        let _blockchain = self.blockchain.lock().unwrap();
        // In a real implementation, this would fetch the latest balance from the blockchain
        wallet.balance = 1000.0; // Simulated balance

        Ok(())
    }

    pub fn get_wallet_balance(&self, wallet_id: &str) -> Result<f64, BlockchainError> {
        let wallets = self.wallets.lock().unwrap();
        let wallet = wallets.get(wallet_id)
            .ok_or(BlockchainError::InvalidInput("Wallet not found".to_string()))?;
        Ok(wallet.balance)
    }

    pub fn update_device_status(&self, device_id: &str, is_online: bool, battery_level: f64) -> Result<(), BlockchainError> {
        let mut devices = self.devices.lock().unwrap();
        if let Some(device) = devices.get_mut(device_id) {
            device.is_online = is_online;
            device.battery_level = battery_level;
        }
        Ok(())
    }
}

// Mobile Support Test Suite
pub struct MobileSupportSuite {
    _manager: MobileManager,
}

impl MobileSupportSuite {
    pub fn new() -> Result<Self, BlockchainError> {
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let manager = MobileManager::new(blockchain);
        
        Ok(Self {
            _manager: manager,
        })
    }

    pub fn run_all_tests(&self) -> Result<(), BlockchainError> {
        println!("Running Mobile Support Tests...");
        
        self.test_device_registration()?;
        self.test_wallet_creation()?;
        self.test_transaction_sending()?;
        self.test_transaction_receiving()?;
        self.test_notifications()?;
        self.test_wallet_sync()?;
        self.test_multi_platform_support()?;
        self.test_offline_capabilities()?;
        self.test_security_features()?;
        self.test_invalid_operations()?;
        
        println!("✅ All Mobile Support tests passed!");
        Ok(())
    }

    fn test_device_registration(&self) -> Result<(), BlockchainError> {
        println!("  Testing device registration...");
        
        let device = MobileDevice {
            device_id: "device_123".to_string(),
            platform: Platform::IOs,
            os_version: "15.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (375, 812),
            network_type: NetworkType::WiFi,
            battery_level: 0.85,
            is_online: true,
        };

        self._manager.register_device(device.clone())?;
        
        let devices = self._manager.devices.lock().unwrap();
        assert!(devices.contains_key("device_123"));
        assert_eq!(devices.get("device_123").unwrap().platform, Platform::IOs);
        
        Ok(())
    }

    fn test_wallet_creation(&self) -> Result<(), BlockchainError> {
        println!("  Testing wallet creation...");
        
        // Register device first
        let device = MobileDevice {
            device_id: "device_456".to_string(),
            platform: Platform::Android,
            os_version: "12.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (360, 640),
            network_type: NetworkType::Cellular5G,
            battery_level: 0.75,
            is_online: true,
        };
        self._manager.register_device(device)?;

        // Create wallet
        let wallet = self._manager.create_wallet("device_456", SecurityLevel::Maximum)?;
        
        assert!(!wallet.wallet_id.is_empty());
        assert_eq!(wallet.device_id, "device_456");
        assert_eq!(wallet.security_level, SecurityLevel::Maximum);
        assert!(wallet.biometric_enabled);
        assert!(wallet.public_address.starts_with("0x"));
        
        Ok(())
    }

    fn test_transaction_sending(&self) -> Result<(), BlockchainError> {
        println!("  Testing transaction sending...");
        
        // Setup device and wallet
        let device = MobileDevice {
            device_id: "device_789".to_string(),
            platform: Platform::Flutter,
            os_version: "11.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (414, 896),
            network_type: NetworkType::WiFi,
            battery_level: 0.90,
            is_online: true,
        };
        self._manager.register_device(device)?;
        
        let wallet = self._manager.create_wallet("device_789", SecurityLevel::Enhanced)?;
        
        // Add balance to wallet
        let mut wallets = self._manager.wallets.lock().unwrap();
        if let Some(w) = wallets.get_mut(&wallet.wallet_id) {
            w.balance = 1000.0;
        }
        drop(wallets);

        // Send transaction
        let mobile_tx = self._manager.send_transaction(&wallet.wallet_id, "0xrecipient123", 100.0)?;
        
        assert_eq!(mobile_tx.amount, 100.0);
        assert_eq!(mobile_tx.recipient, "0xrecipient123");
        assert_eq!(mobile_tx.status, TransactionStatus::Pending);
        assert!(mobile_tx.timestamp > 0);
        
        // Check wallet balance was reduced
        let balance = self._manager.get_wallet_balance(&wallet.wallet_id)?;
        assert!(balance < 1000.0, "Balance should be reduced by transaction amount and fees");
        
        Ok(())
    }

    fn test_transaction_receiving(&self) -> Result<(), BlockchainError> {
        println!("  Testing transaction receiving...");
        
        // Setup device and wallet
        let device = MobileDevice {
            device_id: "device_receiver".to_string(),
            platform: Platform::ReactNative,
            os_version: "14.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (390, 844),
            network_type: NetworkType::Cellular4G,
            battery_level: 0.60,
            is_online: true,
        };
        self._manager.register_device(device)?;
        
        let wallet = self._manager.create_wallet("device_receiver", SecurityLevel::Basic)?;
        let initial_balance = self._manager.get_wallet_balance(&wallet.wallet_id)?;
        
        // Receive transaction
        self._manager.receive_transaction(&wallet.wallet_id, 250.0, "0xsender123")?;
        
        let new_balance = self._manager.get_wallet_balance(&wallet.wallet_id)?;
        assert_eq!(new_balance, initial_balance + 250.0);
        
        Ok(())
    }

    fn test_notifications(&self) -> Result<(), BlockchainError> {
        println!("  Testing notifications...");
        
        let device_id = "device_notifications".to_string();
        
        // Send test notification
        self._manager.send_notification(
            &device_id,
            "Test Title",
            "Test Message",
            NotificationType::TransactionReceived,
        )?;
        
        // Get notifications for device
        let notifications = self._manager.get_device_notifications(&device_id)?;
        assert!(!notifications.is_empty());
        
        let notification = &notifications[0];
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.message, "Test Message");
        assert_eq!(notification.notification_type, NotificationType::TransactionReceived);
        assert!(!notification.is_read);
        
        // Mark as read
        self._manager.mark_notification_read(&notification.notification_id)?;
        
        let updated_notifications = self._manager.get_device_notifications(&device_id)?;
        assert!(updated_notifications[0].is_read);
        
        Ok(())
    }

    fn test_wallet_sync(&self) -> Result<(), BlockchainError> {
        println!("  Testing wallet sync...");
        
        // Setup device and wallet
        let device = MobileDevice {
            device_id: "device_sync".to_string(),
            platform: Platform::Xamarin,
            os_version: "13.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (375, 667),
            network_type: NetworkType::WiFi,
            battery_level: 0.80,
            is_online: true,
        };
        self._manager.register_device(device)?;
        
        let wallet = self._manager.create_wallet("device_sync", SecurityLevel::Enhanced)?;
        
        // Sync wallet
        self._manager.sync_wallet(&wallet.wallet_id)?;
        
        let balance = self._manager.get_wallet_balance(&wallet.wallet_id)?;
        assert_eq!(balance, 1000.0); // Simulated balance from sync
        
        Ok(())
    }

    fn test_multi_platform_support(&self) -> Result<(), BlockchainError> {
        println!("  Testing multi-platform support...");
        
        let platforms = vec![
            (Platform::IOs, "device_ios"),
            (Platform::Android, "device_android"),
            (Platform::Flutter, "device_flutter"),
            (Platform::ReactNative, "device_rn"),
            (Platform::Xamarin, "device_xamarin"),
        ];
        
        for (platform, device_id) in platforms {
            let device = MobileDevice {
                device_id: device_id.to_string(),
                platform: platform.clone(),
                os_version: "1.0".to_string(),
                app_version: "1.0.0".to_string(),
                screen_resolution: (375, 812),
                network_type: NetworkType::WiFi,
                battery_level: 0.85,
                is_online: true,
            };
            
            self._manager.register_device(device)?;
            let wallet = self._manager.create_wallet(device_id, SecurityLevel::Basic)?;
            
            assert!(!wallet.wallet_id.is_empty());
            assert_eq!(wallet.device_id, device_id);
        }
        
        Ok(())
    }

    fn test_offline_capabilities(&self) -> Result<(), BlockchainError> {
        println!("  Testing offline capabilities...");
        
        let device_id = "device_offline".to_string();
        
        // Update device to offline
        self._manager.update_device_status(&device_id, false, 0.30)?;
        
        let devices = self._manager.devices.lock().unwrap();
        if let Some(device) = devices.get(&device_id) {
            assert!(!device.is_online);
            assert_eq!(device.battery_level, 0.30);
        }
        
        Ok(())
    }

    fn test_security_features(&self) -> Result<(), BlockchainError> {
        println!("  Testing security features...");
        
        // Test different security levels
        let security_levels = vec![
            SecurityLevel::Basic,
            SecurityLevel::Enhanced,
            SecurityLevel::Maximum,
        ];
        
        for (i, security_level) in security_levels.iter().enumerate() {
            let device_id = format!("device_security_{}", i);
            
            let device = MobileDevice {
                device_id: device_id.clone(),
                platform: Platform::IOs,
                os_version: "15.0".to_string(),
                app_version: "1.0.0".to_string(),
                screen_resolution: (375, 812),
                network_type: NetworkType::WiFi,
                battery_level: 0.85,
                is_online: true,
            };
            self._manager.register_device(device)?;
            
            let wallet = self._manager.create_wallet(&device_id, security_level.clone())?;
            
            assert_eq!(wallet.security_level, *security_level);
            
            // Check biometric setting
            match security_level {
                SecurityLevel::Maximum => assert!(wallet.biometric_enabled),
                _ => assert!(!wallet.biometric_enabled),
            }
        }
        
        Ok(())
    }

    fn test_invalid_operations(&self) -> Result<(), BlockchainError> {
        println!("  Testing invalid operations...");
        
        // Test sending transaction with insufficient balance
        let device = MobileDevice {
            device_id: "device_invalid".to_string(),
            platform: Platform::Android,
            os_version: "12.0".to_string(),
            app_version: "1.0.0".to_string(),
            screen_resolution: (360, 640),
            network_type: NetworkType::WiFi,
            battery_level: 0.85,
            is_online: true,
        };
        self._manager.register_device(device)?;
        
        let wallet = self._manager.create_wallet("device_invalid", SecurityLevel::Basic)?;
        
        // Try to send more than balance
        let result = self._manager.send_transaction(&wallet.wallet_id, "0xrecipient", 1000.0);
        assert!(result.is_err());
        
        // Test getting balance of non-existent wallet
        let result = self._manager.get_wallet_balance("non_existent_wallet");
        assert!(result.is_err());
        
        Ok(())
    }
}
