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

