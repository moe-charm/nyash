"""
Resolver API (Python version)
Based on src/backend/llvm/compiler/codegen/instructions/resolver.rs
"""

from typing import Dict, Optional, Any, Tuple
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
        
        # Type shortcuts
        self.i64 = ir.IntType(64)
        self.i8p = ir.IntType(8).as_pointer()
        self.f64_type = ir.DoubleType()
    
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

        # Do not trust global vmap across blocks: always localize via preds when available
        
        # Get predecessor blocks
        try:
            bid = int(str(current_block.name).replace('bb',''))
        except Exception:
            bid = -1
        pred_ids = [p for p in preds.get(bid, []) if p != bid]
        
        if not pred_ids:
            # Entry block or no predecessors
            base_val = vmap.get(value_id, ir.Constant(self.i64, 0))
            # Do not emit casts here; if pointer, fall back to zero
            if hasattr(base_val, 'type') and isinstance(base_val.type, ir.IntType):
                result = base_val if base_val.type.width == 64 else ir.Constant(self.i64, 0)
            elif hasattr(base_val, 'type') and isinstance(base_val.type, ir.PointerType):
                result = ir.Constant(self.i64, 0)
            else:
                result = ir.Constant(self.i64, 0)
        else:
            # Create PHI at block start
            saved_pos = None
            if self.builder is not None:
                saved_pos = self.builder.block
                self.builder.position_at_start(current_block)
            
            phi = self.builder.phi(self.i64, name=f"loc_i64_{value_id}")
            
            # Add incoming values from predecessors
            for pred_id in pred_ids:
                pred_vals = block_end_values.get(pred_id, {})
                val = pred_vals.get(value_id)
                # Coerce in predecessor block if needed
                if val is None:
                    coerced = ir.Constant(self.i64, 0)
                else:
                    if hasattr(val, 'type') and isinstance(val.type, ir.IntType):
                        coerced = val if val.type.width == 64 else ir.Constant(self.i64, 0)
                    elif hasattr(val, 'type') and isinstance(val.type, ir.PointerType):
                        # insert ptrtoint in predecessor
                        pred_bb = bb_map.get(pred_id) if bb_map is not None else None
                        if pred_bb is not None:
                            pb = ir.IRBuilder(pred_bb)
                            try:
                                term = pred_bb.terminator
                                if term is not None:
                                    pb.position_before(term)
                                else:
                                    pb.position_at_end(pred_bb)
                            except Exception:
                                pb.position_at_end(pred_bb)
                            coerced = pb.ptrtoint(val, self.i64, name=f"res_p2i_{value_id}_{pred_id}")
                        else:
                            coerced = ir.Constant(self.i64, 0)
                    else:
                        coerced = ir.Constant(self.i64, 0)
                # Use predecessor block if available
                pred_bb = None
                if bb_map is not None:
                    pred_bb = bb_map.get(pred_id)
                if pred_bb is not None:
                    phi.add_incoming(coerced, pred_bb)
            # If no valid incoming were added, fold to zero to avoid invalid PHI
            if len(getattr(phi, 'incoming', [])) == 0:
                # Replace with zero constant and discard phi
                result = ir.Constant(self.i64, 0)
                # Restore position and cache
                if saved_pos and self.builder is not None:
                    self.builder.position_at_end(saved_pos)
                self.i64_cache[cache_key] = result
                return result
            
            # Restore position
            if saved_pos and self.builder is not None:
                self.builder.position_at_end(saved_pos)
            
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
