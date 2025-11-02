//! Cryptographic hash functions for policy and patient commitments
//!
//! This module provides SHA-256 based hashing for:
//! - Policy hash (from canonical JSON)
//! - Patient commitment (from features + salt)

use sha2::{Sha256, Digest};
use rand::Rng;
use myzkp::F;
use anyhow::Result;
use ark_ff::PrimeField;

use crate::patient::PatientFeatures;

/// Compute SHA-256 hash of canonical policy JSON
///
/// # Arguments
/// * `canonical_json` - The canonicalized policy JSON string
///
/// # Returns
/// * 32-byte hash of the policy
///
/// # Example
/// ```
/// use zk_agent::commitment::compute_policy_hash;
///
/// let canonical = r#"{"policy_id":"TEST-001","version":"2025-10-01"}"#;
/// let hash = compute_policy_hash(canonical);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn compute_policy_hash(canonical_json: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json.as_bytes());
    hasher.finalize().into()
}

/// Compute SHA-256 commitment of patient features + salt
///
/// The commitment is computed as:
/// ```text
/// commitment = SHA256(features_bytes || salt)
/// ```
///
/// Where `features_bytes` is a deterministic byte encoding of the features.
///
/// # Arguments
/// * `features` - Patient features to commit to
/// * `salt` - 32-byte random salt
///
/// # Returns
/// * 32-byte commitment hash
pub fn compute_patient_commitment(
    features: &PatientFeatures,
    salt: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    
    // Add features bytes
    let features_bytes = features.to_bytes();
    hasher.update(&features_bytes);
    
    // Add salt
    hasher.update(salt);
    
    hasher.finalize().into()
}

/// Generate a cryptographically secure random salt
///
/// # Returns
/// * 32 random bytes suitable for use as a commitment salt
pub fn generate_random_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill(&mut salt);
    salt
}

/// Convert first 8 bytes of a hash to a field element
///
/// This is used when encoding commitments/hashes in the ZKP trace.
///
/// # Arguments
/// * `bytes` - At least 8 bytes
///
/// # Returns
/// * Field element representation of the first 8 bytes (as u64)
pub fn bytes_to_field(bytes: &[u8]) -> Result<F> {
    if bytes.len() < 8 {
        anyhow::bail!("Need at least 8 bytes to convert to field element");
    }
    
    let val = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
    Ok(F::from(val))
}

/// Convert a field element to bytes (little-endian u64)
///
/// # Arguments
/// * `field` - Field element to convert
///
/// # Returns
/// * 8-byte representation (little-endian)
pub fn field_to_bytes(field: &F) -> Vec<u8> {
    // Extract the underlying u64 value
    // Note: This is simplified; in practice you'd need proper field serialization
    let val = field.into_bigint().0[0];  // Get lowest u64 limb
    val.to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_policy_hash() {
        let json = r#"{"policy_id":"TEST-001","version":"2025-10-01"}"#;
        let hash = compute_policy_hash(json);
        
        assert_eq!(hash.len(), 32);
        
        // Hashing is deterministic
        let hash2 = compute_policy_hash(json);
        assert_eq!(hash, hash2);
        
        // Different inputs produce different hashes
        let json2 = r#"{"policy_id":"TEST-002","version":"2025-10-01"}"#;
        let hash3 = compute_policy_hash(json2);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_compute_patient_commitment() {
        let features = PatientFeatures {
            age_years: 45,
            sex: 1,
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        };
        
        let salt = [0x42; 32];  // Fixed salt for testing
        let commitment = compute_patient_commitment(&features, &salt);
        
        assert_eq!(commitment.len(), 32);
        
        // Deterministic with same inputs
        let commitment2 = compute_patient_commitment(&features, &salt);
        assert_eq!(commitment, commitment2);
        
        // Different salt produces different commitment
        let salt2 = [0x43; 32];
        let commitment3 = compute_patient_commitment(&features, &salt2);
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_generate_random_salt() {
        let salt1 = generate_random_salt();
        let salt2 = generate_random_salt();
        
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
        
        // Should be different (extremely high probability)
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_bytes_to_field() {
        let bytes = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let field = bytes_to_field(&bytes).unwrap();
        
        // Verify it's a valid field element
        assert_eq!(field, F::from(0x0807060504030201u64));
    }

    #[test]
    fn test_bytes_to_field_insufficient() {
        let bytes = [0x01, 0x02, 0x03];  // Only 3 bytes
        assert!(bytes_to_field(&bytes).is_err());
    }

    #[test]
    fn test_commitment_different_features() {
        let features1 = PatientFeatures {
            age_years: 45,
            sex: 1,
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        };
        
        let features2 = PatientFeatures {
            age_years: 46,  // Different age
            sex: 1,
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        };
        
        let salt = [0x42; 32];
        let commitment1 = compute_patient_commitment(&features1, &salt);
        let commitment2 = compute_patient_commitment(&features2, &salt);
        
        assert_ne!(commitment1, commitment2);
    }

    #[test]
    fn test_salt_prevents_reverse_lookup() {
        // Even with known features, commitment should be unpredictable without salt
        let features = PatientFeatures {
            age_years: 45,
            sex: 1,
            primary_icd10: 1002,
            pregnant: 0,
            pos: 22,
            units: 1,
        };
        
        let salt1 = generate_random_salt();
        let salt2 = generate_random_salt();
        
        let commitment1 = compute_patient_commitment(&features, &salt1);
        let commitment2 = compute_patient_commitment(&features, &salt2);
        
        // Same features but different salts = different commitments
        assert_ne!(commitment1, commitment2);
    }
}

