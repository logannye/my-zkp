# ZK-Agent: Privacy-Preserving Medical Authorization

Zero-knowledge proof system for medical claim authorization that proves decisions follow published payer rules without exposing patient PHI.

## 🎯 Overview

This system allows healthcare providers to prove that a patient meets authorization criteria for medical procedures **without revealing the patient's medical data** to payers. It uses zero-knowledge proofs (ZKPs) to provide:

- ✅ **Integrity**: Decisions provably follow published, verifiable rules
- ✅ **Privacy**: No PHI exposure beyond the authorization outcome
- ✅ **Auditability**: Every decision can be re-verified anytime

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (stable toolchain)
- Cargo

### Installation

```bash
# Build the project
cd zk-agent
cargo build --release

# The binary will be at: ./target/release/authz
```

### Generate Your First Proof

```bash
# 1. Generate a proof
cargo run --release --bin authz -- prove \
  --policy ../policies/UHC-COMM-BIOPSY-001.json \
  --patient ../patients/p002-needs-pa.json \
  --code 19081 \
  --lob commercial \
  --out ../out/p002_biopsy_proof.json

# 2. Verify the proof
cargo run --release --bin authz -- verify \
  ../out/p002_biopsy_proof.json
```

## 📋 How It Works

### 1. Policy Definition (Public)

Payers publish authorization policies as JSON files with inclusion/exclusion criteria:

```json
{
  "policy_id": "UHC-COMM-BIOPSY-001",
  "version": "2025-10-01",
  "lob": "commercial",
  "codes": ["19081"],
  "requires_pa": true,
  "inclusion": [
    {"gte": ["age_years", 18]},
    {"in": ["primary_icd10", [1001, 1002, 1003]]}
  ],
  "exclusion": [
    {"eq": ["pregnant", 1]}
  ],
  "admin_rules": {
    "pos_allowed": [11, 22],
    "max_units_per_day": 1
  }
}
```

**Policy Hash**: Computed as `SHA256(canonical_json)` to ensure immutability.

### 2. Patient Features (Private)

Provider extracts integer features from patient medical records:

```json
{
  "patient_id": "local-uuid-456",
  "dob": "1979-03-15",
  "sex": "F",
  "icd10_list": ["C50.912"],
  "pregnant": false,
  "place_of_service": 22,
  "units": 1
}
```

**Patient Commitment**: `SHA256(features || salt)` hides the patient data.

### 3. Authorization Trace

The system converts authorization logic into a computation trace:

```
Row 0: Verify patient commitment
Row 1: Verify policy hash
Row 2-N: Evaluate inclusion criteria (AND logic)
Row N+1-M: Evaluate exclusion criteria (OR logic)
Row M+1-P: Evaluate admin rules (POS, max units)
Row P+1: Final decision (APPROVE/NEEDS_PA/DENY)
```

### 4. ZKP Proof Generation

The trace is fed to the ZKP engine (my-zkp) to generate a cryptographic proof.

**What's proven:**
- ✅ The policy hash is correct (specific policy version was used)
- ✅ The patient commitment is valid (specific patient evaluated)
- ✅ The authorization result is correct (logic followed correctly)

**What's hidden:**
- ❌ Patient age, sex, diagnoses (features stay private)
- ❌ Intermediate evaluation steps (which criteria passed/failed)
- ❌ All other medical information

### 5. Verification

Payer verifies the proof to confirm the authorization decision without learning any patient data.

## 📊 CLI Usage

### `authz prove`

Generate a ZKP proof for an authorization decision.

```bash
authz prove \
  --policy <PATH>      # Path to policy JSON file
  --patient <PATH>     # Path to patient JSON file
  --code <CPT>         # CPT/HCPCS code (informational)
  --lob <LOB>          # Line of business (informational)
  --out <PATH>         # Output path for decision record + proof
```

**Output**: A decision record JSON file containing:
- Policy ID and hash
- Patient commitment (no PHI)
- Authorization result (APPROVE/NEEDS_PA/DENY)
- ZKP proof (base64-encoded)

### `authz verify`

Verify a ZKP proof.

```bash
authz verify <PROOF_FILE>
```

**Output**: Verification status (✅ VERIFIED or ❌ FAILED)

## 🧪 Testing

### Run Unit Tests

```bash
cargo test
```

### Run Integration Tests

```bash
# Test all patient-policy combinations
cargo test --test test_end_to_end
```

### Test with Sample Data

```bash
# Approve case (no PA required)
cargo run --release --bin authz -- prove \
  --policy ../policies/UHC-COMM-CT-CHEST-001.json \
  --patient ../patients/p001-approve.json \
  --code 71250 \
  --lob commercial \
  --out ../out/p001_ct_proof.json

cargo run --release --bin authz -- verify ../out/p001_ct_proof.json
```

## 📁 Project Structure

```
zk-agent/
├── src/
│   ├── lib.rs              # Public library interface
│   ├── main.rs             # CLI entry point
│   ├── policy.rs           # Policy parsing & canonicalization
│   ├── patient.rs          # Patient feature extraction
│   ├── commitment.rs       # Cryptographic hashing
│   ├── criterion.rs        # Criterion evaluation logic
│   ├── trace.rs            # Computation trace builder
│   ├── decision.rs         # Authorization result & decision record
│   ├── icd_map.rs          # ICD-10 to integer mapping
│   └── cli/
│       ├── mod.rs
│       ├── prove.rs        # Prove command
│       └── verify.rs       # Verify command
├── tests/                  # Integration tests
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

## 🔐 Security & Privacy

### What the ZKP Guarantees

**Integrity**: The authorization result was computed by following the exact published policy rules. The policy hash ensures no one can claim a different policy was used.

**Privacy**: The verifier learns:
- ✅ The authorization outcome (Approve/PA/Deny)
- ✅ The policy version used (hash)
- ✅ That a specific patient was evaluated (commitment)

The verifier does NOT learn:
- ❌ Patient age, sex, diagnoses, or any other features
- ❌ Which specific criteria passed or failed
- ❌ How close the patient was to meeting criteria

**Auditability**: Proofs can be stored and re-verified years later. Policy changes are transparent (new hash = new policy).

### What the ZKP Does NOT Guarantee

- **This is not a production payer policy**: Policies are simulated for demo purposes
- **Policy authorship**: The system assumes the payer publishes official policies
- **Medical necessity**: ZK proves you followed the policy correctly, not that the policy itself is medically appropriate

## 🎓 Development Guide

### Adding New ICD-10 Codes

Edit `src/icd_map.rs`:

```rust
m.insert("Z99.999", 9999);  // Add new mapping
```

### Adding New Criterion Operators

Edit `src/criterion.rs` to support new operators beyond:
- `eq`, `neq`, `lt`, `lte`, `gt`, `gte`, `in`

### Extending Patient Features

1. Add field to `PatientFeatures` struct in `src/patient.rs`
2. Update `extract_features()` to populate the new field
3. Update `to_bytes()` for commitment
4. Update `Criterion::get_field_value()` to support the new field

## 📚 References

- **ZKP Engine**: Built on top of [my-zkp](../README.md) (O(√N) memory streaming ZKP system)
- **Hackathon Spec**: See [HACKATHON_PROJECT_SPEC.md](../HACKATHON_PROJECT_SPEC.md)
- **Test Data**: Policies and patients in `../policies/` and `../patients/`

## 📝 License

MIT

## 🤝 Contributing

This is a hackathon project demonstrating privacy-preserving medical authorization with ZKP. Contributions welcome!

## 🔗 Integration

This library is designed to work with the `my-zkp` ZKP engine. To use in your own project:

```toml
[dependencies]
zk-agent = { path = "path/to/zk-agent" }
myzkp = { path = "path/to/my-zkp" }
```

See `src/lib.rs` for API documentation.

