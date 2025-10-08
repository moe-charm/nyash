"""
vmap - Clean vmap management module (Box Theory)

Single Source of Truth for all LLVM vmap operations.

Public API:
- VmapRegistryBox: vmap storage (SSOT)
- VmapResolverBox: unified resolution
- VmapTracerBox: debug/observability
- ResolveContext, ResolveResult: explicit types

Box Theory Principles:
1. "箱にする": VmapRegistryBox centralizes all vmap state
2. "境界を作る": VmapResolverBox is the single resolution point
3. "戻せる": (N/A - clean implementation only, no legacy support)
4. "見える化": VmapTracerBox provides observability
5. "Fail-Fast": ResolveResult makes failures explicit
"""

from .types import (
    ResolveContext,
    ResolveResult,
    ResolveFailureReason
)
from .tracer import VmapTracerBox
from .registry import VmapRegistryBox
from .resolver import VmapResolverBox

__all__ = [
    # Types
    'ResolveContext',
    'ResolveResult',
    'ResolveFailureReason',

    # Boxes
    'VmapTracerBox',
    'VmapRegistryBox',
    'VmapResolverBox',
]

# Module version
__version__ = '1.0.0'
