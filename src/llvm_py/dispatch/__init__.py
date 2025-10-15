"""
Dispatch boxes for unified operand/value resolution at structural merge points.

Currently exposes PhiDispatchPoint for compare/branch/binop to share the same
fallback policy instead of each instruction reinventing heuristics.
"""

from .phi_dispatch import PhiDispatchPoint  # re-export

