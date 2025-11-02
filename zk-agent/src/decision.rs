//! Authorization decision logic and decision record serialization
//!
//! This module defines the authorization outcome (Approve/NeedsPa/Deny)
//! and the decision record format that gets sent to the verifier.

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use myzkp::{F, Proof};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

use crate::policy::Policy;

/// Authorization decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationResult {
    Approve = 1,
    NeedsPa = 2,
    Deny = 3,
}

impl AuthorizationResult {
    /// Compute authorization result from decision logic
    ///
    /// Decision tree:
    /// 1. If medical criteria not met → DENY
    /// 2. Else if admin rules not met → DENY
    /// 3. Else if policy requires PA → NEEDS_PA
    /// 4. Else → APPROVE
    ///
    /// # Arguments
    /// * `medical_ok` - All inclusion criteria pass AND no exclusion hits
    /// * `requires_pa` - Policy flag indicating PA requirement
    /// * `admin_ok` - All admin rules (POS, max units) pass
    pub fn from_logic(medical_ok: bool, requires_pa: bool, admin_ok: bool) -> Self {
        if !medical_ok {
            AuthorizationResult::Deny
        } else if !admin_ok {
            AuthorizationResult::Deny
        } else if requires_pa {
            AuthorizationResult::NeedsPa
        } else {
            AuthorizationResult::Approve
        }
    }
    
    /// Convert to u8 for encoding in ZKP trace
    pub fn to_u8(self) -> u8 {
        self as u8
    }
    
    /// Convert to field element for ZKP trace
    pub fn to_field(self) -> F {
        F::from(self as u64)
    }
    
    /// Convert to string representation
    pub fn to_string(self) -> &'static str {
        match self {
            AuthorizationResult::Approve => "APPROVE",
            AuthorizationResult::NeedsPa => "NEEDS_PA",
            AuthorizationResult::Deny => "DENY",
        }
    }
    
    /// Parse from string
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "APPROVE" => Ok(AuthorizationResult::Approve),
            "NEEDS_PA" => Ok(AuthorizationResult::NeedsPa),
            "DENY" => Ok(AuthorizationResult::Deny),
            _ => anyhow::bail!("Invalid authorization result: {}", s),
        }
    }
}

/// Decision record (sent to verifier, includes proof)
///
/// This is the public output of the authorization process that gets
/// sent to the payer/verifier. It contains:
/// - Policy identifier and hash (public)
/// - Patient commitment (public)
/// - Claimed result (public)
/// - ZKP proof (public)
///
/// What's NOT included: Any patient PHI (age, sex, diagnoses, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub policy_id: String,
    pub policy_hash: String,          // hex-encoded
    pub patient_commitment: String,    // hex-encoded
    pub code: String,                  // CPT/HCPCS code
    pub lob: String,                   // Line of business
    pub claimed_result: String,        // "APPROVE" | "NEEDS_PA" | "DENY"
    pub proof: String,                 // base64-encoded proof
}

impl DecisionRecord {
    /// Create a new decision record
    ///
    /// # Arguments
    /// * `policy` - The policy that was evaluated
    /// * `policy_hash` - SHA-256 hash of canonical policy JSON
    /// * `patient_commitment` - SHA-256 commitment of patient features + salt
    /// * `result` - The authorization decision
    /// * `proof` - The ZKP proof
    pub fn new(
        policy: &Policy,
        policy_hash: &[u8; 32],
        patient_commitment: &[u8; 32],
        result: AuthorizationResult,
        proof: &Proof,
    ) -> Result<Self> {
        // Serialize proof to bytes
        let mut proof_bytes = Vec::new();
        proof.serialize_compressed(&mut proof_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to serialize proof: {}", e))?;
        
        Ok(DecisionRecord {
            policy_id: policy.policy_id.clone(),
            policy_hash: hex::encode(policy_hash),
            patient_commitment: hex::encode(patient_commitment),
            code: policy.codes.first().cloned().unwrap_or_default(),
            lob: policy.lob.clone(),
            claimed_result: result.to_string().to_string(),
            proof: base64::encode(&proof_bytes),
        })
    }
    
    /// Write decision record to a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to write the decision record
    pub fn to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize decision record")?;
        
        fs::write(path, json)
            .with_context(|| format!("Failed to write decision record to {}", path.display()))?;
        
        Ok(())
    }
    
    /// Load a decision record from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the decision record file
    pub fn from_file(path: &Path) -> Result<Self> {
        let json = fs::read_to_string(path)
            .with_context(|| format!("Failed to read decision record: {}", path.display()))?;
        
        let record: DecisionRecord = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse decision record JSON: {}", path.display()))?;
        
        Ok(record)
    }
    
    /// Decode the proof from base64
    pub fn decode_proof(&self) -> Result<Vec<u8>> {
        base64::decode(&self.proof)
            .context("Failed to decode base64 proof")
    }
    
    /// Get the authorization result as enum
    pub fn get_result(&self) -> Result<AuthorizationResult> {
        AuthorizationResult::from_string(&self.claimed_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_result_from_logic() {
        // Medical OK, no PA required, admin OK → APPROVE
        assert_eq!(
            AuthorizationResult::from_logic(true, false, true),
            AuthorizationResult::Approve
        );
        
        // Medical OK, PA required, admin OK → NEEDS_PA
        assert_eq!(
            AuthorizationResult::from_logic(true, true, true),
            AuthorizationResult::NeedsPa
        );
        
        // Medical not OK → DENY
        assert_eq!(
            AuthorizationResult::from_logic(false, false, true),
            AuthorizationResult::Deny
        );
        
        // Medical OK, no PA, admin not OK → DENY
        assert_eq!(
            AuthorizationResult::from_logic(true, false, false),
            AuthorizationResult::Deny
        );
    }

    #[test]
    fn test_authorization_result_conversions() {
        assert_eq!(AuthorizationResult::Approve.to_u8(), 1);
        assert_eq!(AuthorizationResult::NeedsPa.to_u8(), 2);
        assert_eq!(AuthorizationResult::Deny.to_u8(), 3);
        
        assert_eq!(AuthorizationResult::Approve.to_string(), "APPROVE");
        assert_eq!(AuthorizationResult::NeedsPa.to_string(), "NEEDS_PA");
        assert_eq!(AuthorizationResult::Deny.to_string(), "DENY");
    }

    #[test]
    fn test_authorization_result_from_string() {
        assert_eq!(
            AuthorizationResult::from_string("APPROVE").unwrap(),
            AuthorizationResult::Approve
        );
        assert_eq!(
            AuthorizationResult::from_string("NEEDS_PA").unwrap(),
            AuthorizationResult::NeedsPa
        );
        assert_eq!(
            AuthorizationResult::from_string("DENY").unwrap(),
            AuthorizationResult::Deny
        );
        
        assert!(AuthorizationResult::from_string("INVALID").is_err());
    }

    #[test]
    fn test_decision_record_serialization() {
        let record = DecisionRecord {
            policy_id: "TEST-001".to_string(),
            policy_hash: "abc123".to_string(),
            patient_commitment: "def456".to_string(),
            code: "19081".to_string(),
            lob: "commercial".to_string(),
            claimed_result: "NEEDS_PA".to_string(),
            proof: "dGVzdHByb29m".to_string(),  // "testproof" in base64
        };
        
        // Serialize to JSON
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("TEST-001"));
        assert!(json.contains("NEEDS_PA"));
        
        // Deserialize back
        let record2: DecisionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.policy_id, record2.policy_id);
        assert_eq!(record.claimed_result, record2.claimed_result);
    }

    #[test]
    fn test_decision_record_decode_proof() {
        let record = DecisionRecord {
            policy_id: "TEST-001".to_string(),
            policy_hash: "abc123".to_string(),
            patient_commitment: "def456".to_string(),
            code: "19081".to_string(),
            lob: "commercial".to_string(),
            claimed_result: "APPROVE".to_string(),
            proof: base64::encode(b"test proof data"),
        };
        
        let decoded = record.decode_proof().unwrap();
        assert_eq!(decoded, b"test proof data");
    }

    #[test]
    fn test_decision_record_get_result() {
        let record = DecisionRecord {
            policy_id: "TEST-001".to_string(),
            policy_hash: "abc123".to_string(),
            patient_commitment: "def456".to_string(),
            code: "19081".to_string(),
            lob: "commercial".to_string(),
            claimed_result: "NEEDS_PA".to_string(),
            proof: "".to_string(),
        };
        
        assert_eq!(record.get_result().unwrap(), AuthorizationResult::NeedsPa);
    }
}

