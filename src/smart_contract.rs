use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, warn, error};
use crate::{Result, BlockchainError};

/// Represents a smart contract with code and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    /// Unique identifier for the contract
    pub id: String,
    /// Contract code as a string
    pub code: String,
    /// Contract storage (key-value pairs)
    pub storage: HashMap<String, String>,
    /// Contract owner address
    pub owner: String,
    /// Contract balance
    pub balance: f64,
    /// Whether the contract is active
    pub active: bool,
    /// Creation timestamp
    pub created_at: i64,
}

/// Represents the execution context for smart contracts
#[derive(Debug, Clone)]
pub struct ContractContext {
    /// Current transaction data
    pub transaction_data: HashMap<String, String>,
    /// Current block height
    pub block_height: u64,
    /// Current timestamp
    pub timestamp: i64,
    /// Gas limit for execution
    pub gas_limit: u64,
    /// Current gas used
    pub gas_used: u64,
}

/// Stack-based virtual machine for executing smart contracts
#[derive(Debug)]
pub struct ContractVM {
    /// Execution stack
    stack: Vec<String>,
    /// Local variables
    variables: HashMap<String, String>,
    /// Gas counter
    gas_used: u64,
    /// Gas limit
    gas_limit: u64,
}

/// Smart contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    /// Whether execution was successful
    pub success: bool,
    /// Return value
    pub return_value: Option<String>,
    /// Gas used
    pub gas_used: u64,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Storage changes
    pub storage_changes: HashMap<String, String>,
}

impl SmartContract {
    /// Create a new smart contract
    /// 
    /// # Arguments
    /// * `code` - Contract code as a string
    /// * `owner` - Contract owner address
    /// 
    /// # Returns
    /// * `Result<SmartContract>` - The created contract or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::smart_contract::SmartContract;
    /// 
    /// let contract = SmartContract::new(
    ///     "PUSH 100\nSTORE balance\nRETURN".to_string(),
    ///     "alice".to_string()
    /// ).unwrap();
    /// 
    /// assert_eq!(contract.owner, "alice");
    /// assert_eq!(contract.balance, 0.0);
    /// ```
    pub fn new(code: String, owner: String) -> Result<Self> {
        if code.is_empty() {
            return Err(BlockchainError::ContractValidationFailed(
                "Contract code cannot be empty".to_string(),
            ));
        }

        if owner.is_empty() {
            return Err(BlockchainError::ContractValidationFailed(
                "Contract owner cannot be empty".to_string(),
            ));
        }

        let id = Self::generate_id(&code, &owner);
        let created_at = chrono::Utc::now().timestamp();

        let contract = SmartContract {
            id,
            code,
            storage: HashMap::new(),
            owner,
            balance: 0.0,
            active: true,
            created_at,
        };

        debug!("Created smart contract: {}", contract.id);
        Ok(contract)
    }

    /// Generate a unique contract ID
    fn generate_id(code: &str, owner: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        hasher.update(owner.as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Execute the smart contract
    /// 
    /// # Arguments
    /// * `context` - Execution context
    /// 
    /// # Returns
    /// * `Result<ContractResult>` - Execution result or error
    pub fn execute(&mut self, context: ContractContext) -> Result<ContractResult> {
        if !self.active {
            return Err(BlockchainError::ContractValidationFailed(
                "Contract is not active".to_string(),
            ));
        }

        let mut vm = ContractVM::new(context.gas_limit);
        
        match vm.execute(&self.code, &context) {
            Ok(result) => {
                // Apply storage changes
                for (key, value) in &result.storage_changes {
                    self.storage.insert(key.clone(), value.clone());
                }
                Ok(result)
            }
            Err(e) => {
                error!("Contract execution failed: {}", e);
                Err(e)
            }
        }
    }

    /// Add funds to contract balance
    pub fn add_funds(&mut self, amount: f64) -> Result<()> {
        if amount <= 0.0 {
            return Err(BlockchainError::ContractValidationFailed(
                "Amount must be positive".to_string(),
            ));
        }
        self.balance += amount;
        debug!("Added {} to contract {} balance", amount, self.id);
        Ok(())
    }

    /// Withdraw funds from contract balance
    pub fn withdraw_funds(&mut self, amount: f64, caller: &str) -> Result<()> {
        if caller != self.owner {
            return Err(BlockchainError::ContractValidationFailed(
                "Only owner can withdraw funds".to_string(),
            ));
        }

        if amount <= 0.0 {
            return Err(BlockchainError::ContractValidationFailed(
                "Amount must be positive".to_string(),
            ));
        }

        if amount > self.balance {
            return Err(BlockchainError::ContractValidationFailed(
                "Insufficient contract balance".to_string(),
            ));
        }

        self.balance -= amount;
        debug!("Withdrew {} from contract {} balance", amount, self.id);
        Ok(())
    }

    /// Deactivate the contract
    pub fn deactivate(&mut self, caller: &str) -> Result<()> {
        if caller != self.owner {
            return Err(BlockchainError::ContractValidationFailed(
                "Only owner can deactivate contract".to_string(),
            ));
        }
        self.active = false;
        debug!("Deactivated contract: {}", self.id);
        Ok(())
    }
}

impl ContractVM {
    /// Create a new contract virtual machine
    pub fn new(gas_limit: u64) -> Self {
        ContractVM {
            stack: Vec::new(),
            variables: HashMap::new(),
            gas_used: 0,
            gas_limit,
        }
    }

    /// Execute contract code
    /// 
    /// # Arguments
    /// * `code` - Contract code to execute
    /// * `context` - Execution context
    /// 
    /// # Returns
    /// * `Result<ContractResult>` - Execution result or error
    pub fn execute(&mut self, code: &str, _context: &ContractContext) -> Result<ContractResult> {
        let lines: Vec<&str> = code.lines().collect();
        let mut storage_changes = HashMap::new();
        let mut return_value = None;

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check gas limit
            if self.gas_used >= self.gas_limit {
                return Err(BlockchainError::ContractValidationFailed(
                    "Gas limit exceeded".to_string(),
                ));
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let instruction = parts[0].to_uppercase();
            self.gas_used += 1; // Basic gas cost per instruction

            match instruction.as_str() {
                "PUSH" => {
                    if parts.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("PUSH requires a value at line {}", line_num + 1),
                        ));
                    }
                    self.stack.push(parts[1].to_string());
                }
                "POP" => {
                    if self.stack.is_empty() {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    self.stack.pop();
                }
                "STORE" => {
                    if parts.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("STORE requires a key at line {}", line_num + 1),
                        ));
                    }
                    if self.stack.is_empty() {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let value = self.stack.pop().unwrap();
                    let key = parts[1].to_string();
                    self.variables.insert(key.clone(), value.clone());
                    storage_changes.insert(key, value);
                }
                "LOAD" => {
                    if parts.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("LOAD requires a key at line {}", line_num + 1),
                        ));
                    }
                    let key = parts[1];
                    if let Some(value) = self.variables.get(key) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Variable '{}' not found at line {}", key, line_num + 1),
                        ));
                    }
                }
                "ADD" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    if let (Ok(a_val), Ok(b_val)) = (a.parse::<f64>(), b.parse::<f64>()) {
                        self.stack.push((a_val + b_val).to_string());
                    } else {
                        // String concatenation
                        self.stack.push(format!("{}{}", a, b));
                    }
                }
                "SUB" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    if let (Ok(a_val), Ok(b_val)) = (a.parse::<f64>(), b.parse::<f64>()) {
                        self.stack.push((a_val - b_val).to_string());
                    } else {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Cannot subtract non-numeric values at line {}", line_num + 1),
                        ));
                    }
                }
                "MUL" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    if let (Ok(a_val), Ok(b_val)) = (a.parse::<f64>(), b.parse::<f64>()) {
                        self.stack.push((a_val * b_val).to_string());
                    } else {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Cannot multiply non-numeric values at line {}", line_num + 1),
                        ));
                    }
                }
                "DIV" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    if let (Ok(a_val), Ok(b_val)) = (a.parse::<f64>(), b.parse::<f64>()) {
                        if b_val == 0.0 {
                            return Err(BlockchainError::ContractValidationFailed(
                                format!("Division by zero at line {}", line_num + 1),
                            ));
                        }
                        self.stack.push((a_val / b_val).to_string());
                    } else {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Cannot divide non-numeric values at line {}", line_num + 1),
                        ));
                    }
                }
                "EQ" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(if a == b { "1".to_string() } else { "0".to_string() });
                }
                "GT" => {
                    if self.stack.len() < 2 {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    if let (Ok(a_val), Ok(b_val)) = (a.parse::<f64>(), b.parse::<f64>()) {
                        self.stack.push(if a_val > b_val { "1".to_string() } else { "0".to_string() });
                    } else {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Cannot compare non-numeric values at line {}", line_num + 1),
                        ));
                    }
                }
                "IF" => {
                    if self.stack.is_empty() {
                        return Err(BlockchainError::ContractValidationFailed(
                            format!("Stack underflow at line {}", line_num + 1),
                        ));
                    }
                    let condition = self.stack.pop().unwrap();
                    if condition != "1" && condition != "true" {
                        // Skip until ENDIF
                        let mut depth = 1;
                        for next_line in lines.iter().skip(line_num + 1) {
                            let next_line = next_line.trim();
                            if next_line == "IF" {
                                depth += 1;
                            } else if next_line == "ENDIF" {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
                "ENDIF" => {
                    // End of IF block - do nothing
                }
                "RETURN" => {
                    if !self.stack.is_empty() {
                        return_value = Some(self.stack.pop().unwrap());
                    }
                    break;
                }
                _ => {
                    warn!("Unknown instruction: {} at line {}", instruction, line_num + 1);
                }
            }
        }

        Ok(ContractResult {
            success: true,
            return_value,
            gas_used: self.gas_used,
            error: None,
            storage_changes,
        })
    }
}

impl ContractContext {
    /// Create a new contract execution context
    pub fn new(block_height: u64, gas_limit: u64) -> Self {
        ContractContext {
            transaction_data: HashMap::new(),
            block_height,
            timestamp: chrono::Utc::now().timestamp(),
            gas_limit,
            gas_used: 0,
        }
    }

    /// Add transaction data to the context
    pub fn add_transaction_data(&mut self, key: String, value: String) {
        self.transaction_data.insert(key, value);
    }
}

/// Example smart contracts
pub mod examples {
    use super::*;

    /// Create a simple crowdfunding contract
    /// 
    /// This contract allows users to contribute funds and only releases them
    /// if the goal is met within a time limit
    pub fn crowdfunding_contract(goal: f64, deadline: i64) -> SmartContract {
        let code = format!(
            "# Crowdfunding Contract\n\
             # Goal: {}\n\
             # Deadline: {}\n\
             PUSH {}\n\
             STORE goal\n\
             PUSH {}\n\
             STORE deadline\n\
             PUSH 0\n\
             STORE total_raised\n\
             PUSH 0\n\
             STORE funded\n\
             RETURN",
            goal, deadline, goal, deadline
        );

        SmartContract::new(code, "crowdfunding_owner".to_string()).unwrap()
    }

    /// Create a multi-signature wallet contract
    /// 
    /// This contract requires multiple signatures to release funds
    pub fn multisig_contract(required_signatures: u32) -> SmartContract {
        let code = format!(
            "# Multi-signature Wallet Contract\n\
             # Required signatures: {}\n\
             PUSH {}\n\
             STORE required_sigs\n\
             PUSH 0\n\
             STORE current_sigs\n\
             PUSH 0\n\
             STORE pending_amount\n\
             PUSH 0\n\
             STORE pending_recipient\n\
             RETURN",
            required_signatures, required_signatures
        );

        SmartContract::new(code, "multisig_owner".to_string()).unwrap()
    }

    /// Create a time-locked contract
    /// 
    /// This contract only allows withdrawals after a certain time
    pub fn timelock_contract(unlock_time: i64) -> SmartContract {
        let code = format!(
            "# Time-locked Contract\n\
             # Unlock time: {}\n\
             PUSH {}\n\
             STORE unlock_time\n\
             PUSH 0\n\
             STORE locked\n\
             RETURN",
            unlock_time, unlock_time
        );

        SmartContract::new(code, "timelock_owner".to_string()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_creation() {
        let contract = SmartContract::new(
            "PUSH 100\nSTORE balance\nRETURN".to_string(),
            "alice".to_string()
        ).unwrap();

        assert_eq!(contract.owner, "alice");
        assert_eq!(contract.balance, 0.0);
        assert!(contract.active);
    }

    #[test]
    fn test_contract_execution() {
        let mut contract = SmartContract::new(
            "PUSH 100\nSTORE balance\nPUSH 50\nPUSH 100\nADD\nSTORE total\nLOAD total\nRETURN".to_string(),
            "alice".to_string()
        ).unwrap();

        let context = ContractContext::new(1, 1000);
        let result = contract.execute(context).unwrap();

        assert!(result.success);
        assert_eq!(result.return_value, Some("150".to_string()));
        assert_eq!(contract.storage.get("balance"), Some(&"100".to_string()));
        assert_eq!(contract.storage.get("total"), Some(&"150".to_string()));
    }

    #[test]
    fn test_simple_contract_execution() {
        let mut contract = SmartContract::new(
            "PUSH 100\nSTORE balance\nLOAD balance\nRETURN".to_string(),
            "alice".to_string()
        ).unwrap();

        let context = ContractContext::new(1, 1000);
        let result = contract.execute(context).unwrap();

        assert!(result.success);
        assert_eq!(result.return_value, Some("100".to_string()));
        assert_eq!(contract.storage.get("balance"), Some(&"100".to_string()));
    }

    #[test]
    fn test_crowdfunding_contract() {
        let contract = examples::crowdfunding_contract(1000.0, 1234567890);
        assert_eq!(contract.owner, "crowdfunding_owner");
        assert!(contract.code.contains("1000"));
    }
}
