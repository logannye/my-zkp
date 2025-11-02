"""
Service API to fetch rules from generated JSON files.
Usage:
    from rules_service import find_rules
    rules = find_rules(code="27447")
    if rules:
        print(rules["policy_id"], rules["codes"])
"""
import json
import os
from typing import Optional, Dict, Any
from pathlib import Path

# Rules directory
RULES_DIR = Path(__file__).parent / "rules"

def find_rules(code: str) -> Optional[Dict[str, Any]]:
    """
    Find rules.json file for a given CPT code.
    Returns the full rules JSON or None if not found.
    """
    rules_file = RULES_DIR / f"{code}.json"
    
    if not rules_file.exists():
        return None
    
    try:
        with open(rules_file, 'r') as f:
            rules = json.load(f)
        return rules
    except Exception as e:
        print(f"Error reading rules file {rules_file}: {e}")
        return None

def list_all_codes() -> list[str]:
    """
    List all CPT codes that have rules.json files.
    """
    if not RULES_DIR.exists():
        return []
    
    codes = []
    for json_file in RULES_DIR.glob("*.json"):
        code = json_file.stem
        codes.append(code)
    
    return sorted(codes)

def get_all_rules() -> Dict[str, Dict[str, Any]]:
    """
    Get all rules as a dictionary mapping CPT code to rules JSON.
    """
    all_rules = {}
    codes = list_all_codes()
    
    for code in codes:
        rules = find_rules(code)
        if rules:
            all_rules[code] = rules
    
    return all_rules

if __name__ == "__main__":
    # Quick manual test
    print("Testing rules_service...")
    print(f"All available codes: {len(list_all_codes())} codes")
    
    test_code = "27447"
    rules = find_rules(test_code)
    if rules:
        print(f"\nFound rules for {test_code}:")
        print(f"  Policy ID: {rules['policy_id']}")
        print(f"  Codes: {rules['codes']}")
        print(f"  Requires PA: {rules['requires_pa']}")
        print(f"  Inclusion rules: {len(rules['inclusion'])} criteria")
        print(f"  Exclusion rules: {len(rules['exclusion'])} criteria")
    else:
        print(f"\nNo rules found for {test_code}")
