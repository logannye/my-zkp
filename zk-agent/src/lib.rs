//! Privacy-Preserving Medical Authorization Library
//!
//! This library provides zero-knowledge proof capabilities for medical claim
//! authorization, enabling proofs that authorization decisions follow published
//! payer rules without exposing patient PHI.
//!
//! # Overview
//!
//! The library implements a privacy-preserving authorization system using
//! zero-knowledge proofs. The core workflow:
//!
//! 1. **Policy Definition**: Payer publishes authorization rules (policy JSON)
//! 2. **Patient Features**: Provider extracts patient features (stays private)
//! 3. **Trace Building**: Authorization logic converted to computation trace
//! 4. **Proof Generation**: ZKP proves correct evaluation without revealing data
//! 5. **Verification**: Payer verifies proof to confirm authorization decision
//!
//! # Example
//!
//! ```no_run
//! use zk_agent::*;
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Load policy
//! let (policy, canonical_json) = policy::PolicyParser::load_and_canonicalize(
//!     Path::new("policies/UHC-COMM-BIOPSY-001.json")
//! )?;
//!
//! // Extract patient features
//! let patient = patient::PatientExtractor::from_file(
//!     Path::new("patients/p002-needs-pa.json")
//! )?;
//! let features = patient::PatientExtractor::extract_features(&patient)?;
//!
//! // Generate commitment
//! let salt = commitment::generate_random_salt();
//! let patient_commitment = commitment::compute_patient_commitment(&features, &salt);
//! let policy_hash = commitment::compute_policy_hash(&canonical_json);
//!
//! // Build trace and generate proof
//! let (trace, result) = trace::TraceBuilder::build(
//!     &features,
//!     &policy,
//!     &salt,
//!     &policy_hash,
//!     &patient_commitment,
//! )?;
//!
//! // ... generate ZKP proof using myzkp engine ...
//!
//! println!("Authorization result: {}", result.to_string());
//! # Ok(())
//! # }
//! ```
//!
//! # Modules
//!
//! - [`policy`]: Policy data structures and parsing
//! - [`patient`]: Patient data structures and feature extraction
//! - [`commitment`]: Cryptographic hash functions
//! - [`criterion`]: Criterion evaluation logic
//! - [`trace`]: Computation trace builder for ZKP
//! - [`decision`]: Authorization result and decision record
//! - [`icd_map`]: ICD-10 to integer mapping
//!
//! # Privacy Guarantees
//!
//! The ZKP system provides:
//!
//! - **Integrity**: Decisions provably follow published rules
//! - **Privacy**: No PHI exposure beyond the authorization outcome
//! - **Auditability**: Every decision can be re-verified with the same proof

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod policy;
pub mod patient;
pub mod commitment;
pub mod criterion;
pub mod trace;
pub mod decision;
pub mod icd_map;

// Re-export key types for convenience
pub use policy::{Policy, PolicyParser, AdminRules};
pub use patient::{PatientRecord, PatientFeatures, PatientExtractor};
pub use commitment::{compute_policy_hash, compute_patient_commitment, generate_random_salt};
pub use criterion::Criterion;
pub use trace::TraceBuilder;
pub use decision::{AuthorizationResult, DecisionRecord};
pub use icd_map::{map_icd10, is_valid_icd10};

