"""
Tiny service API for your app to fetch the latest rules.json by code.
Usage:
    from rules_service import find_rules
    rules = find_rules(code="27447", payer="uhc", lob="commercial")
    if rules:
        print(rules["policy_hash"], rules["codes"])
"""
from typing import Optional, Dict, Any
from storage import get_policy_by_code
from policy_model import canonicalize

def find_rules(code: str, payer: Optional[str]=None, lob: Optional[str]=None) -> Optional[Dict[str, Any]]:
    rec = get_policy_by_code(code=code, payer=payer, lob=lob)
    if not rec:
        return None
    # Return canonical rules.json shape with policy_hash included
    rules = {
        "policy_id": rec["policy_id"],
        "version": rec["version"],
        "payer": rec["payer"],
        "lob": rec["lob"],
        "codes": rec["codes"],
        "requires_pa": bool(rec["requires_pa"]),
        "inclusion": rec["inclusion"],
        "exclusion": rec["exclusion"],
        "admin": rec["admin"],
        "metadata": rec["metadata"],
        "policy_hash": rec["policy_hash"],
    }
    return rules

if __name__ == "__main__":
    # quick manual test
    print(find_rules("27447", payer="uhc", lob="commercial"))
