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
   - Document expectation: once runtime exception model is defined, nodes become non-degrading.

3. **Metadata Events**
   - Augment `crate::jit::observe` with `lower_shortcircuit`/`lower_try` stubs so instrumentation remains coherent when full support is wired.

Testing Plan
- Extend selfhost Stage-2 smoke file with guard cases (`return break` etc.) once lowering is live.
- Create dedicated JSON fixtures under `tests/json_v0_stage3/` for break/continue/try once behaviour stabilises.
- Update `tools/ny_stage2_shortcircuit_smoke.sh` to ensure Stage-3 constructs do not regress Stage-2 semantics (break/continue degrade). Timing: after lowering is implemented.

Migration Checklist
1. ParserBox emits Stage-3 nodes under `NYASH_PARSER_STAGE3=1` gate to allow gradual rollout.
2. Emitter attaches Stage-3 JSON when gate is enabled (otherwise degrade to existing Stage-2 forms).
3. Bridge honours Stage-3 nodes when gate is on; degrade with warning when off.
4. PyVM/VM/JIT semantics gradually enabled (throw/try remain degrade until corresponding runtime support is merged).
5. Documentation kept in sync (`CURRENT_TASK.md`, release notes).

References
- Stage-2 design (`parser_mvp_stage2.md`)
- CURRENT_TASK stage checklist (Phase 15)
- `docs/guides/language-guide.md` section “Exceptions & Flow Control” (update when Stage-3 fully lands).
