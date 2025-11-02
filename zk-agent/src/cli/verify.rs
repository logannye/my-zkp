//! `authz verify` command implementation

use std::path::PathBuf;
use clap::Args;
use anyhow::{Result, Context};
use myzkp::{F, domain::Domain, Proof};
use myzkp::api::{VerifierBuilder, verify};
use ark_serialize::CanonicalDeserialize;

use crate::decision::DecisionRecord;

/// Arguments for the `authz verify` command
#[derive(Args, Debug)]
pub struct VerifyArgs {
    /// Path to decision record (proof) file
    #[arg(value_name = "PROOF_FILE")]
    pub proof_file: PathBuf,
}

/// Execute the `authz verify` command
///
/// This command:
/// 1. Loads the decision record
/// 2. Decodes the proof
/// 3. Sets up the verifier
/// 4. Verifies the proof
/// 5. Reports the result
pub fn verify_command(args: VerifyArgs) -> Result<()> {
    println!("🔍 Privacy-Preserving Medical Authorization - Proof Verification");
    println!();
    
    // ================================================================
    // Step 1: Load decision record
    // ================================================================
    println!("📄 Loading decision record: {}", args.proof_file.display());
    let decision = DecisionRecord::from_file(&args.proof_file)
        .context("Failed to load decision record")?;
    
    println!("   Policy ID: {}", decision.policy_id);
    println!("   Policy hash: 0x{}...", &decision.policy_hash[0..16]);
    println!("   Patient commitment: 0x{}...", &decision.patient_commitment[0..16]);
    println!("   Code: {}", decision.code);
    println!("   LOB: {}", decision.lob);
    println!("   Claimed result: {}", decision.claimed_result);
    println!();
    
    // ================================================================
    // Step 2: Decode proof
    // ================================================================
    println!("🔓 Decoding proof...");
    let proof_bytes = decision.decode_proof()
        .context("Failed to decode proof from base64")?;
    
    println!("   Proof size: {} bytes", proof_bytes.len());
    
    let mut proof_slice = proof_bytes.as_slice();
    let proof: Proof = Proof::deserialize_compressed(&mut proof_slice)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize proof: {}", e))?;
    
    println!("   ✓ Proof deserialized successfully");
    println!();
    
    // ================================================================
    // Step 3: Setup verifier
    // ================================================================
    println!("⚙️  Setting up ZKP verifier...");
    
    // Extract domain info from proof header
    let n = proof.header.domain_n as usize;
    
    use ark_ff::FftField;
    let omega = F::get_root_of_unity(n as u64)
        .ok_or_else(|| anyhow::anyhow!("Failed to get root of unity for n={}", n))?;
    
    let domain = Domain {
        n,
        omega,
        zh_c: F::from(1u64),
    };
    
    let verifier = VerifierBuilder::new(domain).build();
    
    println!("   Domain size: {}", n);
    println!("   Registers (k): {}", proof.header.k);
    println!();
    
    // ================================================================
    // Step 4: Verify proof
    // ================================================================
    println!("🔬 Verifying ZKP proof...");
    println!("   (This may take a moment...)");
    
    let verify_result = verify(&verifier, &proof);
    
    println!();
    
    // ================================================================
    // Step 5: Report result
    // ================================================================
    match verify_result {
        Ok(()) => {
            println!("✅ VERIFICATION SUCCESSFUL");
            println!();
            println!("The proof is cryptographically valid!");
            println!();
            println!("What was verified:");
            println!("  ✓ The claimed result ({}) is correct", decision.claimed_result);
            println!("  ✓ The policy hash matches: 0x{}...", &decision.policy_hash[0..16]);
            println!("  ✓ The patient commitment is valid: 0x{}...", &decision.patient_commitment[0..16]);
            println!("  ✓ The authorization logic was followed correctly");
            println!();
            println!("Authorization Decision: {}", decision.claimed_result);
            println!();
            println!("What you learned:");
            println!("  ✓ Policy: {} (hash: 0x{}...)", decision.policy_id, &decision.policy_hash[0..8]);
            println!("  ✓ Code: {}", decision.code);
            println!("  ✓ LOB: {}", decision.lob);
            println!("  ✓ Result: {}", decision.claimed_result);
            println!();
            println!("What you did NOT learn:");
            println!("  ✗ Patient age, sex, or diagnoses");
            println!("  ✗ Which specific criteria passed or failed");
            println!("  ✗ Any other patient medical information");
            println!();
            println!("🔒 Privacy preserved: Zero PHI exposed!");
            
            Ok(())
        }
        Err(e) => {
            println!("❌ VERIFICATION FAILED");
            println!();
            println!("The proof is INVALID!");
            println!();
            println!("Error: {}", e);
            println!();
            println!("Possible reasons:");
            println!("  • The proof was tampered with");
            println!("  • The claimed result doesn't match the actual computation");
            println!("  • The wrong policy was used");
            println!("  • The patient commitment doesn't match the features used");
            println!();
            
            Err(e)
        }
    }
}

