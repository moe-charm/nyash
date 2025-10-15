# WASM Line Roadmap (Phase-A → Phase-B)

Goal
- Bring a minimal, predictable WASM line online that matches the VM/LLVM semantics for a small, high‑value subset.

Constraints
- Plugins are disabled on `wasm32` (cfg). HostHandle identity is not applicable.
- Prefer builtin/core boxes only. No dynamic loading.
- Preserve TypeRegistry slot mapping used by VM/LLVM to keep parity simple.

Phases

Phase‑A (Minimal Core, builtin only)
- Scope: ArrayBox, StringBox, MapBox (read‑only subset), Console print.
- ArrayBox: implement len/size/get/set/push/clear in WASM codegen.
  - Representation: header(len, cap) + contiguous i32 data; out‑of‑bounds → 0/null.
  - Slot mapping: reuse TypeRegistry (100..113).
- StringBox: keep existing const + toString print path; substring/indexOf minimal.
- MapBox: size/has/get limited (integer keys only) or SKIP in Phase‑A if not stable.
- BoxCall lowering: `backend/wasm/codegen/boxcall.rs` dispatches Array methods by slot/name.
- Imports: provide minimal print/log imports (no host GC/handles).

Phase‑B (Coverage & Parity)
- Extend String/Map coverage to match VM parity set.
- Add vtable_codegen/unified_dispatch path once stable (no perf regressions).
- Optional: WASM AOT (`wasm-ld`) pipeline hooks in builder scripts.

Acceptance (Phase‑A)
- Quick parity (arithmetic/compare/branch) remains green.
- WASM mini parity: one case for Array len/get/set and print → PASS (auto‑SKIP if toolchain missing).
- No plugin path accidental calls on wasm32 (cfg‑guarded).

Notes
- Identity/HostHandle is a non‑goal for WASM; use pure WASM data representations for core boxes.
- Keep TypeRegistry slot ids as the single source of truth to avoid backend drift.
