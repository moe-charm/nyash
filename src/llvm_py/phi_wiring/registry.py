"""
PhiRegistry — Box for Single-Source PHI Management

Responsibilities
- Ensure at most one PHI placeholder exists per (block_id, dst_vid)
- Centralize PHI creation at block head and registration into builder maps
- Provide lookup helpers for other modules (PhiHandler, wiring.ensure_phi)

Contract
- PHIs are created only at basic block head (grouped at top)
- Registered in both `builder.vmap[dst]` and optional `builder._current_vmap[dst]`
- Identity: keyed by tuple (int(block_id), int(dst_vid))

Non-goals
- This box does not wire incoming edges (see wiring.finalize_phis)
- This box does not synthesize default incoming (handled by wiring)
"""

from __future__ import annotations
from typing import Optional, Tuple

import llvmlite.ir as ir


class PhiRegistry:
    """Simple per-builder registry for PHI placeholders."""

    @staticmethod
    def _reg(builder) -> dict:
        try:
            reg = getattr(builder, 'phi_registry', None)
        except Exception:
            reg = None
        if reg is None:
            reg = {}
            try:
                builder.phi_registry = reg
            except Exception:
                pass
        return reg

    @staticmethod
    def key(block_id: int, dst_vid: int) -> Tuple[int, int]:
        return (int(block_id), int(dst_vid))

    @staticmethod
    def get(builder, block_id: int, dst_vid: int) -> Optional[ir.Instruction]:
        reg = PhiRegistry._reg(builder)
        return reg.get(PhiRegistry.key(block_id, dst_vid))

    @staticmethod
    def register(builder, block_id: int, dst_vid: int, phi: ir.Instruction) -> None:
        reg = PhiRegistry._reg(builder)
        reg[PhiRegistry.key(block_id, dst_vid)] = phi
        # Keep vmap in sync for downstream lookups
        try:
            builder.vmap[int(dst_vid)] = phi
        except Exception:
            pass
        try:
            if hasattr(builder, '_current_vmap') and isinstance(builder._current_vmap, dict):
                builder._current_vmap[int(dst_vid)] = phi
        except Exception:
            pass

    @staticmethod
    def ensure(builder, block_id: int, dst_vid: int, bb: ir.Block) -> ir.Instruction:
        """Return the unique PHI object for (block_id, dst_vid), creating at block head if missing."""
        # 1) Registry first
        ph = PhiRegistry.get(builder, block_id, dst_vid)
        if ph is not None:
            return ph
        # 2) Existing vmap PHI that already belongs to this block
        try:
            cur = builder.vmap.get(int(dst_vid))
        except Exception:
            cur = None
        try:
            if cur is not None and hasattr(cur, 'add_incoming'):
                bb_of = getattr(getattr(cur, 'basic_block', None), 'name', None)
                if bb_of == bb.name:
                    PhiRegistry.register(builder, block_id, dst_vid, cur)
                    return cur
        except Exception:
            pass
        # 3) Create a new PHI at block head
        ib = ir.IRBuilder(bb)
        try:
            ib.position_at_start(bb)
        except Exception:
            pass
        i64 = getattr(builder, 'i64', ir.IntType(64))
        ph = ib.phi(i64, name=f"phi_{int(dst_vid)}")
        PhiRegistry.register(builder, block_id, dst_vid, ph)
        return ph


def ensure_phi(builder, block_id: int, dst_vid: int, bb: ir.Block) -> ir.Instruction:
    """Compatibility shim to match previous `phi_wiring.ensure_phi` interface."""
    return PhiRegistry.ensure(builder, block_id, dst_vid, bb)

