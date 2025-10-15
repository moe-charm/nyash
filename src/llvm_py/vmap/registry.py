"""
vmap/registry.py - Single Source of Truth for vmap management

Box Theory Principle: "箱にする" - Centralize all vmap state in one box
"""
from typing import Dict, Optional, Any
from .tracer import VmapTracerBox


class VmapRegistryBox:
    """
    Single Source of Truth for all vmap operations (Box Theory: SSOT)

    Manages two scopes:
    - Global vmap: Persistent across blocks (PHI values, cross-block refs)
    - Block vmaps: Per-block local values (copies, temporaries)
    """

    def __init__(self, tracer: Optional[VmapTracerBox] = None):
        self._global_vmap: Dict[int, Any] = {}  # Persistent vmap
        self._block_vmaps: Dict[str, Dict[int, Any]] = {}  # Per-block scopes
        self._current_block: Optional[str] = None
        self.tracer = tracer or VmapTracerBox()

    def set_block_scope(self, block_name: str):
        """
        Switch to a new block scope

        Creates a new per-block vmap and copies relevant PHI values from global vmap.
        """
        self._current_block = block_name

        if block_name not in self._block_vmaps:
            # Initialize block vmap with PHI values from global scope
            self._block_vmaps[block_name] = self._copy_phi_values_to_block(block_name)

    def _copy_phi_values_to_block(self, block_name: str) -> Dict[int, Any]:
        """
        Copy PHI values from global vmap to block vmap

        PHI values defined in this block need to be accessible in block scope.
        """
        block_vmap = {}

        for vid, value in self._global_vmap.items():
            # Check if this is a PHI value belonging to this block
            if self._is_phi_value_for_block(value, block_name):
                block_vmap[vid] = value

        return block_vmap

    def _is_phi_value_for_block(self, value: Any, block_name: str) -> bool:
        """Check if value is a PHI node belonging to the given block"""
        # PHI nodes have 'add_incoming' method and 'basic_block' attribute
        if hasattr(value, 'add_incoming'):
            bb = getattr(value, 'basic_block', None)
            if bb:
                return getattr(bb, 'name', None) == block_name
        return False

    def store(self, vid: int, value: Any, scope: str = "auto"):
        """
        Store a value in appropriate scope

        Args:
            vid: Value ID
            value: LLVM IR value
            scope: "global", "block", or "auto" (default)
                   "auto" detects PHI values and stores appropriately
        """
        if scope == "auto":
            # PHI values go to both global and block scope
            if hasattr(value, 'add_incoming'):
                self._global_vmap[vid] = value
                if self._current_block:
                    self._block_vmaps.setdefault(self._current_block, {})[vid] = value
            # Regular values go to current block scope (and global as fallback)
            else:
                if self._current_block:
                    self._block_vmaps.setdefault(self._current_block, {})[vid] = value
                # Also store in global for cross-block access
                self._global_vmap[vid] = value

        elif scope == "global":
            self._global_vmap[vid] = value

        elif scope == "block":
            if self._current_block:
                self._block_vmaps.setdefault(self._current_block, {})[vid] = value
            else:
                # No current block - fallback to global
                self._global_vmap[vid] = value

    def resolve(self, vid: int) -> Optional[Any]:
        """
        Resolve vid to value with 2-tier fallback

        Tier 1: Current block scope (fast path)
        Tier 2: Global vmap (fallback)

        Returns:
            ir.Value if found, None otherwise
        """
        # Tier 1: Block scope (fast path for local values)
        if self._current_block:
            block_vmap = self._block_vmaps.get(self._current_block, {})
            value = block_vmap.get(vid)
            if value is not None:
                return value

        # Tier 2: Global scope (cross-block references, PHI values)
        value = self._global_vmap.get(vid)
        if value is not None:
            return value

        # Not found in any scope
        return None

    def get_global_vmap(self) -> Dict[int, Any]:
        """Get global vmap (for PhiDispatchPoint integration)"""
        return self._global_vmap

    def get_block_vmap(self, block_name: Optional[str] = None) -> Dict[int, Any]:
        """Get block vmap for specific block (or current block)"""
        block = block_name or self._current_block
        return self._block_vmaps.get(block, {}) if block else {}

    def get_current_block(self) -> Optional[str]:
        """Get current block name"""
        return self._current_block

    def get_available_vids(self) -> list:
        """Get list of all available vids (for diagnostics)"""
        all_vids = set(self._global_vmap.keys())
        if self._current_block:
            all_vids.update(self._block_vmaps.get(self._current_block, {}).keys())
        return sorted(all_vids)

    def clear(self):
        """Clear all vmaps (for testing)"""
        self._global_vmap.clear()
        self._block_vmaps.clear()
        self._current_block = None
