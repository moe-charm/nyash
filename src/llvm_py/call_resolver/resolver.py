"""
call_resolver/resolver.py - Unified argument resolution logic

Box Theory Principle: "境界を作る" - Single resolution point, Fail-Fast
"""
from typing import Optional, Any
try:
    from llvmlite import ir
except ImportError:
    ir = None  # For type checking when llvmlite not available

from .types import (
    ArgResolveContext,
    ArgResolveResult,
    ArgResolveFailureReason
)
from .tracer import CallArgTracerBox


class CallArgResolverBox:
    """
    Unified resolution point for all call argument lookups (Box Theory: SSOT)

    This replaces the 8 copies of _resolve_arg() scattered across mir_call.py.

    Resolution strategy (in order):
    1. Direct vmap lookup (fast path)
    2. VmapResolverBox (for PHI nodes and complex cases)
    3. Fail-Fast with explicit error

    Key improvements over old code:
    - NO silent exceptions (Fail-Fast design)
    - NO dual resolution paths (single path via vmap/)
    - NO registry rebuild per call (lazy initialization)
    - NO string comparison hacks (type-safe detection)
    """

    def __init__(self, context: ArgResolveContext, tracer: Optional[CallArgTracerBox] = None):
        self.context = context
        self.tracer = tracer or CallArgTracerBox()
        self._vmap_resolver = None  # Lazy initialization

    def _get_vmap_resolver(self):
        """
        Lazy-load VmapResolverBox to avoid rebuilding registry per argument

        OLD CODE (buggy):
            for arg in args:
                registry = VmapRegistryBox()  # ← Rebuild every time!
                for vid, value in vmap.items():
                    registry.store(vid, value)
                resolver.resolve(arg)

        NEW CODE (efficient):
            resolver = self._get_vmap_resolver()  # ← Build once
            for arg in args:
                resolver.resolve(arg)  # ← Reuse
        """
        if self._vmap_resolver is None:
            from vmap import VmapResolverBox, VmapRegistryBox, VmapTracerBox

            # Build registry once for all arguments in this call
            registry = VmapRegistryBox()
            for vid, value in self.context.vmap.items():
                registry.store(vid, value)

            self.tracer.registry_rebuild(len(self.context.vmap))

            # Create resolver with vmap tracer (can be different from our tracer)
            vmap_tracer = VmapTracerBox()
            self._vmap_resolver = VmapResolverBox(registry, vmap_tracer)

        return self._vmap_resolver

    def resolve_i64(self, vid: int) -> ArgResolveResult:
        """
        Resolve vid to i64 value with explicit result type

        Args:
            vid: Value ID to resolve

        Returns:
            ArgResolveResult.Success(value) - Resolution succeeded
            ArgResolveResult.Failure(reason, diagnostics) - Explicit failure
        """
        # Validation
        if vid is None or vid < 0:
            return ArgResolveResult.Failure(
                ArgResolveFailureReason.INVALID_VID,
                {"vid": vid}
            )

        self.tracer.start_resolve(vid, f"vmap_size={len(self.context.vmap)}")

        # Fast path: Direct vmap lookup
        self.tracer.attempt("vmap_direct", vid)
        if vid in self.context.vmap:
            value = self.context.vmap[vid]

            # CRITICAL: Don't trust if value is None or "looks like" a fallback
            # (This catches the old bug where vmap contained PHI nodes but
            #  legacy resolver returned ir.Constant(i64, 0))
            if value is not None:
                self.tracer.success("vmap_direct", vid, value)
                return ArgResolveResult.Success(value)

        self.tracer.failure("vmap_direct", vid, "not in vmap or value is None")

        # Path 2: Use VmapResolverBox (handles PHI nodes, predecessors, etc.)
        self.tracer.attempt("vmap_resolver", vid)
        try:
            from vmap import ResolveContext

            resolver = self._get_vmap_resolver()
            vmap_ctx = ResolveContext(
                builder=self.context.builder,
                block=self.context.builder.block,
                preds=getattr(self.context.owner, 'preds', None),
                block_end_values=getattr(self.context.owner, 'block_end_values', None),
                bb_map=getattr(self.context.owner, 'bb_map', None)
            )

            result = resolver.resolve_i64(vid, vmap_ctx)
            if result.success:
                self.tracer.success("vmap_resolver", vid, result.value)
                self.tracer.final_result(vid, result)
                return ArgResolveResult.Success(result.value)

            # VmapResolverBox failed
            self.tracer.failure("vmap_resolver", vid, f"{result.reason.value}")

        except Exception as e:
            # DON'T swallow - log and convert to explicit failure
            self.tracer.failure("vmap_resolver", vid, f"exception: {e}")
            diagnostics = {
                "vid": vid,
                "exception": str(e),
                "exception_type": type(e).__name__
            }
            final_result = ArgResolveResult.Failure(
                ArgResolveFailureReason.PHI_RESOLUTION_FAILED,
                diagnostics
            )
            self.tracer.final_result(vid, final_result)
            return final_result

        # All paths failed
        diagnostics = {
            "vid": vid,
            "vmap_keys": list(self.context.vmap.keys())[:10]  # First 10 for debug
        }
        final_result = ArgResolveResult.Failure(
            ArgResolveFailureReason.NOT_FOUND_IN_VMAP,
            diagnostics
        )
        self.tracer.final_result(vid, final_result)
        return final_result

    def resolve_i64_or_zero(self, vid: int):
        """
        Resolve vid to i64 or return ir.Constant(i64, 0) as fallback

        This is a convenience method for the common pattern:
            result = resolver.resolve_i64(vid)
            value = result.value if result.success else ir.Constant(i64, 0)

        Use this when you want graceful degradation instead of Fail-Fast.
        """
        result = self.resolve_i64(vid)
        if result.success:
            return result.value

        # Fallback to zero constant
        if ir is not None:
            return ir.Constant(ir.IntType(64), 0)
        return None

    def resolve_value(self, vid: int) -> ArgResolveResult:
        """
        Resolve vid to value (any type, no i64 coercion)

        This is for cases where you need the raw value without type conversion.
        """
        if vid is None or vid < 0:
            return ArgResolveResult.Failure(
                ArgResolveFailureReason.INVALID_VID,
                {"vid": vid}
            )

        # Simple direct lookup (no PHI dispatch for non-i64 types)
        if vid in self.context.vmap:
            value = self.context.vmap[vid]
            if value is not None:
                return ArgResolveResult.Success(value)

        return ArgResolveResult.Failure(
            ArgResolveFailureReason.NOT_FOUND_IN_VMAP,
            {"vid": vid, "vmap_keys": list(self.context.vmap.keys())[:10]}
        )
