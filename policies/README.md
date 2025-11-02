# Policy Library - Test Data

This directory contains mock payer authorization policies for development and testing of the privacy-preserving medical claim authorization system.

## Policy Files

### UHC-COMM-BIOPSY-001.json
**Procedure**: Breast Biopsy (CPT 19081)  
**Authorization**: Requires Prior Authorization  
**Inclusion Criteria**:
- Age ≥ 18 years
- Primary ICD-10 in {C50.911, C50.912, D05.10} (mapped to {1001, 1002, 1003})

**Exclusion Criteria**:
- Pregnancy

**Admin Rules**:
- Place of Service: Office (11) or Outpatient Hospital (22)
- Max Units: 1 per day

**Expected Outcomes**:
- Meets all criteria → **NEEDS_PA**
- Fails any criterion → **DENY**

---

### UHC-COMM-CT-CHEST-001.json
**Procedure**: CT Chest (CPT 71250, 71260, 71270)  
**Authorization**: Auto-Approve (no PA required)  
**Inclusion Criteria**:
- Age ≥ 18 years
- Primary ICD-10 in {J18.9, C34.90, R91.1} (mapped to {2001, 2002, 2003})

**Exclusion Criteria**:
- Pregnancy

**Admin Rules**:
- Place of Service: Inpatient Hospital (21), Outpatient Hospital (22), Emergency (23)
- Max Units: 1 per day

**Expected Outcomes**:
- Meets all criteria → **APPROVE**
- Fails any criterion → **DENY**

---

### UHC-COMM-MRI-HEAD-001.json
**Procedure**: MRI Brain/Head (CPT 70551, 70552, 70553)  
**Authorization**: Requires Prior Authorization  
**Inclusion Criteria**:
- Age ≥ 18 years
- Age ≤ 80 years
- Primary ICD-10 in {G43.909, S06.0, G40.909, G45.9} (mapped to {3001, 3002, 3003, 3004})

**Exclusion Criteria**:
- Pregnancy

**Admin Rules**:
- Place of Service: Outpatient Hospital (22) or Ambulatory Surgery Center (24)
- Max Units: 1 per day

**Expected Outcomes**:
- Meets all criteria → **NEEDS_PA**
- Fails any criterion → **DENY**

---

### UHC-MEDICARE-COLONOSCOPY-001.json
**Procedure**: Colonoscopy Screening (CPT 45378)  
**Authorization**: Auto-Approve (preventive)  
**Inclusion Criteria**:
- Age ≥ 50 years (Medicare screening guideline)
- Primary ICD-10 in {Z12.11} (mapped to {4001})

**Exclusion Criteria**:
- None (no pregnancy check for Medicare population)

**Admin Rules**:
- Place of Service: Outpatient Hospital (22) or Ambulatory Surgery Center (24)
- Max Units: 1 per day

**Expected Outcomes**:
- Meets all criteria → **APPROVE**
- Fails any criterion → **DENY**

---

### UHC-COMM-PHYSICAL-THERAPY-001.json
**Procedure**: Physical Therapy (CPT 97110)  
**Authorization**: Requires Prior Authorization  
**Inclusion Criteria**:
- Age ≥ 18 years
- Primary ICD-10 in {M54.5, S83.5} (mapped to {5001, 5002})

**Exclusion Criteria**:
- None

**Admin Rules**:
- Place of Service: Office (11) or Outpatient Hospital (22)
- Max Units: 12 per day (monthly session limit)

**Expected Outcomes**:
- Meets all criteria → **NEEDS_PA**
- Fails any criterion → **DENY**

---

### UHC-COMM-SPECIALTY-DRUG-001.json
**Procedure**: Specialty Biologic Drug (HCPCS J1234)  
**Authorization**: Requires Prior Authorization  
**Inclusion Criteria**:
- Age ≥ 18 years
- Age ≤ 75 years
- Primary ICD-10 in {K50.90, M05.9} (mapped to {6001, 6002})

**Exclusion Criteria**:
- Pregnancy (teratogenic risk)

**Admin Rules**:
- Place of Service: Office (11), Inpatient Hospital (21), or Outpatient Hospital (22)
- Max Units: 1 per day

**Expected Outcomes**:
- Meets all criteria → **NEEDS_PA**
- Fails any criterion → **DENY**

---

### UHC-MEDICAID-DENTAL-001.json
**Procedure**: Dental Extraction (CDT D7140)  
**Authorization**: Auto-Approve (essential service)  
**Inclusion Criteria**:
- Age ≥ 6 years (children and adults)
- Primary ICD-10 in {K04.7} (mapped to {7001})

**Exclusion Criteria**:
- None

**Admin Rules**:
- Place of Service: Office (11)
- Max Units: 4 per day (annual limit)

**Expected Outcomes**:
- Meets all criteria → **APPROVE**
- Fails any criterion → **DENY**

---

## ICD-10 to Integer Mapping

For the ZKP system, ICD-10 codes are mapped to integers during feature extraction:

### Breast Cancer / Biopsy
- `C50.911` → `1001` (Malignant neoplasm of unspecified site of right female breast)
- `C50.912` → `1002` (Malignant neoplasm of unspecified site of left female breast)
- `D05.10`  → `1003` (Intraductal carcinoma in situ of unspecified breast)

### Chest / Lung
- `J18.9`   → `2001` (Pneumonia, unspecified organism)
- `C34.90`  → `2002` (Malignant neoplasm of unspecified part of unspecified bronchus or lung)
- `R91.1`   → `2003` (Solitary pulmonary nodule)

### Neurological
- `G43.909` → `3001` (Migraine, unspecified, not intractable, without status migrainosus)
- `S06.0`   → `3002` (Concussion)
- `G40.909` → `3003` (Epilepsy, unspecified, not intractable, without status epilepticus)
- `G45.9`   → `3004` (Transient cerebral ischemic attack, unspecified)

### Screening / Preventive
- `Z12.11`  → `4001` (Encounter for screening for malignant neoplasm of colon)

### Musculoskeletal / Physical Therapy
- `M54.5`   → `5001` (Low back pain)
- `S83.5`   → `5002` (Sprain of cruciate ligament of knee)

### GI / Rheumatology (Specialty Drugs)
- `K50.90`  → `6001` (Crohn's disease, unspecified, without complications)
- `M05.9`   → `6002` (Rheumatoid arthritis, unspecified)

### Dental
- `K04.7`   → `7001` (Periapical abscess without sinus)

### Other (Comorbidities)
- `E11.9`   → `9001` (Type 2 diabetes mellitus without complications)

---

## Policy Hash Computation

Each policy file should be canonicalized (sorted keys, stable arrays) before hashing:

```bash
# Compute policy hash (SHA-256)
cat UHC-COMM-BIOPSY-001.json | jq --sort-keys -c | sha256sum
```

The policy hash is included in the ZKP proof as a public input, ensuring the verifier knows which exact policy version was used.

---

## Testing Matrix

### Core Test Cases

| Policy | Patient | Expected Result | Reason |
|--------|---------|----------------|--------|
| BIOPSY-001 | p002-needs-pa | NEEDS_PA | All criteria met, requires PA |
| BIOPSY-001 | p003-deny-age | DENY | Age < 18 (fails inclusion) |
| BIOPSY-001 | p004-deny-pregnant | DENY | Pregnant (hits exclusion) |
| BIOPSY-001 | p005-deny-pos | DENY | Wrong place of service |
| BIOPSY-001 | p006-deny-units | DENY | Exceeds max units |
| BIOPSY-001 | p009-boundary-age-18 | NEEDS_PA | Exactly 18yo (boundary test) |
| CT-CHEST-001 | p001-approve | APPROVE | All criteria met, no PA required |
| CT-CHEST-001 | p010-ct-male-pregnant-check | APPROVE | Male patient (pregnancy N/A) |
| MRI-HEAD-001 | p007-mri-approve | NEEDS_PA | All criteria met, requires PA |
| MRI-HEAD-001 | p008-mri-deny-old | DENY | Age > 80 (fails upper bound) |

### New Policy Test Cases

| Policy | Patient | Expected Result | Reason |
|--------|---------|----------------|--------|
| MEDICARE-COLONOSCOPY-001 | p011-medicare-colonoscopy | APPROVE | Medicare screening, auto-approve |
| PHYSICAL-THERAPY-001 | p012-pt-approve | NEEDS_PA | Within unit limits, requires PA |
| PHYSICAL-THERAPY-001 | p013-pt-deny-units | DENY | Exceeds max 12 units |
| SPECIALTY-DRUG-001 | p014-drug-approve | NEEDS_PA | All criteria met, requires PA |
| SPECIALTY-DRUG-001 | p015-drug-deny-pregnant | DENY | Pregnant (teratogenic exclusion) |
| MEDICAID-DENTAL-001 | p016-dental-medicaid | APPROVE | Pediatric dental, auto-approve |

### Coverage Summary

- **Total Policies**: 7 (3 imaging, 1 drug, 1 PT, 1 preventive, 1 dental)
- **Total Patients**: 16 
- **Lines of Business**: Commercial (5), Medicare (1), Medicaid (1)
- **Outcomes Tested**: APPROVE (4), NEEDS_PA (5), DENY (7)
- **Age Range**: 12yo (pediatric) to 82yo (elderly)
- **Edge Cases**: Boundary age (exactly 18), upper age limit (>80)

---

## Usage in Development

These policy files are used as inputs to the authorization trace builder:

```rust
// Load policy
let policy_json = std::fs::read_to_string("policies/UHC-COMM-BIOPSY-001.json")?;
let policy: Policy = serde_json::from_str(&policy_json)?;

// Compute policy hash
let policy_hash = compute_policy_hash(&policy_json);

// Use policy to evaluate patient and build ZKP trace
let trace = build_authorization_trace(&patient_features, &policy, &salt);
```

