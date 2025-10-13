#!/usr/bin/env python3
"""
Nyash llvmlite harness (scaffold)

Usage:
  - python3 tools/llvmlite_harness.py --out out.o                # dummy ny_main -> object
  - python3 tools/llvmlite_harness.py --in mir.json --out out.o  # MIR(JSON) -> object (partial support)

Notes:
  - For initial scaffolding, when --in is omitted, a trivial ny_main that returns 0 is emitted.
  - When --in is provided, this script delegates to src/llvm_py/llvm_builder.py.
"""

import argparse
import json
import os
import sys
from pathlib import Path

_default_root = Path(__file__).resolve().parents[1]
_env_root = None
try:
    env_root_str = os.environ.get('NYASH_ROOT')
    if env_root_str:
        cand = Path(env_root_str).resolve()
        if (cand / "src" / "llvm_py" / "llvm_builder.py").exists():
            _env_root = cand
except Exception:
    _env_root = None

ROOT = _env_root or _default_root
PY_BUILDER = ROOT / "src" / "llvm_py" / "llvm_builder.py"

def run_dummy(out_path: str) -> None:
    # Minimal llvmlite program: ny_main() -> i32 0
    import llvmlite.ir as ir
    import llvmlite.binding as llvm

    llvm.initialize()
    llvm.initialize_native_target()
    llvm.initialize_native_asmprinter()

    mod = ir.Module(name="nyash_module")
    i32 = ir.IntType(32)
    ny_main_ty = ir.FunctionType(i32, [])
    ny_main = ir.Function(mod, ny_main_ty, name="ny_main")
    entry = ny_main.append_basic_block("entry")
    b = ir.IRBuilder(entry)
    b.ret(ir.Constant(i32, 0))

    # Emit object via target machine
    m = llvm.parse_assembly(str(mod))
    m.verify()
    target = llvm.Target.from_default_triple()
    tm = target.create_target_machine()
    obj = tm.emit_object(m)
    Path(out_path).parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "wb") as f:
        f.write(obj)

def run_from_json(in_path: str, out_path: str) -> None:
    # Delegate to python builder to keep code unified
    import runpy
    # Enable safe defaults for prepasses unless explicitly disabled by env
    os.environ.setdefault('NYASH_LLVM_PREPASS_LOOP', os.environ.get('NYASH_LLVM_PREPASS_LOOP', '0'))
    os.environ.setdefault('NYASH_LLVM_PREPASS_IFMERGE', os.environ.get('NYASH_LLVM_PREPASS_IFMERGE', '1'))
    # PHI policy: builderization unified. Create-only at block head via PhiHandler; finalize wires only.
    # Default STRICT=1 to avoid double-generation unless caller explicitly overrides.
    os.environ.setdefault('NYASH_LLVM_PHI_STRICT', os.environ.get('NYASH_LLVM_PHI_STRICT', '1'))
    # Ensure src/llvm_py is on sys.path for relative imports
    builder_dir = str(PY_BUILDER.parent)
    if builder_dir not in sys.path:
        sys.path.insert(0, builder_dir)
    # Simulate "python llvm_builder.py <in> -o <out>"
    sys.argv = [str(PY_BUILDER), str(in_path), "-o", str(out_path)]
    runpy.run_path(str(PY_BUILDER), run_name="__main__")

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="infile", help="MIR JSON input", default=None)
    ap.add_argument("--out", dest="outfile", help="output object (.o)", required=True)
    args = ap.parse_args()

    if args.infile is None:
        # Dummy path
        run_dummy(args.outfile)
        if os.environ.get('NYASH_CLI_VERBOSE') == '1':
            print(f"[harness] dummy object written: {args.outfile}")
    else:
        run_from_json(args.infile, args.outfile)
        if os.environ.get('NYASH_CLI_VERBOSE') == '1':
            print(f"[harness] object written: {args.outfile}")

if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        import traceback
        print(f"[harness] error: {e}", file=sys.stderr)
        if os.environ.get('NYASH_CLI_VERBOSE') == '1':
            traceback.print_exc()
        sys.exit(1)
