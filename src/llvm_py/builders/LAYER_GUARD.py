"""
Layer Guard — builders

This file documents the allowed imports for the builders layer.
It is informational and may be enforced by static checks in the future.
"""

LAYER_NAME = "builders"
ALLOWED_IMPORTS = [
    "llvmlite",
    "src.llvm_py.instructions",
    "src.llvm_py.prepass",
    "src.llvm_py.phi_wiring",
    "src.llvm_py.cfg",
    "src.llvm_py.trace",
]
FORBIDDEN_IMPORTS = [
    "src.backend",   # Rust backends
    "src.jit",       # JIT layer
    "crates",        # Rust crates
]

