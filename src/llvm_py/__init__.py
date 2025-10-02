"""Top-level package for Nyash/Hakorune Python backends.

Subpackages:
  - pyvm: Python MIR interpreter (PyVM)
  - instructions/*: llvmlite lowering helpers (AOT harness)

On import, brand environment aliases are applied to support HAKO_/HAKORUNE_
variables while preserving legacy NYASH_* readers.
"""

try:
    from .utils.brand import alias_prefixes_bootstrap as _brand_alias
    _brand_alias()
except Exception:
    pass
