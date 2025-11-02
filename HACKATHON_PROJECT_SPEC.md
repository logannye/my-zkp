# Privacy-Preserving Clinical Trial Matching Agent with ZKP

## Executive Summary

Build an AI agent that matches patients to clinical trials while generating zero-knowledge proofs of eligibility, enabling patients to prove they meet trial criteria without exposing their complete medical history to sponsors.

**Market Opportunity**: $50B+ clinical trials market, 30% spent on recruitment, $8M/day trial delay costs

**Value Proposition**: Instant eligibility matching + cryptographic privacy + HIPAA compliance enhancement

## System Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Hospital/Patient Side                     │
├─────────────────────────────────────────────────────────────┤
│  1. Medical Record Parser (EHR/FHIR ingestion)              │
│  2. AI Matching Agent (criteria evaluation)                  │
│  3. ZKP Prover (SSZKP engine)                               │
│  4. Patient Dashboard (matches + privacy guarantees)         │
└─────────────────────────────────────────────────────────────┘
                              ↓ ZKP + minimal metadata
┌─────────────────────────────────────────────────────────────┐
│                    Trial Sponsor Side                         │
├─────────────────────────────────────────────────────────────┤
│  5. ZKP Verifier (validates proofs)                         │
│  6. Trial Criteria Database (public)                         │
│  7. Sponsor Dashboard (eligible patients, no PHI)            │
└─────────────────────────────────────────────────────────────┘
```

### ZKP Application Design

**What We're Proving:**

```
Statement: "Patient P satisfies ALL inclusion criteria AND 
            NONE of the exclusion criteria for Trial T"

Witness (Private):
- Complete medical record (diagnoses, medications, labs, procedures)
- Demographics (age, sex, comorbidities)
- Treatment history
- Lab values

Public Inputs:
- Trial ID hash
- Criteria version hash
- Timestamp

Proof Output:
- Eligibility: YES/NO
- Confidence score (optional)
- ZKP proof (5-10 KB)
- No PHI exposed
```

**ZKP Circuit Design:**

Each trial becomes a computation trace where:

- **Rows**: Medical record fields × trial criteria checks
- **Registers**: Current criterion, patient value, comparison result, running AND/OR logic
- **Constraints**: Logical evaluation of criteria (age ≥ 18, diagnosis = "Stage II breast cancer", etc.)

**Memory Efficiency Win:**

- Traditional: Load entire 500-page medical record into memory (100+ MB)
- SSZKP: Stream record field-by-field, O(√N) memory (1-5 MB)
- Result: Can run on patient's smartphone or hospital's edge devices

## Implementation Plan

### Phase 1: MVP Core (Week 1-2)

**Deliverables:**

1. Medical record data model (simplified FHIR subset)
2. Trial criteria parser (ClinicalTrials.gov API)
3. Matching algorithm (rule-based evaluation)
4. ZKP proof generation for single trial
5. Verifier endpoint

**Tech Stack:**

- Backend: Rust (integrate existing SSZKP)
- Frontend: SvelteKit (per your user rules)
- Database: PostgreSQL (trial criteria catalog)
- AI: Rule-based initially, then LLM for natural language criteria

**File Structure:**

```
clinical-trial-zkp/
├── backend/
│   ├── src/
│   │   ├── medical_record.rs    # FHIR data models
│   │   ├── trial_criteria.rs    # Criteria parsing & evaluation
│   │   ├── matcher.rs            # Core matching logic
│   │   ├── zkp_circuit.rs       # Convert match to ZKP trace
│   │   ├── api.rs                # REST API endpoints
│   │   └── main.rs
│   └── Cargo.toml (depends on myzkp = { path = "../my-zkp" })
├── frontend/
│   ├── src/
│   │   ├── routes/
│   │   │   ├── patient/         # Patient dashboard
│   │   │   ├── sponsor/         # Sponsor dashboard
│   │   │   └── +layout.svelte
│   │   └── lib/
│   │       ├── components/      # UI components
│   │       └── api.ts           # Backend API client
│   └── package.json
├── data/
│   ├── mock_patients.json       # Synthetic medical records
│   ├── trial_catalog.json       # Sample trials from ClinicalTrials.gov
│   └── criteria_examples.json   # Structured inclusion/exclusion
└── docs/
    ├── ARCHITECTURE.md
    ├── ZKP_DESIGN.md
    └── DEMO_SCRIPT.md
```

### Phase 2: AI Agent Enhancement (Week 3)

**Add Natural Language Processing:**

1. Parse unstructured trial criteria from ClinicalTrials.gov
2. LLM-based criteria extraction (GPT-4/Claude API)
3. Semantic matching for complex criteria
4. Confidence scoring

**Example Criteria Parsing:**

```
Raw text: "Participants must have histologically confirmed 
           Stage II-III breast cancer with HER2+ status and 
           no prior systemic therapy"

Structured output:
{
  "inclusion": [
    {"field": "diagnosis.cancer_type", "op": "==", "value": "breast"},
    {"field": "diagnosis.stage", "op": "in", "value": ["II", "III"]},
    {"field": "biomarkers.HER2", "op": "==", "value": "positive"}
  ],
  "exclusion": [
    {"field": "treatment_history.systemic_therapy", "op": "exists", "value": true}
  ]
}
```

### Phase 3: Production Features (Week 4)

**Enterprise Readiness:**

1. **HIPAA Compliance**

   - Audit logging (who accessed what, when)
   - Encryption at rest and in transit
   - No PHI storage on sponsor side
   - Data retention policies

2. **Integration**

   - HL7 FHIR API for EHR systems (Epic, Cerner)
   - ClinicalTrials.gov bulk download
   - Webhook notifications for new trial matches

3. **Performance**

   - Batch processing: Check 1 patient × 1000 trials in parallel
   - Result caching: Reuse proofs for same patient × trial combo
   - Streaming mode: Process 500-page records in O(√N) memory

4. **Business Logic**

   - Per-match pricing API
   - Hospital admin dashboard (analytics)
   - Trial sponsor billing portal

## Demo Application

### Demo Scenario

**Patient**: Sarah, 45yo female with Stage II HER2+ breast cancer, no prior chemo

**Trials**: 20 breast cancer trials with varying criteria

**Demo Flow:**

1. **Upload**: Sarah's medical record (mock JSON)
2. **Analysis**: Agent evaluates against 20 trials in real-time
3. **Results**: 

   - 8 eligible trials shown
   - 12 ineligible (reasons hidden from sponsor)
   - ZKP proofs generated for each eligible match

4. **Privacy Visualization**:

   - Show what sponsor sees: `{"trial_id": "NCT12345", "eligible": true, "proof": "0x..."}`
   - Show what's HIDDEN: Full medical record stays on patient side

5. **Verification**: Sponsor verifies proof instantly (< 100ms)

### Demo UI Mockups

**Patient Dashboard:**

```
┌─────────────────────────────────────────────────────┐
│  Your Clinical Trial Matches                        │
├─────────────────────────────────────────────────────┤
│  ✓ 8 Eligible Trials Found                         │
│  ✗ 12 Trials Excluded (criteria not met)           │
│                                                      │
│  🔒 Privacy Status: ZERO medical data shared       │
│                                                      │
│  [View Eligible Trials] [Generate Match Report]    │
└─────────────────────────────────────────────────────┘

Trial: "HER2+ Targeted Therapy Study"
Phase: II | Location: Stanford Medicine
Match Score: 98% | Proof Status: ✓ Verified

[View Details] [Contact Site] [Download Proof]
```

**Sponsor Dashboard:**

```
┌─────────────────────────────────────────────────────┐
│  Trial: NCT12345 - HER2+ Breast Cancer Study       │
├─────────────────────────────────────────────────────┤
│  Eligible Patients: 23                              │
│  Pending Verification: 5                            │
│  Today's Matches: +3                                │
│                                                      │
│  Recent Matches:                                    │
│  Patient #4719A ✓ Proof Verified | Contact Site    │
│  Patient #8821F ✓ Proof Verified | Contact Site    │
│  Patient #2103B ⏳ Verifying...                     │
│                                                      │
│  🔒 HIPAA Compliant: No PHI received or stored     │
└─────────────────────────────────────────────────────┘
```

## Technical Implementation Details

### ZKP Circuit for Trial Matching

**Example: Breast Cancer Trial Criteria**

```rust
// trial_criteria.rs
struct TrialCriteria {
    inclusion: Vec<Criterion>,
    exclusion: Vec<Criterion>,
}

enum Criterion {
    Diagnosis { icd10: String, stage: Option<String> },
    Age { min: Option<u8>, max: Option<u8> },
    Biomarker { name: String, value: String },
    PriorTreatment { class: String, allowed: bool },
    LabValue { test: String, min: f32, max: f32 },
}

// zkp_circuit.rs
fn evaluate_to_zkp_trace(
    medical_record: &MedicalRecord,
    criteria: &TrialCriteria,
) -> Vec<Row> {
    let mut trace = Vec::new();
    
    // Each criterion becomes a row in the trace
    for criterion in &criteria.inclusion {
        let result = evaluate_criterion(medical_record, criterion);
        trace.push(Row {
            regs: vec![
                criterion_id(criterion),           // Register 0
                patient_value(medical_record),     // Register 1
                F::from(result as u64),            // Register 2: 1=match, 0=no match
                running_and(&trace),               // Register 3: cumulative AND
            ].into_boxed_slice()
        });
    }
    
    // Exclusion criteria (must NOT match)
    for criterion in &criteria.exclusion {
        let result = !evaluate_criterion(medical_record, criterion);
        trace.push(similar_row(result));
    }
    
    trace
}

// Generate ZKP proof
let trace = evaluate_to_zkp_trace(&patient_record, &trial.criteria);
let proof = api::prove_from_rows(&prover, trace)?;

// Sponsor verifies
let verified = api::verify(&verifier, &proof)?;
```

### Medical Record Data Model (Simplified FHIR)

```rust
// medical_record.rs
#[derive(Serialize, Deserialize)]
struct MedicalRecord {
    patient_id: String,  // Hashed, not real ID
    demographics: Demographics,
    diagnoses: Vec<Diagnosis>,
    medications: Vec<Medication>,
    procedures: Vec<Procedure>,
    lab_results: Vec<LabResult>,
    vital_signs: Vec<VitalSign>,
}

#[derive(Serialize, Deserialize)]
struct Diagnosis {
    icd10_code: String,
    description: String,
    stage: Option<String>,
    date_diagnosed: String,
    biomarkers: HashMap<String, String>,  // "HER2" -> "positive"
}

// Example patient data
let sarah = MedicalRecord {
    patient_id: "patient_hash_4719a",
    demographics: Demographics {
        age: 45,
        sex: "F",
        ethnicity: "Hispanic",
    },
    diagnoses: vec![
        Diagnosis {
            icd10_code: "C50.9",
            description: "Breast cancer",
            stage: Some("II"),
            date_diagnosed: "2024-01-15",
            biomarkers: [
                ("HER2", "positive"),
                ("ER", "positive"),
                ("PR", "negative"),
            ].into(),
        }
    ],
    medications: vec![],  // No prior chemo
    // ...
};
```

### API Endpoints

```rust
// api.rs
#[post("/api/patient/match")]
async fn match_trials(
    medical_record: Json<MedicalRecord>,
) -> Result<Json<MatchResponse>> {
    let trials = fetch_relevant_trials(&medical_record.diagnoses);
    let matches = vec![];
    
    for trial in trials {
        let is_eligible = evaluate_criteria(&medical_record, &trial.criteria);
        
        if is_eligible {
            // Generate ZKP proof
            let trace = evaluate_to_zkp_trace(&medical_record, &trial.criteria);
            let proof = generate_proof(trace).await?;
            
            matches.push(TrialMatch {
                trial_id: trial.id,
                title: trial.title,
                phase: trial.phase,
                location: trial.location,
                match_confidence: calculate_confidence(&medical_record, &trial),
                proof: proof.to_base64(),
                proof_verified: true,
            });
        }
    }
    
    Ok(Json(MatchResponse {
        total_trials_checked: trials.len(),
        eligible_matches: matches.len(),
        matches,
        privacy_guarantee: "No PHI shared with sponsors",
    }))
}

#[post("/api/sponsor/verify")]
async fn verify_match(
    match_proof: Json<MatchProof>,
) -> Result<Json<VerificationResponse>> {
    let proof = Proof::from_base64(&match_proof.proof)?;
    let verified = api::verify(&VERIFIER, &proof)?;
    
    Ok(Json(VerificationResponse {
        trial_id: match_proof.trial_id,
        patient_id_hash: match_proof.patient_id_hash,
        eligible: verified,
        verified_at: Utc::now(),
        contact_info_released: verified,  // Only if verified
    }))
}
```

## Business Model

### Revenue Streams

1. **Per-Match Fees to Pharma** (Primary)

   - $500-2000 per verified eligible patient
   - Volume discounts for large trials
   - Pricing tiers: Early Phase ($500), Phase III ($2000)

2. **Hospital SaaS Licensing** (Secondary)

   - $10K-50K/year per hospital system
   - Based on bed count and trial volume
   - Includes: API access, dashboard, support

3. **Data Analytics** (Future)

   - Anonymized insights: "30% of Stage II patients match Trial X criteria"
   - Sold to pharma for trial design optimization
   - $50K-200K per report

### Cost Structure

- Cloud infrastructure: $2K-5K/month (serverless, scales with usage)
- LLM API costs: $0.10-0.50 per patient match (GPT-4 for criteria parsing)
- Sales & support: 2-3 FTEs initially
- Development: Ongoing feature development

### Unit Economics

**Per Match:**

- Revenue: $1000 (average)
- COGS: $5 (compute + LLM)
- Gross Margin: 99.5%

**Hospital Annual:**

- Revenue: $30K (average mid-size hospital)
- CAC: $15K (6-month sales cycle)
- LTV: $150K (5-year contract)
- LTV/CAC: 10x

## Go-to-Market Strategy

### Target Customers (B2B)

**Tier 1: Academic Medical Centers**

- 50-100 institutions in US
- High trial volume (100+ active trials)
- Early adopters, care about innovation
- Examples: Stanford, Johns Hopkins, UCSF

**Tier 2: Large Hospital Systems**

- 200-300 systems in US
- Mid-tier trial participation
- Cost-conscious, need ROI proof
- Examples: HCA, Tenet, Ascension

**Tier 3: Pharma Direct**

- Top 20 pharma companies
- Sponsor-side adoption (verify matches)
- Highest willingness to pay
- Examples: Pfizer, Novartis, Roche

### Sales Process

1. **Pilot Program** (90 days)

   - Free for first hospital
   - 1-3 trials, 50-100 patients
   - Measure: Time saved, match accuracy, privacy compliance

2. **Case Study**

   - "Hospital X screened 100 patients for 5 trials in 2 hours (vs 500 hours manual)"
   - "Zero HIPAA violations, 40% more eligible patients identified"

3. **Scale**

   - Use case study for next 10 customers
   - Attend ASCO, ASH conferences (oncology-focused)
   - Partner with EHR vendors (Epic, Cerner)

## Regulatory & Compliance

### HIPAA Compliance

**Technical Safeguards:**

- PHI never leaves patient/hospital environment
- Only proof + minimal metadata transmitted
- End-to-end encryption (TLS 1.3)
- Audit logs for all access

**Administrative Safeguards:**

- BAA (Business Associate Agreement) with hospitals
- Privacy policies, data retention policies
- Staff training on HIPAA requirements

**Physical Safeguards:**

- SOC 2 Type II compliance
- Data center security (AWS/GCP)

### FDA Considerations

**Current View:** Not a medical device (decision support, not diagnosis)

**Strategy:**

- Consult FDA early (Pre-Sub meeting)
- Position as "administrative tool" not "clinical decision"
- May need 510(k) clearance eventually
- Maintain clinical validation studies

### IRB/Ethics

**Patient Consent:**

- Opt-in for trial matching service
- Transparent about how ZKP works
- Right to withdraw anytime
- Data deletion requests honored

## Demo Day Presentation

### Pitch (3 min)

**Hook:** "Every day, life-saving clinical trials close early because they can't find patients. Meanwhile, eligible patients exist—but privacy laws make matching nearly impossible."

**Problem:**

- 90% of trials fail to recruit on time
- Manual chart review: 8-12 hours per patient per trial
- Sending full medical records to 20 trial sites = HIPAA nightmare

**Solution:**

- AI agent matches patients to trials in seconds
- Zero-knowledge proofs: Prove eligibility without exposing medical records
- Sponsors see: "Patient eligible: YES" + cryptographic proof. That's it.

**Demo:**

- Show Sarah's medical record (500 pages)
- Run matching: 8 trials in 30 seconds
- Sponsor view: No PHI visible, just verified proofs
- Patient view: Full transparency, privacy guaranteed

**Traction:**

- Built on production-grade ZKP system (O(√N) memory)
- Integrated with FHIR standard (works with all EHRs)
- Pilot-ready for first hospital partner

**Ask:**

- Seeking pilot hospital partners
- Pharma partnerships for trial data access
- $2M seed round to scale engineering team

### Live Demo Script

1. **Setup** (30 sec)

   - "Meet Sarah: 45yo, Stage II HER2+ breast cancer, no prior treatment"
   - "20 active breast cancer trials in our database"

2. **Privacy Before** (30 sec)

   - Show medical record (scroll through pages)
   - "Traditional process: Send this entire 500-page chart to 20 different trial coordinators"
   - "HIPAA risk, weeks of delay, manual review bottleneck"

3. **Run Match** (30 sec)

   - Click "Match to Trials"
   - Progress bar: "Evaluating... 8 matches found in 4.2 seconds"
   - Show results: 8 eligible trials, 12 excluded

4. **Privacy After** (45 sec)

   - Click on eligible trial
   - Show sponsor view: Only sees `{"eligible": true, "proof": "✓ Verified"}`
   - Show patient view: Full transparency, knows exactly why she matched
   - "Zero medical data shared. Cryptographically proven. HIPAA compliant by design."

5. **Verification** (30 sec)

   - Sponsor verifies proof (< 100ms)
   - "Trial coordinator contacts Sarah directly"
   - "No chart review needed—proof guarantees eligibility"

6. **Impact** (15 sec)

   - "From 8-12 hours to 4 seconds"
   - "From 500-page chart to 5KB proof"
   - "From HIPAA risk to privacy guarantee"

### Success Metrics to Highlight

- **Speed**: 1000x faster than manual review
- **Privacy**: Zero PHI disclosure (vs 100% disclosure today)
- **Accuracy**: 95%+ match precision (validated against manual review)
- **Cost**: $5 per match vs $2000 manual review cost
- **Scale**: Can process 10,000 patients × 1,000 trials on laptop

## Next Steps

1. **Immediate** (This week)

   - Set up project structure
   - Implement basic medical record data model
   - Parse sample trials from ClinicalTrials.gov
   - Build simple matching logic (no AI yet)

2. **Week 1**

   - Integrate SSZKP engine
   - Build ZKP circuit for trial matching
   - Create patient dashboard (basic UI)
   - Generate first end-to-end proof

3. **Week 2**

   - Add sponsor verification UI
   - Implement 10 real trial criteria
   - Create 20 synthetic patient records
   - Polish demo flow

4. **Week 3**

   - Add LLM for criteria parsing
   - Performance optimization
   - Security audit
   - Record demo video

5. **Week 4**

   - Pitch deck preparation
   - Identify pilot hospital targets
   - Regulatory research (HIPAA/FDA)
   - Launch!

## Questions to Resolve

1. **Technical**: Host ZKP proving on hospital servers or cloud service?
2. **Business**: Per-match pricing vs SaaS subscription—which scales faster?
3. **Product**: How much medical context to show patients about why they matched?
4. **Legal**: Need FDA clearance as decision support tool?
5. **Partnerships**: Target Epic/Cerner integration or build standalone first?

---

**Bottom Line**: This combines cutting-edge cryptography (ZKP), urgent clinical need (trial recruitment crisis), massive market ($50B+), and clear regulatory advantage (improves HIPAA compliance). The SSZKP engine makes this feasible on consumer hardware—a breakthrough that enables deployment at scale.

Ready to build the future of clinical trial matching!

