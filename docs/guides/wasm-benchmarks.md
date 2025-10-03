# WASM Benchmark Guide

## 📊 Overview

This guide explains how to run and create WASM benchmarks for the Hakorune WASM project.

## 🚀 Quick Start

### Running All Benchmarks

```bash
bash tools/run_wasm_benchmark_suite.sh
```

### Running Hako Source Benchmarks

```bash
bash tools/run_wasm_benchmarks.sh
```

## 📋 Current Benchmark Suite

### Smoke Tests (Working ✅)
- **arithmetic**: Basic arithmetic operations (6 expected)
- **compare**: Comparison operations (5 expected)
- **control_flow**: Control flow with branches (111 expected)
- **binop_all**: All binary operations (44 expected)

### Performance Benchmarks (Working ✅)
- **factorial_12**: Iterative factorial(12) = 479,001,600
- **power_2_30**: Power calculation 2^30 = 1,073,741,824
- **sum_10k**: Loop sum(0..9999) = 49,995,000

## ✅ WASM Backend Capabilities (Phase 15.8 Week 3)

### Fully Supported
- ✅ **Basic Arithmetic**: `+`, `-`, `*`, `/`, `%`
- ✅ **Bitwise Operations**: `&`, `|`, `^`, `<<`, `>>`
- ✅ **Comparison**: `<`, `<=`, `>`, `>=`, `==`, `!=`
- ✅ **Control Flow**: `branch`, `jump`, `ret`
- ✅ **PHI Instructions**: Loop merge points
- ✅ **i32 Range Integers**: Values within 32-bit signed range

### Not Yet Supported
- ❌ **Function Calls**: `call` instruction (no recursion/function calls)
- ❌ **String Operations**: StringBox, string concatenation
- ❌ **i64 Full Range**: Values beyond 32-bit signed range overflow

## 🔍 Known Limitations

### 1. Integer Overflow (i64 → i32)
The current WASM backend treats i64 as i32, causing overflow for large values:

**Problem Example**:
- `factorial(20)` = 2.4×10^18 → **Overflow** ❌
- `sum(0..99999)` = 5×10^9 → **Overflow** ❌

**Working Example**:
- `factorial(12)` = 479,001,600 → **OK** ✅
- `sum(0..9999)` = 49,995,000 → **OK** ✅

### 2. No Function Calls
Recursive algorithms require manual conversion to iterative versions:

**Not Working**:
```json
{
  "op": "call",
  "func": 100,
  "args": [n_minus_1],
  "dst": 2
}
```

**Working Alternative**:
Use loops with PHI instructions instead.

### 3. No String Support
Hako source files that generate StringBox constants cannot compile to WASM:

**Problem**: Auto-generated `Main.factorial/1` function names → StringBox constants
**Solution**: Use hand-written MIR JSON with numerical operations only

## 📝 Creating New Benchmarks

### Option 1: Hand-Written MIR JSON (Recommended)

Create a simple MIR JSON file with numerical operations only:

```json
{
  "functions": [
    {
      "name": "ny_main",
      "params": [],
      "blocks": [
        {
          "id": 0,
          "instructions": [
            {"op": "const", "dst": 1, "value": {"type": "i64", "value": 10}},
            {"op": "const", "dst": 2, "value": {"type": "i64", "value": 20}},
            {"op": "binop", "dst": 3, "operation": "+", "lhs": 1, "rhs": 2},
            {"op": "ret", "value": 3}
          ]
        }
      ]
    }
  ]
}
```

### Option 2: Hako Source (Future)

Once string support is added, Hako source benchmarks will work:

```hako
static box Main {
    factorial(n) {
        // Will work after string support
    }
}
```

## 🎯 Adding to Benchmark Suite

1. Create MIR JSON file: `src/llvm_py/bench_<name>.json`
2. Calculate expected result
3. Add to `tools/run_wasm_benchmark_suite.sh`:

```bash
declare -a BENCHMARKS=(
  ...
  "my_bench:$PROJECT_ROOT/src/llvm_py/bench_my_bench.json:EXPECTED_VALUE"
)
```

## 📊 Benchmark Results Format

```
🚀 WASM Benchmark Suite
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[1] arithmetic
  Building WASM...
  Running WASM...
  ✅ PASS: result=6, time=28ms

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Results: Total=7 Passed=7 Failed=0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 All benchmarks passed!
```

## 🐛 Troubleshooting

### Build Failed
- Check if MIR JSON uses only supported operations
- Avoid `call`, StringBox, and other unsupported features

### Wrong Result (Integer Overflow)
- Reduce input values to fit 32-bit signed range (-2^31 to 2^31-1)
- Example: factorial(12) instead of factorial(20)

### Runtime Error
- Check WASM import errors in console
- Ensure all required WASI functions are implemented in `wasm_runner.js`

## 📈 Future Improvements

- [ ] i64 full range support
- [ ] Function call support (recursion)
- [ ] StringBox and string operations
- [ ] Array operations
- [ ] Box operations (BoxCall)

## 📚 Related Documentation

- [Phase 15.8 README](../development/roadmap/phases/phase-15.8/README.md) - WASM implementation roadmap
- [CLAUDE.md](../../CLAUDE.md) - Project progress and decisions
- [CURRENT_TASK.md](../../CURRENT_TASK.md) - Current development status
