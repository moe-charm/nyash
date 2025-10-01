"""
Resolver API (Python version)
Based on src/backend/llvm/compiler/codegen/instructions/resolver.rs
"""

from typing import Dict, Optional, Any, Tuple
import os
from trace import phi as trace_phi
from trace import values as trace_values
import llvmlite.ir as ir

class Resolver:
    """
    Centralized value resolution with per-block caching.
    Following the Core Invariants from docs/development/design/legacy/LLVM_LAYER_OVERVIEW.md:
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
            try:
                # Keep references to global maps when provided
                self.global_vmap = a if isinstance(a, dict) else None
                self.global_bb_map = b if isinstance(b, dict) else None
            except Exception:
                self.global_vmap = None
                self.global_bb_map = None
        
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
        # Optional: block -> { dst_vid -> [(pred_bid, val_vid), ...] } for PHIs from MIR JSON
        self.block_phi_incomings = {}

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

        # If this block has a declared MIR PHI for the value, prefer that placeholder
        # and avoid creating any PHI here. Incoming is wired by finalize_phis().
        try:
            try:
                block_id = int(str(current_block.name).replace('bb',''))
            except Exception:
                block_id = -1
            if isinstance(self.block_phi_incomings, dict):
                bmap = self.block_phi_incomings.get(block_id)
                if isinstance(bmap, dict) and value_id in bmap:
                    existing_cur = vmap.get(value_id)
                    # Fallback: try builder/global vmap when local map lacks placeholder
                    try:
                        if (existing_cur is None or not hasattr(existing_cur, 'add_incoming')) and hasattr(self, 'global_vmap') and isinstance(self.global_vmap, dict):
                            gcand = self.global_vmap.get(value_id)
                            if gcand is not None and hasattr(gcand, 'add_incoming'):
                                existing_cur = gcand
                    except Exception:
                        pass
                    # Use placeholder only if it belongs to the current block; otherwise
                    # create/ensure a local PHI at the current block head to dominate uses.
                    is_phi_here = False
                    try:
                        is_phi_here = (
                            existing_cur is not None
                            and hasattr(existing_cur, 'add_incoming')
                            and getattr(getattr(existing_cur, 'basic_block', None), 'name', None) == current_block.name
                        )
                    except Exception:
                        is_phi_here = False
                    if is_phi_here:
                        self.i64_cache[cache_key] = existing_cur
                        return existing_cur
                    # Do not synthesize PHI here; expect predeclared placeholder exists.
                    # Fallback to 0 to keep IR consistent if placeholder is missing (should be rare).
                    zero = ir.Constant(self.i64, 0)
                    self.i64_cache[cache_key] = zero
                    return zero
        except Exception:
            pass
        
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
                trace_values(f"[resolve] local reuse: bb{bid} v{value_id}")
                self.i64_cache[cache_key] = existing
                return existing
        else:
            # Do NOT blindly reuse vmap across blocks: it may reference values defined
            # in non-dominating predecessors (e.g., other branches). Only reuse when
            # defined_here (handled above) or at entry/no-preds (handled below).
            pass
        
        if not pred_ids:
            # Entry block or no predecessors: prefer local vmap value (already dominating)
            base_val = vmap.get(value_id)
            if base_val is None:
                result = ir.Constant(self.i64, 0)
                trace_phi(f"[resolve] bb{bid} v{value_id} entry/no-preds → 0")
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
        elif len(pred_ids) == 1:
            # Single-predecessor block: take predecessor end-of-block value directly
            coerced = self._value_at_end_i64(value_id, pred_ids[0], preds, block_end_values, vmap, bb_map)
            self.i64_cache[cache_key] = coerced
            return coerced
        else:
            # Multi-pred: if JSON declares a PHI for (current block, value_id),
            # materialize it on-demand via end-of-block resolver. Otherwise,
            # synthesize a localization PHI at the current block head to ensure
            # dominance for downstream uses (MIR13 PHI-off compatibility).
            try:
                cur_bid = int(str(current_block.name).replace('bb',''))
            except Exception:
                cur_bid = -1
            declared = False
            try:
                if isinstance(self.block_phi_incomings, dict):
                    m = self.block_phi_incomings.get(cur_bid)
                    if isinstance(m, dict) and value_id in m:
                        declared = True
            except Exception:
                declared = False
            if declared:
                # Return existing placeholder if present; do not create a new PHI here.
                trace_phi(f"[resolve] use placeholder PHI: bb{cur_bid} v{value_id}")
                placeholder = vmap.get(value_id)
                if (placeholder is None or not hasattr(placeholder, 'add_incoming')) and hasattr(self, 'global_vmap') and isinstance(self.global_vmap, dict):
                    cand = self.global_vmap.get(value_id)
                    if cand is not None and hasattr(cand, 'add_incoming'):
                        placeholder = cand
                result = placeholder if (placeholder is not None and hasattr(placeholder, 'add_incoming')) else ir.Constant(self.i64, 0)
            else:
                # 箱理論: vmapに既にPHIが存在する場合は、それを使用（PhiHandlerからの重複回避）
                existing_phi = vmap.get(value_id)
                if existing_phi is not None and hasattr(existing_phi, 'add_incoming'):
                    trace_phi(f"[resolve] use existing PHI from vmap: bb{cur_bid} v{value_id}")
                    result = existing_phi
                    return result

                # No declared PHI and multi-pred: synthesize a local PHI at the head of the block
                # using end-of-block snapshots from predecessors.
                try:
                    # Create PHI at block head
                    head_builder = ir.IRBuilder(current_block)
                    try:
                        head_builder.position_at_start(current_block)
                    except Exception:
                        pass
                    phi = head_builder.phi(self.i64, name=f"phi_loc_{value_id}")
                    # Add incoming values from each predecessor
                    incomings = []
                    for p in pred_ids:
                        v_end = self._value_at_end_i64(value_id, p, preds, block_end_values, vmap, bb_map)
                        # Resolve predecessor basic block
                        bbpred = None
                        try:
                            if bb_map is not None:
                                bbpred = bb_map.get(int(p))
                        except Exception:
                            bbpred = None
                        if bbpred is None:
                            # Cannot resolve predecessor; coerce to zero to keep IR valid
                            v_end = ir.Constant(self.i64, 0)
                        incomings.append((v_end, bbpred))
                    # Filter out any missing predecessor blocks (should be rare)
                    incomings = [(v, b) for (v, b) in incomings if b is not None]
                    if len(incomings) == 0:
                        result = ir.Constant(self.i64, 0)
                    else:
                        for (v, b) in incomings:
                            phi.add_incoming(v, b)
                        trace_phi(f"[resolve] synth local PHI: bb{cur_bid} v{value_id} preds={pred_ids}")
                        # Update local map to dominate subsequent users in this block
                        try:
                            vmap[value_id] = phi
                        except Exception:
                            pass
                        result = phi
                except Exception:
                    # Fallback to zero on any error to keep IR generation robust in dev
                    result = ir.Constant(self.i64, 0)
        
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
        trace_phi(f"[resolve] end_i64 enter: bb{block_id} v{value_id}")
        key = (block_id, value_id)
        if key in self._end_i64_cache:
            return self._end_i64_cache[key]
        if _vis is None:
            _vis = set()
        if key in _vis:
            trace_phi(f"[resolve] cycle detected at end_i64(bb{block_id}, v{value_id}) → 0")
            return ir.Constant(self.i64, 0)
        _vis.add(key)

        # Do not synthesize PHIs here. Placeholders are created in the function prepass.

        # If present in snapshot, coerce there
        snap = block_end_values.get(block_id, {})
        if value_id in snap and snap[value_id] is not None:
            val = snap[value_id]
            is_phi_val = False
            try:
                is_phi_val = hasattr(val, 'add_incoming')
            except Exception:
                is_phi_val = False
            try:
                ty = 'phi' if is_phi_val else ('ptr' if hasattr(val, 'type') and isinstance(val.type, ir.PointerType) else ('i'+str(getattr(val.type,'width','?')) if hasattr(val,'type') and isinstance(val.type, ir.IntType) else 'other'))
                trace_phi(f"[resolve]  snap hit: bb{block_id} v{value_id} type={ty}")
            except Exception:
                pass
            if is_phi_val:
                # Accept PHI only when it belongs to the same block (dominates end-of-block).
                try:
                    belongs_here = (getattr(getattr(val, 'basic_block', None), 'name', b'').decode() if hasattr(getattr(val, 'basic_block', None), 'name') else str(getattr(getattr(val, 'basic_block', None), 'name', ''))) == f"bb{block_id}"
                except Exception:
                    belongs_here = False
                if belongs_here:
                    self._end_i64_cache[key] = val
                    return val
                # Otherwise ignore and try predecessors to avoid self-carry from foreign PHI
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

        preds_s = ','.join(str(x) for x in pred_ids)
        trace_phi(f"[resolve] end_i64 miss: bb{block_id} v{value_id} preds=[{preds_s}] → 0")
        z = ir.Constant(self.i64, 0)
        self._end_i64_cache[key] = z
        return z

    def _coerce_in_block_to_i64(self, val: Any, block_id: int, bb_map: Optional[Dict[int, ir.Block]]) -> ir.Value:
        """Ensure a value is available as i64 at the end of the given block by inserting casts/boxing there."""
        if hasattr(val, 'type') and isinstance(val.type, ir.IntType):
            # If already i64, avoid re-materializing in predecessor block.
            # Using a value defined in another block inside pred may violate dominance (e.g., self-referential PHIs).
            if val.type.width == 64:
                return val
            # Otherwise, extend/truncate in predecessor block just before the terminator.
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
