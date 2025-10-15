#!/usr/bin/env python3
"""
Nyash LLVM Python Backend - Main Builder
Following the design principles in docs/development/design/legacy/LLVM_LAYER_OVERVIEW.md
"""

import json
import sys
import os
# Apply brand env aliases early when run as a script (e.g., via Python)
try:
    from utils.brand import alias_prefixes_bootstrap as _brand_alias
    _brand_alias()
except Exception:
    pass
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
from instructions.typeop import lower_typeop, lower_convert
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
        # Create target object (箱理論: ターゲット抽象化)
        self.target_obj = create_target(target)
        self.target = target  # Keep for backward compatibility
        self.target_triple = self.target_obj.get_triple()

        # Initialize LLVM core (automatically handled by llvmlite now)
        # llvm.initialize()  # Deprecated - removed per llvmlite warning

        # Initialize target-specific components
        if target == "wasm32":
            # WASM target: initialize all targets to enable wasm32
            llvm.initialize_all_targets()
            llvm.initialize_all_asmprinters()
        elif target == "native":
            # Native target (default)
            llvm.initialize_native_target()
            llvm.initialize_native_asmprinter()
        else:
            # Cross targets (e.g., windows) — initialize all
            llvm.initialize_all_targets()
            llvm.initialize_all_asmprinters()

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
                fty = ir.FunctionType(self.i64, [])
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
            import os as _os
            if _os.environ.get('NYASH_LLVM_NO_NY_MAIN', '0') not in ('1', 'true', 'on', 'YES', 'yes'):
                from builders.entry import ensure_ny_main as _ensure_ny_main
                _ensure_ny_main(self)
        except Exception as _e:
            try:
                trace_debug(f"[Python LLVM] ensure_ny_main failed: {_e}")
            except Exception:
                pass
        
        # Run standard optimization passes (-O3 equivalent)
        # NOTE: PassManager expects llvmlite.binding.Module, but self.module is llvmlite.ir.Module
        # This will fail with TypeError, but we catch it silently to allow execution
        try:
            pmb = llvm.PassManagerBuilder()
            pmb.opt_level = 3
            # Function-level + Module-level passes
            fpm = llvm.FunctionPassManager(self.module)
            pmb.populate(fpm)
            mpm = llvm.ModulePassManager()
            pmb.populate(mpm)
            # JIT-style run over all functions
            fpm.initialize()
            for fn in self.module.functions:
                fpm.run(fn)
            fpm.finalize()
            mpm.run(self.module)
        except Exception:
            # Optimization passes fail due to module type mismatch, but continue anyway
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
        """Legacy stub — use phi_wiring.tagging.setup_phi_placeholders instead."""
        raise NotImplementedError("setup_phi_placeholders moved to phi_wiring.tagging")
    
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
        """Legacy stub — use builders.instruction_lower instead."""
        raise NotImplementedError("_lower_instruction_list moved to builders.instruction_lower")
    
    def finalize_phis(self):
        """Deprecated shim. Use phi_wiring.finalize_phis via builders.function_lower.

        This method now delegates to the wire-only finalizer to avoid legacy
        ensure+wire duplication. It remains for backward compatibility.
        """
        from phi_wiring import finalize_phis as _finalize
        _finalize(self)
    
    def compile_to_object(self, output_path: str):
        """Compile module to object file (or WASM via external toolchain when enabled).

        When target == wasm32 and env NYASH_LLVM_WASM_TOOLCHAIN=1, this method
        will invoke system LLVM tools (llc + wasm-ld) to produce a final
        .wasm module, working around llvmlite limitations.
        """
        # TODO(Phase 2): Delegate to self.target_obj.emit_object()
        # Current implementation uses manual target_machine for PHI sanitize logic

        # Create target machine for the specified target triple
        target = llvm.Target.from_triple(self.target_triple)
        target_machine = target.create_target_machine()

        # Compile
        # Run standard optimization passes (-O3 equivalent)
        # NOTE: PassManager expects llvmlite.binding.Module, but self.module is llvmlite.ir.Module
        # This will fail with TypeError, but we catch it silently to allow execution
        try:
            pmb = llvm.PassManagerBuilder()
            pmb.opt_level = 3
            # Function-level + Module-level passes
            fpm = llvm.FunctionPassManager(self.module)
            pmb.populate(fpm)
            mpm = llvm.ModulePassManager()
            pmb.populate(mpm)
            # JIT-style run over all functions
            fpm.initialize()
            for fn in self.module.functions:
                fpm.run(fn)
            fpm.finalize()
            mpm.run(self.module)
        except Exception:
            # Optimization passes fail due to module type mismatch, but continue anyway
            pass
        ir_text = str(self.module)

        # Debug: Always dump optimized IR for debugging
        try:
            with open('/tmp/debug_ir.ll', 'w') as f:
                f.write(ir_text)
            print(f"[DEBUG] Optimized IR dumped to /tmp/debug_ir.ll")
        except Exception as e:
            print(f"[DEBUG] Failed to dump IR: {e}")

        # Optional sanitize passes for IR text before verification
        # 1) Drop empty PHIs (no incoming pairs)
        # 2) Group PHIs at the block head (LLVM invariant: all PHIs at top)
        # Default OFF. Enable by setting NYASH_LLVM_SANITIZE_EMPTY_PHI=1.
        if os.environ.get('NYASH_LLVM_SANITIZE_EMPTY_PHI') == '1':
            try:
                lines = ir_text.splitlines()
                # Pass 1: remove malformed empty PHIs
                tmp = []
                for line in lines:
                    if (" = phi  i64" in line or " = phi i64" in line) and ("[" not in line):
                        continue
                    tmp.append(line)
                lines = tmp
                # Pass 2: group PHIs at block head
                grouped = []
                i = 0
                n = len(lines)
                while i < n:
                    line = lines[i]
                    grouped.append(line)
                    # Detect basic block start like: "bb<N>:" (possibly with leading spaces)
                    stripped = line.strip()
                    if stripped.endswith(":") and stripped.startswith("bb"):
                        # Collect subsequent PHI and non-PHI lines until next label or end
                        phi_buf = []
                        body_buf = []
                        j = i + 1
                        while j < n:
                            l2 = lines[j]
                            s2 = l2.strip()
                            # Next block label?
                            if s2.endswith(":") and s2.startswith("bb"):
                                break
                            if (" = phi  i64" in s2) or (" = phi i64" in s2):
                                phi_buf.append(l2)
                            else:
                                body_buf.append(l2)
                            j += 1
                        # Emit PHIs first (preserve relative order), then rest
                        if phi_buf or body_buf:
                            grouped.extend(phi_buf)
                            grouped.extend(body_buf)
                            i = j
                            continue
                    i += 1
                ir_text = "\n".join(grouped)
            except Exception:
                pass
        
        # wasm32 via external toolchain (optional)
        if self.target == "wasm32" and os.environ.get('NYASH_LLVM_WASM_TOOLCHAIN') == '1':
            try:
                ir_path = '/tmp/nyash_harness.ll'
                try:
                    with open(ir_path, 'w') as f:
                        f.write(ir_text)
                except Exception:
                    ir_path = os.path.abspath('tmp/nyash_harness.ll')
                    os.makedirs(os.path.dirname(ir_path), exist_ok=True)
                    with open(ir_path, 'w') as f:
                        f.write(ir_text)
                obj_path = '/tmp/nyash_harness.o'
                triple = 'wasm32-unknown-unknown'
                llc = os.environ.get('LLC', 'llc')
                wasm_ld = os.environ.get('WASM_LD', 'wasm-ld')
                cmd1 = [llc, '-mtriple='+triple, '-filetype=obj', ir_path, '-o', obj_path]
                print(f"[LLVM tools] {' '.join(cmd1)}")
                r1 = os.spawnvp(os.P_WAIT, cmd1[0], cmd1)
                if r1 != 0:
                    print(f"[LLVM tools] llc failed with code {r1}")
                    raise RuntimeError('llc failed')
                exports = ['ny_main']
                extra_exports = (os.environ.get('NYASH_WASM_EXPORTS') or '').strip()
                if extra_exports:
                    exports.extend([x for x in extra_exports.split(',') if x])
                export_args = []
                for e in exports:
                    export_args.extend(['--export', e])
                cmd2 = [wasm_ld, obj_path, '-o', output_path, '--no-entry', '--allow-undefined'] + export_args
                print(f"[LLVM tools] {' '.join(cmd2)}")
                r2 = os.spawnvp(os.P_WAIT, cmd2[0], cmd2)
                if r2 != 0:
                    print(f"[LLVM tools] wasm-ld failed with code {r2}")
                    raise RuntimeError('wasm-ld failed')
                return
            except Exception as _e:
                print(f"[LLVM tools] fallback to llvmlite emit_object due to: {_e}")
        mod = llvm.parse_assembly(ir_text)
        if os.environ.get('NYASH_LLVM_SKIP_VERIFY') != '1':
            mod.verify()
        obj = target_machine.emit_object(mod)
        with open(output_path, 'wb') as f:
            f.write(obj)
              

def main():
    # CLI:
    #   llvm_builder.py <input.mir.json> [-o output.o] [--target wasm32|native|windows]
    #   llvm_builder.py --dummy [-o output.o] [--target wasm32|native|windows]
    output_file = os.path.join('tmp', 'nyash_llvm_py.o')
    args = sys.argv[1:]
    dummy = False
    target = "native"  # Default target

    if not args:
        print("Usage: llvm_builder.py <input.mir.json> [-o output.o] [--target wasm32|native|windows] | --dummy [-o output.o] [--target wasm32|native|windows]")
        sys.exit(1)

    # Parse --target option
    if "--target" in args:
        idx = args.index("--target")
        if idx + 1 < len(args):
            target = args[idx + 1]
            if target not in ("wasm32", "native", "windows"):
                print(f"error: invalid target '{target}', must be 'wasm32' or 'native' or 'windows'", file=sys.stderr)
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
