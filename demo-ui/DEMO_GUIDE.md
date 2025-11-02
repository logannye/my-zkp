# Medical Authorization Portal - Demo Guide

## Quick Start

### Option 1: Automated Setup (Recommended)

```bash
cd demo-ui
./start-demo.sh
```

This script will:
- Check prerequisites (Node.js, Rust, Cargo)
- Install npm dependencies
- Build the ZK backend
- Start the development server

### Option 2: Manual Setup

```bash
# 1. Install frontend dependencies
cd demo-ui
npm install

# 2. Build the ZK backend
cd ..
cargo build --release --package zk-agent

# 3. Start the dev server
cd demo-ui
npm run dev
```

The demo will be available at **http://localhost:3000**

## Demo Walkthrough

### Scenario: CT Chest Authorization

This walkthrough demonstrates the most common use case: requesting authorization for a CT scan.

#### Step 1: Upload Patient Record

1. Open http://localhost:3000
2. Drag and drop `static/demo/sample-patient-john-doe.txt` into the upload zone
   - Or click the zone to browse and select a file
3. You'll see a green checkmark confirming the upload
4. Patient info is extracted: "John Doe, DOB: 1970-05-15"

#### Step 2: Select Procedure Code

1. Click through to the code selection screen
2. Search or scroll to find **71250 - CT Chest without contrast**
3. Notice the **Auto-Approve** badge (no PA required)
4. Click to select this code

#### Step 3: Review Authorization Request

Review the summary:

- **Patient**: John Doe (displayed locally only)
- **Procedure**: CT Chest (71250)
- **Policy**: UHC-COMM-CT-CHEST-001
- **Privacy Guarantee**: Patient data encrypted locally

Click **Submit Authorization Request**

#### Step 4: Watch the ZKP Generation

You'll see a progress animation with status updates:

1. 📄 Parsing patient record... (20%)
2. 🔍 Extracting medical features... (40%)
3. ⚖️ Evaluating authorization criteria... (60%)
4. 🔐 Generating zero-knowledge proof... (80%)
5. ✅ Creating decision record... (100%)

**What's happening behind the scenes:**

- The backend calls the Rust `authz` CLI
- Patient data (`p001-approve.json`) is loaded
- Policy (`UHC-COMM-CT-CHEST-001.json`) is evaluated
- A zero-knowledge proof is generated (streaming mode)
- A verifiable decision record is created

This takes approximately 3-5 seconds.

#### Step 5: View Results

You'll see:

- **Large green badge**: ✅ APPROVED
- **Message**: "Authorization approved. Proceed with scheduling."
- **Privacy guarantees**: No patient data shared, 2KB proof
- **Proof statistics**: Policy ID, proof size, code

Click **View Decision Record Details** to see:
- Policy hash (SHA-256)
- Patient commitment (SHA-256 of features + salt)
- Claimed result
- Proof (base64-encoded, truncated for display)

#### Step 6: Download Decision Record

1. Click **Download Record** to save the full JSON
2. Or click **Copy Proof** to copy the base64 proof to clipboard
3. The JSON file contains everything a payer needs to verify:
   ```json
   {
     "policyId": "UHC-COMM-CT-CHEST-001",
     "policyHash": "0x...",
     "patientCommitment": "0x...",
     "claimedResult": "APPROVE",
     "proof": "base64...",
     "code": "71250",
     "lob": "commercial"
   }
   ```

4. Click **Start New Authorization** to reset and try another scenario

## Additional Scenarios

### Scenario 2: Breast Biopsy (PA Required)

1. Upload any patient file
2. Select **19081 - Breast Biopsy**
3. Notice the **PA Required** badge
4. Submit the request
5. Result: **⚠️ PRIOR AUTHORIZATION REQUIRED**
6. Next steps include submitting to payer portal

### Scenario 3: MRI Head (Auto-Approve)

1. Upload any patient file
2. Select **70551 - MRI Head without contrast**
3. Submit the request
4. Result: **✅ APPROVED**

### Scenario 4: Physical Therapy

1. Upload any patient file
2. Select **97110 - Physical Therapy**
3. Submit the request
4. Result depends on patient data (units requested vs. policy limits)

## Understanding the Results

### ✅ APPROVED

**Meaning**: The claim meets all policy criteria and is automatically approved.

**Next Steps**:
1. Schedule procedure with patient
2. Submit claim with attached proof
3. No further authorization needed

**For Payer**: Verify the proof instantly without accessing patient data.

### ⚠️ NEEDS PA

**Meaning**: The procedure requires prior authorization review by the payer.

**Next Steps**:
1. Download decision record (includes proof)
2. Submit PA request via payer portal
3. Attach decision record to PA submission
4. Payer verifies proof (patient data stays private)

**For Payer**: The proof demonstrates policy compliance but requires human review for PA.

### ❌ DENIED

**Meaning**: The claim does not meet policy criteria and is denied.

**Next Steps**:
1. Review policy requirements
2. Check patient eligibility criteria
3. Consider alternative procedures
4. Resubmit if criteria are met

**For Payer**: The proof demonstrates non-compliance with policy rules.

## Privacy Guarantees

### What Data Is Shared?

**Shared with Payer**:
- Policy hash (32 bytes)
- Patient commitment (32 bytes)
- Authorization result (1 byte)
- Zero-knowledge proof (~2KB)

**Total**: ~2.1 KB

**NOT Shared**:
- Patient name
- Date of birth
- Medical diagnoses (ICD-10 codes)
- Test results
- Doctor notes
- Any other PHI/PII

### How It Works

1. **Commitment**: Patient data is hashed with a random salt
   - `commitment = SHA256(features || salt)`
   - Only the hash is shared

2. **Zero-Knowledge Proof**: Proves the authorization logic was correctly evaluated
   - Proves: "This commitment corresponds to a patient who meets policy criteria"
   - Does NOT reveal: What those criteria are or the patient's actual data

3. **Verification**: Payer verifies the proof in <1ms
   - No access to patient data needed
   - Cryptographically guaranteed correctness

## Troubleshooting

### "Failed to generate authorization proof"

**Cause**: Backend Rust binary not built or not found.

**Solution**:
```bash
cd /Users/logannye/my-zkp
cargo build --release --package zk-agent
```

### "Port 3000 is already in use"

**Solution**: Change the port in `vite.config.ts`:
```typescript
server: {
  port: 3001,
  ...
}
```

### Proof generation takes too long (>10 seconds)

**Cause**: Running in debug mode instead of release mode.

**Solution**: Ensure you built with `--release` flag:
```bash
cargo build --release --package zk-agent
```

### TypeScript errors in the browser console

**Cause**: Usually safe to ignore during development.

**Solution**: Run type checking:
```bash
npm run check
```

## Technical Details

### Backend Integration

The demo calls the Rust CLI via Node.js `child_process`:

```typescript
const command = `cargo run --release --package zk-agent --bin authz -- prove \
  --policy policies/${policyFile} \
  --patient patients/${patientFile} \
  --code ${code} \
  --lob commercial \
  --out ${outputFile}`;
```

### Proof Verification

To verify a downloaded proof:

```bash
cd /Users/logannye/my-zkp
cargo run --release --package zk-agent --bin authz -- verify out/authorization-71250-1234567890.json
```

### Streaming Mode

The backend uses streaming ZKP generation (`SSZKP_BLOCKED_IFFT=1`) for memory efficiency:

- **Traditional ZKP**: O(N) memory, ~16GB for 16M rows
- **Streaming ZKP**: O(√N) memory, ~130MB for 16M rows

This is automatically enabled in the demo.

## Architecture

```
┌─────────────┐
│   Browser   │
│  (Svelte)   │
└──────┬──────┘
       │ HTTP POST /api/authorize
       ↓
┌─────────────┐
│  SvelteKit  │
│    API      │
└──────┬──────┘
       │ exec()
       ↓
┌─────────────┐
│ Rust authz  │
│     CLI     │
└──────┬──────┘
       │
       ↓
┌─────────────┐
│  ZKP Engine │
│  (my-zkp)   │
└─────────────┘
```

## Customization

### Adding New Codes

1. Create policy JSON in `../policies/`
2. Create patient JSON in `../patients/`
3. Update `src/lib/utils/mock-data.ts`:
   ```typescript
   {
     code: 'NEW_CODE',
     description: 'New Procedure',
     policyId: 'YOUR-POLICY-ID',
     requiresPA: false,
     patientFile: 'your-patient.json'
   }
   ```

### Styling

- Modify `tailwind.config.js` for colors
- Edit `src/app.css` for global styles
- Update components in `src/lib/components/`

## Production Considerations

This demo is for educational/hackathon purposes. For production:

1. **Replace mock data** with real EHR/HL7 parsing
2. **Add authentication** (OAuth, SAML)
3. **Secure API endpoints** (rate limiting, CORS)
4. **Use real SRS** (not dev-srs)
5. **Add audit logging** for HIPAA compliance
6. **Database integration** for decision records
7. **Payer API integration** for automatic submission
8. **Error handling and retry logic**
9. **Performance monitoring**
10. **Backup and disaster recovery**

## Support

For issues:
1. Check this guide
2. Review `README.md` in this directory
3. Consult the main project `README.md`
4. Run test scripts in `../scripts/`

---

**Enjoy the demo! 🚀**

