"""
Build context for instruction lowering and helpers.

This structure aggregates frequently passed references so call sites can
remain concise as we gradually refactor instruction signatures.
"""

from dataclasses import dataclass
from typing import Any, Dict, List, Optional
import llvmlite.ir as ir

@dataclass
class BuildCtx:
    # Core IR handles
    module: ir.Module
    i64: ir.IntType
    i32: ir.IntType
    i8: ir.IntType
    i1: ir.IntType
    i8p: ir.PointerType

    # SSA maps and CFG
    vmap: Dict[int, ir.Value]
    bb_map: Dict[int, ir.Block]
    preds: Dict[int, List[int]]
    block_end_values: Dict[int, Dict[int, ir.Value]]

    # Resolver (value queries, casts, string-ish tags)
    resolver: Any

    # Optional diagnostics toggles (read from env by the builder)
    trace_phi: bool = False
    verbose: bool = False

