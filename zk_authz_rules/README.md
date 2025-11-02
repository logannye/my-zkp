# Medicare Coverage Rules Generator

## Overview

This folder contains a system for generating and managing medical authorization rules from the Medicare Coverage Database. It extracts ICD-10-CM codes that support or do not support medical necessity for CPT codes, creating deterministic, versioned policy rules that are canonicalized and hashed for use in zero-knowledge proof systems.

## What This Does

1. **Fetches Medicare Coverage Database articles** - Retrieves HTML content from CMS Medicare Coverage Database articles
2. **Extracts CPT and ICD-10-CM codes** - Parses articles to identify CPT codes and their associated ICD-10-CM diagnosis codes
3. **Identifies medical necessity criteria** - Distinguishes between ICD-10-CM codes that support medical necessity vs. those that don't
4. **Generates canonical rules.json files** - Creates one `rules.json` file per CPT code in `rules/{cpt_code}/rules.json` format
5. **Computes policy hashes** - Generates deterministic SHA256 hashes for cryptographic verification

## File Structure

```
zk_authz_rules/
├── README.md              # This file
├── generate_rules.py       # Main script that fetches Medicare articles and generates rules
├── policy_model.py        # Policy dataclass with canonicalization and hashing
├── storage.py            # SQLite database interface for policy storage
├── rules_service.py      # API for querying rules by CPT code
└── rules/                # Generated rules.json files (one per CPT code)
    ├── 97110/
    │   └── rules.json
    ├── 99213/
    │   └── rules.json
    └── ...
```

## Rules.json Format

Each `rules.json` file follows this structure:

```json
{
  "policy_id": "MEDICARE-{CPT_CODE}",
  "version": "2025-01-01",
  "payer": "medicare",
  "lob": "original",
  "codes": ["97110"],
  "requires_pa": true,
  "inclusion": {
    "icd10_in": ["A5514", "E08", "E0862"]
  },
  "exclusion": {
    "icd10_in": []
  },
  "admin": {
    "max_units_per_day": 1
  },
  "metadata": {
    "source_url": "https://www.cms.gov/medicare-coverage-database/view/article.aspx?articleid=57034&ver=19",
    "articleid": 57034,
    "version": 19,
    "effective_date": "2025-01-01",
    "notes": "Extracted from Medicare Coverage Database..."
  },
  "policy_hash": "0x7b69ac08054547d9ec21d373d316ac8053dafc44504880e0aef47848ef3b869f"
}
```

### Field Descriptions

- **policy_id**: Unique identifier (format: `MEDICARE-{CPT_CODE}`)
- **version**: Policy version date
- **payer**: Insurance payer (`"medicare"`)
- **lob**: Line of business (`"original"`)
- **codes**: Array of CPT/HCPCS codes covered by this rule
- **requires_pa**: Boolean indicating if prior authorization is required
- **inclusion**: Criteria that must be met
  - **icd10_in**: ICD-10-CM codes that support medical necessity (normalized, uppercase, dotless)
- **exclusion**: Criteria that exclude coverage
  - **icd10_in**: ICD-10-CM codes that do NOT support medical necessity
- **admin**: Administrative rules
  - **max_units_per_day**: Maximum units allowed per day
  - **pos_allowed**: Optional array of place of service codes
- **metadata**: Source information
  - **source_url**: URL to the Medicare Coverage Database article
  - **articleid**: Article ID from Medicare database
  - **version**: Article version number
  - **effective_date**: When the policy takes effect
  - **notes**: Human-readable notes
- **policy_hash**: SHA256 hash of canonical JSON (format: `0x{hex}`)

## Quick Start

### 1. Install Dependencies

```bash
pip install requests beautifulsoup4
```

### 2. Generate Rules

```bash
python generate_rules.py
```

This will:
- Fetch Medicare Coverage Database articles for each CPT code
- Extract ICD-10-CM codes and medical necessity information
- Generate `rules/{cpt_code}/rules.json` files (currently generates 69 CPT codes)

### 3. Query Rules Programmatically

```python
from rules_service import find_rules
import json

# Get rules for a specific CPT code
rules = find_rules(code="27447", payer="medicare", lob="original")
if rules:
    print(json.dumps(rules, indent=2))
```

## How It Works

### 1. Article Fetching

The script searches the Medicare Coverage Database using CPT codes. For each CPT code, it:
- Searches for articles containing that CPT code
- Falls back to known article IDs if direct search fails
- Fetches HTML content from CMS articles

### 2. ICD Code Extraction

Uses regex patterns to extract ICD-10-CM codes from article text:
- Normalizes codes (removes dots, uppercases): `M17.11` → `M1711`
- Identifies codes that support vs. don't support medical necessity
- Uses pattern matching to find phrases like "supports medical necessity" or "does not support medical necessity"

### 3. Canonicalization & Hashing

- **Normalization**: ICD-10 codes are normalized (uppercase, dotless)
- **Canonicalization**: JSON keys are sorted; list fields (codes, icd10_in) are sorted
- **Hashing**: SHA256 hash of canonical JSON → `policy_hash`

This ensures deterministic hashes that can be verified cryptographically.

## Configuration

Edit `CPT_CODES_TO_PROCESS` in `generate_rules.py` to add or remove CPT codes:

```python
CPT_CODES_TO_PROCESS = [
    "97110",  # Therapeutic procedure
    "99213",  # Office visit
    "27447",  # Knee arthroplasty
    # ... add more CPT codes here
]
```

## Database Storage

Rules are also stored in SQLite (`rules.db`) for programmatic access via `rules_service.py`. The database schema includes:
- Policy ID, version, payer, LOB
- CPT codes (JSON array)
- Inclusion/exclusion criteria (JSON)
- Administrative rules (JSON)
- Policy hash

## Integration with ZK Proofs

The canonical JSON format and policy hashes enable:
- **Public inputs**: `policy_id`, `policy_hash`, `payer`, `code`, `patient_commitment`, `claimed_result`
- **Private inputs**: `policy_json`, `patient_features`, `salt`
- **Verification**: Prover asserts `hash(policy_json) == policy_hash` and evaluates rules deterministically

## Files

- **generate_rules.py**: Main script that processes Medicare articles and generates rules
- **policy_model.py**: Policy dataclass with validation, canonicalization, and hashing
- **storage.py**: SQLite database interface for storing and querying policies
- **rules_service.py**: Simple API for fetching rules by CPT code

## Notes

- ICD-10-CM codes are normalized (uppercase, dotless) for deterministic matching
- Policy hashes are computed from canonical JSON to ensure stability
- Rules are generated from real Medicare Coverage Database articles
- Each CPT code gets its own `rules/{cpt_code}/rules.json` file

## License

Part of the ZKP project for medical authorization rules.
