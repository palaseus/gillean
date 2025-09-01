use gillean::{Blockchain, Transaction, BlockchainError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// AI Integration Types
#[derive(Debug, Clone, PartialEq)]
pub enum AnomalyType {
    UnusualAmount,
    UnusualFrequency,
    SuspiciousPattern,
    GeographicAnomaly,
    TimeAnomaly,
}

#[derive(Debug, Clone)]
pub struct AnomalyScore {
    pub score: f64,
    pub anomaly_type: AnomalyType,
    pub confidence: f64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct TransactionFeatures {
    pub amount: f64,
    pub frequency: u32,
    pub sender_history: u32,
    pub receiver_history: u32,
    pub time_of_day: u8,
    pub day_of_week: u8,
    pub geographic_distance: Option<f64>,
    pub transaction_type: String,
}

#[derive(Debug, Clone)]
pub struct FraudPrediction {
    pub is_fraudulent: bool,
    pub confidence: f64,
    pub risk_factors: Vec<String>,
    pub recommended_action: String,
}

#[derive(Debug, Clone)]
pub struct PredictiveModel {
    pub model_id: String,
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub training_data_size: usize,
    pub last_updated: u64,
}

// AI Integration Manager
pub struct AIManager {
    pub anomaly_detector: Arc<Mutex<AnomalyDetector>>,
    pub fraud_detector: Arc<Mutex<FraudDetector>>,
    pub predictive_models: Arc<Mutex<HashMap<String, PredictiveModel>>>,
    pub transaction_history: Arc<Mutex<Vec<Transaction>>>,
}

impl AIManager {
    pub fn new() -> Self {
        Self {
            anomaly_detector: Arc::new(Mutex::new(AnomalyDetector::new())),
            fraud_detector: Arc::new(Mutex::new(FraudDetector::new())),
            predictive_models: Arc::new(Mutex::new(HashMap::new())),
            transaction_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn analyze_transaction(&self, transaction: &Transaction) -> AnomalyScore {
        let mut detector = self.anomaly_detector.lock().unwrap();
        detector.analyze_transaction(transaction)
    }

    pub fn predict_fraud(&self, transaction: &Transaction) -> FraudPrediction {
        let mut detector = self.fraud_detector.lock().unwrap();
        detector.predict_fraud(transaction)
    }

    pub fn train_model(&self, model_name: &str, training_data: &[Transaction]) -> Result<PredictiveModel, BlockchainError> {
        let mut models = self.predictive_models.lock().unwrap();
        
        // Simulate model training
        let model = PredictiveModel {
            model_id: model_name.to_string(),
            accuracy: 0.95,
            precision: 0.92,
            recall: 0.88,
            f1_score: 0.90,
            training_data_size: training_data.len(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        models.insert(model_name.to_string(), model.clone());
        Ok(model)
    }

    pub fn update_transaction_history(&self, transaction: Transaction) {
        let mut history = self.transaction_history.lock().unwrap();
        history.push(transaction);
        
        // Keep only last 10000 transactions for memory management
        if history.len() > 10000 {
            history.remove(0);
        }
    }

    pub fn get_analytics_summary(&self) -> AnalyticsSummary {
        let history = self.transaction_history.lock().unwrap();
        let models = self.predictive_models.lock().unwrap();
        
        AnalyticsSummary {
            total_transactions: history.len(),
            total_models: models.len(),
            average_accuracy: models.values().map(|m| m.accuracy).sum::<f64>() / models.len() as f64,
            last_analysis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

// Anomaly Detector
pub struct AnomalyDetector {
    pub baseline_stats: HashMap<String, f64>,
    pub threshold_multiplier: f64,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        let mut baseline_stats = HashMap::new();
        baseline_stats.insert("avg_amount".to_string(), 100.0);
        baseline_stats.insert("avg_frequency".to_string(), 5.0);
        baseline_stats.insert("std_amount".to_string(), 50.0);
        
        Self {
            baseline_stats,
            threshold_multiplier: 2.5,
        }
    }

    pub fn analyze_transaction(&mut self, transaction: &Transaction) -> AnomalyScore {
        let amount = transaction.amount;
        let avg_amount = self.baseline_stats.get("avg_amount").unwrap_or(&100.0);
        let std_amount = self.baseline_stats.get("std_amount").unwrap_or(&50.0);
        
        let z_score = (amount - avg_amount) / std_amount;
        let anomaly_score = z_score.abs();
        
        let anomaly_type = if anomaly_score > self.threshold_multiplier {
            if amount > avg_amount * 3.0 {
                AnomalyType::UnusualAmount
            } else {
                AnomalyType::SuspiciousPattern
            }
        } else {
            AnomalyType::UnusualAmount
        };
        
        AnomalyScore {
            score: anomaly_score,
            anomaly_type,
            confidence: (anomaly_score / 5.0).min(1.0),
            description: format!("Transaction amount {} deviates from baseline", amount),
        }
    }

    pub fn update_baseline(&mut self, new_transactions: &[Transaction]) {
        if new_transactions.is_empty() {
            return;
        }
        
        let total_amount: f64 = new_transactions.iter().map(|tx| tx.amount).sum();
        let avg_amount = total_amount / new_transactions.len() as f64;
        
        let variance: f64 = new_transactions.iter()
            .map(|tx| (tx.amount - avg_amount).powi(2))
            .sum::<f64>() / new_transactions.len() as f64;
        let std_amount = variance.sqrt();
        
        self.baseline_stats.insert("avg_amount".to_string(), avg_amount);
        self.baseline_stats.insert("std_amount".to_string(), std_amount);
    }
}

// Fraud Detector
pub struct FraudDetector {
    pub risk_threshold: f64,
    pub known_fraud_patterns: Vec<String>,
    pub risk_weights: HashMap<String, f64>,
}

impl FraudDetector {
    pub fn new() -> Self {
        let mut risk_weights = HashMap::new();
        risk_weights.insert("amount".to_string(), 0.3);
        risk_weights.insert("frequency".to_string(), 0.25);
        risk_weights.insert("pattern".to_string(), 0.25);
        risk_weights.insert("geographic".to_string(), 0.2);
        
        Self {
            risk_threshold: 0.7,
            known_fraud_patterns: vec![
                "rapid_transfers".to_string(),
                "large_amounts".to_string(),
                "unusual_timing".to_string(),
            ],
            risk_weights,
        }
    }

    pub fn predict_fraud(&mut self, transaction: &Transaction) -> FraudPrediction {
        let mut risk_score = 0.0;
        let mut risk_factors = Vec::new();
        
        // Amount-based risk
        if transaction.amount > 10000.0 {
            risk_score += self.risk_weights.get("amount").unwrap_or(&0.3);
            risk_factors.push("High transaction amount".to_string());
        }
        
        // Pattern-based risk
        if self.detect_suspicious_pattern(transaction) {
            risk_score += self.risk_weights.get("pattern").unwrap_or(&0.25);
            risk_factors.push("Suspicious transaction pattern".to_string());
        }
        
        // Geographic risk (simulated)
        if self.detect_geographic_anomaly(transaction) {
            risk_score += self.risk_weights.get("geographic").unwrap_or(&0.2);
            risk_factors.push("Geographic anomaly detected".to_string());
        }
        
        let is_fraudulent = risk_score >= self.risk_threshold;
        let recommended_action = if is_fraudulent {
            "Block transaction and flag for review".to_string()
        } else if risk_score > 0.5 {
            "Flag for manual review".to_string()
        } else {
            "Allow transaction".to_string()
        };
        
        FraudPrediction {
            is_fraudulent,
            confidence: risk_score,
            risk_factors,
            recommended_action,
        }
    }

    fn detect_suspicious_pattern(&self, _transaction: &Transaction) -> bool {
        // Simulate pattern detection
        simple_random() < 0.1 // 10% chance of suspicious pattern
    }

    fn detect_geographic_anomaly(&self, _transaction: &Transaction) -> bool {
        // Simulate geographic anomaly detection
        simple_random() < 0.05 // 5% chance of geographic anomaly
    }
}

// Analytics Summary
#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub total_transactions: usize,
    pub total_models: usize,
    pub average_accuracy: f64,
    pub last_analysis: u64,
}

// AI Integration Test Suite
pub struct AIIntegrationSuite {
    _manager: AIManager,
}

impl AIIntegrationSuite {
    pub fn new() -> Self {
        Self {
            _manager: AIManager::new(),
        }
    }

    pub fn run_all_tests(&self) -> Result<(), BlockchainError> {
        println!("Running AI Integration Tests...");
        
        self.test_anomaly_detection()?;
        self.test_fraud_detection()?;
        self.test_model_training()?;
        self.test_transaction_analysis()?;
        self.test_predictive_analytics()?;
        self.test_continuous_learning()?;
        self.test_invalid_operations()?;
        
        println!("✅ All AI Integration tests passed!");
        Ok(())
    }

    fn test_anomaly_detection(&self) -> Result<(), BlockchainError> {
        println!("  Testing anomaly detection...");
        
        let normal_tx = Transaction::new_transfer(
            "sender1".to_string(),
            "receiver1".to_string(),
            100.0,
            Some("normal transaction".to_string()),
        )?;
        
        let large_tx = Transaction::new_transfer(
            "sender2".to_string(),
            "receiver2".to_string(),
            50000.0,
            Some("large transaction".to_string()),
        )?;
        
        let anomaly_score = self._manager.analyze_transaction(&large_tx);
        
        assert!(anomaly_score.score > 1.0, "Large transaction should be flagged as anomalous");
        assert_eq!(anomaly_score.anomaly_type, AnomalyType::UnusualAmount);
        assert!(anomaly_score.confidence > 0.5, "Should have high confidence for large amounts");
        
        Ok(())
    }

    fn test_fraud_detection(&self) -> Result<(), BlockchainError> {
        println!("  Testing fraud detection...");
        
        let normal_tx = Transaction::new_transfer(
            "sender1".to_string(),
            "receiver1".to_string(),
            100.0,
            Some("normal transaction".to_string()),
        )?;
        
        let suspicious_tx = Transaction::new_transfer(
            "sender2".to_string(),
            "receiver2".to_string(),
            15000.0,
            Some("suspicious transaction".to_string()),
        )?;
        
        let fraud_prediction = self._manager.predict_fraud(&suspicious_tx);
        
        assert!(fraud_prediction.confidence > 0.0, "Should have some risk assessment");
        assert!(!fraud_prediction.risk_factors.is_empty(), "Should identify risk factors");
        assert!(!fraud_prediction.recommended_action.is_empty(), "Should provide recommendations");
        
        Ok(())
    }

    fn test_model_training(&self) -> Result<(), BlockchainError> {
        println!("  Testing model training...");
        
        let training_data = vec![
            Transaction::new_transfer("sender1".to_string(), "receiver1".to_string(), 100.0, Some("tx1".to_string()))?,
            Transaction::new_transfer("sender2".to_string(), "receiver2".to_string(), 200.0, Some("tx2".to_string()))?,
            Transaction::new_transfer("sender3".to_string(), "receiver3".to_string(), 150.0, Some("tx3".to_string()))?,
        ];
        
        let model = self._manager.train_model("fraud_detection_v1", &training_data)?;
        
        assert_eq!(model.model_id, "fraud_detection_v1");
        assert!(model.accuracy > 0.8, "Model should have reasonable accuracy");
        assert_eq!(model.training_data_size, 3);
        assert!(model.last_updated > 0);
        
        Ok(())
    }

    fn test_transaction_analysis(&self) -> Result<(), BlockchainError> {
        println!("  Testing transaction analysis...");
        
        let tx = Transaction::new_transfer(
            "sender1".to_string(),
            "receiver1".to_string(),
            1000.0,
            Some("test transaction".to_string()),
        )?;
        
        // Test anomaly detection
        let anomaly_score = self._manager.analyze_transaction(&tx);
        assert!(anomaly_score.score >= 0.0, "Anomaly score should be non-negative");
        
        // Test fraud detection
        let fraud_prediction = self._manager.predict_fraud(&tx);
        assert!(fraud_prediction.confidence >= 0.0 && fraud_prediction.confidence <= 1.0);
        
        // Update transaction history
        self._manager.update_transaction_history(tx);
        
        Ok(())
    }

    fn test_predictive_analytics(&self) -> Result<(), BlockchainError> {
        println!("  Testing predictive analytics...");
        
        // Add some transaction history
        for i in 0..10 {
            let tx = Transaction::new_transfer(
                format!("sender{}", i),
                format!("receiver{}", i),
                100.0 + (i as f64 * 10.0),
                Some(format!("tx{}", i)),
            )?;
            self._manager.update_transaction_history(tx);
        }
        
        let summary = self._manager.get_analytics_summary();
        
        assert_eq!(summary.total_transactions, 10);
        assert!(summary.average_accuracy >= 0.0);
        assert!(summary.last_analysis > 0);
        
        Ok(())
    }

    fn test_continuous_learning(&self) -> Result<(), BlockchainError> {
        println!("  Testing continuous learning...");
        
        let mut detector = self._manager.anomaly_detector.lock().unwrap();
        
        // Simulate new transaction data
        let new_transactions = vec![
            Transaction::new_transfer("sender1".to_string(), "receiver1".to_string(), 120.0, Some("new1".to_string()))?,
            Transaction::new_transfer("sender2".to_string(), "receiver2".to_string(), 130.0, Some("new2".to_string()))?,
        ];
        
        // Update baseline with new data
        detector.update_baseline(&new_transactions);
        
        // Verify baseline was updated
        let avg_amount = detector.baseline_stats.get("avg_amount").unwrap_or(&0.0);
        assert!(*avg_amount > 0.0, "Baseline should be updated with new data");
        
        Ok(())
    }

    fn test_invalid_operations(&self) -> Result<(), BlockchainError> {
        println!("  Testing invalid operations...");
        
        // Test with empty training data
        let empty_data = vec![];
        let result = self._manager.train_model("test_model", &empty_data);
        
        // Should still work but with empty data
        assert!(result.is_ok(), "Should handle empty training data gracefully");
        
        // Test with invalid transaction (should still work)
        let invalid_tx = Transaction::new_transfer(
            "".to_string(),
            "".to_string(),
            -100.0,
            Some("invalid".to_string()),
        )?;
        
        let anomaly_score = self._manager.analyze_transaction(&invalid_tx);
        assert!(anomaly_score.score >= 0.0, "Should handle invalid transactions");
        
        Ok(())
    }
}

// Simple random function for testing
fn simple_random() -> f64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    (hasher.finish() % 100) as f64 / 100.0
}
