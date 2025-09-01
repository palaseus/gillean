use crate::{KeyPair, DigitalSignature, Result, BlockchainError};
use crate::crypto::create_address;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use serde::{Serialize, Deserialize};
use log::{info, error};
use std::collections::HashMap;
use uuid::Uuid;
// use base64::engine::general_purpose; // Unused import

/// Wallet-related errors
#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    #[error("Invalid wallet data")]
    InvalidWalletData,
    
    #[error("Key generation error: {0}")]
    KeyGeneration(String),
    
    #[error("Signature error: {0}")]
    Signature(String),
}

impl From<WalletError> for BlockchainError {
    fn from(err: WalletError) -> Self {
        BlockchainError::WalletError(err.to_string())
    }
}

/// Encrypted wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedWallet {
    pub id: String,
    pub address: String,
    pub encrypted_data: Vec<u8>,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Wallet information (public data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub address: String,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub balance: f64,
}

/// Wallet manager for handling multiple wallets
pub struct WalletManager {
    wallets: HashMap<String, EncryptedWallet>,
    storage_path: Option<String>,
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletManager {
    /// Create a new wallet manager
    /// 
    /// # Returns
    /// * `WalletManager` - The wallet manager instance
    pub fn new() -> Self {
        WalletManager {
            wallets: HashMap::new(),
            storage_path: None,
        }
    }
    
    /// Create a new wallet manager with storage
    /// 
    /// # Arguments
    /// * `storage` - Blockchain storage instance
    /// 
    /// # Returns
    /// * `WalletManager` - The wallet manager instance
    pub fn with_storage(storage_path: String) -> Self {
        WalletManager {
            wallets: HashMap::new(),
            storage_path: Some(storage_path),
        }
    }
    
    /// Create a new wallet
    /// 
    /// # Arguments
    /// * `password` - Password to encrypt the wallet
    /// * `name` - Optional wallet name
    /// 
    /// # Returns
    /// * `Result<WalletInfo>` - The created wallet info
    pub fn create_wallet(&mut self, password: &str, name: Option<String>) -> Result<WalletInfo> {
        // Generate new key pair
        let keypair = KeyPair::generate()?;
        let public_key = keypair.public_key();
        let address = create_address(&public_key);
        
        // Create wallet ID
        let id = Uuid::new_v4().to_string();
        
        // Serialize wallet data
        let wallet_data = WalletData {
            private_key: keypair.private_key_hex(),
            public_key: keypair.public_key_hex(),
            name: name.unwrap_or_else(|| format!("Wallet-{}", &id[..8])),
        };
        
        let serialized_data = serde_json::to_vec(&wallet_data)
            .map_err(|e| WalletError::Encryption(format!("Serialization failed: {}", e)))?;
        
        // Encrypt wallet data
        let encrypted_wallet = self.encrypt_wallet_data(&serialized_data, password, &id)?;
        
        // Create wallet info
        let wallet_info = WalletInfo {
            id: id.clone(),
            address: address.clone(),
            public_key: keypair.public_key_hex(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            balance: 0.0,
        };
        
        // Store wallet
        self.wallets.insert(address.clone(), encrypted_wallet);
        
        // Save to storage if available
        if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            let encrypted_data = serde_json::to_vec(&self.wallets[&address])?;
            storage.save_wallet(&address, &encrypted_data)?;
        }
        
        info!("Created new wallet: {}", address);
        Ok(wallet_info)
    }
    
    /// Load a wallet from storage
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `password` - Wallet password
    /// 
    /// # Returns
    /// * `Result<WalletInfo>` - The wallet info
    pub fn load_wallet(&mut self, address: &str, password: &str) -> Result<WalletInfo> {
        // Try to get from memory first
        if let Some(encrypted_wallet) = self.wallets.get(address) {
            return self.decrypt_wallet_info(encrypted_wallet, password);
        }
        
        // Try to load from storage
        if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            if let Some(encrypted_data) = storage.load_wallet(address)? {
                let encrypted_wallet: EncryptedWallet = serde_json::from_slice(&encrypted_data)?;
                self.wallets.insert(address.to_string(), encrypted_wallet.clone());
                return self.decrypt_wallet_info(&encrypted_wallet, password);
            }
        }
        
        Err(WalletError::WalletNotFound(address.to_string()).into())
    }
    
    /// List all available wallets
    /// 
    /// # Returns
    /// * `Result<Vec<WalletInfo>>` - List of wallet infos
    pub fn list_wallets(&self) -> Result<Vec<WalletInfo>> {
        let mut wallet_infos = Vec::new();
        
        // Get from memory
        for (address, encrypted_wallet) in &self.wallets {
            let wallet_info = WalletInfo {
                id: encrypted_wallet.id.clone(),
                address: address.clone(),
                public_key: "".to_string(), // We can't decrypt without password
                created_at: encrypted_wallet.created_at,
                last_accessed: encrypted_wallet.last_accessed,
                balance: 0.0, // Would need blockchain to get actual balance
            };
            wallet_infos.push(wallet_info);
        }
        
        // Get from storage
        if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            let addresses = storage.list_wallets()?;
            for address in addresses {
                if !self.wallets.contains_key(&address) {
                    let wallet_info = WalletInfo {
                        id: "".to_string(),
                        address,
                        public_key: "".to_string(),
                        created_at: chrono::Utc::now(),
                        last_accessed: chrono::Utc::now(),
                        balance: 0.0,
                    };
                    wallet_infos.push(wallet_info);
                }
            }
        }
        
        Ok(wallet_infos)
    }
    
    /// Sign a transaction with a wallet
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `password` - Wallet password
    /// * `transaction_data` - Transaction data to sign
    /// 
    /// # Returns
    /// * `Result<DigitalSignature>` - The signature
    pub fn sign_transaction(&mut self, address: &str, password: &str, transaction_data: &[u8]) -> Result<DigitalSignature> {
        let wallet_data = self.get_wallet_data(address, password)?;
        
        // Create keypair from private key
        let private_key_bytes = crate::utils::hex_to_bytes(&wallet_data.private_key)?;
        let keypair = KeyPair::from_keys(private_key_bytes[..32].to_vec(), private_key_bytes)?;
        
        // Sign the transaction
        let signature = keypair.sign(transaction_data)?;
        
        info!("Signed transaction with wallet: {}", address);
        Ok(signature)
    }
    
    /// Get wallet balance from blockchain
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `blockchain` - Blockchain instance
    /// 
    /// # Returns
    /// * `f64` - Wallet balance
    pub fn get_balance(&self, address: &str, blockchain: &crate::Blockchain) -> f64 {
        blockchain.get_balance(address)
    }
    
    /// Update wallet balances from blockchain
    /// 
    /// # Arguments
    /// * `blockchain` - Blockchain instance
    /// 
    /// # Returns
    /// * `Result<Vec<WalletInfo>>` - Updated wallet infos
    pub fn update_balances(&mut self, blockchain: &crate::Blockchain) -> Result<Vec<WalletInfo>> {
        let mut wallet_infos = Vec::new();
        
        for (address, encrypted_wallet) in &self.wallets {
            let balance = blockchain.get_balance(address);
            
            let wallet_info = WalletInfo {
                id: encrypted_wallet.id.clone(),
                address: address.clone(),
                public_key: "".to_string(), // We can't decrypt without password
                created_at: encrypted_wallet.created_at,
                last_accessed: encrypted_wallet.last_accessed,
                balance,
            };
            wallet_infos.push(wallet_info);
        }
        
        Ok(wallet_infos)
    }
    
    /// Get private key bytes for a wallet
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `password` - Wallet password
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - Private key bytes
    pub fn get_private_key_bytes(&mut self, address: &str, password: &str) -> Result<Vec<u8>> {
        let wallet_data = self.get_wallet_data(address, password)?;
        let private_key_bytes = crate::utils::hex_to_bytes(&wallet_data.private_key)?;
        Ok(private_key_bytes)
    }

    /// Get wallet address
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// 
    /// # Returns
    /// * `String` - Wallet address
    pub fn get_address(&self, address: &str) -> String {
        address.to_string()
    }

    /// Export wallet (returns encrypted data)
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - Encrypted wallet data
    pub fn export_wallet(&self, address: &str) -> Result<Vec<u8>> {
        if let Some(encrypted_wallet) = self.wallets.get(address) {
            Ok(serde_json::to_vec(encrypted_wallet)?)
        } else if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            if let Some(data) = storage.load_wallet(address)? {
                Ok(data)
            } else {
                Err(WalletError::WalletNotFound(address.to_string()).into())
            }
        } else {
            Err(WalletError::WalletNotFound(address.to_string()).into())
        }
    }
    
    /// Import wallet from encrypted data
    /// 
    /// # Arguments
    /// * `encrypted_data` - Encrypted wallet data
    /// 
    /// # Returns
    /// * `Result<WalletInfo>` - The imported wallet info
    pub fn import_wallet(&mut self, encrypted_data: &[u8]) -> Result<WalletInfo> {
        let encrypted_wallet: EncryptedWallet = serde_json::from_slice(encrypted_data)?;
        
        // Verify we can decrypt it (test with empty password)
        let _ = self.decrypt_wallet_data(&encrypted_wallet.encrypted_data, &encrypted_wallet.salt, &encrypted_wallet.nonce, "")?;
        
        self.wallets.insert(encrypted_wallet.address.clone(), encrypted_wallet.clone());
        
        // Save to storage if available
        if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            storage.save_wallet(&encrypted_wallet.address, encrypted_data)?;
        }
        
        let wallet_info = WalletInfo {
            id: encrypted_wallet.id,
            address: encrypted_wallet.address,
            public_key: "".to_string(),
            created_at: encrypted_wallet.created_at,
            last_accessed: encrypted_wallet.last_accessed,
            balance: 0.0,
        };
        
        info!("Imported wallet: {}", wallet_info.address);
        Ok(wallet_info)
    }
    
    /// Delete a wallet
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `password` - Wallet password (for verification)
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if deleted successfully
    pub fn delete_wallet(&mut self, address: &str, password: &str) -> Result<()> {
        // Verify password by trying to decrypt
        let _ = self.get_wallet_data(address, password)?;
        
        // Remove from memory
        self.wallets.remove(address);
        
        // Remove from storage
        if let Some(ref storage_path) = self.storage_path {
            let storage = crate::storage::BlockchainStorage::new(storage_path)?;
            // Note: sled doesn't have a delete method, we'll overwrite with empty data
            storage.save_wallet(address, &[])?;
        }
        
        info!("Deleted wallet: {}", address);
        Ok(())
    }
    
    // Private helper methods
    
    fn encrypt_wallet_data(&self, data: &[u8], password: &str, wallet_id: &str) -> Result<EncryptedWallet> {
        // Generate salt and nonce
        let mut salt = [0u8; 32];
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut salt);
        rand::thread_rng().fill(&mut nonce_bytes);
        
        // Derive key from password and salt
        let key = self.derive_key(password, &salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| WalletError::Encryption(format!("Failed to create cipher: {}", e)))?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt data
        let encrypted_data = cipher.encrypt(nonce, data)
            .map_err(|e| WalletError::Encryption(format!("Encryption failed: {}", e)))?;
        
        let address = self.get_address_from_wallet_id(wallet_id)?;
        
        Ok(EncryptedWallet {
            id: wallet_id.to_string(),
            address,
            encrypted_data,
            salt: salt.to_vec(),
            nonce: nonce_bytes.to_vec(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        })
    }
    
    fn decrypt_wallet_data(&self, encrypted_data: &[u8], salt: &[u8], nonce: &[u8], password: &str) -> Result<Vec<u8>> {
        // Derive key from password and salt
        let key = self.derive_key(password, salt)?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| WalletError::Decryption(format!("Failed to create cipher: {}", e)))?;
        
        let nonce = Nonce::from_slice(nonce);
        
        // Decrypt data
        let decrypted_data = cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| WalletError::Decryption(format!("Decryption failed: {}", e)))?;
        
        Ok(decrypted_data)
    }
    
    fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        // Simple key derivation using PBKDF2-like approach
        // In production, use a proper KDF like Argon2
        let mut key = vec![0u8; 32];
        let password_bytes = password.as_bytes();
        
        for (i, key_byte) in key.iter_mut().enumerate() {
            let mut hash = 0u8;
            for (j, &salt_byte) in salt.iter().enumerate() {
                hash ^= password_bytes[i % password_bytes.len()] ^ salt_byte ^ (j as u8);
            }
            *key_byte = hash;
        }
        
        Ok(key)
    }
    
    fn get_wallet_data(&mut self, address: &str, password: &str) -> Result<WalletData> {
        let encrypted_wallet = self.wallets.get(address)
            .ok_or_else(|| WalletError::WalletNotFound(address.to_string()))?;
        
        let decrypted_data = self.decrypt_wallet_data(
            &encrypted_wallet.encrypted_data,
            &encrypted_wallet.salt,
            &encrypted_wallet.nonce,
            password,
        )?;
        
        let wallet_data: WalletData = serde_json::from_slice(&decrypted_data)
            .map_err(|_| WalletError::InvalidWalletData)?;
        
        Ok(wallet_data)
    }
    
    fn decrypt_wallet_info(&self, encrypted_wallet: &EncryptedWallet, password: &str) -> Result<WalletInfo> {
        let decrypted_data = self.decrypt_wallet_data(
            &encrypted_wallet.encrypted_data,
            &encrypted_wallet.salt,
            &encrypted_wallet.nonce,
            password,
        )?;
        
        let wallet_data: WalletData = serde_json::from_slice(&decrypted_data)
            .map_err(|_| WalletError::InvalidWalletData)?;
        
        Ok(WalletInfo {
            id: encrypted_wallet.id.clone(),
            address: encrypted_wallet.address.clone(),
            public_key: wallet_data.public_key,
            created_at: encrypted_wallet.created_at,
            last_accessed: encrypted_wallet.last_accessed,
            balance: 0.0,
        })
    }
    
    fn get_address_from_wallet_id(&self, wallet_id: &str) -> Result<String> {
        // This is a simplified approach - in a real implementation,
        // you'd need to store the address with the wallet data
        // For now, we'll generate a placeholder
        Ok(format!("wallet_{}", &wallet_id[..8]))
    }
    
    /// Clone the wallet manager for background operations
    pub fn clone_for_background(&self) -> Self {
        WalletManager {
            wallets: self.wallets.clone(),
            storage_path: self.storage_path.clone(),
        }
    }
}

/// Internal wallet data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletData {
    private_key: String,
    public_key: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_wallet_creation() {
        let mut wallet_manager = WalletManager::new();
        let wallet_info = wallet_manager.create_wallet("test_password", Some("Test Wallet".to_string())).unwrap();
        
        assert!(!wallet_info.address.is_empty());
        assert!(!wallet_info.public_key.is_empty());
        assert_eq!(wallet_info.balance, 0.0);
    }
    
    #[test]
    fn test_wallet_encryption_decryption() {
        let mut wallet_manager = WalletManager::new();
        let wallet_info = wallet_manager.create_wallet("test_password", None).unwrap();
        
        // Try to load the wallet from the same manager instance (should be in memory)
        let loaded_info = wallet_manager.load_wallet(&wallet_info.address, "test_password").unwrap();
        assert_eq!(wallet_info.id, loaded_info.id);
        assert_eq!(wallet_info.public_key, loaded_info.public_key);
    }
    
    #[test]
    fn test_wallet_with_storage() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        
        let mut wallet_manager = WalletManager::with_storage(storage_path.clone());
        
        let wallet_info = wallet_manager.create_wallet("test_password", None).unwrap();
        
        // Drop the first wallet manager to release the database lock
        drop(wallet_manager);
        
        // Create new wallet manager to test loading from storage
        let mut wallet_manager2 = WalletManager::with_storage(storage_path);
        
        let loaded_info = wallet_manager2.load_wallet(&wallet_info.address, "test_password").unwrap();
        
        assert_eq!(wallet_info.id, loaded_info.id);
        assert_eq!(wallet_info.public_key, loaded_info.public_key);
    }
    
    #[test]
    fn test_wallet_signing() {
        let mut wallet_manager = WalletManager::new();
        let wallet_info = wallet_manager.create_wallet("test_password", None).unwrap();
        
        let transaction_data = b"test transaction data";
        let signature = wallet_manager.sign_transaction(&wallet_info.address, "test_password", transaction_data).unwrap();
        
        // Verify signature
        let public_key = crate::utils::hex_to_bytes(&wallet_info.public_key).unwrap();
        let public_key_array: [u8; 32] = public_key.try_into().unwrap();
        let verifying_key = VerifyingKey::from_bytes(&public_key_array).unwrap();
        
        // Convert our signature to ed25519_dalek format
        let signature_array: [u8; 64] = signature.signature.try_into().unwrap();
        let ed25519_signature = ed25519_dalek::Signature::from_bytes(&signature_array);
        
        use ed25519_dalek::{Verifier, VerifyingKey};
        assert!(verifying_key.verify(transaction_data, &ed25519_signature).is_ok());
    }
}
