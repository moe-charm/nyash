"""
Resolver API (Python version)
Based on src/backend/llvm/compiler/codegen/instructions/resolver.rs
"""

from typing import Dict, Optional, Any, Tuple
import os
import llvmlite.ir as ir

class Resolver:
    """
    Centralized value resolution with per-block caching.
    Following the Core Invariants from LLVM_LAYER_OVERVIEW.md:
    - Resolver-only reads
    - Localize at block start (PHI creation)
    - Cache per (block, value) to avoid redundant PHIs
    """
    
    def __init__(self, a, b=None):
        """Flexible init: either (builder, module) or (vmap, bb_map) for legacy wiring."""
        if hasattr(a, 'position_at_end'):
            # a is IRBuilder
            self.builder = a
            self.module = b
        else:
            # Legacy constructor (vmap, bb_map) — builder/module will be set later when available
            self.builder = None
            self.module = None
        
        # Caches: (block_name, value_id) -> llvm value
        self.i64_cache: Dict[Tuple[str, int], ir.Value] = {}
        self.ptr_cache: Dict[Tuple[str, int], ir.Value] = {}
        self.f64_cache: Dict[Tuple[str, int], ir.Value] = {}
        # String literal map: value_id -> Python string (for by-name calls)
        self.string_literals: Dict[int, str] = {}
        # Optional: value_id -> i8* pointer for string constants (lower_const can populate)
        self.string_ptrs: Dict[int, ir.Value] = {}
        # Track value-ids that are known to represent string handles (i64)
        # This is a best-effort tag used to decide '+' as string concat when both sides are i64.
        self.string_ids: set[int] = set()
        
        # Type shortcuts
        self.i64 = ir.IntType(64)
        self.i8p = ir.IntType(8).as_pointer()
        self.f64_type = ir.DoubleType()
        # Cache for recursive end-of-block i64 resolution
        self._end_i64_cache: Dict[Tuple[int, int], ir.Value] = {}
        # Lifetime hint: value_id -> set(block_id) where it's known to be defined
        # Populated by the builder when available.
        self.def_blocks = {}

    def mark_string(self, value_id: int) -> None:
        try:
            self.string_ids.add(int(value_id))
        except Exception:
            pass

    def is_stringish(self, value_id: int) -> bool:
        try:
            return int(value_id) in self.string_ids
        except Exception:
            return False
    
    def resolve_i64(
        self,
        value_id: int,
        current_block: ir.Block,
        preds: Dict[int, list],
        block_end_values: Dict[int, Dict[int, Any]],
        vmap: Dict[int, Any],
        bb_map: Optional[Dict[int, ir.Block]] = None
    ) -> ir.Value:
        """
        Resolve a MIR value as i64 dominating the current block.
        Creates PHI at block start if needed, caches the result.
        """
        cache_key = (current_block.name, value_id)
        
        # Check cache
        if cache_key in self.i64_cache:
            return self.i64_cache[cache_key]

        # Do not trust global vmap across blocks unless we know it's defined in this block.
        
        # Get predecessor blocks
        try:
            bid = int(str(current_block.name).replace('bb',''))
        except Exception:
            bid = -1
        pred_ids = [p for p in preds.get(bid, []) if p != bid]

        # Lifetime hint: if value is defined in this block, and present in vmap as i64, reuse it.
        try:
            defined_here = value_id in self.def_blocks and bid in self.def_blocks.get(value_id, set())
        except Exception:
            defined_here = False
        if defined_here:
            existing = vmap.get(value_id)
            if existing is not None and hasattr(existing, 'type') and isinstance(existing.type, ir.IntType) and existing.type.width == 64:
                if os.environ.get('NYASH_LLVM_TRACE_VALUES') == '1':
                    print(f"[VAL] reuse local v{value_id} in bb{bid}", flush=True)
                self.i64_cache[cache_key] = existing
                return existing
        
        if not pred_ids:
            # Entry block or no predecessors: prefer local vmap value (already dominating)
            base_val = vmap.get(value_id)
            if base_val is None:
                result = ir.Constant(self.i64, 0)
            else:
                # If pointer string, box to handle in current block (use local builder)
                if hasattr(base_val, 'type') and isinstance(base_val.type, ir.PointerType) and self.module is not None:
                    pb = ir.IRBuilder(current_block)
                    try:
                        pb.position_at_start(current_block)
                    except Exception:
                        pass
                    i8p = ir.IntType(8).as_pointer()
                    v = base_val
                    try:
                        if hasattr(v.type, 'pointee') and isinstance(v.type.pointee, ir.ArrayType):
                            c0 = ir.Constant(ir.IntType(32), 0)
                            v = pb.gep(v, [c0, c0], name=f"res_gep_{value_id}")
                    except Exception:
                        pass
                    # declare and call boxer
                    for f in self.module.functions:
                        if f.name == 'nyash.box.from_i8_string':
                            box_from = f
                            break
                    else:
                        box_from = ir.Function(self.module, ir.FunctionType(self.i64, [i8p]), name='nyash.box.from_i8_string')
                    result = pb.call(box_from, [v], name=f"res_ptr2h_{value_id}")
                elif hasattr(base_val, 'type') and isinstance(base_val.type, ir.IntType):
                    result = base_val if base_val.type.width == 64 else ir.Constant(self.i64, 0)
                else:
                    result = ir.Constant(self.i64, 0)
        else:
            # Sealed SSA localization: create a PHI at the start of current block
            # that merges i64-coerced snapshots from each predecessor. This guarantees
            # dominance for downstream uses within the current block.
            # Use shared builder so insertion order is respected relative to other instructions.
            # Save current insertion point
            sb = self.builder
            if sb is None:
                # As a conservative fallback, synthesize zero (should not happen in normal lowering)
                result = ir.Constant(self.i64, 0)
                self.i64_cache[cache_key] = result
                return result
            orig_block = sb.block
            # Insert PHI at the very start of current_block
            sb.position_at_start(current_block)
            phi = sb.phi(self.i64, name=f"loc_i64_{value_id}")
            for pred_id in pred_ids:
                # Value at the end of predecessor, coerced to i64 within pred block
                coerced = self._value_at_end_i64(value_id, pred_id, preds, block_end_values, vmap, bb_map)
                pred_bb = bb_map.get(pred_id) if bb_map is not None else None
                if pred_bb is None:
                    continue
                phi.add_incoming(coerced, pred_bb)
            # Restore insertion point to original location
            try:
                if orig_block is not None:
                    term = orig_block.terminator
                    if term is not None:
                        sb.position_before(term)
                    else:
                        sb.position_at_end(orig_block)
            except Exception:
                pass
            # Use the PHI value as the localized definition for this block
            result = phi
        
        # Cache and return
        self.i64_cache[cache_key] = result
        return result
    
    def resolve_ptr(self, value_id: int, current_block: ir.Block, 
                    preds: Dict, block_end_values: Dict, vmap: Dict) -> ir.Value:
        """Resolve as i8* pointer"""
        cache_key = (current_block.name, value_id)
        if cache_key in self.ptr_cache:
            return self.ptr_cache[cache_key]
        # Coerce current vmap value or GlobalVariable to i8*
        val = vmap.get(value_id)
        if val is None:
            result = ir.Constant(self.i8p, None)
        else:
            if hasattr(val, 'type') and isinstance(val, ir.PointerType):
                # If pointer to array (GlobalVariable), GEP to first element
                ty = val.type.pointee if hasattr(val.type, 'pointee') else None
                if ty is not None and hasattr(ty, 'element'):
                    c0 = ir.Constant(ir.IntType(32), 0)
                    result = self.builder.gep(val, [c0, c0], name=f"res_str_gep_{value_id}")
                else:
                    result = val
            elif hasattr(val, 'type') and isinstance(val.type, ir.IntType):
                result = self.builder.inttoptr(val, self.i8p, name=f"res_i2p_{value_id}")
            else:
                # f64 or others -> zero
                result = ir.Constant(self.i8p, None)
        self.ptr_cache[cache_key] = result
        return result

    def _value_at_end_i64(self, value_id: int, block_id: int, preds: Dict[int, list],
                          block_end_values: Dict[int, Dict[int, Any]], vmap: Dict[int, Any],
                          bb_map: Optional[Dict[int, ir.Block]] = None,
                          _vis: Optional[set] = None) -> ir.Value:
        """Resolve value as i64 at the end of a given block by traversing predecessors if needed."""
        key = (block_id, value_id)
        if key in self._end_i64_cache:
            return self._end_i64_cache[key]
        if _vis is None:
            _vis = set()
        if key in _vis:
            return ir.Constant(self.i64, 0)
        _vis.add(key)

        # If present in snapshot, coerce there
        snap = block_end_values.get(block_id, {})
        if value_id in snap and snap[value_id] is not None:
            val = snap[value_id]
            coerced = self._coerce_in_block_to_i64(val, block_id, bb_map)
            self._end_i64_cache[key] = coerced
            return coerced

        # Try recursively from predecessors
        pred_ids = [p for p in preds.get(block_id, []) if p != block_id]
        for p in pred_ids:
            v = self._value_at_end_i64(value_id, p, preds, block_end_values, vmap, bb_map, _vis)
            if v is not None:
                self._end_i64_cache[key] = v
                return v

        # Do not use global vmap here; if not materialized by end of this block
        # (or its preds), bail out with zero to preserve dominance.

        z = ir.Constant(self.i64, 0)
        self._end_i64_cache[key] = z
        return z

    def _coerce_in_block_to_i64(self, val: Any, block_id: int, bb_map: Optional[Dict[int, ir.Block]]) -> ir.Value:
        """Ensure a value is available as i64 at the end of the given block by inserting casts/boxing there."""
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            # Re-materialize an i64 definition in the predecessor block to satisfy dominance
            pred_bb = bb_map.get(block_id) if bb_map is not None else None
            if pred_bb is None:
                return ir.Constant(self.i64, 0)
            pb = ir.IRBuilder(pred_bb)
            try:
                term = pred_bb.terminator
                if term is not None:
                    pb.position_before(term)
                else:
                    pb.position_at_end(pred_bb)
            except Exception:
                pb.position_at_end(pred_bb)
            if val.type.width == 64:
                z = ir.Constant(self.i64, 0)
                return pb.add(val, z, name=f"res_copy_{block_id}")
            else:
                # Extend/truncate to i64 in pred block
                if val.type.width < 64:
                    return pb.zext(val, self.i64, name=f"res_zext_{block_id}")
                else:
                    return pb.trunc(val, self.i64, name=f"res_trunc_{block_id}")
        if hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
            pred_bb = bb_map.get(block_id) if bb_map is not None else None
            if pred_bb is None:
                return ir.Constant(self.i64, 0)
            pb = ir.IRBuilder(pred_bb)
            try:
                term = pred_bb.terminator
                if term is not None:
                    pb.position_before(term)
                else:
                    pb.position_at_end(pred_bb)
            except Exception:
                pb.position_at_end(pred_bb)
            i8p = ir.IntType(8).as_pointer()
            v = val
            try:
                if hasattr(v.type, 'pointee') and isinstance(v.type.pointee, ir.ArrayType):
                    c0 = ir.Constant(ir.IntType(32), 0)
                    v = pb.gep(v, [c0, c0], name=f"res_gep_{block_id}_{id(val)}")
            except Exception:
                pass
            # declare boxer
            box_from = None
            for f in self.module.functions:
                if f.name == 'nyash.box.from_i8_string':
                    box_from = f
                    break
            if box_from is None:
                box_from = ir.Function(self.module, ir.FunctionType(self.i64, [i8p]), name='nyash.box.from_i8_string')
            return pb.call(box_from, [v], name=f"res_ptr2h_{block_id}")
        return ir.Constant(self.i64, 0)
    
    def resolve_f64(self, value_id: int, current_block: ir.Block,
                    preds: Dict, block_end_values: Dict, vmap: Dict) -> ir.Value:
        """Resolve as f64"""
        cache_key = (current_block.name, value_id)
        if cache_key in self.f64_cache:
            return self.f64_cache[cache_key]
        val = vmap.get(value_id)
        if val is None:
            result = ir.Constant(self.f64_type, 0.0)
        else:
            if hasattr(val, 'type') and val.type == self.f64_type:
                result = val
            elif hasattr(val, 'type') and isinstance(val.type, ir.IntType):
                result = self.builder.sitofp(val, self.f64_type)
            elif hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
                tmp = self.builder.ptrtoint(val, self.i64, name=f"res_p2i_{value_id}")
                result = self.builder.sitofp(tmp, self.f64_type, name=f"res_i2f_{value_id}")
            else:
                result = ir.Constant(self.f64_type, 0.0)
        self.f64_cache[cache_key] = result
        return result
    
    def _coerce_to_i64(self, val: Any) -> ir.Value:
        """Coerce various types to i64"""
        if isinstance(val, ir.Constant) and val.type == self.i64:
            return val
        elif hasattr(val, 'type') and val.type.is_pointer:
            # ptr to int
            return self.builder.ptrtoint(val, self.i64, name=f"res_p2i_{getattr(val,'name','x')}") if self.builder is not None else ir.Constant(self.i64, 0)
        elif hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            # int to int (extend/trunc)
            if val.type.width < 64:
                return self.builder.zext(val, self.i64) if self.builder is not None else ir.Constant(self.i64, 0)
            elif val.type.width > 64:
                return self.builder.trunc(val, self.i64) if self.builder is not None else ir.Constant(self.i64, 0)
            return val
        else:
            # Default zero
            return ir.Constant(self.i64, 0)
