// Decentralized Identity (DID) Test Suite
// Tests for self-sovereign identity system

use gillean::{Result, Blockchain, BlockchainError};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    pub id: String,
    pub controller: String,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub service_endpoints: Vec<ServiceEndpoint>,
    pub created: u64,
    pub updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    pub key_type: String,
    pub public_key: String,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub service_type: String,
    pub service_endpoint: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    pub id: String,
    pub issuer: String,
    pub subject: String,
    pub credential_type: Vec<String>,
    pub claims: HashMap<String, serde_json::Value>,
    pub issuance_date: u64,
    pub expiration_date: Option<u64>,
    pub proof: Option<CredentialProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialProof {
    pub proof_type: String,
    pub created: u64,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    pub id: String,
    pub holder: String,
    pub verifiable_credentials: Vec<VerifiableCredential>,
    pub proof: Option<PresentationProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationProof {
    pub proof_type: String,
    pub created: u64,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: String,
}

#[derive(Debug, Clone)]
pub struct IdentityRecovery {
    pub id: String,
    pub did: String,
    pub recovery_methods: Vec<RecoveryMethod>,
    pub guardians: Vec<String>,
    pub threshold: u32,
    pub created: u64,
}

#[derive(Debug, Clone)]
pub struct RecoveryMethod {
    pub id: String,
    pub method_type: RecoveryMethodType,
    pub value: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub enum RecoveryMethodType {
    Email,
    Phone,
    SocialAccount,
    HardwareKey,
    Guardian,
}

#[derive(Debug, Clone)]
pub struct DIDManager {
    pub did_documents: HashMap<String, DIDDocument>,
    pub credentials: HashMap<String, VerifiableCredential>,
    pub presentations: HashMap<String, VerifiablePresentation>,
    pub recovery_plans: HashMap<String, IdentityRecovery>,
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl DIDManager {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            did_documents: HashMap::new(),
            credentials: HashMap::new(),
            presentations: HashMap::new(),
            recovery_plans: HashMap::new(),
            blockchain,
        }
    }

    pub fn create_did(&mut self, controller: String) -> Result<String> {
        let did = format!("did:gillean:{}", uuid::Uuid::new_v4());
        
        let verification_method = VerificationMethod {
            id: format!("{}#keys-1", did),
            controller: did.clone(),
            key_type: "Ed25519VerificationKey2020".to_string(),
            public_key: format!("public_key_{}", uuid::Uuid::new_v4()),
            algorithm: "Ed25519".to_string(),
        };

        let did_document = DIDDocument {
            id: did.clone(),
            controller: controller.clone(),
            verification_methods: vec![verification_method.clone()],
            authentication: vec![verification_method.id.clone()],
            assertion_method: vec![verification_method.id.clone()],
            key_agreement: vec![],
            service_endpoints: vec![],
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.did_documents.insert(did.clone(), did_document);
        Ok(did)
    }

    pub fn add_verification_method(&mut self, did: &str, key_type: String, public_key: String) -> Result<String> {
        let did_document = self.did_documents.get_mut(did)
            .ok_or_else(|| BlockchainError::InvalidInput("DID not found".to_string()))?;

        let method_id = format!("{}#keys-{}", did, did_document.verification_methods.len() + 1);
        
        let verification_method = VerificationMethod {
            id: method_id.clone(),
            controller: did.to_string(),
            key_type,
            public_key,
            algorithm: "Ed25519".to_string(),
        };

        did_document.verification_methods.push(verification_method);
        did_document.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(method_id)
    }

    pub fn add_service_endpoint(&mut self, did: &str, service_type: String, endpoint: String) -> Result<String> {
        let did_document = self.did_documents.get_mut(did)
            .ok_or_else(|| BlockchainError::InvalidInput("DID not found".to_string()))?;

        let service_id = format!("{}#service-{}", did, did_document.service_endpoints.len() + 1);
        
        let service_endpoint = ServiceEndpoint {
            id: service_id.clone(),
            service_type,
            service_endpoint: endpoint,
            priority: 1,
        };

        did_document.service_endpoints.push(service_endpoint);
        did_document.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(service_id)
    }

    pub fn issue_credential(&mut self, issuer: &str, subject: &str, credential_type: Vec<String>, claims: HashMap<String, serde_json::Value>) -> Result<String> {
        let credential_id = format!("credential_{}", uuid::Uuid::new_v4());
        
        let credential = VerifiableCredential {
            id: credential_id.clone(),
            issuer: issuer.to_string(),
            subject: subject.to_string(),
            credential_type,
            claims,
            issuance_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expiration_date: None,
            proof: Some(CredentialProof {
                proof_type: "Ed25519Signature2020".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                verification_method: format!("{}#keys-1", issuer),
                proof_purpose: "assertionMethod".to_string(),
                proof_value: format!("proof_{}", uuid::Uuid::new_v4()),
            }),
        };

        self.credentials.insert(credential_id.clone(), credential);
        Ok(credential_id)
    }

    pub fn create_presentation(&mut self, holder: &str, credential_ids: Vec<String>) -> Result<String> {
        let presentation_id = format!("presentation_{}", uuid::Uuid::new_v4());
        
        let mut verifiable_credentials = Vec::new();
        for credential_id in credential_ids {
            if let Some(credential) = self.credentials.get(&credential_id) {
                verifiable_credentials.push(credential.clone());
            } else {
                return Err(BlockchainError::InvalidInput(format!("Credential {} not found", credential_id)));
            }
        }

        let presentation = VerifiablePresentation {
            id: presentation_id.clone(),
            holder: holder.to_string(),
            verifiable_credentials,
            proof: Some(PresentationProof {
                proof_type: "Ed25519Signature2020".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                verification_method: format!("{}#keys-1", holder),
                proof_purpose: "authentication".to_string(),
                proof_value: format!("proof_{}", uuid::Uuid::new_v4()),
            }),
        };

        self.presentations.insert(presentation_id.clone(), presentation);
        Ok(presentation_id)
    }

    pub fn verify_credential(&self, credential_id: &str) -> Result<bool> {
        let credential = self.credentials.get(credential_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Credential not found".to_string()))?;

        // Check if credential is expired
        if let Some(expiration_date) = credential.expiration_date {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if current_time > expiration_date {
                return Ok(false);
            }
        }

        // Verify proof (simplified)
        if let Some(proof) = &credential.proof {
            if !proof.proof_value.starts_with("proof_") {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn verify_presentation(&self, presentation_id: &str) -> Result<bool> {
        let presentation = self.presentations.get(presentation_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Presentation not found".to_string()))?;

        // Verify all credentials in the presentation
        for credential in &presentation.verifiable_credentials {
            if !self.verify_credential(&credential.id)? {
                return Ok(false);
            }
        }

        // Verify presentation proof (simplified)
        if let Some(proof) = &presentation.proof {
            if !proof.proof_value.starts_with("proof_") {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn setup_recovery_plan(&mut self, did: &str, guardians: Vec<String>, threshold: u32) -> Result<String> {
        let recovery_id = format!("recovery_{}", uuid::Uuid::new_v4());
        
        let recovery_plan = IdentityRecovery {
            id: recovery_id.clone(),
            did: did.to_string(),
            recovery_methods: vec![],
            guardians,
            threshold,
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.recovery_plans.insert(recovery_id.clone(), recovery_plan);
        Ok(recovery_id)
    }

    pub fn add_recovery_method(&mut self, recovery_id: &str, method_type: RecoveryMethodType, value: String) -> Result<String> {
        let recovery_plan = self.recovery_plans.get_mut(recovery_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Recovery plan not found".to_string()))?;

        let method_id = format!("method_{}", uuid::Uuid::new_v4());
        
        let recovery_method = RecoveryMethod {
            id: method_id.clone(),
            method_type,
            value,
            is_active: true,
        };

        recovery_plan.recovery_methods.push(recovery_method);
        Ok(method_id)
    }

    pub fn initiate_recovery(&mut self, recovery_id: &str, guardian_signatures: Vec<String>) -> Result<bool> {
        let recovery_plan = self.recovery_plans.get(recovery_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Recovery plan not found".to_string()))?;

        if guardian_signatures.len() < recovery_plan.threshold as usize {
            return Err(BlockchainError::InvalidInput("Insufficient guardian signatures".to_string()));
        }

        // Verify guardian signatures (simplified)
        for signature in guardian_signatures {
            if !signature.starts_with("guardian_sig_") {
                return Ok(false);
            }
        }

        println!("🔄 Identity recovery initiated for DID: {}", recovery_plan.did);
        Ok(true)
    }

    pub fn get_did_document(&self, did: &str) -> Result<&DIDDocument> {
        self.did_documents.get(did)
            .ok_or_else(|| BlockchainError::InvalidInput("DID not found".to_string()))
    }

    pub fn revoke_credential(&mut self, credential_id: &str) -> Result<()> {
        if !self.credentials.contains_key(credential_id) {
            return Err(BlockchainError::InvalidInput("Credential not found".to_string()));
        }

        self.credentials.remove(credential_id);
        println!("🗑️ Credential {} has been revoked", credential_id);
        Ok(())
    }

    pub fn update_did_document(&mut self, did: &str) -> Result<()> {
        let did_document = self.did_documents.get_mut(did)
            .ok_or_else(|| BlockchainError::InvalidInput("DID not found".to_string()))?;

        did_document.updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }
}

pub struct DIDSuite {
    _manager: DIDManager,
}

impl DIDSuite {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            _manager: DIDManager::new(blockchain),
        }
    }

    pub async fn test_did_creation(&self) -> Result<()> {
        println!("🧪 Testing DID creation...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DID
        let did = manager.create_did("alice".to_string())?;

        assert!(did.starts_with("did:gillean:"));
        assert!(manager.did_documents.contains_key(&did));
        
        let did_document = manager.get_did_document(&did)?;
        assert_eq!(did_document.controller, "alice");
        assert_eq!(did_document.verification_methods.len(), 1);
        assert_eq!(did_document.authentication.len(), 1);

        println!("✅ DID creation test passed!");
        Ok(())
    }

    pub async fn test_verification_method_management(&self) -> Result<()> {
        println!("🧪 Testing verification method management...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DID
        let did = manager.create_did("alice".to_string())?;

        // Add verification method
        let method_id = manager.add_verification_method(
            &did,
            "RsaVerificationKey2018".to_string(),
            "rsa_public_key_123".to_string(),
        )?;

        let did_document = manager.get_did_document(&did)?;
        assert_eq!(did_document.verification_methods.len(), 2);
        assert!(method_id.starts_with(&did));

        println!("✅ Verification method management test passed!");
        Ok(())
    }

    pub async fn test_service_endpoint_management(&self) -> Result<()> {
        println!("🧪 Testing service endpoint management...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DID
        let did = manager.create_did("alice".to_string())?;

        // Add service endpoint
        let _service_id = manager.add_service_endpoint(
            &did,
            "LinkedDomains".to_string(),
            "https://alice.example.com".to_string(),
        )?;

        let did_document = manager.get_did_document(&did)?;
        assert_eq!(did_document.service_endpoints.len(), 1);
        assert_eq!(did_document.service_endpoints[0].service_type, "LinkedDomains");
        assert_eq!(did_document.service_endpoints[0].service_endpoint, "https://alice.example.com");

        println!("✅ Service endpoint management test passed!");
        Ok(())
    }

    pub async fn test_credential_issuance(&self) -> Result<()> {
        println!("🧪 Testing credential issuance...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DIDs
        let issuer_did = manager.create_did("university".to_string())?;
        let subject_did = manager.create_did("alice".to_string())?;

        // Issue credential
        let mut claims = HashMap::new();
        claims.insert("degree".to_string(), serde_json::json!("Bachelor of Science"));
        claims.insert("major".to_string(), serde_json::json!("Computer Science"));
        claims.insert("graduationYear".to_string(), serde_json::json!(2023));

        let credential_id = manager.issue_credential(
            &issuer_did,
            &subject_did,
            vec!["UniversityDegree".to_string()],
            claims,
        )?;

        assert!(manager.credentials.contains_key(&credential_id));
        let credential = &manager.credentials[&credential_id];
        assert_eq!(credential.issuer, issuer_did);
        assert_eq!(credential.subject, subject_did);
        assert!(credential.proof.is_some());

        println!("✅ Credential issuance test passed!");
        Ok(())
    }

    pub async fn test_presentation_creation(&self) -> Result<()> {
        println!("🧪 Testing presentation creation...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DIDs and credential
        let issuer_did = manager.create_did("university".to_string())?;
        let holder_did = manager.create_did("alice".to_string())?;

        let mut claims = HashMap::new();
        claims.insert("degree".to_string(), serde_json::json!("Bachelor of Science"));
        let credential_id = manager.issue_credential(
            &issuer_did,
            &holder_did,
            vec!["UniversityDegree".to_string()],
            claims,
        )?;

        // Create presentation
        let presentation_id = manager.create_presentation(
            &holder_did,
            vec![credential_id],
        )?;

        assert!(manager.presentations.contains_key(&presentation_id));
        let presentation = &manager.presentations[&presentation_id];
        assert_eq!(presentation.holder, holder_did);
        assert_eq!(presentation.verifiable_credentials.len(), 1);
        assert!(presentation.proof.is_some());

        println!("✅ Presentation creation test passed!");
        Ok(())
    }

    pub async fn test_credential_verification(&self) -> Result<()> {
        println!("🧪 Testing credential verification...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DIDs and credential
        let issuer_did = manager.create_did("university".to_string())?;
        let subject_did = manager.create_did("alice".to_string())?;

        let mut claims = HashMap::new();
        claims.insert("degree".to_string(), serde_json::json!("Bachelor of Science"));
        let credential_id = manager.issue_credential(
            &issuer_did,
            &subject_did,
            vec!["UniversityDegree".to_string()],
            claims,
        )?;

        // Verify credential
        let is_valid = manager.verify_credential(&credential_id)?;
        assert!(is_valid);

        // Test invalid credential
        let is_valid = manager.verify_credential("non_existent");
        assert!(is_valid.is_err());

        println!("✅ Credential verification test passed!");
        Ok(())
    }

    pub async fn test_presentation_verification(&self) -> Result<()> {
        println!("🧪 Testing presentation verification...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DIDs, credential, and presentation
        let issuer_did = manager.create_did("university".to_string())?;
        let holder_did = manager.create_did("alice".to_string())?;

        let mut claims = HashMap::new();
        claims.insert("degree".to_string(), serde_json::json!("Bachelor of Science"));
        let credential_id = manager.issue_credential(
            &issuer_did,
            &holder_did,
            vec!["UniversityDegree".to_string()],
            claims,
        )?;

        let presentation_id = manager.create_presentation(
            &holder_did,
            vec![credential_id],
        )?;

        // Verify presentation
        let is_valid = manager.verify_presentation(&presentation_id)?;
        assert!(is_valid);

        println!("✅ Presentation verification test passed!");
        Ok(())
    }

    pub async fn test_identity_recovery(&self) -> Result<()> {
        println!("🧪 Testing identity recovery...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DID
        let did = manager.create_did("alice".to_string())?;

        // Setup recovery plan
        let recovery_id = manager.setup_recovery_plan(
            &did,
            vec!["guardian1".to_string(), "guardian2".to_string(), "guardian3".to_string()],
            2,
        )?;

        // Add recovery methods
        manager.add_recovery_method(
            &recovery_id,
            RecoveryMethodType::Email,
            "alice@example.com".to_string(),
        )?;

        manager.add_recovery_method(
            &recovery_id,
            RecoveryMethodType::Phone,
            "+1234567890".to_string(),
        )?;

        // Initiate recovery
        let guardian_signatures = vec![
            "guardian_sig_1".to_string(),
            "guardian_sig_2".to_string(),
        ];

        let recovery_successful = manager.initiate_recovery(&recovery_id, guardian_signatures)?;
        assert!(recovery_successful);

        println!("✅ Identity recovery test passed!");
        Ok(())
    }

    pub async fn test_credential_revocation(&self) -> Result<()> {
        println!("🧪 Testing credential revocation...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DIDs and credential
        let issuer_did = manager.create_did("university".to_string())?;
        let subject_did = manager.create_did("alice".to_string())?;

        let mut claims = HashMap::new();
        claims.insert("degree".to_string(), serde_json::json!("Bachelor of Science"));
        let credential_id = manager.issue_credential(
            &issuer_did,
            &subject_did,
            vec!["UniversityDegree".to_string()],
            claims,
        )?;

        // Verify credential exists
        assert!(manager.credentials.contains_key(&credential_id));

        // Revoke credential
        manager.revoke_credential(&credential_id)?;

        // Verify credential is removed
        assert!(!manager.credentials.contains_key(&credential_id));

        println!("✅ Credential revocation test passed!");
        Ok(())
    }

    pub async fn test_did_document_updates(&self) -> Result<()> {
        println!("🧪 Testing DID document updates...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Create DID
        let did = manager.create_did("alice".to_string())?;
        let original_updated = manager.get_did_document(&did)?.updated;

        // Wait a moment to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Update DID document
        manager.update_did_document(&did)?;

        let updated_document = manager.get_did_document(&did)?;
        assert!(updated_document.updated > original_updated);

        println!("✅ DID document updates test passed!");
        Ok(())
    }

    pub async fn test_invalid_operations(&self) -> Result<()> {
        println!("🧪 Testing invalid operations...");

        let mut manager = DIDManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));

        // Test getting non-existent DID
        let result = manager.get_did_document("did:gillean:non_existent");
        assert!(result.is_err());

        // Test adding verification method to non-existent DID
        let result = manager.add_verification_method(
            "did:gillean:non_existent",
            "RsaVerificationKey2018".to_string(),
            "public_key".to_string(),
        );
        assert!(result.is_err());

        // Test creating presentation with non-existent credential
        let holder_did = manager.create_did("alice".to_string())?;
        let result = manager.create_presentation(
            &holder_did,
            vec!["non_existent_credential".to_string()],
        );
        assert!(result.is_err());

        // Test recovery with insufficient signatures
        let did = manager.create_did("bob".to_string())?;
        let recovery_id = manager.setup_recovery_plan(
            &did,
            vec!["guardian1".to_string(), "guardian2".to_string()],
            2,
        )?;

        let result = manager.initiate_recovery(&recovery_id, vec!["guardian_sig_1".to_string()]);
        assert!(result.is_err());

        println!("✅ Invalid operations test passed!");
        Ok(())
    }

    pub async fn run_all_tests(&self) -> Result<()> {
        println!("🚀 Running Decentralized Identity (DID) test suite...");
        
        self.test_did_creation().await?;
        self.test_verification_method_management().await?;
        self.test_service_endpoint_management().await?;
        self.test_credential_issuance().await?;
        self.test_presentation_creation().await?;
        self.test_credential_verification().await?;
        self.test_presentation_verification().await?;
        self.test_identity_recovery().await?;
        self.test_credential_revocation().await?;
        self.test_did_document_updates().await?;
        self.test_invalid_operations().await?;

        println!("✅ All Decentralized Identity (DID) tests passed!");
        Ok(())
    }
}
