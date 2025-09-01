use serde::{Deserialize, Serialize};
use log::{debug, info, warn};
use crate::{Result, BlockchainError, utils};

/// Proof of Work implementation for blockchain mining
/// 
/// This module handles the mining process where miners compete to find a nonce
/// that produces a hash with a specified number of leading zeros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    /// The difficulty level (number of leading zeros required)
    pub difficulty: u32,
    /// Maximum number of attempts before timing out
    pub max_attempts: u64,
    /// Current target hash pattern
    pub target: String,
}

impl ProofOfWork {
    /// Create a new Proof of Work instance
    /// 
    /// # Arguments
    /// * `difficulty` - Number of leading zeros required
    /// * `max_attempts` - Maximum mining attempts before timeout
    /// 
    /// # Returns
    /// * `Result<ProofOfWork>` - The PoW instance or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::proof_of_work::ProofOfWork;
    /// 
    /// let pow = ProofOfWork::new(4, 1000000).unwrap();
    /// assert_eq!(pow.difficulty, 4);
    /// ```
    pub fn new(difficulty: u32, max_attempts: u64) -> Result<Self> {
        if difficulty > 32 {
            return Err(BlockchainError::InvalidDifficulty(difficulty));
        }

        let target = "0".repeat(difficulty as usize);
        
        Ok(ProofOfWork {
            difficulty,
            max_attempts,
            target,
        })
    }

    /// Create a default Proof of Work instance with difficulty 4
    /// 
    /// # Returns
    /// * `ProofOfWork` - The default PoW instance
    pub fn new_default() -> Self {
        Self::new(4, 1_000_000).expect("Default PoW should be valid")
    }

    /// Mine a block by finding a valid nonce
    /// 
    /// # Arguments
    /// * `block_data` - The block data to mine (without nonce)
    /// * `previous_hash` - Hash of the previous block
    /// 
    /// # Returns
    /// * `Result<(u64, String)>` - Tuple of (nonce, hash) or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::proof_of_work::ProofOfWork;
    /// 
    /// let pow = ProofOfWork::new(2, 1000).unwrap();
    /// let (nonce, hash) = pow.mine("block data", "previous_hash").unwrap();
    /// assert!(hash.starts_with("00"));
    /// ```
    pub fn mine(&self, block_data: &str, previous_hash: &str) -> Result<(u64, String)> {
        info!("Starting mining with difficulty {}", self.difficulty);
        debug!("Mining block data: {}", block_data);
        debug!("Previous hash: {}", previous_hash);

        let start_time = std::time::Instant::now();
        let mut attempts = 0u64;

        loop {
            attempts += 1;

            if attempts > self.max_attempts {
                warn!("Mining timeout after {} attempts", attempts);
                return Err(BlockchainError::MiningTimeout(attempts));
            }

            // Create the data to hash
            let data = format!("{}:{}:{}", block_data, previous_hash, attempts);
            let hash = utils::calculate_hash(data);

            // Check if the hash meets the difficulty requirement
            if utils::hash_meets_difficulty(&hash, self.difficulty) {
                let duration = start_time.elapsed();
                info!(
                    "Mining successful! Nonce: {}, Hash: {}, Attempts: {}, Time: {:?}",
                    attempts, hash, attempts, duration
                );
                return Ok((attempts, hash));
            }

            // Log progress every 10000 attempts
            if attempts % 10000 == 0 {
                debug!("Mining attempt {}: {}", attempts, hash);
            }
        }
    }

    /// Validate that a hash meets the proof of work requirements
    /// 
    /// # Arguments
    /// * `hash` - The hash to validate
    /// 
    /// # Returns
    /// * `bool` - True if the hash is valid
    pub fn validate_hash(&self, hash: &str) -> bool {
        utils::hash_meets_difficulty(hash, self.difficulty)
    }

    /// Validate a complete mining solution
    /// 
    /// # Arguments
    /// * `block_data` - The block data that was mined
    /// * `previous_hash` - Hash of the previous block
    /// * `nonce` - The nonce that was found
    /// * `hash` - The resulting hash
    /// 
    /// # Returns
    /// * `Result<bool>` - True if the solution is valid
    pub fn validate_solution(
        &self,
        block_data: &str,
        previous_hash: &str,
        nonce: u64,
        hash: &str,
    ) -> Result<bool> {
        // Recalculate the hash to verify
        let data = format!("{}:{}:{}", block_data, previous_hash, nonce);
        let calculated_hash = utils::calculate_hash(data);

        if calculated_hash != hash {
            return Err(BlockchainError::InvalidHash(format!(
                "Hash mismatch: expected {}, got {}",
                calculated_hash, hash
            )));
        }

        // Check if the hash meets difficulty requirements
        if !self.validate_hash(hash) {
            return Err(BlockchainError::InvalidProofOfWork(format!(
                "Hash {} does not meet difficulty requirement of {} leading zeros",
                hash, self.difficulty
            )));
        }

        Ok(true)
    }

    /// Adjust difficulty based on mining time
    /// 
    /// # Arguments
    /// * `target_time` - Target time for mining (in seconds)
    /// * `actual_time` - Actual time taken for mining (in seconds)
    /// 
    /// # Returns
    /// * `u32` - The adjusted difficulty
    pub fn adjust_difficulty(&self, target_time: f64, actual_time: f64) -> u32 {
        let ratio = actual_time / target_time;
        let mut new_difficulty = self.difficulty as f64;

        if ratio > 1.5 {
            // Mining took too long, decrease difficulty
            new_difficulty -= 1.0;
        } else if ratio < 0.5 {
            // Mining was too fast, increase difficulty
            new_difficulty += 1.0;
        }

        // Ensure difficulty stays within reasonable bounds
        new_difficulty = new_difficulty.clamp(1.0, 32.0);
        
        info!(
            "Adjusting difficulty from {} to {} (ratio: {:.2})",
            self.difficulty, new_difficulty as u32, ratio
        );

        new_difficulty as u32
    }

    /// Get the current target pattern
    /// 
    /// # Returns
    /// * `String` - The target pattern
    pub fn get_target(&self) -> &str {
        &self.target
    }

    /// Get the probability of finding a valid hash
    /// 
    /// # Returns
    /// * `f64` - The probability (0.0 to 1.0)
    pub fn get_probability(&self) -> f64 {
        if self.difficulty == 0 {
            return 1.0;
        }
        
        // Probability = 1 / (16^difficulty)
        // Since we're using hex, each character has 16 possible values
        1.0 / (16.0_f64.powi(self.difficulty as i32))
    }

    /// Get estimated attempts needed to find a valid hash
    /// 
    /// # Returns
    /// * `u64` - Estimated number of attempts
    pub fn get_estimated_attempts(&self) -> u64 {
        if self.difficulty == 0 {
            return 1;
        }
        
        // Expected attempts = 1 / probability
        (1.0 / self.get_probability()) as u64
    }
}

impl Default for ProofOfWork {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_creation() {
        let pow = ProofOfWork::new(4, 1000).unwrap();
        assert_eq!(pow.difficulty, 4);
        assert_eq!(pow.max_attempts, 1000);
        assert_eq!(pow.target, "0000");
    }

    #[test]
    fn test_pow_default() {
        let pow = ProofOfWork::default();
        assert_eq!(pow.difficulty, 4);
    }

    #[test]
    fn test_invalid_difficulty() {
        let result = ProofOfWork::new(33, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_mining_low_difficulty() {
        let pow = ProofOfWork::new(1, 1000).unwrap();
        let (nonce, hash) = pow.mine("test data", "previous_hash").unwrap();
        
        assert!(hash.starts_with('0'));
        assert!(nonce > 0);
    }

    #[test]
    fn test_validate_hash() {
        let pow = ProofOfWork::new(2, 1000).unwrap();
        
        assert!(pow.validate_hash("00abcdef"));
        assert!(!pow.validate_hash("0abcdef"));
        assert!(!pow.validate_hash("abcdef"));
    }

    #[test]
    fn test_validate_solution() {
        let pow = ProofOfWork::new(1, 1000).unwrap();
        let (nonce, hash) = pow.mine("test data", "previous_hash").unwrap();
        
        assert!(pow.validate_solution("test data", "previous_hash", nonce, &hash).unwrap());
    }

    #[test]
    fn test_invalid_solution() {
        let pow = ProofOfWork::new(1, 1000).unwrap();
        
        let result = pow.validate_solution("test data", "previous_hash", 1, "invalid_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_adjust_difficulty() {
        let pow = ProofOfWork::new(4, 1000).unwrap();
        
        // Mining took too long
        let new_diff = pow.adjust_difficulty(10.0, 20.0);
        assert_eq!(new_diff, 3);
        
        // Mining was too fast
        let new_diff = pow.adjust_difficulty(10.0, 3.0);
        assert_eq!(new_diff, 5);
    }

    #[test]
    fn test_get_probability() {
        let pow = ProofOfWork::new(1, 1000).unwrap();
        let prob = pow.get_probability();
        assert_eq!(prob, 1.0 / 16.0);
    }

    #[test]
    fn test_get_estimated_attempts() {
        let pow = ProofOfWork::new(1, 1000).unwrap();
        let attempts = pow.get_estimated_attempts();
        assert_eq!(attempts, 16);
    }
}
