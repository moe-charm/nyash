"""
Box-related operations for the Nyash PyVM: newbox and boxcall.
Kept behaviorally identical to the original vm.py code.
"""
from __future__ import annotations

from typing import Any, Dict, List
import os


def op_newbox(owner, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    btype = inst.get("type")
    # Sandbox gate: only allow minimal boxes when sandbox is active
    if not owner._sandbox_allow_newbox(str(btype)):
        val = {"__box__": str(btype), "__denied__": True}
    elif btype == "ConsoleBox":
        val = {"__box__": "ConsoleBox"}
    elif btype == "StringBox":
        # empty string instance
        val = ""
    elif btype == "ArrayBox":
        val = {"__box__": "ArrayBox", "__arr": []}
    elif btype == "MapBox":
        val = {"__box__": "MapBox", "__map": {}}
    elif btype == "JsonDocBox":
        # Minimal JsonDocBox (PyVM-only): stores parsed JSON and last error
        val = {"__box__": "JsonDocBox", "__doc": None, "__err": None}
    elif btype == "JsonNodeBox":
        # Minimal JsonNodeBox with empty Null node
        val = {"__box__": "JsonNodeBox", "__node": None}
    else:
        # Unknown box -> opaque
        val = {"__box__": btype}
    owner._set(regs, inst.get("dst"), val)


def op_boxcall(owner, fn, inst: Dict[str, Any], regs: Dict[int, Any]) -> None:
    recv = owner._read(regs, inst.get("box"))
    method = inst.get("method")
    args: List[Any] = [owner._read(regs, a) for a in inst.get("args", [])]
    out: Any = None
    owner._dbg(f"[pyvm] boxcall recv={recv} method={method} args={args}")

    # Sandbox gate: disallow unsafe/unknown boxcalls
    if not owner._sandbox_allow_boxcall(recv, method):
        owner._set(regs, inst.get("dst"), out)
        return

    # Special-case: inside a method body, 'me.method(...)' lowers to a
    # boxcall with a synthetic receiver marker '__me__'. Resolve it by
    # dispatching to the current box's lowered function if available.
    if isinstance(recv, str) and recv == "__me__" and isinstance(method, str):
        box_name = ""
        try:
            if "." in fn.name:
                box_name = fn.name.split(".")[0]
        except Exception:
            box_name = ""
        if box_name:
            cand = f"{box_name}.{method}/{len(args)}"
            callee = owner.functions.get(cand)
            if callee is not None:
                owner._dbg(f"[pyvm] boxcall(__me__) -> {cand} args={args}")
                out = owner._exec_function(callee, args)
                owner._set(regs, inst.get("dst"), out)
                return

    # User-defined box: dispatch to lowered function if available (Box.method/N)
    # Skip built-in boxes (ArrayBox, MapBox, etc.) to let them fall through to their implementations
    if isinstance(recv, dict) and isinstance(method, str) and "__box__" in recv:
        box_name = recv.get("__box__")
        # Skip built-in boxes - let them fall through to built-in implementations below
        if box_name not in ("ArrayBox", "MapBox", "ConsoleBox", "FileBox", "PathBox", "JsonDocBox", "JsonNodeBox"):
            cand = f"{box_name}.{method}/{len(args)}"
            callee = owner.functions.get(cand)
            if callee is not None:
                owner._dbg(f"[pyvm] boxcall dispatch -> {cand} args={args}")
                out = owner._exec_function(callee, args)
                owner._set(regs, inst.get("dst"), out)
                return
            else:
                if owner._debug:
                    prefix = f"{box_name}.{method}/"
                    cands = sorted([k for k in owner.functions.keys() if k.startswith(prefix)])
                    if cands:
                        owner._dbg(f"[pyvm] boxcall unresolved: '{cand}' — available: {cands}")
                    else:
                        any_for_box = sorted([k for k in owner.functions.keys() if k.startswith(f"{box_name}.")])
                        owner._dbg(f"[pyvm] boxcall unresolved: '{cand}' — no candidates; methods for {box_name}: {any_for_box}")

    # ConsoleBox methods
    if method in ("print", "println", "log") and owner._is_console(recv):
        s = args[0] if args else ""
        if s is None:
            s = ""
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
            out = os.path.dirname(p)
            if out == "":
                out = "."
        elif method == "join":
            base = str(args[0]) if len(args) > 0 else ""
            rel = str(args[1]) if len(args) > 1 else ""
            out = os.path.join(base, rel)
        else:
            out = None

    # ArrayBox minimal methods
    elif isinstance(recv, dict) and recv.get("__box__") == "ArrayBox":
        arr = recv.get("__arr", [])
        if method in ("birth",):
            # No-op initializer for parity with Nyash VM
            out = 0
        elif method in ("len", "size"):
            out = len(arr)
        elif method == "get":
            idx = int(args[0]) if args else 0
            out = arr[idx] if 0 <= idx < len(arr) else None
        elif method == "set":
            idx = int(args[0]) if len(args) > 0 else 0
            val = args[1] if len(args) > 1 else None
            if 0 <= idx < len(arr):
                arr[idx] = val
            elif idx == len(arr):
                arr.append(val)
            else:
                while len(arr) < idx:
                    arr.append(None)
                arr.append(val)
            out = 0
        elif method == "push":
            val = args[0] if args else None
            arr.append(val)
            out = len(arr)
        elif method == "toString":
            out = "[" + ",".join(str(x) for x in arr) + "]"
        else:
            out = None
        recv["__arr"] = arr

    # MapBox minimal methods
    elif isinstance(recv, dict) and recv.get("__box__") == "MapBox":
        m = recv.get("__map", {})
        if method == "size":
            out = len(m)
        elif method == "has":
            key = str(args[0]) if args else ""
            out = 1 if key in m else 0
        elif method == "get":
            key = str(args[0]) if args else ""
            out = m.get(key)
        elif method == "set":
            key = str(args[0]) if len(args) > 0 else ""
            val = args[1] if len(args) > 1 else None
            m[key] = val
            out = 0
        elif method == "toString":
            items = ",".join(f"{k}:{m[k]}" for k in m)
            out = "{" + items + "}"
        else:
            out = None
        recv["__map"] = m

    # JsonDocBox (PyVM-native shim)
    elif isinstance(recv, dict) and recv.get("__box__") == "JsonDocBox":
        import json
        if method == "parse":
            s = args[0] if args else ""
            try:
                recv["__doc"] = json.loads(str(s))
                recv["__err"] = None
            except Exception as e:
                recv["__doc"] = None
                recv["__err"] = str(e)
            out = 0
        elif method == "root":
            out = {"__box__": "JsonNodeBox", "__node": recv.get("__doc", None)}
        elif method == "error":
            out = recv.get("__err") or ""
        else:
            out = None

    # JsonNodeBox (PyVM-native shim)
    elif isinstance(recv, dict) and recv.get("__box__") == "JsonNodeBox":
        node = recv.get("__node", None)
        if method == "kind":
            if node is None:
                out = "null"
            elif isinstance(node, bool):
                out = "bool"
            elif isinstance(node, int):
                out = "int"
            elif isinstance(node, float):
                out = "real"
            elif isinstance(node, str):
                out = "string"
            elif isinstance(node, list):
                out = "array"
            elif isinstance(node, dict):
                out = "object"
            else:
                out = "null"
        elif method == "get":
            key = str(args[0]) if args else ""
            if isinstance(node, dict) and key in node:
                out = {"__box__": "JsonNodeBox", "__node": node.get(key)}
            else:
                out = {"__box__": "JsonNodeBox", "__node": None}
        elif method == "size":
            if isinstance(node, list):
                out = len(node)
            elif isinstance(node, dict):
                out = len(node)
            else:
                out = 0
        elif method == "at":
            try:
                idx = int(args[0]) if args else 0
            except Exception:
                idx = 0
            if isinstance(node, list) and 0 <= idx < len(node):
                out = {"__box__": "JsonNodeBox", "__node": node[idx]}
            else:
                out = {"__box__": "JsonNodeBox", "__node": None}
        elif method == "str":
            if isinstance(node, str):
                out = node
            elif isinstance(node, dict) and isinstance(node.get("value"), str):
                out = node.get("value")
            else:
                out = ""
        elif method == "int":
            if isinstance(node, int):
                out = node
            elif isinstance(node, dict):
                v = node.get("value")
                out = int(v) if isinstance(v, int) else 0
            else:
                out = 0
        elif method == "bool":
            out = bool(node) if isinstance(node, bool) else False
        else:
            out = None

    elif method == "esc_json":
        s = args[0] if args else ""
        s = "" if s is None else str(s)
        out_chars: List[str] = []
        for ch in s:
            if ch == "\\":
                out_chars.append("\\\\")
            elif ch == '"':
                out_chars.append('\\"')
            else:
                out_chars.append(ch)
        out = "".join(out_chars)

    elif method == "length":
        out = len(str(recv))

    elif method == "substring":
        s = str(recv)
        start = int(args[0]) if (len(args) > 0 and args[0] is not None) else 0
        end = int(args[1]) if (len(args) > 1 and args[1] is not None) else len(s)
        out = s[start:end]

    elif method == "lastIndexOf":
        s = str(recv)
        needle = str(args[0]) if args else ""
        if len(args) > 1 and args[1] is not None:
            try:
                start = int(args[1])
            except Exception:
                start = 0
            out = s.rfind(needle, start)
        else:
            out = s.rfind(needle)

    elif method == "indexOf":
        s = str(recv)
        needle = str(args[0]) if args else ""
        if len(args) > 1 and args[1] is not None:
            try:
                start = int(args[1])
            except Exception:
                start = 0
            out = s.find(needle, start)
        else:
            out = s.find(needle)

    else:
        # Unimplemented method -> no-op
        out = None

    owner._set(regs, inst.get("dst"), out)
