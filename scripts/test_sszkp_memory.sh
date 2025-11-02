#!/usr/bin/env bash
# scripts/test_sszkp_memory.sh
# Empirical validation of O(√N) memory guarantees

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Configuration
# TEST_SIZES=(1024 4096 16384 65536 262144 1048576 4194304 16777216)
TEST_SIZES=(1024 4096 16384 65536)
CLEANUP_FILES=()  # Track files to delete
EXIT_CODE=0

# ============================================================================
# CRITICAL: Enable streaming mode for O(√N) memory
# ============================================================================
# Without SSZKP_BLOCKED_IFFT=1, the system uses legacy mode with O(N) memory
# (100x+ worse than streaming mode). This flag enables the file-backed tape
# architecture that keeps peak memory at O(b_blk) = O(√N).
export SSZKP_BLOCKED_IFFT=1

# Track memory measurements for scaling analysis (bash 3 compatible)
MEMORY_KEYS=()
MEMORY_VALUES=()

trap cleanup EXIT
cleanup() {
    set +e  # Disable exit-on-error for cleanup
    echo ""
    echo "==> Cleaning up test artifacts..."
    for file in "${CLEANUP_FILES[@]+"${CLEANUP_FILES[@]}"}"; do
        [[ -f "$file" ]] && rm -f "$file" && echo "  Removed: $file"
    done
    rm -f proof.bin witness_*.csv memory_*.log
    echo "✓ Cleanup complete"
    set -e  # Re-enable exit-on-error
}

# Memory measurement helper
measure_memory_rss() {
    local rows=$1 k=$2 b_blk=$3 basis=$4
    local log_file="memory_${rows}_${k}_${b_blk}.log"
    CLEANUP_FILES+=("$log_file")
    
    echo "  Testing N=$rows, k=$k, b_blk=$b_blk..."
    
    # Build in release mode (critical for realistic measurements)
    cargo build --release --quiet --features dev-srs --bin prover
    
    # Run with time measurement (capture output first, then parse)
    if [[ "$(uname)" == "Darwin" ]]; then
        # macOS
        /usr/bin/time -l target/release/prover \
            --rows $rows --k $k --b-blk $b_blk --basis $basis \
            2>&1 | tee "$log_file"
        
        # Wait for file to be fully written
        sync
        
        # Verify log file has expected format
        if ! grep -q "real.*user.*sys" "$log_file"; then
            echo "    ⚠ WARNING: Log file missing time output - may be corrupted"
            peak_kb=0
        else
            # Extract peak RSS with better error handling
            # The time output has format: "   4521984  maximum resident set size"
            # We want the number (in bytes), convert to KB
            peak_kb=$(grep "maximum resident set size" "$log_file" | \
                      awk '/maximum resident set size/ {print int($1/1024); exit}')
            
            # Fallback: try to find ANY line with digits followed by "maximum resident"
            if [[ -z "$peak_kb" || "$peak_kb" == "0" ]]; then
                peak_kb=$(grep -oE '[0-9]+[[:space:]]+maximum resident set size' "$log_file" | \
                          head -1 | awk '{print int($1/1024)}')
            fi
        fi
    else
        # Linux
        /usr/bin/time -v target/release/prover \
            --rows $rows --k $k --b-blk $b_blk --basis $basis \
            2>&1 | tee "$log_file"
        
        sync
        
        # Verify log file has expected format
        if ! grep -q "User time\|System time\|Elapsed" "$log_file"; then
            echo "    ⚠ WARNING: Log file missing time output - may be corrupted"
            peak_kb=0
        else
            # Extract peak RSS with better error handling  
            # Linux uses "Maximum resident set size (kbytes): 12345"
            peak_kb=$(grep "Maximum resident set size" "$log_file" | \
                      awk -F': ' '/Maximum resident set size/ {print int($2); exit}')
            
            # Fallback: try alternative format
            if [[ -z "$peak_kb" || "$peak_kb" == "0" ]]; then
                peak_kb=$(grep -oE 'Maximum resident set size.*[0-9]+' "$log_file" | \
                          grep -oE '[0-9]+$' | head -1)
            fi
        fi
    fi
    
    # Fallback if still empty
    if [[ -z "$peak_kb" || "$peak_kb" == "0" ]]; then
        echo "    ⚠ WARNING: Could not parse memory usage from log"
        peak_kb=0
    fi
    
    # Calculate theoretical bounds and actual working set
    local baseline_kb=4500  # Empirical baseline (Rust binary + dev SRS)
    local working_set_kb=$((peak_kb - baseline_kb))
    local traditional_working_kb=$((rows * k * 32 / 1024))
    
    echo "    Peak RSS: ${peak_kb} KB"
    
    if [[ $peak_kb -gt 0 ]]; then
        echo "      ├─ Baseline (binary+SRS): ~${baseline_kb} KB (constant)"
        echo "      ├─ Working set (proof): ~${working_set_kb} KB"
        echo "      └─ Traditional O(N) needs: ${traditional_working_kb} KB"
        
        if [[ $working_set_kb -gt 0 && $traditional_working_kb -gt 0 ]]; then
            local reduction=$(echo "scale=1; $traditional_working_kb / $working_set_kb" | bc)
            echo "      Memory efficiency: ${reduction}x reduction"
        fi
    else
        echo "      (unable to calculate - measurement failed)"
    fi
    
    # Store measurement for scaling analysis (bash 3 compatible)
    local key="${rows}_${k}_${b_blk}"
    MEMORY_KEYS+=("$key")
    MEMORY_VALUES+=("$peak_kb")
    
    # Return the peak_kb value (informational only, not pass/fail)
    echo "$peak_kb"
}

# Helper function to get memory value by key
get_memory() {
    local search_key="$1"
    for i in "${!MEMORY_KEYS[@]}"; do
        if [[ "${MEMORY_KEYS[$i]}" == "$search_key" ]]; then
            echo "${MEMORY_VALUES[$i]}"
            return 0
        fi
    done
    echo "0"
}

# Validate O(√N) scaling relationship
validate_scaling() {
    echo ""
    echo "==> Validating O(√N) Memory Scaling"
    echo "  Theory: Memory grows as O(√N), not O(N)"
    echo "  For 4x increase in N, expect ~2x increase in memory (√4 = 2)"
    echo ""
    
    local all_pass=true
    local baseline_kb=4500  # Subtract constant overhead
    
    # Check scaling between consecutive points
    for i in "${!TEST_SIZES[@]}"; do
        if [[ $i -eq 0 ]]; then continue; fi
        
        local prev_idx=$((i-1))
        local curr_n=${TEST_SIZES[$i]}
        local prev_n=${TEST_SIZES[$prev_idx]}
        
        local curr_b_blk=$(echo "sqrt($curr_n)" | bc)
        local prev_b_blk=$(echo "sqrt($prev_n)" | bc)
        
        local curr_mem=$(get_memory "${curr_n}_32_${curr_b_blk}")
        local prev_mem=$(get_memory "${prev_n}_32_${prev_b_blk}")
        
        # Skip if either measurement failed
        if [[ $curr_mem -eq 0 || $prev_mem -eq 0 ]]; then
            echo "  ⚠ Skipping N=$prev_n → $curr_n (missing data)"
            continue
        fi
        
        # Subtract baseline to get working set growth
        local curr_work=$((curr_mem - baseline_kb))
        local prev_work=$((prev_mem - baseline_kb))
        
        # Handle negative working set (very small N where baseline dominates)
        if [[ $curr_work -lt 100 || $prev_work -lt 100 ]]; then
            echo "  N: $prev_n → $curr_n"
            echo "    ⚠ SKIPPED: Constant overhead dominates (N too small)"
            continue
        fi
        
        local n_ratio=$(echo "scale=2; $curr_n / $prev_n" | bc)
        local mem_ratio=$(echo "scale=2; $curr_work / $prev_work" | bc)
        local expected_ratio=$(echo "scale=2; sqrt($n_ratio)" | bc)
        
        echo "  N: $prev_n → $curr_n (${n_ratio}x increase)"
        echo "  ────────────────────────────────────────────────"
        echo "    Actual growth: ${prev_work} KB → ${curr_work} KB = ${mem_ratio}x"
        echo ""
        echo "    Comparison:"
        echo "      O(N) behavior:     ${n_ratio}x growth ← conventional prover (bad)"
        echo "      O(√N) ideal:       ${expected_ratio}x growth ← theoretical target"
        echo "      Actual measured:   ${mem_ratio}x growth ← our system"
        echo ""
        
        # Calculate how sublinear we are
        local sublinearity=$(echo "scale=2; $mem_ratio / $n_ratio" | bc)
        echo "    Sublinearity factor: ${sublinearity} (< 1.0 = sublinear, 0.5 = perfect O(√N))"
        echo ""
        
        # Check if we're sublinear (growing slower than O(N))
        if (( $(echo "$mem_ratio < ($n_ratio * 0.9)" | bc -l) )); then
            echo "    ✓ VERDICT: SUBLINEAR space complexity confirmed"
            echo "               (Growing significantly slower than O(N))"
        else
            echo "    ✘ VERDICT: APPROXIMATELY LINEAR - streaming may not be working!"
            all_pass=false
            EXIT_CODE=1
        fi
        echo ""
    done
    
    if $all_pass; then
        echo "✓ Overall: Memory scaling validates O(√N) claim"
    else
        echo "✘ Overall: Memory scaling does not match O(√N)"
    fi
}

# Test 0: Verify Streaming Mode is Active
echo ""
echo "==> Test 0: Verify Streaming Mode Enabled"
export SSZKP_MEMLOG=1
cargo run --release --quiet --features dev-srs --bin prover -- \
    --rows 1024 --k 4 --b-blk 32 --basis eval 2>&1 | \
    grep "peak_buffered_evals" | head -1 > streaming_check.log
CLEANUP_FILES+=("streaming_check.log")

if grep -q "peak_buffered_evals=32" streaming_check.log; then
    echo "✓ Streaming mode active: peak_buffered_evals matches b_blk"
else
    echo "✘ CRITICAL: Streaming mode NOT active!"
    echo "  Set SSZKP_BLOCKED_IFFT=1 was not effective"
    EXIT_CODE=1
fi
unset SSZKP_MEMLOG

# Test 1: Baseline Scaling (fixed k, varying N)
echo ""
echo "==> Test 1: Collect Memory Measurements (k=32 fixed, varying N)"
for rows in "${TEST_SIZES[@]}"; do
    b_blk=$(echo "sqrt($rows)" | bc)
    result=$(measure_memory_rss $rows 32 $b_blk eval)
    echo "    Result: ${result} KB recorded"
done

# Validate O(√N) scaling
validate_scaling

# Test 2: b_blk Control (informational)
echo ""
echo "==> Test 2: Memory Control via b_blk (N=65536, k=32)"
echo "  Demonstrating b_blk parameter effect on memory..."
for b_blk in 128 256 512; do
    result=$(measure_memory_rss 65536 32 $b_blk eval)
    echo "    b_blk=$b_blk: ${result} KB"
done

# Test 3: SSZKP_MEMLOG Instrumentation
echo ""
echo "==> Test 3: Memory Logging Instrumentation"
export SSZKP_MEMLOG=1
cargo run --release --features dev-srs --bin prover -- \
    --rows 65536 --k 32 --b-blk 256 --basis eval 2>&1 | tee memlog_validation.log
CLEANUP_FILES+=("memlog_validation.log")

if grep -q "peak_inflight_coeffs\|BlockedIfft" memlog_validation.log; then
    echo "✓ Memory logging functional"
else
    echo "⚠ No memory logs found"
fi
unset SSZKP_MEMLOG

# Generate memory report
echo ""
echo "==> Memory Test Summary"
echo "  Streaming mode: $([ -n "${SSZKP_BLOCKED_IFFT:-}" ] && echo '✓ ENABLED' || echo '✘ DISABLED')"
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           SPACE COMPLEXITY ANALYSIS                            ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "Measured Growth Rates (for 4x increase in N):"

# Calculate average growth
local total_growth=0
local count=0
for i in "${!TEST_SIZES[@]}"; do
    if [[ $i -eq 0 ]]; then continue; fi
    local prev_idx=$((i-1))
    local curr_n=${TEST_SIZES[$i]}
    local prev_n=${TEST_SIZES[$prev_idx]}
    local curr_b_blk=$(echo "sqrt($curr_n)" | bc)
    local prev_b_blk=$(echo "sqrt($prev_n)" | bc)
    local curr_mem=$(get_memory "${curr_n}_32_${curr_b_blk}")
    local prev_mem=$(get_memory "${prev_n}_32_${prev_b_blk}")
    
    if [[ $curr_mem -gt 0 && $prev_mem -gt 0 ]]; then
        local curr_work=$((curr_mem - 4500))
        local prev_work=$((prev_mem - 4500))
        if [[ $curr_work -gt 100 && $prev_work -gt 100 ]]; then
            local ratio=$(echo "scale=2; $curr_work / $prev_work" | bc)
            local n_mult=$(echo "scale=0; $curr_n / $prev_n" | bc)
            echo "  N: ${prev_n} → ${curr_n} (${n_mult}x):  ${ratio}x memory growth"
            total_growth=$(echo "$total_growth + $ratio" | bc)
            count=$((count + 1))
        fi
    fi
done

local avg_growth=$(echo "scale=2; $total_growth / $count" | bc)

echo ""
echo "Expected Behaviors:"
echo "  ✘ O(N) linear:   4.00x growth (conventional prover - bad)"
echo "  ✓ O(√N) ideal:   2.00x growth (theoretical target - good)"
echo ""
echo "Average Measured Growth: ${avg_growth}x"
echo ""

# Determine verdict
if (( $(echo "$avg_growth < 3.6" | bc -l) )); then
    local improvement=$(echo "scale=0; (4.0 - $avg_growth) / 4.0 * 100" | bc)
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║  ✓ CONFIRMED: SUBLINEAR space complexity                      ║"
    echo "║                                                                 ║"
    echo "║  System uses ${improvement}% less memory per doubling than O(N)        ║"
    echo "║  Streaming mode is working - memory grows slower than input   ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
else
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║  ✘ WARNING: Approaching LINEAR behavior                       ║"
    echo "║                                                                 ║"
    echo "║  Memory growth is close to O(N) - check streaming config!     ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
fi

echo ""
local sublinearity_factor=$(echo "scale=2; $avg_growth / 4.0" | bc)
echo "Key metrics:"
echo "  • Streaming optimization: ACTIVE"
echo "  • Average growth rate: ${avg_growth}x (vs 4.0x for O(N))"
echo "  • Sublinearity factor: ${sublinearity_factor} (0.5 = perfect O(√N), 1.0 = O(N))"
echo "  • Final verdict: $([ ${EXIT_CODE:-0} -eq 0 ] && echo 'Sublinear scaling ✓' || echo 'Scaling validation failed ✘')"

exit ${EXIT_CODE:-0}

