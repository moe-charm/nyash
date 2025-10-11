# Nyash Phase 15.7: Unified Calls, Mini‑VM in Nyash, and a Stable Path to Self‑Hosting

## Abstract
We present a behavior-preserving consolidation of Nyash’s method-call pipeline and a Nyash-written Mini‑VM that executes a minimal MIR(JSON v0) subset. The unified call design (Known/Rewrite + RouterPolicy) reduces ambiguity without changing defaults, guarded by development-only diagnostics. Our Mini‑VM (MirVmMin) provides a precise, brace-balanced single-pass executor for const/binop/compare/branch/jump/ret, enabling tight, reproducible smoke tests. Quick profile passes 72/72, integration (llvmlite harness) is green, and a self-host compiler path (dev-only) reliably emits non-empty JSON headers. These foundations set a stable, incremental path towards full self-hosting.

## 1. Introduction
Nyash is a language with multiple execution lines (Rust VM and LLVM/llvmlite harness; a historical PyVM existed for development but is now withdrawn by default), targeting practical self-hosting. During Phase 15.7 we found that stability and observability benefit from unifying how instance methods become function calls and from a minimal Nyash-written VM that validates MIR(JSON v0) control-flow.

## 2. Background
The builder emits MIR from Nyash source. Historically, instance calls could traverse heterogeneous paths, occasionally complicating materialization (LocalSSA) and PHI merges. Concurrently, a minimal Nyash Mini‑VM helps validate semantics and JSON segmentation independent from the Rust VM.

## 3. Design
- Unified Calls: Known/Rewrite consolidates obj.m into Class.m(me, …) when unique and safe; Unknown receivers fall back to BoxCall (always-on, stability-first). String-like methods normalize receivers to StringBox.
- LocalSSA & Materialize: Guarantees per-block copies before use; φ→Copy→Call ordering enforced.
- VarMapGuard: Prevents accidental me binding at joins.
- Mini‑VM: Single-pass, brace-balanced instruction segmentation; v0/v1 compare forms supported.

## 4. Implementation
Key modules in builder (RouterPolicy, ReceiverInference, LocalSSA/materialize, VarMapGuard) and MirVmMin implemented in Nyash (selfhost/vm/boxes/mir_vm_min.nyash). Flags are default-OFF for diagnostics; behavior remains unchanged by default.

## 5. Evaluation
- Quick profile: 72/72 PASS (includes new Mini‑VM edges: mixed compare, branch undef cond, jump chain, div/mod zero, no-ret fallback).
- Integration: PASS with llvmlite harness.
- Dev-only self-host: `NYASH_JSON_ONLY=1` yields non-empty `{version,kind}` JSON headers.

## 6. Case Studies
- ParserBox string APIs (length/substring/indexOf/lastIndexOf) stabilized via receiver normalization and unified call routing.
- Mini‑VM ensures precise compare→ret correctness under varied JSON encodings.

## 7. Related Work
Self-hosted compilers, SSA materialization strategies, and harness-mediated AOT testing pipelines.

## 8. Limitations & Future Work
Remaining dev valves are default-OFF and slated for removal. Phase 16 will extend PHI/SSA generalization. Self-host compiler MVP progresses next with reproducible gates.

## 9. Conclusion
By unifying calls and validating control-flow via a Nyash Mini‑VM, we maintain green pipelines and establish a clean, incremental trajectory to self-hosting without altering default user semantics.
