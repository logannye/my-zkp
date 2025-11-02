//! Privacy-Preserving Medical Authorization CLI
//!
//! Command-line interface for generating and verifying zero-knowledge proofs
//! of medical claim authorization decisions.

use clap::{Parser, Subcommand};
use anyhow::Result;

mod icd_map;
mod patient;
mod policy;
mod criterion;
mod commitment;
mod decision;
mod trace;
mod cli;

#[derive(Parser)]
#[command(
    name = "authz",
    version = "0.1.0",
    about = "Privacy-preserving medical authorization with zero-knowledge proofs",
    long_about = "\
Privacy-Preserving Medical Authorization System

This tool uses zero-knowledge proofs to prove that medical claim authorization
decisions follow published payer rules without exposing patient medical data.

The system provides three guarantees:
  • Integrity: Decisions follow published, verifiable rules
  • Privacy: No PHI exposure beyond the outcome
  • Auditability: Every decision can be re-verified anytime

Examples:
  # Generate a proof for a claim
  authz prove \\
    --policy policies/UHC-COMM-BIOPSY-001.json \\
    --patient patients/p002-needs-pa.json \\
    --code 19081 \\
    --lob commercial \\
    --out out/p002_biopsy_proof.json

  # Verify a proof
  authz verify out/p002_biopsy_proof.json
"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a ZKP proof for an authorization decision
    ///
    /// This command loads a policy and patient record, evaluates the authorization
    /// logic, and generates a cryptographic proof. The proof can be verified by
    /// anyone without exposing the patient's medical data.
    Prove(cli::ProveArgs),
    
    /// Verify a ZKP proof
    ///
    /// This command verifies a previously generated authorization proof. Verification
    /// confirms that the claimed authorization decision is correct according to the
    /// published policy, without revealing any patient medical information.
    Verify(cli::VerifyArgs),
}

fn main() -> Result<()> {
    // Set up better error messages
    if let Err(err) = run() {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Prove(args) => cli::prove_command(args),
        Commands::Verify(args) => cli::verify_command(args),
    }
}

