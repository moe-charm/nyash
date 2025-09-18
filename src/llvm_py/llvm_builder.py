#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/design/LLVM_LAYER_OVERVIEW.md
"""

import json
import sys
import os
from typing import Dict, Any, Optional, List, Tuple
import llvmlite.ir as ir
import llvmlite.binding as llvm

# Import instruction handlers
from instructions.const import lower_const
from instructions.binop import lower_binop
from instructions.compare import lower_compare
from instructions.controlflow.jump import lower_jump
from instructions.controlflow.branch import lower_branch
from instructions.ret import lower_return
from instructions.copy import lower_copy
# PHI are deferred; finalize_phis wires incoming edges after snapshots
from instructions.call import lower_call
from instructions.boxcall import lower_boxcall
from instructions.externcall import lower_externcall
from instructions.typeop import lower_typeop, lower_convert
from instructions.newbox import lower_newbox
from instructions.safepoint import lower_safepoint, insert_automatic_safepoint
from instructions.barrier import lower_barrier
from instructions.loopform import lower_while_loopform
from instructions.controlflow.while_ import lower_while_regular
from phi_wiring import setup_phi_placeholders as _setup_phi_placeholders, finalize_phis as _finalize_phis
from trace import debug as trace_debug
from trace import phi as trace_phi
try:
    # Structured JSON trace for PHI wiring (shared with phi_wiring)
    from phi_wiring.common import trace as trace_phi_json
except Exception:
    def trace_phi_json(_msg):
        pass
from prepass.loops import detect_simple_while
from prepass.if_merge import plan_ret_phi_predeclare
from build_ctx import BuildCtx

from resolver import Resolver
from mir_reader import MIRReader

class NyashLLVMBuilder:
    """Main LLVM IR builder for Nyash MIR"""
    
    def __init__(self):
        # Initialize LLVM
        llvm.initialize()
        llvm.initialize_native_target()
        llvm.initialize_native_asmprinter()
        
        # Module and basic types
        self.module = ir.Module(name="nyash_module")
        self.i64 = ir.IntType(64)
        self.i32 = ir.IntType(32)
        self.i8 = ir.IntType(8)
        self.i1 = ir.IntType(1)
        self.i8p = self.i8.as_pointer()
        self.f64 = ir.DoubleType()
        self.void = ir.VoidType()
        
        # Value and block maps
        self.vmap: Dict[int, ir.Value] = {}  # value_id -> LLVM value
        self.bb_map: Dict[int, ir.Block] = {}  # block_id -> LLVM block
        
        # PHI deferrals for sealed block approach: (block_id, dst_vid, incoming)
        self.phi_deferrals: List[Tuple[int, int, List[Tuple[int, int]]]] = []
        # Predecessor map and per-block end snapshots
        self.preds: Dict[int, List[int]] = {}
        self.block_end_values: Dict[int, Dict[int, ir.Value]] = {}
        # Definition map: value_id -> set(block_id) where the value is defined
        # Used as a lightweight lifetime hint to avoid over-localization
        self.def_blocks: Dict[int, set] = {}
        
        # Resolver for unified value resolution
        self.resolver = Resolver(self.vmap, self.bb_map)
        
        # Statistics
        self.loop_count = 0
        # Heuristics for minor gated fixes
        self.current_function_name: Optional[str] = None
        self._last_substring_vid: Optional[int] = None
        # Map of (block_id, value_id) -> predeclared PHI for ret-merge if-merge prepass
        self.predeclared_ret_phis: Dict[Tuple[int, int], ir.Instruction] = {}
        
    def build_from_mir(self, mir_json: Dict[str, Any]) -> str:
        """Build LLVM IR from MIR JSON"""
        # Parse MIR
        reader = MIRReader(mir_json)
        functions = reader.get_functions()
        
        if not functions:
            # No functions - create dummy ny_main
            return self._create_dummy_main()
        
        # Pre-declare all functions with default i64 signature to allow cross-calls
        import re
        for func_data in functions:
            name = func_data.get("name", "unknown")
            # Derive arity from name suffix '/N' if params list is empty
            m = re.search(r"/(\d+)$", name)
            if m:
                arity = int(m.group(1))
            else:
                arity = len(func_data.get("params", []))
            if name == "ny_main":
                fty = ir.FunctionType(self.i32, [])
            else:
                fty = ir.FunctionType(self.i64, [self.i64] * arity)
            exists = False
            for f in self.module.functions:
                if f.name == name:
                    exists = True
                    break
            if not exists:
                ir.Function(self.module, fty, name=name)
        
        # Process each function (finalize PHIs per function to avoid cross-function map collisions)
        for func_data in functions:
            self.lower_function(func_data)

        # Create ny_main wrapper if necessary
        has_ny_main = any(f.name == 'ny_main' for f in self.module.functions)
        # Prefer static box entry: Main.main/1; fallback to plain main (0-arity)
        fn_main_box = None
        fn_main_plain = None
        for f in self.module.functions:
            if f.name == 'Main.main/1':
                fn_main_box = f
            elif f.name == 'main':
                fn_main_plain = f
        target_fn = fn_main_box or fn_main_plain
        if target_fn is not None and not has_ny_main:
            # Hide the target to avoid symbol conflicts
            try:
                target_fn.linkage = 'private'
            except Exception:
                pass
            # i32 ny_main() { return (i32) Main.main(args) | main(); }
            ny_main_ty = ir.FunctionType(self.i64, [])
            ny_main = ir.Function(self.module, ny_main_ty, name='ny_main')
            entry = ny_main.append_basic_block('entry')
            b = ir.IRBuilder(entry)
            if fn_main_box is not None:
                # Build default args = new ArrayBox() via nyash.env.box.new_i64x
                i64 = self.i64
                i8 = self.i8
                i8p = self.i8p
                # Declare callee
                callee = None
                for f in self.module.functions:
                    if f.name == 'nyash.env.box.new_i64x':
                        callee = f
                        break
                if callee is None:
                    callee = ir.Function(self.module, ir.FunctionType(i64, [i8p, i64, i64, i64, i64, i64]), name='nyash.env.box.new_i64x')
                # Create "ArrayBox\0" global
                sbytes = b"ArrayBox\0"
                arr_ty = ir.ArrayType(i8, len(sbytes))
                g = ir.GlobalVariable(self.module, arr_ty, name='.ny_main_arraybox')
                g.linkage = 'private'
                g.global_constant = True
                g.initializer = ir.Constant(arr_ty, bytearray(sbytes))
                c0 = ir.Constant(self.i32, 0)
                ptr = b.gep(g, [c0, c0], inbounds=True)
                zero = ir.Constant(i64, 0)
                args_handle = b.call(callee, [ptr, zero, zero, zero, zero, zero], name='ny_main_args')
                rv = b.call(fn_main_box, [args_handle], name='call_Main_main_1')
            else:
                # Plain main() fallback
                if len(fn_main_plain.args) == 0:
                    rv = b.call(fn_main_plain, [], name='call_user_main')
                else:
                    rv = ir.Constant(self.i64, 0)
            if hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width != 32:
                rv64 = b.trunc(rv, self.i64) if rv.type.width > 64 else b.zext(rv, self.i64)
                b.ret(rv64)
            elif hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width == 64:
                b.ret(rv)
            else:
                b.ret(ir.Constant(self.i64, 0))
        
        ir_text = str(self.module)
        # Optional IR dump to file for debugging
        try:
            dump_path = os.environ.get('NYASH_LLVM_DUMP_IR')
            if dump_path:
                os.makedirs(os.path.dirname(dump_path), exist_ok=True)
                with open(dump_path, 'w') as f:
                    f.write(ir_text)
            else:
                # Default dump location when verbose and not explicitly set
                if os.environ.get('NYASH_CLI_VERBOSE') == '1':
                    os.makedirs('tmp', exist_ok=True)
                    with open('tmp/nyash_harness.ll', 'w') as f:
                        f.write(ir_text)
        except Exception:
            pass
        return ir_text
    
    def _create_dummy_main(self) -> str:
        """Create dummy ny_main that returns 0"""
        ny_main_ty = ir.FunctionType(self.i64, [])
        ny_main = ir.Function(self.module, ny_main_ty, name="ny_main")
        block = ny_main.append_basic_block(name="entry")
        builder = ir.IRBuilder(block)
        builder.ret(ir.Constant(self.i32, 0))
        return str(self.module)
    
    def lower_function(self, func_data: Dict[str, Any]):
        """Lower a single MIR function to LLVM IR"""
        name = func_data.get("name", "unknown")
        self.current_function_name = name
        import re
        params = func_data.get("params", [])
        blocks = func_data.get("blocks", [])
        
        # Determine function signature
        if name == "ny_main":
            # Special case: ny_main returns i32
            func_ty = ir.FunctionType(self.i32, [])
        else:
            # Default: i64(i64, ...) signature; derive arity from '/N' suffix when params missing
            m = re.search(r"/(\d+)$", name)
            arity = int(m.group(1)) if m else len(params)
            param_types = [self.i64] * arity
            func_ty = ir.FunctionType(self.i64, param_types)
        
        # Reset per-function maps and resolver caches to avoid cross-function collisions
        try:
            self.vmap.clear()
        except Exception:
            self.vmap = {}
        # Reset basic-block map per function (block ids are local to function)
        try:
            self.bb_map.clear()
        except Exception:
            self.bb_map = {}
        # Reset resolver caches (they key by block name; avoid collisions across functions)
        try:
            self.resolver.i64_cache.clear()
            self.resolver.ptr_cache.clear()
            self.resolver.f64_cache.clear()
            if hasattr(self.resolver, '_end_i64_cache'):
                self.resolver._end_i64_cache.clear()
            if hasattr(self.resolver, 'string_ids'):
                self.resolver.string_ids.clear()
            if hasattr(self.resolver, 'string_literals'):
                self.resolver.string_literals.clear()
            if hasattr(self.resolver, 'string_ptrs'):
                self.resolver.string_ptrs.clear()
        except Exception:
            pass

        # Create or reuse function
        func = None
        for f in self.module.functions:
            if f.name == name:
                func = f
                break
        if func is None:
            func = ir.Function(self.module, func_ty, name=name)
        
        # Map parameters to vmap (value_id: 0..arity-1)
        try:
            arity = len(func.args)
            for i in range(arity):
                self.vmap[i] = func.args[i]
        except Exception:
            pass

        # Build predecessor map from control-flow edges
        self.preds = {}
        for block_data in blocks:
            bid = block_data.get("id", 0)
            self.preds.setdefault(bid, [])
        for block_data in blocks:
            src = block_data.get("id", 0)
            for inst in block_data.get("instructions", []):
                op = inst.get("op")
                if op == "jump":
                    t = inst.get("target")
                    if t is not None:
                        self.preds.setdefault(t, []).append(src)
                elif op == "branch":
                    th = inst.get("then")
                    el = inst.get("else")
                    if th is not None:
                        self.preds.setdefault(th, []).append(src)
                    if el is not None:
                        self.preds.setdefault(el, []).append(src)

        # Create all blocks first
        for block_data in blocks:
            bid = block_data.get("id", 0)
            block_name = f"bb{bid}"
            bb = func.append_basic_block(block_name)
            self.bb_map[bid] = bb
        
        # Build quick lookup for blocks by id
        block_by_id: Dict[int, Dict[str, Any]] = {}
        for block_data in blocks:
            block_by_id[block_data.get("id", 0)] = block_data

        # Determine entry block: first with no predecessors; fallback to first block
        entry_bid = None
        for bid, preds in self.preds.items():
            if len(preds) == 0:
                entry_bid = bid
                break
        if entry_bid is None and blocks:
            entry_bid = blocks[0].get("id", 0)

        # Compute a preds-first (approx topological) order
        visited = set()
        order: List[int] = []

        def visit(bid: int):
            if bid in visited:
                return
            visited.add(bid)
            for p in self.preds.get(bid, []):
                visit(p)
            order.append(bid)

        if entry_bid is not None:
            visit(entry_bid)
        # Include any blocks not reachable from entry
        for bid in block_by_id.keys():
            if bid not in visited:
                visit(bid)

        # Process blocks in the computed order
        # Prepass: collect producer stringish hints and PHI metadata for all blocks
        # and create placeholders at each block head so that resolver can safely
        # return existing PHIs without creating new ones.
        _setup_phi_placeholders(self, blocks)

        # Optional: if-merge prepass → predeclare PHI for return-merge blocks
        # Gate with NYASH_LLVM_PREPASS_IFMERGE=1
        try:
            if os.environ.get('NYASH_LLVM_PREPASS_IFMERGE') == '1':
                plan = plan_ret_phi_predeclare(block_by_id)
                if plan:
                    # Ensure block_phi_incomings map exists
                    if not hasattr(self, 'block_phi_incomings') or self.block_phi_incomings is None:
                        self.block_phi_incomings = {}
                    for bbid, ret_vid in plan.items():
                        # Create a placeholder PHI at block head if missing
                        bb0 = self.bb_map.get(bbid)
                        if bb0 is not None:
                            b0 = ir.IRBuilder(bb0)
                            try:
                                b0.position_at_start(bb0)
                            except Exception:
                                pass
                            cur = self.vmap.get(ret_vid)
                            need_new = True
                            try:
                                need_new = not (cur is not None and hasattr(cur, 'add_incoming'))
                            except Exception:
                                need_new = True
                            if need_new:
                                ph = b0.phi(self.i64, name=f"phi_ret_{ret_vid}")
                                self.vmap[ret_vid] = ph
                            else:
                                ph = cur
                            # Record for later unify
                            try:
                                self.predeclared_ret_phis[(int(bbid), int(ret_vid))] = ph
                            except Exception:
                                pass
                        # Record declared incoming metadata using the same value-id
                        # for each predecessor; finalize_phis will resolve per-pred end values.
                        try:
                            preds_raw = [p for p in self.preds.get(bbid, []) if p != bbid]
                        except Exception:
                            preds_raw = []
                        # Dedup while preserving order
                        seen = set()
                        preds_list = []
                        for p in preds_raw:
                            if p not in seen:
                                preds_list.append(p)
                                seen.add(p)
                        try:
                            # finalize_phis reads pairs as (decl_b, v_src) and maps to nearest predecessor.
                            # We provide (bb_pred, ret_vid) for all preds.
                            self.block_phi_incomings.setdefault(int(bbid), {})[int(ret_vid)] = [
                                (int(p), int(ret_vid)) for p in preds_list
                            ]
                        except Exception:
                            pass
                        try:
                            trace_debug(f"[prepass] if-merge: predeclare PHI at bb{bbid} for v{ret_vid} preds={preds_list}")
                        except Exception:
                            pass
        except Exception:
            pass

        # Predeclare PHIs for values used in a block but defined in predecessors (multi-pred only).
        # This keeps PHI nodes grouped at the top and avoids late synthesis during operand resolution.
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
            def _collect_uses(block):
                uses = set()
                for ins in block.get('instructions') or []:
                    # Minimal keys: lhs/rhs (binop), value (ret/copy), cond (branch), box_val (boxcall)
                    for k in ('lhs','rhs','value','cond','box_val'):
                        try:
                            v = ins.get(k)
                            if isinstance(v, int):
                                uses.add(int(v))
                        except Exception:
                            pass
                return uses
            # Ensure map for declared incomings exists
            if not hasattr(self, 'block_phi_incomings') or self.block_phi_incomings is None:
                self.block_phi_incomings = {}
            for bid, blk in block_by_id.items():
                # Only multi-pred blocks need PHIs
                try:
                    preds_raw = [p for p in local_preds.get(int(bid), []) if p != int(bid)]
                except Exception:
                    preds_raw = []
                # Dedup preds preserve order
                seen = set(); preds_list = []
                for p in preds_raw:
                    if p not in seen: preds_list.append(p); seen.add(p)
                if len(preds_list) <= 1:
                    continue
                defs = _collect_defs(blk)
                uses = _collect_uses(blk)
                need = [u for u in uses if u not in defs]
                if not need:
                    continue
                bb0 = self.bb_map.get(int(bid))
                if bb0 is None:
                    continue
                b0 = ir.IRBuilder(bb0)
                try:
                    b0.position_at_start(bb0)
                except Exception:
                    pass
                for vid in need:
                    # Skip if we already have a PHI mapped for (bid, vid)
                    cur = self.vmap.get(int(vid))
                    has_phi_here = False
                    try:
                        has_phi_here = (
                            cur is not None and hasattr(cur, 'add_incoming') and
                            getattr(getattr(cur, 'basic_block', None), 'name', None) == bb0.name
                        )
                    except Exception:
                        has_phi_here = False
                    if not has_phi_here:
                        ph = b0.phi(self.i64, name=f"phi_{vid}")
                        self.vmap[int(vid)] = ph
                    # Record incoming metadata for finalize_phis (pred -> same vid)
                    try:
                        self.block_phi_incomings.setdefault(int(bid), {}).setdefault(int(vid), [])
                        # Overwrite with dedup list of (pred, vid)
                        self.block_phi_incomings[int(bid)][int(vid)] = [(int(p), int(vid)) for p in preds_list]
                    except Exception:
                        pass
            # Expose to resolver
            try:
                self.resolver.block_phi_incomings = self.block_phi_incomings
            except Exception:
                pass
        except Exception:
            pass

        # Optional: simple loop prepass → synthesize a structured while body
        loop_plan = None
        try:
            if os.environ.get('NYASH_LLVM_PREPASS_LOOP') == '1':
                loop_plan = detect_simple_while(block_by_id)
                if loop_plan is not None:
                    trace_debug(f"[prepass] detect loop header=bb{loop_plan['header']} then=bb{loop_plan['then']} latch=bb{loop_plan['latch']} exit=bb{loop_plan['exit']}")
        except Exception:
            loop_plan = None

        # Provide predeclared ret-phi map to resolver for ret lowering to reuse
        try:
            self.resolver.ret_phi_map = self.predeclared_ret_phis
        except Exception:
            pass

        # Now lower blocks
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
                bb = self.bb_map[bid]
                builder = ir.IRBuilder(bb)
                try:
                    self.resolver.builder = builder
                    self.resolver.module = self.module
                except Exception:
                    pass
                # Lower while via loopform (if enabled) or regular fallback
                self.loop_count += 1
                body_insts = loop_plan.get('body_insts', [])
                cond_vid = loop_plan.get('cond')
                from instructions.loopform import lower_while_loopform
                ok = False
                try:
                    # Use a clean per-while vmap context seeded from global placeholders
                    self._current_vmap = dict(self.vmap)
                    ok = lower_while_loopform(
                        builder,
                        func,
                        cond_vid,
                        body_insts,
                        self.loop_count,
                        self.vmap,
                        self.bb_map,
                        self.resolver,
                        self.preds,
                        self.block_end_values,
                        getattr(self, 'ctx', None),
                    )
                except Exception:
                    ok = False
                if not ok:
                    # Prepare resolver backref for instruction dispatcher
                    try:
                        self.resolver._owner_lower_instruction = self.lower_instruction
                    except Exception:
                        pass
                    lower_while_regular(builder, func, cond_vid, body_insts,
                                        self.loop_count, self.vmap, self.bb_map,
                                        self.resolver, self.preds, self.block_end_values)
                # Clear while vmap context
                try:
                    delattr(self, '_current_vmap')
                except Exception:
                    pass
                # Mark blocks to skip
                for bskip in loop_plan.get('skip_blocks', []):
                    skipped.add(bskip)
                # Ensure skipped original blocks have a valid terminator: branch to while exit
                try:
                    exit_name = f"while{self.loop_count}_exit"
                    exit_bb = None
                    for bbf in func.blocks:
                        try:
                            if str(bbf.name) == exit_name:
                                exit_bb = bbf
                                break
                        except Exception:
                            pass
                    if exit_bb is not None:
                        # Connect while exit to original exit block if available
                        try:
                            orig_exit_bb = self.bb_map.get(loop_plan.get('exit'))
                            if orig_exit_bb is not None and exit_bb.terminator is None:
                                ibx = ir.IRBuilder(exit_bb)
                                ibx.branch(orig_exit_bb)
                        except Exception:
                            pass
                        for bskip in loop_plan.get('skip_blocks', []):
                            if bskip == loop_plan.get('header'):
                                continue
                            bb_skip = self.bb_map.get(bskip)
                            if bb_skip is None:
                                continue
                            try:
                                if bb_skip.terminator is None:
                                    ib = ir.IRBuilder(bb_skip)
                                    ib.branch(exit_bb)
                            except Exception:
                                pass
                except Exception:
                    pass
                continue
            if bid in skipped:
                continue
            bb = self.bb_map[bid]
            self.lower_block(bb, block_data, func)

        # Provide lifetime hints to resolver (which blocks define which values)
        try:
            self.resolver.def_blocks = self.def_blocks
            # Provide phi metadata for this function to resolver
            self.resolver.block_phi_incomings = getattr(self, 'block_phi_incomings', {})
            # Attach a BuildCtx object for future refactors (non-breaking)
            try:
                self.ctx = BuildCtx(
                    module=self.module,
                    i64=self.i64,
                    i32=self.i32,
                    i8=self.i8,
                    i1=self.i1,
                    i8p=self.i8p,
                    vmap=self.vmap,
                    bb_map=self.bb_map,
                    preds=self.preds,
                    block_end_values=self.block_end_values,
                    resolver=self.resolver,
                    trace_phi=os.environ.get('NYASH_LLVM_TRACE_PHI') == '1',
                    verbose=os.environ.get('NYASH_CLI_VERBOSE') == '1',
                )
                # Also expose via resolver for convenience until migration completes
                self.resolver.ctx = self.ctx
            except Exception:
                pass
        except Exception:
            pass
        # Finalize PHIs for this function now that all snapshots for it exist
        _finalize_phis(self)


    def setup_phi_placeholders(self, blocks: List[Dict[str, Any]]):
        """Predeclare PHIs and collect incoming metadata for finalize_phis.

        This pass is function-local and must be invoked after basic blocks are
        created and before lowering individual blocks. It also tags string-ish
        values eagerly to help downstream resolvers choose correct intrinsics.
        """
        try:
            # Pass A: collect producer stringish hints per value-id
            produced_str: Dict[int, bool] = {}
            for block_data in blocks:
                for inst in block_data.get("instructions", []) or []:
                    try:
                        opx = inst.get("op")
                        dstx = inst.get("dst")
                        if dstx is None:
                            continue
                        is_str = False
                        if opx == "const":
                            v = inst.get("value", {}) or {}
                            t = v.get("type")
                            if t == "string" or (isinstance(t, dict) and t.get("kind") in ("handle","ptr") and t.get("box_type") == "StringBox"):
                                is_str = True
                        elif opx in ("binop","boxcall","externcall"):
                            t = inst.get("dst_type")
                            if isinstance(t, dict) and t.get("kind") == "handle" and t.get("box_type") == "StringBox":
                                is_str = True
                        if is_str:
                            produced_str[int(dstx)] = True
                    except Exception:
                        pass
            # Pass B: materialize PHI placeholders and record incoming metadata
            self.block_phi_incomings = {}
            for block_data in blocks:
                bid0 = block_data.get("id", 0)
                bb0 = self.bb_map.get(bid0)
                for inst in block_data.get("instructions", []) or []:
                    if inst.get("op") == "phi":
                        try:
                            dst0 = int(inst.get("dst"))
                            incoming0 = inst.get("incoming", []) or []
                        except Exception:
                            dst0 = None; incoming0 = []
                        if dst0 is None:
                            continue
                        # Record incoming metadata for finalize_phis
                        try:
                            self.block_phi_incomings.setdefault(bid0, {})[dst0] = [
                                (int(b), int(v)) for (v, b) in incoming0
                            ]
                        except Exception:
                            pass
                        # Ensure placeholder exists at block head
                        if bb0 is not None:
                            b0 = ir.IRBuilder(bb0)
                            try:
                                b0.position_at_start(bb0)
                            except Exception:
                                pass
                            existing = self.vmap.get(dst0)
                            is_phi = False
                            try:
                                is_phi = hasattr(existing, 'add_incoming')
                            except Exception:
                                is_phi = False
                            if not is_phi:
                                ph0 = b0.phi(self.i64, name=f"phi_{dst0}")
                                self.vmap[dst0] = ph0
                            # Tag propagation: if explicit dst_type marks string or any incoming was produced as string-ish, tag dst
                            try:
                                dst_type0 = inst.get("dst_type")
                                mark_str = isinstance(dst_type0, dict) and dst_type0.get("kind") == "handle" and dst_type0.get("box_type") == "StringBox"
                                if not mark_str:
                                    for (v_id, _b_id) in incoming0:
                                        try:
                                            if produced_str.get(int(v_id)):
                                                mark_str = True; break
                                        except Exception:
                                            pass
                                if mark_str and hasattr(self.resolver, 'mark_string'):
                                    self.resolver.mark_string(int(dst0))
                            except Exception:
                                pass
                            # Definition hint: PHI defines dst in this block
                            try:
                                self.def_blocks.setdefault(int(dst0), set()).add(int(bid0))
                            except Exception:
                                pass
            # Sync to resolver
            try:
                self.resolver.block_phi_incomings = self.block_phi_incomings
            except Exception:
                pass
        except Exception:
            pass
    
    def lower_block(self, bb: ir.Block, block_data: Dict[str, Any], func: ir.Function):
        """Lower a single basic block.

        Emit all non-terminator ops first, then control-flow terminators
        (branch/jump/ret). This avoids generating IR after a terminator.
        """
        builder = ir.IRBuilder(bb)
        try:
            import os
            trace_debug(f"[llvm-py] === lower_block bb{block_data.get('id')} ===")
        except Exception:
            pass
        # Provide builder/module to resolver for PHI/casts insertion
        try:
            self.resolver.builder = builder
            self.resolver.module = self.module
        except Exception:
            pass
        instructions = block_data.get("instructions", [])
        # Ensure JSON-declared PHIs are materialized at block start before any terminator
        try:
            phi_insts = [inst for inst in (instructions or []) if inst.get('op') == 'phi']
            if phi_insts:
                btop = ir.IRBuilder(bb)
                btop.position_at_start(bb)
                for pinst in phi_insts:
                    dstp = pinst.get('dst')
                    if isinstance(dstp, int):
                        cur = self.vmap.get(dstp)
                        need_new = True
                        try:
                            need_new = not (cur is not None and hasattr(cur, 'add_incoming'))
                        except Exception:
                            need_new = True
                        if need_new:
                            phi = btop.phi(self.i64, name=f"phi_{dstp}")
                            self.vmap[dstp] = phi
        except Exception:
            pass
        # Partition into body ops and terminators
        body_ops: List[Dict[str, Any]] = []
        term_ops: List[Dict[str, Any]] = []
        for inst in (instructions or []):
            opx = inst.get("op")
            if opx in ("branch", "jump", "ret"):
                term_ops.append(inst)
            elif opx == "phi":
                continue
            else:
                body_ops.append(inst)
        # Per-block SSA map (avoid cross-block vmap pollution)
        # Seed with non-PHI globals and PHIs that belong to this block only.
        vmap_cur: Dict[int, ir.Value] = {}
        try:
            for _vid, _val in (self.vmap or {}).items():
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
            vmap_cur = dict(self.vmap)
        # Expose to lower_instruction users (e.g., while_ regular lowering)
        self._current_vmap = vmap_cur
        created_ids: List[int] = []
        # Compute ids defined in this block to help with copy/PHI decisions
        defined_here_all: set = set()
        for _inst in body_ops:
            try:
                d = _inst.get('dst')
                if isinstance(d, int):
                    defined_here_all.add(d)
            except Exception:
                pass
        # Keep PHI synthesis on-demand in resolver; avoid predeclaring here to reduce clashes.
        # Lower body ops first in-order
        for i_idx, inst in enumerate(body_ops):
            try:
                import os
                trace_debug(f"[llvm-py] body op: {inst.get('op')} dst={inst.get('dst')} cond={inst.get('cond')}")
            except Exception:
                pass
            try:
                if bb.terminator is not None:
                    break
            except Exception:
                pass
            builder.position_at_end(bb)
            # Special-case copy: avoid forward self-block dependencies only when src is defined later in this block
            if inst.get('op') == 'copy':
                src_i = inst.get('src')
                skip_now = False
                if isinstance(src_i, int):
                    try:
                        # Check if src will be defined in a subsequent instruction
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
                    # Skip now; a later copy will remap after src becomes available
                    pass
                else:
                    self.lower_instruction(builder, inst, func)
            else:
                self.lower_instruction(builder, inst, func)
            # Sync per-block vmap snapshot with any new definitions that were
            # written into the global vmap by lowering routines (e.g., copy)
            try:
                dst = inst.get("dst")
                if isinstance(dst, int):
                    if dst in self.vmap:
                        _gval = self.vmap[dst]
                        # Avoid syncing PHIs that belong to other blocks (placeholders)
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
        # Ret-phi proactive insertion removed; resolver handles ret localization as needed.

        # Lower terminators at end, preserving order
        for inst in term_ops:
            try:
                import os
                trace_debug(f"[llvm-py] term op: {inst.get('op')} dst={inst.get('dst')} cond={inst.get('cond')}")
            except Exception:
                pass
            try:
                if bb.terminator is not None:
                    break
            except Exception:
                pass
            builder.position_at_end(bb)
            # (if-merge handled by resolver + finalize_phis)
            self.lower_instruction(builder, inst, func)
        # Sync back local PHIs created in this block into the global vmap so that
        # finalize_phis targets the same SSA nodes as terminators just used.
        try:
            for vid in created_ids:
                val = vmap_cur.get(vid)
                if val is not None and hasattr(val, 'add_incoming'):
                    try:
                        if getattr(getattr(val, 'basic_block', None), 'name', None) == bb.name:
                            self.vmap[vid] = val
                    except Exception:
                        self.vmap[vid] = val
        except Exception:
            pass
        # Snapshot end-of-block values for sealed PHI wiring
        bid = block_data.get("id", 0)
        # Robust snapshot: clone the entire vmap at block end so that
        # values that were not redefined in this block (but remain live)
        # are available to PHI finalize wiring. This avoids omissions of
        # phi-dst/cyclic and carry-over values.
        snap: Dict[int, ir.Value] = dict(vmap_cur)
        try:
            import os
            keys = sorted(list(snap.keys()))
            # Emit structured snapshot event for up to first 20 keys
            try:
                trace_phi_json({"phi": "snapshot", "block": int(bid), "keys": [int(k) for k in keys[:20]]})
            except Exception:
                pass
        except Exception:
            pass
        # Record block-local definitions for lifetime hinting
        for vid in created_ids:
            if vid in vmap_cur:
                self.def_blocks.setdefault(vid, set()).add(block_data.get("id", 0))
        self.block_end_values[bid] = snap
        # Clear current vmap context
        try:
            delattr(self, '_current_vmap')
        except Exception:
            pass
    
    def lower_instruction(self, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
        """Dispatch instruction to appropriate handler"""
        op = inst.get("op")
        # Pick current vmap context
        vmap_ctx = getattr(self, '_current_vmap', self.vmap)
        
        if op == "const":
            dst = inst.get("dst")
            value = inst.get("value")
            lower_const(builder, self.module, dst, value, vmap_ctx, self.resolver)
            
        elif op == "binop":
            operation = inst.get("operation")
            lhs = inst.get("lhs")
            rhs = inst.get("rhs")
            dst = inst.get("dst")
            dst_type = inst.get("dst_type")
            lower_binop(builder, self.resolver, operation, lhs, rhs, dst,
                        vmap_ctx, builder.block, self.preds, self.block_end_values, self.bb_map,
                        dst_type=dst_type)
            
        elif op == "jump":
            target = inst.get("target")
            lower_jump(builder, target, self.bb_map)
            
        elif op == "copy":
            dst = inst.get("dst")
            src = inst.get("src")
            lower_copy(builder, dst, src, vmap_ctx, self.resolver, builder.block, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            
        elif op == "branch":
            cond = inst.get("cond")
            then_bid = inst.get("then")
            else_bid = inst.get("else")
            lower_branch(builder, cond, then_bid, else_bid, vmap_ctx, self.bb_map, self.resolver, self.preds, self.block_end_values)
            
        elif op == "ret":
            value = inst.get("value")
            lower_return(builder, value, vmap_ctx, func.function_type.return_type,
                         self.resolver, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            
        elif op == "phi":
            # No-op here: PHIはメタのみ（resolverがon‑demand生成）
            return
            
        elif op == "compare":
            # Dedicated compare op
            operation = inst.get("operation") or inst.get("op")
            lhs = inst.get("lhs")
            rhs = inst.get("rhs")
            dst = inst.get("dst")
            cmp_kind = inst.get("cmp_kind")
            lower_compare(builder, operation, lhs, rhs, dst, vmap_ctx,
                          self.resolver, builder.block, self.preds, self.block_end_values, self.bb_map,
                          meta={"cmp_kind": cmp_kind} if cmp_kind else None,
                          ctx=getattr(self, 'ctx', None))
            
        elif op == "call":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_call(builder, self.module, func_name, args, dst, vmap_ctx, self.resolver, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            
        elif op == "boxcall":
            box_vid = inst.get("box")
            method = inst.get("method")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_boxcall(builder, self.module, box_vid, method, args, dst,
                          vmap_ctx, self.resolver, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            # Optional: honor explicit dst_type for tagging (string handle)
            try:
                dst_type = inst.get("dst_type")
                if dst is not None and isinstance(dst_type, dict):
                    if dst_type.get("kind") == "handle" and dst_type.get("box_type") == "StringBox":
                        if hasattr(self.resolver, 'mark_string'):
                            self.resolver.mark_string(int(dst))
                # Track last substring for optional esc_json fallback
                try:
                    if isinstance(method, str) and method == 'substring' and isinstance(dst, int):
                        self._last_substring_vid = int(dst)
                except Exception:
                    pass
            except Exception:
                pass
            
        elif op == "externcall":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_externcall(builder, self.module, func_name, args, dst,
                             vmap_ctx, self.resolver, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            
        elif op == "newbox":
            box_type = inst.get("type")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_newbox(builder, self.module, box_type, args, dst, 
                        vmap_ctx, self.resolver, getattr(self, 'ctx', None))
            
        elif op == "typeop":
            operation = inst.get("operation")
            src = inst.get("src")
            dst = inst.get("dst")
            target_type = inst.get("target_type")
            lower_typeop(builder, operation, src, dst, target_type,
                         vmap_ctx, self.resolver, self.preds, self.block_end_values, self.bb_map, getattr(self, 'ctx', None))
            
        elif op == "safepoint":
            live = inst.get("live", [])
            lower_safepoint(builder, self.module, live, vmap_ctx,
                            resolver=self.resolver, preds=self.preds,
                            block_end_values=self.block_end_values, bb_map=self.bb_map,
                            ctx=getattr(self, 'ctx', None))
            
        elif op == "barrier":
            barrier_type = inst.get("type", "memory")
            lower_barrier(builder, barrier_type, ctx=getattr(self, 'ctx', None))
            
        elif op == "while":
            # Experimental LoopForm lowering
            cond = inst.get("cond")
            body = inst.get("body", [])
            self.loop_count += 1
            if not lower_while_loopform(builder, func, cond, body,
                                       self.loop_count, self.vmap, self.bb_map,
                                       self.resolver, self.preds, self.block_end_values,
                                       getattr(self, 'ctx', None)):
                # Fallback to regular while (structured)
                try:
                    self.resolver._owner_lower_instruction = self.lower_instruction
                except Exception:
                    pass
                lower_while_regular(builder, func, cond, body,
                                    self.loop_count, self.vmap, self.bb_map,
                                    self.resolver, self.preds, self.block_end_values)
        else:
            trace_debug(f"[Python LLVM] Unknown instruction: {op}")
        # Record per-inst definition for lifetime hinting as soon as available
        try:
            dst_maybe = inst.get("dst")
            if isinstance(dst_maybe, int) and dst_maybe in self.vmap:
                cur_bid = None
                try:
                    cur_bid = int(str(builder.block.name).replace('bb',''))
                except Exception:
                    pass
                if cur_bid is not None:
                    self.def_blocks.setdefault(dst_maybe, set()).add(cur_bid)
        except Exception:
            pass
    
    # NOTE: regular while lowering is implemented in
    # instructions/controlflow/while_.py::lower_while_regular and invoked
    # from NyashLLVMBuilder.lower_instruction(). This legacy helper is removed
    # to avoid divergence between two implementations.

    def _lower_instruction_list(self, builder: ir.IRBuilder, insts: List[Dict[str, Any]], func: ir.Function):
        """Lower a flat list of instructions using current builder and function."""
        for sub in insts:
            # If current block already has a terminator, create a continuation block
            if builder.block.terminator is not None:
                cont = func.append_basic_block(name=f"cont_bb_{builder.block.name}")
                builder.position_at_end(cont)
            self.lower_instruction(builder, sub, func)
    
    def finalize_phis(self):
        """Finalize PHIs declared in JSON by wiring incoming edges at block heads.
        Uses resolver._value_at_end_i64 to materialize values at predecessor ends,
        ensuring casts/boxing are inserted in predecessor blocks (dominance-safe)."""
        # Iterate JSON-declared PHIs per block
        # Build succ map for nearest-predecessor mapping
        succs: Dict[int, List[int]] = {}
        for to_bid, from_list in (self.preds or {}).items():
            for fr in from_list:
                succs.setdefault(fr, []).append(to_bid)
        for block_id, dst_map in (getattr(self, 'block_phi_incomings', {}) or {}).items():
            try:
                trace_phi_json({"phi": "finalize_begin", "block": int(block_id), "dsts": [int(k) for k in (dst_map or {}).keys()]})
            except Exception:
                pass
            bb = self.bb_map.get(block_id)
            if bb is None:
                continue
            b = ir.IRBuilder(bb)
            try:
                b.position_at_start(bb)
            except Exception:
                pass
            for dst_vid, incoming in (dst_map or {}).items():
                try:
                    trace_phi_json({"phi": "finalize_dst", "block": int(block_id), "dst": int(dst_vid), "incoming": [(int(v), int(b)) for (b, v) in [(b, v) for (v, b) in (incoming or [])]]})
                except Exception:
                    pass
                # Ensure placeholder exists at block head
                # Prefer predeclared ret-phi when available and force using it.
                predecl = getattr(self, 'predeclared_ret_phis', {}) if hasattr(self, 'predeclared_ret_phis') else {}
                phi = predecl.get((int(block_id), int(dst_vid))) if predecl else None
                if phi is not None:
                    # Bind as canonical target
                    self.vmap[dst_vid] = phi
                else:
                    phi = self.vmap.get(dst_vid)
                    # Ensure we target a PHI belonging to the current block; if a
                    # global mapping points to a PHI in another block (due to
                    # earlier localization), create/replace with a local PHI.
                    need_local_phi = False
                    try:
                        if not (phi is not None and hasattr(phi, 'add_incoming')):
                            need_local_phi = True
                        else:
                            bb_of_phi = getattr(getattr(phi, 'basic_block', None), 'name', None)
                            if bb_of_phi != bb.name:
                                need_local_phi = True
                    except Exception:
                        need_local_phi = True
                    if need_local_phi:
                        phi = b.phi(self.i64, name=f"phi_{dst_vid}")
                        self.vmap[dst_vid] = phi
                n = getattr(phi, 'name', b'').decode() if hasattr(getattr(phi, 'name', None), 'decode') else str(getattr(phi, 'name', ''))
                try:
                    trace_phi_json({"phi": "finalize_target", "block": int(block_id), "dst": int(dst_vid), "ir": str(n)})
                except Exception:
                    pass
                # Wire incoming per CFG predecessor; map src_vid when provided
                preds_raw = [p for p in self.preds.get(block_id, []) if p != block_id]
                # Deduplicate while preserving order
                seen = set()
                preds_list: List[int] = []
                for p in preds_raw:
                    if p not in seen:
                        preds_list.append(p)
                        seen.add(p)
                # Helper: find the nearest immediate predecessor on a path decl_b -> ... -> block_id
                def nearest_pred_on_path(decl_b: int) -> Optional[int]:
                    # BFS from decl_b to block_id; return the parent of block_id on that path.
                    from collections import deque
                    q = deque([decl_b])
                    visited = set([decl_b])
                    parent: Dict[int, Optional[int]] = {decl_b: None}
                    while q:
                        cur = q.popleft()
                        if cur == block_id:
                            par = parent.get(block_id)
                            return par if par in preds_list else None
                        for nx in succs.get(cur, []):
                            if nx not in visited:
                                visited.add(nx)
                                parent[nx] = cur
                                q.append(nx)
                    return None
                # Precompute a non-self initial source (if present) to use for self-carry cases
                init_src_vid: Optional[int] = None
                for (b_decl0, v_src0) in incoming:
                    try:
                        vs0 = int(v_src0)
                    except Exception:
                        continue
                    if vs0 != int(dst_vid):
                        init_src_vid = vs0
                        break
                # Pre-resolve declared incomings to nearest immediate predecessors
                chosen: Dict[int, ir.Value] = {}
                for (b_decl, v_src) in incoming:
                    try:
                        bd = int(b_decl); vs = int(v_src)
                    except Exception:
                        continue
                    pred_match = nearest_pred_on_path(bd)
                    if pred_match is None:
                        continue
                    # If self-carry is specified (vs == dst_vid), map to init_src_vid when available
                    if vs == int(dst_vid) and init_src_vid is not None:
                        vs = int(init_src_vid)
                    try:
                        val = self.resolver._value_at_end_i64(vs, pred_match, self.preds, self.block_end_values, self.vmap, self.bb_map)
                    except Exception:
                        val = None
                    if val is None:
                        val = ir.Constant(self.i64, 0)
                    chosen[pred_match] = val
                # Fill remaining predecessors with dst carry or (optionally) a synthesized default
                for pred_bid in preds_list:
                    if pred_bid not in chosen:
                        val = None
                        # Optional gated fix for esc_json: default branch should append current char
                        try:
                            import os
                            if os.environ.get('NYASH_LLVM_ESC_JSON_FIX','0') == '1':
                                fname = getattr(self, 'current_function_name', '') or ''
                                sub_vid = getattr(self, '_last_substring_vid', None)
                                if isinstance(fname, str) and 'esc_json' in fname and isinstance(sub_vid, int):
                                    # Compute out_at_end and ch_at_end in pred block, then concat_hh
                                    out_end = self.resolver._value_at_end_i64(int(dst_vid), pred_bid, self.preds, self.block_end_values, self.vmap, self.bb_map)
                                    ch_end = self.resolver._value_at_end_i64(int(sub_vid), pred_bid, self.preds, self.block_end_values, self.vmap, self.bb_map)
                                    if out_end is not None and ch_end is not None:
                                        pb = ir.IRBuilder(self.bb_map.get(pred_bid))
                                        try:
                                            t = self.bb_map.get(pred_bid).terminator
                                            if t is not None:
                                                pb.position_before(t)
                                            else:
                                                pb.position_at_end(self.bb_map.get(pred_bid))
                                        except Exception:
                                            pass
                                        fnty = ir.FunctionType(self.i64, [self.i64, self.i64])
                                        callee = None
                                        for f in self.module.functions:
                                            if f.name == 'nyash.string.concat_hh':
                                                callee = f; break
                                        if callee is None:
                                            callee = ir.Function(self.module, fnty, name='nyash.string.concat_hh')
                                        val = pb.call(callee, [out_end, ch_end], name=f"phi_def_concat_{dst_vid}_{pred_bid}")
                        except Exception:
                            pass
                        if val is None:
                            try:
                                val = self.resolver._value_at_end_i64(dst_vid, pred_bid, self.preds, self.block_end_values, self.vmap, self.bb_map)
                            except Exception:
                                val = None
                        if val is None:
                            val = ir.Constant(self.i64, 0)
                        chosen[pred_bid] = val
                # Finally add incomings (each predecessor at most once)
                for pred_bid, val in chosen.items():
                    pred_bb = self.bb_map.get(pred_bid)
                    if pred_bb is None:
                        continue
                    phi.add_incoming(val, pred_bb)
                    try:
                        trace_phi(f"[finalize]   add incoming: bb{pred_bid} -> v{dst_vid}")
                    except Exception:
                        pass
                # Tag dst as string-ish if any declared source was string-ish (post-lowering info)
                try:
                    if hasattr(self.resolver, 'is_stringish') and hasattr(self.resolver, 'mark_string'):
                        any_str = False
                        for (_b_decl_i, v_src_i) in incoming:
                            try:
                                if self.resolver.is_stringish(int(v_src_i)):
                                    any_str = True; break
                            except Exception:
                                pass
                        if any_str:
                            self.resolver.mark_string(int(dst_vid))
                except Exception:
                    pass
        # Clear legacy deferrals if any
        try:
            self.phi_deferrals.clear()
        except Exception:
            pass
    
    def compile_to_object(self, output_path: str):
        """Compile module to object file"""
        # Create target machine
        target = llvm.Target.from_default_triple()
        target_machine = target.create_target_machine()
        
        # Compile
        mod = llvm.parse_assembly(str(self.module))
        # Allow skipping verifier for iterative bring-up
        if os.environ.get('NYASH_LLVM_SKIP_VERIFY') != '1':
            mod.verify()
        
        # Generate object code
        obj = target_machine.emit_object(mod)
        
        # Write to file
        with open(output_path, 'wb') as f:
            f.write(obj)

def main():
    # CLI:
    #   llvm_builder.py <input.mir.json> [-o output.o]
    #   llvm_builder.py --dummy [-o output.o]
    output_file = "nyash_llvm_py.o"
    args = sys.argv[1:]
    dummy = False

    if not args:
        print("Usage: llvm_builder.py <input.mir.json> [-o output.o] | --dummy [-o output.o]")
        sys.exit(1)

    if "-o" in args:
        idx = args.index("-o")
        if idx + 1 < len(args):
            output_file = args[idx + 1]
            del args[idx:idx+2]

    if args and args[0] == "--dummy":
        dummy = True
        del args[0]

    builder = NyashLLVMBuilder()

    if dummy:
        # Emit dummy ny_main
        ir_text = builder._create_dummy_main()
        trace_debug(f"[Python LLVM] Generated dummy IR:\n{ir_text}")
        builder.compile_to_object(output_file)
        print(f"Compiled to {output_file}")
        return

    if not args:
        print("error: missing input MIR JSON (or use --dummy)", file=sys.stderr)
        sys.exit(2)

    input_file = args[0]
    with open(input_file, 'r') as f:
        mir_json = json.load(f)

    llvm_ir = builder.build_from_mir(mir_json)
    trace_debug("[Python LLVM] Generated LLVM IR (see NYASH_LLVM_DUMP_IR or tmp/nyash_harness.ll)")

    builder.compile_to_object(output_file)
    print(f"Compiled to {output_file}")

if __name__ == "__main__":
    main()
