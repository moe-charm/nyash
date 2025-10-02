"""
PhiDispatchPoint — unify value resolution around PHI/merge boundaries.

Policy (MVP):
- Prefer Resolver (strict, dominance-aware)
- If declared PHI exists for the value in current function: use that SSA
- If still inconclusive and we are in a loop/body: prefer the last add_ in
  the current block (increment pattern) as a structural hint
- Finally coerce to i64

Notes:
- This is a thin box to centralize the small but critical fallbacks that were
  previously scattered across compare/branch. It keeps behavior equivalent to
  the hardening patches while providing a single entry point.
"""

from typing import Any, Dict, Optional
import llvmlite.ir as ir


class PhiDispatchPoint:
    @staticmethod
    def _phi_from_decl(resolver, bb_map, vid: int):
        try:
            decls = getattr(resolver, 'block_phi_incomings', {}) if resolver is not None else {}
            for b, dmap in (decls or {}).items():
                if int(vid) in (dmap or {}):
                    bb = bb_map.get(int(b)) if bb_map is not None else None
                    if bb is None:
                        continue
                    for inst in getattr(bb, 'instructions', []):
                        try:
                            if not hasattr(inst, 'add_incoming'):
                                break
                            nm = str(getattr(inst, 'name', '') or '')
                            if nm.startswith('phi_'):
                                tail = nm[4:].split('.')[0]
                                if tail.isdigit() and int(tail) == int(vid):
                                    return inst
                        except Exception:
                            break
        except Exception:
            pass
        return None

    @staticmethod
    def _last_add_in_block(current_block: ir.Block):
        try:
            insts = list(getattr(current_block, 'instructions', []) or [])
        except Exception:
            insts = []
        for ins in reversed(insts):
            try:
                nm = str(getattr(ins, 'name', '') or '')
                if nm.startswith('add_'):
                    return ins
            except Exception:
                continue
        return None

    @staticmethod
    def _coerce_i64(builder: ir.IRBuilder, val: Any) -> ir.Value:
        i64 = ir.IntType(64)
        if val is None:
            return ir.Constant(i64, 0)
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            return builder.ptrtoint(val, i64)
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType) and val.type.width != 64:
            return builder.zext(val, i64)
        if isinstance(val, ir.Constant) and val.type == i64:
            return val
        return val

    @staticmethod
    def resolve_i64(builder: ir.IRBuilder,
                    resolver,
                    vid: int,
                    current_block: ir.Block,
                    preds: Dict[int, list],
                    block_end_values: Dict[int, Dict[int, Any]],
                    vmap: Dict[int, Any],
                    bb_map: Optional[Dict[int, ir.Block]] = None) -> ir.Value:
        # 0) Normalize via same-block alias chase (copy連鎖を基底値に畳み込む)
        base_vid = int(vid)
        try:
            bid = None
            name = str(current_block.name)
            if name.startswith('bb'):
                bid = int(name[2:])
            aliases = getattr(resolver, 'block_aliases', {}) if resolver is not None else {}
            amap = (aliases or {}).get(int(bid)) if bid is not None else None
            steps = 0
            seen = set()
            while amap and int(base_vid) in amap and steps < 8:
                nxt = amap.get(int(base_vid))
                if nxt is None or int(nxt) in seen:
                    break
                seen.add(int(base_vid))
                base_vid = int(nxt)
                steps += 1
        except Exception:
            base_vid = int(vid)

        # 1) Strict resolver path
        i64 = ir.IntType(64)
        if resolver is not None:
            try:
                val = resolver.resolve_i64(base_vid, current_block, preds, block_end_values, vmap, bb_map)
                if val is not None and not (isinstance(val, ir.Constant) and val.constant == 0):
                    return PhiDispatchPoint._coerce_i64(builder, val)
            except Exception:
                pass
        # 2) Declared PHI placeholder
        try:
            p = PhiDispatchPoint._phi_from_decl(resolver, bb_map, base_vid)
            if p is not None:
                return p
        except Exception:
            pass
        # 3) Last add in current block (increment patterns)
        try:
            addv = PhiDispatchPoint._last_add_in_block(current_block)
            if addv is not None:
                return PhiDispatchPoint._coerce_i64(builder, addv)
        except Exception:
            pass
        # 4) default zero
        return ir.Constant(i64, 0)
