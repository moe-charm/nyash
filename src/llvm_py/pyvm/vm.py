"""
Minimal Python VM for Nyash MIR(JSON) parity with llvmlite.

Supported ops (MVP):
  - const/binop/compare/branch/jump/ret
  - phi (select by predecessor block)
  - newbox: ConsoleBox, StringBox (minimal semantics)
  - boxcall: String.length/substring/lastIndexOf, Console.print/println/log
  - externcall: nyash.console.println

Value model:
  - i64 -> Python int
  - f64 -> Python float
  - string -> Python str
  - void/null -> None
  - ConsoleBox -> {"__box__":"ConsoleBox"}
  - StringBox receiver -> Python str
"""

from __future__ import annotations
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple
import os
from .ops_core import (
    op_phi,
    op_const,
    op_binop,
    op_compare,
    op_typeop,
    op_unop,
    op_copy,
)
from .ops_box import op_newbox, op_boxcall
from .ops_ctrl import op_externcall
from .ops_flow import op_branch, op_jump, op_ret, op_call
from .intrinsic import try_intrinsic as _intrinsic_try


@dataclass
class Block:
    id: int
    instructions: List[Dict[str, Any]]


@dataclass
class Function:
    name: str
    params: List[int]
    blocks: Dict[int, Block]


class PyVM:
    def __init__(self, program: Dict[str, Any]):
        self.functions: Dict[str, Function] = {}
        self._debug = os.environ.get('NYASH_PYVM_DEBUG') in ('1','true','on')
        # Targeted trace controls (default OFF)
        self._trace_fn = os.environ.get('NYASH_PYVM_TRACE_FN')
        self._trace_reg = os.environ.get('NYASH_PYVM_TRACE_REG')  # string compare
        self._cur_fn: Optional[str] = None
        for f in program.get("functions", []):
            name = f.get("name")
            params = [int(p) for p in f.get("params", [])]
            bmap: Dict[int, Block] = {}
            for bb in f.get("blocks", []):
                bmap[int(bb.get("id"))] = Block(id=int(bb.get("id")), instructions=list(bb.get("instructions", [])))
            # Register each function inside the loop (bugfix)
            self.functions[name] = Function(name=name, params=params, blocks=bmap)

    def _dbg(self, *a):
        if self._debug:
            try:
                import sys as _sys
                print(*a, file=_sys.stderr)
            except Exception:
                pass

    def _type_name(self, v: Any) -> str:
        """Pretty type name for debug traces mapped to MIR conventions."""
        if v is None:
            return "null"
        if isinstance(v, bool):
            # Booleans are encoded as i64 0/1 in MIR
            return "i64"
        if isinstance(v, int):
            return "i64"
        if isinstance(v, float):
            return "f64"
        if isinstance(v, str):
            return "string"
        if isinstance(v, dict) and "__box__" in v:
            return f"Box({v.get('__box__')})"
        return type(v).__name__

    # --- Capability helpers (macro sandbox) ---
    def _macro_sandbox_active(self) -> bool:
        """Detect if we are running under macro sandbox.

        Heuristics:
        - Explicit flag NYASH_MACRO_SANDBOX=1
        - Macro child default envs (plugins off + macro off)
        - Any MACRO_CAP_* enabled
        """
        if os.environ.get("NYASH_MACRO_SANDBOX", "0") in ("1", "true", "on"):
            return True
        if os.environ.get("NYASH_DISABLE_PLUGINS") in ("1", "true", "on") and os.environ.get("NYASH_MACRO_ENABLE") in ("0", "false", "off"):
            return True
        if self._cap_env() or self._cap_io() or self._cap_net():
            return True
        return False

    def _cap_env(self) -> bool:
        return os.environ.get("NYASH_MACRO_CAP_ENV", "0") in ("1", "true", "on")

    def _cap_io(self) -> bool:
        return os.environ.get("NYASH_MACRO_CAP_IO", "0") in ("1", "true", "on")

    def _cap_net(self) -> bool:
        return os.environ.get("NYASH_MACRO_CAP_NET", "0") in ("1", "true", "on")

    def _read(self, regs: Dict[int, Any], v: Optional[int]) -> Any:
        if v is None:
            return None
        return regs.get(int(v))

    def _set(self, regs: Dict[int, Any], dst: Optional[int], val: Any) -> None:
        if dst is None:
            return
        rid = int(dst)
        regs[rid] = val
        try:
            if self._trace_fn and self._cur_fn == self._trace_fn:
                if self._trace_reg is None or self._trace_reg == str(rid):
                    self._dbg(f"[pyvm][set] fn={self._cur_fn} r{rid}={val}")
        except Exception:
            pass

    def _truthy(self, v: Any) -> bool:
        if isinstance(v, bool):
            return v
        if isinstance(v, (int, float)):
            return v != 0
        if isinstance(v, str):
            return len(v) != 0
        return v is not None

    def _is_console(self, v: Any) -> bool:
        return isinstance(v, dict) and v.get("__box__") == "ConsoleBox"

    def _sandbox_allow_newbox(self, box_type: str) -> bool:
        """Allow-list for constructing boxes under macro sandbox."""
        if not self._macro_sandbox_active():
            return True
        if box_type in ("ConsoleBox", "StringBox", "ArrayBox", "MapBox"):
            return True
        if box_type in ("FileBox", "PathBox", "DirBox"):
            return self._cap_io()
        # Simple net-related boxes
        if box_type in ("HTTPBox", "HttpBox", "SocketBox"):
            return self._cap_net()
        # Unknown boxes are denied in sandbox
        return False

    def _sandbox_allow_boxcall(self, recv: Any, method: Optional[str]) -> bool:
        if not self._macro_sandbox_active():
            return True
        # Console methods are fine
        if self._is_console(recv):
            return True
        # String methods (our VM treats StringBox receiver as Python str)
        if isinstance(recv, str):
            return method in ("length", "substring", "lastIndexOf", "indexOf")
        # File/Path/Dir need IO cap
        if isinstance(recv, dict) and recv.get("__box__") in ("FileBox", "PathBox", "DirBox"):
            return self._cap_io()
        # Other boxes are denied in sandbox
        return False

    def run(self, entry: str) -> Any:
        fn = self.functions.get(entry)
        if fn is None:
            raise RuntimeError(f"entry function not found: {entry}")
        self._dbg(f"[pyvm] run entry={entry}")
        return self._exec_function(fn, [])

    def run_args(self, entry: str, args: list[Any]) -> Any:
        fn = self.functions.get(entry)
        if fn is None:
            raise RuntimeError(f"entry function not found: {entry}")
        self._dbg(f"[pyvm] run entry={entry} argv={args}")
        call_args = list(args)
        # If entry is a typical main (main / *.main), pack argv into an ArrayBox-like value
        # to match Nyash's `main(args)` convention regardless of param count.
        try:
            if entry == 'main' or entry.endswith('.main'):
                call_args = [{"__box__": "ArrayBox", "__arr": list(args)}]
            elif fn.params and len(fn.params) == 1:
                call_args = [{"__box__": "ArrayBox", "__arr": list(args)}]
        except Exception:
            pass
        return self._exec_function(fn, call_args)

    def _exec_function(self, fn: Function, args: List[Any]) -> Any:
        self._cur_fn = fn.name
        self._dbg(f"[pyvm] call {fn.name} args={args}")
        # Intrinsic fast path for small helpers used in smokes
        ok, ret = self._try_intrinsic(fn.name, args)
        if ok:
            return ret
        # Initialize registers and bind params
        regs: Dict[int, Any] = {}
        if fn.params:
            # If this function was lowered from a method (e.g., Main.foo/N), the first
            # parameter is an implicit 'me' and call sites pass only N args.
            # Align by detecting off-by-one and shifting args to skip the implicit receiver.
            if len(args) + 1 == len(fn.params):
                # Fill implicit 'me' (unused by our lowering at runtime) and map the rest
                if fn.params:
                    regs[int(fn.params[0])] = None  # placeholder for 'me'
                for i, pid in enumerate(fn.params[1:]):
                    regs[int(pid)] = args[i] if i < len(args) else None
            else:
                # Direct positional bind
                for i, pid in enumerate(fn.params):
                    regs[int(pid)] = args[i] if i < len(args) else None
        else:
            # Heuristic: derive param count from name suffix '/N' and bind to vids 0..N-1
            n = 0
            if "/" in fn.name:
                try:
                    n = int(fn.name.split("/")[-1])
                except Exception:
                    n = 0
            for i in range(n):
                regs[i] = args[i] if i < len(args) else None
        # Choose a deterministic first block (lowest id)
        if not fn.blocks:
            return 0
        cur = min(fn.blocks.keys())
        prev: Optional[int] = None

        # Simple block execution loop with step budget to avoid infinite hangs
        max_steps = 0
        try:
            max_steps = int(os.environ.get("NYASH_PYVM_MAX_STEPS", "200000"))
        except Exception:
            max_steps = 200000
        steps = 0
        while True:
            steps += 1
            if max_steps and steps > max_steps:
                raise RuntimeError(f"pyvm: max steps exceeded ({max_steps}) in function {fn.name}")
            block = fn.blocks.get(cur)
            if block is None:
                raise RuntimeError(f"block not found: {cur}")
            # Evaluate instructions sequentially
            i = 0
            while i < len(block.instructions):
                inst = block.instructions[i]
                op = inst.get("op")

                if op == "phi":
                    op_phi(self, inst, regs, prev)
                    i += 1
                    continue

                if op == "const":
                    op_const(self, inst, regs)
                    i += 1
                    continue

                if op == "binop":
                    op_binop(self, inst, regs)
                    i += 1
                    continue

                if op == "compare":
                    op_compare(self, inst, regs)
                    i += 1
                    continue

                if op == "typeop":
                    op_typeop(self, inst, regs)
                    i += 1
                    continue

                if op == "unop":
                    op_unop(self, inst, regs)
                    i += 1
                    continue

                if op == "newbox":
                    op_newbox(self, inst, regs)
                    i += 1
                    continue

                if op == "copy":
                    op_copy(self, inst, regs)
                    i += 1
                    continue

                if op == "boxcall":
                    op_boxcall(self, fn, inst, regs)
                    i += 1
                    continue

                if op == "externcall":
                    op_externcall(self, inst, regs)
                    i += 1
                    continue

                if op == "branch":
                    prev, cur = op_branch(self, inst, regs, cur, prev)
                    break

                if op == "jump":
                    prev, cur = op_jump(self, inst, regs, cur, prev)
                    break

                if op == "ret":
                    return op_ret(self, inst, regs)

                if op == "call":
                    result = op_call(self, fn, inst, regs)
                    self._set(regs, inst.get("dst"), result)
                    i += 1
                    continue

                # Unhandled op -> skip
                i += 1

            else:
                # No explicit terminator; finish
                return 0

    def _try_intrinsic(self, name: str, args: List[Any]) -> Tuple[bool, Any]:
        return _intrinsic_try(name, args)
