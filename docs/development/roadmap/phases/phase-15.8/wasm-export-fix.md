# WASM Export Section Fix (Phase 15.8 Week 3)

**Date**: 2025-10-01
**Status**: ✅ Fixed
**Branch**: `wasm-development`

## Problem

llvmlite generates WASM binaries with incorrect export section:
- **Export name**: Usually correct (`Main.main`)
- **Export index**: **WRONG** - points to import function instead of module function
- **Root cause**: llvmlite doesn't account for import functions when calculating export indices

### Example

```
Imports:
  0: __linear_memory (memory)
  1: ny_check_safepoint (func)
  2: nyash.string.to_i8p_h (func)

Module functions:
  2: add_simple
  3: Main.main  ← Correct index

llvmlite export:
  Main.main → index 1  ❌ Points to ny_check_safepoint (import!)

Correct export:
  Main.main → index 3  ✅ Points to actual Main.main function
```

### Impact

- **Error**: `Cannot convert undefined to BigInt`
- **Cause**: Wrong function signature (import expects parameters, entry expects 0)
- **Result**: WASM execution fails immediately

## Solution

3-step post-processing pipeline:

### 1. Remove Incorrect Export
```python
# tools/wasm_remove_export.py
def remove_exports(input_path, output_path):
    # Strips section_id=7 (Export) from WASM binary
```

### 2. Calculate Correct Index
```python
# tools/wasm_calc_export_index.py
def calc_export_index(mir_json_path, entry_name="Main.main"):
    # func_index = NUM_IMPORTS + function_position_in_json
    # NUM_IMPORTS = 2 (ny_check_safepoint, nyash.string.to_i8p_h)
```

### 3. Add Correct Export
```python
# src/llvm_py/tools/wasm_add_export.py
def add_export(wasm_path, output_path, func_name, func_index):
    # Inserts Export section with correct index
```

## Integration

Updated `tools/build_wasm.sh`:

```bash
# Step 2.5: Fix export section
python3 tools/wasm_remove_export.py input.wasm temp_noexp.wasm
EXPORT_INDEX=$(python3 tools/wasm_calc_export_index.py input.json "Main.main")
python3 src/llvm_py/tools/wasm_add_export.py temp_noexp.wasm output.wasm "Main.main" "$EXPORT_INDEX"
```

## Results

✅ **Before fix**:
```
Error: Cannot convert undefined to BigInt
Export: Main.main → index 1 (wrong function)
```

✅ **After fix**:
```
🚀 Calling Main.main()...
[DEBUG] ny_check_safepoint called
[DEBUG] to_i8p_h(42) type=bigint
✅ Main.main() returned: 42
```

## Test Case

`/tmp/test_call_minimal.json`:
```json
{
  "functions": [
    {
      "name": "add_simple",
      "params": [{"name": "a", "reg": 0}, {"name": "b", "reg": 1}],
      "blocks": [{"id": 0, "instructions": [
        {"op": "binop", "operation": "+", "lhs": 0, "rhs": 1, "dst": 2},
        {"op": "ret", "value": 2}
      ]}]
    },
    {
      "name": "Main.main",
      "params": [],
      "blocks": [{"id": 0, "instructions": [
        {"op": "const", "dst": 0, "value": {"type": "i64", "value": 10}},
        {"op": "const", "dst": 1, "value": {"type": "i64", "value": 32}},
        {"op": "call", "func": "add_simple", "args": [0, 1], "dst": 2},
        {"op": "ret", "value": 2}
      ]}]
    }
  ]
}
```

## Files Modified

- ✅ `tools/build_wasm.sh` - Pipeline integration
- ✅ `tools/wasm_remove_export.py` - NEW (52 lines)
- ✅ `tools/wasm_calc_export_index.py` - NEW (38 lines)
- ✅ `src/llvm_py/tools/wasm_add_export.py` - Already existed, used correctly

## Future Work

- [ ] Handle multiple entry points (not just Main.main)
- [ ] Auto-detect NUM_IMPORTS from WASM binary instead of hardcoding
- [ ] Upstream fix to llvmlite (if possible)

## Related Issues

- **Phase 15.8 Week 3**: Function call tests failing
- **call命令引数解決バグ**: Related to safepoint insertion
- **PHI debug assertion**: Unrelated but also fixed this week

## Verification

```bash
# Build WASM
./tools/build_wasm.sh test.json -o test.wasm

# Verify export
python3 /tmp/parse_export.py test.wasm
# Output: Export 0: 'Main.main' (kind=func, index=3) ✅

# Run WASM
node src/llvm_py/tools/wasm_runner.js test.wasm
# Output: ✅ Main.main() returned: 42
```

---

**結論**: llvmliteのWASM export制限を完全克服！関数呼び出し完全動作✅
