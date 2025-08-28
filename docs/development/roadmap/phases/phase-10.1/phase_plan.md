# Phase 10.1 – Plugin Unification Path (MIR→JIT/AOT via C ABI)

This plan refines how we leverage the existing plugin system (BID-FFI) to unify JIT and AOT (EXE) paths using a single C ABI surface.

## Goals
- Unify calls from JIT and AOT to the same C ABI (`nyrt_*` / `nyplug_*`).
- Convert builtin Boxes to Plugin Boxes in small steps (read-only first).
- Produce a minimal standalone EXE via static linking after unification.

## Feasibility Summary
- JIT: emit calls to `extern "C"` symbols (no change in semantics, only target).
- AOT: emit `.o` with unresolved `nyrt_*` / `nyplug_*` and link with `libnyrt.a` + plugin `.a`.
- Compatibility: guard with `NYASH_USE_PLUGIN_BUILTINS` and keep HostCall fallback.

## Phase Breakdown

### 10.1: Plugin PoC + C ABI base (1 week)
- Deliverables:
  - Minimal headers: `nyrt.h` (runtime), `nyplug_array.h` (ArrayBox plugin).
  - ArrayBox as a plugin (`cdylib` + `staticlib`), ABI version functions.
  - VM loader integration and `NYASH_USE_PLUGIN_BUILTINS` switch.
  - Smoke: `new/len/push/get` working via plugin.
- DoD:
  - Array plugin works on VM path; perf regression ≤10% on micro bench.

### 10.2: JIT Lowering unification (Array first) (1–1.5 weeks)
- Deliverables:
  - IRBuilder: `emit_plugin_invoke(type_id, method_id, args, sig)`.
  - LowerCore BoxCall for Array routes to `plugin_invoke` (events/stats intact).
  - Feature-flagged enablement: `NYASH_USE_PLUGIN_BUILTINS=1`.
- DoD:
  - JIT execution of Array read/write (policy-constrained) via plugin path.
  - Behavior parity with HostCall; no regressions on CI smoke.

### 10.2b: JIT Coverage Unblockers (0.5–1 week)
- Goal:
  - Remove practical blockers so plugin_invoke can be exercised in typical Nyash functions and `.o` can be produced.
- Deliverables:
  - Lowering for `NewBox` of pluginized builtins → translate `new <Box>()` to plugin `birth()` via `emit_plugin_invoke(type_id, 0, argc=1 recvr-param)` with appropriate handle threading.
  - Treat `Print/Debug` as no-op/hostcall for v0 to avoid function-wide skip.
  - Keep conservative skip policy by default; document `NYASH_AOT_ALLOW_UNSUPPORTED=1` for validation-only `.o` emission.
- DoD:
  - Minimal demo function with `String.length()` compiled by JIT (Cranelift) and `.o` emitted. Plugin events visible under JIT.

### 10.3: Broaden plugin coverage + Compatibility (2 weeks)
- Targets: String/Integer/Bool/Map (read-only first).
- Deliverables:
  - Pluginized Boxes and `plugin_invoke` lowering for BoxCall.
  - HostCall route retained; whitelist-driven co-existence.
  - Added smoke and microbenches comparing HostCall vs Plugin.
- DoD:
  - ≥5 builtin Boxes pluginized; `NYASH_USE_PLUGIN_BUILTINS=1` green on smoke.

### 10.4: AOT/EXE minimal pipeline (2–3 weeks)
- Deliverables:
  - ObjectWriter path to emit `.o` with unresolved `nyrt_*`/`nyplug_*`.
  - `libnyrt.a` minimal runtime + selected plugin `.a`.
  - Link scripts and `nyc build-aot` proof-of-concept.
  - Hello World-level standalone EXE on Linux/macOS.
- DoD:
  - `nyc build-aot <file.nyash> -o app` runs without JIT/VM.
  - Basic debug info and minimal unwind.

### 10.5: Python Integration (moved; separate phase)
- Python work is deferred to 10.5 and builds on the plugin/AOT foundation.

## Flags & Compatibility
- `NYASH_USE_PLUGIN_BUILTINS=1` – enables plugin path for builtin Boxes.
- `NYASH_JIT_HOSTCALL=1` – preserves HostCall path for comparison.
- Call conv alignment: x86_64 SysV, aarch64 AAPCS64, Win64.
- ABI version checks: `nyrt_abi_version()`, `nyplug_*_abi_version()` hard-fail on mismatch.

## Risks & Mitigations
- ABI drift: minimal headers + version checks.
- Linking complexity: start with the smallest set (Array/Print/GC-minimal), expand gradually.
- Performance: keep RO-first; benchmark and fall back to HostCall if needed.
- Windows linkage: prioritize Linux/macOS, then handle Win specifics in a follow-up task.
- JIT coverage: adopt staged lowering (NewBox→birth, Print/Debug no-op) to clear blockers; retain strict skip policy otherwise.

## References
- `c_abi_unified_design.md`
- `implementation_steps.md`
- `../phase-10.5/` (Python integration)

---

Everything is Plugin → unified paths for JIT and AOT.
