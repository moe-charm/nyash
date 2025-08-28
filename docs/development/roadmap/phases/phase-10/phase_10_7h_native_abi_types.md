# Phase 10.7h — Native ABI Types (F64/Bool)

Goal
- Extend the minimal i64-only JIT ABI to support f64 and bool as native parameter/return types in CLIF.

Principles
- Keep JIT independent from VM internals (use JitValue + adapters at boundary)
- Avoid silent truncation; perform explicit conversions in the lowerer
- Maintain safety-first fallback to VM for unsupported ops

Plan
1) JitValue widening
   - JitValue already has I64/F64/Bool/Handle — keep this as the ABI surface
   - Adapter: refine to/from VMValue mappings (no lossy coercion by default)

2) CLIF signature selection
   - Augment CraneliftBuilder to build signatures based on (arity × type shape)
   - Start with small shapes: (I64|F64|Bool)* → I64|F64|Bool
   - Closure trampoline: transmute to matching extern "C" fn type; dispatch by shape id

3) Condition handling
   - Bool: prefer b1 in IR; allow i64!=0 normalization when comparing integers
   - Comparisons yield b1; lower branch consumes b1 directly

4) Conversions in lowerer (explicit only)
   - add_const_f64, add_convert_{i64_to_f64, f64_to_i64}
   - prohibit implicit int<->float coercion in arithmetic; gate conversions via explicit MIR ops or intrinsics

5) Observability and flags
   - NYASH_JIT_NATIVE_F64=1 / NYASH_JIT_NATIVE_BOOL=1 to enable paths
   - Dump: show chosen signature shape and conversions when NYASH_JIT_DUMP=1

6) Rollout
   - Phase A: const/binop/ret for f64; comparisons yield b1
   - Phase B: mixed-type ops via explicit converts
   - Phase C: HostCall bridging for f64/bool PODs (read-only first)

Risks / Mitigation
- Signature explosion: start with a few common shapes; fallback to i64 path
- Platform ABI mismatches: rely on Cranelift default call conv; e2e-perf and correctness first

Acceptance
- Examples with pure f64 pipelines run under JIT with matching results vs VM
- No silent lossy conversions; conversions visible in MIR/Lower logs

