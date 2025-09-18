"""
PHI wiring package

Submodules
- analysis: analyze_incomings and stringish production scan
- wiring: ensure_phi, wire_incomings, finalize_phis
- tagging: setup_phi_placeholders (predeclare + tagging + sync)

This package re-exports the primary helpers for backward compatibility with
`from phi_wiring import ...` and `from src.llvm_py import phi_wiring` usage.
"""

from .analysis import analyze_incomings, collect_produced_stringish
from .wiring import ensure_phi, wire_incomings, finalize_phis, build_succs, nearest_pred_on_path, phi_at_block_head
from .tagging import setup_phi_placeholders

# Backward-compatible aliases for tests that used private helpers
_build_succs = build_succs
_nearest_pred_on_path = nearest_pred_on_path

__all__ = [
    "analyze_incomings",
    "collect_produced_stringish",
    "ensure_phi",
    "wire_incomings",
    "finalize_phis",
    "phi_at_block_head",
    "build_succs",
    "nearest_pred_on_path",
    "setup_phi_placeholders",
    "_build_succs",
    "_nearest_pred_on_path",
]
