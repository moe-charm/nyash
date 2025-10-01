#!/usr/bin/env python3
"""Calculate correct WASM export function index from MIR JSON

Function index = num_imports + function_position_in_json

Current imports (hardcoded):
  0: ny_check_safepoint
  1: nyash.string.to_i8p_h
"""

import sys
import json

def calc_export_index(mir_json_path, entry_name="Main.main"):
    """Calculate WASM function index for entry function"""

    with open(mir_json_path, 'r') as f:
        mir = json.load(f)

    # Hardcoded number of imports (Phase 15.8)
    NUM_IMPORTS = 2  # ny_check_safepoint, nyash.string.to_i8p_h

    # Find entry function position
    functions = mir.get('functions', [])
    for i, func in enumerate(functions):
        if func.get('name') == entry_name:
            func_index = NUM_IMPORTS + i
            print(f"{func_index}")  # Output only the index
            return func_index

    # Entry function not found
    print(f"Error: Entry function '{entry_name}' not found in MIR JSON", file=sys.stderr)
    sys.exit(1)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: wasm_calc_export_index.py <mir.json> [entry_name]", file=sys.stderr)
        sys.exit(1)

    mir_json = sys.argv[1]
    entry_name = sys.argv[2] if len(sys.argv) > 2 else "Main.main"

    calc_export_index(mir_json, entry_name)
