"""
Control/side-effect ops for the Nyash PyVM that don't affect block control
flow directly: externcall normalization. Branch/jump/ret/call stay in vm.py.
"""
from __future__ import annotations

from typing import Any, Dict


def op_externcall(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    func = inst.get("func")
    args = [owner._read(regs, a) for a in inst.get("args", [])]
    out: Any = None
    owner._dbg(f"[pyvm] externcall func={func} args={args}")
    # Normalize known console/debug externs
    if isinstance(func, str):
        if func in ("nyash.console.println", "nyash.console.log", "env.console.log"):
            s = args[0] if args else ""
            if s is None:
                s = ""
            print(str(s))
            out = 0
        elif func in (
            "nyash.console.warn",
            "env.console.warn",
            "nyash.console.error",
            "env.console.error",
            "nyash.debug.trace",
            "env.debug.trace",
        ):
            s = args[0] if args else ""
            if s is None:
                s = ""
            try:
                import sys as _sys
                print(str(s), file=_sys.stderr)
            except Exception:
                print(str(s))
            out = 0
        else:
            # Macro sandbox: disallow unknown externcall unless explicitly whitelisted by future caps
            out = 0
    owner._set(regs, inst.get("dst"), out)

