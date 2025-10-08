"""
vmap/resolver.py - Unified resolution logic

Box Theory Principle: "境界を作る" - Single resolution point, Fail-Fast
"""
from typing import Optional, Any
try:
    from llvmlite import ir
except ImportError:
    ir = None  # For type checking when llvmlite not available

from .types import ResolveContext, ResolveResult, ResolveFailureReason
from .registry import VmapRegistryBox
from .tracer import VmapTracerBox


class VmapResolverBox:
    """
    Unified resolution point for all vmap lookups (Box Theory: single boundary)

    Resolution tiers (in order):
    1. VmapRegistryBox (block scope → global scope)
    2. PhiDispatchPoint (copy chain, predecessors, block_end_values)
    3. Future: Additional resolution strategies

    Fail-Fast: No silent exceptions, explicit ResolveResult
    """

    def __init__(self, registry: VmapRegistryBox, tracer: Optional[VmapTracerBox] = None):
        self.registry = registry
        self.tracer = tracer or VmapTracerBox()
        self._phi_dispatch = None  # Lazy-loaded to avoid circular import

    def _get_phi_dispatch(self):
        """Lazy-load PhiDispatchPoint to avoid circular import"""
        if self._phi_dispatch is None:
            try:
                from ..dispatch.phi_dispatch import PhiDispatchPoint
                self._phi_dispatch = PhiDispatchPoint
            except ImportError:
                self._phi_dispatch = None
        return self._phi_dispatch

    def resolve_i64(self, vid: int, context: ResolveContext) -> ResolveResult:
        """
        Resolve vid to i64 value with explicit result type

        Args:
            vid: Value ID to resolve
            context: Resolution context (builder, block, preds, etc.)

        Returns:
            ResolveResult.Success(value) - Resolution succeeded
            ResolveResult.Failure(reason, diagnostics) - Explicit failure
        """
        # Validation
        if vid is None or vid < 0:
            return ResolveResult.Failure(
                ResolveFailureReason.INVALID_VID,
                {"vid": vid}
            )

        self.tracer.start_resolve(vid, context)

        # Tier 1: VmapRegistryBox (fast path)
        self.tracer.attempt("registry", vid, f"block={self.registry.get_current_block()}")
        value = self.registry.resolve(vid)
        if value is not None:
            self.tracer.success("registry", vid, value)
            coerced = self._coerce_i64(context.builder, value)
            return ResolveResult.Success(coerced)

        self.tracer.failure("registry", vid, "not found in registry")

        # Tier 2: PhiDispatchPoint (copy chain, predecessors)
        PhiDispatchPoint = self._get_phi_dispatch()
        if PhiDispatchPoint and context.preds is not None:
            self.tracer.attempt("phi_dispatch", vid, "resolving via PhiDispatchPoint")
            try:
                value = PhiDispatchPoint.resolve_i64(
                    builder=context.builder,
                    resolver=self,  # Pass self for recursive resolution
                    vid=vid,
                    block=context.block,
                    preds=context.preds,
                    block_end_values=context.block_end_values,
                    vmap=self.registry.get_global_vmap(),
                    bb_map=context.bb_map
                )
                self.tracer.success("phi_dispatch", vid, value)
                return ResolveResult.Success(value)
            except Exception as e:
                # DON'T swallow - convert to explicit failure
                self.tracer.failure("phi_dispatch", vid, str(e))
                return ResolveResult.Failure(
                    ResolveFailureReason.PHI_DISPATCH_FAILED,
                    {
                        "vid": vid,
                        "exception": str(e),
                        "exception_type": type(e).__name__,
                        "available_vids": self.registry.get_available_vids(),
                        "block": context.block.name if context.block else None
                    }
                )

        # All tiers failed
        diagnostics = {
            "vid": vid,
            "available_vids": self.registry.get_available_vids(),
            "block": context.block.name if context.block else None,
            "current_block": self.registry.get_current_block()
        }
        self.tracer.end_failure(vid, ResolveFailureReason.NOT_FOUND_IN_REGISTRY, diagnostics)
        return ResolveResult.Failure(ResolveFailureReason.NOT_FOUND_IN_REGISTRY, diagnostics)

    def _coerce_i64(self, builder, value: Any):
        """
        Coerce value to i64 type

        Handles:
        - ptr → i64 (ptrtoint)
        - i1/i8/i16/i32 → i64 (zext)
        - i64 → i64 (pass through)
        """
        if ir is None:
            return value  # No llvmlite available

        i64 = ir.IntType(64)

        # Already i64
        if hasattr(value, 'type') and isinstance(value.type, ir.IntType):
            if value.type.width == 64:
                return value
            # Smaller int → zext to i64
            elif value.type.width < 64:
                return builder.zext(value, i64, name=f"zext_to_i64")

        # Pointer → i64
        if hasattr(value, 'type') and isinstance(value.type, ir.PointerType):
            return builder.ptrtoint(value, i64, name=f"ptr_to_i64")

        # Unknown type - pass through and let LLVM handle it
        return value

    def resolve_value(self, vid: int, context: ResolveContext) -> ResolveResult:
        """
        Resolve vid to value (any type, no i64 coercion)

        This is for non-integer values (pointers, structs, etc.)
        """
        if vid is None or vid < 0:
            return ResolveResult.Failure(
                ResolveFailureReason.INVALID_VID,
                {"vid": vid}
            )

        # Simple registry lookup (no PhiDispatchPoint for non-i64)
        value = self.registry.resolve(vid)
        if value is not None:
            return ResolveResult.Success(value)

        return ResolveResult.Failure(
            ResolveFailureReason.NOT_FOUND_IN_REGISTRY,
            {
                "vid": vid,
                "available_vids": self.registry.get_available_vids()
            }
        )
