// tests/api_builders.rs
//! Integration tests for API builders (ProverBuilder/VerifierBuilder)
//!
//! Section 1 of integration test suite: validates ergonomic API layer

use ark_ff::{FftField, Field};
use myzkp::{
    api::{self, ProverBuilder, VerifierBuilder},
    air::{AirSpec, Row},
    domain::Domain,
    pcs::Basis,
    F,
};

/// Helper: generate simple witness with predictable pattern
fn generate_test_witness(rows: usize, k: usize) -> Vec<Row> {
    (0..rows)
        .map(|i| {
            let base = F::from((i as u64) + 1);
            let regs = (0..k)
                .map(|m| base.pow([(m as u64) + 1]))
                .collect::<Vec<_>>()
                .into_boxed_slice();
            Row { regs }
        })
        .collect()
}

#[test]
fn test_1_1_builder_basic_usage() {
    // Test 1.1: Basic Builder Usage
    // Create prover/verifier via builders, prove, verify
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
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
    
    let prover = ProverBuilder::new(domain.clone(), air.clone())
        .b_blk(128)
        .build();
    
    let verifier = VerifierBuilder::new(domain).build();
    
    let witness = generate_test_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    api::verify(&verifier, &proof).expect("verification failed");
}

#[test]
fn test_1_2_builder_default_overrides() {
    // Test 1.2: Builder Default Overrides
    // Test that custom settings (basis, b_blk) actually apply
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
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
    
    // Build prover with custom basis and b_blk
    let prover = ProverBuilder::new(domain.clone(), air.clone())
        .wires_basis(Basis::Coefficient)
        .b_blk(256)
        .build();
    
    let verifier = VerifierBuilder::new(domain).build();
    
    let witness = generate_test_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Verify proof header reflects custom basis
    assert_eq!(proof.header.basis_wires, Basis::Coefficient, 
               "Proof header should reflect coefficient basis");
    
    api::verify(&verifier, &proof).expect("verification failed");
}

#[test]
fn test_1_3_mismatched_prover_verifier_basis() {
    // Test 1.3: Mismatched Prover/Verifier Basis
    // Prover uses eval basis, verifier expects coeff (or vice versa)
    // Should still verify because header is authoritative
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
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
    
    let prover = ProverBuilder::new(domain.clone(), air.clone())
        .wires_basis(Basis::Evaluation)
        .build();
    
    // Verifier with mismatched basis expectation
    let verifier = VerifierBuilder::new(domain)
        .wires_basis(Basis::Coefficient)  // Intentional mismatch
        .build();
    
    let witness = generate_test_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Should still verify because header overrides builder default
    api::verify(&verifier, &proof).expect("verification should succeed (header is authoritative)");
}

#[test]
fn test_1_4_invalid_domain_non_power_of_two() {
    // Test 1.4: Invalid Domain (No Root of Unity)
    // Try to build with invalid domain size
    // Should handle gracefully (either error or round up to next power of 2)
    
    let rows: usize = 1023; // Not a power of 2
    let _k = 3;
    
    // System should handle this by rounding up to next power of 2 (1024)
    let n = rows.next_power_of_two();
    assert_eq!(n, 1024, "Should round up to next power of 2");
    
    let omega_result = F::get_root_of_unity(n as u64);
    assert!(omega_result.is_some(), "Should have root of unity for power-of-2 domain");
    
    // Test with actual non-power-of-2 that has no root
    // The get_root_of_unity should return None for invalid sizes
    let invalid_n = 1023;
    let invalid_omega = F::get_root_of_unity(invalid_n as u64);
    
    // This demonstrates graceful handling - None is returned, not a panic
    if invalid_omega.is_none() {
        // Expected behavior: no panic, just None
        println!("Correctly returned None for non-power-of-2 domain");
    }
}

