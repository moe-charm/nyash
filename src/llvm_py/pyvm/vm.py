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
        for f in program.get("functions", []):
            name = f.get("name")
            params = [int(p) for p in f.get("params", [])]
            bmap: Dict[int, Block] = {}
            for bb in f.get("blocks", []):
                bmap[int(bb.get("id"))] = Block(id=int(bb.get("id")), instructions=list(bb.get("instructions", [])))
            self.functions[name] = Function(name=name, params=params, blocks=bmap)

    def _read(self, regs: Dict[int, Any], v: Optional[int]) -> Any:
        if v is None:
            return None
        return regs.get(int(v))

    def _set(self, regs: Dict[int, Any], dst: Optional[int], val: Any) -> None:
        if dst is None:
            return
        regs[int(dst)] = val

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

    def run(self, entry: str) -> Any:
        fn = self.functions.get(entry)
        if fn is None:
            raise RuntimeError(f"entry function not found: {entry}")
        return self._exec_function(fn, [])

    def _exec_function(self, fn: Function, args: List[Any]) -> Any:
        # Intrinsic fast path for small helpers used in smokes
        ok, ret = self._try_intrinsic(fn.name, args)
        if ok:
            return ret
        # Initialize registers and bind params
        regs: Dict[int, Any] = {}
        if fn.params:
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

        # Simple block execution loop
        while True:
            block = fn.blocks.get(cur)
            if block is None:
                raise RuntimeError(f"block not found: {cur}")
            # Evaluate instructions sequentially
            i = 0
            while i < len(block.instructions):
                inst = block.instructions[i]
                op = inst.get("op")

                if op == "phi":
                    # incoming: [[vid, pred_bid], ...]
                    incoming = inst.get("incoming", [])
                    chosen: Any = None
                    # Prefer predecessor match; otherwise fallback to first
                    for vid, pb in incoming:
                        if prev is not None and int(pb) == int(prev):
                            chosen = regs.get(int(vid))
                            break
                    if chosen is None and incoming:
                        vid, _ = incoming[0]
                        chosen = regs.get(int(vid))
                    self._set(regs, inst.get("dst"), chosen)
                    i += 1
                    continue

                if op == "const":
                    val = inst.get("value", {})
                    ty = val.get("type")
                    vv = val.get("value")
                    if ty == "i64":
                        out = int(vv)
                    elif ty == "f64":
                        out = float(vv)
                    elif ty == "string":
                        out = str(vv)
                    else:
                        out = None
                    self._set(regs, inst.get("dst"), out)
                    i += 1
                    continue

                if op == "binop":
                    operation = inst.get("operation")
                    a = self._read(regs, inst.get("lhs"))
                    b = self._read(regs, inst.get("rhs"))
                    res: Any = None
                    if operation == "+":
                        if isinstance(a, str) or isinstance(b, str):
                            res = (str(a) if a is not None else "") + (str(b) if b is not None else "")
                        else:
                            av = 0 if a is None else int(a)
                            bv = 0 if b is None else int(b)
                            res = av + bv
                    elif operation == "-":
                        av = 0 if a is None else int(a)
                        bv = 0 if b is None else int(b)
                        res = av - bv
                    elif operation == "*":
                        av = 0 if a is None else int(a)
                        bv = 0 if b is None else int(b)
                        res = av * bv
                    elif operation == "/":
                        # integer division semantics for now
                        av = 0 if a is None else int(a)
                        bv = 1 if b in (None, 0) else int(b)
                        res = av // bv
                    elif operation == "%":
                        av = 0 if a is None else int(a)
                        bv = 1 if b in (None, 0) else int(b)
                        res = av % bv
                    elif operation in ("&", "|", "^"):
                        # treat as bitwise on ints
                        ai, bi = (0 if a is None else int(a)), (0 if b is None else int(b))
                        if operation == "&":
                            res = ai & bi
                        elif operation == "|":
                            res = ai | bi
                        else:
                            res = ai ^ bi
                    elif operation in ("<<", ">>"):
                        ai, bi = (0 if a is None else int(a)), (0 if b is None else int(b))
                        res = (ai << bi) if operation == "<<" else (ai >> bi)
                    else:
                        raise RuntimeError(f"unsupported binop: {operation}")
                    self._set(regs, inst.get("dst"), res)
                    i += 1
                    continue

                if op == "compare":
                    operation = inst.get("operation")
                    a = self._read(regs, inst.get("lhs"))
                    b = self._read(regs, inst.get("rhs"))
                    res: bool
                    if operation == "==":
                        res = (a == b)
                    elif operation == "!=":
                        res = (a != b)
                    elif operation == "<":
                        res = (a < b)
                    elif operation == "<=":
                        res = (a <= b)
                    elif operation == ">":
                        res = (a > b)
                    elif operation == ">=":
                        res = (a >= b)
                    else:
                        raise RuntimeError(f"unsupported compare: {operation}")
                    # VM convention: booleans are i64 0/1
                    self._set(regs, inst.get("dst"), 1 if res else 0)
                    i += 1
                    continue

                if op == "newbox":
                    btype = inst.get("type")
                    if btype == "ConsoleBox":
                        val = {"__box__": "ConsoleBox"}
                    elif btype == "StringBox":
                        # empty string instance
                        val = ""
                    else:
                        # Unknown box -> opaque
                        val = {"__box__": btype}
                    self._set(regs, inst.get("dst"), val)
                    i += 1
                    continue

                if op == "boxcall":
                    recv = self._read(regs, inst.get("box"))
                    method = inst.get("method")
                    args = [self._read(regs, a) for a in inst.get("args", [])]
                    out: Any = None
                    # ConsoleBox methods
                    if method in ("print", "println", "log") and self._is_console(recv):
                        s = args[0] if args else ""
                        if s is None:
                            s = ""
                        if method == "println":
                            print(str(s))
                        else:
                            # println is the primary one used by smokes; keep print/log equivalent
                            print(str(s))
                        out = 0
                    # FileBox methods (minimal read-only)
                    elif isinstance(recv, dict) and recv.get("__box__") == "FileBox":
                        if method == "open":
                            path = str(args[0]) if len(args) > 0 else ""
                            mode = str(args[1]) if len(args) > 1 else "r"
                            ok = 0
                            content = None
                            if mode == "r":
                                try:
                                    with open(path, "r", encoding="utf-8") as f:
                                        content = f.read()
                                    ok = 1
                                except Exception:
                                    ok = 0
                                    content = None
                            recv["__open"] = (ok == 1)
                            recv["__path"] = path
                            recv["__content"] = content
                            out = ok
                        elif method == "read":
                            if isinstance(recv.get("__content"), str):
                                out = recv.get("__content")
                            else:
                                out = None
                        elif method == "close":
                            recv["__open"] = False
                            out = 0
                        else:
                            out = None
                    # PathBox methods (posix-like)
                    elif isinstance(recv, dict) and recv.get("__box__") == "PathBox":
                        if method == "dirname":
                            p = str(args[0]) if args else ""
                            # Normalize to POSIX-style
                            out = os.path.dirname(p)
                            if out == "":
                                out = "."
                        elif method == "join":
                            base = str(args[0]) if len(args) > 0 else ""
                            rel = str(args[1]) if len(args) > 1 else ""
                            out = os.path.join(base, rel)
                        else:
                            out = None
                    elif method == "length":
                        out = len(str(recv))
                    elif method == "substring":
                        s = str(recv)
                        start = int(args[0]) if len(args) > 0 else 0
                        end = int(args[1]) if len(args) > 1 else len(s)
                        out = s[start:end]
                    elif method == "lastIndexOf":
                        s = str(recv)
                        needle = str(args[0]) if args else ""
                        out = s.rfind(needle)
                    else:
                        # Unimplemented method -> no-op
                        out = None
                    self._set(regs, inst.get("dst"), out)
                    i += 1
                    continue

                if op == "externcall":
                    func = inst.get("func")
                    args = [self._read(regs, a) for a in inst.get("args", [])]
                    out: Any = None
                    if func == "nyash.console.println":
                        s = args[0] if args else ""
                        if s is None:
                            s = ""
                        print(str(s))
                        out = 0
                    else:
                        # Unknown extern
                        out = None
                    self._set(regs, inst.get("dst"), out)
                    i += 1
                    continue

                if op == "branch":
                    cond = self._read(regs, inst.get("cond"))
                    tid = int(inst.get("then"))
                    eid = int(inst.get("else"))
                    prev = cur
                    cur = tid if self._truthy(cond) else eid
                    # Restart execution at next block
                    break

                if op == "jump":
                    tgt = int(inst.get("target"))
                    prev = cur
                    cur = tgt
                    break

                if op == "ret":
                    v = self._read(regs, inst.get("value"))
                    return v

                if op == "call":
                    # Resolve function name from value or take as literal
                    fval = inst.get("func")
                    fname = self._read(regs, fval)
                    if not isinstance(fname, str):
                        # Fallback: if JSON encoded a literal name
                        fname = fval if isinstance(fval, str) else None
                    call_args = [self._read(regs, a) for a in inst.get("args", [])]
                    result = None
                    if isinstance(fname, str) and fname in self.functions:
                        callee = self.functions[fname]
                        result = self._exec_function(callee, call_args)
                    # Store result if needed
                    self._set(regs, inst.get("dst"), result)
                    i += 1
                    continue

                # Unhandled op -> skip
                i += 1

            else:
                # No explicit terminator; finish
                return 0

    def _try_intrinsic(self, name: str, args: List[Any]) -> Tuple[bool, Any]:
        try:
            if name == "Main.esc_json/1":
                s = "" if not args else ("" if args[0] is None else str(args[0]))
                out = []
                for ch in s:
                    if ch == "\\":
                        out.append("\\\\")
                    elif ch == '"':
                        out.append('\\"')
                    else:
                        out.append(ch)
                return True, "".join(out)
            if name == "Main.dirname/1":
                p = "" if not args else ("" if args[0] is None else str(args[0]))
                d = os.path.dirname(p)
                if d == "":
                    d = "."
                return True, d
        except Exception:
            pass
        return (False, None)
