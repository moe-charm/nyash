# Parser MVP Stage-3 Design (Phase 15)

Scope
- Extend Stage-2 parser emission to cover control flow constructs usually seen in routine code bases:
  - `break` / `continue`
  - `throw expr`
  - `try { ... } catch (Type err) { ... } finally { ... }`
  - Alert: other Stage-3 ideas (switch/async) remain out of scope until after self-host parity.
- Preserve existing Stage-2 behaviour (locals/loop/if/call/method/new/ternary) with no regressions.

Guiding Principles
- JSON v0 must remain stable for the Stage-2 path; Stage-3 additions should be feature-flagged or degrade safely when disabled.
- Short-circuit semantics are already mirrored via logical nodes; Stage-3 should reuse the same block-building infrastructure (Bridge/VM/JIT) to avoid special cases.
- Continue the "degrade to expression" approach when code generation is not ready (e.g. throw/try) so that Stage-2 tests stay green while the full implementation is developed.

Current Status (Phase 15.3 – 2025-09-16)
- ParserBox / Selfhost compiler expose `stage3_enable` and `--stage3` CLI flag, defaulting to the safe Stage-2 surface.
- Break/Continue JSON emission and Bridge lowering are implemented. Bridge now emits `Jump` to loop exit/head and records instrumentation events.
- Throw/Try nodes are emitted when the gate is on, but still degrade to expression/no-op during lowering; runtime semantics remain TBD.
- Documentation for JSON v0 (Stage-3 nodes) is updated; remaining runtime work is tracked in CURRENT_TASK.md.

Runtime snapshot
- MIR Builder already lowers `ASTNode::Throw` into `MirInstruction::Throw` (unless disabled via `NYASH_BUILDER_DISABLE_THROW`) and has a provisional `build_try_catch_statement` that emits `Catch`/`Jump` scaffolding with env flags controlling fallback.
- Rust VM (`interpreter::ControlFlow::Throw`) supports catch/finally semantics and rethrows unhandled exceptions.
- Bridge degradation prevents these MIR paths from activating unless `NYASH_BRIDGE_THROW_ENABLE=1`;既定では Const0 を出し、フラグONで実際に `Throw` を生成する。

PyVM plan
- Current PyVM runner treats Stage-3 constructs as no-ops because JSON v0 never emits MIR throws; once Bridge emits them, PyVM must mirror Rust VM semantics:
  - Introduce a lightweight `exception` representation (reuse ErrorBox JSON form) and propagate via structured returns.
  - Implement try/catch/finally execution order identical to Rust VM (catch matches first, finally always runs, rethrow on miss).
  - Add minimal smoke tests under `tools/pyvm_stage2_smoke.sh` (gated) to ensure PyVM and LLVM stay in sync when Stage-3 is enabled.

LLVM plan
- Short term: continue degrading throw/try to keep LLVM pipeline green while implementation lands (Stage-3 smoke ensures awareness).
- Implementation steps once runtime semantics are ready:
  1. Ensure MIR output contains `Throw`/`Catch` instructions; update LLVM codegen to treat `Throw` as a call to a runtime helper (`nyash.rt.throw`) that unwinds or aborts.
  2. Model catch/finally blocks using landing pads or structured IR (likely via `invoke`/`landingpad` in LLVM); document minimal ABI expected from NyRT.
  3. Add gated smoke (`NYASH_LLVM_STAGE3_SMOKE`) that expects non-degraded behaviour (distinct exit codes or printed markers) once helper is active.
- Until landing pad support exists, document that Stage-3 throw/try is unsupported in LLVM release mode and falls back to interpreter/PyVM.

Testing plan
- JSON fixtures: create `tests/json_v0_stage3/{break_continue,throw_basic,try_catch_finally}.json` to lock parser/bridge output and allow regression diffs.
- PyVM/VM: extend Stage-3 smoke scripts with throw/try cases (under gate) to ensure runtime consistency before enabling by default.
- LLVM: `NYASH_LLVM_STAGE3_SMOKE=1` は `NYASH_BRIDGE_THROW_ENABLE=1` / `NYASH_BRIDGE_TRY_ENABLE=1` と組み合わせて実際の例外経路を確認。将来的に常時ONへ移行予定。
- CI gating: add optional job that runs Stage-3 smokes (PyVM + LLVM) nightly to guard against regressions while feature is still experimental.

JSON v0 Additions
| Construct  | JSON v0 Node                                    | Notes |
|------------|-------------------------------------------------|-------|
| break      | `{ "type": "Break" }`                          | Lowered into loop exit block with implicit jump. |
| continue   | `{ "type": "Continue" }`                       | Lowered into loop head block jump. |
| throw expr | `{ "type": "Throw", "expr": Expr }`           | Initial implementation can degrade to `{ "type": "Expr", "expr": expr }` until VM/JIT semantics are ready. |
| try/catch/finally | `{ "type": "Try", "try": Stmt[], "catches": Catch[], "finally": Stmt[]? }` | Each `Catch` includes `{ "param": String?, "body": Stmt[] }`. Stage-1 implementation may treat as pass-through expression block. |

Lowering Strategy (Bridge)
1. **Break/Continue**
   - Bridge stores loop header/exit blocks on a loop stack.
   - `Break` maps to `Jump { target: loop_exit }`, `Continue` to `Jump { target: loop_head }`.
   - MirBuilder already has `LoopBuilder`; expose helpers to fetch head/exit blocks.

2. **Throw/Try**
   - Phase 15 MVP keeps them syntax-only to avoid VM/JIT churn. Parser/Emitter produce nodes; Bridge either degrades (Expr) or logs a structured event for future handling.
  - Bridge helper `lower_throw` respects `NYASH_BRIDGE_THROW_ENABLE=1`; defaultは Const i64 0 のデグレード、フラグONで `MirInstruction::Throw` を実際に生成。
   - Try lowering plan:
     1. Parse-time JSON already includes `catches`/`finally`. Bridge should map `try` body into a fresh region, emit basic blocks for each `catch`, and wire `finally` as a postamble block.
     2. MIR needs explicit instructions/metadata for exception edges. Evaluate whether existing `MirInstruction::Throw` + `ControlFlow::Throw` is sufficient or if `Catch` terminators are required.
     3. Until runtime implementation lands, keep current degrade path but log a structured event to flag unhandled try/catch.

3. **Metadata Events**
   - Augment `crate::jit::observe` with `lower_shortcircuit`/`lower_try` stubs so instrumentation remains coherent when full support is wired.

Testing Plan
- Extend selfhost Stage-2 smoke file with guard cases (`return break` etc.) once lowering is live.
- Create dedicated JSON fixtures under `tests/json_v0_stage3/` for break/continue/try once behaviour stabilises.
- Update `tools/ny_stage2_shortcircuit_smoke.sh` to ensure Stage-3 constructs do not regress Stage-2 semantics (break/continue degrade when gate off, jump when on).

Migration Checklist
1. ParserBox emits Stage-3 nodes under `stage3_enable` gate to allow gradual rollout. ✅
2. Emitter attaches Stage-3 JSON when gate is enabled (otherwise degrade to existing Stage-2 forms). ✅
3. Bridge honours Stage-3 nodes when gate is on; break/continue lowering implemented, throw/try still degrade. ✅ (partial)
4. PyVM/VM/JIT semantics gradually enabled (throw/try remain degrade until corresponding runtime support is merged). 🔄 Pending runtime work.
5. Documentation kept in sync (`CURRENT_TASK.md`, release notes). ✅ (break/continue) / 🔄 (throw/try runtime notes).

References
- Stage-2 design (`parser_mvp_stage2.md`)
- CURRENT_TASK stage checklist (Phase 15)
- `docs/guides/language-guide.md` section “Exceptions & Flow Control” (update when Stage-3 fully lands).
