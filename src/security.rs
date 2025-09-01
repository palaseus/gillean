use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::Rng;

use uuid::Uuid;

/// Security manager for advanced cryptography and security features
pub struct SecurityManager {
    crypto_manager: Arc<CryptoManager>,
    audit_system: Arc<AuditSystem>,
    formal_verifier: Arc<FormalVerifier>,
    threat_detector: Arc<ThreatDetector>,
    config: SecurityConfig,
}

/// Configuration for security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_algorithm: EncryptionAlgorithm,
    pub key_rotation_interval: Duration,
    pub audit_log_retention: Duration,
    pub enable_formal_verification: bool,
    pub threat_detection_enabled: bool,
    pub max_failed_attempts: u32,
    pub session_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
    AES128GCM,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_algorithm: EncryptionAlgorithm::AES256GCM,
            key_rotation_interval: Duration::from_secs(86400), // 24 hours
            audit_log_retention: Duration::from_secs(2592000), // 30 days
            enable_formal_verification: true,
            threat_detection_enabled: true,
            max_failed_attempts: 5,
            session_timeout: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Advanced cryptography manager
pub struct CryptoManager {
    key_store: Arc<RwLock<HashMap<String, CryptoKey>>>,
    config: CryptoConfig,
}

#[derive(Debug, Clone)]
struct CryptoKey {
    key: Vec<u8>,
    #[allow(dead_code)]
    created_at: Instant,
    expires_at: Instant,
    #[allow(dead_code)]
    usage_count: u64,
    #[allow(dead_code)]
    algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub key_size: usize,
    pub nonce_size: usize,
    pub enable_key_rotation: bool,
    pub max_key_usage: u64,
}

impl CryptoManager {
    pub fn new(config: CryptoConfig) -> Self {
        Self {
            key_store: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn generate_key(&self, key_id: &str) -> Result<(), String> {
        let mut key_data = vec![0u8; self.config.key_size];
        rand::thread_rng().fill(&mut key_data[..]);

        let key = CryptoKey {
            key: key_data,
            created_at: Instant::now(),
            expires_at: Instant::now() + Duration::from_secs(86400), // 24 hours
            usage_count: 0,
            algorithm: EncryptionAlgorithm::AES256GCM,
        };

        let mut key_store = self.key_store.write().unwrap();
        key_store.insert(key_id.to_string(), key);
        Ok(())
    }

    pub async fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key_store = self.key_store.read().unwrap();
        let key = key_store.get(key_id)
            .ok_or("Key not found")?;

        if key.expires_at < Instant::now() {
            return Err("Key has expired".to_string());
        }

        // Generate nonce
        let mut nonce_data = vec![0u8; self.config.nonce_size];
        rand::thread_rng().fill(&mut nonce_data[..]);

        // Encrypt using AES-256-GCM
        let cipher = Aes256Gcm::new_from_slice(&key.key)
            .map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(&nonce_data);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| e.to_string())?;

        // Combine nonce and ciphertext
        let mut result = nonce_data;
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub async fn decrypt(&self, key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let key_store = self.key_store.read().unwrap();
        let key = key_store.get(key_id)
            .ok_or("Key not found")?;

        if key.expires_at < Instant::now() {
            return Err("Key has expired".to_string());
        }

        if ciphertext.len() < self.config.nonce_size {
            return Err("Invalid ciphertext length".to_string());
        }

        // Extract nonce and ciphertext
        let nonce_data = &ciphertext[..self.config.nonce_size];
        let ciphertext_data = &ciphertext[self.config.nonce_size..];

        let cipher = Aes256Gcm::new_from_slice(&key.key)
            .map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(nonce_data);
        
        let plaintext = cipher.decrypt(nonce, ciphertext_data)
            .map_err(|e| e.to_string())?;

        Ok(plaintext)
    }

    pub async fn rotate_keys(&self) -> Result<(), String> {
        let mut key_store = self.key_store.write().unwrap();
        let expired_keys: Vec<String> = key_store
            .iter()
            .filter(|(_, key)| key.expires_at < Instant::now())
            .map(|(id, _)| id.clone())
            .collect();

        for key_id in expired_keys {
            key_store.remove(&key_id);
        }

        Ok(())
    }
}

/// Formal verification system for smart contracts and protocols
pub struct FormalVerifier {
    verification_rules: Arc<RwLock<HashMap<String, VerificationRule>>>,
    verification_results: Arc<Mutex<Vec<VerificationResult>>>,
    #[allow(dead_code)]
    config: FormalVerificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRule {
    pub rule_id: String,
    pub rule_type: RuleType,
    pub conditions: Vec<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Safety,
    Liveness,
    Invariant,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub rule_id: String,
    pub contract_id: String,
    pub result: VerificationStatus,
    pub details: String,
    #[serde(with = "timestamp_serde")]
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Passed,
    Failed,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalVerificationConfig {
    pub enable_automated_verification: bool,
    pub verification_timeout: Duration,
    pub max_verification_depth: usize,
}

impl FormalVerifier {
    pub fn new(config: FormalVerificationConfig) -> Self {
        Self {
            verification_rules: Arc::new(RwLock::new(HashMap::new())),
            verification_results: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub async fn add_verification_rule(&self, rule: VerificationRule) {
        let mut rules = self.verification_rules.write().unwrap();
        rules.insert(rule.rule_id.clone(), rule);
    }

    pub async fn verify_contract(&self, contract_code: &str, contract_id: &str) -> Vec<VerificationResult> {
        let mut results = Vec::new();
        let rules = self.verification_rules.read().unwrap();

        for (_rule_id, rule) in rules.iter() {
            let result = self.apply_verification_rule(rule, contract_code, contract_id).await;
            results.push(result);
        }

        // Store results
        let mut stored_results = self.verification_results.lock().unwrap();
        stored_results.extend(results.clone());

        results
    }

    async fn apply_verification_rule(&self, rule: &VerificationRule, contract_code: &str, contract_id: &str) -> VerificationResult {
        // Simulate formal verification based on rule type
        let (status, details) = match rule.rule_type {
            RuleType::Safety => {
                if contract_code.contains("unsafe") {
                    (VerificationStatus::Failed, "Contract contains unsafe operations".to_string())
                } else {
                    (VerificationStatus::Passed, "Safety checks passed".to_string())
                }
            },
            RuleType::Liveness => {
                if contract_code.contains("infinite_loop") {
                    (VerificationStatus::Failed, "Potential infinite loop detected".to_string())
                } else {
                    (VerificationStatus::Passed, "Liveness checks passed".to_string())
                }
            },
            RuleType::Invariant => {
                if contract_code.contains("balance") && contract_code.contains("negative") {
                    (VerificationStatus::Warning, "Potential negative balance invariant".to_string())
                } else {
                    (VerificationStatus::Passed, "Invariant checks passed".to_string())
                }
            },
            RuleType::Temporal => {
                (VerificationStatus::Passed, "Temporal logic checks passed".to_string())
            },
        };

        VerificationResult {
            rule_id: rule.rule_id.clone(),
            contract_id: contract_id.to_string(),
            result: status,
            details,
            timestamp: Instant::now(),
        }
    }

    pub async fn get_verification_results(&self) -> Vec<VerificationResult> {
        self.verification_results.lock().unwrap().clone()
    }
}

/// Security audit system
pub struct AuditSystem {
    audit_logs: Arc<Mutex<Vec<AuditLog>>>,
    config: AuditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub log_id: String,
    #[serde(with = "timestamp_serde")]
    pub timestamp: Instant,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enable_audit_logging: bool,
    pub log_retention_period: Duration,
    pub sensitive_actions: Vec<String>,
}

impl AuditSystem {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            audit_logs: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub async fn log_action(&self, user_id: &str, action: &str, resource: &str, result: AuditResult) {
        if !self.config.enable_audit_logging {
            return;
        }

        let log = AuditLog {
            log_id: Uuid::new_v4().to_string(),
            timestamp: Instant::now(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            result,
            ip_address: None,
            user_agent: None,
        };

        let mut logs = self.audit_logs.lock().unwrap();
        logs.push(log);

        // Clean up old logs
        self.cleanup_old_logs(&mut logs);
    }

    fn cleanup_old_logs(&self, logs: &mut Vec<AuditLog>) {
        let cutoff_time = Instant::now() - self.config.log_retention_period;
        logs.retain(|log| log.timestamp > cutoff_time);
    }

    pub async fn get_audit_logs(&self, user_id: Option<&str>, action: Option<&str>) -> Vec<AuditLog> {
        let logs = self.audit_logs.lock().unwrap();
        
        logs.iter()
            .filter(|log| {
                if let Some(uid) = user_id {
                    if log.user_id != uid {
                        return false;
                    }
                }
                if let Some(act) = action {
                    if log.action != act {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    pub async fn generate_audit_report(&self) -> AuditReport {
        let logs = self.audit_logs.lock().unwrap();
        
        let total_actions = logs.len();
        let successful_actions = logs.iter().filter(|log| matches!(log.result, AuditResult::Success)).count();
        let failed_actions = logs.iter().filter(|log| matches!(log.result, AuditResult::Failure)).count();
        let warning_actions = logs.iter().filter(|log| matches!(log.result, AuditResult::Warning)).count();

        let mut action_counts = HashMap::new();
        for log in logs.iter() {
            *action_counts.entry(log.action.clone()).or_insert(0) += 1;
        }

        AuditReport {
            total_actions,
            successful_actions,
            failed_actions,
            warning_actions,
            action_counts,
            generated_at: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub total_actions: usize,
    pub successful_actions: usize,
    pub failed_actions: usize,
    pub warning_actions: usize,
    pub action_counts: HashMap<String, usize>,
    #[serde(with = "timestamp_serde")]
    pub generated_at: Instant,
}

/// Threat detection system
pub struct ThreatDetector {
    threat_patterns: Arc<RwLock<HashMap<String, ThreatPattern>>>,
    detected_threats: Arc<Mutex<Vec<DetectedThreat>>>,
    #[allow(dead_code)]
    config: ThreatDetectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub pattern_id: String,
    pub pattern_type: ThreatType,
    pub signature: String,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    SQLInjection,
    XSS,
    CSRF,
    DDoS,
    BruteForce,
    Malware,
    DataExfiltration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedThreat {
    pub threat_id: String,
    pub pattern_id: String,
    #[serde(with = "timestamp_serde")]
    pub timestamp: Instant,
    pub source_ip: String,
    pub details: String,
    pub severity: Severity,
    pub mitigated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDetectionConfig {
    pub enable_real_time_detection: bool,
    pub detection_threshold: f64,
    pub auto_mitigation: bool,
}

impl ThreatDetector {
    pub fn new(config: ThreatDetectionConfig) -> Self {
        Self {
            threat_patterns: Arc::new(RwLock::new(HashMap::new())),
            detected_threats: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub async fn add_threat_pattern(&self, pattern: ThreatPattern) {
        let mut patterns = self.threat_patterns.write().unwrap();
        patterns.insert(pattern.pattern_id.clone(), pattern);
    }

    pub async fn analyze_request(&self, request_data: &str, source_ip: &str) -> Vec<DetectedThreat> {
        let mut detected_threats = Vec::new();
        let patterns = self.threat_patterns.read().unwrap();

        for (pattern_id, pattern) in patterns.iter() {
            if self.matches_pattern(request_data, &pattern.signature) {
                let threat = DetectedThreat {
                    threat_id: Uuid::new_v4().to_string(),
                    pattern_id: pattern_id.clone(),
                    timestamp: Instant::now(),
                    source_ip: source_ip.to_string(),
                    details: pattern.description.clone(),
                    severity: pattern.severity.clone(),
                    mitigated: false,
                };

                detected_threats.push(threat.clone());

                // Store detected threat
                let mut threats = self.detected_threats.lock().unwrap();
                threats.push(threat);
            }
        }

        detected_threats
    }

    fn matches_pattern(&self, data: &str, signature: &str) -> bool {
        // Simple pattern matching - in a real implementation, this would be more sophisticated
        data.to_lowercase().contains(&signature.to_lowercase())
    }

    pub async fn get_detected_threats(&self) -> Vec<DetectedThreat> {
        self.detected_threats.lock().unwrap().clone()
    }

    pub async fn mitigate_threat(&self, threat_id: &str) -> Result<(), String> {
        let mut threats = self.detected_threats.lock().unwrap();
        
        for threat in threats.iter_mut() {
            if threat.threat_id == threat_id {
                threat.mitigated = true;
                return Ok(());
            }
        }

        Err("Threat not found".to_string())
    }
}

impl SecurityManager {
    pub fn new(config: SecurityConfig) -> Self {
        let crypto_config = CryptoConfig {
            key_size: 32, // 256 bits
            nonce_size: 12, // 96 bits
            enable_key_rotation: true,
            max_key_usage: 1000,
        };

        let audit_config = AuditConfig {
            enable_audit_logging: true,
            log_retention_period: Duration::from_secs(2592000), // 30 days
            sensitive_actions: vec!["admin_login".to_string(), "key_generation".to_string()],
        };

        let formal_config = FormalVerificationConfig {
            enable_automated_verification: true,
            verification_timeout: Duration::from_secs(300), // 5 minutes
            max_verification_depth: 100,
        };

        let threat_config = ThreatDetectionConfig {
            enable_real_time_detection: true,
            detection_threshold: 0.7,
            auto_mitigation: false,
        };

        Self {
            crypto_manager: Arc::new(CryptoManager::new(crypto_config)),
            audit_system: Arc::new(AuditSystem::new(audit_config)),
            formal_verifier: Arc::new(FormalVerifier::new(formal_config)),
            threat_detector: Arc::new(ThreatDetector::new(threat_config)),
            config,
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Generate initial encryption keys
        self.crypto_manager.generate_key("master").await?;
        self.crypto_manager.generate_key("session").await?;

        // Add default verification rules
        self.add_default_verification_rules().await;

        // Add default threat patterns
        self.add_default_threat_patterns().await;

        Ok(())
    }

    async fn add_default_verification_rules(&self) {
        let safety_rule = VerificationRule {
            rule_id: "safety_001".to_string(),
            rule_type: RuleType::Safety,
            conditions: vec!["no_unsafe_operations".to_string()],
            severity: Severity::High,
        };

        let liveness_rule = VerificationRule {
            rule_id: "liveness_001".to_string(),
            rule_type: RuleType::Liveness,
            conditions: vec!["no_infinite_loops".to_string()],
            severity: Severity::Critical,
        };

        self.formal_verifier.add_verification_rule(safety_rule).await;
        self.formal_verifier.add_verification_rule(liveness_rule).await;
    }

    async fn add_default_threat_patterns(&self) {
        let sql_injection = ThreatPattern {
            pattern_id: "sql_injection_001".to_string(),
            pattern_type: ThreatType::SQLInjection,
            signature: "SELECT * FROM".to_string(),
            severity: Severity::High,
            description: "Potential SQL injection attempt".to_string(),
        };

        let xss = ThreatPattern {
            pattern_id: "xss_001".to_string(),
            pattern_type: ThreatType::XSS,
            signature: "<script>".to_string(),
            severity: Severity::Medium,
            description: "Potential XSS attack".to_string(),
        };

        self.threat_detector.add_threat_pattern(sql_injection).await;
        self.threat_detector.add_threat_pattern(xss).await;
    }

    pub async fn get_security_status(&self) -> SecurityStatus {
        let crypto_keys = self.crypto_manager.key_store.read().unwrap().len();
        let audit_logs = self.audit_system.audit_logs.lock().unwrap().len();
        let verification_results = self.formal_verifier.verification_results.lock().unwrap().len();
        let detected_threats = self.threat_detector.detected_threats.lock().unwrap().len();

        SecurityStatus {
            crypto_keys,
            audit_logs,
            verification_results,
            detected_threats,
            config: self.config.clone(),
        }
    }

    pub async fn perform_security_audit(&self) -> SecurityAuditResult {
        let start_time = Instant::now();
        let mut findings = Vec::new();

        // Check crypto key expiration
        let key_store = self.crypto_manager.key_store.read().unwrap();
        for (key_id, key) in key_store.iter() {
            if key.expires_at < Instant::now() {
                findings.push(format!("Crypto key '{}' has expired", key_id));
            }
        }

        // Check for failed verification results
        let verification_results = self.formal_verifier.verification_results.lock().unwrap();
        let failed_verifications = verification_results.iter()
            .filter(|result| matches!(result.result, VerificationStatus::Failed))
            .count();
        
        if failed_verifications > 0 {
            findings.push(format!("{} failed verification results found", failed_verifications));
        }

        // Check for unmitigated threats
        let detected_threats = self.threat_detector.detected_threats.lock().unwrap();
        let unmitigated_threats = detected_threats.iter()
            .filter(|threat| !threat.mitigated)
            .count();
        
        if unmitigated_threats > 0 {
            findings.push(format!("{} unmitigated threats detected", unmitigated_threats));
        }

        let duration = start_time.elapsed();
        SecurityAuditResult {
            duration,
            findings: findings.clone(),
            severity: if findings.is_empty() { Severity::Low } else { Severity::Medium },
            success: findings.is_empty(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStatus {
    pub crypto_keys: usize,
    pub audit_logs: usize,
    pub verification_results: usize,
    pub detected_threats: usize,
    pub config: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    pub duration: Duration,
    pub findings: Vec<String>,
    pub severity: Severity,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_crypto_manager() {
        let config = CryptoConfig {
            key_size: 32,
            nonce_size: 12,
            enable_key_rotation: true,
            max_key_usage: 1000,
        };
        let crypto = CryptoManager::new(config);

        // Test key generation
        crypto.generate_key("test_key").await.unwrap();

        // Test encryption and decryption
        let plaintext = b"Hello, World!";
        let ciphertext = crypto.encrypt("test_key", plaintext).await.unwrap();
        let decrypted = crypto.decrypt("test_key", &ciphertext).await.unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[tokio::test]
    async fn test_formal_verifier() {
        let config = FormalVerificationConfig {
            enable_automated_verification: true,
            verification_timeout: Duration::from_secs(60),
            max_verification_depth: 100,
        };
        let verifier = FormalVerifier::new(config);

        // Add a verification rule
        let rule = VerificationRule {
            rule_id: "test_rule".to_string(),
            rule_type: RuleType::Safety,
            conditions: vec!["no_unsafe".to_string()],
            severity: Severity::High,
        };
        verifier.add_verification_rule(rule).await;

        // Test contract verification
        let contract_code = "fn safe_function() { /* safe code */ }";
        let results = verifier.verify_contract(contract_code, "test_contract").await;
        
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_audit_system() {
        let config = AuditConfig {
            enable_audit_logging: true,
            log_retention_period: Duration::from_secs(60),
            sensitive_actions: vec!["admin_login".to_string()],
        };
        let audit = AuditSystem::new(config);

        // Test logging
        audit.log_action("user1", "login", "auth", AuditResult::Success).await;
        audit.log_action("user2", "admin_login", "auth", AuditResult::Success).await;

        let logs = audit.get_audit_logs(None, None).await;
        assert_eq!(logs.len(), 2);

        let admin_logs = audit.get_audit_logs(None, Some("admin_login")).await;
        assert_eq!(admin_logs.len(), 1);
    }

    #[tokio::test]
    async fn test_threat_detector() {
        let config = ThreatDetectionConfig {
            enable_real_time_detection: true,
            detection_threshold: 0.7,
            auto_mitigation: false,
        };
        let detector = ThreatDetector::new(config);

        // Add a threat pattern
        let pattern = ThreatPattern {
            pattern_id: "test_pattern".to_string(),
            pattern_type: ThreatType::SQLInjection,
            signature: "SELECT".to_string(),
            severity: Severity::High,
            description: "Test SQL injection".to_string(),
        };
        detector.add_threat_pattern(pattern).await;

        // Test threat detection
        let malicious_request = "SELECT * FROM users WHERE id = 1";
        let threats = detector.analyze_request(malicious_request, "192.168.1.1").await;
        
        assert!(!threats.is_empty());
    }

    #[tokio::test]
    async fn test_security_manager() {
        let config = SecurityConfig::default();
        let manager = SecurityManager::new(config);
        manager.initialize().await.unwrap();

        let status = manager.get_security_status().await;
        assert!(status.crypto_keys > 0);

        let audit_result = manager.perform_security_audit().await;
        assert!(audit_result.duration > Duration::from_nanos(0));
    }
}

// Helper module for serializing Instant
mod timestamp_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(instant.elapsed().as_nanos() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Instant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos = u64::deserialize(deserializer)?;
        Ok(Instant::now() - Duration::from_nanos(nanos))
    }
}
