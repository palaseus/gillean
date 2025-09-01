//! # Counter Smart Contract
//! 
//! A simple counter contract that demonstrates basic smart contract functionality
//! on the Gillean blockchain platform.
//! 
//! ## Features
//! 
//! - Increment counter
//! - Decrement counter
//! - Reset counter
//! - Get current value
//! 
//! ## Usage
//! 
//! ```rust
//! // Deploy the contract
//! let counter = Counter::new();
//! 
//! // Get initial value
//! let value = counter.get_value(); // Returns 0
//! 
//! // Increment the counter
//! counter.increment();
//! let new_value = counter.get_value(); // Returns 1
//! 
//! // Decrement the counter
//! counter.decrement();
//! let final_value = counter.get_value(); // Returns 0
//! ```

use gillean_contract::*;
use serde::{Deserialize, Serialize};

/// Counter smart contract
#[contract]
pub struct Counter {
    /// Current counter value
    value: u64,
    /// Contract owner
    owner: Address,
    /// Maximum value limit
    max_value: u64,
}

impl Counter {
    /// Create a new counter contract
    #[constructor]
    pub fn new() -> Self {
        Self {
            value: 0,
            owner: get_caller(),
            max_value: 1000,
        }
    }
    
    /// Create a new counter contract with custom maximum value
    #[constructor]
    pub fn new_with_max(max_value: u64) -> Self {
        Self {
            value: 0,
            owner: get_caller(),
            max_value,
        }
    }
    
    /// Get the current counter value
    #[view]
    pub fn get_value(&self) -> u64 {
        self.value
    }
    
    /// Get the contract owner
    #[view]
    pub fn get_owner(&self) -> Address {
        self.owner
    }
    
    /// Get the maximum value limit
    #[view]
    pub fn get_max_value(&self) -> u64 {
        self.max_value
    }
    
    /// Increment the counter by 1
    #[payable]
    pub fn increment(&mut self) -> Result<u64, String> {
        if self.value >= self.max_value {
            return Err("Counter has reached maximum value".to_string());
        }
        
        self.value += 1;
        emit_event("CounterIncremented", &serde_json::json!({
            "new_value": self.value,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Increment the counter by a specified amount
    #[payable]
    pub fn increment_by(&mut self, amount: u64) -> Result<u64, String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }
        
        if self.value + amount > self.max_value {
            return Err("Increment would exceed maximum value".to_string());
        }
        
        self.value += amount;
        emit_event("CounterIncremented", &serde_json::json!({
            "new_value": self.value,
            "increment": amount,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Decrement the counter by 1
    #[payable]
    pub fn decrement(&mut self) -> Result<u64, String> {
        if self.value == 0 {
            return Err("Counter cannot go below 0".to_string());
        }
        
        self.value -= 1;
        emit_event("CounterDecremented", &serde_json::json!({
            "new_value": self.value,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Decrement the counter by a specified amount
    #[payable]
    pub fn decrement_by(&mut self, amount: u64) -> Result<u64, String> {
        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }
        
        if self.value < amount {
            return Err("Decrement would go below 0".to_string());
        }
        
        self.value -= amount;
        emit_event("CounterDecremented", &serde_json::json!({
            "new_value": self.value,
            "decrement": amount,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Reset the counter to 0
    #[payable]
    pub fn reset(&mut self) -> Result<u64, String> {
        // Only owner can reset the counter
        if get_caller() != self.owner {
            return Err("Only owner can reset the counter".to_string());
        }
        
        let old_value = self.value;
        self.value = 0;
        
        emit_event("CounterReset", &serde_json::json!({
            "old_value": old_value,
            "new_value": self.value,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Set the counter to a specific value
    #[payable]
    pub fn set_value(&mut self, new_value: u64) -> Result<u64, String> {
        // Only owner can set the value
        if get_caller() != self.owner {
            return Err("Only owner can set the value".to_string());
        }
        
        if new_value > self.max_value {
            return Err("Value exceeds maximum limit".to_string());
        }
        
        let old_value = self.value;
        self.value = new_value;
        
        emit_event("CounterSet", &serde_json::json!({
            "old_value": old_value,
            "new_value": self.value,
            "caller": get_caller()
        }));
        
        Ok(self.value)
    }
    
    /// Update the maximum value limit
    #[payable]
    pub fn set_max_value(&mut self, new_max: u64) -> Result<u64, String> {
        // Only owner can update the maximum value
        if get_caller() != self.owner {
            return Err("Only owner can update maximum value".to_string());
        }
        
        if new_max < self.value {
            return Err("New maximum cannot be less than current value".to_string());
        }
        
        let old_max = self.max_value;
        self.max_value = new_max;
        
        emit_event("MaxValueUpdated", &serde_json::json!({
            "old_max": old_max,
            "new_max": self.max_value,
            "caller": get_caller()
        }));
        
        Ok(self.max_value)
    }
    
    /// Transfer ownership of the contract
    #[payable]
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<Address, String> {
        // Only current owner can transfer ownership
        if get_caller() != self.owner {
            return Err("Only owner can transfer ownership".to_string());
        }
        
        if new_owner == self.owner {
            return Err("New owner must be different from current owner".to_string());
        }
        
        let old_owner = self.owner;
        self.owner = new_owner;
        
        emit_event("OwnershipTransferred", &serde_json::json!({
            "old_owner": old_owner,
            "new_owner": self.owner,
            "caller": get_caller()
        }));
        
        Ok(self.owner)
    }
    
    /// Get contract statistics
    #[view]
    pub fn get_stats(&self) -> CounterStats {
        CounterStats {
            value: self.value,
            owner: self.owner,
            max_value: self.max_value,
            percentage_used: (self.value as f64 / self.max_value as f64) * 100.0,
        }
    }
}

/// Counter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterStats {
    /// Current counter value
    pub value: u64,
    /// Contract owner
    pub owner: Address,
    /// Maximum value limit
    pub max_value: u64,
    /// Percentage of maximum value used
    pub percentage_used: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_counter_creation() {
        let counter = Counter::new();
        assert_eq!(counter.get_value(), 0);
        assert_eq!(counter.get_max_value(), 1000);
    }
    
    #[test]
    fn test_counter_with_custom_max() {
        let counter = Counter::new_with_max(500);
        assert_eq!(counter.get_value(), 0);
        assert_eq!(counter.get_max_value(), 500);
    }
    
    #[test]
    fn test_increment() {
        let mut counter = Counter::new();
        assert_eq!(counter.increment().unwrap(), 1);
        assert_eq!(counter.get_value(), 1);
    }
    
    #[test]
    fn test_decrement() {
        let mut counter = Counter::new();
        counter.increment().unwrap();
        assert_eq!(counter.decrement().unwrap(), 0);
        assert_eq!(counter.get_value(), 0);
    }
    
    #[test]
    fn test_decrement_below_zero() {
        let mut counter = Counter::new();
        assert!(counter.decrement().is_err());
    }
    
    #[test]
    fn test_increment_by() {
        let mut counter = Counter::new();
        assert_eq!(counter.increment_by(5).unwrap(), 5);
        assert_eq!(counter.get_value(), 5);
    }
    
    #[test]
    fn test_decrement_by() {
        let mut counter = Counter::new();
        counter.increment_by(10).unwrap();
        assert_eq!(counter.decrement_by(3).unwrap(), 7);
        assert_eq!(counter.get_value(), 7);
    }
    
    #[test]
    fn test_reset() {
        let mut counter = Counter::new();
        counter.increment_by(5).unwrap();
        assert_eq!(counter.reset().unwrap(), 0);
        assert_eq!(counter.get_value(), 0);
    }
    
    #[test]
    fn test_set_value() {
        let mut counter = Counter::new();
        assert_eq!(counter.set_value(42).unwrap(), 42);
        assert_eq!(counter.get_value(), 42);
    }
    
    #[test]
    fn test_set_max_value() {
        let mut counter = Counter::new();
        assert_eq!(counter.set_max_value(2000).unwrap(), 2000);
        assert_eq!(counter.get_max_value(), 2000);
    }
    
    #[test]
    fn test_get_stats() {
        let mut counter = Counter::new();
        counter.increment_by(500).unwrap();
        let stats = counter.get_stats();
        assert_eq!(stats.value, 500);
        assert_eq!(stats.max_value, 1000);
        assert_eq!(stats.percentage_used, 50.0);
    }
}
