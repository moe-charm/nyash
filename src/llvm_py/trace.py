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

def _enabled(env_key: str) -> bool:
    return os.environ.get(env_key) == '1'

def debug(msg: str) -> None:
    if _enabled('NYASH_CLI_VERBOSE'):
        try:
            print(msg, flush=True)
        except Exception:
            pass

def phi(msg: str) -> None:
    if _enabled('NYASH_LLVM_TRACE_PHI'):
        try:
            print(msg, flush=True)
        except Exception:
            pass

def values(msg: str) -> None:
    if _enabled('NYASH_LLVM_TRACE_VALUES'):
        try:
            print(msg, flush=True)
        except Exception:
            pass

