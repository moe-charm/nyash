#!/usr/bin/env python3
"""
phi_trace_check.py — Validate LLVM PHI wiring trace (JSONL)

Usage:
  python3 tools/phi_trace_check.py --file tmp/phi_trace.jsonl [--strict-zero] [--summary]

Checks:
  - For each dst in finalize_dst events, ensure the set of preds from
    wire_choose equals the set from add_incoming.
  - Ensure at least one add_incoming exists per dst.
  - Optional (--strict-zero): fail if any [PHI] line reports zero=1.
  - Optional (--summary): print summary stats as JSON.

Input:
  Lines are JSON or plain text. JSON lines come from phi_wiring/common.trace
  and include objects like:
    {"phi":"finalize_dst","block":3,"dst":12,"incoming":[[7,1],[9,2]]}
    {"phi":"wire_choose","pred":1,"dst":12,"src":7}
    {"phi":"add_incoming","dst":12,"pred":1}

Plain text lines may look like:
  "[PHI] bb3 v12 incoming=2 zero=0"
These are used only for --strict-zero detection.
"""

import argparse
import json
import re
import sys
from collections import defaultdict


def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--file", required=True, help="Path to JSONL trace (NYASH_LLVM_TRACE_OUT)")
    ap.add_argument("--strict-zero", action="store_true", help="Fail if any PHI reports zero=1")
    ap.add_argument("--summary", action="store_true", help="Print summary stats")
    return ap.parse_args()


def main():
    args = parse_args()
    choose_preds = defaultdict(set)  # dst -> {preds chosen}
    add_preds = defaultdict(set)     # dst -> {preds added}
    finalize_dst = set()             # dst ids that were finalized
    zero_flag = set()                # dst ids with synthesized zero

    line_no = 0
    with open(args.file, "r", encoding="utf-8") as f:
        for raw in f:
            line_no += 1
            s = raw.strip()
            if not s:
                continue
            obj = None
            if s.startswith("{"):
                try:
                    obj = json.loads(s)
                except Exception:
                    obj = None
            if isinstance(obj, dict) and obj.get("phi"):
                kind = obj.get("phi")
                if kind == "wire_choose":
                    dst = int(obj.get("dst"))
                    pred = int(obj.get("pred"))
                    choose_preds[dst].add(pred)
                elif kind == "add_incoming":
                    dst = int(obj.get("dst"))
                    pred = int(obj.get("pred"))
                    add_preds[dst].add(pred)
                elif kind == "finalize_dst":
                    dst = int(obj.get("dst"))
                    finalize_dst.add(dst)
                # Other kinds are informational
                continue
            # Fallback: parse plain text PHI line for zero detection
            m = re.search(r"\[PHI\].*v(\d+).*zero=(\d)", s)
            if m:
                try:
                    dst = int(m.group(1))
                    zero = int(m.group(2))
                    if zero == 1:
                        zero_flag.add(dst)
                except Exception:
                    pass

    errors = []
    diffs = {}
    # Validate per-dst consistency
    all_dsts = finalize_dst.union(choose_preds.keys()).union(add_preds.keys())
    for dst in sorted(all_dsts):
        cset = choose_preds.get(dst, set())
        aset = add_preds.get(dst, set())
        if not aset:
            errors.append(f"dst v{dst}: no add_incoming recorded")
        if cset and (cset != aset):
            missing = sorted(list(cset - aset))
            extra = sorted(list(aset - cset))
            diffs[dst] = {"missing": missing, "extra": extra, "choose": sorted(list(cset)), "add": sorted(list(aset))}
            errors.append(f"dst v{dst}: mismatch preds (missing={missing} extra={extra})")
        if args.strict_zero and dst in zero_flag:
            errors.append(f"dst v{dst}: synthesized zero detected (strict-zero)")

    if args.summary:
        total_dsts = len(all_dsts)
        mismatches = sum(1 for dst in all_dsts if choose_preds.get(dst, set()) and (choose_preds.get(dst, set()) != add_preds.get(dst, set())))
        no_add = sum(1 for dst in all_dsts if not add_preds.get(dst))
        zeros = len(zero_flag)
        print(json.dumps({
            "dst_total": total_dsts,
            "mismatch": mismatches,
            "no_add": no_add,
            "zero_flag": zeros,
        }, separators=(",", ":")))

    if errors:
        for e in errors:
            print(f"[phi_trace_check] ERROR: {e}")
        if diffs:
            print(json.dumps({"diffs": diffs}, separators=(",", ":")))
        return 1
    print("[phi_trace_check] OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
