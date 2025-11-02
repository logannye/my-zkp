# Demo Files

This directory can contain sample patient record files for the demo.

For the demo to work, you can place any file here (PDF, JSON, TXT) and the system will extract mock patient information from the filename.

## Example Files

- `patient-john-doe.pdf` - Will extract name "John Doe"
- `patient-001.json` - Will use patient ID 001
- `medical-record-jane-smith.txt` - Will extract name "Jane Smith"

The actual ZKP proof generation uses the pre-defined patient JSON files from the parent project's `patients/` directory.

## Production

In production, this would be replaced with:
- Real PDF/HL7 parsing
- FHIR integration
- Secure file upload with encryption
- Automatic feature extraction

