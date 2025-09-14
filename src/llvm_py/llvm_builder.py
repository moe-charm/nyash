#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/LLVM_LAYER_OVERVIEW.md
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
from instructions.jump import lower_jump
from instructions.branch import lower_branch
from instructions.ret import lower_return
# PHI are deferred; finalize_phis wires incoming edges after snapshots
from instructions.call import lower_call
from instructions.boxcall import lower_boxcall
from instructions.externcall import lower_externcall
from instructions.typeop import lower_typeop, lower_convert
from instructions.newbox import lower_newbox
from instructions.safepoint import lower_safepoint, insert_automatic_safepoint
from instructions.barrier import lower_barrier
from instructions.loopform import lower_while_loopform

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
        main_fn = None
        for f in self.module.functions:
            if f.name == 'main':
                main_fn = f
                break
        if main_fn is not None:
            # Hide the user main to avoid conflict with NyRT's main symbol
            try:
                main_fn.linkage = 'private'
            except Exception:
                pass
            if not has_ny_main:
                # i32 ny_main() { return (i32) main(); }
                ny_main_ty = ir.FunctionType(self.i32, [])
                ny_main = ir.Function(self.module, ny_main_ty, name='ny_main')
                entry = ny_main.append_basic_block('entry')
                b = ir.IRBuilder(entry)
                if len(main_fn.args) == 0:
                    rv = b.call(main_fn, [], name='call_user_main')
                else:
                    # If signature mismatches, return 0
                    rv = ir.Constant(self.i64, 0)
                if hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width != 32:
                    rv32 = b.trunc(rv, self.i32) if rv.type.width > 32 else b.zext(rv, self.i32)
                    b.ret(rv32)
                elif hasattr(rv, 'type') and isinstance(rv.type, ir.IntType) and rv.type.width == 32:
                    b.ret(rv)
                else:
                    b.ret(ir.Constant(self.i32, 0))
        
        ir_text = str(self.module)
        # Optional IR dump to file for debugging
        try:
            dump_path = os.environ.get('NYASH_LLVM_DUMP_IR')
            if dump_path:
                os.makedirs(os.path.dirname(dump_path), exist_ok=True)
                with open(dump_path, 'w') as f:
                    f.write(ir_text)
            elif os.environ.get('NYASH_CLI_VERBOSE') == '1':
                # Default dump location when verbose and not explicitly set
                os.makedirs('tmp', exist_ok=True)
                with open('tmp/nyash_harness.ll', 'w') as f:
                    f.write(ir_text)
        except Exception:
            pass
        return ir_text
    
    def _create_dummy_main(self) -> str:
        """Create dummy ny_main that returns 0"""
        ny_main_ty = ir.FunctionType(self.i32, [])
        ny_main = ir.Function(self.module, ny_main_ty, name="ny_main")
        block = ny_main.append_basic_block(name="entry")
        builder = ir.IRBuilder(block)
        builder.ret(ir.Constant(self.i32, 0))
        return str(self.module)
    
    def lower_function(self, func_data: Dict[str, Any]):
        """Lower a single MIR function to LLVM IR"""
        name = func_data.get("name", "unknown")
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
        
        # Now lower blocks
        for bid in order:
            block_data = block_by_id.get(bid)
            if block_data is None:
                continue
            bb = self.bb_map[bid]
            self.lower_block(bb, block_data, func)

        # Provide lifetime hints to resolver (which blocks define which values)
        try:
            self.resolver.def_blocks = self.def_blocks
            # Provide phi metadata for this function to resolver
            self.resolver.block_phi_incomings = getattr(self, 'block_phi_incomings', {})
        except Exception:
            pass
        # Finalize PHIs for this function now that all snapshots for it exist
        self.finalize_phis()
    
    def lower_block(self, bb: ir.Block, block_data: Dict[str, Any], func: ir.Function):
        """Lower a single basic block"""
        builder = ir.IRBuilder(bb)
        # Provide builder/module to resolver for PHI/casts insertion
        try:
            self.resolver.builder = builder
            self.resolver.module = self.module
        except Exception:
            pass
        instructions = block_data.get("instructions", [])
        # Lower non-PHI instructions strictly in original program order.
        # Reordering here can easily introduce use-before-def within the same
        # basic block (e.g., string ops that depend on prior me.* calls).
        created_ids: List[int] = []
        non_phi_insts = [inst for inst in instructions if inst.get("op") != "phi"]
        for inst in non_phi_insts:
            # Stop if a terminator has already been emitted for this block
            try:
                if bb.terminator is not None:
                    break
            except Exception:
                pass
            builder.position_at_end(bb)
            self.lower_instruction(builder, inst, func)
            try:
                dst = inst.get("dst")
                if isinstance(dst, int) and dst not in created_ids and dst in self.vmap:
                    created_ids.append(dst)
            except Exception:
                pass
        # Snapshot end-of-block values for sealed PHI wiring
        bid = block_data.get("id", 0)
        # Robust snapshot: clone the entire vmap at block end so that
        # values that were not redefined in this block (but remain live)
        # are available to PHI finalize wiring. This avoids omissions of
        # phi-dst/cyclic and carry-over values.
        snap: Dict[int, ir.Value] = dict(self.vmap)
        try:
            import os
            if os.environ.get('NYASH_LLVM_TRACE_PHI') == '1':
                keys = sorted(list(snap.keys()))
                print(f"[builder] snapshot bb{bid} keys={keys[:20]}...", flush=True)
        except Exception:
            pass
        # Record block-local definitions for lifetime hinting
        for vid in created_ids:
            if vid in self.vmap:
                self.def_blocks.setdefault(vid, set()).add(block_data.get("id", 0))
        self.block_end_values[bid] = snap
    
    def lower_instruction(self, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
        """Dispatch instruction to appropriate handler"""
        op = inst.get("op")
        
        if op == "const":
            dst = inst.get("dst")
            value = inst.get("value")
            lower_const(builder, self.module, dst, value, self.vmap, self.resolver)
            
        elif op == "binop":
            operation = inst.get("operation")
            lhs = inst.get("lhs")
            rhs = inst.get("rhs")
            dst = inst.get("dst")
            dst_type = inst.get("dst_type")
            lower_binop(builder, self.resolver, operation, lhs, rhs, dst,
                        self.vmap, builder.block, self.preds, self.block_end_values, self.bb_map,
                        dst_type=dst_type)
            
        elif op == "jump":
            target = inst.get("target")
            lower_jump(builder, target, self.bb_map)
            
        elif op == "branch":
            cond = inst.get("cond")
            then_bid = inst.get("then")
            else_bid = inst.get("else")
            lower_branch(builder, cond, then_bid, else_bid, self.vmap, self.bb_map, self.resolver, self.preds, self.block_end_values)
            
        elif op == "ret":
            value = inst.get("value")
            lower_return(builder, value, self.vmap, func.function_type.return_type,
                         self.resolver, self.preds, self.block_end_values, self.bb_map)
            
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
            lower_compare(builder, operation, lhs, rhs, dst, self.vmap,
                          self.resolver, builder.block, self.preds, self.block_end_values, self.bb_map,
                          meta={"cmp_kind": cmp_kind} if cmp_kind else None)
            
        elif op == "call":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_call(builder, self.module, func_name, args, dst, self.vmap, self.resolver, self.preds, self.block_end_values, self.bb_map)
            
        elif op == "boxcall":
            box_vid = inst.get("box")
            method = inst.get("method")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_boxcall(builder, self.module, box_vid, method, args, dst,
                          self.vmap, self.resolver, self.preds, self.block_end_values, self.bb_map)
            # Optional: honor explicit dst_type for tagging (string handle)
            try:
                dst_type = inst.get("dst_type")
                if dst is not None and isinstance(dst_type, dict):
                    if dst_type.get("kind") == "handle" and dst_type.get("box_type") == "StringBox":
                        if hasattr(self.resolver, 'mark_string'):
                            self.resolver.mark_string(int(dst))
            except Exception:
                pass
            
        elif op == "externcall":
            func_name = inst.get("func")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_externcall(builder, self.module, func_name, args, dst,
                             self.vmap, self.resolver, self.preds, self.block_end_values, self.bb_map)
            
        elif op == "newbox":
            box_type = inst.get("type")
            args = inst.get("args", [])
            dst = inst.get("dst")
            lower_newbox(builder, self.module, box_type, args, dst, 
                        self.vmap, self.resolver)
            
        elif op == "typeop":
            operation = inst.get("operation")
            src = inst.get("src")
            dst = inst.get("dst")
            target_type = inst.get("target_type")
            lower_typeop(builder, operation, src, dst, target_type,
                         self.vmap, self.resolver, self.preds, self.block_end_values, self.bb_map)
            
        elif op == "safepoint":
            live = inst.get("live", [])
            lower_safepoint(builder, self.module, live, self.vmap,
                            resolver=self.resolver, preds=self.preds,
                            block_end_values=self.block_end_values, bb_map=self.bb_map)
            
        elif op == "barrier":
            barrier_type = inst.get("type", "memory")
            lower_barrier(builder, barrier_type)
            
        elif op == "while":
            # Experimental LoopForm lowering
            cond = inst.get("cond")
            body = inst.get("body", [])
            self.loop_count += 1
            if not lower_while_loopform(builder, func, cond, body,
                                       self.loop_count, self.vmap, self.bb_map,
                                       self.resolver, self.preds, self.block_end_values):
                # Fallback to regular while
                self._lower_while_regular(builder, inst, func)
        else:
            if os.environ.get('NYASH_CLI_VERBOSE') == '1':
                print(f"[Python LLVM] Unknown instruction: {op}")
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
    
    def _lower_while_regular(self, builder: ir.IRBuilder, inst: Dict[str, Any], func: ir.Function):
        """Fallback regular while lowering"""
        # Create basic blocks: cond -> body -> cond, and exit
        cond_vid = inst.get("cond")
        body_insts = inst.get("body", [])

        cur_bb = builder.block
        cond_bb = func.append_basic_block(name=f"while{self.loop_count}_cond")
        body_bb = func.append_basic_block(name=f"while{self.loop_count}_body")
        exit_bb = func.append_basic_block(name=f"while{self.loop_count}_exit")

        # Jump from current to cond
        builder.branch(cond_bb)

        # Cond block
        cbuild = ir.IRBuilder(cond_bb)
        try:
            cond_val = self.resolver.resolve_i64(cond_vid, builder.block, self.preds, self.block_end_values, self.vmap, self.bb_map)
        except Exception:
            cond_val = self.vmap.get(cond_vid)
        if cond_val is None:
            cond_val = ir.Constant(self.i1, 0)
        # Normalize to i1
        if hasattr(cond_val, 'type'):
            if isinstance(cond_val.type, ir.IntType) and cond_val.type.width == 64:
                zero64 = ir.Constant(self.i64, 0)
                cond_val = cbuild.icmp_unsigned('!=', cond_val, zero64, name="while_cond_i1")
            elif isinstance(cond_val.type, ir.PointerType):
                nullp = ir.Constant(cond_val.type, None)
                cond_val = cbuild.icmp_unsigned('!=', cond_val, nullp, name="while_cond_p1")
            elif isinstance(cond_val.type, ir.IntType) and cond_val.type.width == 1:
                # already i1
                pass
            else:
                # Fallback: treat as false
                cond_val = ir.Constant(self.i1, 0)
        else:
            cond_val = ir.Constant(self.i1, 0)

        cbuild.cbranch(cond_val, body_bb, exit_bb)

        # Body block
        bbuild = ir.IRBuilder(body_bb)
        # Allow nested lowering of body instructions within this block
        self._lower_instruction_list(bbuild, body_insts, func)
        # Ensure terminator: if not terminated, branch back to cond
        if bbuild.block.terminator is None:
            bbuild.branch(cond_bb)

        # Continue at exit
        builder.position_at_end(exit_bb)

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
            bb = self.bb_map.get(block_id)
            if bb is None:
                continue
            b = ir.IRBuilder(bb)
            try:
                b.position_at_start(bb)
            except Exception:
                pass
            for dst_vid, incoming in (dst_map or {}).items():
                # Ensure placeholder exists at block head
                phi = self.vmap.get(dst_vid)
                try:
                    is_phi = hasattr(phi, 'add_incoming')
                except Exception:
                    is_phi = False
                if not is_phi:
                    phi = b.phi(self.i64, name=f"phi_{dst_vid}")
                    self.vmap[dst_vid] = phi
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
                # Fill remaining predecessors with dst carry or zero
                for pred_bid in preds_list:
                    if pred_bid not in chosen:
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
        if os.environ.get('NYASH_CLI_VERBOSE') == '1':
            print(f"[Python LLVM] Generated dummy IR:\n{ir_text}")
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
    if os.environ.get('NYASH_CLI_VERBOSE') == '1':
        print(f"[Python LLVM] Generated LLVM IR (see NYASH_LLVM_DUMP_IR or tmp/nyash_harness.ll)")

    builder.compile_to_object(output_file)
    print(f"Compiled to {output_file}")

if __name__ == "__main__":
    main()
