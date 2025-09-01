use sha2::{Sha256, Digest};
use hex;
use log::debug;

/// Calculate SHA-256 hash of the given data
/// 
/// # Arguments
/// * `data` - The data to hash
/// 
/// # Returns
/// * `String` - The hex-encoded hash
/// 
/// # Example
/// ```
/// use gillean::utils::calculate_hash;
/// 
/// let hash = calculate_hash("Hello, Blockchain!");
/// assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
/// ```
pub fn calculate_hash<T: AsRef<[u8]>>(data: T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_ref());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Calculate SHA-256 hash of multiple data pieces concatenated
/// 
/// # Arguments
/// * `data_pieces` - Vector of data pieces to concatenate and hash
/// 
/// # Returns
/// * `String` - The hex-encoded hash
pub fn calculate_hash_concat(data_pieces: &[&str]) -> String {
    let concatenated = data_pieces.join("");
    calculate_hash(concatenated.as_bytes())
}

/// Check if a hash meets the proof-of-work difficulty requirement
/// 
/// # Arguments
/// * `hash` - The hash to check
/// * `difficulty` - Number of leading zeros required
/// 
/// # Returns
/// * `bool` - True if the hash meets the difficulty requirement
/// 
/// # Example
/// ```
/// use gillean::utils::hash_meets_difficulty;
/// 
/// // This hash has 4 leading zeros
/// let hash = "0000abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456";
/// assert!(hash_meets_difficulty(hash, 4));
/// assert!(!hash_meets_difficulty(hash, 5));
/// ```
pub fn hash_meets_difficulty(hash: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        return true;
    }
    
    let target = "0".repeat(difficulty as usize);
    hash.starts_with(&target)
}

/// Convert bytes to hex string
/// 
/// # Arguments
/// * `bytes` - The bytes to convert
/// 
/// # Returns
/// * `String` - The hex-encoded string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
/// 
/// # Arguments
/// * `hex_str` - The hex string to convert
/// 
/// # Returns
/// * `Result<Vec<u8>>` - The decoded bytes or an error
pub fn hex_to_bytes(hex_str: &str) -> crate::Result<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| crate::BlockchainError::SerializationError(e.to_string()))
}

/// Generate a random hex string of specified length
/// 
/// # Arguments
/// * `length` - The length of the hex string to generate
/// 
/// # Returns
/// * `String` - A random hex string
pub fn generate_random_hex(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..length / 2).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

/// Validate that a string is a valid hex string
/// 
/// # Arguments
/// * `hex_str` - The string to validate
/// 
/// # Returns
/// * `bool` - True if the string is valid hex
pub fn is_valid_hex(hex_str: &str) -> bool {
    hex_str.chars().all(|c| c.is_ascii_hexdigit())
}

/// Calculate the size of a block in bytes (approximate)
/// 
/// # Arguments
/// * `block_data` - The block data to measure
/// 
/// # Returns
/// * `usize` - The approximate size in bytes
pub fn calculate_block_size(block_data: &str) -> usize {
    block_data.len()
}

/// Format a timestamp for display
/// 
/// # Arguments
/// * `timestamp` - The timestamp to format
/// 
/// # Returns
/// * `String` - The formatted timestamp
pub fn format_timestamp(timestamp: i64) -> String {
    use chrono::DateTime;
    let dt = DateTime::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Log a debug message with a prefix
/// 
/// # Arguments
/// * `prefix` - The prefix for the log message
/// * `message` - The message to log
pub fn debug_log(prefix: &str, message: &str) {
    debug!("[{}] {}", prefix, message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash = calculate_hash("test");
        assert_eq!(hash.len(), 64);
        assert!(is_valid_hex(&hash));
    }

    #[test]
    fn test_calculate_hash_concat() {
        let pieces = vec!["hello", "world", "blockchain"];
        let hash = calculate_hash_concat(&pieces);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_meets_difficulty() {
        let hash = "0000abcdef1234567890abcdef1234567890abcdef1234567890abcdef123456";
        assert!(hash_meets_difficulty(hash, 4));
        assert!(!hash_meets_difficulty(hash, 5));
        assert!(hash_meets_difficulty(hash, 0));
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x01, 0x02, 0x03, 0x04];
        let hex_str = bytes_to_hex(&bytes);
        assert_eq!(hex_str, "01020304");
    }

    #[test]
    fn test_hex_to_bytes() {
        let hex_str = "01020304";
        let bytes = hex_to_bytes(hex_str).unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_generate_random_hex() {
        let hex_str = generate_random_hex(32);
        assert_eq!(hex_str.len(), 32);
        assert!(is_valid_hex(&hex_str));
    }

    #[test]
    fn test_is_valid_hex() {
        assert!(is_valid_hex("abcdef123456"));
        assert!(is_valid_hex("ABCDEF123456"));
        assert!(!is_valid_hex("abcdef12345g"));
        assert!(!is_valid_hex("not hex"));
    }

    #[test]
    fn test_calculate_block_size() {
        let data = "test block data";
        let size = calculate_block_size(data);
        assert_eq!(size, data.len());
    }
}
