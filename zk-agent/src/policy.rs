//! Policy data structures and JSON parsing
//!
//! This module handles payer authorization policies, including parsing
//! from JSON and canonicalization for deterministic hashing.

use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Context};

/// Complete policy specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Policy {
    pub policy_id: String,
    pub version: String,
    pub lob: String,                 // Line of business: "commercial", "medicare", "medicaid"
    pub codes: Vec<String>,          // CPT/HCPCS codes
    pub requires_pa: bool,           // Whether prior authorization is required
    pub inclusion: Vec<Value>,       // Inclusion criteria (must ALL pass)
    pub exclusion: Vec<Value>,       // Exclusion criteria (any hit = deny)
    pub admin_rules: AdminRules,
}

/// Administrative rules (non-clinical requirements)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminRules {
    pub pos_allowed: Vec<u32>,       // Allowed place of service codes
    pub max_units_per_day: u32,      // Maximum units per day
}

/// Policy parser and utilities
pub struct PolicyParser;

impl PolicyParser {
    /// Load a policy from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the policy JSON file
    ///
    /// # Returns
    /// * `Ok(Policy)` - Parsed policy
    /// * `Err` - If file cannot be read or JSON is invalid
    pub fn from_file(path: &Path) -> Result<Policy> {
        let json = fs::read_to_string(path)
            .with_context(|| format!("Failed to read policy file: {}", path.display()))?;
        
        let policy: Policy = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse policy JSON: {}", path.display()))?;
        
        Ok(policy)
    }
    
    /// Canonicalize a JSON string for deterministic hashing
    ///
    /// This function:
    /// 1. Parses the JSON
    /// 2. Re-serializes with sorted keys
    /// 3. Removes all whitespace
    ///
    /// This ensures that identical policies always produce the same hash,
    /// regardless of key ordering or formatting.
    ///
    /// # Arguments
    /// * `json` - The JSON string to canonicalize
    ///
    /// # Returns
    /// * `Ok(String)` - Canonicalized JSON string
    /// * `Err` - If JSON is invalid
    pub fn canonicalize_json(json: &str) -> Result<String> {
        let value: Value = serde_json::from_str(json)
            .context("Failed to parse JSON for canonicalization")?;
        
        // Serialize with compact format (no whitespace)
        // Note: serde_json sorts object keys by default
        let canonical = serde_json::to_string(&value)
            .context("Failed to serialize canonical JSON")?;
        
        Ok(canonical)
    }
    
    /// Load and canonicalize a policy file in one step
    ///
    /// Returns both the parsed policy and its canonical JSON representation
    /// for hashing.
    pub fn load_and_canonicalize(path: &Path) -> Result<(Policy, String)> {
        let json = fs::read_to_string(path)
            .with_context(|| format!("Failed to read policy file: {}", path.display()))?;
        
        let policy: Policy = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse policy JSON: {}", path.display()))?;
        
        let canonical = Self::canonicalize_json(&json)?;
        
        Ok((policy, canonical))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_policy_file(json: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(json.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_load_policy() {
        let json = r#"{
            "policy_id": "UHC-COMM-BIOPSY-001",
            "version": "2025-10-01",
            "lob": "commercial",
            "codes": ["19081"],
            "requires_pa": true,
            "inclusion": [
                {"gte": ["age_years", 18]},
                {"in": ["primary_icd10", [1001, 1002, 1003]]}
            ],
            "exclusion": [
                {"eq": ["pregnant", 1]}
            ],
            "admin_rules": {
                "pos_allowed": [11, 22],
                "max_units_per_day": 1
            }
        }"#;
        
        let file = create_test_policy_file(json);
        let policy = PolicyParser::from_file(file.path()).unwrap();
        
        assert_eq!(policy.policy_id, "UHC-COMM-BIOPSY-001");
        assert_eq!(policy.version, "2025-10-01");
        assert_eq!(policy.lob, "commercial");
        assert_eq!(policy.codes, vec!["19081"]);
        assert_eq!(policy.requires_pa, true);
        assert_eq!(policy.inclusion.len(), 2);
        assert_eq!(policy.exclusion.len(), 1);
        assert_eq!(policy.admin_rules.pos_allowed, vec![11, 22]);
        assert_eq!(policy.admin_rules.max_units_per_day, 1);
    }

    #[test]
    fn test_canonicalize_json() {
        // JSON with different formatting and key order
        let json1 = r#"{"b": 2, "a": 1}"#;
        let json2 = r#"{
            "a": 1,
            "b": 2
        }"#;
        
        let canonical1 = PolicyParser::canonicalize_json(json1).unwrap();
        let canonical2 = PolicyParser::canonicalize_json(json2).unwrap();
        
        // Both should produce the same canonical form
        assert_eq!(canonical1, canonical2);
        
        // Canonical form should have no whitespace
        assert!(!canonical1.contains(' '));
        assert!(!canonical1.contains('\n'));
    }

    #[test]
    fn test_canonicalize_preserves_structure() {
        let json = r#"{
            "policy_id": "TEST-001",
            "codes": ["12345", "67890"],
            "requires_pa": true,
            "inclusion": [{"gte": ["age_years", 18]}]
        }"#;
        
        let canonical = PolicyParser::canonicalize_json(json).unwrap();
        
        // Re-parse to verify structure is preserved
        let value: Value = serde_json::from_str(&canonical).unwrap();
        assert_eq!(value["policy_id"], "TEST-001");
        assert_eq!(value["codes"].as_array().unwrap().len(), 2);
        assert_eq!(value["requires_pa"], true);
    }

    #[test]
    fn test_load_and_canonicalize() {
        let json = r#"{
            "policy_id": "TEST-002",
            "version": "2025-11-01",
            "lob": "medicare",
            "codes": ["99999"],
            "requires_pa": false,
            "inclusion": [],
            "exclusion": [],
            "admin_rules": {
                "pos_allowed": [11],
                "max_units_per_day": 2
            }
        }"#;
        
        let file = create_test_policy_file(json);
        let (policy, canonical) = PolicyParser::load_and_canonicalize(file.path()).unwrap();
        
        assert_eq!(policy.policy_id, "TEST-002");
        assert!(!canonical.contains('\n'));
        
        // Verify canonical can be re-parsed
        let reparsed: Policy = serde_json::from_str(&canonical).unwrap();
        assert_eq!(reparsed.policy_id, policy.policy_id);
    }

    #[test]
    fn test_invalid_json() {
        let invalid_json = r#"{"invalid": json syntax}"#;
        assert!(PolicyParser::canonicalize_json(invalid_json).is_err());
    }
}

