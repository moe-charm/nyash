from typing import Dict, Any, List
from llvmlite import ir
from trace import debug as trace_debug
from trace import phi_json as trace_phi_json

# 箱理論: PHI処理統一ハンドラー
from builders.phi_handler import PhiHandler


def lower_blocks(builder, func: ir.Function, block_by_id: Dict[int, Dict[str, Any]], order: List[int], loop_plan: Dict[str, Any] | None):
    skipped: set[int] = set()
    if loop_plan is not None:
        try:
            for bskip in loop_plan.get('skip_blocks', []):
                if bskip != loop_plan.get('header'):
                    skipped.add(int(bskip))
        except Exception:
            pass
    for bid in order:
        block_data = block_by_id.get(bid)
        if block_data is None:
            continue
        # If loop prepass applies, lower while once at header and skip loop-internal blocks
        if loop_plan is not None and bid == loop_plan.get('header'):
            bb = builder.bb_map[bid]
            ib = ir.IRBuilder(bb)
            try:
                builder.resolver.builder = ib
                builder.resolver.module = builder.module
            except Exception:
                pass
            builder.loop_count += 1
            body_insts = loop_plan.get('body_insts', [])
            cond_vid = loop_plan.get('cond')
            from instructions.loopform import lower_while_loopform
            ok = False
            try:
                builder._current_vmap = dict(builder.vmap)
                ok = lower_while_loopform(
                    ib,
                    func,
                    cond_vid,
                    body_insts,
                    builder.loop_count,
                    builder.vmap,
                    builder.bb_map,
                    builder.resolver,
                    builder.preds,
                    builder.block_end_values,
                    getattr(builder, 'ctx', None),
                )
            except Exception:
                ok = False
            if not ok:
                try:
                    builder.resolver._owner_lower_instruction = builder.lower_instruction
                except Exception:
                    pass
                from instructions.controlflow.while_ import lower_while_regular
                lower_while_regular(ib, func, cond_vid, body_insts,
                                    builder.loop_count, builder.vmap, builder.bb_map,
                                    builder.resolver, builder.preds, builder.block_end_values)
            try:
                delattr(builder, '_current_vmap')
            except Exception:
                pass
            for bskip in loop_plan.get('skip_blocks', []):
                skipped.add(bskip)
            # Ensure skipped original blocks have a valid terminator: branch to while exit
            try:
                exit_name = f"while{builder.loop_count}_exit"
                exit_bb = None
                for bbf in func.blocks:
                    try:
                        if str(bbf.name) == exit_name:
                            exit_bb = bbf
                            break
                    except Exception:
                        pass
                if exit_bb is not None:
                    try:
                        orig_exit_bb = builder.bb_map.get(loop_plan.get('exit'))
                        if orig_exit_bb is not None and exit_bb.terminator is None:
                            ibx = ir.IRBuilder(exit_bb)
                            ibx.branch(orig_exit_bb)
                    except Exception:
                        pass
                    for bskip in loop_plan.get('skip_blocks', []):
                        if bskip == loop_plan.get('header'):
                            continue
                        bb_skip = builder.bb_map.get(bskip)
                        if bb_skip is None:
                            continue
                        try:
                            if bb_skip.terminator is None:
                                ib = ir.IRBuilder(bb_skip)
                                if orig_exit_bb is not None:
                                    ib.branch(orig_exit_bb)
                        except Exception:
                            pass
            except Exception:
                pass
            continue

        if bid in skipped:
            continue
        bb = builder.bb_map[bid]
        ib = ir.IRBuilder(bb)
        try:
            builder.resolver.builder = ib
            builder.resolver.module = builder.module
        except Exception:
            pass
        block_data = block_by_id.get(bid, {})
        insts = block_data.get('instructions', []) or []

        # 箱理論: PhiHandlerでPHI命令を分離
        import os
        phi_verbose = os.environ.get('NYASH_PHI_VERBOSE') == '1'
        phi_handler = PhiHandler(builder, verbose=phi_verbose)
        phi_ops, non_phi_insts = phi_handler.collect_phi_instructions(insts)

        # Split non-PHI instructions into body and terminator ops
        body_ops: List[Dict[str, Any]] = []
        term_ops: List[Dict[str, Any]] = []
        for inst in non_phi_insts:
            try:
                opx = inst.get('op')
            except Exception:
                opx = None
            if opx in ("ret","jump","branch"):
                term_ops.append(inst)
            else:
                body_ops.append(inst)
        # Per-block SSA map
        vmap_cur: Dict[int, ir.Value] = {}
        try:
            for _vid, _val in (builder.vmap or {}).items():
                keep = True
                try:
                    if hasattr(_val, 'add_incoming'):
                        bb_of = getattr(getattr(_val, 'basic_block', None), 'name', None)
                        keep = (bb_of == bb.name)
                except Exception:
                    keep = False
                if keep:
                    vmap_cur[_vid] = _val
        except Exception:
            vmap_cur = dict(builder.vmap)
        builder._current_vmap = vmap_cur
        created_ids: List[int] = []
        defined_here_all: set = set()
        for _inst in body_ops:
            try:
                d = _inst.get('dst')
                if isinstance(d, int):
                    defined_here_all.add(d)
            except Exception:
                pass

        # 箱理論: PHI命令を最初に処理（ブロック先頭）
        if phi_ops:
            try:
                phi_handler.process_phi_instructions(phi_ops, bb, func)
                trace_debug(f"[llvm-py] Processed {len(phi_ops)} PHI instructions at block head")
            except Exception as e:
                trace_debug(f"[llvm-py] PHI processing error: {e}")
                import traceback
                traceback.print_exc()

        # Lower body ops
        for i_idx, inst in enumerate(body_ops):
            try:
                trace_debug(f"[llvm-py] body op: {inst.get('op')} dst={inst.get('dst')} cond={inst.get('cond')}")
            except Exception:
                pass
            try:
                if bb.terminator is not None:
                    break
            except Exception:
                pass
            ib.position_at_end(bb)
            if inst.get('op') == 'copy':
                src_i = inst.get('src')
                skip_now = False
                if isinstance(src_i, int):
                    try:
                        for _rest in body_ops[i_idx+1:]:
                            try:
                                if int(_rest.get('dst')) == int(src_i):
                                    skip_now = True
                                    break
                            except Exception:
                                pass
                    except Exception:
                        pass
                if skip_now:
                    pass
                else:
                    builder.lower_instruction(ib, inst, func)
            else:
                builder.lower_instruction(ib, inst, func)
            try:
                dst = inst.get("dst")
                if isinstance(dst, int):
                    if dst in builder.vmap:
                        _gval = builder.vmap[dst]
                        try:
                            if hasattr(_gval, 'add_incoming'):
                                bb_of = getattr(getattr(_gval, 'basic_block', None), 'name', None)
                                if bb_of == bb.name:
                                    vmap_cur[dst] = _gval
                            else:
                                vmap_cur[dst] = _gval
                        except Exception:
                            vmap_cur[dst] = _gval
                    if dst not in created_ids and dst in vmap_cur:
                        created_ids.append(dst)
            except Exception:
                pass
        # Lower terminators
        for inst in term_ops:
            try:
                trace_debug(f"[llvm-py] term op: {inst.get('op')} dst={inst.get('dst')} cond={inst.get('cond')}")
            except Exception:
                pass
            try:
                if bb.terminator is not None:
                    break
            except Exception:
                pass
            ib.position_at_end(bb)
            builder.lower_instruction(ib, inst, func)
        try:
            for vid in created_ids:
                val = vmap_cur.get(vid)
                if val is not None and hasattr(val, 'add_incoming'):
                    try:
                        builder.vmap[vid] = val
                    except Exception:
                        pass
        except Exception:
            pass
        # End-of-block snapshot
        snap = dict(vmap_cur)
        try:
            keys = sorted(list(snap.keys()))
        except Exception:
            keys = list(snap.keys())
        trace_phi_json({"phi": "snapshot", "block": int(bid), "keys": [int(k) for k in keys[:20]]})
        for vid in created_ids:
            if vid in vmap_cur:
                builder.def_blocks.setdefault(vid, set()).add(block_data.get("id", 0))
        builder.block_end_values[bid] = snap
        try:
            delattr(builder, '_current_vmap')
        except Exception:
            pass
