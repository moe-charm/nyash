"""
CFG utilities
Build predecessor/successor maps and simple helpers.
"""

from typing import Dict, List, Any, Tuple

def build_preds_succs(block_by_id: Dict[int, Dict[str, Any]]) -> Tuple[Dict[int, List[int]], Dict[int, List[int]]]:
    """Construct predecessor and successor maps from MIR(JSON) blocks."""
    succs: Dict[int, List[int]] = {}
    preds: Dict[int, List[int]] = {}
    for b in block_by_id.values():
        bid = int(b.get('id', 0))
        preds.setdefault(bid, [])
    for b in block_by_id.values():
        src = int(b.get('id', 0))
        for inst in b.get('instructions', []) or []:
            op = inst.get('op')
            if op == 'jump':
                t = inst.get('target')
                if t is not None:
                    t = int(t)
                    succs.setdefault(src, []).append(t)
                    preds.setdefault(t, []).append(src)
            elif op == 'branch':
                th = inst.get('then'); el = inst.get('else')
                if th is not None:
                    th = int(th)
                    succs.setdefault(src, []).append(th)
                    preds.setdefault(th, []).append(src)
                if el is not None:
                    el = int(el)
                    succs.setdefault(src, []).append(el)
                    preds.setdefault(el, []).append(src)
    return preds, succs

