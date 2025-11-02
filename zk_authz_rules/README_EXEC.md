# Quick start

# 1) Python environment
python -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install --upgrade pip

# (no external deps needed; stdlib only)

# 2) Generate seed policies (writes to SQLite and ./rules_out/*.json)
python generate_rules.py

# You should see:
# [OK] UHC-KNEE-ARTHROPLASTY-001@2025-10-01 -> rules_out/UHC-KNEE-ARTHROPLASTY-001_2025-10-01.json hash=0x...

# 3) Fetch a rules.json for a code (for your app integration)
python -c "from rules_service import find_rules; import json; print(json.dumps(find_rules('27447','uhc','commercial'), indent=2))"

# 4) Replace illustrative ICD lists & metadata with real UHC values
#    - Edit SEED_POLICIES in generate_rules.py (add/remove policies)
#    - Re-run: python generate_rules.py
#    - New hashes will be computed; JSON files regenerated in ./rules_out

# Database file is rules.db (SQLite). Safe to ship with your app build if needed.
