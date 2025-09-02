use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use log::debug;
use crate::{Result, BlockchainError, utils};
use argon2::{Argon2, PasswordHasher, password_hash::{rand_core::OsRng as ArgonOsRng, SaltString}};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

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
    /// Generate a new random key pair using cryptographically secure random number generation
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
        // Use cryptographically secure random number generator
        let mut rng = OsRng;
        
        // Generate 32 bytes for the secret key
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        
        // Validate that we got non-zero entropy
        if secret_bytes.iter().all(|&b| b == 0) {
            return Err(BlockchainError::TransactionValidationFailed(
                "Failed to generate secure random bytes".to_string(),
            ));
        }
        
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        let keypair = KeyPair {
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
        };

        debug!("Generated new key pair with public key: {}", utils::bytes_to_hex(&keypair.public_key));
        Ok(keypair)
    }

    /// Generate a key pair from a password using Argon2 key derivation
    /// 
    /// # Arguments
    /// * `password` - The password to derive the key from
    /// * `salt` - Optional salt (if None, a random salt will be generated)
    /// 
    /// # Returns
    /// * `Result<KeyPair>` - The derived key pair or an error
    pub fn from_password(password: &str, salt: Option<&[u8]>) -> Result<Self> {
        if password.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Password cannot be empty".to_string(),
            ));
        }

        let salt_string = if let Some(salt_bytes) = salt {
            if salt_bytes.len() != 16 {
                return Err(BlockchainError::TransactionValidationFailed(
                    "Salt must be 16 bytes".to_string(),
                ));
            }
            SaltString::encode_b64(salt_bytes)
                .map_err(|e| BlockchainError::TransactionValidationFailed(
                    format!("Invalid salt: {}", e)
                ))?
        } else {
            SaltString::generate(&mut ArgonOsRng)
        };

        // Use Argon2id for key derivation
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| BlockchainError::TransactionValidationFailed(
                format!("Key derivation failed: {}", e)
            ))?;

        // Extract the hash bytes and use first 32 bytes as private key
        let hash = password_hash.hash.unwrap();
        let hash_bytes = hash.as_bytes();
        if hash_bytes.len() < 32 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Derived key too short".to_string(),
            ));
        }

        let mut private_key_bytes = [0u8; 32];
        private_key_bytes.copy_from_slice(&hash_bytes[..32]);
        
        let signing_key = SigningKey::from_bytes(&private_key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(KeyPair {
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
        })
    }

    /// Generate a key pair using PBKDF2 key derivation
    /// 
    /// # Arguments
    /// * `password` - The password to derive the key from
    /// * `salt` - The salt to use for key derivation
    /// * `iterations` - Number of iterations (minimum 100,000 recommended)
    /// 
    /// # Returns
    /// * `Result<KeyPair>` - The derived key pair or an error
    pub fn from_password_pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Result<Self> {
        if password.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Password cannot be empty".to_string(),
            ));
        }

        if salt.len() < 16 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Salt must be at least 16 bytes".to_string(),
            ));
        }

        if iterations < 100_000 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Iterations must be at least 100,000 for security".to_string(),
            ));
        }

        let mut private_key_bytes = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut private_key_bytes);
        
        let signing_key = SigningKey::from_bytes(&private_key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(KeyPair {
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
        })
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

    #[test]
    fn test_password_based_key_derivation_argon2() {
        let password = "secure_password_123";
        let keypair1 = KeyPair::from_password(password, None).unwrap();
        let keypair2 = KeyPair::from_password(password, None).unwrap();
        
        // Different salts should produce different keys
        assert_ne!(keypair1.private_key, keypair2.private_key);
        
        // But both should be valid keypairs
        assert_eq!(keypair1.public_key.len(), 32);
        assert_eq!(keypair1.private_key.len(), 32);
        assert_eq!(keypair2.public_key.len(), 32);
        assert_eq!(keypair2.private_key.len(), 32);
        
        // Test signing and verification
        let message = "Test message";
        let signature1 = keypair1.sign(message.as_bytes()).unwrap();
        let signature2 = keypair2.sign(message.as_bytes()).unwrap();
        
        assert!(signature1.verify(message.as_bytes()).unwrap());
        assert!(signature2.verify(message.as_bytes()).unwrap());
    }

    #[test]
    fn test_password_based_key_derivation_argon2_with_salt() {
        let password = "secure_password_123";
        let salt = b"fixed_salt_16byt"; // Exactly 16 bytes
        
        let keypair1 = KeyPair::from_password(password, Some(salt)).unwrap();
        let keypair2 = KeyPair::from_password(password, Some(salt)).unwrap();
        
        // Same password and salt should produce same keys
        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key, keypair2.public_key);
    }

    #[test]
    fn test_password_based_key_derivation_pbkdf2() {
        let password = "secure_password_123";
        let salt = b"fixed_salt_16byt"; // Exactly 16 bytes
        let iterations = 100_000;
        
        let keypair1 = KeyPair::from_password_pbkdf2(password, salt, iterations).unwrap();
        let keypair2 = KeyPair::from_password_pbkdf2(password, salt, iterations).unwrap();
        
        // Same password, salt, and iterations should produce same keys
        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key, keypair2.public_key);
        
        // Test signing and verification
        let message = "Test message";
        let signature = keypair1.sign(message.as_bytes()).unwrap();
        assert!(signature.verify(message.as_bytes()).unwrap());
    }

    #[test]
    fn test_password_validation() {
        // Empty password should fail
        assert!(KeyPair::from_password("", None).is_err());
        assert!(KeyPair::from_password_pbkdf2("", b"valid_salt_16by", 100_000).is_err());
        
        // Invalid salt length should fail
        assert!(KeyPair::from_password("password", Some(b"short")).is_err());
        assert!(KeyPair::from_password_pbkdf2("password", b"short", 100_000).is_err());
        
        // Insufficient iterations should fail
        assert!(KeyPair::from_password_pbkdf2("password", b"valid_salt_16by", 1000).is_err());
    }

    #[test]
    fn test_secure_random_generation() {
        // Generate multiple keypairs and ensure they're different
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        let keypair3 = KeyPair::generate().unwrap();
        
        assert_ne!(keypair1.private_key, keypair2.private_key);
        assert_ne!(keypair2.private_key, keypair3.private_key);
        assert_ne!(keypair1.private_key, keypair3.private_key);
        
        assert_ne!(keypair1.public_key, keypair2.public_key);
        assert_ne!(keypair2.public_key, keypair3.public_key);
        assert_ne!(keypair1.public_key, keypair3.public_key);
    }

    #[test]
    fn test_key_derivation_deterministic() {
        let password = "test_password";
        let salt = b"deterministic_16"; // Exactly 16 bytes
        
        // Generate keypair multiple times with same parameters
        let keypair1 = KeyPair::from_password(password, Some(salt)).unwrap();
        let keypair2 = KeyPair::from_password(password, Some(salt)).unwrap();
        
        // Should be identical
        assert_eq!(keypair1.private_key, keypair2.private_key);
        assert_eq!(keypair1.public_key, keypair2.public_key);
        
        // Test with PBKDF2 as well
        let keypair3 = KeyPair::from_password_pbkdf2(password, salt, 100_000).unwrap();
        let keypair4 = KeyPair::from_password_pbkdf2(password, salt, 100_000).unwrap();
        
        assert_eq!(keypair3.private_key, keypair4.private_key);
        assert_eq!(keypair3.public_key, keypair4.public_key);
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let salt = b"same_salt_16byte"; // Exactly 16 bytes
        
        let keypair1 = KeyPair::from_password("password1", Some(salt)).unwrap();
        let keypair2 = KeyPair::from_password("password2", Some(salt)).unwrap();
        
        // Different passwords should produce different keys
        assert_ne!(keypair1.private_key, keypair2.private_key);
        assert_ne!(keypair1.public_key, keypair2.public_key);
    }
}
