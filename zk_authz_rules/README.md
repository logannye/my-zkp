# Medicare Coverage Rules Generator

## Overview

This folder contains a system for generating medical authorization rules from the Medicare Coverage Database. It processes verified Medicare article URLs, extracts CPT codes and ICD-10-CM codes that support or do not support medical necessity, and generates deterministic rule JSON files for use in zero-knowledge proof systems.

## What This Does

1. **Processes Medicare Coverage Database articles** - Fetches HTML content from verified CMS Medicare Coverage Database articles using article IDs (e.g., `articleId=57765`)
2. **Handles license agreements** - Automatically accepts AMA license agreements if required
3. **Extracts CPT and ICD-10-CM codes** - Parses articles to identify CPT/HCPCS procedure codes and their associated ICD-10-CM diagnosis codes
4. **Identifies medical necessity criteria** - Distinguishes between ICD-10-CM codes that support medical necessity vs. those that don't
5. **Generates rules.json files** - Creates one `rules.json` file per CPT code found, stored directly in the `rules` folder as `{CPT_CODE}.json`
6. **Maps ICD-10 codes to integers** - Converts ICD-10-CM codes to integers for ZK processing compatibility
7. **Uses article URLs as policy IDs** - Each rule's `policy_id` is the actual article URL used to generate it

## File Structure

```
zk_authz_rules/
├── README.md                   # This file
├── generate_rules.py           # Main script that fetches and parses Medicare articles
├── policy_model.py             # Policy dataclass with canonicalization and hashing
├── storage.py                  # SQLite database interface for policy storage
├── rules_service.py            # API for querying rules by CPT code
└── rules/                      # Generated rules.json files (one per CPT code)
    ├── 21244.json
    ├── 27447.json
    ├── 97110.json
    └── ...
```

## Rules.json Format

Each `rules.json` file follows this structure:

```json
{
  "policy_id": "https://www.cms.gov/medicare-coverage-database/view/article.aspx?articleId=57765",
  "version": "2025-10-01",
  "lob": "commercial",
  "codes": ["27447"],
  "requires_pa": true,
  "inclusion": [
    {
      "gte": ["age_years", 18]
    },
    {
      "lte": ["age_years", 80]
    },
    {
      "in": ["primary_icd10", [3001, 3002, 3003, 3004]]
    }
  ],
  "exclusion": [
    {
      "in": ["primary_icd10", [5001, 5002]]
    }
  ],
  "admin_rules": {
    "pos_allowed": [22, 24],
    "max_units_per_day": 1
  }
}
```

### Field Descriptions

- **`policy_id`**: The full URL of the Medicare article used to generate this rule
- **`version`**: Rule version date (currently "2025-10-01")
- **`lob`**: Line of business (currently "commercial")
- **`codes`**: Array of CPT/HCPCS procedure codes (typically one code per file)
- **`requires_pa`**: Boolean indicating if prior authorization is required
- **`inclusion`**: Array of inclusion criteria:
  - `{"gte": ["age_years", 18]}` - Minimum age requirement
  - `{"lte": ["age_years", 80]}` - Maximum age requirement
  - `{"in": ["primary_icd10", [3001, 3002, ...]]}` - ICD-10 codes (as integers) that support medical necessity
- **`exclusion`**: Array of exclusion criteria:
  - `{"in": ["primary_icd10", [5001, 5002, ...]]}` - ICD-10 codes (as integers) that do NOT support medical necessity
- **`admin_rules`**: Administrative rules:
  - `pos_allowed`: Array of HIPAA place of service codes where the procedure is allowed
  - `max_units_per_day`: Maximum number of units allowed per day

## Quick Start

### 1. Install Dependencies

```bash
pip install requests beautifulsoup4
```

### 2. Run the Generator

```bash
python generate_rules.py
```

This will:
- Remove any existing `rules/` folder
- Process articles from the `ARTICLE_IDS` list
- Stop after successfully parsing 50 articles (skipping failures without retrying)
- Generate one `rules.json` file per CPT code found
- Save all files directly in the `rules/` folder

### 3. View Generated Rules

```bash
ls rules/
cat rules/27447.json
```

## How It Works

1. **Article Processing**: The script iterates through a list of Medicare article IDs
2. **URL Construction**: Builds URLs like `https://www.cms.gov/medicare-coverage-database/view/article.aspx?articleId={id}`
3. **License Handling**: Automatically detects and accepts AMA license agreements if required
4. **HTML Parsing**: Uses BeautifulSoup to extract text content from article HTML
5. **Code Extraction**: Uses regex patterns to identify:
   - CPT/HCPCS codes (5-digit numeric codes)
   - ICD-10-CM codes (alphanumeric codes like "M25.561")
6. **Medical Necessity Classification**: Analyzes text context to determine which ICD-10 codes support vs. don't support medical necessity
7. **Rule Generation**: Creates JSON rules with:
   - ICD-10 codes mapped to integers via SHA256 hash
   - Age constraints extracted from text (or defaults)
   - Place of service codes if found in article
   - Prior authorization requirements
8. **File Writing**: Writes one JSON file per CPT code to `rules/{CPT_CODE}.json`

## Integration with ZK Proofs

The generated rules are designed to work with zero-knowledge proof systems:

- **Deterministic format**: Rules follow a consistent JSON structure
- **Integer-mapped ICD codes**: ICD-10 codes are converted to integers for efficient ZK circuit processing
- **Verifiable sources**: Each rule's `policy_id` is the actual article URL, allowing verification
- **Complete criteria**: Inclusion/exclusion rules enable comprehensive authorization logic

## Notes

- **No retries**: If an article fails to fetch or parse, the script moves on to the next one
- **License agreements**: The script automatically handles AMA license acceptance when required
- **Article verification**: Only processes articles that successfully load and contain valid content
- **CPT code deduplication**: If multiple articles contain the same CPT code, later articles overwrite earlier ones
- **Default values**: Age limits default to 18-80 years if not found in the article

## License

This project processes publicly available Medicare Coverage Database information for educational and development purposes.
