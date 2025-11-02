Zero-Knowledge–Ready Policy Rules (UHC-focused) — README
Why this exists (30-second version)

You want a deterministic, versioned policy library (e.g., UnitedHealthcare prior-auth/coverage style rules) that your app can query as rules.json per CPT/HCPCS code.
These rules are:

Canonicalized & hashed → stable policy_hash for proofs/audits

Stored in SQLite → easy to update/ship

Emittable as JSON → your ZK prover/verifier can consume

This repo gives you:

A tiny policy DSL (age gates, ICD10 in, exclusions, admin checks, PA flag)

Seed policies (illustrative UHC-style — replace with real criteria)

A rules generator that writes JSON files + inserts into SQLite

A lookup function your main app can import: find_rules(code, payer, lob)

Table of Contents

Architecture & Plan

Policy DSL

Normalization & Hashing

What’s Implemented

How to Run

How to Fetch Rules from Your App

Extending with Real UHC Criteria

Hooking in ZK (Rough I/O Contract)

Folder Layout

Roadmap (Next 1–2 days)

FAQ

Architecture & Plan

Goal: “Given (payer, LOB, code), return deterministic rules.json + policy_hash.”
Inputs: curated policy rules (from UHC docs) with small ICD lists and basic gates.
Outputs: canonical JSON in ./rules_out (for inspection/sharing) and the same in SQLite (for programmatic lookup).

Flow (offline build):

Author/parse policy → DSL JSON

Canonicalize + compute policy_hash

Insert into SQLite + emit rules_out/<policy>@<version>.json

Flow (runtime use):

App calls find_rules(code, payer, lob) → gets the canonical rules.json (with policy_hash) to pass into your evaluation/ZK layers.

Later you’ll plug this into:

Deterministic evaluator (APPROVE/NEEDS_PA/DENY on private features)

ZK Prover (proves result with policy_hash & patient_commitment)

Policy DSL

Minimal, ZK-friendly structure:

{
  "policy_id": "UHC-KNEE-ARTHROPLASTY-001",
  "version": "2025-10-01",
  "payer": "uhc",
  "lob": "commercial",
  "codes": ["27447"],
  "requires_pa": true,
  "inclusion": {
    "age_gte": 50,
    "icd10_in": ["M170","M1711","M1712"],
    "failed_conservative": 1
  },
  "exclusion": {
    "icd10_in": ["M009"]
  },
  "admin": {
    "pos_allowed": [11,22],
    "max_units_per_day": 1
  },
  "metadata": {
    "source_url": "https://uhcprovider.com/…",
    "effective_date": "2025-10-01",
    "notes": "Illustrative; replace from UHC PDF."
  }
}


Operators supported by your downstream evaluator/ZK (recommended set):

Comparison: eq, neq, lt, lte, gt, gte

Membership: in (used as icd10_in)

Admin checks: pos_allowed, max_units_per_day

Boolean flags as 0/1 (e.g., failed_conservative)

All ICD10 codes in the JSON are normalized (upper, dotless).

Normalization & Hashing

ICD10 normalization:

Convert "M17.11" → "M1711"

Uppercase; strip whitespace

Canonicalization:

Sort object keys

Sort list fields that are set-like (codes, icd10_in, pos_allowed)

Hash: policy_hash = "0x" + SHA256(canonical_json)

This guarantees stable hashes and prevents accidental ordering drift.

What’s Implemented

Python files you have:

policy_model.py

Policy dataclass

Canonicalization + hashing helpers

ICD normalization (M17.11 → M1711)

Policy.hash() → returns policy_hash

storage.py

SQLite schema (rules.db)

init_db() to create tables

insert_policy(Policy) to write canonical JSON + policy_hash

get_policy_by_code(code, payer, lob) to read back the latest policy by code

generate_rules.py

Seed policies (illustrative UHC-style for demo)

python generate_rules.py:

initializes DB

inserts policies

emits canonical JSON + policy_hash into ./rules_out/

Replace the seed ICD lists/URLs with real UHC criteria whenever you’re ready

rules_service.py

Importable interface for your app:

from rules_service import find_rules
rules = find_rules("27447", payer="uhc", lob="commercial")


Returns canonical rules.json dict (already includes policy_hash)

How to Run
# 1) Create a venv (optional)
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install --upgrade pip

# 2) Generate seed policies into SQLite + ./rules_out
python generate_rules.py
# [OK] UHC-KNEE-ARTHROPLASTY-001@2025-10-01 -> rules_out/UHC-KNEE-ARTHROPLASTY-001_2025-10-01.json  hash=0x...

# 3) Fetch a rule (for your application integration)
python -c "from rules_service import find_rules; import json; print(json.dumps(find_rules('27447','uhc','commercial'), indent=2))"


You now have:

rules.db (SQLite) with canonical policies

rules_out/*.json with the same content + policy_hash for inspection

How to Fetch Rules from Your App
# app_policy_loader.py (example use)
from rules_service import find_rules

rules = find_rules(code="27447", payer="uhc", lob="commercial")
if not rules:
    raise RuntimeError("No policy found")
print(rules["policy_hash"], rules["codes"], rules["requires_pa"])


Return shape (example):

{
  "policy_id": "UHC-KNEE-ARTHROPLASTY-001",
  "version": "2025-10-01",
  "payer": "uhc",
  "lob": "commercial",
  "codes": ["27447"],
  "requires_pa": true,
  "inclusion": { "age_gte": 50, "icd10_in": ["M170","M1711","M1712"], "failed_conservative": 1 },
  "exclusion": { "icd10_in": ["M009"] },
  "admin": { "pos_allowed": [11,22], "max_units_per_day": 1 },
  "metadata": { "source_url": "https://…", "effective_date": "2025-10-01", "notes": "…" },
  "policy_hash": "0xabc123…"
}

Extending with Real UHC Criteria

Replace the seeds in generate_rules.py:

For each policy you care about, update:

codes (CPT/HCPCS)

requires_pa (true/false per UHC PA grids)

inclusion.icd10_in (normalize: dotless codes)

Any gates (e.g., age_gte, flags)

admin (e.g., pos_allowed)

metadata.source_url, effective_date, notes

Re-run python generate_rules.py → new JSON + new policy_hash

Tip: keep the policy_id stable and bump version when you refresh.
Your app can key decisions on policy_id@version + policy_hash.

Hooking in ZK (Rough I/O Contract)

When you add the prover, you’ll pass the canonical rules.json as either:

Public rules: include the whole JSON as a public input (simplest), or

Committed rules: keep rules private; expose only policy_hash and let the prover assert hash(policy_json) == policy_hash

Public inputs (suggestion):

policy_id, policy_hash, payer, lob, code
patient_commitment          # Hash(features || salt)
claimed_result              # APPROVE / NEEDS_PA / DENY


Private inputs (witness):

policy_json, features, salt


Verifier sees only:

{policy_id, policy_hash, code, patient_commitment, claimed_result, proof_blob}

Folder Layout
zk_authz_rules/
  policy_model.py       # DSL + canonicalization + hashing helpers
  storage.py            # SQLite CRUD + hash persistence
  generate_rules.py     # seed policies -> DB + ./rules_out JSON
  rules_service.py      # find_rules(code,payer,lob) -> canonical rules.json
  rules.db              # created at runtime
  rules_out/            # emitted canonical JSON files
  README_RULES.md       # this document

Roadmap (Next 1–2 days)

High-impact quick wins

Add a deterministic evaluator (pure function) so you can test APPROVE/NEEDS_PA/DENY locally before ZK.

Add multi-code support (vector of codes → array of results) if you want batching.

Add a tiny policy registry index (CSV or JSON) that enumerates: policy_id,version,policy_hash,source_url,effective_date,codes[].

Data realism

Swap illustrative seeds for real UHC policy/PA details (you can hand-curate 10–20 codes fast).

Validate ICDs against CDC ICD-10-CM tables; normalize on ingest.

ZK integration

zkVM (e.g., RISC Zero) guest that:

checks hash(policy_json)==policy_hash

checks hash(features||salt)==patient_commitment

runs deterministic rules → asserts result==claimed_result

FAQ

Q: Why canonicalize and hash policies?
A: To make every decision cryptographically tied to an exact policy version. This enables ZK proofs and external audits.

Q: Do I need to map ICDs to integers?
A: Not if you use a zkVM—you can compare normalized strings or their hashes. For circuits later, compare fixed-length arrays of hashed ICDs or use a Merkle set.

Q: Can I keep rules public?
A: Yes. ZK protects patient features, not policy text. Public rules are easiest for the demo and still fully ZK-compatible.

Q: One code or multiple codes per authorization?
A: Start with one code per rule/proof. Add multi-code later if needed; the storage and hashing model already supports it.

Credits / Notes

Seed policies in generate_rules.py are illustrative. Replace with actual UHC criteria you care about.

This backend is deliberately stdlib-only for portability. You can layer web APIs, scrapers, or LLM parsers later without changing the storage/DSL contracts.