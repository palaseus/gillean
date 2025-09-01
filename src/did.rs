use crate::error::BlockchainError;
use crate::storage::BlockchainStorage;

use crate::crypto::{KeyPair, PublicKey};
use ed25519_dalek::{SigningKey, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{self, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use base64::Engine;



/// DID Document structure following W3C DID specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    pub id: String,
    pub controller: Option<String>,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub service_endpoints: Vec<ServiceEndpoint>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

/// Verification method for DID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    pub key_type: String,
    pub public_key_multibase: String,
    pub public_key_jwk: Option<serde_json::Value>,
}

/// Service endpoint for DID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub service_endpoint: String,
}

/// Decentralized Identity management system
pub struct DecentralizedIdentity {
    storage: Arc<BlockchainStorage>,
    did_documents: Arc<RwLock<HashMap<String, DIDDocument>>>,
    wallet_links: Arc<RwLock<HashMap<String, String>>>, // wallet_address -> did
}

/// DID creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDCreationRequest {
    pub controller: Option<String>,
    pub service_endpoints: Vec<ServiceEndpoint>,
}

/// DID verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDVerificationResult {
    pub is_valid: bool,
    pub error_message: Option<String>,
    pub verification_method: Option<String>,
}

impl DecentralizedIdentity {
    /// Create a new DID system instance
    pub async fn new(storage: Arc<BlockchainStorage>) -> Result<Self, BlockchainError> {
        let did_system = Self {
            storage,
            did_documents: Arc::new(RwLock::new(HashMap::new())),
            wallet_links: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing DIDs from storage
        did_system.load_dids().await?;

        Ok(did_system)
    }

    /// Create a new DID
    pub async fn create_did(
        &self,
        request: DIDCreationRequest,
    ) -> Result<(String, SigningKey), BlockchainError> {
        // Generate new keypair
        let secret_bytes = rand::random::<[u8; 32]>();
        let keypair = SigningKey::from_bytes(&secret_bytes);
        
        // Create DID identifier
        let public_key = PublicKey { key: keypair.verifying_key().to_bytes().to_vec() };
        let did = self.generate_did_identifier(&public_key);
        
        // Create verification method
        let verification_method = VerificationMethod {
            id: format!("{}#keys-1", did),
            controller: did.clone(),
            key_type: "Ed25519VerificationKey2020".to_string(),
            public_key_multibase: self.encode_public_key_multibase(&public_key),
            public_key_jwk: None,
        };

        // Create DID document
        let now = chrono::Utc::now();
        let document = DIDDocument {
            id: did.clone(),
            controller: request.controller,
            verification_methods: vec![verification_method],
            authentication: vec![format!("{}#keys-1", did)],
            assertion_method: vec![format!("{}#keys-1", did)],
            key_agreement: vec![],
            service_endpoints: request.service_endpoints,
            created: now,
            updated: now,
        };

        // Store DID document
        {
            let mut documents = self.did_documents.write().await;
            documents.insert(did.clone(), document.clone());
        }

        // Save to persistent storage
        self.save_did_document(&document).await?;

        info!("Created new DID: {}", did);
        Ok((did, keypair))
    }

    /// Link a DID to a wallet address
    pub async fn link_did_to_wallet(
        &self,
        did: &str,
        wallet_address: &str,
    ) -> Result<(), BlockchainError> {
        // Verify DID exists
        {
            let documents = self.did_documents.read().await;
            if !documents.contains_key(did) {
                return Err(BlockchainError::NotFound(format!("DID not found: {}", did)));
            }
        }

        // Link DID to wallet
        {
            let mut links = self.wallet_links.write().await;
            links.insert(wallet_address.to_string(), did.to_string());
        }

        // Save link to storage
        self.save_wallet_link(wallet_address, did).await?;

        info!("Linked DID {} to wallet {}", did, wallet_address);
        Ok(())
    }

    /// Get DID document
    pub async fn get_did_document(&self, did: &str) -> Result<Option<DIDDocument>, BlockchainError> {
        let documents = self.did_documents.read().await;
        Ok(documents.get(did).cloned())
    }

    /// Get DID for wallet address
    pub async fn get_did_for_wallet(&self, wallet_address: &str) -> Result<Option<String>, BlockchainError> {
        let links = self.wallet_links.read().await;
        Ok(links.get(wallet_address).cloned())
    }

    /// Verify DID signature
    pub async fn verify_did_signature(
        &self,
        did: &str,
        message: &[u8],
        signature: &[u8],
    ) -> Result<DIDVerificationResult, BlockchainError> {
        // Get DID document
        let document = self.get_did_document(did).await?
            .ok_or_else(|| BlockchainError::NotFound(format!("DID not found: {}", did)))?;

        // Find verification method
        let verification_method = document.verification_methods
            .iter()
            .find(|vm| vm.id == format!("{}#keys-1", did))
            .ok_or_else(|| BlockchainError::ValidatorError("No verification method found".to_string()))?;

        // Decode public key
        let crypto_public_key = self.decode_public_key_multibase(&verification_method.public_key_multibase)?;
        let public_key_bytes: [u8; 32] = crypto_public_key.key.as_slice().try_into()
            .map_err(|_| BlockchainError::InvalidSignature("Invalid public key length".to_string()))?;
        let public_key = ed25519_dalek::VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|_| BlockchainError::InvalidSignature("Invalid public key format".to_string()))?;

        // Verify signature
        let signature_bytes: [u8; 64] = signature.try_into()
            .map_err(|_| BlockchainError::InvalidSignature("Invalid signature length".to_string()))?;
        let signature_obj = ed25519_dalek::Signature::from_bytes(&signature_bytes);
        match public_key.verify(message, &signature_obj) {
            Ok(_) => Ok(DIDVerificationResult {
                is_valid: true,
                error_message: None,
                verification_method: Some(verification_method.id.clone()),
            }),
            Err(e) => Ok(DIDVerificationResult {
                is_valid: false,
                error_message: Some(format!("Signature verification failed: {}", e)),
                verification_method: Some(verification_method.id.clone()),
            }),
        }
    }

    /// Update DID document
    pub async fn update_did_document(
        &self,
        did: &str,
        _keypair: &KeyPair,
        updates: DIDDocument,
    ) -> Result<(), BlockchainError> {
        // Verify DID exists
        {
            let documents = self.did_documents.read().await;
            if !documents.contains_key(did) {
                return Err(BlockchainError::NotFound(format!("DID not found: {}", did)));
            }
        }

        // Verify signature on update
        // In a real implementation, you'd want to verify that the update is signed by the DID controller
        let _update_message = serde_json::to_string(&updates)
            .map_err(|e| BlockchainError::SerializationError(format!("Failed to serialize update: {}", e)))?;
        
        // For now, we'll just update the document
        let mut updated_document = updates;
        updated_document.updated = chrono::Utc::now();

        // Store updated document
        {
            let mut documents = self.did_documents.write().await;
            documents.insert(did.to_string(), updated_document.clone());
        }

        // Save to persistent storage
        self.save_did_document(&updated_document).await?;

        info!("Updated DID document: {}", did);
        Ok(())
    }

    /// Revoke DID
    pub async fn revoke_did(&self, did: &str) -> Result<(), BlockchainError> {
        // Remove DID document
        {
            let mut documents = self.did_documents.write().await;
            documents.remove(did);
        }

        // Remove from storage
        self.storage.delete(&format!("did:{}", did))?;

        // Remove wallet links
        {
            let mut links = self.wallet_links.write().await;
            links.retain(|_, linked_did| linked_did != did);
        }

        info!("Revoked DID: {}", did);
        Ok(())
    }

    /// Get all DIDs
    pub async fn get_all_dids(&self) -> Result<Vec<String>, BlockchainError> {
        let documents = self.did_documents.read().await;
        Ok(documents.keys().cloned().collect())
    }

    /// Clone for background processing
    pub fn clone_for_background(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            did_documents: self.did_documents.clone(),
            wallet_links: self.wallet_links.clone(),
        }
    }

    /// Get DID statistics
    pub async fn get_did_stats(&self) -> Result<DIDStats, BlockchainError> {
        let documents = self.did_documents.read().await;
        let links = self.wallet_links.read().await;

        Ok(DIDStats {
            total_dids: documents.len() as u64,
            linked_wallets: links.len() as u64,
            verification_methods: documents.values()
                .map(|doc| doc.verification_methods.len() as u64)
                .sum(),
        })
    }

    /// Generate DID identifier
    fn generate_did_identifier(&self, public_key: &PublicKey) -> String {
        let key_bytes = &public_key.key;
        let key_hash = sha2::Sha256::digest(key_bytes);
        format!("did:gillean:{}", hex::encode(&key_hash[..16]))
    }

    /// Encode public key in multibase format
    fn encode_public_key_multibase(&self, public_key: &PublicKey) -> String {
        let key_bytes = &public_key.key;
        format!("z{}", base64::engine::general_purpose::STANDARD.encode(key_bytes))
    }

    /// Decode public key from multibase format
    fn decode_public_key_multibase(&self, multibase: &str) -> Result<PublicKey, BlockchainError> {
        if !multibase.starts_with('z') {
            return Err(BlockchainError::ValidatorError("Invalid multibase format".to_string()));
        }

        let base64_data = &multibase[1..];
        let key_bytes = base64::engine::general_purpose::STANDARD.decode(base64_data)
            .map_err(|e| BlockchainError::InvalidSignature(format!("Failed to decode base64: {}", e)))?;

        PublicKey::from_bytes(key_bytes.clone())
            .map_err(|e| BlockchainError::InvalidSignature(format!("Invalid public key: {}", e)))
    }

    /// Save DID document to storage
    async fn save_did_document(&self, document: &DIDDocument) -> Result<(), BlockchainError> {
        let key = format!("did:{}", document.id);
        let value = serde_json::to_string(document)
            .map_err(|e| BlockchainError::SerializationError(format!("Failed to serialize DID document: {}", e)))?;
        
        Ok(self.storage.set(&key, value.as_bytes())?)
    }

    /// Save wallet link to storage
    async fn save_wallet_link(&self, wallet_address: &str, did: &str) -> Result<(), BlockchainError> {
        let key = format!("wallet_link:{}", wallet_address);
        Ok(self.storage.set(&key, did.as_bytes())?)
    }

    /// Load DIDs from storage
    async fn load_dids(&self) -> Result<(), BlockchainError> {
        // Load DID documents
        let did_prefix = "did:";
        let documents = self.storage.get_by_prefix(did_prefix)?;
        
        let mut did_documents = self.did_documents.write().await;
        
        for (key, value) in documents.iter() {
            if let Ok(document) = serde_json::from_str::<DIDDocument>(&String::from_utf8_lossy(value)) {
                let did = key.strip_prefix(did_prefix).unwrap_or(key).to_string();
                did_documents.insert(did, document);
            }
        }

        // Load wallet links
        let link_prefix = "wallet_link:";
        let links = self.storage.get_by_prefix(link_prefix)?;
        
        let mut wallet_links = self.wallet_links.write().await;
        
        for (key, value) in links {
            let wallet_address = key.strip_prefix(link_prefix).unwrap_or(&key).to_string();
            wallet_links.insert(wallet_address, String::from_utf8_lossy(&value).to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DIDStats {
    pub total_dids: u64,
    pub linked_wallets: u64,
    pub verification_methods: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::BlockchainStorage;
    use tempfile::tempdir;
    use ed25519_dalek::Signer;

    #[tokio::test]
    async fn test_did_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(BlockchainStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let did_system = DecentralizedIdentity::new(storage).await.unwrap();

        let request = DIDCreationRequest {
            controller: None,
            service_endpoints: vec![],
        };

        let (did, _keypair) = did_system.create_did(request).await.unwrap();
        assert!(did.starts_with("did:gillean:"));
        assert_eq!(did_system.get_all_dids().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_did_linking() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(BlockchainStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let did_system = DecentralizedIdentity::new(storage).await.unwrap();

        let request = DIDCreationRequest {
            controller: None,
            service_endpoints: vec![],
        };

        let (did, _) = did_system.create_did(request).await.unwrap();
        let wallet_address = "alice";

        did_system.link_did_to_wallet(&did, wallet_address).await.unwrap();
        
        let linked_did = did_system.get_did_for_wallet(wallet_address).await.unwrap();
        assert_eq!(linked_did, Some(did));
    }

    #[tokio::test]
    async fn test_did_verification() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(BlockchainStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let did_system = DecentralizedIdentity::new(storage).await.unwrap();

        let request = DIDCreationRequest {
            controller: None,
            service_endpoints: vec![],
        };

        let (did, keypair) = did_system.create_did(request).await.unwrap();
        
        let message = b"Hello, DID!";
        let signature = keypair.sign(message);
        
        let result = did_system.verify_did_signature(&did, message, &signature.to_bytes()).await.unwrap();
        assert!(result.is_valid);
    }
}
