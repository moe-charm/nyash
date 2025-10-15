# Benchmark Results - Baseline (Before Optimization)

**測定日**: 2025-10-04
**環境**: Linux WSL2, x86_64
**測定方式**: Fixed-time (5 seconds per benchmark)

## 📊 Summary Table

| ベンチマーク | C -O3 | Rust -O3 | Python | Hakorune LLVM | Hakorune VM |
|------------|-------|----------|--------|---------------|-------------|
| **Fibonacci(12)** µs/op | 0.119 | 0.242 | 9.058 | 572 | 1864 |
| **PHI Stress If** µs/op | 3.462 | 1.941 | 540.876 | 29 | 107765 |
| **PHI Stress Loop** µs/op | 0.022 | 0.017 | 1923.725 | 44 | 384692 |

## 📈 Detailed Results

### Fibonacci(12) - Recursive

```
C -O3:
  Iterations: 41,848,998 in 5.000s
  µs/op: 0.119
  ops/sec: 8,369,799

Rust -O3:
  Iterations: 20,626,374 in 5.000s
  µs/op: 0.242
  ops/sec: 4,125,275

Python:
  Iterations: 552,020 in 5.000s
  µs/op: 9.058
  ops/sec: 110,404

Hakorune LLVM:
  Iterations: 8,736 in 5s
  µs/op: 572
  ops/sec: 1,747

Hakorune VM:
  Iterations: 2,682 in 5s
  µs/op: 1,864
  ops/sec: 536
```

**Performance Comparison (vs Python)**:
- Hakorune LLVM: **63x slower** 😭
- Hakorune VM: **206x slower**

### PHI Stress If - 8-way Branch

```
C -O3:
  Iterations: 1,444,207 in 5.000s
  µs/op: 3.462
  ops/sec: 288,841

Rust -O3:
  Iterations: 2,576,512 in 5.000s
  µs/op: 1.941
  ops/sec: 515,302

Python:
  Iterations: 9,245 in 5.000s
  µs/op: 540.876
  ops/sec: 1,849

Hakorune LLVM:
  Iterations: 168,863 in 5s
  µs/op: 29
  ops/sec: 33,772

Hakorune VM:
  Iterations: 47 in 5s
  µs/op: 107,765
  ops/sec: 9
```

**Performance Comparison (vs Python)**:
- Hakorune LLVM: **19x faster** ✨
- Hakorune VM: **199x slower**

### PHI Stress Loop - Nested 50x50x50

```
C -O3:
  Iterations: 226,050,315 in 5.000s
  µs/op: 0.022
  ops/sec: 45,210,063

Rust -O3:
  Iterations: 294,367,046 in 5.000s
  µs/op: 0.017
  ops/sec: 58,873,407

Python:
  Iterations: 2,600 in 5.002s
  µs/op: 1,923.725
  ops/sec: 520

Hakorune LLVM:
  Iterations: 112,948 in 5s
  µs/op: 44
  ops/sec: 22,589

Hakorune VM:
  Iterations: 13 in 5s
  µs/op: 384,692
  ops/sec: 2
```

**Performance Comparison (vs Python)**:
- Hakorune LLVM: **44x faster** ✨
- Hakorune VM: **200x slower**

## 🔍 Key Findings

### Hakorune LLVM Performance

**Strengths**:
- ✅ Loop-heavy workloads: 19-44x faster than Python
- ✅ Branch-heavy workloads: Competitive performance

**Weaknesses**:
- ❌ Recursive workloads: 63x slower than Python
- ❌ Reason: No optimization passes applied (equivalent to -O0)

**Root Causes Identified**:
1. **Unnecessary string Box allocations** (466x per Fibonacci(12))
2. **Excessive safepoint checks** (466x per Fibonacci(12))
3. **Dead code not eliminated** (unused type conversions)
4. **No LLVM optimization passes** (-O0 equivalent)
5. **call_legacy instead of call_direct** (dynamic dispatch overhead)

### Hakorune VM Performance

**Status**: Needs significant optimization
- 175-206x slower than Python across all benchmarks
- Still using unoptimized interpreter dispatch

## 🎯 Optimization Roadmap

### Phase 1: Quick Wins (1 week)
- [ ] Enable LLVM -O3 optimization passes
- [ ] Convert call_legacy → call_direct for static functions
- [ ] Remove dead type conversions

**Expected Impact**: 10-50x speedup (reaching Python-level performance)

### Phase 2: Safepoint Optimization (2 weeks)
- [ ] Remove safepoints from pure functions
- [ ] Optimize safepoint placement

**Expected Impact**: 2-5x additional speedup

### Phase 3: JIT Compilation (2-3 months)
- [ ] Implement hot-spot detection
- [ ] Add JIT compilation layer

**Expected Impact**: 100-1000x speedup

## 📝 Notes

- All benchmarks use fixed-time measurement (5 seconds)
- 1 second warmup before measurement
- Results are deterministic and reproducible
- Python version: CPython (default)
- Compiler versions: gcc/rustc latest stable

## 🔗 Related Files

- Python benchmark: `apps/benchmarks/micro/bench_fixed_time.py`
- Hakorune benchmark: `apps/benchmarks/harness/bench_fixed_time.hako`
- Rust benchmark: `apps/benchmarks/micro/bench_all.rs`
- C benchmark: `apps/benchmarks/micro/bench_fixed_time.c`
