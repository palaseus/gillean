use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use log::debug;
use crate::{Result, BlockchainError, utils};

/// Cryptographic key pair for signing transactions
/// 
/// Contains both the signing key (private) and verifying key (public)
/// for Ed25519 digital signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key for verification
    pub public_key: Vec<u8>,
    /// Private key for signing (should be kept secret)
    pub private_key: Vec<u8>,
}

/// Public key for transaction verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PublicKey {
    /// Raw public key bytes
    pub key: Vec<u8>,
}

/// Digital signature for transaction authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DigitalSignature {
    /// Raw signature bytes
    pub signature: Vec<u8>,
    /// Public key of the signer
    pub public_key: Vec<u8>,
}

impl KeyPair {
    /// Generate a new random key pair
    /// 
    /// # Returns
    /// * `Result<KeyPair>` - The generated key pair or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::crypto::KeyPair;
    /// 
    /// let keypair = KeyPair::generate().unwrap();
    /// assert!(!keypair.public_key.is_empty());
    /// assert!(!keypair.private_key.is_empty());
    /// ```
    pub fn generate() -> Result<Self> {
        let mut rng = OsRng;
        
        // Generate 32 bytes for the secret key
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let keypair = KeyPair {
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
        };

        debug!("Generated new key pair with public key: {}", utils::bytes_to_hex(&keypair.public_key));
        Ok(keypair)
    }

    /// Create a key pair from existing keys
    /// 
    /// # Arguments
    /// * `public_key` - The public key bytes
    /// * `private_key` - The private key bytes
    /// 
    /// # Returns
    /// * `Result<KeyPair>` - The key pair or an error
    pub fn from_keys(public_key: Vec<u8>, private_key: Vec<u8>) -> Result<Self> {
        // Validate key lengths
        if public_key.len() != 32 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid public key length".to_string(),
            ));
        }

        if private_key.len() != 32 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid private key length".to_string(),
            ));
        }

        Ok(KeyPair {
            public_key,
            private_key,
        })
    }

    /// Create a key pair from private key bytes
    /// 
    /// # Arguments
    /// * `private_key_bytes` - The private key bytes
    /// 
    /// # Returns
    /// * `Result<KeyPair>` - The key pair or an error
    pub fn from_private_key_bytes(private_key_bytes: &[u8]) -> Result<Self> {
        // Validate private key length
        if private_key_bytes.len() != 32 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid private key length".to_string(),
            ));
        }

        let private_key_bytes: [u8; 32] = private_key_bytes.try_into()
            .map_err(|_| BlockchainError::TransactionValidationFailed(
                "Invalid private key length".to_string()
            ))?;
        
        let signing_key = SigningKey::from_bytes(&private_key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(KeyPair {
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
        })
    }

    /// Sign a message with the private key
    /// 
    /// # Arguments
    /// * `message` - The message to sign
    /// 
    /// # Returns
    /// * `Result<DigitalSignature>` - The digital signature or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::crypto::KeyPair;
    /// 
    /// let keypair = KeyPair::generate().unwrap();
    /// let message = "Hello, Blockchain!";
    /// let signature = keypair.sign(message.as_bytes()).unwrap();
    /// 
    /// assert!(signature.verify(message.as_bytes()).unwrap());
    /// ```
    pub fn sign(&self, message: &[u8]) -> Result<DigitalSignature> {
        let private_key_bytes: [u8; 32] = self.private_key.as_slice().try_into()
            .map_err(|_| BlockchainError::TransactionValidationFailed(
                "Invalid private key length".to_string()
            ))?;
        
        let signing_key = SigningKey::from_bytes(&private_key_bytes);

        let signature = signing_key.sign(message);
        
        let digital_signature = DigitalSignature {
            signature: signature.to_bytes().to_vec(),
            public_key: self.public_key.clone(),
        };

        debug!("Signed message with {} bytes", message.len());
        Ok(digital_signature)
    }

    /// Get the public key
    /// 
    /// # Returns
    /// * `PublicKey` - The public key
    pub fn public_key(&self) -> PublicKey {
        PublicKey {
            key: self.public_key.clone(),
        }
    }

    /// Get the public key as a hex string
    /// 
    /// # Returns
    /// * `String` - The hex-encoded public key
    pub fn public_key_hex(&self) -> String {
        utils::bytes_to_hex(&self.public_key)
    }

    /// Get the private key as a hex string
    /// 
    /// # Returns
    /// * `String` - The hex-encoded private key
    pub fn private_key_hex(&self) -> String {
        utils::bytes_to_hex(&self.private_key)
    }
}

impl PublicKey {
    /// Create a public key from bytes
    /// 
    /// # Arguments
    /// * `key` - The public key bytes
    /// 
    /// # Returns
    /// * `Result<PublicKey>` - The public key or an error
    pub fn from_bytes(key: Vec<u8>) -> Result<Self> {
        if key.len() != 32 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid public key length".to_string(),
            ));
        }

        Ok(PublicKey { key })
    }

    /// Create a public key from a hex string
    /// 
    /// # Arguments
    /// * `hex_key` - The hex-encoded public key
    /// 
    /// # Returns
    /// * `Result<PublicKey>` - The public key or an error
    pub fn from_hex(hex_key: &str) -> Result<Self> {
        let key = utils::hex_to_bytes(hex_key)?;
        Self::from_bytes(key)
    }

    /// Get the public key as a hex string
    /// 
    /// # Returns
    /// * `String` - The hex-encoded public key
    pub fn to_hex(&self) -> String {
        utils::bytes_to_hex(&self.key)
    }

    /// Get the short version of the public key (first 8 characters)
    /// 
    /// # Returns
    /// * `String` - The short public key
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl DigitalSignature {
    /// Create a new digital signature
    /// 
    /// # Arguments
    /// * `signature` - The signature bytes
    /// * `public_key` - The public key bytes
    /// 
    /// # Returns
    /// * `DigitalSignature` - The digital signature
    pub fn new(signature: Vec<u8>, public_key: Vec<u8>) -> Self {
        DigitalSignature {
            signature,
            public_key,
        }
    }

    /// Verify the signature against a message
    /// 
    /// # Arguments
    /// * `message` - The message that was signed
    /// 
    /// # Returns
    /// * `Result<bool>` - True if signature is valid, error otherwise
    /// 
    /// # Example
    /// ```
    /// use gillean::crypto::{KeyPair, DigitalSignature};
    /// 
    /// let keypair = KeyPair::generate().unwrap();
    /// let message = "Test message";
    /// let signature = keypair.sign(message.as_bytes()).unwrap();
    /// 
    /// assert!(signature.verify(message.as_bytes()).unwrap());
    /// ```
    pub fn verify(&self, message: &[u8]) -> Result<bool> {
        let public_key_bytes: [u8; 32] = self.public_key.as_slice().try_into()
            .map_err(|_| BlockchainError::TransactionValidationFailed(
                "Invalid public key length".to_string()
            ))?;
        
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| BlockchainError::TransactionValidationFailed(
                format!("Invalid public key: {}", e)
            ))?;

        let signature_bytes: [u8; 64] = self.signature.as_slice().try_into()
            .map_err(|_| BlockchainError::TransactionValidationFailed(
                "Invalid signature length".to_string()
            ))?;
        
        let signature = Signature::from_bytes(&signature_bytes);

        let is_valid = verifying_key.verify(message, &signature).is_ok();
        
        if is_valid {
            debug!("Signature verification successful");
        } else {
            debug!("Signature verification failed");
        }

        Ok(is_valid)
    }

    /// Get the signature as a hex string
    /// 
    /// # Returns
    /// * `String` - The hex-encoded signature
    pub fn to_hex(&self) -> String {
        utils::bytes_to_hex(&self.signature)
    }

    /// Get the public key as a hex string
    /// 
    /// # Returns
    /// * `String` - The hex-encoded public key
    pub fn public_key_hex(&self) -> String {
        utils::bytes_to_hex(&self.public_key)
    }

    /// Get the size of the signature in bytes
    /// 
    /// # Returns
    /// * `usize` - The size in bytes
    pub fn size(&self) -> usize {
        self.signature.len() + self.public_key.len()
    }
}

/// Generate a random key pair for testing
/// 
/// # Returns
/// * `KeyPair` - A randomly generated key pair
pub fn generate_test_keypair() -> KeyPair {
    KeyPair::generate().expect("Failed to generate test keypair")
}

/// Create a wallet address from a public key
/// 
/// # Arguments
/// * `public_key` - The public key
/// 
/// # Returns
/// * `String` - The wallet address
pub fn create_address(public_key: &PublicKey) -> String {
    let hash = utils::calculate_hash(public_key.to_hex());
    format!("GIL{}", &hash[..40]) // GIL + first 40 chars of hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        
        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.private_key.len(), 32);
        assert!(!keypair.public_key_hex().is_empty());
        assert!(!keypair.private_key_hex().is_empty());
    }

    #[test]
    fn test_keypair_from_keys() {
        let original = KeyPair::generate().unwrap();
        let keypair = KeyPair::from_keys(original.public_key.clone(), original.private_key.clone()).unwrap();
        
        assert_eq!(keypair.public_key, original.public_key);
        assert_eq!(keypair.private_key, original.private_key);
    }

    #[test]
    fn test_invalid_key_lengths() {
        let result = KeyPair::from_keys(vec![1, 2, 3], vec![1, 2, 3, 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate().unwrap();
        let message = "Hello, Blockchain!";
        
        let signature = keypair.sign(message.as_bytes()).unwrap();
        assert!(signature.verify(message.as_bytes()).unwrap());
    }

    #[test]
    fn test_signature_tampering() {
        let keypair = KeyPair::generate().unwrap();
        let message = "Original message";
        
        let mut signature = keypair.sign(message.as_bytes()).unwrap();
        signature.signature[0] ^= 1; // Tamper with signature
        
        assert!(!signature.verify(message.as_bytes()).unwrap());
    }

    #[test]
    fn test_wrong_message() {
        let keypair = KeyPair::generate().unwrap();
        let original_message = "Original message";
        let wrong_message = "Wrong message";
        
        let signature = keypair.sign(original_message.as_bytes()).unwrap();
        assert!(!signature.verify(wrong_message.as_bytes()).unwrap());
    }

    #[test]
    fn test_public_key_operations() {
        let keypair = KeyPair::generate().unwrap();
        let public_key = keypair.public_key();
        
        let hex_key = public_key.to_hex();
        let restored = PublicKey::from_hex(&hex_key).unwrap();
        
        assert_eq!(public_key, restored);
        assert_eq!(public_key.short().len(), 8);
    }

    #[test]
    fn test_address_creation() {
        let keypair = KeyPair::generate().unwrap();
        let public_key = keypair.public_key();
        let address = create_address(&public_key);
        
        assert!(address.starts_with("GIL"));
        assert_eq!(address.len(), 43); // GIL + 40 chars
    }

    #[test]
    fn test_signature_size() {
        let keypair = KeyPair::generate().unwrap();
        let message = "Test message";
        let signature = keypair.sign(message.as_bytes()).unwrap();
        
        assert!(signature.size() > 0);
        assert!(!signature.to_hex().is_empty());
        assert!(!signature.public_key_hex().is_empty());
    }
}
