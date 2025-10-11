# 3-Backend Benchmark Status Report
**Date**: 2025-10-01
**Branch**: wasm-development
**Purpose**: Verify Hakorune/Nyash code execution across all three backends

---

## 🎯 **3-Backend Trinity Architecture**

```
                    ┌─────────────────────┐
                    │  Hakorune Source    │
                    │  (.nyash files)     │
                    └──────────┬──────────┘
                               │
                ┌──────────────┼──────────────┐
                │              │              │
        ┌───────▼─────┐  ┌────▼─────┐  ┌────▼──────┐
        │  Rust VM    │  │ LLVM     │  │  LLVM     │
        │  (Interp)   │  │ Native   │  │  WASM     │
        │             │  │ (exe)    │  │  (wasm)   │
        └─────────────┘  └──────────┘  └───────────┘
             ↓                ↓              ↓
        ┌─────────────────────────────────────────┐
        │    Same Result Verification             │
        └─────────────────────────────────────────┘
```

---

## 📊 **Current Benchmark Results**

### ✅ **Benchmark 1: simple_add**
**File**: `apps/benchmarks/simple_add/main.nyash`
**Expected Result**: 42 (15 + 27)
**Tests**: Basic integer arithmetic

| Backend | Status | Result | Verified |
|---------|--------|--------|----------|
| Rust VM | ✅ Pass | 42 | 2025-10-01 ✅ |
| LLVM Native | ⏳ Pending | N/A | Needs --features llvm |
| LLVM WASM | 🚧 IR OK | 0 | IR generated, runner incomplete |

```nyash
static box Main {
  main() {
    local result = 15 + 27
    print(result)
    return result  // → 42
  }
}
```

### ✅ **Benchmark 2: fibonacci**
**File**: `apps/benchmarks/fibonacci/main.nyash`
**Expected Result**: 55 (fibonacci(10))
**Tests**: Integer arithmetic, recursion, control flow

| Backend | Status | Result | Verified |
|---------|--------|--------|----------|
| Rust VM | ✅ Pass | 55 | 2025-10-01 ✅ |
| LLVM Native | ⏳ Pending | N/A | Needs --features llvm |
| LLVM WASM | 🚧 IR OK | ERROR | IR generated, runner incomplete |

```nyash
static box Main {
  compute(n) {
    if n <= 1 {
      return n
    }
    return me.compute(n - 1) + me.compute(n - 2)
  }

  main() {
    local result = me.compute(10)
    print(result)
    return result  // → 55
  }
}
```

---

## 🔧 **Execution Commands**

### 1️⃣ **Rust VM** (開発・デバッグ用)
```bash
# simple_add
./target/release/nyash --backend vm apps/benchmarks/simple_add/main.nyash
# Output: 42 ✅

# fibonacci
./target/release/nyash --backend vm apps/benchmarks/fibonacci/main.nyash
# Output: 55 ✅
```

### 2️⃣ **LLVM Native** (本番・最適化用)
```bash
# Prerequisites: Build with LLVM feature (3-5 minutes)
cargo build --release --features llvm

# Execution
NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT=/tmp/bench.o \
  ./target/release/nyash --backend llvm apps/benchmarks/simple_add/main.nyash
```

**Status**: ⏳ Not tested - requires rebuild with `--features llvm`

### 3️⃣ **LLVM WASM** (ブラウザ・エッジ環境用)
```bash
# Prerequisites: Python 3 + llvmlite
cd src/llvm_py
python3 -m venv venv
./venv/bin/pip install llvmlite

# Generate MIR JSON
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash \
  --emit-mir-json /tmp/bench.json apps/benchmarks/simple_add/main.nyash

# Compile to WASM
cd src/llvm_py
./venv/bin/python llvm_builder.py \
  --target wasm32 \
  /tmp/bench.json \
  -o /tmp/bench.wasm

# Execute (Node.js with WASI support)
node --experimental-wasi-unstable-preview1 wasm_runner.js /tmp/bench.wasm
```

**Status**: ⏳ Infrastructure exists, but needs:
- WASM runner script (`wasm_runner.js`)
- Entry point export handling
- WASI runtime initialization

---

## 🎯 **Entry Point Resolution**

**Issue History**: Fibonacci benchmark initially failed due to entry point mismatch

**Root Cause**: Compiler was generating `Fibonacci.main/0()` but VM was looking for `Main.main()`

**Fix Applied** (2025-10-01):
- Enhanced entry resolution in core (default ON)
- Now supports unique `Box.main` as entry point (e.g., `Fibonacci.main`)
- Environment variable: `NYASH_ENTRY_PREFER_STATIC_MAIN=0` to disable
- Fallback to `Main.main()` if no unique Box.main found

**Result**: Both benchmarks now work with `static box Main { main() {} }` pattern ✅

---

## 📋 **Next Steps**

### Short-term (Week 3-4)
- [ ] Test LLVM Native execution (requires rebuild)
- [ ] Complete WASM execution pipeline
  - [ ] Create/update `wasm_runner.js`
  - [ ] Test fibonacci in WASM
  - [ ] Verify result matches VM/Native
- [ ] Add more benchmarks:
  - [ ] Loop iteration benchmark
  - [ ] Array operations benchmark
  - [ ] String manipulation benchmark

### Mid-term
- [ ] Automated benchmark runner script
  - [ ] Parse expected results from comments
  - [ ] Run all three backends
  - [ ] Compare results and report
- [ ] Performance comparison metrics
  - [ ] Execution time measurement
  - [ ] Memory usage tracking
  - [ ] Binary size comparison

### Long-term
- [ ] Continuous integration for 3-backend verification
- [ ] Benchmark regression tracking
- [ ] Performance optimization based on benchmarks

---

## 🔍 **Technical Details**

### MIR Generation
Both benchmarks generate proper MIR with:
- Entry function: `Main.main()` (i64 return)
- Recursive calls via `call_legacy` (fibonacci)
- Proper PHI nodes for control flow merge
- Console log via `extern_call env.console.log`

### WASM Target Implementation
**Location**: `src/llvm_py/targets/wasm.py`

**Features**:
- Target triple: `wasm32-unknown-wasi`
- External linkage for exports
- Direct WASM binary generation via llvmlite
- No LLC/wasm-ld required (self-contained)

### Rust VM Features
- High-speed execution (no compilation overhead)
- Type-safe operations
- Complete PHI/SSA support
- Perfect for development/debugging

---

## 📚 **Related Documents**

- **Benchmark Issue Report**: `BENCHMARK_ISSUE_REPORT.md` - Entry point fix history
- **Phase 15.8 Plan**: `docs/development/roadmap/phases/phase-15.8/README.md` - WASM development roadmap
- **WASM Smoke Tests**: `tools/smokes/v2/` - Regression test framework
- **Language Reference**: `docs/reference/language/quick-reference.md` - Syntax guide

---

**Status Summary**: 🟢 Rust VM verified, 🟡 LLVM/WASM pending implementation
