#!/usr/bin/env python3
"""
Nyash PyVM runner (scaffold)

Usage:
  - python3 tools/pyvm_runner.py --in mir.json [--entry Main.main] [--args-env NYASH_SCRIPT_ARGS_JSON]

Executes MIR(JSON) using a tiny Python interpreter for a minimal opcode set:
  - const/binop/compare/branch/jump/ret
  - newbox (ConsoleBox, StringBox minimal)
  - boxcall (String: length/substring/lastIndexOf; Console: print/println/log)
  - externcall (nyash.console.println)

On success, exits with the integer return value if it is an Integer; otherwise 0.
Outputs produced by println/log are written to stdout.
"""

import argparse
import json
import sys
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYVM_DIR = ROOT / "src" / "llvm_py" / "pyvm"

# Ensure imports can find the package root (src)
SRC_DIR = ROOT / "src"
if str(SRC_DIR) not in sys.path:
    sys.path.insert(0, str(SRC_DIR))

from llvm_py.pyvm.vm import PyVM  # type: ignore


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="infile", required=True, help="MIR JSON input")
    ap.add_argument("--entry", dest="entry", default="Main.main", help="Entry function (default Main.main)")
    ap.add_argument("--args-env", dest="args_env", default="NYASH_SCRIPT_ARGS_JSON", help="Env var containing JSON array of argv to pass to entry")
    args = ap.parse_args()

    with open(args.infile, "r") as f:
        program = json.load(f)

    vm = PyVM(program)
    # Fallbacks for entry name
    entry = args.entry
    fun_names = {f.get("name", "") for f in program.get("functions", [])}
    if entry not in fun_names:
        if "main" in fun_names:
            entry = "main"
        elif "Main.main" in fun_names:
            entry = "Main.main"

    # Load argv if present
    argv: list[str] = []
    if args.args_env:
        js = os.environ.get(args.args_env)
        if js:
            try:
                arr = json.loads(js)
                if isinstance(arr, list):
                    argv = [str(x) if x is not None else "" for x in arr]
            except Exception:
                pass

    result = vm.run_args(entry, argv) if argv else vm.run(entry)
    # Exit code convention: integers propagate; bool -> 0/1; else 0
    code = 0
    if isinstance(result, bool):
        code = 1 if result else 0
    elif isinstance(result, int):
        # Clamp to 32-bit exit code domain
        code = int(result) & 0xFFFFFFFF
        if code & 0x80000000:
            code = -((~code + 1) & 0xFFFFFFFF)
    # For parity comparisons, avoid emitting extra lines here.
    sys.exit(code)


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        import traceback
        print(f"[pyvm] error: {e}", file=sys.stderr)
        if sys.stderr and (os.environ.get('NYASH_CLI_VERBOSE') == '1' or True):
            traceback.print_exc()
        sys.exit(1)
