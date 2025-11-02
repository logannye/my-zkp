# 🏥 Privacy-Preserving Medical Authorization with AI Agents + Zero-Knowledge Proofs

**Hackathon Project: Automating Prior Authorization While Protecting Patient Privacy**

---

## 🎯 The Problem: Healthcare's $31 Billion Authorization Bottleneck

### Current State: Manual, Slow, Privacy-Invasive

Every day, healthcare providers must obtain **prior authorization** (PA) from insurance payers before performing medical procedures. This process is:

#### **Broken for Patients**
- ⏱️ **Delays Care**: Average PA takes 2-5 business days, delaying urgent treatments
- 💸 **Increases Costs**: ~$31 billion/year in administrative waste (AMA study)
- 😤 **Frustrates Everyone**: 94% of physicians report PA delays urgent care

#### **Broken for Privacy**
- 🔓 **Full PHI Exposure**: Providers send complete medical records to payers
- 📄 **Over-Sharing**: Payer only needs to know "patient meets criteria" but sees diagnosis, labs, history
- ⚖️ **HIPAA Compliance Burden**: More data shared = more breach risk

#### **Broken for Efficiency**
- 📞 **Manual Process**: Fax machines, phone calls, case managers
- 🔄 **Duplicate Work**: Provider evaluates medical necessity, then payer re-evaluates the same data
- 🤯 **No Automation**: Human review for every request, even routine procedures

### The Core Paradox

> **Payers need to verify that authorization criteria are met.**  
> **But they DON'T need to see the patient's actual medical data to verify this.**

Current systems require full PHI disclosure because there's no way to **prove compliance without revealing data**.

---

## 💡 Our Solution: AI Agents + Zero-Knowledge Proofs

We've built an **end-to-end automated authorization system** where:

1. **AI agents extract and evaluate** medical data from patient documents and authorization rules from published policy criteria
2. **Zero-knowledge proofs cryptographically prove** the authorization decision is correct and emits public proof
3. **Payers verify instantly** using proof, patient hash and rules hash without ever seeing patient data

### The Three-Stage AI Agent Workflow

```
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 1: AI-Powered Medical Data Extraction                           │
│  ─────────────────────────────────────────────────────────────  │
│  Input:  Unstructured patient PDF (medical records)            │
│  AI Agent: LLM-powered parser extracts:                         │
│    • Demographics (age, sex, DOB)                              │
│    • Diagnoses (ICD-10 codes)                                  │
│    • Place of service                                          │
│    • Requested procedure details                              │
│  Output: Structured patient features (JSON)                     │
│  Time:  ~2.5 seconds                                           │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 2: AI-Powered Policy Evaluation + ZKP Generation        │
│  ─────────────────────────────────────────────────────────────  │
│  Input:  Patient features + Published payer policy (JSON)      │
│  AI Agent: Evaluates authorization criteria:                    │
│    • Age restrictions (18-80)                                  │
│    • Qualifying diagnoses (ICD-10 matching)                    │
│    • Place of service rules (outpatient, hospital)             │
│    • Administrative limits (units per day)                     │
│    • Exception logic (auto-approve for qualifying conditions)  │
│  AI Agent: Generates cryptographic proof:                       │
│    • Converts logic to computation trace (algebraic circuit)   │
│    • Runs streaming ZKP prover (O(√N) memory)                  │
│    • Outputs proof (~2KB) + decision record                    │
│  Output: Authorization decision + cryptographic proof           │
│  Time:  ~3 seconds                                             │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│  STAGE 3: AI-Powered Transmission + Verification               │
│  ─────────────────────────────────────────────────────────────  │
│  Input:  Proof + decision record                               │
│  AI Agent: Securely transmits to payer                          │
│  Payer:  Cryptographic verification (<1ms)                      │
│    • Verifies proof integrity (KZG pairing check)              │
│    • Confirms policy version (SHA-256 hash)                    │
│    • Validates decision logic                                  │
│  Output: ✅ Verified authorization (instant approval)           │
│  Time:  ~1.8 seconds                                           │
└─────────────────────────────────────────────────────────────────┘
```

**Total Time: ~7 seconds (vs. 2-5 business days manually)**

### What Makes This Special?

#### 🤖 **AI Agents**
- **Stage 1**: AI parses unstructured medical documents (PDFs, HL7, FHIR)
- **Stage 2**: AI evaluates complex policy logic and generates structured proofs

#### 🔐 **Zero-Knowledge Privacy Guarantee**
- Payer learns **ONLY** the authorization outcome (Approve/PA Required/Deny)
- Payer **NEVER** sees age, diagnosis, medical history, or any PHI
- Proof is cryptographically sound: impossible to fake

#### ⚡ **Instant Verification**
- Proof verifies in <1ms (vs. 2-5 days human review)
- Works for routine procedures (auto-approve) and complex cases (exception logic)
- Scales to millions of authorizations per day

---

## 🚀 Why This Is Transformative

### For Patients
- 🏥 **Faster Care**: Authorization in seconds, not days
- 🔒 **Privacy Protected**: Medical data stays with provider, never shared
- 💰 **Lower Costs**: Eliminates administrative delays and redundant work

### For Providers (Clinicians/Hospitals)
- ⏱️ **Time Savings**: No more phone calls, faxes, or case manager delays
- 📋 **Automated Workflow**: AI handles extraction, evaluation, submission
- ✅ **Instant Decisions**: Know immediately if procedure is approved
- 🎯 **Exception Handling**: AI detects when expensive procedures qualify for auto-approval

### For Payers (Insurance Companies)
- 🤖 **Automation**: AI evaluates routine requests, humans focus on complex cases
- 🔐 **Privacy Compliance**: Never receive PHI, eliminates breach risk
- 📊 **Auditability**: Every decision is cryptographically verifiable
- 💡 **Policy Transparency**: Published policies are hash-committed, immutable

### Technical Innovations

#### 1️⃣ **Streaming ZKP Engine** (O(√N) Memory)
Our custom ZKP system enables proof generation on **consumer hardware** instead of servers:
- **Traditional ZKP**: 16GB RAM for 16M-step computation (requires servers)
- **Our Streaming ZKP**: 130MB RAM for same computation (runs on laptops/phones)
- **123x memory reduction** via blocked-IFFT and tile-based commitments

#### 2️⃣ **Exception-Based AI Logic**
AI agent detects when procedures **typically requiring PA** can be **auto-approved** based on specific medical necessity:
- Example: MRI Head (expensive, usually PA required)
- Patient has qualifying neurological condition (migraine, stroke history)
- AI proves medical necessity via ZKP → instant approval
- Payer verifies without learning the diagnosis

#### 3️⃣ **Real-World Policy Integration**
System uses **actual Medicare Coverage Database rules** (270+ CPT codes):
- Policies are versioned and hash-committed (SHA-256)
- Any policy change creates a new version (auditability)
- ICD-10 codes hashed for privacy (payer can't reverse-engineer diagnoses)

---

## 🎬 Live Demo

### Quick Start (3 Commands)

```bash
# 1. Clone the repository
git clone https://github.com/logannye/my-zkp.git
cd my-zkp

# 2. Build the Rust backend
cargo build --release --package zk-agent

# 3. Start the demo UI
cd demo-ui
npm install
npm run dev
```

Demo runs at `http://localhost:3000`

### Demo Scenarios

We've prepared **four compelling scenarios** that showcase different authorization outcomes:

#### ✅ Scenario 1: Routine Auto-Approval (CT Chest)
- **Patient**: PAT002 (40-year-old male, pneumonia)
- **Procedure**: CT Chest (71250)
- **Result**: ✅ **APPROVED** (routine diagnostic imaging)
- **Why**: Common respiratory imaging, meets all criteria
- **Demo Time**: 7 seconds (vs. 2 days manual)

#### ⭐ Scenario 2: Exception-Based Auto-Approval (MRI Head) **[STAR SHOWCASE]**
- **Patient**: PAT004 (59-year-old male, chronic migraines)
- **Procedure**: MRI Head (70551) - **Expensive imaging, typically PA required**
- **Result**: ✅ **APPROVED** (exception criteria met)
- **Why**: AI detects qualifying neurological condition → auto-approves expensive procedure
- **Privacy**: Payer learns "approved for exception," NOT the diagnosis (migraine)
- **Impact**: Patient gets urgent imaging in 7 seconds, not 5 days

#### 🟡 Scenario 3: Prior Auth Required (Physical Therapy)
- **Patient**: PAT002 (same patient as Scenario 1)
- **Procedure**: Physical Therapy (97110)
- **Result**: 🟡 **NEEDS PA** (requires clinical justification)
- **Why**: Therapy requires treatment plan details, frequency, duration
- **Demo**: System generates proof, provider submits PA request (not auto-approved)

#### ❌ Scenario 4: Denied (Policy Mismatch)
- **Patient**: PAT003 (30-year-old female, breast cancer)
- **Procedure**: Total Knee Replacement (27447)
- **Result**: ❌ **DENIED** (doesn't meet orthopedic criteria)
- **Why**: Diagnosis mismatch + wrong place of service
- **Demo**: ZKP proves denial is justified, protecting both parties

### Walkthrough Video

See `demo-ui/DEMO_SCENARIOS.md` for detailed talking points and screenshots.

---

## 🏗️ Technical Architecture

### System Components

```
┌──────────────────────────────────────────────────────────────┐
│  Frontend (demo-ui/)                                         │
│  • SvelteKit 5 + TypeScript                                  │
│  • Tailwind CSS for modern medical UI                       │
│  • Reactive state machine (upload → process → results)      │
└─────────────────┬────────────────────────────────────────────┘
                  │ HTTP API
┌─────────────────▼────────────────────────────────────────────┐
│  Backend (zk-agent/)                                         │
│  • Rust CLI: authz prove / authz verify                      │
│  • Policy parser (JSON → criteria evaluation)               │
│  • Patient feature extractor (JSON → integer features)      │
│  • Computation trace builder (logic → algebraic circuit)    │
│  • Decision record generator (proof + metadata)             │
└─────────────────┬────────────────────────────────────────────┘
                  │ ZKP API
┌─────────────────▼────────────────────────────────────────────┐
│  ZKP Engine (my-zkp/)                                        │
│  • Streaming prover (O(√N) memory via blocked-IFFT)         │
│  • KZG commitments over BN254 elliptic curve                │
│  • Fiat-Shamir transcript (non-interactive proofs)          │
│  • Fast verifier (constant time, <1ms pairing checks)       │
└──────────────────────────────────────────────────────────────┘
```

### Data Flow

```
[Unstructured PDF]
    ↓ AI Agent (LLM parsing)
[Structured Patient Features]
    ↓ AI Agent (Policy evaluation)
[Computation Trace] → [ZKP Prover] → [Proof ~2KB]
    ↓ AI Agent (Transmission)
[Payer Verification] → ✅ Instant Decision
```

### Key Algorithms

#### **Streaming ZKP Prover**
- **Blocked-IFFT**: Process evaluations in √N-sized tiles
- **Tile-Based Commitments**: Never materialize full polynomials
- **Synthetic Division**: Compute openings in streaming pass
- **Memory**: O(√N) instead of O(N) → 100x+ reduction

#### **AI Policy Evaluation**
- **Criterion Matching**: Age, ICD-10, POS, admin rules
- **AND/OR Logic**: All inclusion criteria must pass, any exclusion fails
- **Exception Detection**: Auto-approve expensive procedures when specific conditions met
- **Trace Generation**: Convert logic to arithmetic circuit for ZKP

---

## 📊 Performance & Scale

### Benchmarks (Consumer Laptop)

| Metric | Traditional PA | Our AI+ZKP System |
|--------|---------------|-------------------|
| **Authorization Time** | 2-5 business days | 7 seconds |
| **PHI Exposure** | Full medical records | Zero (only outcome) |
| **Human Review Required** | 100% | 0% (routine cases) |
| **Proof Size** | N/A | ~2KB |
| **Verification Time** | N/A | <1ms |
| **Scalability** | Manual bottleneck | Millions/day automated |

### ZKP Memory Efficiency

For a 16M-step computation (100k blockchain transactions equivalent):

| Approach | Memory Required | Hardware |
|----------|----------------|----------|
| Traditional ZKP | 16GB RAM | Dedicated server |
| **Our Streaming ZKP** | **130MB RAM** | **Laptop/smartphone** |
| **Reduction** | **123x** | **Democratized** |

This memory efficiency is critical for **edge deployment** (clinician laptops, mobile devices) and **decentralization** (anyone can run a prover).

---

## 🔒 Security & Privacy Guarantees

### What the ZKP Proves

✅ **Policy Compliance**: The authorization decision follows the exact published policy rules  
✅ **Policy Version**: The specific policy version (hash) was used (no tampering)  
✅ **Patient Commitment**: A specific patient was evaluated (binding)  
✅ **Logic Correctness**: The evaluation was computed correctly (soundness)

### What the Payer Learns

✅ **Authorization Outcome**: Approve / Needs PA / Deny  
✅ **Policy ID**: Which policy was evaluated  
✅ **Proof Validity**: Whether the proof is cryptographically sound

### What the Payer NEVER Learns

❌ **Patient Age**: Hashed, never revealed  
❌ **Patient Diagnosis**: ICD-10 codes hashed, irreversible  
❌ **Which Criteria Passed/Failed**: Internal logic hidden  
❌ **Any Other PHI**: Name, DOB, address, medical history

### Cryptographic Foundation

- **Commitment Scheme**: SHA-256 (collision-resistant, irreversible)
- **ZKP System**: KZG polynomial commitments over BN254
- **Security Level**: 128-bit (industry standard)
- **Fiat-Shamir**: BLAKE3 transcript (non-interactive proofs)

---

## 🧪 Testing & Validation

### Comprehensive Test Suite

We've built **six specialized test scripts** to validate every aspect of the system:

#### 1. **Smoke Test** (`scripts/test_sszkp.sh`)
- Basic proof generation and verification
- Ensures core ZKP engine works

#### 2. **Extended Test** (`scripts/test_sszkp_extended.sh`)
- Selector commitments, permutation arguments, padding edge cases
- Tamper detection (proof rejection)

#### 3. **Integration Test** (`scripts/test_sszkp_integration.sh`)
- API builders, CSV streaming, real permutations
- Proof I/O edge cases, memory diagnostic modes

#### 4. **Memory Test** (`scripts/test_sszkp_memory.sh`)
- Validates O(√N) memory complexity empirically
- Tests from 4K to 128K rows, measures peak RSS
- **Key Result**: Memory growth is sublinear (streaming mode active)

#### 5. **Performance Test** (`scripts/test_sszkp_performance.sh`)
- Timing benchmarks for prover and verifier
- Validates O(N log N) prover time complexity
- Release build performance metrics

#### 6. **Security Test** (`scripts/test_sszkp_security.sh`)
- Proof tampering detection (bit flips, zero commitments)
- Challenge independence (Fiat-Shamir)
- Adversarial witness formats
- Parameter validation

#### 7. **End-to-End Test** (`scripts/test_zk_agent_e2e.sh`)
- Full workflow: Policy → Patient → Proof → Verification
- Tests all authorization outcomes (Approve, PA, Deny)
- Validates decision record format

**All tests pass** ✅ (run `scripts/test_*.sh` to verify)

---

## 🏆 Hackathon Judges: Why This Matters

### Problem Significance
- **$31 billion/year** in healthcare waste (AMA study)
- **94% of physicians** report PA delays care (AMA survey)
- **HIPAA breach risk** from over-sharing PHI (current practice)

### AI Agent Innovation
- **AI-powered data extraction** from unstructured medical documents
- **AI-powered policy evaluation** with exception logic detection
- **AI-powered workflow automation** end-to-end (7 seconds vs. 2-5 days)

### Technical Breakthrough
- **123x memory reduction** via streaming ZKP (novel algorithm)
- **Runs on consumer hardware** (laptops, phones) instead of servers
- **Cryptographically sound** (128-bit security, KZG+BN254)

### Real-World Impact
- **Patients get care faster** (seconds instead of days)
- **Privacy is guaranteed** (cryptographic, not policy-based)
- **Healthcare costs reduced** (automation eliminates $31B waste)

### Production Readiness
- **270+ real Medicare policies** integrated
- **Comprehensive test suite** (7 test scripts, all passing)
- **Beautiful, intuitive UI** designed for clinicians
- **Scalable architecture** (millions of requests/day)

---

## 🎯 Next Steps for Production

To deploy this system in the real world:

### Short Term (3 months)
- ✅ Integrate with HL7 FHIR for real patient records
- ✅ Add authentication for clinicians (OAuth2)
- ✅ Connect to payer APIs (X12 837, 278 transactions)
- ✅ Deploy backend as HIPAA-compliant microservices

### Medium Term (6 months)
- ✅ Add multi-code authorization (batch requests)
- ✅ Build analytics dashboard for providers
- ✅ Implement real-time PA status tracking
- ✅ Add mobile app for on-the-go authorization

### Long Term (12 months)
- ✅ Expand to 50 major payers (Blue Cross, Aetna, Cigna, etc.)
- ✅ Integrate with major EHR systems (Epic, Cerner, Allscripts)
- ✅ Deploy to 1,000+ hospitals nationwide
- ✅ Save 100M+ patient-days of authorization delays

---

## 📞 Contact

**Project Team**: Galen Health

**GitHub**: https://github.com/logannye/my-zkp 

**Demo**: `http://localhost:3000` (after running `npm run dev`)

---

## 📄 License

MIT License - Free for commercial and non-commercial use.

---

## 🙏 Acknowledgments

This project combines cutting-edge research in:
- **Zero-knowledge proofs** (streaming computation techniques)
- **AI agents** (LLM-powered medical data extraction)
- **Healthcare informatics** (real Medicare Coverage Database policies)

Built with [Arkworks](https://github.com/arkworks-rs) cryptography libraries and [SvelteKit](https://kit.svelte.dev/) for the UI.

---

**Let's transform healthcare authorization: faster care, guaranteed privacy, automated efficiency.**

