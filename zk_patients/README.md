# Patient Data Extraction System

## Overview

This folder contains an LLM-powered agent that extracts structured patient data from PDF medical records. It uses OpenAI's API to parse patient information and generates JSON files compatible with the ZK authorization rules system.

## What This Does

1. **Processes Patient PDFs** - Reads medical record PDFs from the `patients/` folder
2. **Extracts Patient Data** - Uses OpenAI GPT-4 to intelligently extract structured patient information
3. **Generates JSON Output** - Creates JSON files with patient data compatible with `rules.json` evaluation
4. **Normalizes Data** - Ensures ICD-10 codes, dates, and other fields are properly formatted

## Patient JSON Format

Each extracted patient JSON follows this structure (compatible with `rules.json` evaluation):

```json
{
  "patient_id": "PAT001",
  "dob": "1970-04-02",
  "sex": "F",
  "icd10_list": ["C50.912", "E11.9"],
  "pregnant": false,
  "place_of_service": 22,
  "units": 1
}
```

### Field Descriptions

- **patient_id**: Unique patient identifier
- **dob**: Date of birth (YYYY-MM-DD format)
- **sex**: Gender ("F" for female, "M" for male)
- **icd10_list**: Array of ICD-10-CM diagnosis codes (uppercase)
- **pregnant**: Boolean indicating pregnancy status (only relevant for females)
- **place_of_service**: HIPAA place of service code (integer)
  - 11 = Office
  - 22 = Outpatient hospital
  - 23 = Emergency room
  - 19 = Off campus-outpatient hospital
- **units**: Number of units requested (integer, typically 1)

## Setup

### 1. Install Dependencies

```bash
pip install -r requirements.txt
```

### 2. Configure OpenAI API Key

Create a `.env` file in this directory:

```bash
OPENAI_API_KEY=your_openai_api_key_here
```

**Important**: The `.env` file is already in `.gitignore` and will not be committed to git.

### 3. Generate Sample Patient PDFs

```bash
python generate_patient_pdfs.py
```

This creates 10 sample patient PDF files in the `patients/` folder with mock medical record data.

## Usage

### Extract Patient Data from PDFs

```bash
python extract_patient_data.py
```

This will:
1. Read all PDF files from the `patients/` folder
2. Extract text from each PDF
3. Use OpenAI API to extract structured patient data
4. Generate JSON files in the `output/` folder (one per patient PDF)

### Process a Single PDF

You can modify the script or call the function directly:

```python
from extract_patient_data import process_patient_pdf

patient_data = process_patient_pdf("patients/PAT001.pdf", "output")
print(patient_data)
```

## File Structure

```
zk_patients/
├── README.md                  # This file
├── requirements.txt           # Python dependencies
├── .env                       # OpenAI API key (not in git)
├── .gitignore                # Git ignore rules
├── generate_patient_pdfs.py   # Script to generate sample PDFs
├── extract_patient_data.py   # Main LLM extraction agent
├── patients/                  # Input PDF files (patient medical records)
│   ├── PAT001.pdf
│   ├── PAT002.pdf
│   └── ...
└── output/                    # Generated JSON files (one per patient)
    ├── PAT001.json
    ├── PAT002.json
    └── ...
```

## Integration with ZK Rules

The extracted patient JSON can be used with the `rules.json` files from `zk_authz_rules/`:

1. **Load Patient Data**: Read the JSON file
2. **Calculate Age**: Compute age from DOB for age-based rules
3. **Normalize ICD Codes**: Convert ICD-10 codes to match rules format (uppercase, dotless)
4. **Evaluate Rules**: Check patient data against `rules.json` inclusion/exclusion criteria
5. **Generate Proof**: Use patient features in ZK proof generation

Example integration:

```python
import json
from datetime import datetime

# Load patient data
with open("output/PAT001.json") as f:
    patient = json.load(f)

# Calculate age
dob = datetime.strptime(patient["dob"], "%Y-%m-%d")
age_years = (datetime.now() - dob).days // 365

# Normalize ICD codes (remove dots, uppercase)
icd_codes = [code.replace(".", "").upper() for code in patient["icd10_list"]]

# Use with rules.json for ZK evaluation
features = {
    "age_years": age_years,
    "sex": 1 if patient["sex"] == "F" else 0,
    "primary_icd10_hash": hash(icd_codes[0]) if icd_codes else 0,
    "pregnant": 1 if patient["pregnant"] else 0,
    "pos": patient["place_of_service"],
    "units": patient["units"]
}
```

## Notes

- **API Costs**: Using OpenAI API incurs costs. GPT-4o-mini is used by default for cost efficiency.
- **PDF Quality**: Text extraction quality depends on PDF format. Scanned images won't work (need OCR first).
- **Data Privacy**: Patient PDFs should contain mock data for testing only.
- **Error Handling**: The script will skip PDFs that can't be processed and continue with others.
- **LLM Accuracy**: Extraction accuracy depends on PDF format and clarity. Review generated JSON for accuracy.


## License

Part of the ZKP project for medical authorization rules.

