from typing import Dict, Any, List
from llvmlite import ir
from trace import debug as trace_debug
from trace import phi_json as trace_phi_json

# 箱理論: PHI処理統一ハンドラー
from builders.phi_handler import PhiHandler
from builders.block_vmap import BlockVMap


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

        # Precompute simple i64 constant plan for this block to avoid "0 drop" when
        # an argument uses a value defined later in the same block (out-of-order const/copy).
        # We only plan numeric i64 constants and copy chains that resolve to them.
        try:
            if not hasattr(builder, 'block_i64_plan'):
                builder.block_i64_plan = {}
            plan_i64: dict[int, int] = {}
            aliases: dict[int, int] = {}
            # 1st pass: gather raw i64 consts and copy aliases
            for _inst in insts:
                try:
                    if _inst.get('op') == 'const':
                        v = _inst.get('value', {})
                        if v == None:
                            continue
                        if (v.get('type') == 'i64') or (isinstance(v.get('type'), str) and v.get('type') == 'i64'):
                            dst = _inst.get('dst')
                            ival = int(v.get('value'))
                            if isinstance(dst, int):
                                plan_i64[dst] = ival
                    elif _inst.get('op') == 'copy':
                        dst = _inst.get('dst')
                        src = _inst.get('src')
                        if isinstance(dst, int) and isinstance(src, int):
                            aliases[dst] = src
                except Exception:
                    pass
            # 2nd pass: resolve aliases transitively to numbers
            # Iterate a few times (depth small in practice)
            for _ in range(8):
                progressed = False
                for d, s in list(aliases.items()):
                    try:
                        if d in plan_i64:
                            continue
                        # if source resolves to number directly or via alias chain, record
                        cur = s
                        seen = set()
                        ok = False
                        while True:
                            if cur in plan_i64:
                                plan_i64[d] = plan_i64[cur]
                                ok = True
                                break
                            if cur not in aliases:
                                break
                            if cur in seen:
                                break
                            seen.add(cur)
                            cur = aliases[cur]
                        if ok:
                            progressed = True
                    except Exception:
                        pass
                if not progressed:
                    break
            builder.block_i64_plan[int(bid)] = plan_i64
            # Share with resolver for easy access
            try:
                builder.resolver.block_i64_plan = builder.block_i64_plan
            except Exception:
                pass
            # Expose raw alias chains (copy src/dst) for same-block value chasing (e.g., branch cond)
            try:
                if not hasattr(builder, 'block_aliases'):
                    builder.block_aliases = {}
                builder.block_aliases[int(bid)] = dict(aliases)
                try:
                    builder.resolver.block_aliases = builder.block_aliases
                except Exception:
                    pass
            except Exception:
                pass
        except Exception:
            # Non-fatal: plan is a bring-up optimization only
            pass

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
        # Per-block SSA map（箱化: BlockVMap）
        # Use a permissive local vmap that includes globals as-is.
        # Rationale: PHI values defined in predecessor headers must be visible
        # to successor blocks' body lowering (e.g., loop counters). Filtering to
        # same-block-only would hide dominating PHIs and cause 0-fallbacks.
        vmap_cur: Dict[int, ir.Value] = dict(builder.vmap)
        # Wrap local/global maps with BlockVMap for unified access
        bvm = BlockVMap(builder.vmap, vmap_cur)
        # Backward compat: expose dict view to existing lowering helpers
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
                # Ensure PHIs created at block head are visible in the block-local map
                # so that end-of-block snapshots include them for predecessor resolution.
                try:
                    for _p in phi_ops:
                        _d = _p.get('dst')
                        if isinstance(_d, int):
                            _val = builder.vmap.get(int(_d))
                            if _val is not None:
                                bvm.set(int(_d), _val)
                except Exception:
                    pass
            except Exception as e:
                # Fail-fast by default (箱理論: エラーは隠さず即座に失敗)
                if os.environ.get('NYASH_LLVM_PHI_LENIENT') == '1':
                    # Lenient mode: log and continue (development only)
                    trace_debug(f"[llvm-py] PHI processing error (lenient mode): {e}")
                    import traceback
                    traceback.print_exc()
                else:
                    # Fail-fast: raise immediately
                    raise RuntimeError(f"PHI processing failed at block {bb.name}") from e

        # 追加: プレ宣言された PHI プレースホルダをブロック先頭に必ず用意する
        # 目的:
        #  - ループ/分岐で block_phi_incomings に登録された (block,dst) に対し、
        #    後段の body 降下中に PHI 合成が発生しないよう、先に head で占位させる。
        try:
            from phi_wiring import ensure_phi as _ensure_phi
            decls = getattr(builder, 'block_phi_incomings', {}) or {}
            bdecl = decls.get(int(bid)) if decls is not None else None
            if isinstance(bdecl, dict):
                for _dst in list(bdecl.keys()):
                    # 既に同一ブロックのPHIが存在するならスキップ
                    cur = builder.vmap.get(int(_dst))
                    same_block = False
                    try:
                        same_block = (cur is not None and hasattr(cur, 'add_incoming') and getattr(getattr(cur, 'basic_block', None), 'name', None) == bb.name)
                    except Exception:
                        same_block = False
                    if not same_block:
                        _ensure_phi(builder, int(bid), int(_dst), bb)
        except Exception:
            pass

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
                                    bvm.set(dst, _gval)
                            else:
                                bvm.set(dst, _gval)
                        except Exception:
                            bvm.set(dst, _gval)
                    if dst not in created_ids and bvm.get(dst) is not None:
                        created_ids.append(dst)
            except Exception:
                pass

        # 箱理論: 未完了PHIの完成（ループPHI対応）
        # Complete incomplete PHI nodes after body ops are lowered (for loop PHIs)
        if phi_ops:
            try:
                phi_handler.complete_incomplete_phis()
                trace_debug(f"[llvm-py] Completed incomplete PHI edges")
            except Exception as e:
                trace_debug(f"[llvm-py] Warning: Failed to complete PHI edges: {e}")

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
                val = bvm.get(vid)
                if val is not None and hasattr(val, 'add_incoming'):
                    try:
                        builder.vmap[vid] = val
                    except Exception:
                        pass
        except Exception:
            pass
        # End-of-block snapshot（箱化: BlockVMap）
        snap = bvm.snapshot()
        try:
            keys = sorted(list(snap.keys()))
        except Exception:
            keys = list(snap.keys())
        trace_phi_json({"phi": "snapshot", "block": int(bid), "keys": [int(k) for k in keys[:20]]})
        for vid in created_ids:
            if bvm.get(vid) is not None:
                builder.def_blocks.setdefault(vid, set()).add(block_data.get("id", 0))
        builder.block_end_values[bid] = snap
        try:
            delattr(builder, '_current_vmap')
        except Exception:
            pass
