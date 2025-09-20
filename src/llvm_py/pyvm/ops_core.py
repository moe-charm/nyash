"""
Core operation handlers for the Nyash PyVM.

These helpers are pure functions that take the VM instance (owner),
the instruction dict, and the current register file, then update regs
via owner._set as needed. They return nothing; control flow is handled
by the caller in vm.py.
"""
from __future__ import annotations

from typing import Any, Dict, Optional


def op_phi(owner, inst: Dict[str, Any], regs: Dict[int, Any], prev: Optional[int]) -> None:
    incoming = inst.get("incoming", [])
    chosen: Any = None
    dbg = owner and getattr(owner, "_debug", False) and (owner and (owner.__class__.__name__ == "PyVM"))
    # Use dedicated env flag for phi trace (matches existing behavior)
    import os
    if os.environ.get("NYASH_PYVM_DEBUG_PHI") == "1":
        print(f"[pyvm.phi] prev={prev} incoming={incoming}")
        dbg = True
    # Prefer [vid, pred] that matches prev
    for pair in incoming:
        if not isinstance(pair, (list, tuple)) or len(pair) < 2:
            continue
        a, b = pair[0], pair[1]
        if prev is not None and int(b) == int(prev) and int(a) in regs:
            chosen = regs.get(int(a))
            if dbg:
                print(f"[pyvm.phi] case1 match: use v{a} from pred {b} -> {chosen}")
            break
    # Fallback: first resolvable vid
    if chosen is None and incoming:
        for pair in incoming:
            if not isinstance(pair, (list, tuple)) or len(pair) < 2:
                continue
            a, _b = pair[0], pair[1]
            if int(a) in regs:
                chosen = regs.get(int(a))
                break
    if os.environ.get("NYASH_PYVM_DEBUG_PHI") == "1":
        print(f"[pyvm.phi] chosen={chosen}")
    owner._set(regs, inst.get("dst"), chosen)


def op_const(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    val = inst.get("value", {})
    ty = val.get("type")
    vv = val.get("value")
    if ty == "i64":
        out = int(vv)
    elif ty == "f64":
        out = float(vv)
    elif ty == "string":
        out = str(vv)
    elif isinstance(ty, dict) and ty.get("kind") in ("handle", "ptr") and ty.get("box_type") == "StringBox":
        out = str(vv)
    else:
        out = None
    owner._set(regs, inst.get("dst"), out)


def op_binop(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    operation = inst.get("operation")
    a = owner._read(regs, inst.get("lhs"))
    b = owner._read(regs, inst.get("rhs"))
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
        av = 0 if a is None else int(a)
        bv = 1 if b in (None, 0) else int(b)
        res = av // bv
    elif operation == "%":
        av = 0 if a is None else int(a)
        bv = 1 if b in (None, 0) else int(b)
        res = av % bv
    elif operation in ("&", "|", "^"):
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
    owner._set(regs, inst.get("dst"), res)


def op_compare(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    operation = inst.get("operation")
    a = owner._read(regs, inst.get("lhs"))
    b = owner._read(regs, inst.get("rhs"))
    res: bool
    # For ordering comparisons, be robust to None by coercing to ints
    if operation in ("<", "<=", ">", ">="):
        try:
            ai = 0 if a is None else (int(a) if not isinstance(a, str) else 0)
        except Exception:
            ai = 0
        try:
            bi = 0 if b is None else (int(b) if not isinstance(b, str) else 0)
        except Exception:
            bi = 0
        if operation == "<":
            res = ai < bi
        elif operation == "<=":
            res = ai <= bi
        elif operation == ">":
            res = ai > bi
        else:
            res = ai >= bi
    elif operation == "==":
        res = (a == b)
    elif operation == "!=":
        res = (a != b)
    else:
        raise RuntimeError(f"unsupported compare: {operation}")
    owner._set(regs, inst.get("dst"), 1 if res else 0)


def op_typeop(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    # operation: "check" | "cast" ("as" is treated as cast for MVP)
    operation = inst.get("operation") or inst.get("op")
    src_vid = inst.get("src")
    dst_vid = inst.get("dst")
    target = (inst.get("target_type") or "")
    src_val = owner._read(regs, src_vid)

    def is_type(val: Any, ty: str) -> bool:
        t = (ty or "").strip()
        t = t.lower()
        # Normalize aliases
        if t in ("stringbox",):
            t = "string"
        if t in ("integerbox", "int", "i64"):
            t = "integer"
        if t in ("floatbox", "f64"):
            t = "float"
        if t in ("boolbox", "boolean"):
            t = "bool"
        # Check by Python types/our boxed representations
        if t == "string":
            return isinstance(val, str)
        if t == "integer":
            # Treat Python ints (including 0/1) as integer (bools are ints in Python; original code excluded bool)
            return isinstance(val, int) and not isinstance(val, bool)
        if t == "float":
            return isinstance(val, float)
        if t == "bool":
            # Our VM uses 0/1 ints for bool; accept 0 or 1
            return isinstance(val, int) and (val == 0 or val == 1)
        # Boxed receivers
        if t.endswith("box"):
            box_name = ty
            if isinstance(val, dict) and val.get("__box__") == box_name:
                return True
            if box_name == "StringBox" and isinstance(val, str):
                return True
            if box_name == "ConsoleBox" and owner._is_console(val):
                return True
            if box_name == "ArrayBox" and isinstance(val, dict) and val.get("__box__") == "ArrayBox":
                return True
            if box_name == "MapBox" and isinstance(val, dict) and val.get("__box__") == "MapBox":
                return True
            return False
        return False

    if (operation or "").lower() in ("check", "is"):
        out = 1 if is_type(src_val, str(target)) else 0
        owner._set(regs, dst_vid, out)
    else:
        # cast/as: MVP pass-through
        owner._set(regs, dst_vid, src_val)


def op_unop(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    kind = inst.get("kind")
    src = owner._read(regs, inst.get("src"))
    out: Any
    if kind == "neg":
        if isinstance(src, (int, float)):
            out = -src
        elif src is None:
            out = 0
        else:
            try:
                out = -int(src)
            except Exception:
                out = 0
    elif kind == "not":
        out = 0 if owner._truthy(src) else 1
    elif kind == "bitnot":
        out = ~int(src) if src is not None else -1
    else:
        out = None
    owner._set(regs, inst.get("dst"), out)


def op_copy(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    src = owner._read(regs, inst.get("src"))
    owner._set(regs, inst.get("dst"), src)
