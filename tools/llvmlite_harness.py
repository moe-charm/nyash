#!/usr/bin/env python3
"""
Experimental llvmlite-based LLVM emission harness for Nyash.

Usage:
  python3 tools/llvmlite_harness.py [--in MIR.json] --out OUTPUT.o

Notes:
- First cut emits a trivial ny_main that returns 0 to validate toolchain.
- Extend to lower MIR14 JSON incrementally.
"""
from __future__ import annotations
import argparse
import json
import os
import sys

try:
    from llvmlite import ir, binding
except Exception as e:  # noqa: BLE001
    sys.stderr.write(
        "llvmlite is required. Install with: python3 -m pip install llvmlite\n"
    )
    sys.stderr.write(f"Import error: {e}\n")
    sys.exit(2)


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description="Nyash llvmlite harness")
    ap.add_argument("--in", dest="in_path", help="MIR14 JSON input (optional)")
    ap.add_argument("--out", dest="out_path", required=True, help="Output object file path")
    return ap.parse_args()


def load_mir(path: str | None) -> dict | None:
    if not path:
        return None
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def build_trivial_module() -> ir.Module:
    mod = ir.Module(name="nyash_harness")
    mod.triple = binding.get_default_triple()
    i64 = ir.IntType(64)
    i8 = ir.IntType(8)
    i8p = i8.as_pointer()
    i8pp = i8p.as_pointer()
    fn_ty = ir.FunctionType(i64, [i64, i8pp])
    fn = ir.Function(mod, fn_ty, name="ny_main")
    entry = fn.append_basic_block(name="entry")
    b = ir.IRBuilder(entry)
    b.ret(ir.Constant(i64, 0))
    return mod


def emit_object(mod: ir.Module, out_path: str) -> None:
    binding.initialize()
    binding.initialize_native_target()
    binding.initialize_native_asmprinter()

    target = binding.Target.from_default_triple()
    tm = target.create_target_machine()
    llvm_mod = binding.parse_assembly(str(mod))
    llvm_mod.verify()
    obj = tm.emit_object(llvm_mod)
    with open(out_path, "wb") as f:
        f.write(obj)


def main() -> int:
    ns = parse_args()
    _mir = load_mir(ns.in_path)
    # For now, ignore MIR content and emit a trivial module.
    mod = build_trivial_module()
    os.makedirs(os.path.dirname(ns.out_path) or ".", exist_ok=True)
    emit_object(mod, ns.out_path)
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())

