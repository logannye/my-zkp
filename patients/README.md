# Patient Test Data

This directory contains mock patient records for development and testing of the privacy-preserving medical claim authorization system.

**IMPORTANT**: These are synthetic test records only. No real patient data (PHI) is included.

## Patient Files

### p001-approve.json
**Scenario**: Meets all criteria for CT Chest (auto-approve)  
**Demographics**: 55yo Male  
**Primary ICD-10**: J18.9 (Pneumonia) → `2001`  
**Expected Result with CT-CHEST-001**: **APPROVE** ✓

**Feature Vector**:
```
age_years: 55
sex: 0 (M)
primary_icd10: 2001
pregnant: 0
pos: 22 (outpatient hospital)
units: 1
```

---

### p002-needs-pa.json
**Scenario**: Meets all criteria for Breast Biopsy (requires PA)  
**Demographics**: 45yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **NEEDS_PA** ✓

**Feature Vector**:
```
age_years: 45
sex: 1 (F)
primary_icd10: 1002
pregnant: 0
pos: 22 (outpatient hospital)
units: 1
```

---

### p003-deny-age.json
**Scenario**: Too young (fails age inclusion criterion)  
**Demographics**: 16yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **DENY** (age < 18)

**Feature Vector**:
```
age_years: 16  ← FAILS age ≥ 18
sex: 1 (F)
primary_icd10: 1002
pregnant: 0
pos: 22
units: 1
```

---

### p004-deny-pregnant.json
**Scenario**: Pregnant (hits exclusion criterion)  
**Demographics**: 34yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **DENY** (pregnancy exclusion)

**Feature Vector**:
```
age_years: 34
sex: 1 (F)
primary_icd10: 1002
pregnant: 1  ← TRIGGERS exclusion
pos: 22
units: 1
```

---

### p005-deny-pos.json
**Scenario**: Wrong place of service (fails admin rule)  
**Demographics**: 45yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **DENY** (POS not in allowed list)

**Feature Vector**:
```
age_years: 45
sex: 1 (F)
primary_icd10: 1002
pregnant: 0
pos: 12  ← NOT in allowed list [11, 22]
units: 1
```

**Note**: POS 12 = "Home" (HIPAA code), which is not allowed for biopsy procedures.

---

### p006-deny-units.json
**Scenario**: Exceeds maximum units (fails admin rule)  
**Demographics**: 40yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **DENY** (exceeds max units)

**Feature Vector**:
```
age_years: 40
sex: 1 (F)
primary_icd10: 1002
pregnant: 0
pos: 22
units: 2  ← EXCEEDS max_units_per_day = 1
```

---

### p007-mri-approve.json
**Scenario**: Matches MRI policy (fills testing gap)  
**Demographics**: 45yo Male  
**Primary ICD-10**: G43.909 (Migraine) → `3001`  
**Expected Result with MRI-HEAD-001**: **NEEDS_PA** ✓

**Feature Vector**:
```
age_years: 45
sex: 0 (M)
primary_icd10: 3001
pregnant: 0
pos: 22
units: 1
```

---

### p008-mri-deny-old.json
**Scenario**: Too old for MRI (tests upper age bound)  
**Demographics**: 82yo Female  
**Primary ICD-10**: G45.9 (TIA) → `3004`  
**Expected Result with MRI-HEAD-001**: **DENY** (age > 80)

**Feature Vector**:
```
age_years: 82  ← FAILS age ≤ 80
sex: 1 (F)
primary_icd10: 3004
pregnant: 0
pos: 22
units: 1
```

---

### p009-boundary-age-18.json
**Scenario**: Exactly at age threshold (boundary test)  
**Demographics**: 18yo Female  
**Primary ICD-10**: C50.912 (Left breast cancer) → `1002`  
**Expected Result with BIOPSY-001**: **NEEDS_PA** (tests age ≥ 18 boundary)

**Feature Vector**:
```
age_years: 18  ← EXACTLY at threshold
sex: 1 (F)
primary_icd10: 1002
pregnant: 0
pos: 22
units: 1
```

---

### p010-ct-male-pregnant-check.json
**Scenario**: Male patient (pregnancy exclusion irrelevant)  
**Demographics**: 65yo Male  
**Primary ICD-10**: C34.90 (Lung cancer) → `2002`  
**Expected Result with CT-CHEST-001**: **APPROVE**

**Feature Vector**:
```
age_years: 65
sex: 0 (M)  ← Male, so pregnancy check N/A
primary_icd10: 2002
pregnant: 0
pos: 22
units: 1
```

---

### p011-medicare-colonoscopy.json
**Scenario**: Medicare screening colonoscopy  
**Demographics**: 65yo Male  
**Primary ICD-10**: Z12.11 (Colon screening) → `4001`  
**Expected Result with MEDICARE-COLONOSCOPY-001**: **APPROVE** ✓

**Feature Vector**:
```
age_years: 65
sex: 0 (M)
primary_icd10: 4001
pregnant: 0
pos: 22
units: 1
```

---

### p012-pt-approve.json
**Scenario**: Physical therapy candidate (within session limits)  
**Demographics**: 45yo Female  
**Primary ICD-10**: M54.5 (Low back pain) → `5001`  
**Expected Result with PHYSICAL-THERAPY-001**: **NEEDS_PA** ✓

**Feature Vector**:
```
age_years: 45
sex: 1 (F)
primary_icd10: 5001
pregnant: 0
pos: 11
units: 8  ← WITHIN max 12
```

---

### p013-pt-deny-units.json
**Scenario**: Exceeds PT session limit  
**Demographics**: 35yo Male  
**Primary ICD-10**: S83.5 (Knee sprain) → `5002`  
**Expected Result with PHYSICAL-THERAPY-001**: **DENY** (exceeds max units)

**Feature Vector**:
```
age_years: 35
sex: 0 (M)
primary_icd10: 5002
pregnant: 0
pos: 22
units: 15  ← EXCEEDS max 12
```

---

### p014-drug-approve.json
**Scenario**: Specialty drug candidate  
**Demographics**: 42yo Female  
**Primary ICD-10**: K50.90 (Crohn's disease) → `6001`  
**Expected Result with SPECIALTY-DRUG-001**: **NEEDS_PA** ✓

**Feature Vector**:
```
age_years: 42
sex: 1 (F)
primary_icd10: 6001
pregnant: 0
pos: 11
units: 1
```

---

### p015-drug-deny-pregnant.json
**Scenario**: Pregnant (teratogenic drug exclusion)  
**Demographics**: 28yo Female  
**Primary ICD-10**: M05.9 (Rheumatoid arthritis) → `6002`  
**Expected Result with SPECIALTY-DRUG-001**: **DENY** (pregnancy exclusion)

**Feature Vector**:
```
age_years: 28
sex: 1 (F)
primary_icd10: 6002
pregnant: 1  ← TRIGGERS exclusion
pos: 22
units: 1
```

---

### p016-dental-medicaid.json
**Scenario**: Medicaid dental (pediatric case)  
**Demographics**: 12yo Male  
**Primary ICD-10**: K04.7 (Periapical abscess) → `7001`  
**Expected Result with MEDICAID-DENTAL-001**: **APPROVE** ✓

**Feature Vector**:
```
age_years: 12  ← PEDIATRIC patient
sex: 0 (M)
primary_icd10: 7001
pregnant: 0
pos: 11
units: 1
```

---

## Feature Extraction

Patient JSON files are converted to fixed integer feature vectors during preprocessing:

### Field Mappings

**Sex**:
- `"M"` → `0`
- `"F"` → `1`

**Pregnant**:
- `false` → `0`
- `true` → `1`

**Age Calculation**:
```rust
// Compute age from DOB (as of 2025-11-02)
let dob = parse_date(patient.dob)?;  // e.g., "1979-03-15"
let age_years = 2025 - dob.year;     // Simple year difference for demo
```

**ICD-10 Mapping**:
- First code in `icd10_list` becomes `primary_icd10`
- Mapped using lookup table (see `policies/README.md`)

**Place of Service**:
- HIPAA standard codes used as-is (integers)
- Common codes: 11 (office), 12 (home), 21 (inpatient), 22 (outpatient), 23 (emergency), 24 (ambulatory surgery)

---

## Patient Commitment

The patient commitment is computed from the feature vector + random salt:

```rust
// Feature vector → bytes
let features_bytes = encode_features(&features);  // [age, sex, icd, preg, pos, units]

// Commitment = SHA256(features || salt)
let commitment = sha256(&[features_bytes, salt].concat());
```

**Critical**: The commitment is computed on the **provider side** and never reveals the underlying features. Only the commitment (32-byte hash) is shared with the payer.

---

## Usage in Development

These patient files are used as inputs to the feature extractor and trace builder:

```rust
// Load patient
let patient_json = std::fs::read_to_string("patients/p002-needs-pa.json")?;
let patient: PatientRecord = serde_json::from_str(&patient_json)?;

// Extract features
let features = extract_patient_features(&patient)?;
// features = PatientFeatures {
//     age_years: 45,
//     sex: 1,
//     primary_icd10: 1002,
//     pregnant: 0,
//     pos: 22,
//     units: 1,
// }

// Generate commitment
let salt = generate_random_salt();  // 32 random bytes
let commitment = compute_patient_commitment(&features, &salt);

// Build ZKP trace
let trace = build_authorization_trace(&features, &policy, &salt);
```

---

## Testing Matrix

### Original Test Cases (Imaging Procedures)

| Patient | Policy | Expected | Reason |
|---------|--------|----------|--------|
| p001-approve | CT-CHEST-001 | APPROVE | Meets all criteria, no PA required |
| p002-needs-pa | BIOPSY-001 | NEEDS_PA | Meets all criteria, PA required |
| p003-deny-age | BIOPSY-001 | DENY | Age < 18 (fails inclusion) |
| p004-deny-pregnant | BIOPSY-001 | DENY | Pregnant (hits exclusion) |
| p005-deny-pos | BIOPSY-001 | DENY | POS 12 not in [11, 22] |
| p006-deny-units | BIOPSY-001 | DENY | Units 2 > max 1 |

### Edge Case & Boundary Tests

| Patient | Policy | Expected | Reason |
|---------|--------|----------|--------|
| p007-mri-approve | MRI-HEAD-001 | NEEDS_PA | Male, migraine diagnosis |
| p008-mri-deny-old | MRI-HEAD-001 | DENY | Age 82 > max 80 (upper bound) |
| p009-boundary-age-18 | BIOPSY-001 | NEEDS_PA | Exactly 18yo (boundary test) |
| p010-ct-male-pregnant-check | CT-CHEST-001 | APPROVE | Male (pregnancy N/A) |

### New Service Type Tests

| Patient | Policy | Expected | Reason |
|---------|--------|----------|--------|
| p011-medicare-colonoscopy | MEDICARE-COLONOSCOPY-001 | APPROVE | Medicare screening, age ≥ 50 |
| p012-pt-approve | PHYSICAL-THERAPY-001 | NEEDS_PA | 8 units < max 12, requires PA |
| p013-pt-deny-units | PHYSICAL-THERAPY-001 | DENY | 15 units > max 12 |
| p014-drug-approve | SPECIALTY-DRUG-001 | NEEDS_PA | Meets criteria, requires PA |
| p015-drug-deny-pregnant | SPECIALTY-DRUG-001 | DENY | Pregnant (teratogenic exclusion) |
| p016-dental-medicaid | MEDICAID-DENTAL-001 | APPROVE | Pediatric dental, age ≥ 6 |

**Total Coverage**: 16 patients × 7 policies = 112 possible test combinations

**Comprehensive Coverage**:
- ✓ APPROVE outcome (4 patients)
- ✓ NEEDS_PA outcome (5 patients)
- ✓ DENY - inclusion failure (2 patients)
- ✓ DENY - exclusion hit (2 patients)
- ✓ DENY - admin: place of service (1 patient)
- ✓ DENY - admin: max units (2 patients)
- ✓ Boundary testing (exactly at thresholds)
- ✓ Upper/lower age bounds
- ✓ Gender diversity (M and F)
- ✓ Multiple LOBs (Commercial, Medicare, Medicaid)
- ✓ Pediatric (12yo) to elderly (82yo)
- ✓ Service types: Imaging, PT, drugs, screening, dental

---

## Privacy Note

In the actual system:
- These JSON files represent patient data that **stays on the provider side**
- Only the `patient_commitment` (a 32-byte hash) is transmitted to the payer
- The ZKP proof reveals **nothing** about the patient's age, sex, diagnoses, or other features
- The payer learns **only** the authorization outcome (Approve/PA/Deny) and policy version used

