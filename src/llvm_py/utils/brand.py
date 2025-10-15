"""
Brand helpers (HAKO/HAKORUNE ↔ NYASH) for environment variables.

Goal
- Allow developers to use HAKO_* or HAKORUNE_* prefixes while the codebase
  continues to read NYASH_* for backward compatibility.

Policy
- Non-destructive: copy HAKO_/HAKU_/HRN_ → NYASH_ only when NYASH_* is unset.
- ROOT alias: HAKO_ROOT/HAKU_ROOT/HRN_ROOT → NYASH_ROOT
"""

from __future__ import annotations
import os


def alias_prefixes_bootstrap() -> None:
    # ROOT aliases first
    if os.getenv("NYASH_ROOT") is None:
        for k in ("HAKO_ROOT", "HAKU_ROOT", "HRN_ROOT"):
            v = os.getenv(k)
            if v is not None:
                os.environ["NYASH_ROOT"] = v
                break
    # General mapping: copy HAKO_/HAKU_/HRN_ → NYASH_
    snap = dict(os.environ)
    for k, v in snap.items():
        tail = None
        if k.startswith("HAKO_"):
            tail = k[len("HAKO_"):]
        elif k.startswith("HAKU_"):
            tail = k[len("HAKU_"):]
        elif k.startswith("HRN_"):
            tail = k[len("HRN_"):]
        if tail is None:
            continue
        ny = f"NYASH_{tail}"
        if os.getenv(ny) is None:
            os.environ[ny] = v

