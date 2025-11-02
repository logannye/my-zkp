from __future__ import annotations
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Any, Optional, Union
import json
import hashlib
import time

JSON = Dict[str, Any]

# ---- Canonical JSON (stable hash) helpers ----

_CANON_LIST_KEYS_TREATED_AS_SETS = {
    ("inclusion", "icd10_in"),
    ("exclusion", "icd10_in"),
    ("admin", "pos_allowed"),
    ("codes",),  # top-level codes
}

def _normalize_icd(code: str) -> str:
    # Uppercase and strip dot to keep deterministic matching, e.g. "M17.11" -> "M1711"
    return code.replace(".", "").upper().strip()

def _canon(obj: Any, parent_path: tuple = ()) -> Any:
    """
    Canonicalize for deterministic hashing:
    - dict keys sorted
    - lists that represent sets are sorted (ICD lists, codes, pos_allowed)
    - ICD values normalized
    """
    if isinstance(obj, dict):
        return {k: _canon(v, parent_path + (k,)) for k, v in sorted(obj.items(), key=lambda kv: kv[0])}
    if isinstance(obj, list):
        val = [_canon(v, parent_path) for v in obj]
        # Sort "set-like" lists deterministically
        if parent_path in _CANON_LIST_KEYS_TREATED_AS_SETS or parent_path[-1:] in _CANON_LIST_KEYS_TREATED_AS_SETS:
            # If list contains strings that look like ICDs, normalize & sort lexicographically
            if val and isinstance(val[0], str):
                val = [_normalize_icd(v) for v in val]  # normalize ICD strings
                return sorted(val)
            # Otherwise just sort JSON-serializable content
            try:
                return sorted(val, key=lambda x: json.dumps(x, separators=(",", ":"), sort_keys=True))
            except Exception:
                return val
        return val
    if isinstance(obj, str) and ("icd" in "".join(parent_path).lower()):
        return _normalize_icd(obj)
    return obj

def canonicalize(policy: JSON) -> JSON:
    return _canon(policy)

def policy_hash(policy: JSON) -> str:
    can = canonicalize(policy)
    s = json.dumps(can, separators=(",", ":"), sort_keys=True)
    digest = hashlib.sha256(s.encode("utf-8")).hexdigest()
    return "0x" + digest

# ---- Policy dataclass & validation ----

@dataclass
class Policy:
    policy_id: str
    version: str
    payer: str
    lob: str
    codes: List[str]
    requires_pa: bool
    inclusion: JSON = field(default_factory=dict)
    exclusion: JSON = field(default_factory=dict)
    admin: JSON = field(default_factory=dict)
    metadata: JSON = field(default_factory=dict)

    def to_json(self) -> JSON:
        d = asdict(self)
        return d

    def canonical_json(self) -> JSON:
        return canonicalize(self.to_json())

    def hash(self) -> str:
        return policy_hash(self.to_json())

    def validate(self) -> None:
        assert self.policy_id and self.version and self.payer and self.lob
        assert isinstance(self.codes, list) and all(isinstance(c, str) for c in self.codes)
        # Inclusion/exclusion DSL sanity
        # Supported ops: eq, neq, lt, lte, gt, gte, in  (values must be ints / strings)
        def _check_block(block: JSON, name: str):
            if not block:
                return
            if "icd10_in" in block:
                assert isinstance(block["icd10_in"], list), f"{name}.icd10_in must be list"
            # optional: age gates etc. live inside inclusion/exclusion as list of primitive ops in your engine
        # keep flexible; rigor can be added later
        # Admin sanity
        if "pos_allowed" in self.admin:
            assert isinstance(self.admin["pos_allowed"], list)
        if "max_units_per_day" in self.admin:
            assert isinstance(self.admin["max_units_per_day"], int)

    @staticmethod
    def now_version_tag() -> str:
        return time.strftime("%Y-%m-%d")
