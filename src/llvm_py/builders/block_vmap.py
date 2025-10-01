"""
BlockVMap — per‑block SSA map view

Thin wrapper to unify access to the global vmap and the current block‑local
view. Adoptable incrementally: existing code can keep using dicts while new
callers migrate to this API.
"""

from __future__ import annotations
from typing import Dict, Any


class BlockVMap:
    def __init__(self, global_vmap: Dict[int, Any], local_vmap: Dict[int, Any] | None = None):
        self._global = global_vmap
        self._local = local_vmap if local_vmap is not None else global_vmap

    @classmethod
    def from_owner(cls, owner) -> "BlockVMap":
        g = getattr(owner, "vmap", {})
        l = getattr(owner, "_current_vmap", None)
        return cls(g, l if isinstance(l, dict) else None)

    def get(self, vid: int):
        return self._local.get(vid)

    def set(self, vid: int, value: Any) -> None:
        self._local[vid] = value

    def snapshot(self) -> Dict[int, Any]:
        return dict(self._local)

    def commit_to_global(self) -> None:
        if self._local is self._global:
            return
        self._global.update(self._local)

