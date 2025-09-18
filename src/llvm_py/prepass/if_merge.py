"""
If-merge prepass utilities
For blocks that end with return and have multiple predecessors, plan PHI predeclare for return value ids.
"""

from typing import Dict, Any, Optional
from cfg.utils import build_preds_succs

def plan_ret_phi_predeclare(block_by_id: Dict[int, Dict[str, Any]]) -> Optional[Dict[int, int]]:
    """Return a map {block_id: value_id} for blocks that end with ret <value>
    and have multiple predecessors. The caller can predeclare a PHI for value_id
    at the block head to ensure dominance for the return.
    """
    preds, _ = build_preds_succs(block_by_id)
    plan: Dict[int, int] = {}
    for bid, blk in block_by_id.items():
        term = None
        if blk.get('instructions'):
            last = blk.get('instructions')[-1]
            if last.get('op') in ('jump','branch','ret'):
                term = last
        if term is None and 'terminator' in blk:
            t = blk['terminator']
            if t and t.get('op') in ('jump','branch','ret'):
                term = t
        if not term or term.get('op') != 'ret':
            continue
        val = term.get('value')
        if not isinstance(val, int):
            continue
        # Heuristic: skip when the return value is freshly defined in this block
        # (e.g., returning a const computed in cleanup). Predeclaring a PHI for such
        # values is unnecessary and may violate PHI grouping/order.
        try:
            defined_here = False
            for ins in blk.get('instructions') or []:
                if isinstance(ins, dict) and ins.get('dst') == int(val):
                    defined_here = True
                    break
            if defined_here:
                continue
        except Exception:
            pass
        pred_list = [p for p in preds.get(int(bid), []) if p != int(bid)]
        if len(pred_list) > 1:
            plan[int(bid)] = int(val)
    return plan or None
