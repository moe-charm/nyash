"""
Flow/control-related ops for Nyash PyVM: branch, jump, ret, call.
These mutate the control flow (cur/prev) or return from the function.
"""
from __future__ import annotations

from typing import Any, Dict, List, Tuple


def op_branch(owner, inst: Dict[str, Any], regs: Dict[int, Any], cur: int, prev: int | None) -> Tuple[int | None, int]:
    cond = owner._read(regs, inst.get("cond"))
    tid = int(inst.get("then"))
    eid = int(inst.get("else"))
    prev = cur
    cur = tid if owner._truthy(cond) else eid
    owner._dbg(f"[pyvm] branch cond={cond} -> next={cur}")
    return prev, cur


def op_jump(owner, inst: Dict[str, Any], _regs: Dict[int, Any], cur: int, prev: int | None) -> Tuple[int | None, int]:
    tgt = int(inst.get("target"))
    prev = cur
    cur = tgt
    owner._dbg(f"[pyvm] jump -> {cur}")
    return prev, cur


def op_ret(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> Any:
    v = owner._read(regs, inst.get("value"))
    if getattr(owner, "_debug", False):
        owner._dbg(f"[pyvm] ret {owner._type_name(v)} value={v}")
    return v


def op_call(owner, fn, inst: Dict[str, Any], regs: Dict[int, Any]) -> Any:
    # Resolve function name from value or take as literal
    fval = inst.get("func")
    if isinstance(fval, str):
        fname = fval
    else:
        fname = owner._read(regs, fval)
        if not isinstance(fname, str):
            # Fallback: if JSON encoded a literal name
            fname = fval if isinstance(fval, str) else None
    call_args = [owner._read(regs, a) for a in inst.get("args", [])]
    result = None
    if isinstance(fname, str):
        # Direct hit
        if fname in owner.functions:
            callee = owner.functions[fname]
            owner._dbg(f"[pyvm] call -> {fname} args={call_args}")
            result = owner._exec_function(callee, call_args)
        else:
            # Heuristic resolution: match suffix ".name/arity"; prefer current box context on ties
            arity = len(call_args)
            suffix = f".{fname}/{arity}"
            candidates = [k for k in owner.functions.keys() if k.endswith(suffix)]
            if len(candidates) > 1:
                # Prefer the current box if available (MiniVm.* when inside MiniVm.*)
                try:
                    cur_box = fn.name.split(".")[0] if "." in fn.name else ""
                except Exception:
                    cur_box = ""
                if cur_box:
                    scoped = [k for k in candidates if k.startswith(cur_box + ".")]
                    if len(scoped) == 1:
                        candidates = scoped
                # Still multiple: pick the lexicographically first for determinism
                if len(candidates) > 1:
                    candidates = [sorted(candidates)[0]]
            if len(candidates) == 1:
                callee = owner.functions[candidates[0]]
                owner._dbg(f"[pyvm] call -> {candidates[0]} args={call_args}")
                result = owner._exec_function(callee, call_args)
            elif getattr(owner, "_debug", False) and len(candidates) > 1:
                owner._dbg(f"[pyvm] call unresolved: '{fname}'/{arity} has multiple candidates: {candidates}")
            elif getattr(owner, "_debug", False):
                # Suggest close candidates across arities using suffix ".name/"
                any_cands = sorted([k for k in owner.functions.keys() if k.endswith(f".{fname}/") or f".{fname}/" in k])
                if any_cands:
                    owner._dbg(f"[pyvm] call unresolved: '{fname}'/{arity} — available: {any_cands}")
                else:
                    owner._dbg(f"[pyvm] call unresolved: '{fname}'/{arity} not found")
    return result
