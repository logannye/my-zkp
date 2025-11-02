//! `authz prove` command implementation

use std::path::PathBuf;
use clap::Args;
use anyhow::{Result, Context};
use myzkp::{F, domain::Domain, air::AirSpec};
use myzkp::api::{ProverBuilder, prove_from_rows};

use crate::policy::PolicyParser;
use crate::patient::PatientExtractor;
use crate::commitment::{compute_policy_hash, compute_patient_commitment, generate_random_salt};
use crate::trace::TraceBuilder;
use crate::decision::DecisionRecord;

/// Arguments for the `authz prove` command
#[derive(Args, Debug)]
pub struct ProveArgs {
    /// Path to policy JSON file
    #[arg(long)]
    pub policy: PathBuf,
    
    /// Path to patient JSON file
    #[arg(long)]
    pub patient: PathBuf,
    
    /// CPT/HCPCS code (informational, taken from policy)
    #[arg(long)]
    pub code: String,
    
    /// Line of business (informational, taken from policy)
    #[arg(long)]
    pub lob: String,
    
    /// Output path for decision record + proof
    #[arg(long)]
    pub out: PathBuf,
}

/// Execute the `authz prove` command
///
/// This command:
/// 1. Loads and canonicalizes the policy
/// 2. Extracts patient features
/// 3. Generates patient commitment
/// 4. Builds authorization trace
/// 5. Generates ZKP proof
/// 6. Writes decision record to file
pub fn prove_command(args: ProveArgs) -> Result<()> {
    println!("🔐 Privacy-Preserving Medical Authorization - Proof Generation");
    println!();
    
    // ================================================================
    // Step 1: Load and parse policy
    // ================================================================
    println!("📋 Loading policy: {}", args.policy.display());
    let (policy, canonical_json) = PolicyParser::load_and_canonicalize(&args.policy)
        .context("Failed to load policy")?;
    
    println!("   Policy ID: {}", policy.policy_id);
    println!("   Version: {}", policy.version);
    println!("   LOB: {}", policy.lob);
    println!("   Codes: {}", policy.codes.join(", "));
    println!("   Requires PA: {}", policy.requires_pa);
    println!("   Inclusion criteria: {}", policy.inclusion.len());
    println!("   Exclusion criteria: {}", policy.exclusion.len());
    
    // ================================================================
    // Step 2: Compute policy hash
    // ================================================================
    let policy_hash = compute_policy_hash(&canonical_json);
    println!("   Policy hash: 0x{}", hex::encode(&policy_hash[0..8]));
    println!();
    
    // ================================================================
    // Step 3: Load and extract patient features
    // ================================================================
    println!("👤 Loading patient: {}", args.patient.display());
    let patient_record = PatientExtractor::from_file(&args.patient)
        .context("Failed to load patient")?;
    
    let features = PatientExtractor::extract_features(&patient_record)
        .context("Failed to extract patient features")?;
    
    println!("   Patient ID: {}", patient_record.patient_id);
    println!("   Age: {} years", features.age_years);
    println!("   Sex: {}", if features.sex == 1 { "F" } else { "M" });
    println!("   Primary ICD-10: {} (mapped to {})", 
             patient_record.icd10_list.first().unwrap_or(&"N/A".to_string()),
             features.primary_icd10);
    println!();
    
    // ================================================================
    // Step 4: Generate patient commitment
    // ================================================================
    println!("🔒 Generating patient commitment...");
    let salt = generate_random_salt();
    let patient_commitment = compute_patient_commitment(&features, &salt);
    println!("   Commitment: 0x{}", hex::encode(&patient_commitment[0..8]));
    println!("   (Patient data hidden behind commitment)");
    println!();
    
    // ================================================================
    // Step 5: Build authorization trace
    // ================================================================
    println!("⚙️  Building authorization trace...");
    let (trace, result) = TraceBuilder::build(
        &features,
        &policy,
        &salt,
        &policy_hash,
        &patient_commitment,
    ).context("Failed to build authorization trace")?;
    
    println!("   Trace rows: {}", trace.len());
    println!("   Authorization result: {}", result.to_string());
    println!();
    
    // ================================================================
    // Step 6: Setup ZKP prover
    // ================================================================
    println!("🔬 Setting up ZKP prover...");
    let k = 6;  // 6 registers per row
    let rows = trace.len();
    let n = rows.next_power_of_two();
    
    // Get root of unity for domain
    use ark_ff::FftField;
    let omega = F::get_root_of_unity(n as u64)
        .ok_or_else(|| anyhow::anyhow!("Failed to get root of unity for n={}", n))?;
    
    let domain = Domain {
        n,
        omega,
        zh_c: F::from(1u64),
    };
    
    let air = AirSpec {
        k,
        id_table: vec![],
        sigma_table: vec![],
        selectors: vec![],
    };
    
    let prover = ProverBuilder::new(domain, air)
        .b_blk(128)  // Streaming block size
        .build();
    
    println!("   Domain size: {} (next power of 2 from {})", n, rows);
    println!("   Registers (k): {}", k);
    println!();
    
    // ================================================================
    // Step 7: Generate ZKP proof
    // ================================================================
    println!("🔐 Generating ZKP proof...");
    println!("   (This may take a few seconds...)");
    
    let proof = prove_from_rows(&prover, trace)
        .context("Failed to generate ZKP proof")?;
    
    println!("   ✓ Proof generated successfully!");
    println!();
    
    // ================================================================
    // Step 8: Create decision record
    // ================================================================
    println!("📝 Creating decision record...");
    let decision_record = DecisionRecord::new(
        &policy,
        &policy_hash,
        &patient_commitment,
        result,
        &proof,
    ).context("Failed to create decision record")?;
    
    // ================================================================
    // Step 9: Write to file
    // ================================================================
    decision_record.to_file(&args.out)
        .with_context(|| format!("Failed to write decision record to {}", args.out.display()))?;
    
    println!("   ✓ Decision record written to: {}", args.out.display());
    println!();
    
    // ================================================================
    // Summary
    // ================================================================
    println!("✅ PROOF GENERATION COMPLETE");
    println!();
    println!("Summary:");
    println!("  Policy: {} (v{})", policy.policy_id, policy.version);
    println!("  Code: {}", policy.codes.first().unwrap_or(&"N/A".to_string()));
    println!("  Authorization Result: {}", result.to_string());
    println!("  Output: {}", args.out.display());
    println!();
    println!("What was proven:");
    println!("  ✓ Policy hash: 0x{}...", hex::encode(&policy_hash[0..4]));
    println!("  ✓ Patient commitment: 0x{}...", hex::encode(&patient_commitment[0..4]));
    println!("  ✓ Result: {}", result.to_string());
    println!();
    println!("What was NOT revealed:");
    println!("  ✗ Patient age, sex, diagnoses, or any other features");
    println!("  ✗ Intermediate evaluation steps");
    println!("  ✗ Why the decision was made (only the outcome)");
    println!();
    
    Ok(())
}

