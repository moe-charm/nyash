# Figures & Repro Notes (WIP)

## DOT (CFG/PHI with b1 labels)
- Enable JIT + PHI-min: `NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_PHI_MIN=1`
- Write DOT: `NYASH_JIT_DOT=out.dot ./target/release/nyash --backend vm examples/jit_phi_demo.nyash`
- Expected labels:
  - Branch edges: `then cond:b1`, `else cond:b1`
  - Node label: `phi:N (b1:M)` when boolean PHIs exist
- Render with Graphviz: `dot -Tpng out.dot -o out.png`

## Bench Table (early)
- Command: `./target/release/nyash --benchmark --iterations 50 --jit-stats`
- Cases:
  - simple_add (note: early stub impact)
  - arith_loop_100k (JIT ≈ 1.40× VM)
  - branch_return (≈ VM)
  - f64_add (JIT ≈ 1.06× VM)

## Capability Probe
- Current toolchain: `supports_b1_sig=false` (B1 in signatures disabled)
- Future: flip to true after Cranelift upgrade/verification, then switch:
  - `ParamKind::B1 → types::B1` (one-line change in builder)
