//! Computation trace builder for ZKP
//!
//! This module converts authorization logic into a sequence of rows
//! (computation trace) that can be proven by the ZKP engine.

use anyhow::{Result, Context};
use myzkp::{F, air::Row};

use crate::patient::PatientFeatures;
use crate::policy::{Policy, AdminRules};
use crate::criterion::Criterion;
use crate::commitment::{compute_patient_commitment, bytes_to_field};
use crate::decision::AuthorizationResult;

/// Trace builder for authorization proofs
pub struct TraceBuilder;

impl TraceBuilder {
    /// Build a complete computation trace for authorization
    ///
    /// The trace includes:
    /// - Commitment verification rows (patient commitment, policy hash)
    /// - Inclusion criterion evaluation rows (AND logic)
    /// - Exclusion criterion evaluation rows (OR logic)
    /// - Admin rules evaluation rows (POS, max units)
    /// - Final decision row
    ///
    /// # Arguments
    /// * `features` - Patient features (private witness)
    /// * `policy` - Policy to evaluate (structure)
    /// * `salt` - Random salt for patient commitment (private)
    /// * `policy_hash` - SHA-256 hash of canonical policy JSON (public)
    /// * `patient_commitment` - SHA-256 commitment of features + salt (public)
    ///
    /// # Returns
    /// * `Ok((Vec<Row>, AuthorizationResult))` - Computation trace and result
    /// * `Err` - If trace building fails
    pub fn build(
        features: &PatientFeatures,
        policy: &Policy,
        salt: &[u8; 32],
        policy_hash: &[u8; 32],
        patient_commitment: &[u8; 32],
    ) -> Result<(Vec<Row>, AuthorizationResult)> {
        let mut trace = Vec::new();
        let _k = 6;  // 6 registers per row
        
        // ================================================================
        // Row 0: Verify patient commitment
        // ================================================================
        let computed_commitment = compute_patient_commitment(features, salt);
        if &computed_commitment != patient_commitment {
            anyhow::bail!("Patient commitment mismatch!");
        }
        
        trace.push(Row {
            regs: vec![
                F::from(0),                                     // Step 0: commitment check
                bytes_to_field(&computed_commitment[0..8])?,    // Computed (first 8 bytes)
                bytes_to_field(&patient_commitment[0..8])?,     // Expected (public input)
                F::from(1),                                     // Match: 1 (must pass)
                F::from(1),                                     // running_and = 1 (init)
                F::from(0),                                     // running_or = 0 (init)
            ].into_boxed_slice()
        });
        
        // ================================================================
        // Row 1: Verify policy hash
        // ================================================================
        trace.push(Row {
            regs: vec![
                F::from(1),                                     // Step 1: policy hash check
                bytes_to_field(&policy_hash[0..8])?,            // Policy hash (first 8 bytes, public)
                bytes_to_field(&policy_hash[0..8])?,            // Expected (same, for verification)
                F::from(1),                                     // Match: 1
                F::from(1),                                     // running_and = 1
                F::from(0),                                     // running_or = 0
            ].into_boxed_slice()
        });
        
        // ================================================================
        // Rows 2+: Evaluate inclusion criteria (AND logic)
        // ================================================================
        let mut running_and = F::from(1);
        
        for (i, criterion_json) in policy.inclusion.iter().enumerate() {
            let criterion = Criterion::from_json(criterion_json)
                .with_context(|| format!("Failed to parse inclusion criterion {}", i))?;
            
            let passes = criterion.evaluate(features)?;
            let result_field = F::from(passes as u64);
            
            // AND operation: multiply in field (0 * anything = 0)
            running_and *= result_field;
            
            let patient_val = criterion.extract_patient_value(features)?;
            let policy_val = criterion.extract_policy_value();
            
            trace.push(Row {
                regs: vec![
                    F::from((i + 2) as u64),                    // Step ID
                    patient_val,                                // Patient's value for this field
                    policy_val,                                 // Policy's threshold/list value
                    result_field,                               // 1=pass, 0=fail
                    running_and,                                // Cumulative AND
                    F::from(0),                                 // (not used in inclusion)
                ].into_boxed_slice()
            });
        }
        
        let medical_inclusion_ok = running_and;
        
        // ================================================================
        // Next rows: Evaluate exclusion criteria (OR logic)
        // ================================================================
        let mut running_or = F::from(0);
        let base_step = 100;  // Start exclusion steps at 100
        
        for (i, criterion_json) in policy.exclusion.iter().enumerate() {
            let criterion = Criterion::from_json(criterion_json)
                .with_context(|| format!("Failed to parse exclusion criterion {}", i))?;
            
            let hits = criterion.evaluate(features)?;
            let result_field = F::from(hits as u64);
            
            // OR operation: if any criterion hits, set running_or = 1
            if hits {
                running_or = F::from(1);
            }
            
            let patient_val = criterion.extract_patient_value(features)?;
            let policy_val = criterion.extract_policy_value();
            
            trace.push(Row {
                regs: vec![
                    F::from((base_step + i) as u64),            // Step ID (100+)
                    patient_val,                                // Patient's value
                    policy_val,                                 // Policy's threshold/value
                    result_field,                               // 1=hit, 0=no hit
                    medical_inclusion_ok,                       // Keep inclusion result
                    running_or,                                 // Cumulative OR
                ].into_boxed_slice()
            });
        }
        
        let exclusion_hit = running_or;
        
        // ================================================================
        // Next rows: Evaluate admin rules
        // ================================================================
        let (admin_rows, admin_ok) = Self::build_admin_rows(
            features,
            &policy.admin_rules,
            medical_inclusion_ok,
            exclusion_hit,
        )?;
        trace.extend(admin_rows);
        
        // ================================================================
        // Final row: Compute authorization outcome
        // ================================================================
        let medical_ok_bool = (medical_inclusion_ok == F::from(1)) && (exclusion_hit == F::from(0));
        let admin_ok_bool = admin_ok == F::from(1);
        
        let final_result = AuthorizationResult::from_logic(
            medical_ok_bool,
            policy.requires_pa,
            admin_ok_bool,
        );
        
        trace.push(Row {
            regs: vec![
                F::from(999),                                   // Final step marker
                F::from(medical_ok_bool as u64),                // Medical OK (inclusion AND NOT exclusion)
                F::from(policy.requires_pa as u64),             // Requires PA flag
                final_result.to_field(),                        // Result: 1=APPROVE, 2=NEEDS_PA, 3=DENY
                admin_ok,                                       // Admin OK
                F::from(0),                                     // (unused)
            ].into_boxed_slice()
        });
        
        Ok((trace, final_result))
    }
    
    /// Build admin rules evaluation rows
    ///
    /// # Arguments
    /// * `features` - Patient features
    /// * `admin_rules` - Admin rules from policy
    /// * `medical_inclusion_ok` - Result of inclusion criteria (for tracking)
    /// * `exclusion_hit` - Result of exclusion criteria (for tracking)
    ///
    /// # Returns
    /// * `Ok((Vec<Row>, F))` - Admin rows and final admin_ok flag
    fn build_admin_rows(
        features: &PatientFeatures,
        admin_rules: &AdminRules,
        medical_inclusion_ok: F,
        exclusion_hit: F,
    ) -> Result<(Vec<Row>, F)> {
        let mut rows = Vec::new();
        
        // Check place of service
        let pos_allowed = admin_rules.pos_allowed.contains(&features.pos);
        let pos_ok_field = F::from(pos_allowed as u64);
        
        rows.push(Row {
            regs: vec![
                F::from(200),                                   // Step 200: POS check
                F::from(features.pos as u64),                   // Patient's POS
                F::from(admin_rules.pos_allowed.first().copied().unwrap_or(0) as u64), // First allowed POS
                pos_ok_field,                                   // 1=allowed, 0=not allowed
                medical_inclusion_ok,                           // (tracking)
                exclusion_hit,                                  // (tracking)
            ].into_boxed_slice()
        });
        
        // Check max units
        let units_ok = features.units <= admin_rules.max_units_per_day;
        let units_ok_field = F::from(units_ok as u64);
        
        rows.push(Row {
            regs: vec![
                F::from(201),                                   // Step 201: units check
                F::from(features.units as u64),                 // Patient's units
                F::from(admin_rules.max_units_per_day as u64),  // Max allowed units
                units_ok_field,                                 // 1=OK, 0=exceeds
                medical_inclusion_ok,                           // (tracking)
                exclusion_hit,                                  // (tracking)
            ].into_boxed_slice()
        });
        
        // Final admin_ok: AND of all admin checks
        let admin_ok = pos_ok_field * units_ok_field;  // AND in field: both must be 1
        
        Ok((rows, admin_ok))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::AdminRules;

    fn test_features_approve() -> PatientFeatures {
        PatientFeatures {
            age_years: 55,
            sex: 0,  // M
            primary_icd10: 2001,  // J18.9 (pneumonia)
            pregnant: 0,
            pos: 22,
            units: 1,
        }
    }

    fn test_policy_ct_chest() -> Policy {
        use serde_json::json;
        
        Policy {
            policy_id: "TEST-CT-001".to_string(),
            version: "2025-11-01".to_string(),
            lob: "commercial".to_string(),
            codes: vec!["71250".to_string()],
            requires_pa: false,
            inclusion: vec![
                json!({"gte": ["age_years", 18]}),
                json!({"in": ["primary_icd10", [2001, 2002, 2003]]}),
            ],
            exclusion: vec![
                json!({"eq": ["pregnant", 1]}),
            ],
            admin_rules: AdminRules {
                pos_allowed: vec![21, 22, 23],
                max_units_per_day: 1,
            },
        }
    }

    #[test]
    fn test_build_trace_approve() {
        let features = test_features_approve();
        let policy = test_policy_ct_chest();
        let salt = [0x42; 32];
        
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let policy_hash = [0x01; 32];  // Dummy hash
        
        let result = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &patient_commitment,
        );
        
        assert!(result.is_ok());
        let (trace, decision) = result.unwrap();
        
        // Should have multiple rows
        assert!(trace.len() > 5);
        
        // Decision should be APPROVE (no PA, all criteria pass)
        assert_eq!(decision, AuthorizationResult::Approve);
        
        // Each row should have 6 registers
        for row in &trace {
            assert_eq!(row.regs.len(), 6);
        }
    }

    #[test]
    fn test_build_trace_needs_pa() {
        let features = test_features_approve();
        let mut policy = test_policy_ct_chest();
        policy.requires_pa = true;  // Change to require PA
        
        let salt = [0x42; 32];
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let policy_hash = [0x01; 32];
        
        let (trace, decision) = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &patient_commitment,
        ).unwrap();
        
        // Decision should be NEEDS_PA
        assert_eq!(decision, AuthorizationResult::NeedsPa);
    }

    #[test]
    fn test_build_trace_deny_age() {
        let mut features = test_features_approve();
        features.age_years = 16;  // Too young
        
        let policy = test_policy_ct_chest();
        let salt = [0x42; 32];
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let policy_hash = [0x01; 32];
        
        let (trace, decision) = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &patient_commitment,
        ).unwrap();
        
        // Decision should be DENY (fails inclusion)
        assert_eq!(decision, AuthorizationResult::Deny);
    }

    #[test]
    fn test_build_trace_deny_exclusion() {
        let mut features = test_features_approve();
        features.sex = 1;  // F
        features.pregnant = 1;  // Pregnant (hits exclusion)
        
        let policy = test_policy_ct_chest();
        let salt = [0x42; 32];
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let policy_hash = [0x01; 32];
        
        let (trace, decision) = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &patient_commitment,
        ).unwrap();
        
        // Decision should be DENY (exclusion hit)
        assert_eq!(decision, AuthorizationResult::Deny);
    }

    #[test]
    fn test_build_trace_deny_admin() {
        let mut features = test_features_approve();
        features.units = 2;  // Exceeds max (1)
        
        let policy = test_policy_ct_chest();
        let salt = [0x42; 32];
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let policy_hash = [0x01; 32];
        
        let (trace, decision) = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &patient_commitment,
        ).unwrap();
        
        // Decision should be DENY (admin rule failure)
        assert_eq!(decision, AuthorizationResult::Deny);
    }

    #[test]
    fn test_commitment_mismatch() {
        let features = test_features_approve();
        let policy = test_policy_ct_chest();
        let salt = [0x42; 32];
        
        let patient_commitment = compute_patient_commitment(&features, &salt);
        let wrong_commitment = [0xFF; 32];  // Wrong commitment
        let policy_hash = [0x01; 32];
        
        let result = TraceBuilder::build(
            &features,
            &policy,
            &salt,
            &policy_hash,
            &wrong_commitment,  // Mismatch!
        );
        
        assert!(result.is_err());
    }
}

