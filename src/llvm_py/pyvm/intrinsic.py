"""
Intrinsic helpers for PyVM. Keep logic identical to the inline version.
"""
from __future__ import annotations

from typing import Any, List, Tuple
import os


def try_intrinsic(name: str, args: List[Any]) -> Tuple[bool, Any]:
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
        if name == "MiniVm.read_digits/2":
            s = "" if not args or args[0] is None else str(args[0])
            pos = 0 if len(args) < 2 or args[1] is None else int(args[1])
            out_chars: list[str] = []
            while pos < len(s):
                ch = s[pos]
                if '0' <= ch <= '9':
                    out_chars.append(ch)
                    pos += 1
                else:
                    break
            return True, "".join(out_chars)
        if name == "MiniVm.parse_first_int/1":
            js = "" if not args or args[0] is None else str(args[0])
            key = '"value":{"type":"int","value":'
            idx = js.rfind(key)
            if idx < 0:
                return True, "0"
            start = idx + len(key)
            ok, digits = try_intrinsic("MiniVm.read_digits/2", [js, start])
            return True, digits
        if name == "Main.dirname/1":
            p = "" if not args else ("" if args[0] is None else str(args[0]))
            d = os.path.dirname(p)
            if d == "":
                d = "."
            return True, d
    except Exception:
        pass
    return (False, None)

