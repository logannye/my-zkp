"""
Seed generator for UnitedHealthcare-style policies (illustrative).
Replace ICD lists & URLs with values extracted from UHC documentation.
"""

import json
import os
from typing import List
from policy_model import Policy
from storage import init_db, insert_policy

OUT_DIR = "rules_out"

SEED_POLICIES: List[Policy] = [
    # Example: Total knee arthroplasty (CPT 27447)
    Policy(
        policy_id="UHC-KNEE-ARTHROPLASTY-001",
        version="2025-10-01",
        payer="uhc",
        lob="commercial",
        codes=["27447"],
        requires_pa=True,
        inclusion={
            "age_gte": 50,
            "icd10_in": ["M17.0", "M17.11", "M17.12"],  # <- replace with real ICDs from policy
            "failed_conservative": 1
        },
        exclusion={
            "icd10_in": ["M00.9"],  # active infection (illustrative)
        },
        admin={
            "pos_allowed": [11, 22],
            "max_units_per_day": 1
        },
        metadata={
            "source_url": "https://www.uhcprovider.com/en/policies-protocols/medical-policies.html",
            "effective_date": "2025-10-01",
            "notes": "Illustrative only; replace with exact criteria from UHC policy PDF."
        }
    ),
    # Example: MRI brain (CPT 70551)
    Policy(
        policy_id="UHC-MRI-BRAIN-001",
        version="2025-10-01",
        payer="uhc",
        lob="commercial",
        codes=["70551"],
        requires_pa=True,
        inclusion={
            "icd10_in": ["G40.909", "R51", "G43.909"],  # epilepsy, headache, migraine (illustrative)
        },
        exclusion={},
        admin={
            "pos_allowed": [11, 22],
            "max_units_per_day": 1
        },
        metadata={
            "source_url": "https://www.uhcprovider.com/en/policies-protocols/medical-benefit-drug-policies.html",
            "effective_date": "2025-10-01",
            "notes": "Illustrative; confirm indications & PA from UHC imaging policy list."
        }
    ),
    # Example: PET/CT (HCPCS 78815)
    Policy(
        policy_id="UHC-PET-CT-001",
        version="2025-10-01",
        payer="uhc",
        lob="commercial",
        codes=["78815"],
        requires_pa=True,
        inclusion={
            "icd10_in": ["C50911","C3490","C189"],  # breast, lung, colon CA (normalized form, illustrative)
        },
        exclusion={},
        admin={
            "pos_allowed": [22, 19],  # outpatient hospital, off-campus
            "max_units_per_day": 1
        },
        metadata={
            "source_url": "https://www.uhcprovider.com/",
            "effective_date": "2025-10-01",
            "notes": "Illustrative oncology PET coverage; confirm exact ICDs in policy."
        }
    ),
    # Example: BRCA testing (CPT 81211)
    Policy(
        policy_id="UHC-BRCA-GENETIC-TEST-001",
        version="2025-10-01",
        payer="uhc",
        lob="commercial",
        codes=["81211"],
        requires_pa=True,
        inclusion={
            "age_gte": 18,
            "icd10_in": ["Z8043","C50911"],  # family history of breast CA, personal hx (illustrative)
        },
        exclusion={},
        admin={
            "pos_allowed": [11, 22],
            "max_units_per_day": 1
        },
        metadata={
            "source_url": "https://www.uhcprovider.com/",
            "effective_date": "2025-10-01",
            "notes": "Illustrative; genetic testing criteria are detailed—replace with exact UHC policy."
        }
    ),
]

def write_rule_file(policy: Policy, canonical: dict, h: str) -> str:
    os.makedirs(OUT_DIR, exist_ok=True)
    fname = f"{policy.policy_id}_{policy.version}.json"
    path = os.path.join(OUT_DIR, fname)
    blob = dict(canonical)
    blob["policy_hash"] = h
    with open(path, "w") as f:
        json.dump(blob, f, indent=2, ensure_ascii=False)
    return path

def main():
    init_db()
    for p in SEED_POLICIES:
        h = insert_policy(p)
        path = write_rule_file(p, p.canonical_json(), h)
        print(f"[OK] {p.policy_id}@{p.version} -> {path}  hash={h}")

if __name__ == "__main__":
    main()
