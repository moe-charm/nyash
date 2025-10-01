from __future__ import annotations
from typing import Any
import os
import json

def trace(msg: Any):
    if os.environ.get("NYASH_LLVM_TRACE_PHI", "0") != "1":
        return
    out = os.environ.get("NYASH_LLVM_TRACE_OUT")
    if not isinstance(msg, (str, bytes)):
        try:
            msg = json.dumps(msg, ensure_ascii=False, separators=(",", ":"))
        except Exception:
            msg = str(msg)
    if out:
        try:
            with open(out, "a", encoding="utf-8") as f:
                f.write(msg.rstrip() + "\n")
        except Exception:
            pass
    else:
        try:
            print(msg)
        except Exception:
            pass

def incoming_pairs_vb(inst: Any):
    """Return list of (value, block) pairs for PHI instruction JSON.

    Supports new shape ("values": [{"value":v, "block":b}, ...]) and
    falls back to legacy shape ("incoming": [[v,b], ...]).
    """
    try:
        vals = inst.get("values")
        if isinstance(vals, list):
            out = []
            for it in vals:
                if isinstance(it, dict):
                    v = it.get("value")
                    b = it.get("block")
                    if v is not None and b is not None:
                        out.append((int(v), int(b)))
            return out
        inc = inst.get("incoming", []) or []
        out = []
        for t in inc:
            # Handle dict format (same as "values")
            if isinstance(t, dict):
                v = t.get("value")
                b = t.get("block")
                if v is not None and b is not None:
                    out.append((int(v), int(b)))
            # Handle legacy array format [[v,b], ...]
            elif isinstance(t, (list, tuple)) and len(t) == 2:
                v, b = t
                out.append((int(v), int(b)))
        return out
    except Exception:
        return []
