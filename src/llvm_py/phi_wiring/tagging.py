from __future__ import annotations
from typing import Dict, List, Any

from .common import trace
from .analysis import analyze_incomings, collect_produced_stringish
from .wiring import ensure_phi


def setup_phi_placeholders(builder, blocks: List[Dict[str, Any]]):
    """Predeclare PHIs and collect incoming metadata for finalize_phis.

    Function-local: must be invoked after basic blocks are created and before
    lowering individual blocks. Also tags string-ish values to help downstream
    resolvers.
    """
    try:
        produced_str = collect_produced_stringish(blocks)
        builder.block_phi_incomings = analyze_incomings(blocks)
        trace({"phi": "setup", "produced_str_keys": list(produced_str.keys())})
        for block_data in blocks:
            bid0 = block_data.get("id", 0)
            bb0 = builder.bb_map.get(bid0)
            for inst in block_data.get("instructions", []) or []:
                if inst.get("op") != "phi":
                    continue
                try:
                    dst0 = int(inst.get("dst"))
                    from .common import incoming_pairs_vb
                incoming0 = incoming_pairs_vb(inst)
                except Exception:
                    dst0 = None
                    incoming0 = []
                if dst0 is None or bb0 is None:
                    continue
                # Do not materialize PHI here; finalize_phis will ensure and wire at block head.
                # _ = ensure_phi(builder, bid0, dst0, bb0)
                # Tag propagation
                try:
                    dst_type0 = inst.get("dst_type")
                    mark_str = (
                        isinstance(dst_type0, dict)
                        and dst_type0.get("kind") == "handle"
                        and dst_type0.get("box_type") == "StringBox"
                    )
                    if not mark_str:
                        # JSON v0 incoming pairs are (value, block)
                        for (v_src_i, _b_decl_i) in incoming0:
                            try:
                                if produced_str.get(int(v_src_i)):
                                    mark_str = True
                                    break
                            except Exception:
                                pass
                    if mark_str and hasattr(builder.resolver, "mark_string"):
                        builder.resolver.mark_string(int(dst0))
                except Exception:
                    pass
                # Definition hint: PHI defines dst in this block
                try:
                    builder.def_blocks.setdefault(int(dst0), set()).add(int(bid0))
                except Exception:
                    pass
        try:
            builder.resolver.block_phi_incomings = builder.block_phi_incomings
        except Exception:
            pass
    except Exception:
        pass
