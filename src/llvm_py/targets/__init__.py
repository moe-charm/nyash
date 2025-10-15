"""
LLVM Targets Module
箱理論実践: ターゲット抽象化で責任分離

Usage:
    from targets import WasmTarget, NativeTarget

    # WASM compilation
    target = WasmTarget()
    target.emit_object(module, "output.wasm")

    # Native compilation
    target = NativeTarget()
    target.emit_object(module, "output.o")
"""

from .base import BaseTarget
from .wasm import WasmTarget
from .native import NativeTarget
from .windows import WindowsTarget

__all__ = ['BaseTarget', 'WasmTarget', 'NativeTarget']


def create_target(target_name: str) -> BaseTarget:
    """
    Factory function to create target instance

    Args:
        target_name: "wasm32" or "native" or "windows"

    Returns:
        BaseTarget instance

    Examples:
        >>> target = create_target("wasm32")
        >>> isinstance(target, WasmTarget)
        True
    """
    if target_name == "wasm32":
        return WasmTarget()
    elif target_name == "native":
        return NativeTarget()
    elif target_name in ("windows", "win64", "x86_64-pc-windows-msvc"):
        return WindowsTarget()
    else:
        raise ValueError(f"Unknown target: {target_name}. Use 'wasm32' or 'native' or 'windows'.")
