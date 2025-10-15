"""
call_resolver/types.py - Type definitions for call argument resolution

Box Theory Principle: Explicit types for Fail-Fast design
"""
from dataclasses import dataclass
from typing import Optional, Any, Dict
from enum import Enum


class ArgResolveFailureReason(Enum):
    """Explicit failure reasons for argument resolution (Fail-Fast)"""
    INVALID_VID = "invalid_vid"
    NOT_FOUND_IN_VMAP = "not_found_in_vmap"
    PHI_RESOLUTION_FAILED = "phi_resolution_failed"
    TYPE_COERCION_FAILED = "type_coercion_failed"
    LEGACY_RESOLVER_FAILED = "legacy_resolver_failed"


@dataclass
class ArgResolveContext:
    """
    Context bundle for argument resolution (Box Theory: 箱にする)

    This encapsulates all the state needed for resolving call arguments,
    avoiding parameter explosion in function signatures.
    """
    builder: Any        # ir.IRBuilder
    vmap: Dict[int, Any]  # Current vmap
    resolver: Any       # Legacy resolver (optional, for backward compat)
    owner: Any          # Builder instance (has preds, block_end_values, bb_map)

    # Optional context
    module: Optional[Any] = None  # ir.Module (for type coercion)


@dataclass
class ArgResolveResult:
    """
    Explicit result type for argument resolution (Fail-Fast design)

    Never returns None or throws exceptions silently. All failures are explicit.
    """
    success: bool
    value: Optional[Any] = None
    reason: Optional[ArgResolveFailureReason] = None
    diagnostics: Optional[Dict[str, Any]] = None

    @classmethod
    def Success(cls, value: Any):
        """Create successful result"""
        return cls(success=True, value=value)

    @classmethod
    def Failure(cls, reason: ArgResolveFailureReason, diagnostics: Optional[Dict] = None):
        """Create failure result with explicit reason"""
        return cls(success=False, reason=reason, diagnostics=diagnostics or {})

    def unwrap_or(self, default: Any) -> Any:
        """Unwrap value or return default (convenience method)"""
        return self.value if self.success else default
