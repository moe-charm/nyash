"""
Lightweight tracing helpers for the LLVM Python backend.

Environment flags (string '1' to enable):
- NYASH_CLI_VERBOSE: general lowering/debug logs
- NYASH_LLVM_TRACE_PHI: PHI resolution/snapshot wiring logs
- NYASH_LLVM_TRACE_VALUES: value resolution logs

Import and use:
  from trace import debug, phi, values
  debug("message")
  phi("phi message")
  values("values message")
"""

import os
import json

_TRACE_OUT = os.environ.get('NYASH_LLVM_TRACE_OUT')

def _write(msg: str) -> None:
    if _TRACE_OUT:
        try:
            with open(_TRACE_OUT, 'a', encoding='utf-8') as f:
                f.write(msg.rstrip() + "\n")
            return
        except Exception:
            pass
    try:
        print(msg, flush=True)
    except Exception:
        pass

def _enabled(env_key: str) -> bool:
    return os.environ.get(env_key) == '1'

def debug(msg: str) -> None:
    if _enabled('NYASH_CLI_VERBOSE'):
        _write(msg)

def phi(msg) -> None:
    if _enabled('NYASH_LLVM_TRACE_PHI'):
        # Accept raw strings or arbitrary objects; non-strings are JSON-encoded
        if not isinstance(msg, (str, bytes)):
            try:
                msg = json.dumps(msg, ensure_ascii=False, separators=(",", ":"))
            except Exception:
                msg = str(msg)
        _write(msg)

def values(msg: str) -> None:
    if _enabled('NYASH_LLVM_TRACE_VALUES'):
        _write(msg)

def phi_json(msg):
    """Safe JSON-style PHI trace delegator.

    - Gated by NYASH_LLVM_TRACE_PHI=1 (same gate as phi())
    - Delegates to phi_wiring.common.trace if available; otherwise no-op
    - Accepts arbitrary Python objects and forwards as-is
    """
    if not _enabled('NYASH_LLVM_TRACE_PHI'):
        return
    try:
        from phi_wiring.common import trace as _trace_phi_json  # type: ignore
        _trace_phi_json(msg)
    except Exception:
        # Fallback: stringify and route via plain phi
        try:
            if not isinstance(msg, (str, bytes)):
                try:
                    msg = json.dumps(msg, ensure_ascii=False, separators=(",", ":"))
                except Exception:
                    msg = str(msg)
            phi(msg)
        except Exception:
            pass
