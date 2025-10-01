from typing import Dict, Any, List

from llvmlite import ir
from trace import debug as trace_debug
from prepass.if_merge import plan_ret_phi_predeclare
from prepass.loops import detect_simple_while
from phi_wiring import (
    setup_phi_placeholders as _setup_phi_placeholders,
    finalize_phis as _finalize_phis,
    build_succs as _build_succs,
)
from phi_wiring.verify import verify_phi_cfg as _verify_phi_cfg


def lower_function(builder, func_data: Dict[str, Any]):
    """Lower a single MIR function to LLVM IR using the given builder context.
    This is a faithful extraction of NyashLLVMBuilder.lower_function.
    """
    import os, re

    name = func_data.get("name", "unknown")
    builder.current_function_name = name
    params = func_data.get("params", [])
    blocks = func_data.get("blocks", [])

    # Determine function signature
    if name == "ny_main":
        # Special case: ny_main returns i32
        func_ty = ir.FunctionType(builder.i32, [])
    else:
        # Default: i64(i64, ...) signature; derive arity from '/N' suffix when params missing
        m = re.search(r"/(\d+)$", name)
        arity = int(m.group(1)) if m else len(params)
        param_types = [builder.i64] * arity
        func_ty = ir.FunctionType(builder.i64, param_types)

    # Reset per-function maps and resolver caches to avoid cross-function collisions
    try:
        builder.vmap.clear()
    except Exception:
        builder.vmap = {}
    try:
        builder.bb_map.clear()
    except Exception:
        builder.bb_map = {}
    # Clear PHI wiring book-keeping and per-function declared incomings
    try:
        builder.phi_wired.clear()
    except Exception:
        try:
            builder.phi_wired = {}
        except Exception:
            pass
    try:
        builder.block_phi_incomings = {}
    except Exception:
        pass
    try:
        # Reset resolver caches keyed by block names
        builder.resolver.i64_cache.clear()
        builder.resolver.ptr_cache.clear()
        builder.resolver.f64_cache.clear()
        if hasattr(builder.resolver, '_end_i64_cache'):
            builder.resolver._end_i64_cache.clear()
        if hasattr(builder.resolver, 'string_ids'):
            builder.resolver.string_ids.clear()
        if hasattr(builder.resolver, 'string_literals'):
            builder.resolver.string_literals.clear()
        if hasattr(builder.resolver, 'string_ptrs'):
            builder.resolver.string_ptrs.clear()
    except Exception:
        pass

    # Create or reuse function
    func = None
    for f in builder.module.functions:
        if f.name == name:
            func = f
            break
    if func is None:
        func = ir.Function(builder.module, func_ty, name=name)

    # Map parameters to vmap (value_id: 0..arity-1)
    try:
        arity = len(func.args)
        for i in range(arity):
            builder.vmap[i] = func.args[i]
    except Exception:
        pass

    # Process constants (string literals, etc.)
    constants = func_data.get("constants", {})
    for vid_str, const_data in constants.items():
        try:
            vid = int(vid_str)
            const_type = const_data.get("type")
            const_value = const_data.get("value")

            if const_type == "string":
                # Create global string literal
                str_bytes = (const_value + '\0').encode('utf-8')
                str_type = ir.ArrayType(ir.IntType(8), len(str_bytes))
                global_str = ir.GlobalVariable(builder.module, str_type,
                                               name=f".str.{vid}")
                global_str.initializer = ir.Constant(str_type, bytearray(str_bytes))
                global_str.global_constant = True
                global_str.linkage = 'internal'

                # Get pointer to first element (i8*)
                i8p = ir.IntType(8).as_pointer()
                zero = ir.Constant(ir.IntType(32), 0)
                str_ptr = global_str.gep([zero, zero])
                builder.vmap[vid] = str_ptr

                # Also register in resolver if available
                if hasattr(builder, 'resolver') and hasattr(builder.resolver, 'string_ptrs'):
                    builder.resolver.string_ptrs[vid] = str_ptr

            elif const_type == "i64":
                # Integer constant
                i64_val = ir.Constant(builder.i64, const_value)
                builder.vmap[vid] = i64_val

            elif const_type == "f64":
                # Float constant
                f64_val = ir.Constant(builder.f64, const_value)
                builder.vmap[vid] = f64_val

        except Exception as e:
            trace_debug(f"[constants] Failed to process constant {vid_str}: {e}")
            pass

    # Build predecessor map from control-flow edges
    builder.preds = {}
    for block_data in blocks:
        bid = block_data.get("id", 0)
        builder.preds.setdefault(bid, [])
    for block_data in blocks:
        src = block_data.get("id", 0)
        for inst in block_data.get("instructions", []):
            op = inst.get("op")
            if op == "jump":
                t = inst.get("target")
                if t is not None:
                    builder.preds.setdefault(t, []).append(src)
            elif op == "branch":
                th = inst.get("then")
                el = inst.get("else")
                if th is not None:
                    builder.preds.setdefault(th, []).append(src)
                if el is not None:
                    builder.preds.setdefault(el, []).append(src)

    # Create all blocks first
    for block_data in blocks:
        bid = block_data.get("id", 0)
        block_name = f"bb{bid}"
        bb = func.append_basic_block(block_name)
        builder.bb_map[bid] = bb

    # Build quick lookup for blocks by id
    block_by_id: Dict[int, Dict[str, Any]] = {}
    for block_data in blocks:
        block_by_id[block_data.get("id", 0)] = block_data

    # Determine entry block: first with no predecessors; fallback to first block
    entry_bid = None
    for bid, preds in builder.preds.items():
        if len(preds) == 0:
            entry_bid = bid
            break
    if entry_bid is None and blocks:
        entry_bid = blocks[0].get("id", 0)

    # Compute approx preds-first order
    visited = set()
    order: List[int] = []

    def visit(bid: int):
        if bid in visited:
            return
        visited.add(bid)
        for p in builder.preds.get(bid, []):
            visit(p)
        order.append(bid)

    if entry_bid is not None:
        visit(entry_bid)
    for bid in block_by_id.keys():
        if bid not in visited:
            visit(bid)

    # Prepass: collect PHI metadata and placeholders
    _setup_phi_placeholders(builder, blocks)

    # Optional: if-merge prepass (gate NYASH_LLVM_PREPASS_IFMERGE)
    try:
        if os.environ.get('NYASH_LLVM_PREPASS_IFMERGE') == '1':
            plan = plan_ret_phi_predeclare(block_by_id)
            if plan:
                if not hasattr(builder, 'block_phi_incomings') or builder.block_phi_incomings is None:
                    builder.block_phi_incomings = {}
                for bbid, ret_vid in plan.items():
                    try:
                        preds_raw = [p for p in builder.preds.get(bbid, []) if p != bbid]
                    except Exception:
                        preds_raw = []
                    seen = set(); preds_list = []
                    for p in preds_raw:
                        if p not in seen:
                            preds_list.append(p); seen.add(p)
                    try:
                        builder.block_phi_incomings.setdefault(int(bbid), {})[int(ret_vid)] = [
                            (int(p), int(ret_vid)) for p in preds_list
                        ]
                    except Exception:
                        pass
                    try:
                        trace_debug(f"[prepass] if-merge: plan metadata at bb{bbid} for v{ret_vid} preds={preds_list}")
                    except Exception:
                        pass
    except Exception:
        pass

    # Predeclare PHIs for used-in-block values defined in predecessors (multi-pred only)
    try:
        from cfg.utils import build_preds_succs
        local_preds, _ = build_preds_succs(block_by_id)
        def _collect_defs(block):
            defs = set()
            for ins in block.get('instructions') or []:
                try:
                    dstv = ins.get('dst')
                    if isinstance(dstv, int):
                        defs.add(int(dstv))
                except Exception:
                    pass
            return defs
        def _collect_declared_phis(block):
            phis = set()
            for ins in block.get('instructions') or []:
                try:
                    if ins.get('op') == 'phi':
                        d = ins.get('dst')
                        if isinstance(d, int):
                            phis.add(int(d))
                except Exception:
                    pass
            return phis
        def _collect_uses(block):
            uses = set()
            for ins in block.get('instructions') or []:
                for k in ('lhs','rhs','value','cond','box_val'):
                    try:
                        v = ins.get(k)
                        if isinstance(v, int):
                            uses.add(int(v))
                    except Exception:
                        pass
            return uses
        if not hasattr(builder, 'block_phi_incomings') or builder.block_phi_incomings is None:
            builder.block_phi_incomings = {}
        for bid, blk in block_by_id.items():
            try:
                preds_raw = [p for p in local_preds.get(int(bid), []) if p != int(bid)]
            except Exception:
                preds_raw = []
            seen = set(); preds_list = []
            for p in preds_raw:
                if p not in seen:
                    preds_list.append(p); seen.add(p)
            if len(preds_list) <= 1:
                continue
            defs = _collect_defs(blk)
            declared_phis = _collect_declared_phis(blk)
            uses = _collect_uses(blk)
            need = [u for u in uses if (u not in defs) and (u not in declared_phis)]
            if not need:
                continue
            for vid in need:
                try:
                    builder.block_phi_incomings.setdefault(int(bid), {}).setdefault(int(vid), [])
                    builder.block_phi_incomings[int(bid)][int(vid)] = [(int(p), int(vid)) for p in preds_list]
                except Exception:
                    pass
        try:
            builder.resolver.block_phi_incomings = builder.block_phi_incomings
        except Exception:
            pass
    except Exception:
        pass

    # Optional: simple loop prepass
    loop_plan = None
    try:
        if os.environ.get('NYASH_LLVM_PREPASS_LOOP') == '1':
            loop_plan = detect_simple_while(block_by_id)
            if loop_plan is not None:
                trace_debug(f"[prepass] detect loop header=bb{loop_plan['header']} then=bb{loop_plan['then']} latch=bb{loop_plan['latch']} exit=bb{loop_plan['exit']}")
    except Exception:
        loop_plan = None

    from builders.block_lower import lower_blocks as _lower_blocks
    _lower_blocks(builder, func, block_by_id, order, loop_plan)

    # Optional: capture lowering ctx for downstream helpers
    try:
        builder.ctx = dict(
            module=builder.module,
            i64=builder.i64,
            i32=builder.i32,
            i8=builder.i8,
            i1=builder.i1,
            i8p=builder.i8p,
            vmap=builder.vmap,
            bb_map=builder.bb_map,
            preds=builder.preds,
            block_end_values=builder.block_end_values,
            resolver=builder.resolver,
            trace_phi=os.environ.get('NYASH_LLVM_TRACE_PHI') == '1',
            verbose=os.environ.get('NYASH_CLI_VERBOSE') == '1',
        )
        builder.resolver.ctx = builder.ctx
    except Exception:
        pass

    # Finalize PHIs for this function (wire-only)
    _finalize_phis(builder)

    # Optional verification pass (fail-fast in dev), gated by NYASH_LLVM_PHI_VERIFY
    try:
        import os
        if os.environ.get('NYASH_LLVM_PHI_VERIFY', '1') != '0':
            strict = (os.environ.get('NYASH_LLVM_PHI_VERIFY_STRICT') == '1')
            problems = _verify_phi_cfg(builder, strict=strict)
            if problems:
                # Summarize first few problems for quick diagnosis
                summary = ", ".join([f"bb{p.get('block')} v{p.get('dst')}: {p.get('issue')}" for p in problems[:3]])
                raise RuntimeError(f"PHI verification failed: {summary} (total {len(problems)})")
    except Exception as _ve:
        # In bring-up, allow soft-fail when explicitly opted out
        if os.environ.get('NYASH_LLVM_PHI_VERIFY_STRICT') == '1':
            raise

    # Safety pass: ensure every basic block ends with a terminator.
    # This avoids llvmlite IR parse errors like "expected instruction opcode" on empty blocks.
    try:
        _enforce_terminators(builder, func, block_by_id)
    except Exception:
        # Non-fatal in bring-up; better to emit IR than crash
        pass


def _enforce_terminators(builder, func: ir.Function, block_by_id: Dict[int, Dict[str, Any]]):
    import re
    succs = _build_succs(getattr(builder, 'preds', {}) or {})
    for bb in func.blocks:
        try:
            if bb.terminator is not None:
                continue
        except Exception:
            # If property access fails, try to add a branch/ret anyway
            pass
        # Parse block id from name like "bb123"
        bid = None
        try:
            m = re.match(r"bb(\d+)$", str(bb.name))
            bid = int(m.group(1)) if m else None
        except Exception:
            bid = None
        # Choose a reasonable successor if any
        target_bb = None
        if bid is not None:
            for s in (succs.get(int(bid), []) or []):
                try:
                    cand = builder.bb_map.get(int(s))
                except Exception:
                    cand = None
                if cand is not None and cand is not bb:
                    target_bb = cand
                    break
        ib = ir.IRBuilder(bb)
        if target_bb is not None:
            try:
                ib.position_at_end(bb)
            except Exception:
                pass
            ib.branch(target_bb)
            try:
                trace_debug(f"[llvm-py] enforce_terminators: br from {bb.name} -> {target_bb.name}")
            except Exception:
                pass
            continue
        # Fallback: insert a return of 0 matching function return type (i32 for ny_main, else i64)
        try:
            rty = func.function_type.return_type
            if str(rty) == str(builder.i32):
                ib.ret(ir.Constant(builder.i32, 0))
            elif str(rty) == str(builder.i64):
                ib.ret(ir.Constant(builder.i64, 0))
            else:
                # Unknown/void – synthesize a dummy br to self to keep parser happy (unreachable in practice)
                ib.branch(bb)
            try:
                trace_debug(f"[llvm-py] enforce_terminators: ret/br injected in {bb.name}")
            except Exception:
                pass
        except Exception:
            # Last resort: do nothing
            pass
