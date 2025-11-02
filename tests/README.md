# Integration Test Suite

This directory contains comprehensive integration tests for production-critical paths in the SSZKP system that aren't covered by the basic CLI tests.

## Overview

The integration test suite validates:

1. **API Builders** (`api_builders.rs`) - ProverBuilder/VerifierBuilder ergonomics
2. **CSV Streaming** (`csv_streaming.rs`) - CsvRows adapter and file-based witness streaming
3. **Real Permutations** (`permutations.rs`) - Copy constraints with non-identity permutation tables
4. **Proof I/O** (`proof_io.rs`) - Serialization/deserialization edge cases
5. **Memory Diagnostics** - SSZKP_MEMLOG and SSZKP_BLOCKED_IFFT modes
6. **Large Register Counts** - Testing with k ∈ {1, 16, 32} registers

## Running Tests

### Run All Integration Tests

```bash
./scripts/test_sszkp_integration.sh
```

### Run Specific Test Suite

```bash
# API builders only
cargo test --features dev-srs --test api_builders

# CSV streaming only
cargo test --features dev-srs --test csv_streaming

# Permutation arguments only
cargo test --features dev-srs --test permutations

# Proof I/O only
cargo test --features dev-srs --test proof_io
```

### Run Individual Test

```bash
cargo test --features dev-srs --test api_builders test_1_1_builder_basic_usage
```

### Run with Verbose Output

```bash
cargo test --features dev-srs --test csv_streaming -- --show-output
```

### Run Slow Tests

Some tests are marked `#[ignore]` because they're slow. Run them explicitly:

```bash
# Large CSV test (100K rows)
cargo test --features dev-srs --test csv_streaming test_2_4 -- --ignored --show-output
```

Or enable via environment variable:

```bash
SSZKP_RUN_SLOW_TESTS=1 ./scripts/test_sszkp_integration.sh
```

## Test Coverage

### Section 1: API Builders (4 tests)

| Test | Purpose |
|------|---------|
| 1.1 | Basic ProverBuilder/VerifierBuilder usage |
| 1.2 | Custom basis and b_blk settings |
| 1.3 | Mismatched prover/verifier basis (header is authoritative) |
| 1.4 | Invalid domain handling (non-power-of-2) |

**Why it matters**: These are the primary user-facing APIs documented in the README. The CLI tests bypass these builders.

### Section 2: CSV Streaming (10 tests)

| Test | Purpose |
|------|---------|
| 2.1 | Basic CSV parsing with commas |
| 2.2 | CSV with space delimiters |
| 2.3 | CSV with mixed delimiters |
| 2.4 | Large CSV (100K rows) with memory validation |
| 2.5a | Wrong column count error handling |
| 2.5b | Non-numeric data error handling |
| 2.5c | Missing file error handling |
| 2.5d | Empty file handling |
| 2.6 | Re-streaming correctness (multiple passes) |

**Why it matters**: CSV streaming is the core value proposition for "billions of rows" claims. Must validate true O(√N) memory behavior.

### Section 3: Real Permutation Arguments (4 tests)

| Test | Purpose |
|------|---------|
| 3.1 | Simple 2-register copy constraint |
| 3.2 | Multi-column permutation (4 registers) |
| 3.3 | Permutation soundness (reject violations) |
| 3.4 | Large permutation table (k=32) |

**Why it matters**: Existing tests only use identity permutations. This validates the actual Plonk-style copy constraint logic.

### Section 4: Proof I/O (6 tests)

| Test | Purpose |
|------|---------|
| 4.1 | Round-trip: write → read → verify |
| 4.2 | Corrupt magic bytes (graceful error) |
| 4.3 | Wrong version number (graceful error) |
| 4.4 | Truncated file (deserialization error) |
| 4.5 | Large proof with selectors |
| 4.6 | File permissions error |

**Why it matters**: Robust I/O is critical for the tinyzkp_api. Users will encounter corrupted files and edge cases.

### Section 5: Memory Diagnostics (4 tests)

| Test | Purpose |
|------|---------|
| 5.1 | SSZKP_MEMLOG output validation |
| 5.2 | SSZKP_BLOCKED_IFFT tape mode |
| 5.3 | Empirical memory measurement (optional) |
| 5.4 | Memory stability across phases |

**Why it matters**: These are documented features. Must validate O(√N) memory claims empirically.

### Section 6: Large Register Counts (6 tests)

| Test | Purpose |
|------|---------|
| 6.1 | k=16 baseline |
| 6.2 | k=32 (production scale, matches README) |
| 6.3 | k=64 extreme (stress test) |
| 6.4 | k=1 edge case |
| 6.5 | k mismatch error handling |
| 6.6 | Memory scaling with k |

**Why it matters**: Existing tests max out at k=4. The 16M row example uses k=32, which must be validated.

## Expected Failures

Some tests are designed to fail (soundness checks):

- `test_2_5a_csv_wrong_column_count` - `#[should_panic]`
- `test_2_5b_csv_non_numeric_data` - `#[should_panic]`
- `test_3_3_permutation_soundness_check` - `#[should_panic]`

These validate error handling and constraint enforcement.

## Environment Variables

### Test Behavior

- `SSZKP_RUN_SLOW_TESTS=1` - Enable slow tests (large CSV, etc.)
- `SSZKP_MEASURE_MEMORY=1` - Enable empirical memory measurement

### Memory Diagnostics (tested by suite)

- `SSZKP_MEMLOG=1` - Enable memory usage logging
- `SSZKP_BLOCKED_IFFT=1` - Enable disk-tape blocked IFFT

## CI Integration

Add to `.github/workflows/test.yml`:

```yaml
- name: Run Integration Tests
  run: ./scripts/test_sszkp_integration.sh
```

## Debugging Failed Tests

### Verbose Output

```bash
cargo test --features dev-srs --test permutations -- --show-output
```

### Run Single Test

```bash
cargo test --features dev-srs --test api_builders test_1_2_builder_default_overrides -- --show-output
```

### Check Generated Files

Some tests create temporary CSV files or proof files. Check the workspace root if cleanup fails.

### Linting

```bash
cargo clippy --features dev-srs --tests
```

## Maintenance

### Adding New Tests

1. Create test in appropriate file (`api_builders.rs`, etc.)
2. Follow naming convention: `test_<section>_<number>_<description>`
3. Add docstring explaining purpose
4. Mark slow tests with `#[ignore]`
5. Mark expected-failure tests with `#[should_panic]`

### Test Template

```rust
#[test]
fn test_X_Y_descriptive_name() {
    // Test X.Y: Brief description
    // Why this test matters
    
    // Setup
    let domain = /* ... */;
    
    // Execute
    let result = /* ... */;
    
    // Assert
    assert!(result.is_ok(), "Should succeed because...");
}
```

## Performance

Typical run times:

- **API Builders**: ~5s
- **CSV Streaming** (basic): ~10s
- **CSV Streaming** (with large test): ~60s
- **Permutations**: ~8s
- **Proof I/O**: ~6s
- **Memory Diagnostics**: ~15s
- **Large k**: ~20s

**Total** (without slow tests): ~64 seconds
**Total** (with slow tests): ~124 seconds

## Troubleshooting

### "No such file or directory" Errors

Tests create temporary files in the workspace root. Ensure you run from the correct directory:

```bash
cd /path/to/my-zkp
./scripts/test_sszkp_integration.sh
```

### Permission Errors (Test 4.6)

The file permissions test might behave differently on different filesystems. It's designed to handle this gracefully.

### Memory Measurement Not Available (Test 5.3)

Requires `/usr/bin/time` (available on macOS/Linux). Set `SSZKP_MEASURE_MEMORY=1` to enable.

## Related Documentation

- [test_sszkp.sh](../scripts/test_sszkp.sh) - Basic CLI smoke tests
- [test_sszkp_extended.sh](../scripts/test_sszkp_extended.sh) - Extended protocol tests
- [README.md](../README.md) - Main project documentation

## Success Metrics

Integration test suite is considered successful if:

- ✅ All 30+ test cases pass
- ✅ At least 10 explicit failure-mode tests
- ✅ Memory diagnostics validated
- ✅ Production configurations tested (k=32, large CSVs)
- ✅ Clear error messages for all failure cases

## Questions?

See the comprehensive test plan in `/revise-readme.plan.md` for detailed rationale and design decisions.

