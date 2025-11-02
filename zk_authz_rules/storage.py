import sqlite3
import json
from typing import List, Dict, Any, Optional, Tuple
from policy_model import Policy, policy_hash, canonicalize

DB_PATH = "rules.db"

SCHEMA = """
CREATE TABLE IF NOT EXISTS policies (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  policy_id TEXT NOT NULL,
  version TEXT NOT NULL,
  payer TEXT NOT NULL,
  lob TEXT NOT NULL,
  codes_json TEXT NOT NULL,
  requires_pa INTEGER NOT NULL,
  inclusion_json TEXT NOT NULL,
  exclusion_json TEXT NOT NULL,
  admin_json TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  policy_hash TEXT NOT NULL,
  UNIQUE(policy_id, version)
);

CREATE INDEX IF NOT EXISTS idx_code ON policies (policy_id);
"""

def connect():
    conn = sqlite3.connect(DB_PATH)
    return conn

def init_db() -> None:
    with connect() as cx:
        cx.executescript(SCHEMA)

def insert_policy(p: Policy) -> str:
    p.validate()
    can = p.canonical_json()
    h = policy_hash(can)
    with connect() as cx:
        cx.execute(
            """INSERT OR REPLACE INTO policies
               (policy_id, version, payer, lob, codes_json, requires_pa, inclusion_json, exclusion_json, admin_json, metadata_json, policy_hash)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                p.policy_id, p.version, p.payer, p.lob,
                json.dumps(can["codes"]), int(can["requires_pa"]),
                json.dumps(can.get("inclusion", {})),
                json.dumps(can.get("exclusion", {})),
                json.dumps(can.get("admin", {})),
                json.dumps(can.get("metadata", {})),
                h,
            ),
        )
    return h

def get_policy_by_code(code: str, payer: Optional[str]=None, lob: Optional[str]=None) -> Optional[Dict[str, Any]]:
    q = """SELECT policy_id, version, payer, lob, codes_json, requires_pa, inclusion_json, exclusion_json, admin_json, metadata_json, policy_hash
           FROM policies
           WHERE json_extract(codes_json, '$') LIKE ?"""
    params = [f'%"{code}"%']
    if payer:
        q += " AND payer = ?"
        params.append(payer)
    if lob:
        q += " AND lob = ?"
        params.append(lob)
    q += " ORDER BY version DESC LIMIT 1"
    with connect() as cx:
        cur = cx.execute(q, params)
        row = cur.fetchone()
        if not row:
            return None
        keys = ["policy_id","version","payer","lob","codes_json","requires_pa","inclusion_json","exclusion_json","admin_json","metadata_json","policy_hash"]
        rec = dict(zip(keys, row))
        # Rehydrate JSON fields
        rec["codes"] = json.loads(rec.pop("codes_json"))
        rec["inclusion"] = json.loads(rec.pop("inclusion_json"))
        rec["exclusion"] = json.loads(rec.pop("exclusion_json"))
        rec["admin"] = json.loads(rec.pop("admin_json"))
        rec["metadata"] = json.loads(rec.pop("metadata_json"))
        return rec
