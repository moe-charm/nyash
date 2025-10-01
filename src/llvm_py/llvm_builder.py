#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/development/design/legacy/LLVM_LAYER_OVERVIEW.md
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
from phi_wiring import ensure_phi as _ensure_phi
from trace import debug as trace_debug
from trace import phi as trace_phi
from trace import phi_json as trace_phi_json
from prepass.loops import detect_simple_while
from prepass.if_merge import plan_ret_phi_predeclare
from build_ctx import BuildCtx

from resolver import Resolver
from mir_reader import MIRReader
from targets import create_target

class NyashLLVMBuilder:
    """Main LLVM IR builder for Nyash MIR"""

    def __init__(self, target="native"):
        """
        Initialize LLVM IR builder

        Args:
            target: Target architecture ("native" or "wasm32")
                    - "native": Native platform (default)
                    - "wasm32": WebAssembly (wasm32-unknown-wasi)
        """
        # Initialize LLVM
        llvm.initialize()

        # Create target object (箱理論: ターゲット抽象化)
        self.target_obj = create_target(target)
        self.target = target  # Keep for backward compatibility
        self.target_triple = self.target_obj.get_triple()

        # Initialize target-specific components
        if target == "wasm32":
            # WASM target: initialize all targets to enable wasm32
            llvm.initialize_all_targets()
            llvm.initialize_all_asmprinters()
        else:
            # Native target (default)
            llvm.initialize_native_target()
            llvm.initialize_native_asmprinter()

        # Module and basic types
        self.module = ir.Module(name="nyash_module")
        self.module.triple = self.target_triple  # Set target triple on module
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
            # Derive arity:
            # - For method-like names (Class.method/N), include implicit 'me' by using len(params)
            # - Otherwise, prefer suffix '/N' when present; fallback to params length
            m = re.search(r"/(\d+)$", name)
            params_list = func_data.get("params", []) or []
            if "." in name:
                arity = len(params_list)
            else:
                arity = int(m.group(1)) if m else len(params_list)
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
                func = ir.Function(self.module, fty, name=name)
                # Configure function for target (箱理論: ターゲット固有設定)
                self.target_obj.configure_function(func)
        
        # Process each function (finalize PHIs per function to avoid cross-function map collisions)
        for func_data in functions:
            self.lower_function(func_data)

        # Create ny_main wrapper if necessary (delegated builder; no legacy fallback)
        try:
            from builders.entry import ensure_ny_main as _ensure_ny_main
            _ensure_ny_main(self)
        except Exception as _e:
            try:
                trace_debug(f"[Python LLVM] ensure_ny_main failed: {_e}")
            except Exception:
                pass
        
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
        """Lower a single MIR function to LLVM IR (delegated, no legacy fallback)."""
        try:
            from builders.function_lower import lower_function as _lower
            return _lower(self, func_data)
        except Exception as _e:
            try:
                trace_debug(f"[Python LLVM] lower_function failed: {_e}")
            except Exception:
                pass
            raise


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
        # JSON-declared PHIs are not materialized here; placeholders are created uniformly
        # via ensure_phi in finalize_phis to keep PHIs grouped at block head.
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
        from builders.instruction_lower import lower_instruction as _li
        return _li(self, builder, inst, func)
    
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
            for dst_vid, incoming in (dst_map or {}).items():
                try:
                    trace_phi_json({"phi": "finalize_dst", "block": int(block_id), "dst": int(dst_vid), "incoming": [(int(v), int(b)) for (b, v) in [(b, v) for (v, b) in (incoming or [])]]})
                except Exception:
                    pass
                # Ensure placeholder exists at block head with common helper
                phi = _ensure_phi(self, int(block_id), int(dst_vid), bb)
                self.vmap[int(dst_vid)] = phi
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
        # TODO(Phase 2): Delegate to self.target_obj.emit_object()
        # Current implementation uses manual target_machine for PHI sanitize logic

        # Create target machine for the specified target triple
        target = llvm.Target.from_triple(self.target_triple)
        target_machine = target.create_target_machine()

        # Compile
        ir_text = str(self.module)

        # Debug: Always dump IR for debugging
        try:
            with open('/tmp/debug_ir.ll', 'w') as f:
                f.write(ir_text)
            print(f"[DEBUG] IR dumped to /tmp/debug_ir.ll")
        except Exception as e:
            print(f"[DEBUG] Failed to dump IR: {e}")
        # Optional sanitize: drop any empty PHI rows (no incoming list) to satisfy IR parser.
        # Gate with NYASH_LLVM_SANITIZE_EMPTY_PHI=1. Additionally, auto-enable when harness is requested.
        if os.environ.get('NYASH_LLVM_SANITIZE_EMPTY_PHI') == '1' or os.environ.get('NYASH_LLVM_USE_HARNESS') == '1':
            try:
                fixed_lines = []
                for line in ir_text.splitlines():
                    if (" = phi  i64" in line or " = phi i64" in line) and ("[" not in line):
                        # Skip malformed PHI without incoming pairs
                        continue
                    fixed_lines.append(line)
                ir_text = "\n".join(fixed_lines)
            except Exception:
                pass
        mod = llvm.parse_assembly(ir_text)
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
    #   llvm_builder.py <input.mir.json> [-o output.o] [--target wasm32|native]
    #   llvm_builder.py --dummy [-o output.o] [--target wasm32|native]
    output_file = os.path.join('tmp', 'nyash_llvm_py.o')
    args = sys.argv[1:]
    dummy = False
    target = "native"  # Default target

    if not args:
        print("Usage: llvm_builder.py <input.mir.json> [-o output.o] [--target wasm32|native] | --dummy [-o output.o] [--target wasm32|native]")
        sys.exit(1)

    # Parse --target option
    if "--target" in args:
        idx = args.index("--target")
        if idx + 1 < len(args):
            target = args[idx + 1]
            if target not in ("wasm32", "native"):
                print(f"error: invalid target '{target}', must be 'wasm32' or 'native'", file=sys.stderr)
                sys.exit(1)
            del args[idx:idx+2]

    # Parse -o option
    if "-o" in args:
        idx = args.index("-o")
        if idx + 1 < len(args):
            output_file = args[idx + 1]
            del args[idx:idx+2]

    # Parse --dummy option
    if args and args[0] == "--dummy":
        dummy = True
        del args[0]

    # Create builder with specified target
    builder = NyashLLVMBuilder(target=target)

    if dummy:
        # Emit dummy ny_main
        ir_text = builder._create_dummy_main()
        trace_debug(f"[Python LLVM] Generated dummy IR:\n{ir_text}")
        try:
            os.makedirs(os.path.dirname(output_file), exist_ok=True)
        except Exception:
            pass
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

    try:
        os.makedirs(os.path.dirname(output_file), exist_ok=True)
    except Exception:
        pass
    builder.compile_to_object(output_file)
    print(f"Compiled to {output_file}")

if __name__ == "__main__":
    main()
