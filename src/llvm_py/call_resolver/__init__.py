"""
call_resolver - Clean call argument resolution module (Box Theory)

Single Source of Truth for all LLVM call argument resolution in mir_call.py.

Public API:
- CallArgResolverBox: Main resolver (境界 = single boundary)
- TypeCoercerBox: Type conversion
- CallArgTracerBox: Debug/observability
- ArgResolveContext, ArgResolveResult: Explicit types

Box Theory Principles:
1. "箱にする": CallArgResolverBox centralizes all resolution state
2. "境界を作る": Clean public API, single entry point per concern
3. "戻せる": N/A - clean implementation only
4. "見える化": CallArgTracerBox provides observability
5. "Fail-Fast": ArgResolveResult makes failures explicit

Usage Example:
    from call_resolver import (
        CallArgResolverBox,
        TypeCoercerBox,
        ArgResolveContext
    )

    # Create context once per function call
    context = ArgResolveContext(
        builder=builder,
        vmap=vmap,
        resolver=resolver,  # Legacy (optional)
        owner=owner,
        module=module
    )

    # Create resolver (builds vmap registry once)
    arg_resolver = CallArgResolverBox(context)
    type_coercer = TypeCoercerBox(builder, module)

    # Resolve arguments (reuses registry)
    arg0 = arg_resolver.resolve_i64_or_zero(args[0])
    arg1 = arg_resolver.resolve_i64_or_zero(args[1])

    # Coerce types if needed
    arg0_i64 = type_coercer.to_i64_handle(arg0, vid=args[0])
    arg1_i64 = type_coercer.to_i64_handle(arg1, vid=args[1])

Migration from old code:
    # OLD (160 lines duplicated 8 times):
    def lower_some_call(...):
        def _resolve_arg(vid: int):  # ← 20 lines
            if resolver and hasattr(resolver, 'resolve_i64'):
                try:
                    return resolver.resolve_i64(...)
                except:
                    pass
            return vmap.get(vid)
        arg0 = _resolve_arg(args[0])
        ...

    # NEW (3 lines):
    from call_resolver import CallArgResolverBox, ArgResolveContext

    def lower_some_call(...):
        ctx = ArgResolveContext(builder, vmap, resolver, owner)
        arg_resolver = CallArgResolverBox(ctx)
        arg0 = arg_resolver.resolve_i64_or_zero(args[0])
        ...
"""

from .types import (
    ArgResolveContext,
    ArgResolveResult,
    ArgResolveFailureReason
)
from .tracer import CallArgTracerBox
from .resolver import CallArgResolverBox
from .coercer import TypeCoercerBox

__all__ = [
    # Types
    'ArgResolveContext',
    'ArgResolveResult',
    'ArgResolveFailureReason',

    # Boxes
    'CallArgTracerBox',
    'CallArgResolverBox',
    'TypeCoercerBox',
]

# Module version
__version__ = '1.0.0'
