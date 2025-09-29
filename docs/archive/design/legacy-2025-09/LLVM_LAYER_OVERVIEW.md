# LLVM Layer Overview (Phase 15)

Scope
- Practical guide to LLVM lowering architecture and invariants used in Phase 15.
- Complements LOWERING_LLVM.md (rules), RESOLVER_API.md (value resolution), and docs/reference/architecture/llvm-harness.md (harness).

Module Layout
- `src/backend/llvm/compiler/codegen/`
  - `instructions/` split by concern: `flow.rs`, `blocks.rs`, `arith.rs`, `arith_ops.rs`, `mem.rs`,
    `strings.rs`, `arrays.rs`, `maps.rs`, `boxcall/`, `externcall/`, `call.rs`, `loopform.rs`, `resolver.rs`.
  - `builder_cursor.rs`: central insertion/terminator guard.

Core Invariants
- Phase‑15 終盤（MIR13運用）: MIR 生成層では PHI を生成しない。PHI 合成は LLVM 層（llvmlite/Resolver）が担う。
- Resolver-only reads: lowerers fetch MIR values through `Resolver`（クロスBBの vmap 直接参照は禁止）。
- Localize at block start: PHIs are created at the beginning of the current BB（non‑PHI より手前）で優位性を保証。
- Cast placement: ptr↔int/幅変換は PHI の外側（BB先頭 or pred終端直前）に配置。
- Sealed SSA: 後続ブロックの PHI は pred スナップショットと `seal_block` で配線し、branch/jump 自体は incoming を直接積まない。
- Cursor discipline: 生成は `BuilderCursor` 経由のみ。terminator 後の挿入は禁止。

LoopForm（次フェーズ予定/MIR18）
- Phase‑15 では LoopForm を MIR に導入しない。既存 CFG（preheader→header→{body|exit}; body→header）から llvmlite がループ搬送 PHI を合成。
- 次フェーズで LoopForm（`LoopHeader/Enter/Latch` などの占位）を MIR に追加し、Resolver/PHI 合成は維持する。

Types and Bridges
- Box handle is `i64` across NyRT boundary; strings prefer `i8*` fast paths.
- Convert rules: `ensure_i64/ensure_i1/ensure_ptr` style helpers (planned extraction) to centralize casting.

Harness (optional)
- llvmlite harness exists for fast prototyping and structural checks.
- Gate: `NYASH_LLVM_USE_HARNESS=1` (planned wiring); target parity tested by Acceptance A5.

References
- LOWERING_LLVM.md — lowering rules and runtime calls
- RESOLVER_API.md — Resolver design and usage
- docs/reference/architecture/llvm-harness.md — llvmlite harness interface and usage
