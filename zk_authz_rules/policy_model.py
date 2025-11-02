"""
Policy model and utilities for rule processing.

This module provides utilities for working with the rule JSON format,
including canonicalization and hashing for ZK proof systems.
"""
from __future__ import annotations
from typing import List, Dict, Any, Optional
import json
import hashlib

JSON = Dict[str, Any]

# ---- Canonical JSON (stable hash) helpers ----

def canonicalize(rule_json: JSON) -> JSON:
    """
    Canonicalize rule JSON for deterministic hashing:
    - Sort dictionary keys
    - Sort arrays that represent sets (ICD codes, CPT codes, POS codes)
    - Normalize integer arrays
    """
    if isinstance(rule_json, dict):
        # Sort dictionary keys
        result = {}
        for key in sorted(rule_json.keys()):
            result[key] = canonicalize(rule_json[key])
        return result
    elif isinstance(rule_json, list):
        # Sort list items (for sets like ICD codes, CPT codes)
        items = [canonicalize(item) for item in rule_json]
        # If all items are the same type and sortable, sort them
        try:
            if all(isinstance(item, (int, str)) for item in items):
                return sorted(items)
            elif all(isinstance(item, dict) for item in items):
                # Sort dictionaries by their JSON representation
                return sorted(items, key=lambda x: json.dumps(x, sort_keys=True))
        except (TypeError, ValueError):
            pass
        return items
    else:
        return rule_json

def compute_policy_hash(rule_json: JSON) -> str:
    """
    Compute SHA256 hash of canonicalized rule JSON.
    Returns hex digest prefixed with '0x'.
    """
    canonical = canonicalize(rule_json)
    json_str = json.dumps(canonical, sort_keys=True, separators=(',', ':'))
    digest = hashlib.sha256(json_str.encode('utf-8')).hexdigest()
    return "0x" + digest

def validate_rule(rule_json: JSON) -> bool:
    """
    Validate that a rule JSON has the required structure.
    Returns True if valid, False otherwise.
    """
    required_fields = ["policy_id", "version", "lob", "codes", "requires_pa", "inclusion", "exclusion", "admin_rules"]
    
    for field in required_fields:
        if field not in rule_json:
            return False
    
    # Validate types
    if not isinstance(rule_json["policy_id"], str):
        return False
    if not isinstance(rule_json["version"], str):
        return False
    if not isinstance(rule_json["lob"], str):
        return False
    if not isinstance(rule_json["codes"], list):
        return False
    if not isinstance(rule_json["requires_pa"], bool):
        return False
    if not isinstance(rule_json["inclusion"], list):
        return False
    if not isinstance(rule_json["exclusion"], list):
        return False
    if not isinstance(rule_json["admin_rules"], dict):
        return False
    
    # Validate admin_rules
    admin = rule_json["admin_rules"]
    if "max_units_per_day" not in admin:
        return False
    if not isinstance(admin["max_units_per_day"], int):
        return False
    if "pos_allowed" in admin:
        if not isinstance(admin["pos_allowed"], list):
            return False
    
    # Validate inclusion/exclusion structure
    # Each item should be a dict with an operation (gte, lte, in, eq, etc.)
    for inclusion_item in rule_json["inclusion"]:
        if not isinstance(inclusion_item, dict):
            return False
    
    for exclusion_item in rule_json["exclusion"]:
        if not isinstance(exclusion_item, dict):
            return False
    
    return True

def normalize_icd_code(icd_code: str) -> str:
    """
    Normalize ICD-10 code: remove dots, uppercase.
    Example: "M17.11" -> "M1711"
    """
    return icd_code.replace(".", "").upper().strip()

def extract_inclusion_icd_codes(rule_json: JSON) -> List[int]:
    """
    Extract all ICD-10 codes (as integers) from inclusion criteria.
    Returns a sorted list of integer ICD codes.
    """
    icd_codes = []
    
    for item in rule_json.get("inclusion", []):
        if isinstance(item, dict) and "in" in item:
            op = item["in"]
            if isinstance(op, list) and len(op) == 2:
                field, codes = op
                if field == "primary_icd10" and isinstance(codes, list):
                    icd_codes.extend([c for c in codes if isinstance(c, int)])
    
    return sorted(list(set(icd_codes)))

def extract_exclusion_icd_codes(rule_json: JSON) -> List[int]:
    """
    Extract all ICD-10 codes (as integers) from exclusion criteria.
    Returns a sorted list of integer ICD codes.
    """
    icd_codes = []
    
    for item in rule_json.get("exclusion", []):
        if isinstance(item, dict) and "in" in item:
            op = item["in"]
            if isinstance(op, list) and len(op) == 2:
                field, codes = op
                if field == "primary_icd10" and isinstance(codes, list):
                    icd_codes.extend([c for c in codes if isinstance(c, int)])
    
    return sorted(list(set(icd_codes)))

# Backward compatibility: policy_hash function
def policy_hash(rule_json: JSON) -> str:
    """Alias for compute_policy_hash for backward compatibility."""
    return compute_policy_hash(rule_json)
