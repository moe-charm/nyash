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
from instructions.phi import lower_phi, defer_phi_wiring
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
        
        # Process each function
        for func_data in functions:
            self.lower_function(func_data)
        
        # Wire deferred PHIs
        self._wire_deferred_phis()

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
        
        return str(self.module)
    
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
        for bid in order:
            block_data = block_by_id.get(bid)
            if block_data is None:
                continue
            bb = self.bb_map[bid]
            self.lower_block(bb, block_data, func)

        # Provide lifetime hints to resolver (which blocks define which values)
        try:
            self.resolver.def_blocks = self.def_blocks
        except Exception:
            pass
    
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
        created_ids: List[int] = []
        # Two-pass: lower all PHIs first to keep them grouped at top
        phi_insts = [inst for inst in instructions if inst.get("op") == "phi"]
        non_phi_insts = [inst for inst in instructions if inst.get("op") != "phi"]
        # Lower PHIs
        if phi_insts:
            # Ensure insertion at block start
            builder.position_at_start(bb)
            for inst in phi_insts:
                self.lower_instruction(builder, inst, func)
                try:
                    dst = inst.get("dst")
                    if isinstance(dst, int) and dst not in created_ids and dst in self.vmap:
                        created_ids.append(dst)
                except Exception:
                    pass
        # Lower non-PHI instructions strictly in original program order.
        # Reordering here can easily introduce use-before-def within the same
        # basic block (e.g., string ops that depend on prior me.* calls).
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
        snap: Dict[int, ir.Value] = {}
        # include function args (avoid 0 constant confusion later via special-case)
        try:
            arity = len(func.args)
        except Exception:
            arity = 0
        for i in range(arity):
            if i in self.vmap:
                snap[i] = self.vmap[i]
        for vid in created_ids:
            val = self.vmap.get(vid)
            if val is not None:
                snap[vid] = val
                # Record block-local definition for lifetime hinting
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
            lower_binop(builder, self.resolver, operation, lhs, rhs, dst,
                        self.vmap, builder.block, self.preds, self.block_end_values, self.bb_map)
            
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
            dst = inst.get("dst")
            incoming = inst.get("incoming", [])
            # Wire PHI immediately at the start of the current block using snapshots
            lower_phi(builder, dst, incoming, self.vmap, self.bb_map, builder.block, self.resolver, self.block_end_values, self.preds)
        
        elif op == "compare":
            # Dedicated compare op
            operation = inst.get("operation") or inst.get("op")
            lhs = inst.get("lhs")
            rhs = inst.get("rhs")
            dst = inst.get("dst")
            lower_compare(builder, operation, lhs, rhs, dst, self.vmap,
                          self.resolver, builder.block, self.preds, self.block_end_values, self.bb_map)
            
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
    
    def _wire_deferred_phis(self):
        """Wire all deferred PHI nodes"""
        for cur_bid, dst_vid, incoming in self.phi_deferrals:
            bb = self.bb_map.get(cur_bid)
            if bb is None:
                continue
            b = ir.IRBuilder(bb)
            b.position_at_start(bb)
            # Determine phi type: prefer pointer if any incoming is pointer; else f64; else i64
            phi_type = self.i64
            for (val_id, pred_bid) in incoming:
                snap = self.block_end_values.get(pred_bid, {})
                val = snap.get(val_id)
                if val is not None and hasattr(val, 'type'):
                    if hasattr(val.type, 'is_pointer') and val.type.is_pointer:
                        phi_type = val.type
                        break
                    elif str(val.type) == str(self.f64):
                        phi_type = self.f64
            phi = b.phi(phi_type, name=f"phi_{dst_vid}")
            for (val_id, pred_bid) in incoming:
                pred_bb = self.bb_map.get(pred_bid)
                if pred_bb is None:
                    continue
                # Self-reference takes precedence regardless of snapshot
                if val_id == dst_vid:
                    val = phi
                else:
                    # Prefer resolver-driven localization at the end of the predecessor block
                    if hasattr(self, 'resolver') and self.resolver is not None:
                        try:
                            pred_block_obj = pred_bb
                            val = self.resolver.resolve_i64(val_id, pred_block_obj, self.preds, self.block_end_values, self.vmap, self.bb_map)
                        except Exception:
                            val = None
                    else:
                        # Snapshot fallback
                        snap = self.block_end_values.get(pred_bid, {})
                        # Special-case: incoming 0 means typed zero/null, not value-id 0
                        if isinstance(val_id, int) and val_id == 0:
                            val = None
                        else:
                            val = snap.get(val_id)
                    if val is None:
                        # Default based on phi type
                        if isinstance(phi_type, ir.IntType):
                            val = ir.Constant(phi_type, 0)
                        elif isinstance(phi_type, ir.DoubleType):
                            val = ir.Constant(phi_type, 0.0)
                        else:
                            val = ir.Constant(phi_type, None)
                # Type adjust if needed
                if hasattr(val, 'type') and val.type != phi_type:
                    # Insert cast in predecessor block before its terminator
                    pb = ir.IRBuilder(pred_bb)
                    try:
                        term = pred_bb.terminator
                        if term is not None:
                            pb.position_before(term)
                        else:
                            pb.position_at_end(pred_bb)
                    except Exception:
                        pb.position_at_end(pred_bb)
                    if isinstance(phi_type, ir.IntType) and hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
                        val = pb.ptrtoint(val, phi_type, name=f"phi_p2i_{dst_vid}_{pred_bid}")
                    elif isinstance(phi_type, ir.PointerType) and hasattr(val, 'type') and isinstance(val.type, ir.IntType):
                        val = pb.inttoptr(val, phi_type, name=f"phi_i2p_{dst_vid}_{pred_bid}")
                    elif isinstance(phi_type, ir.IntType) and hasattr(val, 'type') and isinstance(val.type, ir.IntType):
                        if phi_type.width > val.type.width:
                            val = pb.zext(val, phi_type, name=f"phi_zext_{dst_vid}_{pred_bid}")
                        elif phi_type.width < val.type.width:
                            val = pb.trunc(val, phi_type, name=f"phi_trunc_{dst_vid}_{pred_bid}")
                phi.add_incoming(val, pred_bb)
            self.vmap[dst_vid] = phi
    
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
        print(f"[Python LLVM] Generated LLVM IR:\n{llvm_ir}")

    builder.compile_to_object(output_file)
    print(f"Compiled to {output_file}")

if __name__ == "__main__":
    main()
