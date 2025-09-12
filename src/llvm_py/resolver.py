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
    
    def __init__(self, builder: ir.IRBuilder, module: ir.Module):
        self.builder = builder
        self.module = module
        
        # Caches: (block_name, value_id) -> llvm value
        self.i64_cache: Dict[Tuple[str, int], ir.Value] = {}
        self.ptr_cache: Dict[Tuple[str, int], ir.Value] = {}
        self.f64_cache: Dict[Tuple[str, int], ir.Value] = {}
        
        # Type shortcuts
        self.i64 = ir.IntType(64)
        self.i8p = ir.IntType(8).as_pointer()
        self.f64_type = ir.DoubleType()
    
    def resolve_i64(
        self,
        value_id: int,
        current_block: ir.Block,
        preds: Dict[str, list],
        block_end_values: Dict[str, Dict[int, Any]],
        vmap: Dict[int, Any]
    ) -> ir.Value:
        """
        Resolve a MIR value as i64 dominating the current block.
        Creates PHI at block start if needed, caches the result.
        """
        cache_key = (current_block.name, value_id)
        
        # Check cache
        if cache_key in self.i64_cache:
            return self.i64_cache[cache_key]
        
        # Get predecessor blocks
        pred_names = preds.get(current_block.name, [])
        
        if not pred_names:
            # Entry block or no predecessors
            base_val = vmap.get(value_id, ir.Constant(self.i64, 0))
            result = self._coerce_to_i64(base_val)
        else:
            # Create PHI at block start
            saved_pos = self.builder.block
            self.builder.position_at_start(current_block)
            
            phi = self.builder.phi(self.i64, name=f"loc_i64_{value_id}")
            
            # Add incoming values from predecessors
            for pred_name in pred_names:
                pred_vals = block_end_values.get(pred_name, {})
                val = pred_vals.get(value_id, ir.Constant(self.i64, 0))
                coerced = self._coerce_to_i64(val)
                # Note: In real implementation, need pred block reference
                phi.add_incoming(coerced, pred_name)  # Simplified
            
            # Restore position
            if saved_pos:
                self.builder.position_at_end(saved_pos)
            
            result = phi
        
        # Cache and return
        self.i64_cache[cache_key] = result
        return result
    
    def resolve_ptr(self, value_id: int, current_block: ir.Block, 
                    preds: Dict, block_end_values: Dict, vmap: Dict) -> ir.Value:
        """Resolve as i8* pointer"""
        # Similar to resolve_i64 but with pointer type
        # TODO: Implement
        pass
    
    def resolve_f64(self, value_id: int, current_block: ir.Block,
                    preds: Dict, block_end_values: Dict, vmap: Dict) -> ir.Value:
        """Resolve as f64"""
        # Similar pattern
        # TODO: Implement
        pass
    
    def _coerce_to_i64(self, val: Any) -> ir.Value:
        """Coerce various types to i64"""
        if isinstance(val, ir.Constant) and val.type == self.i64:
            return val
        elif hasattr(val, 'type') and val.type.is_pointer:
            # ptr to int
            return self.builder.ptrtoint(val, self.i64)
        elif hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            # int to int (extend/trunc)
            if val.type.width < 64:
                return self.builder.zext(val, self.i64)
            elif val.type.width > 64:
                return self.builder.trunc(val, self.i64)
            return val
        else:
            # Default zero
            return ir.Constant(self.i64, 0)