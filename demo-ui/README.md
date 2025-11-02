# Medical Authorization Portal - Demo UI

A clinician-facing web application that demonstrates privacy-preserving prior authorization using zero-knowledge proofs.

## Overview

This demo showcases an end-to-end workflow where clinicians can:

1. Upload patient medical records
2. Select procedure codes (CPT/HCPCS)
3. Submit authorization requests
4. Receive instant decisions with zero-knowledge proofs
5. Download verifiable decision records

**Key Features:**

- 🔒 **Privacy-First**: Patient data never leaves the local system
- ⚡ **Instant Verification**: Proofs verify in <1ms
- 📦 **Compact Proofs**: ~2KB vs. full medical records
- ✅ **HIPAA-Compliant**: By design, not by policy
- 🎨 **Modern UI**: Built with SvelteKit + Tailwind CSS

## Technology Stack

- **Frontend**: SvelteKit 5 + TypeScript
- **Styling**: Tailwind CSS
- **Icons**: Lucide Svelte
- **Backend**: Rust `authz` CLI (ZK proof generation)
- **ZKP Engine**: Custom streaming ZKP from `zk-agent` crate

## Prerequisites

Before running the demo, ensure you have:

1. **Node.js** (v18 or later)
2. **Rust** (for the `zk-agent` backend)
3. **Cargo** (Rust package manager)

The demo requires the parent `my-zkp` project to be built with the `zk-agent` crate.

## Installation

```bash
cd demo-ui

# Install dependencies
npm install

# Build the ZK backend (from project root)
cd ..
cargo build --release --package zk-agent

# Return to demo-ui
cd demo-ui
```

## Running the Demo

### Development Mode

```bash
npm run dev
```

This starts the development server at `http://localhost:3000`.

### Production Build

```bash
npm run build
npm run preview
```

## Demo Workflow

### Step 1: Upload Patient Record

- Drag and drop a patient record file (PDF, JSON, TXT)
- For demo purposes, patient info is extracted from the filename
- In production, this would parse actual medical records

### Step 2: Select Procedure Code

Choose from available CPT/HCPCS codes:

- **71250**: CT Chest (Auto-Approve)
- **19081**: Breast Biopsy (PA Required)
- **70551**: MRI Head (Auto-Approve)
- **97110**: Physical Therapy (Auto-Approve)
- **J3590**: Specialty Drug (PA Required)
- **G0472**: Screening Colonoscopy (Auto-Approve)

### Step 3: Review Submission

- Patient information (displayed locally only)
- Selected procedure details
- Applicable policy
- Privacy guarantee explanation

### Step 4: Processing

The system:
1. Parses patient data
2. Extracts medical features
3. Evaluates authorization criteria
4. **Generates zero-knowledge proof** (~3-5 seconds)
5. Creates verifiable decision record

### Step 5: View Results

Three possible outcomes:

- ✅ **APPROVED**: Proceed with scheduling
- ⚠️ **NEEDS PA**: Submit PA request with proof
- ❌ **DENIED**: Review policy requirements

### Step 6: Next Steps

- Download decision record (JSON with embedded proof)
- Copy proof to clipboard
- Start new authorization

## Architecture

### Component Structure

```
src/
├── routes/
│   ├── +page.svelte              # Main orchestrator
│   ├── +page.ts                  # Page config
│   └── api/authorize/+server.ts  # Backend API (calls Rust CLI)
├── lib/
│   ├── components/
│   │   ├── FileUpload.svelte
│   │   ├── CodeSelector.svelte
│   │   ├── ReviewSummary.svelte
│   │   ├── ProcessingAnimation.svelte
│   │   ├── ResultsDisplay.svelte
│   │   ├── NextSteps.svelte
│   │   └── ui/                   # Reusable UI components
│   ├── stores/
│   │   └── workflow.svelte.ts    # State management
│   ├── types/index.ts            # TypeScript interfaces
│   └── utils/
│       ├── mock-data.ts          # Demo patient/code mappings
│       └── cn.ts                 # Tailwind utility
└── app.css                       # Global styles
```

### State Management

The app uses a reactive state machine (`workflow.svelte.ts`):

```
upload → select → review → processing → results
   ↑                                       ↓
   └────────────────reset──────────────────┘
```

### API Endpoint

`/api/authorize` bridges the SvelteKit frontend to the Rust `authz` CLI:

1. Receives patient info + procedure code
2. Maps to appropriate policy/patient JSON files
3. Executes `cargo run --bin authz -- prove ...`
4. Returns decision record with ZKP proof

## Mock Data

For demo purposes, the app uses pre-defined patient and policy files from the parent project:

- **Policies**: `../policies/*.json`
- **Patients**: `../patients/*.json`

The mapping is defined in `src/lib/utils/mock-data.ts`.

## Customization

### Adding New Procedure Codes

Edit `src/lib/utils/mock-data.ts`:

```typescript
export const availableCodes: Code[] = [
  {
    code: 'YOUR_CODE',
    description: 'Your Procedure Description',
    policyId: 'YOUR-POLICY-ID',
    requiresPA: false,
    patientFile: 'your-patient-file.json'
  },
  // ... existing codes
];
```

### Styling

The app uses Tailwind CSS with a custom color palette:

- **Primary**: Blue (#3B82F6) - Trust, medical
- **Success**: Green (#10B981) - Approved
- **Warning**: Yellow (#F59E0B) - Needs PA
- **Danger**: Red (#EF4444) - Denied
- **Privacy**: Purple (#8B5CF6) - ZK/Crypto

Modify `tailwind.config.js` to customize colors.

## Troubleshooting

### Proof generation fails

**Error**: `Failed to generate authorization proof`

**Solutions**:
1. Ensure `zk-agent` is built: `cargo build --release --package zk-agent`
2. Check that policy/patient JSON files exist in `../policies/` and `../patients/`
3. Verify `SSZKP_BLOCKED_IFFT=1` is set (for streaming mode)

### Port already in use

**Error**: `Port 3000 is already in use`

**Solution**: Change the port in `vite.config.ts`:

```typescript
server: {
  port: 3001, // Use a different port
  ...
}
```

### TypeScript errors

**Solution**: Run type checking:

```bash
npm run check
```

## Production Deployment

For production use, you'll need to:

1. **Replace mock data** with real patient record parsing
2. **Add authentication** for clinicians
3. **Secure API endpoints** with rate limiting
4. **Store decision records** in a database
5. **Integrate with payer APIs** for automatic submission
6. **Add audit logging** for compliance
7. **Use real SRS** (not dev-srs) for production proofs

## Future Enhancements

- [ ] Real PDF parsing for patient records
- [ ] Multi-code authorization (batch requests)
- [ ] History view of past authorizations
- [ ] Direct payer API integration
- [ ] Analytics dashboard
- [ ] Mobile-responsive optimizations
- [ ] HL7 FHIR integration

## License

This demo is part of the `my-zkp` project. See the parent project for license information.

## Support

For issues or questions:

1. Check the main project README
2. Review test scripts in `../scripts/`
3. Consult the `zk-agent` documentation

---

**Demo Application - For Educational and Hackathon Purposes**

