"""
vmap/types.py - Explicit types for vmap resolution

Box Theory Principle: "境界を作る" - Explicit context and result types
"""
from dataclasses import dataclass
from typing import Optional, Dict, Any
from enum import Enum


class ResolveFailureReason(Enum):
    """Explicit failure reasons (no silent errors!)"""
    NOT_FOUND_IN_REGISTRY = "vid not found in registry"
    NOT_FOUND_IN_BLOCK_SCOPE = "vid not found in block scope"
    PHI_DISPATCH_FAILED = "PhiDispatchPoint resolution failed"
    INVALID_VID = "invalid vid (negative or None)"
    NO_CURRENT_BLOCK = "no current block scope set"


@dataclass
class ResolveContext:
    """Context information for vmap resolution"""
    builder: Any  # ir.IRBuilder
    block: Any    # ir.Block
    preds: Optional[list] = None
    block_end_values: Optional[Dict] = None
    bb_map: Optional[Dict] = None

    def __repr__(self):
        block_name = self.block.name if self.block else "None"
        return f"ResolveContext(block={block_name}, has_preds={self.preds is not None})"


@dataclass
class ResolveResult:
    """Explicit success/failure result (Fail-Fast design)"""
    success: bool
    value: Optional[Any] = None  # ir.Value on success
    reason: Optional[ResolveFailureReason] = None
    diagnostics: Optional[Dict[str, Any]] = None

    @classmethod
    def Success(cls, value):
        """Resolution succeeded"""
        return cls(success=True, value=value)

    @classmethod
    def Failure(cls, reason: ResolveFailureReason, diagnostics: Optional[Dict] = None):
        """Resolution failed (explicit reason)"""
        return cls(success=False, reason=reason, diagnostics=diagnostics or {})

    def unwrap(self):
        """Unwrap value (raises on failure)"""
        if not self.success:
            raise RuntimeError(
                f"ResolveResult.unwrap() called on Failure: {self.reason.value}\n"
                f"Diagnostics: {self.diagnostics}"
            )
        return self.value

    def unwrap_or(self, default):
        """Unwrap value or return default"""
        return self.value if self.success else default

    def __repr__(self):
        if self.success:
            return f"ResolveResult.Success({self.value})"
        else:
            return f"ResolveResult.Failure({self.reason.value})"
