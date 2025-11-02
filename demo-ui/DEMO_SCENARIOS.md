# Demo Scenarios - Medical Authorization Portal

This document outlines the three key demo scenarios showcasing different authorization outcomes.

## 🎬 Scenario Overview

The demo demonstrates three distinct authorization results:
1. ✅ **Auto-Approved** - No prior authorization required
2. 🟡 **Prior Auth Required** - Meets criteria but PA needed
3. ❌ **Denied** - Does not meet policy criteria

---

## ✅ Scenario 1: Auto-Approved (CT Chest)

**Patient**: PAT002.pdf
- **DOB**: 1985-03-15 (Age: 40)
- **Sex**: Male
- **ICD-10 Codes**: 
  - J18.9 (Pneumonia, unspecified organism)
  - E11.65 (Type 2 diabetes with hyperglycemia)
- **Place of Service**: 22 (Outpatient Hospital)
- **Units**: 1

**Procedure**: 71250 - CT Chest without contrast

**Policy**: 
- **ID**: UHC-COMM-CT-CHEST-001
- **Requires PA**: ❌ **FALSE** (Auto-approve)
- **Inclusion Criteria**:
  - Age: 18-100 ✓
  - ICD-10: Respiratory conditions (Pneumonia matches) ✓
  - Place of Service: 11, 21, 22, 23, 24 ✓
- **Exclusion Criteria**:
  - Not pregnant ✓

**Expected Result**: **APPROVE**
- **Message**: "Authorization approved. Procedure automatically cleared for scheduling."
- **ZKP Proof**: Generated and verified
- **Privacy**: No patient data shared with payer, only proof of medical necessity

**Why It Works**:
- CT Chest for respiratory conditions (pneumonia) is considered routine diagnostic imaging
- No prior authorization required for established diagnostic protocols
- Patient meets all age and clinical criteria

---

## 🟡 Scenario 2: Prior Auth Required (Physical Therapy)

**Patient**: PAT002.pdf (same patient as above)
- **ICD-10 Codes**: J18.9, E11.65

**Procedure**: 97110 - Physical Therapy - Therapeutic Exercise

**Policy**:
- **ID**: Medicare Article 57067
- **Requires PA**: ✅ **TRUE**
- **Inclusion Criteria**:
  - Age: 18-80 ✓
  - ICD-10: Must be in approved list
  - Place of Service: 22, 24 ✓

**Expected Result**: **NEEDS_PA**
- **Message**: "Prior authorization required. Submit additional documentation."
- **ZKP Proof**: Generated and verified
- **Next Steps**: Clinician must submit PA request with supporting documentation

**Why PA Required**:
- Physical therapy requires clinical justification (treatment plan, frequency, duration)
- Insurer needs to verify medical necessity for ongoing treatments
- Prevents overutilization while ensuring appropriate care

---

## ❌ Scenario 3: Denied (Wrong Criteria)

**Patient**: PAT003.pdf
- **DOB**: 1995-06-20 (Age: 30)
- **Sex**: Female
- **ICD-10 Codes**: 
  - C50.912 (Malignant neoplasm of unspecified site of left female breast)
- **Place of Service**: 11 (Office) ❌ **Incorrect!**
- **Units**: 1

**Procedure**: 27447 - Total Knee Replacement

**Policy**:
- **ID**: Medicare Article (Orthopedic Surgery)
- **Requires PA**: TRUE
- **Inclusion Criteria**:
  - Age: 18-80 ✓
  - ICD-10: Must be orthopedic condition (Breast cancer does not match) ❌
  - Place of Service: 22, 24 (PAT003 has POS=11) ❌

**Expected Result**: **DENY**
- **Message**: "Authorization denied. Patient does not meet policy criteria."
- **ZKP Proof**: Generated and verified (proves denial is justified)
- **Reason**: ICD-10 mismatch (breast cancer not orthopedic) AND wrong place of service

**Why It's Denied**:
- Total knee replacement requires orthopedic diagnosis (arthritis, injury, etc.)
- Breast cancer (C50.912) is not a qualifying condition
- Office setting (POS=11) inappropriate for major surgery
- Should be hospital outpatient (22) or ambulatory surgical center (24)

---

## 🎯 Demo Flow Recommendations

### **Opening Statement**:
> "Today we're demonstrating a privacy-preserving medical authorization system using zero-knowledge proofs. We'll show three scenarios: automatic approval, prior auth required, and denial. In each case, the patient's medical data remains private—only the proof of medical necessity is shared with the payer."

### **Demo Sequence**:

1. **Start with Auto-Approve** (Positive first impression)
   - Upload PAT002.pdf
   - Select "71250 - CT Chest"
   - Show instant approval
   - Highlight: "No patient data shared, only cryptographic proof"

2. **Show Prior Auth** (Real-world complexity)
   - Same patient (PAT002)
   - Select "97110 - Physical Therapy"
   - Show "NEEDS_PA" result
   - Explain: "System verifies criteria met, but PA required per policy"

3. **Demonstrate Denial** (System integrity)
   - Upload PAT003.pdf
   - Select "27447 - Total Knee Replacement"
   - Show "DENY" result
   - Explain: "ZKP proves patient doesn't meet criteria, all while keeping data private"

### **Key Talking Points**:

✅ **Privacy**: "At no point does the payer see the patient's diagnosis, age, or medical history"

✅ **Verification**: "The payer can cryptographically verify the decision is correct"

✅ **Compliance**: "HIPAA-compliant by design—no PHI transmitted"

✅ **Efficiency**: "Instant authorization decisions for routine procedures"

✅ **Transparency**: "Both parties see the same proof and can independently verify"

---

## 📊 Technical Details

**ICD-10 Hash Values** (for reference):
- J18.9 (Pneumonia): `15079`
- E11.65 (Type 2 diabetes): `44790`
- E11.9 (Type 2 diabetes): `12029`
- C34.90 (Lung cancer): `7552`
- R91.1 (Pulmonary nodule): `50697`
- C50.912 (Breast cancer): `15564`

**Place of Service Codes**:
- 11: Office
- 21: Inpatient Hospital
- 22: Outpatient Hospital
- 23: Emergency Room
- 24: Ambulatory Surgical Center

**Policy Files**:
- Auto-approve: `/policies/71250.json`
- PA required: `/policies/97110.json`, `/policies/27447.json`
- All policies: `/policies/*.json` (270+ Medicare policies)

---

## 🚀 Running the Demo

1. **Start the demo**: http://localhost:3000
2. **Patient files**: Available in `demo-ui/static/patients/PAT001-PAT010.pdf`
3. **Drag and drop** patient PDF
4. **Wait 2.5 seconds** for "processing" (simulates LLM extraction)
5. **Select procedure** from showcase list
6. **Submit** and view result with ZKP proof

**Demo is production-ready!** ✨

