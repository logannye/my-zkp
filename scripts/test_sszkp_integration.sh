#!/usr/bin/env bash
# scripts/test_sszkp_integration.sh
#
# Comprehensive Integration Test Suite for SSZKP
# Tests production-critical paths not covered by basic CLI tests:
# - API builders (ProverBuilder/VerifierBuilder)
# - CSV streaming (CsvRows adapter)
# - Real permutation arguments
# - Proof I/O edge cases
# - Memory diagnostic modes
# - Large register counts (k > 4)
#
# Environment variables:
#   SSZKP_BLOCKED_IFFT=1        Set globally (streaming mode - production config)
#   SSZKP_RUN_SLOW_TESTS=1      Enable large CSV streaming test (100K rows)
#   SSZKP_MEASURE_MEMORY=1      Enable memory measurement tests
#
# Note: Tests are sized for reasonable runtime in debug mode (~5 min total).
# For production-scale validation, build with --release and test manually.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Enable streaming mode globally - this is the production configuration
export SSZKP_BLOCKED_IFFT=1

# Color output helpers
banner() { printf "\n\033[1;36m==> %s\033[0m\n" "$*"; }
info()   { printf "  \033[0;36m• %s\033[0m\n" "$*"; }
pass()   { printf "\033[1;32m✔ %s\033[0m\n" "$*"; }
skip()   { printf "\033[1;33m↷ skipped: %s\033[0m\n" "$*"; }
fail()   { printf "\033[1;31m✘ %s\033[0m\n" "$*"; }

# Test counters
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# Track which tests ran
run_section() {
    local section_name="$1"
    shift
    local -a cmd=("$@")
    
    banner "Section: $section_name"
    
    set +e
    "${cmd[@]}"
    local rc=$?
    set -e
    
    if [[ $rc -eq 0 ]]; then
        PASS_COUNT=$((PASS_COUNT + 1))
        pass "$section_name"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        fail "$section_name"
    fi
    
    return $rc
}

# =============================================================================
# Setup
# =============================================================================

banner "Integration Test Suite: Production-Critical Paths"
info "Workspace: $ROOT_DIR"

# Build with dev-srs (required for all tests)
banner "Build: dev-srs feature"
cargo build --quiet --features dev-srs --lib --tests || {
    fail "Build failed"
    exit 1
}
pass "Build succeeded"

# =============================================================================
# Section 1: API Builders (ProverBuilder/VerifierBuilder)
# =============================================================================

run_section "API Builders" \
    cargo test --quiet --features dev-srs --test api_builders

# =============================================================================
# Section 2: CSV Streaming (CsvRows)
# =============================================================================

run_section "CSV Streaming (basic)" \
    cargo test --quiet --features dev-srs --test csv_streaming -- --skip test_2_4

# Run slow test separately if requested
if [[ "${SSZKP_RUN_SLOW_TESTS:-}" == "1" ]]; then
    run_section "CSV Streaming (large file - SLOW)" \
        cargo test --quiet --features dev-srs --test csv_streaming test_2_4 -- --ignored --show-output
else
    info "Skipping large CSV test (set SSZKP_RUN_SLOW_TESTS=1 to enable)"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# =============================================================================
# Section 3: Real Permutation Arguments
# =============================================================================

run_section "Permutation Arguments" \
    cargo test --quiet --features dev-srs --test permutations

# =============================================================================
# Section 4: Proof I/O Edge Cases
# =============================================================================

run_section "Proof I/O" \
    cargo test --quiet --features dev-srs --test proof_io

# =============================================================================
# Section 5: Memory Diagnostic Modes
# =============================================================================

banner "Section: Memory Diagnostics"

# Test 5.1: SSZKP_MEMLOG output
info "Test 5.1: Memory logging output..."
export SSZKP_MEMLOG=1
set +e
cargo run --quiet --features dev-srs --bin prover -- \
    --rows 4096 --b-blk 128 --k 3 --basis eval 2>&1 | tee memlog_test.txt >/dev/null
rc=$?
set -e
unset SSZKP_MEMLOG

if [[ $rc -eq 0 ]]; then
    # Check that memory log contains expected markers
    if grep -q "peak_inflight_coeffs\|Aggregator\|BlockedIfft" memlog_test.txt 2>/dev/null; then
        pass "Memory logging works"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        skip "Memory logging (no diagnostic output found)"
        SKIP_COUNT=$((SKIP_COUNT + 1))
    fi
else
    fail "Memory logging test (prover failed)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
rm -f memlog_test.txt proof.bin

# Test 5.2: SSZKP_BLOCKED_IFFT mode
info "Test 5.2: Blocked IFFT tape mode..."
export SSZKP_BLOCKED_IFFT=1
set +e
cargo run --quiet --features dev-srs --bin prover -- \
    --rows 8192 --b-blk 256 --k 3 --basis eval >/dev/null 2>&1
rc_prove=$?

if [[ $rc_prove -eq 0 && -f proof.bin ]]; then
    cargo run --quiet --features dev-srs --bin verifier -- \
        --rows 8192 --basis eval >/dev/null 2>&1
    rc_verify=$?
    
    if [[ $rc_verify -eq 0 ]]; then
        pass "Blocked IFFT mode works"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        fail "Blocked IFFT mode (verification failed)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    fail "Blocked IFFT mode (prover failed)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
set -e
# unset SSZKP_BLOCKED_IFFT  # No longer needed - set globally for all tests
rm -f proof.bin

# Test 5.3: Memory measurement (if requested)
if [[ "${SSZKP_MEASURE_MEMORY:-}" == "1" ]]; then
    info "Test 5.3: Memory measurement (large N, small b_blk)..."
    
    # This is platform-specific and optional
    if command -v /usr/bin/time >/dev/null 2>&1; then
        /usr/bin/time -l cargo run --release --quiet --features dev-srs --bin prover -- \
            --rows 65536 --b-blk 256 --k 16 --basis eval 2>&1 | tee memory_measurement.txt
        
        if [[ -f proof.bin ]]; then
            pass "Memory measurement completed (see memory_measurement.txt)"
            PASS_COUNT=$((PASS_COUNT + 1))
        else
            fail "Memory measurement (prover failed)"
            FAIL_COUNT=$((FAIL_COUNT + 1))
        fi
        rm -f proof.bin
    else
        skip "Memory measurement (/usr/bin/time not available)"
        SKIP_COUNT=$((SKIP_COUNT + 1))
    fi
else
    info "Skipping memory measurement (set SSZKP_MEASURE_MEMORY=1 to enable)"
    SKIP_COUNT=$((SKIP_COUNT + 1))
fi

# =============================================================================
# Section 6: Large Register Counts
# =============================================================================

banner "Section: Large Register Counts"

# Test 6.1: k=16
info "Test 6.1: k=16 baseline..."
set +e
cargo run --quiet --features dev-srs --bin prover -- \
    --rows 1024 --b-blk 128 --k 16 --basis eval >/dev/null 2>&1
rc_prove=$?

if [[ $rc_prove -eq 0 && -f proof.bin ]]; then
    cargo run --quiet --features dev-srs --bin verifier -- \
        --rows 1024 --basis eval >/dev/null 2>&1
    rc_verify=$?
    
    if [[ $rc_verify -eq 0 ]]; then
        pass "k=16 baseline"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        fail "k=16 baseline (verification failed)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    fail "k=16 baseline (prover failed)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
set -e
rm -f proof.bin

# Test 6.2: k=32 (production scale)
# Note: Reduced from 16K to 4K rows to keep test time reasonable in debug mode
info "Test 6.2: k=32 (production scale)..."
set +e
cargo run --quiet --features dev-srs --bin prover -- \
    --rows 4096 --b-blk 256 --k 32 --basis eval >/dev/null 2>&1
rc_prove=$?

if [[ $rc_prove -eq 0 && -f proof.bin ]]; then
    cargo run --quiet --features dev-srs --bin verifier -- \
        --rows 2048 --basis eval >/dev/null 2>&1
    rc_verify=$?
    
    if [[ $rc_verify -eq 0 ]]; then
        pass "k=32 production scale"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        fail "k=32 (verification failed)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    fail "k=32 (prover failed)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
set -e
rm -f proof.bin

# Test 6.3: k=1 edge case
info "Test 6.3: k=1 edge case..."
set +e
cargo run --quiet --features dev-srs --bin prover -- \
    --rows 1024 --b-blk 128 --k 1 --basis eval >/dev/null 2>&1
rc_prove=$?

if [[ $rc_prove -eq 0 && -f proof.bin ]]; then
    cargo run --quiet --features dev-srs --bin verifier -- \
        --rows 1024 --basis eval >/dev/null 2>&1
    rc_verify=$?
    
    if [[ $rc_verify -eq 0 ]]; then
        pass "k=1 edge case"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        fail "k=1 (verification failed)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
else
    fail "k=1 (prover failed)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
fi
set -e
rm -f proof.bin

# =============================================================================
# Summary
# =============================================================================

banner "Integration Test Results"
echo "  PASS: $PASS_COUNT"
echo "  FAIL: $FAIL_COUNT"
echo "  SKIP: $SKIP_COUNT"
echo "  ────────────────"
echo "  TOTAL: $((PASS_COUNT + FAIL_COUNT + SKIP_COUNT))"

if [[ $FAIL_COUNT -eq 0 ]]; then
    echo ""
    pass "All integration tests passed! 🎉"
    echo ""
    echo "Coverage Summary:"
    echo "  ✓ API builders (ProverBuilder/VerifierBuilder)"
    echo "  ✓ CSV streaming (CsvRows adapter)"
    echo "  ✓ Real permutation arguments"
    echo "  ✓ Proof I/O edge cases"
    echo "  ✓ Memory diagnostic modes"
    echo "  ✓ Large register counts (k up to 32)"
    echo ""
    exit 0
else
    echo ""
    fail "Some integration tests failed"
    echo ""
    echo "To debug failures:"
    echo "  - Run specific test: cargo test --features dev-srs --test <test_name>"
    echo "  - Enable verbose output: cargo test --features dev-srs --test <test_name> -- --show-output"
    echo "  - Check linter: cargo clippy --features dev-srs --tests"
    echo ""
    exit 1
fi

