Nyash Dev Ideas / Roadmap Scratchpad

Purpose
- Quick scratchpad to capture ideas before they get shaped into tasks/PRs.
- Keep entries short and action-oriented; promote to RFC/docs when stable.

Near-Term (High Impact)
- Strengthen plugin ABI tester:
  - Parse `nyash.toml` and verify method_id wiring with minimal invokes (birth + no-arg methods).
  - Add contract tests for net plugin boxes (HttpRequest/Response/Client/Server).
  - Fail fast with clear diffs when IDs drift.
- Make debug logs opt-in (done):
  - `NYASH_DEBUG_PLUGIN=1` gates VM→Plugin bridge logs.
  - `NYASH_NET_LOG=1` gates net plugin logs.
- Regression tests:
  - Minimal end-to-end for HTTP: get→accept→respond→readBody (body non-empty).
  - ABI sanity tests per box type (already added for HttpRequestBox).

Identity & Copy Semantics
- Enforce `is_identity()` and `clone_or_share()` usage across interpreter paths.
- Audit remaining `clone_box()` callsites; switch to `clone_or_share()` where identity is required.
- Optional: add dev assertions when PluginBoxV2 instance_id changes unexpectedly in a single flow.

Single Source of Truth for Method IDs
- Avoid hand-edit drift between plugin code and `nyash.toml`:
  - Option A: Generate `nyash.toml` from Rust consts (build.rs or small generator).
  - Option B: Generate Rust method_id consts from `nyash.toml`.
  - Start with a checker that diffs the two (cheap safety net).

CI / Tooling
- Add `plugin_contract_*` tests to default CI.
- Add a `--ci-quiet` profile to suppress debug logs; emit logs only on failure.
- Introduce a small utility to reserve/randomize test ports to avoid collisions.

Net Plugin Polishing
- Keep TCP-only path; prefer accepted TCP requests in server.accept (done).
- Mirror response body into client handle using X-Nyash-Resp-Id (done); document behavior.
- Add timeouts and clearer error messages for readBody stalls.

Docs
- Expand `docs/development/box_identity_and_copy_semantics.md` with examples and anti-patterns.
- Add a quickstart: “How to debug the net plugin” (env flags, key logs, typical pitfalls).

Future Ideas
- Lightweight codegen for plugin schemas (IDs, arg signatures, returns_result) → toml + Rust.
- Lint/pass that forbids `clone_box()` on identity boxes in critical paths.
- Structured logging with categories and levels (trace/info/warn) pluggable to CI.

Notes
- Keep this file as a living list; prune as items graduate to tracked issues/PRs.

---

2025-08-23 Updates (VM × Plugins focus)
- VM Stats frontdoor (done): CLI flags `--vm-stats`, `--vm-stats-json`; JSON schema includes total/counts/top20/elapsed_ms.
  - Next: integrate with `--benchmark` to emit per-backend stats; add `NYASH_VM_STATS_FORMAT=json` docs.
- ResultBox in VM (done): dispatch for `isOk/getValue/getError`; generic `toString()` fallback for any Box.
  - Impact: HTTP Result paths now work end-to-end in VM.
- MIR if-merge bug (done): bind merged variable to Phi result across Program blocks (ret now returns phi dst).
  - Next: add verifier check for "use-before-def across merge"; snapshot a failing MIR pattern as a test.
- Net plugin error mapping (done): on TCP connect failure, return TLV string; loader maps to Result.Err(ErrorBox).
  - Next: formalize `returns_result` ok-type in nyash.toml (e.g., ok_returns = "HttpResponseBox"); tighten loader.
- E2E coverage (done):
  - FileBox: open/write/read, copyFrom(handle)
  - Net: GET/POST/status/header/body; 204 empty body; client error (unreachable port) → Err
  - Next: 404/5xx reply from server side; timeouts; large bodies; header casing behavior.

Short-Term TODOs
- Add vm-stats samples for normal/error HTTP flows; feed into 26-instruction diet discussion.
- CI: run `--features plugins` E2E on a dedicated job; gate on Linux only; quiet logs unless failed.
- Docs: append "VM→Plugin TLV debugging" quick tips (env flags, TLV preview).

26-Instruction Diet Hooks
- Candidate demotions: Debug/Nop/Safepoint → meta; TypeCheck/Cast → fold or verify-time.
- Keep hot path: BoxCall/NewBox/Branch/Jump/Phi/BinOp/Compare/Return.
- Track Weak*/Barrier usage; keep as extended-set unless surfaced in vm-stats.
