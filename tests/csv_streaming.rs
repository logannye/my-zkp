// tests/csv_streaming.rs
//! Integration tests for CSV streaming adapter (CsvRows)
//!
//! Section 2 of integration test suite: validates real-world CSV patterns

use ark_ff::FftField;
use myzkp::{
    api::{self, adapters::CsvRows, ProverBuilder, VerifierBuilder},
    air::AirSpec,
    domain::Domain,
    F,
};
use std::fs;
use std::io::Write;

/// Helper: generate test CSV file
fn generate_csv(path: &str, rows: usize, cols: usize, delimiter: &str) {
    let mut file = fs::File::create(path).expect("create CSV file");
    for i in 0..rows {
        let row: Vec<String> = (0..cols)
            .map(|c| ((i * cols + c) as u64).to_string())
            .collect();
        writeln!(file, "{}", row.join(delimiter)).expect("write CSV row");
    }
}

#[test]
fn test_2_1_basic_csv_parsing() {
    // Test 2.1: Basic CSV Parsing
    // Generate simple CSV, prove via CsvRows
    
    let csv_path = "test_basic.csv";
    let rows: usize = 1024;
    let k = 3;
    
    // Generate test CSV
    generate_csv(csv_path, rows, k, ",");
    
    // Setup domain
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(128).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    // Prove via CsvRows
    let csv_witness = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    let proof = api::prove_from_stream(&prover, &csv_witness).expect("proof generation failed");
    
    api::verify(&verifier, &proof).expect("verification failed");
    
    // Cleanup
    let _ = fs::remove_file(csv_path);
}

#[test]
fn test_2_2_csv_whitespace_delimiters() {
    // Test 2.2: CSV with Whitespace Delimiters
    // CSV using spaces instead of commas
    
    let csv_path = "test_whitespace.csv";
    let rows: usize = 1024;
    let k = 3;
    
    // Generate CSV with space delimiters
    generate_csv(csv_path, rows, k, " ");
    
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(128).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    let csv_witness = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    let proof = api::prove_from_stream(&prover, &csv_witness).expect("proof generation failed");
    
    api::verify(&verifier, &proof).expect("verification failed");
    
    let _ = fs::remove_file(csv_path);
}

#[test]
fn test_2_3_csv_mixed_delimiters() {
    // Test 2.3: CSV with Mixed Delimiters
    // Tabs, multiple spaces, mixed with commas
    
    let csv_path = "test_mixed.csv";
    let rows: usize = 100;
    let k = 3;
    
    // Generate CSV with mixed delimiters
    let mut file = fs::File::create(csv_path).expect("create CSV");
    for i in 0..rows {
        // Mix commas, spaces, and tabs
        writeln!(file, "{}, {}   \t{}", i * 3, i * 3 + 1, i * 3 + 2).expect("write");
    }
    drop(file);
    
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(64).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    let csv_witness = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    let proof = api::prove_from_stream(&prover, &csv_witness).expect("proof generation failed");
    
    api::verify(&verifier, &proof).expect("verification failed");
    
    let _ = fs::remove_file(csv_path);
}

#[test]
#[ignore] // Slow test - run explicitly with --ignored
fn test_2_4_large_csv_streaming_validation() {
    // Test 2.4: Large CSV (Streaming Validation)
    // 100K row CSV, prove with small b_blk to validate true streaming
    
    let csv_path = "test_large.csv";
    let rows = 100_000;
    let k = 3;
    
    println!("Generating {}K row CSV...", rows / 1000);
    generate_csv(csv_path, rows, k, ",");
    
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    // Small b_blk to test streaming (256 × 3 × 32 = 24KB working set)
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(256).build();
    let verifier = VerifierBuilder::new(domain).build();
    
    let csv_witness = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    
    println!("Proving with b_blk=256 (streaming mode)...");
    let proof = api::prove_from_stream(&prover, &csv_witness).expect("proof generation failed");
    
    println!("Verifying...");
    api::verify(&verifier, &proof).expect("verification failed");
    
    let _ = fs::remove_file(csv_path);
    println!("Large CSV test passed!");
}

#[test]
#[should_panic(expected = "expected k=3")]
fn test_2_5a_csv_wrong_column_count() {
    // Test 2.5a: Wrong column count
    // Should error with clear message
    
    let csv_path = "test_wrong_cols.csv";
    let k = 3;
    let rows: usize = 4;
    
    let mut file = fs::File::create(csv_path).expect("create CSV");
    writeln!(file, "1,2,3").expect("write");
    writeln!(file, "4,5").expect("write"); // Missing column
    writeln!(file, "6,7,8").expect("write");
    writeln!(file, "9,10,11").expect("write");
    drop(file);
    
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    let prover = ProverBuilder::new(domain, air).b_blk(4).build();
    
    let csv_witness = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    
    // This should panic when it tries to parse row 2 with wrong column count
    let _ = api::prove_from_stream(&prover, &csv_witness);
    let _ = fs::remove_file(csv_path);
}

#[test]
#[should_panic(expected = "parse")]
fn test_2_5b_csv_non_numeric_data() {
    // Test 2.5b: Non-numeric data
    // Should error with parse error
    
    let csv_path = "test_non_numeric.csv";
    let k = 3;
    
    let mut file = fs::File::create(csv_path).expect("create CSV");
    writeln!(file, "1,2,3").expect("write");
    writeln!(file, "4,abc,6").expect("write"); // Invalid number
    drop(file);
    
    let csv = CsvRows::new_from_path(csv_path, k).expect("create CSV adapter");
    
    // Error should occur during streaming (when parsing rows)
    let n = 1024;
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    let prover = ProverBuilder::new(domain, air).b_blk(64).build();
    
    let _ = api::prove_from_stream(&prover, &csv).expect("should error on parse");
    let _ = fs::remove_file(csv_path);
}

#[test]
fn test_2_5c_csv_file_not_found() {
    // Test 2.5c: File doesn't exist
    // Should return clear error message
    
    let result = CsvRows::new_from_path("nonexistent.csv", 3);
    assert!(result.is_err(), "Should error on missing file");
    
    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(err_msg.contains("nonexistent.csv") || err_msg.contains("No such file"),
                "Error should mention file name: {}", err_msg);
    }
}

#[test]
fn test_2_5d_csv_empty_file() {
    // Test 2.5d: Empty file
    // Should handle gracefully (zero rows)
    
    let csv_path = "test_empty.csv";
    fs::File::create(csv_path).expect("create empty CSV");
    
    let csv = CsvRows::new_from_path(csv_path, 3).expect("load empty CSV");
    
    // Should have zero rows
    use myzkp::stream::Restreamer;
    assert_eq!(csv.len_rows(), 0, "Empty CSV should have 0 rows");
    
    let _ = fs::remove_file(csv_path);
}

#[test]
fn test_2_6_csv_restreaming_correctness() {
    // Test 2.6: CSV Re-streaming Correctness
    // Multiple proofs from same CSV file should be identical
    
    let csv_path = "test_restream.csv";
    let rows = 512;
    let k = 3;
    
    generate_csv(csv_path, rows, k, ",");
    
    let n = rows.next_power_of_two();
    let omega = F::get_root_of_unity(n as u64).expect("root of unity");
    let domain = Domain { n, omega, zh_c: F::from(1u64) };
    
    let air = AirSpec { k, id_table: vec![], sigma_table: vec![], selectors: vec![] };
    
    let prover = ProverBuilder::new(domain.clone(), air).b_blk(128).build();
    
    let csv = CsvRows::new_from_path(csv_path, k).expect("load CSV");
    
    // Generate two proofs from same CSV
    let proof1 = api::prove_from_stream(&prover, &csv).expect("first proof failed");
    let proof2 = api::prove_from_stream(&prover, &csv).expect("second proof failed");
    
    // Proofs should be identical (deterministic witness)
    assert_eq!(proof1.q_comm, proof2.q_comm, "Quotient commitments should match");
    assert_eq!(proof1.wire_comms, proof2.wire_comms, "Wire commitments should match");
    
    let _ = fs::remove_file(csv_path);
}

