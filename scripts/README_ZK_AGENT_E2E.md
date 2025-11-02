# ZK-Agent End-to-End Test Script

## Overview

`test_zk_agent_e2e.sh` is a comprehensive test suite that validates the complete workflow of the privacy-preserving medical authorization system built on the `zk-agent` crate.

## What It Tests

### 1. **Environment & Build (Section 0)**
- Rust toolchain availability
- zk-agent build in release mode
- Binary existence and location
- Required directory structure (policies/, patients/, out/)

### 2. **Policy File Validation (Section 1)**
- Parsing all policy JSON files
- Computing policy hashes for auditability
- Validating policy structure completeness
- Displaying policy metadata (ID, version, codes, PA requirement, criteria counts)

### 3. **Patient File Validation (Section 2)**
- Parsing all patient JSON files
- Verifying feature extraction readiness
- Simulating patient commitment hashes
- Displaying patient metadata (age, sex, diagnoses, etc.)

### 4. **Proof Generation (Section 4)** - Core Functionality
Tests all authorization outcomes:
- **Test 4.1**: APPROVE (auto-approval, no PA required)
- **Test 4.2**: NEEDS_PA (criteria met, but PA required)
- **Test 4.3**: DENY (age inclusion criterion fails)
- **Test 4.4**: DENY (exclusion criterion triggered - pregnancy)
- **Test 4.5**: DENY (admin rule fails - wrong place of service)
- **Test 4.6**: DENY (admin rule fails - exceeds max units)

### 5. **Proof Verification (Section 5)**
- Verifies all proofs generated in Section 4
- Validates ZKP verifier correctness
- Displays policy hash and patient commitment for each proof

### 6. **Privacy Guarantees (Section 6)**
- **Test 6.1**: Confirms decision records contain NO PHI (Protected Health Information)
  - Verifies absence of: age, sex, DOB, diagnoses
  - Confirms only public fields present: policy_hash, patient_commitment, claimed_result, proof
- **Test 6.2**: Validates proof sizes are reasonable (< 50 KB)

### 7. **Error Handling (Section 7)**
- **Test 7.1**: Malformed policy JSON (graceful error handling)
- **Test 7.2**: Invalid patient file (missing required fields)
- **Test 7.3**: Tampered proof verification (soundness check)

### 8. **Cross-Policy Tests (Section 8)**
- **Test 8.1**: Same patient against multiple policies (shows policy-dependent outcomes)
- **Test 8.2**: Policy version tracking (ensures different versions produce different hashes)

## Usage

### Basic Run
```bash
./scripts/test_zk_agent_e2e.sh
```

### Verbose Output (if implemented)
```bash
VERBOSE=1 ./scripts/test_zk_agent_e2e.sh
```

### Run Specific Section (if implemented)
```bash
TEST_SECTION=4 ./scripts/test_zk_agent_e2e.sh  # Only proof generation
```

## Prerequisites

1. **Rust toolchain** (cargo, rustc)
2. **jq** - JSON processor
   - macOS: `brew install jq`
   - Linux: `apt-get install jq` or `yum install jq`
3. **Dummy data files**:
   - Policy files in `policies/`
   - Patient files in `patients/`
   - Output directory `out/`

## Exit Codes

- **0**: All tests passed
- **1**: One or more tests failed

## Output Interpretation

### Color Coding
- **Green (✓)**: Test passed
- **Red (✗)**: Test failed
- **Blue (→)**: Informational message
- **Yellow**: Section headers

### Test Format
```
Test X.Y: Test Description
────────────────────────────────────────────────────────
→ Context information...
→ Expected result...

✓ PASS: Specific success message
or
✗ FAIL: Specific failure message with details
```

### Summary Section
At the end, you'll see:
- Total test count
- Pass/fail breakdown
- Test coverage checklist
- Proof statistics (count, size)
- Overall result (ALL TESTS PASSED or X FAILED)

## Example Output

```
═══════════════════════════════════════════════════════
  ZK-Agent End-to-End Test Suite
═══════════════════════════════════════════════════════

Testing privacy-preserving medical authorization with zero-knowledge proofs

...

Test 4.2: NEEDS_PA Case (PA Required)
────────────────────────────────────────────────────────
→ Policy: UHC-COMM-BIOPSY-001
→ Patient: p002-needs-pa
→ Expected Result: NEEDS_PA

  Generating proof...
  Proof generated: out/test_42_proof.json
  Claimed result: NEEDS_PA

✓ PASS: Result matches expected: NEEDS_PA

...

═══════════════════════════════════════════════════════
  Test Summary
═══════════════════════════════════════════════════════

Test Execution Complete

Results:
  Total Tests:  25
  Passed:       25
  Failed:       0
  Duration:     45s

Test Coverage:
  ✓ Environment & build validation
  ✓ Policy parsing & structure validation
  ✓ Patient feature extraction readiness
  ✓ Proof generation (all outcome types)
  ✓ Proof verification
  ✓ Privacy guarantee checks
  ✓ Error handling
  ✓ Cross-policy scenarios

Proof Statistics:
  Files generated: 6
  Total size:      42K

═══════════════════════════════════════════════════════
Result: ALL TESTS PASSED ✓
═══════════════════════════════════════════════════════
```

## Troubleshooting

### "jq: command not found"
Install jq: `brew install jq` (macOS) or `apt-get install jq` (Linux)

### "authz binary not found"
Run: `cargo build --release --manifest-path zk-agent/Cargo.toml`

### "Policy/Patient files not found"
Ensure you're running from the project root and that `policies/` and `patients/` directories exist with JSON files.

### Proof generation or verification fails
- Check that the zk-agent crate builds successfully
- Verify that policy and patient JSON files are valid
- Ensure the myzkp engine is properly installed and configured

### Privacy test fails (PHI detected)
This is a critical failure - it means the decision record is exposing protected health information. Review the DecisionRecord struct in `zk-agent/src/decision.rs`.

## Integration with CI/CD

This script is designed to be CI/CD ready:
- Returns proper exit codes (0 = success, 1 = failure)
- Outputs structured, parseable results
- Runs in non-interactive mode
- Can be time-limited if needed

Example GitHub Actions usage:
```yaml
- name: Run ZK-Agent E2E Tests
  run: ./scripts/test_zk_agent_e2e.sh
  timeout-minutes: 10
```

## Development Workflow

1. **After code changes**: Run this script to catch regressions
2. **Before commits**: Ensure all tests pass
3. **Demo preparation**: Use this script to validate the system is working
4. **Hackathon judging**: Show this output to demonstrate comprehensive testing

## Notes

- Proof generation in release mode is significantly faster than debug mode
- First run may take longer due to initial compilation
- Generated proof files in `out/test_*_proof.json` are automatically cleaned up on exit
- The script uses `set -euo pipefail` for strict error handling

## Related Scripts

- `test_sszkp.sh` - Basic myzkp engine tests
- `test_sszkp_extended.sh` - Comprehensive myzkp engine tests
- `test_sszkp_integration.sh` - Integration tests for myzkp components
- `test_sszkp_memory.sh` - Memory efficiency validation
- `test_sszkp_performance.sh` - Performance benchmarks
- `test_sszkp_security.sh` - Cryptographic security validation

This script specifically tests the **zk-agent application layer** built on top of the myzkp engine.

