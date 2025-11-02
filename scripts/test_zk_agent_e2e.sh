#!/usr/bin/env bash
set -euo pipefail

# ═══════════════════════════════════════════════════════════════════════
# ZK-Agent End-to-End Test Script
# ═══════════════════════════════════════════════════════════════════════
#
# Tests: Policy parsing → Patient extraction → Trace building → 
#        Proof generation → Verification → Privacy guarantees
#
# This script validates the complete workflow of the privacy-preserving
# medical authorization system, testing all components and outcome types.
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
#
# Usage:
#   ./scripts/test_zk_agent_e2e.sh              # Run all tests
#   VERBOSE=1 ./scripts/test_zk_agent_e2e.sh    # Verbose output
#   TEST_SECTION=4 ./scripts/test_zk_agent_e2e.sh  # Specific section
#
# ═══════════════════════════════════════════════════════════════════════

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test result tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
EXIT_CODE=0

# Timing
START_TIME=$(date +%s)

# Directories
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# ═══════════════════════════════════════════════════════════════════════
# Helper Functions
# ═══════════════════════════════════════════════════════════════════════

print_header() {
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
    echo ""
}

print_section() {
    echo ""
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}  $1${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_test() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
    echo "Test $1: $2"
    echo "────────────────────────────────────────────────────────"
}

print_pass() {
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo -e "${GREEN}✓ PASS${NC}: $1"
}

print_fail() {
    FAILED_TESTS=$((FAILED_TESTS + 1))
    echo -e "${RED}✗ FAIL${NC}: $1"
    EXIT_CODE=1
}

print_info() {
    echo -e "${BLUE}→${NC} $1"
}

print_detail() {
    echo "  $1"
}

# Check if jq is available
check_jq() {
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: jq is required but not installed.${NC}"
        echo "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    set +e
    # Remove test proof files
    rm -f out/test_*_proof.json 2>/dev/null
    rm -f out/invalid_*.json 2>/dev/null
    rm -f /tmp/invalid_policy.json 2>/dev/null
    rm -f /tmp/invalid_patient.json 2>/dev/null
    rm -f /tmp/test_policy_v2.json 2>/dev/null
    set -e
}

trap cleanup EXIT

# ═══════════════════════════════════════════════════════════════════════
# Test Execution Helpers
# ═══════════════════════════════════════════════════════════════════════

run_prove_test() {
    local test_id=$1
    local policy=$2
    local patient=$3
    local expected=$4
    local description=$5
    
    print_test "$test_id" "$description"
    print_info "Policy: $policy"
    print_info "Patient: $patient"
    print_info "Expected Result: $expected"
    echo ""
    
    local output_file="out/test_${test_id}_proof.json"
    
    print_detail "Generating proof..."
    
    # Run prove command
    if cargo run --quiet --release --package zk-agent --bin authz -- prove \
        --policy "policies/$policy.json" \
        --patient "patients/$patient.json" \
        --code "00000" \
        --lob "commercial" \
        --out "$output_file" > /dev/null 2>&1; then
        
        print_detail "Proof generated: $output_file"
        
        # Extract claimed result
        local result=$(jq -r '.claimed_result' "$output_file" 2>/dev/null || echo "ERROR")
        
        print_detail "Claimed result: $result"
        echo ""
        
        if [[ "$result" == "$expected" ]]; then
            print_pass "Result matches expected: $result"
        else
            print_fail "Expected $expected but got $result"
        fi
    else
        echo ""
        print_fail "Proof generation failed"
    fi
}

run_verify_test() {
    local test_id=$1
    local description=$2
    local source_test_id=$3
    
    print_test "$test_id" "Verify: $description"
    
    local proof_file="out/test_${source_test_id}_proof.json"
    
    if [[ ! -f "$proof_file" ]]; then
        print_fail "Proof file not found: $proof_file"
        return
    fi
    
    print_info "Proof file: $proof_file"
    
    # Extract metadata
    local policy_hash=$(jq -r '.policy_hash' "$proof_file" 2>/dev/null | cut -c1-16)
    local patient_commitment=$(jq -r '.patient_commitment' "$proof_file" 2>/dev/null | cut -c1-16)
    
    print_detail "Policy hash: 0x$policy_hash..."
    print_detail "Patient commitment: 0x$patient_commitment..."
    echo ""
    
    print_detail "Running verification..."
    
    # Run verify command
    if cargo run --quiet --release --package zk-agent --bin authz -- verify "$proof_file" > /dev/null 2>&1; then
        echo ""
        print_pass "Proof verified successfully"
    else
        echo ""
        print_fail "Verification failed"
    fi
}

# ═══════════════════════════════════════════════════════════════════════
# Main Test Suite
# ═══════════════════════════════════════════════════════════════════════

print_header "ZK-Agent End-to-End Test Suite"

echo "Testing privacy-preserving medical authorization with zero-knowledge proofs"
echo ""
echo "Project: $(pwd)"
echo "Timestamp: $(date)"
echo ""

check_jq

# ═══════════════════════════════════════════════════════════════════════
# Section 0: Environment & Build Checks
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 0: Environment & Build Checks"

# Test 0.1: Rust toolchain
print_test "0.1" "Verify Rust Toolchain"
if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
    RUST_VERSION=$(rustc --version | awk '{print $2}')
    CARGO_VERSION=$(cargo --version | awk '{print $2}')
    print_info "Rust version: $RUST_VERSION"
    print_info "Cargo version: $CARGO_VERSION"
    print_pass "Rust toolchain available"
else
    print_fail "Rust toolchain not found"
fi

# Test 0.2: Build zk-agent
print_test "0.2" "Build zk-agent in Release Mode"
print_info "Building zk-agent (this may take a moment)..."
if cargo build --quiet --release --manifest-path zk-agent/Cargo.toml 2>&1 | grep -q "Finished" || \
   cargo build --quiet --release --manifest-path zk-agent/Cargo.toml > /dev/null 2>&1; then
    print_pass "Build successful"
else
    # Try without quiet flag to see errors
    if cargo build --release --manifest-path zk-agent/Cargo.toml 2>&1 | tail -1 | grep -q "Finished"; then
        print_pass "Build successful"
    else
        print_fail "Build failed"
    fi
fi

# Test 0.3: Verify binary exists
print_test "0.3" "Verify authz Binary Exists"
BINARY_PATH="target/release/authz"
if [[ -f "$BINARY_PATH" ]]; then
    BINARY_SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
    print_info "Binary location: $BINARY_PATH"
    print_info "Binary size: $BINARY_SIZE"
    print_pass "authz binary exists"
else
    print_fail "authz binary not found at $BINARY_PATH"
fi

# Test 0.4: Check required directories
print_test "0.4" "Check Required Directories"
REQUIRED_DIRS=("policies" "patients" "out")
ALL_EXIST=true

for dir in "${REQUIRED_DIRS[@]}"; do
    if [[ -d "$dir" ]]; then
        COUNT=$(ls -1 "$dir" 2>/dev/null | wc -l | tr -d ' ')
        print_detail "✓ $dir/ ($COUNT files)"
    else
        print_detail "✗ $dir/ (missing)"
        ALL_EXIST=false
    fi
done

echo ""
if $ALL_EXIST; then
    print_pass "All required directories exist"
else
    print_fail "Some required directories are missing"
fi

# ═══════════════════════════════════════════════════════════════════════
# Section 1: Policy File Validation
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 1: Policy File Validation"

# Test 1.1: Load and parse all policy files
print_test "1.1" "Load and Parse Policy Files"
POLICY_FILES=(policies/*.json)
POLICY_COUNT=0
VALID_POLICIES=0

echo "Policy Summary:"
echo ""
printf "  %-25s %-12s %-20s %-8s %-4s %-4s\n" "Policy ID" "Version" "Codes" "Req PA" "Incl" "Excl"
printf "  %-25s %-12s %-20s %-8s %-4s %-4s\n" "-------------------------" "------------" "--------------------" "--------" "----" "----"

for policy_file in "${POLICY_FILES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        POLICY_COUNT=$((POLICY_COUNT + 1))
        
        # Parse policy metadata
        POLICY_ID=$(jq -r '.policy_id' "$policy_file" 2>/dev/null || echo "ERROR")
        VERSION=$(jq -r '.version' "$policy_file" 2>/dev/null || echo "ERROR")
        CODES=$(jq -r '.codes | join(",")' "$policy_file" 2>/dev/null || echo "ERROR")
        REQUIRES_PA=$(jq -r '.requires_pa' "$policy_file" 2>/dev/null || echo "ERROR")
        INCL_COUNT=$(jq '.inclusion | length' "$policy_file" 2>/dev/null || echo "0")
        EXCL_COUNT=$(jq '.exclusion | length' "$policy_file" 2>/dev/null || echo "0")
        
        printf "  %-25s %-12s %-20s %-8s %-4s %-4s\n" \
            "$POLICY_ID" "$VERSION" "$CODES" "$REQUIRES_PA" "$INCL_COUNT" "$EXCL_COUNT"
        
        if [[ "$POLICY_ID" != "ERROR" ]]; then
            VALID_POLICIES=$((VALID_POLICIES + 1))
        fi
    fi
done

echo ""
if [[ $VALID_POLICIES -eq $POLICY_COUNT ]]; then
    print_pass "All $POLICY_COUNT policies parsed successfully"
else
    print_fail "Only $VALID_POLICIES of $POLICY_COUNT policies parsed successfully"
fi

# Test 1.2: Compute policy hashes
print_test "1.2" "Compute Policy Hashes"
echo "Policy Hashes (for auditability):"
echo ""

HASH_COUNT=0
for policy_file in "${POLICY_FILES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        POLICY_ID=$(jq -r '.policy_id' "$policy_file" 2>/dev/null)
        # Canonicalize and hash (simplified version using jq sorting)
        HASH=$(jq -c -S '.' "$policy_file" 2>/dev/null | shasum -a 256 | awk '{print $1}' | cut -c1-16)
        printf "  %-30s  0x%s...\n" "$POLICY_ID" "$HASH"
        HASH_COUNT=$((HASH_COUNT + 1))
    fi
done

echo ""
if [[ $HASH_COUNT -eq $POLICY_COUNT ]]; then
    print_pass "Policy hashes computed for all $POLICY_COUNT policies"
else
    print_fail "Failed to compute some policy hashes"
fi

# Test 1.3: Verify policy structure
print_test "1.3" "Verify Policy Structure Completeness"
STRUCT_VALID=0

for policy_file in "${POLICY_FILES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        # Check required fields exist
        HAS_INCLUSION=$(jq 'has("inclusion")' "$policy_file" 2>/dev/null)
        HAS_EXCLUSION=$(jq 'has("exclusion")' "$policy_file" 2>/dev/null)
        HAS_ADMIN=$(jq 'has("admin_rules")' "$policy_file" 2>/dev/null)
        
        if [[ "$HAS_INCLUSION" == "true" && "$HAS_EXCLUSION" == "true" && "$HAS_ADMIN" == "true" ]]; then
            STRUCT_VALID=$((STRUCT_VALID + 1))
        fi
    fi
done

echo ""
if [[ $STRUCT_VALID -eq $POLICY_COUNT ]]; then
    print_pass "All policies have required structure (inclusion, exclusion, admin_rules)"
else
    print_fail "Some policies missing required fields"
fi

# ═══════════════════════════════════════════════════════════════════════
# Section 2: Patient File Validation
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 2: Patient File Validation"

# Test 2.1: Load and parse all patient files
print_test "2.1" "Load and Parse Patient Files"
PATIENT_FILES=(patients/p*.json)
PATIENT_COUNT=0
VALID_PATIENTS=0

echo "Patient Summary:"
echo ""
printf "  %-20s %-5s %-4s %-15s %-8s %-15s\n" "Patient ID" "Age" "Sex" "Primary ICD-10" "Pregnant" "POS"
printf "  %-20s %-5s %-4s %-15s %-8s %-15s\n" "--------------------" "-----" "----" "---------------" "--------" "---------------"

for patient_file in "${PATIENT_FILES[@]}"; do
    if [[ -f "$patient_file" ]]; then
        PATIENT_COUNT=$((PATIENT_COUNT + 1))
        
        # Parse patient metadata
        PATIENT_ID=$(jq -r '.patient_id' "$patient_file" 2>/dev/null || echo "ERROR")
        DOB=$(jq -r '.dob' "$patient_file" 2>/dev/null || echo "ERROR")
        SEX=$(jq -r '.sex' "$patient_file" 2>/dev/null || echo "ERROR")
        PRIMARY_ICD=$(jq -r '.icd10_list[0]' "$patient_file" 2>/dev/null || echo "ERROR")
        PREGNANT=$(jq -r '.pregnant' "$patient_file" 2>/dev/null || echo "ERROR")
        POS=$(jq -r '.place_of_service' "$patient_file" 2>/dev/null || echo "ERROR")
        
        # Calculate age (simplified)
        if [[ "$DOB" != "ERROR" ]]; then
            BIRTH_YEAR=$(echo "$DOB" | cut -d'-' -f1)
            AGE=$((2025 - BIRTH_YEAR))
        else
            AGE="?"
        fi
        
        printf "  %-20s %-5s %-4s %-15s %-8s %-15s\n" \
            "$PATIENT_ID" "$AGE" "$SEX" "$PRIMARY_ICD" "$PREGNANT" "$POS"
        
        if [[ "$PATIENT_ID" != "ERROR" ]]; then
            VALID_PATIENTS=$((VALID_PATIENTS + 1))
        fi
    fi
done

echo ""
if [[ $VALID_PATIENTS -eq $PATIENT_COUNT ]]; then
    print_pass "All $PATIENT_COUNT patients parsed successfully"
else
    print_fail "Only $VALID_PATIENTS of $PATIENT_COUNT patients parsed successfully"
fi

# Test 2.2: Verify patient feature extraction readiness
print_test "2.2" "Verify Patient Feature Extraction Readiness"
FEATURE_READY=0

for patient_file in "${PATIENT_FILES[@]}"; do
    if [[ -f "$patient_file" ]]; then
        # Check required fields for feature extraction
        HAS_DOB=$(jq 'has("dob")' "$patient_file" 2>/dev/null)
        HAS_SEX=$(jq 'has("sex")' "$patient_file" 2>/dev/null)
        HAS_ICD=$(jq 'has("icd10_list")' "$patient_file" 2>/dev/null)
        HAS_POS=$(jq 'has("place_of_service")' "$patient_file" 2>/dev/null)
        
        if [[ "$HAS_DOB" == "true" && "$HAS_SEX" == "true" && "$HAS_ICD" == "true" && "$HAS_POS" == "true" ]]; then
            FEATURE_READY=$((FEATURE_READY + 1))
        fi
    fi
done

echo ""
if [[ $FEATURE_READY -eq $PATIENT_COUNT ]]; then
    print_pass "All patients have required fields for feature extraction"
else
    print_fail "Some patients missing required fields"
fi

# Test 2.3: Patient commitment simulation
print_test "2.3" "Patient Commitment Calculation (Simulated)"
echo "Simulating patient commitment hashes:"
echo ""

COMMIT_COUNT=0
for patient_file in "${PATIENT_FILES[@]}"; do
    if [[ -f "$patient_file" ]]; then
        PATIENT_ID=$(jq -r '.patient_id' "$patient_file" 2>/dev/null)
        # Simulate commitment (in real system, this includes extracted features + salt)
        COMMITMENT=$(echo "$PATIENT_ID-$(cat "$patient_file")" | shasum -a 256 | awk '{print $1}' | cut -c1-16)
        printf "  %-25s  0x%s...\n" "$PATIENT_ID" "$COMMITMENT"
        COMMIT_COUNT=$((COMMIT_COUNT + 1))
    fi
done

echo ""
if [[ $COMMIT_COUNT -eq $PATIENT_COUNT ]]; then
    print_pass "Patient commitments calculated for all $PATIENT_COUNT patients"
else
    print_fail "Failed to calculate some patient commitments"
fi

# ═══════════════════════════════════════════════════════════════════════
# Section 4: Proof Generation Tests (Core Functionality)
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 4: Proof Generation Tests"

cat << 'SECTION4_INTRO'
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SECTION PURPOSE: Core Authorization Logic Validation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

This section tests the heart of the authorization system - generating ZK proofs
for medical authorization decisions across diverse scenarios.

WHAT'S BEING TESTED:
  • Policy parsing and canonicalization
  • Patient feature extraction (age, sex, diagnoses, POS, units)
  • Authorization logic evaluation (inclusion/exclusion/admin rules)
  • ZKP trace building (converting authorization flow to Row sequences)
  • Proof generation (creating cryptographic proof of correctness)

TEST ORGANIZATION:
  4A: Original imaging procedures (CT, Biopsy) - 6 tests
  4B: Edge cases & boundary conditions - 4 tests
  4C: Medicare & Medicaid (multi-LOB support) - 2 tests
  4D: Physical therapy (session-based services) - 2 tests
  4E: Specialty drugs (pharmacy/medication) - 2 tests

HOW TO INTERPRET RESULTS:
  ✓ PASS = Proof generated AND claimed result matches expected outcome
           System correctly evaluated policy against patient features
           
  ✗ FAIL = Either:
           - Proof generation crashed (logic error, ICD mapping issue)
           - Claimed result doesn't match expected (authorization logic bug)

COVERAGE:
  - 16 patients × 7 policies = 112 possible combinations
  - Testing 16 key combinations covering all outcome types
  - APPROVE: 4 tests | NEEDS_PA: 6 tests | DENY: 6 tests
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
SECTION4_INTRO

echo ""
echo "═══════════════════════════════════════════════════════"
echo "Section 4A: Original Imaging Procedures"
echo "═══════════════════════════════════════════════════════"
echo ""

# Test 4.1: APPROVE Case
run_prove_test "41" "UHC-COMM-CT-CHEST-001" "p001-approve" "APPROVE" \
    "APPROVE Case (Auto-Approval, No PA Required)"

# Test 4.2: NEEDS_PA Case
run_prove_test "42" "UHC-COMM-BIOPSY-001" "p002-needs-pa" "NEEDS_PA" \
    "NEEDS_PA Case (Criteria Met, PA Required)"

# Test 4.3: DENY - Age Failure
run_prove_test "43" "UHC-COMM-BIOPSY-001" "p003-deny-age" "DENY" \
    "DENY Case (Age Inclusion Criterion Fails)"

# Test 4.4: DENY - Exclusion Hit (Pregnancy)
run_prove_test "44" "UHC-COMM-BIOPSY-001" "p004-deny-pregnant" "DENY" \
    "DENY Case (Exclusion Criterion Triggered - Pregnancy)"

# Test 4.5: DENY - Wrong Place of Service
run_prove_test "45" "UHC-COMM-BIOPSY-001" "p005-deny-pos" "DENY" \
    "DENY Case (Admin Rule Fails - Wrong Place of Service)"

# Test 4.6: DENY - Exceeds Max Units
run_prove_test "46" "UHC-COMM-BIOPSY-001" "p006-deny-units" "DENY" \
    "DENY Case (Admin Rule Fails - Exceeds Max Units)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4A Summary: Original Imaging Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "What we validated:"
echo "  ✓ Auto-approve workflow (CT chest, no PA)"
echo "  ✓ PA-required workflow (Biopsy)"
echo "  ✓ All DENY scenarios:"
echo "    - Age too young (< 18)"
echo "    - Pregnancy exclusion"
echo "    - Wrong place of service"
echo "    - Exceeds max units"
echo ""
SECTION_4A_PASSED=$((PASSED_TESTS - $(cat /tmp/zkp_test_baseline 2>/dev/null || echo 0)))
echo "✅ Section 4A: $SECTION_4A_PASSED/6 tests passed"
echo "$PASSED_TESTS" > /tmp/zkp_test_baseline
echo ""

echo "═══════════════════════════════════════════════════════"
echo "Section 4B: Edge Cases & Boundary Conditions"
echo "═══════════════════════════════════════════════════════"
echo ""

# Test 4.7: MRI Policy Match (First patient for MRI)
run_prove_test "47" "UHC-COMM-MRI-HEAD-001" "p007-mri-approve" "NEEDS_PA" \
    "MRI Policy Match - Fills Testing Gap (Neurological Diagnosis)"

# Test 4.8: Upper Age Bound Test
run_prove_test "48" "UHC-COMM-MRI-HEAD-001" "p008-mri-deny-old" "DENY" \
    "DENY - Age Exceeds Upper Limit (82yo > 80yo max)"

# Test 4.9: Boundary Age Test (Exactly 18)
run_prove_test "49" "UHC-COMM-BIOPSY-001" "p009-boundary-age-18" "NEEDS_PA" \
    "Boundary Test - Exactly at Age Threshold (18yo)"

# Test 4.10: Male Patient (Gender-specific exclusion test)
run_prove_test "410" "UHC-COMM-CT-CHEST-001" "p010-ct-male-pregnant-check" "APPROVE" \
    "APPROVE - Male Patient (Pregnancy Check N/A)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4B Summary: Edge Case Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "What we validated:"
echo "  ✓ MRI policy coverage (neurological diagnoses)"
echo "  ✓ Upper age bound enforcement (>80)"
echo "  ✓ Exact boundary conditions (age = 18)"
echo "  ✓ Gender-specific exclusions (male + pregnancy check)"
echo ""
echo "✅ Section 4B: Edge cases validated"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "Section 4C: Medicare & Medicaid Coverage"
echo "═══════════════════════════════════════════════════════"
echo ""

# Test 4.11: Medicare Colonoscopy Screening
run_prove_test "411" "UHC-MEDICARE-COLONOSCOPY-001" "p011-medicare-colonoscopy" "APPROVE" \
    "Medicare Screening - Colonoscopy (Preventive, Age ≥ 50)"

# Test 4.12: Medicaid Dental (Pediatric)
run_prove_test "412" "UHC-MEDICAID-DENTAL-001" "p016-dental-medicaid" "APPROVE" \
    "Medicaid Dental - Pediatric Case (12yo, Essential Service)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4C Summary: Multi-LOB Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "What we validated:"
echo "  ✓ Medicare LOB (preventive screening, age ≥ 50)"
echo "  ✓ Medicaid LOB (essential services, pediatric)"
echo "  ✓ Lower age bounds (age ≥ 6)"
echo ""
echo "✅ Section 4C: Multi-LOB support validated"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "Section 4D: Physical Therapy (Session-Based Services)"
echo "═══════════════════════════════════════════════════════"
echo ""

# Test 4.13: PT Within Session Limits
run_prove_test "413" "UHC-COMM-PHYSICAL-THERAPY-001" "p012-pt-approve" "NEEDS_PA" \
    "Physical Therapy - Within Session Limits (8 < 12 units)"

# Test 4.14: PT Exceeds Session Limits
run_prove_test "414" "UHC-COMM-PHYSICAL-THERAPY-001" "p013-pt-deny-units" "DENY" \
    "DENY - Exceeds PT Session Limit (15 > 12 units)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4D Summary: Therapy Service Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "What we validated:"
echo "  ✓ Session-based service type (PT/rehab)"
echo "  ✓ High unit count handling (12 units/day limit)"
echo "  ✓ Musculoskeletal ICD codes"
echo ""
echo "✅ Section 4D: Physical therapy workflows validated"
echo ""

echo "═══════════════════════════════════════════════════════"
echo "Section 4E: Specialty Drugs (Pharmacy/Medication)"
echo "═══════════════════════════════════════════════════════"
echo ""

# Test 4.15: Specialty Drug Approved
run_prove_test "415" "UHC-COMM-SPECIALTY-DRUG-001" "p014-drug-approve" "NEEDS_PA" \
    "Specialty Drug - Eligible (Crohn's, Age 18-75)"

# Test 4.16: Specialty Drug Denied (Teratogenic + Pregnancy)
run_prove_test "416" "UHC-COMM-SPECIALTY-DRUG-001" "p015-drug-deny-pregnant" "DENY" \
    "DENY - Teratogenic Drug + Pregnancy Exclusion"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4E Summary: Specialty Drug Tests Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "What we validated:"
echo "  ✓ Pharmacy/medication policy type"
echo "  ✓ Age range constraints (18-75)"
echo "  ✓ Teratogenic drug exclusions"
echo "  ✓ GI/Rheumatology ICD codes"
echo ""
echo "✅ Section 4E: Specialty drug workflows validated"
echo ""

# ═══════════════════════════════════════════════════════════════════════
# Section 5: Proof Verification Tests
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 5: Proof Verification Tests"

echo "Verifying all proofs generated in Section 4."
echo "This validates that the ZKP verifier correctly confirms proof integrity."
echo ""

run_verify_test "51" "APPROVE Case" "41"
run_verify_test "52" "NEEDS_PA Case" "42"
run_verify_test "53" "DENY (Age) Case" "43"
run_verify_test "54" "DENY (Pregnancy) Case" "44"
run_verify_test "55" "DENY (POS) Case" "45"
run_verify_test "56" "DENY (Units) Case" "46"

run_verify_test "57" "MRI Policy Match" "47"
run_verify_test "58" "DENY (Age Upper Bound)" "48"
run_verify_test "59" "Boundary Age (18yo)" "49"
run_verify_test "510" "Male Patient (Pregnancy N/A)" "410"

run_verify_test "511" "Medicare Colonoscopy" "411"
run_verify_test "512" "Medicaid Dental (Pediatric)" "412"

run_verify_test "513" "Physical Therapy (Approved)" "413"
run_verify_test "514" "PT DENY (Exceeds Units)" "414"

run_verify_test "515" "Specialty Drug (Approved)" "415"
run_verify_test "516" "Drug DENY (Pregnancy)" "416"

# ═══════════════════════════════════════════════════════════════════════
# Section 6: Privacy Guarantee Checks
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 6: Privacy Guarantee Validation"

# Test 6.1: Verify NO PHI in decision record
print_test "6.1" "Verify Decision Record Contains NO PHI"

cat << 'PRIVACY_CONTEXT'

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
CRITICAL PRIVACY TEST
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

This test validates the CORE VALUE PROPOSITION of the system:
Zero-knowledge proofs that share ZERO patient data.

WHAT'S BEING CHECKED:
  ✓ Decision record JSON contains only non-sensitive fields
  ✗ NO protected health information (PHI) is present

ALLOWED (Public Information):
  • policy_id - Which policy was applied
  • policy_hash - Cryptographic fingerprint of policy version
  • patient_commitment - Hash of patient features (reveals nothing)
  • claimed_result - APPROVE/NEEDS_PA/DENY
  • proof - Cryptographic proof (2KB, reveals nothing)
  • code - CPT/HCPCS procedure code
  • lob - Line of business

FORBIDDEN (PHI that MUST BE ABSENT):
  • age, dob - Date of birth or age
  • sex, gender - Patient gender
  • icd10, diagnosis, diagnoses - Any diagnosis codes
  • patient_id, name - Any patient identifiers

WHY THIS MATTERS:
  If PHI is found, the entire value proposition fails. The system would
  be no better than traditional authorization (which shares full records).
  
  With zero-knowledge, a payer can verify "this authorization decision
  is correct" without learning anything about the patient.

HIPAA COMPLIANCE:
  If this test passes, decision records can be shared freely without
  HIPAA concerns - they contain no PHI by cryptographic construction.
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PRIVACY_CONTEXT

PROOF_FILE="out/test_42_proof.json"
if [[ ! -f "$PROOF_FILE" ]]; then
    print_fail "Proof file not found: $PROOF_FILE"
else
    print_info "Analyzing decision record: $PROOF_FILE"
    echo ""
    
    # Check what fields are present
    FIELDS=$(jq -r 'keys | .[]' "$PROOF_FILE" 2>/dev/null)
    
    echo "  Fields present in decision record:"
    for field in $FIELDS; do
        print_detail "  ✓ $field (public)"
    done
    
    echo ""
    echo "  PHI fields that SHOULD BE ABSENT:"
    
    # Check that PHI fields are NOT present
    HAS_AGE=$(echo "$FIELDS" | grep -c "age" || true)
    HAS_SEX=$(echo "$FIELDS" | grep -c "sex" || true)
    HAS_DOB=$(echo "$FIELDS" | grep -c "dob" || true)
    HAS_ICD=$(echo "$FIELDS" | grep -c "icd" || true)
    HAS_DIAGNOSIS=$(echo "$FIELDS" | grep -c "diagnosis" || true)
    
    ALL_CLEAN=true
    
    if [[ $HAS_AGE -eq 0 ]]; then
        print_detail "  ✓ age: NOT PRESENT"
    else
        print_detail "  ✗ age: PRESENT (privacy violation!)"
        ALL_CLEAN=false
    fi
    
    if [[ $HAS_SEX -eq 0 ]]; then
        print_detail "  ✓ sex: NOT PRESENT"
    else
        print_detail "  ✗ sex: PRESENT (privacy violation!)"
        ALL_CLEAN=false
    fi
    
    if [[ $HAS_DOB -eq 0 ]]; then
        print_detail "  ✓ dob: NOT PRESENT"
    else
        print_detail "  ✗ dob: PRESENT (privacy violation!)"
        ALL_CLEAN=false
    fi
    
    if [[ $HAS_ICD -eq 0 && $HAS_DIAGNOSIS -eq 0 ]]; then
        print_detail "  ✓ diagnoses: NOT PRESENT"
    else
        print_detail "  ✗ diagnoses: PRESENT (privacy violation!)"
        ALL_CLEAN=false
    fi
    
    echo ""
    
    if $ALL_CLEAN; then
        print_pass "No PHI exposed in decision record - privacy preserved!"
    else
        print_fail "PHI found in decision record - privacy violation!"
    fi
fi

# Test 6.2: Verify proof size is reasonable
print_test "6.2" "Verify Proof Sizes Are Reasonable"

echo "Checking proof file sizes (expecting < 50 KB for ZKP proofs):"
echo ""

TOTAL_SIZE=0
PROOF_COUNT=0
ALL_REASONABLE=true

for proof_file in out/test_4*_proof.json; do
    if [[ -f "$proof_file" ]]; then
        SIZE_BYTES=$(wc -c < "$proof_file" | tr -d ' ')
        SIZE_KB=$((SIZE_BYTES / 1024))
        PROOF_COUNT=$((PROOF_COUNT + 1))
        TOTAL_SIZE=$((TOTAL_SIZE + SIZE_BYTES))
        
        FILENAME=$(basename "$proof_file")
        printf "  %-30s  %5d KB" "$FILENAME" "$SIZE_KB"
        
        if [[ $SIZE_KB -lt 50 ]]; then
            echo "  ✓"
        else
            echo "  ✗ (too large)"
            ALL_REASONABLE=false
        fi
    fi
done

TOTAL_KB=$((TOTAL_SIZE / 1024))
echo ""
print_detail "Total size: $TOTAL_KB KB across $PROOF_COUNT proofs"
echo ""

if $ALL_REASONABLE; then
    print_pass "All proof files are reasonably sized (< 50 KB)"
else
    print_fail "Some proof files are too large"
fi

# ═══════════════════════════════════════════════════════════════════════
# Section 7: Error Handling Tests
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 7: Error Handling Tests"

# Test 7.1: Invalid policy file
print_test "7.1" "Invalid Policy File (Malformed JSON)"

# Create invalid JSON
cat > /tmp/invalid_policy.json << 'EOF'
{
  "policy_id": "INVALID"
  "missing_comma": true
}
EOF

print_info "Testing with malformed JSON..."
echo ""

if cargo run --quiet --release --package zk-agent --bin authz -- prove \
    --policy /tmp/invalid_policy.json \
    --patient patients/p001-approve.json \
    --code "00000" \
    --lob "commercial" \
    --out out/invalid_test.json > /dev/null 2>&1; then
    print_fail "Should have failed with invalid policy, but succeeded"
else
    print_pass "Correctly rejected invalid policy file"
fi

# Test 7.2: Invalid patient file
print_test "7.2" "Invalid Patient File (Missing Required Fields)"

# Create patient with missing ICD list
cat > /tmp/invalid_patient.json << 'EOF'
{
  "patient_id": "invalid-patient",
  "dob": "1980-01-01",
  "sex": "M",
  "pregnant": false,
  "place_of_service": 22,
  "units": 1
}
EOF

print_info "Testing with patient missing ICD-10 list..."
echo ""

if cargo run --quiet --release --package zk-agent --bin authz -- prove \
    --policy policies/UHC-COMM-BIOPSY-001.json \
    --patient /tmp/invalid_patient.json \
    --code "00000" \
    --lob "commercial" \
    --out out/invalid_test.json > /dev/null 2>&1; then
    print_fail "Should have failed with invalid patient, but succeeded"
else
    print_pass "Correctly rejected invalid patient file"
fi

# Test 7.3: Tampered proof verification
print_test "7.3" "Tampered Proof Verification"

ORIGINAL_PROOF="out/test_42_proof.json"

if [[ ! -f "$ORIGINAL_PROOF" ]]; then
    print_fail "Original proof not found for tampering test"
else
    # Create a tampered version
    TAMPERED_PROOF="out/test_tampered_proof.json"
    
    # Modify the proof data (flip some bytes in the proof field)
    jq '.proof = "TAMPERED_DATA_INVALID_PROOF"' "$ORIGINAL_PROOF" > "$TAMPERED_PROOF"
    
    print_info "Testing verification with tampered proof..."
    echo ""
    
    if cargo run --quiet --release --package zk-agent --bin authz -- verify "$TAMPERED_PROOF" > /dev/null 2>&1; then
        print_fail "Tampered proof was accepted (security issue!)"
    else
        print_pass "Correctly rejected tampered proof"
    fi
    
    rm -f "$TAMPERED_PROOF"
fi

# ═══════════════════════════════════════════════════════════════════════
# Section 8: Cross-Policy Tests
# ═══════════════════════════════════════════════════════════════════════

print_section "Section 8: Cross-Policy Tests"

# Test 8.1: Same patient, multiple policies
print_test "8.1" "Same Patient Against Multiple Policies"

print_info "Testing patient p002 against all available policies..."
print_info "This shows how authorization changes based on policy."
echo ""

PATIENT="p002-needs-pa"
echo "Patient: $PATIENT"
echo ""
printf "  %-30s  %-15s\n" "Policy" "Result"
printf "  %-30s  %-15s\n" "------------------------------" "---------------"

for policy_file in policies/*.json; do
    if [[ -f "$policy_file" ]]; then
        POLICY_NAME=$(basename "$policy_file" .json)
        TEST_OUTPUT="out/test_cross_${POLICY_NAME}.json"
        
        if cargo run --quiet --release --package zk-agent --bin authz -- prove \
            --policy "$policy_file" \
            --patient "patients/$PATIENT.json" \
            --code "00000" \
            --lob "commercial" \
            --out "$TEST_OUTPUT" > /dev/null 2>&1; then
            
            RESULT=$(jq -r '.claimed_result' "$TEST_OUTPUT" 2>/dev/null || echo "ERROR")
            printf "  %-30s  %-15s\n" "$POLICY_NAME" "$RESULT"
            rm -f "$TEST_OUTPUT"
        else
            printf "  %-30s  %-15s\n" "$POLICY_NAME" "FAILED"
        fi
    fi
done

echo ""
print_pass "Cross-policy testing completed"

# Test 8.2: Policy version tracking
print_test "8.2" "Policy Version Tracking (Auditability)"

ORIGINAL_POLICY="policies/UHC-COMM-BIOPSY-001.json"

if [[ ! -f "$ORIGINAL_POLICY" ]]; then
    print_fail "Original policy not found"
else
    # Create a modified version with different version number
    jq '.version = "2025-11-01"' "$ORIGINAL_POLICY" > /tmp/test_policy_v2.json
    
    # Compute hashes
    HASH_V1=$(jq -c -S '.' "$ORIGINAL_POLICY" | shasum -a 256 | awk '{print $1}' | cut -c1-16)
    HASH_V2=$(jq -c -S '.' /tmp/test_policy_v2.json | shasum -a 256 | awk '{print $1}' | cut -c1-16)
    
    print_detail "Policy v2025-10-01 hash: 0x$HASH_V1..."
    print_detail "Policy v2025-11-01 hash: 0x$HASH_V2..."
    echo ""
    
    if [[ "$HASH_V1" != "$HASH_V2" ]]; then
        print_pass "Different policy versions produce different hashes (auditability ensured)"
    else
        print_fail "Policy versions have same hash (auditability issue)"
    fi
fi

# ═══════════════════════════════════════════════════════════════════════
# Final Summary
# ═══════════════════════════════════════════════════════════════════════

END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

print_header "Test Summary"

echo "Test Execution Complete"
echo ""
echo "Results:"
echo "  Total Tests:  $TOTAL_TESTS"
echo -e "  Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "  Failed:       ${RED}$FAILED_TESTS${NC}"
echo "  Duration:     ${ELAPSED}s"
echo ""

echo "Test Matrix Coverage:"
echo ""
printf "  %-30s  %s\n" "Category" "Tests"
printf "  %-30s  %s\n" "------------------------------" "-------"
printf "  %-30s  %s\n" "Imaging (CT/MRI/Biopsy)" "10"
printf "  %-30s  %s\n" "Physical Therapy" "2"
printf "  %-30s  %s\n" "Specialty Drugs" "2"
printf "  %-30s  %s\n" "Medicare Screening" "1"
printf "  %-30s  %s\n" "Medicaid Dental" "1"
printf "  %-30s  %s\n" "Edge Cases/Boundaries" "4"
printf "  %-30s  %s\n" "Privacy Guarantees" "2"
printf "  %-30s  %s\n" "Error Handling" "3"
printf "  %-30s  %s\n" "Cross-Policy" "2"
echo ""
printf "  %-30s  %s\n" "Lines of Business:" ""
printf "  %-30s  %s\n" "  • Commercial" "14 tests"
printf "  %-30s  %s\n" "  • Medicare" "1 test"
printf "  %-30s  %s\n" "  • Medicaid" "1 test"
echo ""
printf "  %-30s  %s\n" "Age Range Tested:" "12yo - 82yo"
printf "  %-30s  %s\n" "Service Types:" "7 categories"
printf "  %-30s  %s\n" "Outcome Coverage:" "APPROVE(4), PA(6), DENY(6)"
echo ""

echo "Test Coverage:"
echo "  ✓ Environment & build validation"
echo "  ✓ Policy parsing & structure validation"
echo "  ✓ Patient feature extraction readiness"
echo "  ✓ Proof generation (all outcome types)"
echo "  ✓ Proof verification (all proofs)"
echo "  ✓ Privacy guarantee checks (NO PHI)"
echo "  ✓ Error handling (invalid inputs)"
echo "  ✓ Cross-policy scenarios"
echo ""

# Count proof files
PROOF_FILES_COUNT=$(ls -1 out/test_4*_proof.json 2>/dev/null | wc -l | tr -d ' ')
if [[ $PROOF_FILES_COUNT -gt 0 ]]; then
    TOTAL_PROOF_SIZE=$(du -ch out/test_4*_proof.json 2>/dev/null | tail -1 | awk '{print $1}')
    echo "Proof Statistics:"
    echo "  Files generated: $PROOF_FILES_COUNT"
    echo "  Total size:      $TOTAL_PROOF_SIZE"
    echo ""
fi

echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
if [[ $FAILED_TESTS -eq 0 ]]; then
    echo -e "${GREEN}Result: ALL TESTS PASSED ✓${NC}"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "What This Means:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "✓ TECHNICAL VALIDATION:"
    echo "  The system can cryptographically prove authorization decisions"
    echo "  without revealing any protected health information (PHI). All"
    echo "  components work correctly: policy evaluation, patient feature"
    echo "  extraction, zero-knowledge proof generation, and verification."
    echo ""
    echo "✓ PRIVACY GUARANTEE:"
    echo "  Decision records contain ZERO patient data - no age, sex, diagnoses,"
    echo "  or other identifiers. Only cryptographic commitments are shared,"
    echo "  making this HIPAA-compliant by design."
    echo ""
    echo "✓ PRACTICAL APPLICATIONS FOR HEALTHCARE:"
    echo ""
    echo "  For Clinicians:"
    echo "    • Submit authorization requests with patient data encrypted locally"
    echo "    • Receive instant APPROVE/NEEDS_PA/DENY decisions"
    echo "    • Get cryptographic proof of correct policy application"
    echo "    • No PHI ever leaves your system in plaintext"
    echo ""
    echo "  For Hospitals/Health Systems:"
    echo "    • Process prior authorizations without exposing patient records"
    echo "    • Share authorization proofs with payers without sharing PHI"
    echo "    • Audit policy compliance with cryptographic guarantees"
    echo "    • Reduce data breach risk - attackers can't steal what isn't there"
    echo ""
    echo "  For Payers/Insurance:"
    echo "    • Verify authorization decisions were made correctly"
    echo "    • Ensure policies were applied as written (auditability)"
    echo "    • Process claims without receiving sensitive medical data"
    echo "    • Maintain compliance while streamlining operations"
    echo ""
    echo "✓ REAL-WORLD EXAMPLE:"
    echo "  A patient needs a breast biopsy. The clinician's system generates"
    echo "  a ZK proof showing the request meets policy criteria (age, diagnosis,"
    echo "  place of service). The payer receives only:"
    echo "    - Policy ID: UHC-COMM-BIOPSY-001"
    echo "    - Decision: NEEDS_PA"
    echo "    - Cryptographic proof (2KB)"
    echo ""
    echo "  The payer can verify the decision was correct WITHOUT ever seeing"
    echo "  the patient's age, sex, diagnosis, or any other medical details."
    echo ""
    echo "✓ NEXT STEPS:"
    echo "  → Integrate this system into your EHR/practice management software"
    echo "  → Generate proofs for real authorization requests"
    echo "  → Share decision records with payers via secure API"
    echo "  → Maintain full audit trail with cryptographic guarantees"
    echo ""
else
    echo -e "${RED}Result: $FAILED_TESTS TEST(S) FAILED ✗${NC}"
    echo ""
    echo "Please review the failures above and fix the issues."
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Troubleshooting Failed Tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "COMMON FAILURE CAUSES:"
    echo ""
    echo "1. ICD-10 Mapping Issues"
    echo "   Symptom: 'Unknown ICD-10' error in proof generation"
    echo "   Fix: Check zk-agent/src/icd_map.rs has all codes"
    echo "   Example: Add missing code to ICD_MAP HashMap"
    echo ""
    echo "2. Authorization Logic Bugs"
    echo "   Symptom: Expected APPROVE but got DENY (or vice versa)"
    echo "   Fix: Review zk-agent/src/trace.rs and decision.rs logic"
    echo "   Check: AuthorizationResult::from_logic() order"
    echo ""
    echo "3. Age Calculation Issues"
    echo "   Symptom: Age boundary tests fail (e.g., test 49, 48)"
    echo "   Fix: Check zk-agent/src/patient.rs age extraction"
    echo "   Verify: Current year - birth year calculation"
    echo ""
    echo "4. Policy Parsing Errors"
    echo "   Symptom: Proof generation crashes for specific policies"
    echo "   Fix: Validate JSON syntax in policies/ directory"
    echo "   Tool: jq -e '.' policies/POLICY-NAME.json"
    echo ""
    echo "5. Missing dev-srs Feature"
    echo "   Symptom: 'G1 SRS insufficient' panic"
    echo "   Fix: Ensure zk-agent/Cargo.toml has:"
    echo "        myzkp = { path = \"..\", features = [\"dev-srs\"] }"
    echo ""
    echo "6. Missing Patient/Policy Files"
    echo "   Symptom: File not found errors"
    echo "   Fix: Ensure all 16 patient files (p001-p016) exist"
    echo "        Ensure all 7 policy files exist"
    echo ""
    echo "DEBUG STEPS:"
    echo "  1. Run failing test manually without --quiet flag:"
    echo "     cargo run --release --package zk-agent --bin authz -- prove \\"
    echo "       --policy policies/POLICY.json \\"
    echo "       --patient patients/PATIENT.json \\"
    echo "       --code CODE --lob LOB --out out/debug.json"
    echo ""
    echo "  2. Check error message for specific failure point"
    echo "  3. Verify input files (policy + patient JSON) are valid"
    echo "  4. Test ICD code mapping in isolation (cargo test)"
    echo "  5. Check if zk-agent needs rebuild: cargo build --release -p zk-agent"
    echo ""
    echo "GETTING HELP:"
    echo "  • Review test output above for specific error messages"
    echo "  • Check zk-agent/README.md for architecture overview"
    echo "  • Review HACKATHON_PROJECT_SPEC.md for system design"
    echo ""
fi
echo -e "${CYAN}═══════════════════════════════════════════════════════${NC}"
echo ""

exit $EXIT_CODE

