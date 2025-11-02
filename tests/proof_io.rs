// tests/proof_io.rs
//! Integration tests for proof I/O edge cases
//!
//! Section 4 of integration test suite: validates serialization/deserialization robustness

use ark_ff::{FftField, Field};
use myzkp::{
    api::{self, io, ProverBuilder, VerifierBuilder},
    air::{AirSpec, Row},
    domain::Domain,
    F,
};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Helper: generate simple witness
fn generate_witness(rows: usize, k: usize) -> Vec<Row> {
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
fn test_4_1_round_trip_correctness() {
    // Test 4.1: Round-Trip Correctness
    // Generate proof → write to file → read from file → verify
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(128).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    let witness = generate_witness(rows, k);
    
    // Generate proof
    let proof1 = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Write to file
    let proof_path = Path::new("test_roundtrip.bin");
    io::write_proof(proof_path, &proof1).expect("write proof failed");
    
    // Read from file
    let proof2 = io::read_proof(proof_path).expect("read proof failed");
    
    // Verify the read proof
    api::verify(&verifier, &proof2).expect("verification failed");
    
    // Check that fields match
    assert_eq!(proof1.header.domain_n, proof2.header.domain_n);
    assert_eq!(proof1.header.k, proof2.header.k);
    assert_eq!(proof1.q_comm, proof2.q_comm, "Quotient commitments should match");
    
    // Cleanup
    let _ = fs::remove_file(proof_path);
}

#[test]
fn test_4_2_wrong_magic_bytes() {
    // Test 4.2: Wrong Magic Bytes
    // Corrupt file header, should error gracefully
    
    let rows: usize = 512;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain, air).b_blk(128).build();
    let witness = generate_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Write proof
    let proof_path = Path::new("test_bad_magic.bin");
    io::write_proof(proof_path, &proof).expect("write proof failed");
    
    // Corrupt magic bytes
    let mut file = fs::OpenOptions::new()
        .write(true)
        .open(proof_path)
        .expect("open for corruption");
    file.write_all(b"BADMAGIC").expect("write bad magic");
    drop(file);
    
    // Try to read - should error
    let result = io::read_proof(proof_path);
    assert!(result.is_err(), "Should error on bad magic bytes");
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("magic") || err_msg.contains("bad proof"),
            "Error should mention magic bytes: {}", err_msg);
    
    let _ = fs::remove_file(proof_path);
}

#[test]
fn test_4_3_wrong_version_number() {
    // Test 4.3: Wrong Version Number
    // Set version byte to unsupported value
    
    let rows: usize = 512;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain, air).b_blk(128).build();
    let witness = generate_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Write proof
    let proof_path = Path::new("test_bad_version.bin");
    io::write_proof(proof_path, &proof).expect("write proof failed");
    
    // Corrupt version bytes (offset 8-9)
    let mut data = fs::read(proof_path).expect("read file");
    data[8] = 0;
    data[9] = 99; // Version 99
    fs::write(proof_path, data).expect("write corrupted file");
    
    // Try to read - should error
    let result = io::read_proof(proof_path);
    assert!(result.is_err(), "Should error on unsupported version");
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("version") || err_msg.contains("99"),
            "Error should mention version: {}", err_msg);
    
    let _ = fs::remove_file(proof_path);
}

#[test]
fn test_4_4_truncated_proof_file() {
    // Test 4.4: Truncated Proof File
    // Cut off last 100 bytes, should error gracefully
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain, air).b_blk(128).build();
    let witness = generate_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Write proof
    let proof_path = Path::new("test_truncated.bin");
    io::write_proof(proof_path, &proof).expect("write proof failed");
    
    // Truncate file
    let mut data = fs::read(proof_path).expect("read file");
    if data.len() > 100 {
        data.truncate(data.len() - 100);
        fs::write(proof_path, data).expect("write truncated file");
    }
    
    // Try to read - should error
    let result = io::read_proof(proof_path);
    assert!(result.is_err(), "Should error on truncated file");
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("deserialize") || err_msg.contains("EOF") || err_msg.contains("unexpected end"),
            "Error should mention deserialization issue: {}", err_msg);
    
    let _ = fs::remove_file(proof_path);
}

#[test]
fn test_4_5_large_proof_selector_heavy() {
    // Test 4.5: Large Proof (Selector-Heavy)
    // Proof with selector columns
    
    let rows: usize = 1024;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    // Create selectors (5 columns)
    let selectors: Vec<Box<[F]>> = (0..5)
        .map(|col| {
            (0..n).map(|row| F::from((row * 10 + col) as u64))
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect();
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(128).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    let witness = generate_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Write and read back
    let proof_path = Path::new("test_large_selectors.bin");
    io::write_proof(proof_path, &proof).expect("write proof failed");
    
    let file_size = fs::metadata(proof_path).expect("get metadata").len();
    println!("Proof with 5 selector columns: {} bytes", file_size);
    
    // Should be reasonable size (a few tens of KB)
    assert!(file_size < 100_000, "Proof size should be reasonable: {} bytes", file_size);
    
    let proof2 = io::read_proof(proof_path).expect("read proof failed");
    api::verify(&verifier, &proof2).expect("verification failed");
    
    let _ = fs::remove_file(proof_path);
}

#[test]
fn test_4_6_file_permissions_error() {
    // Test 4.6: File Permissions Error
    // Try to write to read-only location
    
    let rows: usize = 512;
    let k = 3;
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain, air).b_blk(128).build();
    let witness = generate_witness(rows, k);
    let proof = api::prove_from_rows(&prover, witness).expect("proof generation failed");
    
    // Create a read-only file
    let proof_path = Path::new("test_readonly.bin");
    let mut file = fs::File::create(proof_path).expect("create file");
    file.write_all(b"placeholder").expect("write");
    drop(file);
    
    // Make it read-only
    let mut perms = fs::metadata(proof_path).expect("get metadata").permissions();
    perms.set_readonly(true);
    fs::set_permissions(proof_path, perms).expect("set readonly");
    
    // Try to write - should error
    let result = io::write_proof(proof_path, &proof);
    
    // Note: On some systems, we might need to skip this test if permissions don't work as expected
    if result.is_err() {
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        println!("Got expected error: {}", err_msg);
        assert!(err_msg.contains("permission") || err_msg.contains("denied") || err_msg.contains("read-only"),
                "Error should mention permissions: {}", err_msg);
    } else {
        println!("Warning: Read-only file write didn't error (filesystem might not support)");
    }
    
    // Cleanup (restore write permission first)
    let mut perms = fs::metadata(proof_path).expect("get metadata").permissions();
    perms.set_readonly(false);
    fs::set_permissions(proof_path, perms).ok();
    let _ = fs::remove_file(proof_path);
}

