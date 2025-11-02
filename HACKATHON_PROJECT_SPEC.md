# Privacy-Preserving Medical Claim Authorization - Project Spec

## MVP Goal

Build a zero-knowledge proof system that proves medical claim authorization decisions follow published payer rules without exposing patient PHI.

**Given**: A CPT/HCPCS code and minimal patient facts (age, sex, ICD-10 list, place of service)

**Decide**: "Clinically eligible", "Needs PA", or "Not covered" according to a small set of public-looking rules you control

**Prove (ZK)**: You applied the exact rules (by hash) to a committed patient record to get that outcome—without revealing the patient data

**Scope**: This is a 2-day hackathon MVP. You will NOT integrate with real payer portals. Instead, you'll simulate payer rules with a tiny policy library that looks like real payer policy (e.g., "Applicable Codes", ICD lists, age gates, site-of-service). This is sufficient to demonstrate the product story and zero-knowledge capabilities.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Provider Side (Private)                │
├─────────────────────────────────────────────────────────┤
│  • Patient medical records (stay local)                  │
│  • Feature extraction (JSON → integer vector)           │
│  • Patient commitment (HASH(features || salt))          │
│  • ZK Prover (proves rule evaluation)                   │
└─────────────────────────────────────────────────────────┘
                            ↓
        Only: commitment + proof + claimed_result
                            ↓
┌─────────────────────────────────────────────────────────┐
│                 Payer/Verifier Side (Public)             │
├─────────────────────────────────────────────────────────┤
│  • Policy library (published rules + hashes)            │
│  • ZK Verifier (validates proofs)                       │
│  • Decision dashboard (shows approved/denied/PA)        │
└─────────────────────────────────────────────────────────┘
```

**Privacy Guarantee**: No PHI leaves the provider box except commitments + proofs. The payer learns ONLY the authorization outcome (Approve/Needs PA/Deny) and which policy version was used.

## What You'll Build (End-to-End)

1. **Policy Mini-Library** (YAML/JSON) — 5–10 codes with realistic rules
2. **Patient Feature Extractor** (Python/Rust) — turns mock patient JSON → fixed feature vector + salt + commitment
3. **Deterministic Rules Engine** (pure function) — integer/boolean comparisons only
4. **ZK Prover** (zkVM recommended) — proves engine output came from committed patient features and hashed policy
5. **Verifier API + Dashboard** — verifies proofs and shows "Approved/Needs PA/Denied" with policy/version trace

Everything runs locally. No PHI leaves the "provider" box except commitments + proofs.

## Data Formats (Copy-Pasteable)

### 1. Policy Format (The "Payer Rules")

Canonical JSON structure. You write these once (handcrafted or LLM-assisted). Keep them small.

**Example: Breast Biopsy Policy**

```json
{
  "policy_id": "UHC-COMM-BIOPSY-001",
  "version": "2025-10-01",
  "lob": "commercial",
  "codes": ["19081"],
  "requires_pa": true,
  "inclusion": [
    {"gte": ["age_years", 18]},
    {"in": ["primary_icd10", ["C50.911", "C50.912", "D05.10"]]}
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

**Policy Hash Computation**:
1. Canonicalize (sort keys, stable arrays)
2. Compute: `policy_hash = SHA256(canonical_json)`

**For the hackathon**: Create 5–10 such files covering:
- Common imaging (CT, MRI)
- Biopsy procedures
- Infusion therapy
- Genetic testing
- Simple procedures

Each with short ICD lists (3-10 codes), age gates, pregnancy exclusions, and POS restrictions.

### 2. Patient Features (Provider-Side, Private)

**Input: Mock Patient JSON**

```json
{
  "patient_id": "local-uuid-123",
  "dob": "1970-04-02",
  "sex": "F",
  "icd10_list": ["C50.912", "E11.9"],
  "pregnant": false,
  "place_of_service": 22,
  "units": 1
}
```

**Preprocess → Fixed, Integerized Features**

```json
{
  "features": {
    "age_years": 55,
    "sex": 1,
    "primary_icd10_hash": 83421,
    "pregnant": 0,
    "pos": 22,
    "units": 1
  },
  "salt": "0x3c3b...cafe"
}
```

**Integer Mapping**:
- `sex`: F=1, M=0
- `pregnant`: true=1, false=0
- `primary_icd10_hash`: Map ICD-10 codes to small integers (use hash or lookup table)
- `pos`: Use actual HIPAA place-of-service codes (11=office, 22=outpatient hospital, etc.)

**Commitment (Provider-Side)**:

```
patient_commitment = SHA256(encode(features) || salt)
```

**CRITICAL**: Features and salt are NEVER shared. Only the commitment is public.

### 3. Public Decision Record

This is what leaves the provider system and goes to the payer/verifier:

```json
{
  "policy_id": "UHC-COMM-BIOPSY-001",
  "policy_hash": "0xabc123...",
  "code": "19081",
  "lob": "commercial",
  "claimed_result": "NEEDS_PA",
  "patient_commitment": "0xdef456...",
  "proof": "<BASE64_OR_HEX_PROOF>"
}
```

**No PHI included**. The verifier learns:
- Which policy was used (by hash)
- What code was requested
- The outcome (Approve/Needs PA/Deny)
- That the decision was correctly computed (via proof verification)

The verifier does NOT learn: patient age, sex, diagnoses, or any other medical facts.

## Deterministic Rules Engine Logic

This is the core function you'll run inside the ZK prover. It must be:
- **Pure**: No network calls, no file I/O, no randomness
- **Deterministic**: Same inputs always produce same output
- **Integer-only**: No floating-point arithmetic

**Pseudocode**:

```rust
// ========== INPUTS ==========
// Private (witness):
//   - features: {age_years, sex, primary_icd10_hash, pregnant, pos, units}
//   - salt: random 32 bytes
//   - canonical_policy_json: the full policy text
// Public:
//   - policy_hash_pub
//   - patient_commitment_pub
//   - claimed_result_pub: "APPROVE" | "NEEDS_PA" | "DENY"

// ========== VERIFY COMMITMENTS ==========
assert SHA256(features || salt) == patient_commitment_pub
assert SHA256(canonical_policy_json) == policy_hash_pub

// ========== PARSE POLICY ==========
let policy = parse_json(canonical_policy_json);

// ========== EVALUATE MEDICAL NECESSITY ==========
let inclusion_ok = true;
for clause in policy.inclusion {
    if !eval_clause(clause, features) {
        inclusion_ok = false;
        break;
    }
}

let exclusion_hit = false;
for clause in policy.exclusion {
    if eval_clause(clause, features) {
        exclusion_hit = true;
        break;
    }
}

let medical_ok = inclusion_ok && !exclusion_hit;

// ========== EVALUATE ADMIN RULES ==========
let pos_allowed = features.pos in policy.admin_rules.pos_allowed;
let units_ok = features.units <= policy.admin_rules.max_units_per_day;
let admin_ok = pos_allowed && units_ok;

// ========== DETERMINE OUTCOME ==========
let result;
if !medical_ok {
    result = "DENY";
} else if policy.requires_pa {
    result = "NEEDS_PA";
} else if !admin_ok {
    result = "DENY";
} else {
    result = "APPROVE";
}

// ========== ASSERT CLAIMED RESULT ==========
assert result == claimed_result_pub;

// If we reach here, the proof succeeds
```

**Clause Evaluator (`eval_clause`)**:

Supported operations for hackathon:
- `eq`: Equal (`["eq", ["age_years", 18]]` → features.age_years == 18)
- `neq`: Not equal
- `lt`, `lte`, `gt`, `gte`: Comparison operators
- `in`: Set membership (`["in", ["primary_icd10", [123, 456, 789]]]` → features.primary_icd10 in {123, 456, 789})

**ICD-10 Handling (Simplified for Hackathon)**:

Option A: Pre-map ICD-10 codes to small integers during preprocessing
```
"C50.911" → 1001
"C50.912" → 1002
"D05.10"  → 1003
```

Option B: Use a tiny hash-set checker inside zkVM (bounded list, e.g., max 20 codes)

## Zero-Knowledge Implementation

### Recommended Approach: zkVM (RISC Zero or SP1)

**Why zkVM for the hackathon**:
- Write normal Rust for the rules engine
- VM produces ZK proof automatically
- Easier debugging (can test engine logic outside ZK first)
- Fast prototyping (get to demo faster)

**Prover Inputs**:
- **Private (witness)**: `features`, `salt`, `canonical_policy_json`
- **Public**: `policy_hash`, `patient_commitment`, `code`, `lob`, `claimed_result`

**Prover Output**:
- `proof_blob`: The ZK proof (typically 5-10 KB for zkVM)
- `public_journal`: Echoes all public inputs (policy_hash, patient_commitment, claimed_result, etc.)

**Verifier**:
- **Input**: `proof_blob` + `public_inputs`
- **Output**: `verified` (true/false)

### Alternative Approach: Circuit-Based (Noir)

If you prefer circuits:
- Encode features as field elements
- Use Poseidon hash (efficient in circuits)
- Limit clause counts (e.g., max 6 inclusion, max 6 exclusion, pad with no-ops if needed)
- Requires more upfront constraint design

**Recommendation**: Use zkVM for the hackathon. Can document how to port to circuits (or to the custom my-zkp SSZKP system) post-hackathon.

### Integration with my-zkp Repository

**Option 1: Use my-zkp Directly**

If you want to use the custom SSZKP engine from this repository:
- The rules engine evaluation becomes the "computation trace"
- Each clause evaluation is a row in the trace
- Registers hold: `clause_id`, `patient_value`, `comparison_result`, `running_and/or`
- Commitment verification is part of the trace
- Advantage: Demonstrates O(√N) memory efficiency for large policy sets

**Option 2: Use zkVM (Recommended for Hackathon Speed)**

- RISC Zero or SP1 provides faster development
- Write normal Rust, get ZK "for free"
- Can still reference my-zkp for inspiration on streaming/memory efficiency
- Document how to potentially port to SSZKP post-hackathon

**Integration Note for README**:

> "This hackathon demo uses [RISC Zero/SP1] for rapid prototyping. The deterministic rules engine is designed to be portable to custom ZK backends like the SSZKP system in ../my-zkp for production deployments requiring O(√N) memory efficiency when processing large policy libraries or patient feature sets."

## CLI Implementation

### Prove Command

```bash
authz prove \
  --policy ./policies/UHC-COMM-BIOPSY-001.json \
  --patient ./patients/p001.json \
  --code 19081 \
  --lob commercial \
  --out ./out/p001_19081_proof.json
```

**Output File (`p001_19081_proof.json`)**:

```json
{
  "policy_id": "UHC-COMM-BIOPSY-001",
  "policy_hash": "0xabc123...",
  "patient_commitment": "0xdef456...",
  "code": "19081",
  "lob": "commercial",
  "claimed_result": "NEEDS_PA",
  "proof": "0x..."
}
```

### Verify Command

```bash
authz verify ./out/p001_19081_proof.json
```

**Output**:

```
✓ VERIFIED: true
Policy: UHC-COMM-BIOPSY-001 (v2025-10-01)
Code: 19081
Result: NEEDS_PA
```

### Optional: Verifier API

```bash
# Start verifier service
python verifier/server.py

# Verify via API
curl -X POST http://localhost:8000/verify \
  -F "proof_file=@out/p001_19081_proof.json"
```

**Response**:

```json
{
  "verified": true,
  "policy_id": "UHC-COMM-BIOPSY-001",
  "code": "19081",
  "result": "NEEDS_PA",
  "timestamp": "2025-11-02T15:30:00Z"
}
```

## Demo Preparation

### Policy Library (5-10 Policies to Create)

1. **Breast Biopsy** (CPT 19081)
   - Inclusion: Age ≥18, ICD-10 in {C50.911, C50.912, D05.10}
   - Exclusion: Pregnant
   - Admin: POS in {11, 22}, max 1 unit/day
   - Requires PA: Yes

2. **MRI Brain/Head** (CPT 70551, 70552, 70553)
   - Inclusion: Age ≥18, ICD-10 in {G43.909 (migraine), S06.0 (concussion), G40.909 (epilepsy)}
   - Exclusion: None
   - Admin: POS in {22, 24}, max 1 unit/day
   - Requires PA: Yes

3. **CT Chest** (CPT 71250, 71260, 71270)
   - Inclusion: Age ≥18, ICD-10 in {J18.9 (pneumonia), C34.90 (lung cancer), R91.1 (lung nodule)}
   - Exclusion: Pregnant
   - Admin: POS in {21, 22, 23}, max 1 unit/day
   - Requires PA: No (auto-approve if criteria met)

4. **Infusion Therapy** (CPT 96365, 96366)
   - Inclusion: Age ≥18, ICD-10 in {K50.90 (Crohn's), L40.50 (psoriasis), M05.9 (RA)}
   - Exclusion: None
   - Admin: POS in {11, 19, 22}, max 2 units/day
   - Requires PA: Yes

5. **Genetic Testing** (CPT 81479)
   - Inclusion: Age ≥18, ICD-10 in {C50.919 (breast cancer), C61 (prostate cancer)}
   - Exclusion: None
   - Admin: POS in {11, 81}, max 1 unit/day
   - Requires PA: Yes

6-10. **Additional policies** for variety (orthopedic procedures, cardiac imaging, etc.)

### Patient Scenarios (5-10 Patients to Create)

1. **Patient p001**: Meets all criteria → **APPROVE** (use CT Chest policy with requires_pa=false)
   - 55yo female, ICD J18.9 (pneumonia), POS 22, not pregnant

2. **Patient p002**: Meets medical criteria, requires PA → **NEEDS_PA**
   - 45yo female, ICD C50.912 (breast cancer), POS 22, not pregnant
   - Policy: Breast Biopsy (requires_pa=true)

3. **Patient p003**: Too young → **DENY**
   - 16yo female, ICD C50.912, POS 22, not pregnant
   - Fails age ≥18 inclusion criterion

4. **Patient p004**: Wrong ICD-10 code → **DENY**
   - 50yo female, ICD E11.9 (diabetes), POS 22, not pregnant
   - ICD not in policy inclusion list

5. **Patient p005**: Exclusion criterion hit → **DENY**
   - 30yo female, ICD C50.912, POS 22, **pregnant=true**
   - Hits pregnancy exclusion

6. **Patient p006**: Wrong place of service → **DENY**
   - 45yo female, ICD C50.912, **POS 12** (home), not pregnant
   - POS not in allowed list {11, 22}

7. **Patient p007**: Exceeds max units → **DENY**
   - 45yo female, ICD C50.912, POS 22, not pregnant, **units=2**
   - Exceeds max_units_per_day=1

### Dashboard UI (Optional but Impressive)

**Layout**:

```
┌──────────────────────────────────────────────────────┐
│  ZK Medical Authorization Dashboard                  │
├─────────────────────┬────────────────────────────────┤
│  Policy Library     │  Verification Results          │
│  (Left Panel)       │  (Right Panel)                 │
│                     │                                │
│  □ Breast Biopsy    │  Code: 19081  Result: NEEDS_PA │
│    CPT 19081        │  Policy: UHC-COMM-BIOPSY-001   │
│    Hash: 0xabc...   │  Status: ✓ VERIFIED            │
│                     │  Timestamp: 2025-11-02 15:30   │
│  □ MRI Brain        │  [View Proof Details]          │
│    CPT 70551-70553  │                                │
│    Hash: 0xdef...   │  ────────────────────────────  │
│                     │                                │
│  □ CT Chest         │  Code: 71250  Result: APPROVE  │
│    CPT 71250-71270  │  Policy: UHC-COMM-CT-001       │
│    Hash: 0x123...   │  Status: ✓ VERIFIED            │
│                     │  Timestamp: 2025-11-02 15:32   │
│  [+ Upload Policy]  │  [View Proof Details]          │
│                     │                                │
│                     │  [+ Verify New Proof]          │
└─────────────────────┴────────────────────────────────┘
```

**Features**:
- Left panel: Show policy JSON + hash when clicked
- Right panel: List of verified claims with color coding (green=approved, yellow=needs PA, red=denied)
- Upload proof file for instant verification
- Show verification status with checkmark or X

## Demo Storyboard (5-Minute Demo Flow)

### Opening (30 seconds)

**Say**: 
- "Prior authorization today requires providers to send full patient medical records to payers"
- "This creates privacy risks and delays care—sometimes by weeks"
- "What if we could prove a patient meets authorization criteria without revealing their medical data?"
- "That's what we built with zero-knowledge proofs"

### Policy Introduction (30 seconds)

**Show**: `policies/UHC-COMM-BIOPSY-001.json` in terminal or editor

**Say**:
- "Here's a payer policy for breast biopsy (CPT 19081)"
- "Inclusion criteria: Age ≥18, specific breast cancer ICD-10 codes"
- "Exclusion: Pregnancy"
- "Admin rules: Must be in office or hospital, max 1 unit per day"

**Do**: Compute policy hash

```bash
python scripts/make_policy_hash.py policies/UHC-COMM-BIOPSY-001.json
# Output: policy_hash = 0xabc123...
```

**Say**: "This policy is public and verifiable by anyone. The hash ensures no one can tamper with it."

### Patient Commitment (30 seconds)

**Show**: `patients/p002.json`

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

**Say**:
- "Here's a patient: 45yo female with breast cancer (ICD C50.912)"
- "We extract features—age, sex, diagnosis—and create a commitment"

**Do**: Generate commitment

```bash
python scripts/make_patient_commitment.py patients/p002.json
# Output: patient_commitment = 0xdef456...
```

**Say**: "This commitment is a cryptographic hash. We NEVER send the actual patient data—only this commitment."

### Proof Generation (45 seconds)

**Do**: Run the prover

```bash
authz prove \
  --policy policies/UHC-COMM-BIOPSY-001.json \
  --patient patients/p002.json \
  --code 19081 \
  --lob commercial \
  --out out/p002_biopsy.json
```

**Say** (while it runs):
- "Our rules engine is running inside a zero-knowledge virtual machine"
- "It's evaluating the policy against the patient's features"
- "The ZK system is generating a cryptographic proof"

**Output**: `Proof generated in 3.2 seconds. Size: 8.7 KB`

**Say**:
- "Done! The outcome is: NEEDS_PA (prior authorization required)"
- "The proof is only 8.7 KB—no patient data included"

**Show**: `out/p002_biopsy.json` (briefly)

```json
{
  "policy_id": "UHC-COMM-BIOPSY-001",
  "policy_hash": "0xabc123...",
  "patient_commitment": "0xdef456...",
  "code": "19081",
  "lob": "commercial",
  "claimed_result": "NEEDS_PA",
  "proof": "0x..."
}
```

**Say**: "This is what goes to the payer. Notice: no age, no diagnosis, no PHI. Just the commitment, the outcome, and the proof."

### Verification (30 seconds)

**Do**: Run the verifier

```bash
authz verify out/p002_biopsy.json
```

**Output**:
```
✓ VERIFIED: true
Policy: UHC-COMM-BIOPSY-001 (v2025-10-01)
Code: 19081
Result: NEEDS_PA
```

**Say**:
- "The payer's verifier checks two things:"
- "1. Did the rules engine run correctly?"
- "2. Did it use the committed patient data and the published policy?"
- "The answer is YES—verified in under 100 milliseconds"
- "The payer learns: This patient needs prior authorization. Nothing else."

### Outcome Variants (45 seconds)

**Say**: "Let's see what happens with different patients"

**Case 1: Denied - Too Young**

```bash
authz prove --policy policies/UHC-COMM-BIOPSY-001.json \
            --patient patients/p003.json \
            --code 19081 --out out/p003_biopsy.json
authz verify out/p003_biopsy.json
```

**Output**: `VERIFIED: true, Result: DENY`

**Say**: "Patient p003 is only 16 years old—fails the age ≥18 criterion. Denied."

**Case 2: Denied - Pregnancy Exclusion**

```bash
authz prove --policy policies/UHC-COMM-BIOPSY-001.json \
            --patient patients/p005.json \
            --code 19081 --out out/p005_biopsy.json
authz verify out/p005_biopsy.json
```

**Output**: `VERIFIED: true, Result: DENY`

**Say**: "Patient p005 is pregnant—hits the exclusion criterion. Denied."

**Case 3: Approved - Auto-Approval Policy**

```bash
authz prove --policy policies/UHC-COMM-CT-001.json \
            --patient patients/p001.json \
            --code 71250 --out out/p001_ct.json
authz verify out/p001_ct.json
```

**Output**: `VERIFIED: true, Result: APPROVE`

**Say**: "Patient p001 meets all criteria for a CT chest scan, and this policy doesn't require PA. Approved instantly."

**Say**: "In every case, the proof is cryptographically sound, and zero patient data is revealed."

### Closing (30 seconds)

**Say**: "This system provides three guarantees:"

1. **Integrity**: Decisions follow published, verifiable rules. No one can cheat.
2. **Privacy**: No PHI exposure beyond the outcome. Patient data stays with the provider.
3. **Auditability**: Every decision can be re-verified anytime, even years later.

**Say**: "This solves the prior authorization privacy dilemma: prove medical necessity without exposing patient records."

**Optional**: Show dashboard if built (quick screenshot or live view of verified claims)

## Repository Structure

```
zk-medical-authz/
├── policies/                          # Payer rule library
│   ├── UHC-COMM-BIOPSY-001.json      # Breast biopsy
│   ├── UHC-COMM-MRI-HEAD-001.json    # MRI brain/head
│   ├── UHC-COMM-CT-CHEST-001.json    # CT chest
│   ├── UHC-COMM-INFUSION-001.json    # Infusion therapy
│   └── UHC-COMM-GENETIC-001.json     # Genetic testing
├── patients/                          # Mock patient records
│   ├── p001.json                      # Approve case (CT, no PA)
│   ├── p002.json                      # Needs PA case (biopsy)
│   ├── p003.json                      # Deny - too young
│   ├── p004.json                      # Deny - wrong ICD
│   ├── p005.json                      # Deny - pregnant
│   ├── p006.json                      # Deny - wrong POS
│   └── p007.json                      # Deny - exceeds max units
├── engine/                            # Core authorization logic
│   ├── guest/                         # zkVM guest program (RISC Zero/SP1)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs                 # Deterministic engine + hashing
│   ├── host/                          # CLI wrapper
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                # Prove/verify commands
│   └── common/                        # Shared types
│       └── src/
│           ├── policy.rs              # Policy data structures
│           ├── patient.rs             # Patient feature structures
│           └── decision.rs            # Decision record format
├── verifier/                          # Verification service (optional)
│   ├── requirements.txt
│   └── server.py                      # FastAPI: POST /verify
├── web/                               # Simple dashboard (optional)
│   ├── index.html                     # Policy viewer + results table
│   ├── style.css
│   └── app.js                         # Table rendering + upload
├── scripts/                           # Utilities
│   ├── make_policy_hash.py            # Compute policy hash
│   ├── make_patient_commitment.py     # Compute patient commitment
│   └── generate_mock_data.py          # Generate policies + patients
├── out/                               # Proof outputs
│   └── .gitkeep
├── README.md                          # Hackathon setup guide
└── Cargo.toml                         # Workspace root
```

## Division of Work (2-Day Hackathon)

### Day 1 Morning (4 hours): Setup & Core Logic

#### Person A: Policy Engineering

- [ ] Author 5 policy JSON files with realistic rules
  - Start with breast biopsy, MRI, CT, infusion, genetic testing
  - Each with: inclusion (age + ICD list), exclusion (pregnancy), admin (POS + max units)
- [ ] Implement canonicalization function (Python or Rust)
  - Sort JSON keys alphabetically
  - Ensure stable array ordering
- [ ] Implement policy hash computation (SHA-256)
- [ ] Create `scripts/make_policy_hash.py` for batch hashing
- [ ] Test: Verify same policy JSON produces same hash across runs

#### Person B: Patient Feature Engineering

- [ ] Design fixed integer feature vector schema
  - Document: `age_years: u8, sex: u8, primary_icd10_hash: u32, pregnant: u8, pos: u8, units: u8`
- [ ] Implement patient JSON → features extractor (Python or Rust)
- [ ] Create enum mappings:
  - `sex`: F=1, M=0
  - `pregnant`: true=1, false=0
  - `pos`: HIPAA codes (11, 22, etc.)
- [ ] Implement ICD-10 → integer mapping
  - Simple approach: Hash ICD-10 string to u32, or use lookup table
- [ ] Implement commitment generator: `SHA256(encode(features) || salt)`
- [ ] Create `scripts/make_patient_commitment.py`
- [ ] Test: Verify same features+salt produce same commitment

#### Person C: Rules Engine (Pure Function)

- [ ] Implement deterministic evaluation logic in Rust
- [ ] Write clause evaluator supporting: `eq, neq, lt, lte, gt, gte, in`
- [ ] Implement inclusion logic: `AND` over all clauses
- [ ] Implement exclusion logic: `OR` over all clauses
- [ ] Implement admin rules checking:
  - `pos in allowed_list`
  - `units <= max_units_per_day`
- [ ] Implement decision tree:
  ```rust
  if !medical_ok: DENY
  else if requires_pa: NEEDS_PA
  else if !admin_ok: DENY
  else: APPROVE
  ```
- [ ] Test: Run engine with mock inputs outside ZK (unit tests)

### Day 1 Afternoon (4 hours): ZK Integration

#### Person C (continued): zkVM Guest Program

- [ ] Set up RISC Zero or SP1 project (follow quickstart guide)
- [ ] Move rules engine logic into guest program
- [ ] Add commitment verification:
  - `assert SHA256(features || salt) == patient_commitment_pub`
- [ ] Add policy hash verification:
  - `assert SHA256(canonical_policy_json) == policy_hash_pub`
- [ ] Add result assertion:
  - `assert computed_result == claimed_result_pub`
- [ ] Define public inputs/outputs (journal)
- [ ] Test: Run guest program in dev mode (non-ZK) first

#### Person A: Host CLI (Prover)

- [ ] Implement `authz prove` command in Rust
- [ ] Parse command-line arguments: `--policy, --patient, --code, --lob, --out`
- [ ] Read policy JSON and patient JSON from disk
- [ ] Extract patient features using Person B's extractor
- [ ] Generate random salt (32 bytes)
- [ ] Compute patient commitment
- [ ] Call zkVM prover with:
  - Private inputs: features, salt, policy JSON
  - Public inputs: policy_hash, patient_commitment, code, lob, claimed_result
- [ ] Capture proof blob + journal
- [ ] Write proof to output file (JSON)
- [ ] Test: Generate proof for one policy+patient pair

#### Person B: Integration Testing Support

- [ ] Help debug any issues with feature extraction
- [ ] Verify commitment calculation matches between Python scripts and Rust code
- [ ] Create 2-3 test patient JSONs for integration testing

#### All: Integration Testing

- [ ] End-to-end test: policy → patient → prove → proof file
- [ ] Debug any issues with:
  - JSON parsing
  - Hash mismatches
  - zkVM setup
  - Feature encoding

### Day 2 Morning (4 hours): Verifier & Demo Data

#### Person B: Verifier Implementation

- [ ] Implement `authz verify` CLI command in Rust
- [ ] Parse proof file (read JSON)
- [ ] Extract public inputs + proof blob
- [ ] Call zkVM verifier
- [ ] Return verified status (true/false) + claimed result
- [ ] Pretty-print verification result:
  ```
  ✓ VERIFIED: true
  Policy: UHC-COMM-BIOPSY-001 (v2025-10-01)
  Code: 19081
  Result: NEEDS_PA
  ```
- [ ] Test: Verify proofs from Day 1
- [ ] Test: Verify tampered proof fails

#### Person A: Mock Data Generation

- [ ] Create 5 additional policy JSON files (total 10)
- [ ] Create 7 patient JSON files covering all outcome types:
  - 1x APPROVE (meets all criteria, no PA)
  - 1x NEEDS_PA (meets criteria, requires PA)
  - 2x DENY (age, wrong ICD)
  - 1x DENY (exclusion - pregnant)
  - 1x DENY (admin - wrong POS)
  - 1x DENY (admin - exceeds max units)
- [ ] Document which patient tests which rule/edge case
- [ ] Create matrix: patient × policy → expected outcome
- [ ] Run proof generation for all combinations

#### Person C: Verifier API (Optional)

- [ ] Set up FastAPI project (`verifier/server.py`)
- [ ] Implement `POST /verify` endpoint
  - Accept proof file upload (multipart/form-data)
  - Read proof JSON
  - Call Rust verifier (via subprocess or FFI)
  - Return JSON: `{verified, policy_id, code, result, timestamp}`
- [ ] Test: Upload proof via curl or Postman
- [ ] Optional: Add CORS headers for web dashboard

### Day 2 Afternoon (4 hours): Dashboard & Demo Polish

#### Person B: Web Dashboard (Optional but Impressive)

- [ ] Create `web/index.html` with two-panel layout
- [ ] Left panel: Policy viewer
  - List policy files
  - Click to view JSON + hash
- [ ] Right panel: Verification results table
  - Columns: Code, Result, Verified, Policy ID, Timestamp
  - Color-coded: Green (APPROVE), Yellow (NEEDS_PA), Red (DENY)
- [ ] Add "Upload Proof" button
  - Upload file to verifier API
  - Display result in table
- [ ] Simple CSS for clean look (`web/style.css`)
- [ ] JavaScript for table rendering (`web/app.js`)
- [ ] Test: Verify proof via dashboard UI

#### Person A: Demo Script & Narrative

- [ ] Write demo narrative (see Demo Storyboard above)
- [ ] Prepare terminal commands for 3-4 demo cases:
  1. Breast biopsy → NEEDS_PA
  2. Denied - too young
  3. Denied - pregnant
  4. CT chest → APPROVE
- [ ] Test full demo flow
- [ ] Time the demo (target < 5 minutes)
- [ ] Create slides or notes (optional)
- [ ] Prepare to explain ZK guarantees (integrity, privacy, auditability)

#### Person C: Performance & Code Cleanup

- [ ] Optimize proof generation time (if slow)
- [ ] Clean up code, add comments
- [ ] Remove debug prints
- [ ] Add error handling (graceful failures)
- [ ] Update README with:
  - Prerequisites
  - Setup instructions
  - Quick start commands
  - Demo script

#### All: Final Testing & Dry Run

- [ ] Run all patient-policy combinations
- [ ] Verify all proofs successfully
- [ ] Check for edge cases (empty ICD lists, missing fields, etc.)
- [ ] Dry-run the full demo (time it)
- [ ] Fix any last-minute bugs
- [ ] Commit and push to GitHub

## Technical Guardrails (Keep You on Track)

### Keep It Simple

1. **Fix integer enums upfront** (document in README):
   - `sex`: F=1, M=0
   - `pregnant`: true=1, false=0
   - `pos`: Use actual HIPAA codes (11=office, 22=outpatient, 23=emergency, 24=ambulatory surgery, 81=independent lab)

2. **No floating-point arithmetic anywhere**:
   - Use integers for age (years), units, etc.
   - Compute age as `(current_year - birth_year)` in preprocessing, outside zkVM

3. **Limit clause counts**:
   - ≤6 inclusion clauses per policy
   - ≤6 exclusion clauses per policy
   - Pad with no-op clauses if needed (e.g., `{"eq": [0, 0]}`)

4. **Pick one hash function and use it everywhere**:
   - zkVM path: SHA-256 (available in RISC Zero/SP1)
   - Circuit path: Poseidon (if using Noir)
   - Don't mix hash functions

5. **Version every policy**:
   - Use `policy_id@version` format (e.g., `UHC-COMM-BIOPSY-001@2025-10-01`)
   - Pin policy hashes in a manifest file for easy lookup

6. **Test canonicalization thoroughly**:
   - Different JSON key orders must produce the same hash
   - Arrays must be in stable order (sort ICD lists if needed)

### Avoid Common Pitfalls

1. **Don't use floats** → Will break determinism and zkVM efficiency
2. **Don't fetch data during proof** → All inputs must be provided upfront
3. **Don't use unbounded loops** → zkVM will be slow; pre-define max iterations
4. **Don't mix hash functions** → Commitment hash ≠ policy hash will cause confusion
5. **Don't skip error handling** → Gracefully handle missing files, invalid JSON, etc.
6. **Don't forget salt** → Patient commitment without salt is not secure

## What the ZK Guarantees (Demo Talking Points)

### Integrity ✓

- The outcome (Approve/Needs PA/Deny) was computed by the **exact published rules**
- The policy hash is **independently verifiable** by anyone
- No one can claim a different policy was used (hash mismatch would fail verification)
- The rules engine cannot be tampered with (it's inside the ZK proof)

### Privacy ✓

- Verifier learns **ONLY the outcome** and policy version used
- Patient age, sex, diagnoses, and all other features remain **hidden**
- Even the fact that a patient *almost* qualified is hidden
- Only mathematical commitment is revealed (irreversible without the salt)

### Auditability ✓

- Every decision is **cryptographically bound** to a specific policy version
- Proofs can be stored and **re-verified years later**
- Policy changes are **transparent** (new hash = new policy)
- Disputes can be resolved by checking the proof against the published policy
- Immutable audit trail for compliance (HIPAA, regulatory)

### What It Does NOT Guarantee (Be Honest)

- **This is not UHC's actual policy** (it's a simulation for demo purposes)
- **The policy library is controlled by you** (in production, the payer would publish official policies)
- **This doesn't prove medical necessity** (that's the policy's job; ZK proves you followed the policy correctly)
- **Not a substitute for clinical judgment** (authorization ≠ medical decision-making)

## README Quick Start (Copy-Pasteable)

```markdown
# ZK Medical Authorization - Hackathon Demo

Privacy-preserving medical claim authorization using zero-knowledge proofs.

## Prerequisites

- Rust 1.70+
- Python 3.9+ (for scripts)
- RISC Zero or SP1 toolchain

## Quick Start

### 1. Generate Mock Data

```bash
python scripts/generate_mock_data.py
```

This creates 10 policies and 7 patient records.

### 2. Prove a Claim

```bash
cargo run --release --bin authz -- prove \
  --policy policies/UHC-COMM-BIOPSY-001.json \
  --patient patients/p002.json \
  --code 19081 \
  --lob commercial \
  --out out/p002_biopsy.json
```

Output: `Proof generated in 3.2s. Size: 8.7 KB`

### 3. Verify the Proof

```bash
cargo run --release --bin authz -- verify out/p002_biopsy.json
```

Output:
```
✓ VERIFIED: true
Policy: UHC-COMM-BIOPSY-001 (v2025-10-01)
Code: 19081
Result: NEEDS_PA
```

### 4. Run Full Demo

```bash
./demo.sh
```

This proves and verifies all 7 patient scenarios.

## What's Inside

- **policies/**: 10 payer policy files (JSON)
- **patients/**: 7 mock patient records (JSON)
- **engine/**: ZK rules engine (Rust + zkVM)
- **scripts/**: Policy hash + patient commitment utilities (Python)
- **out/**: Generated proofs

## How It Works

1. **Policy**: Payer publishes authorization rules with a cryptographic hash
2. **Patient**: Provider computes a commitment to patient features (age, sex, ICD-10)
3. **Prove**: Rules engine runs in zkVM, generates proof of correct evaluation
4. **Verify**: Payer verifies proof, learns only the outcome (Approve/Needs PA/Deny)

**Privacy Guarantee**: No PHI leaves the provider. Only commitments + proofs are shared.

## Demo Scenarios

- `p001`: CT chest → APPROVE (no PA required)
- `p002`: Breast biopsy → NEEDS_PA
- `p003`: Too young → DENY
- `p004`: Wrong ICD-10 → DENY
- `p005`: Pregnant (exclusion) → DENY
- `p006`: Wrong place of service → DENY
- `p007`: Exceeds max units → DENY

## Integration with my-zkp

This demo uses [RISC Zero/SP1] for rapid prototyping. The deterministic rules engine is designed to be portable to custom ZK backends like the SSZKP system in `../my-zkp` for production deployments requiring O(√N) memory efficiency.

## License

MIT
```

## Success Metrics for Hackathon Demo

**Technical**:
- [ ] 5-10 policies authored with realistic rules
- [ ] 7+ patient scenarios covering all outcome types
- [ ] End-to-end proof generation + verification working
- [ ] Proof size < 10 KB
- [ ] Proof generation time < 10 seconds (on laptop)
- [ ] Verification time < 1 second

**Demo Impact**:
- [ ] Clear narrative explaining the privacy problem
- [ ] Visual distinction between "what's revealed" vs "what's hidden"
- [ ] Multiple outcome variants demonstrated (approve/PA/deny)
- [ ] ZK guarantees clearly explained (integrity, privacy, auditability)
- [ ] Demo runs smoothly in < 5 minutes

**Code Quality**:
- [ ] Clean, readable code with comments
- [ ] Error handling for common failures
- [ ] README with setup instructions
- [ ] Demo script documented

**Bonus Points**:
- [ ] Web dashboard showing policy viewer + verification results
- [ ] Batch proof generation (all patients × all policies)
- [ ] Performance benchmarking (proof gen time vs policy complexity)
- [ ] Integration note explaining how to port to my-zkp SSZKP system

---

**Ready to build! Good luck with your hackathon! 🚀**
