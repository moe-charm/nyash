# CURRENT TASK — Self‑Host Quick Resume (Phase 15)

Scope
- This repo/folder tracks the Self‑Hosting line (VM/LLVM first). WASM work lives in a separate folder/branch. Ignore WASM commits here except the minimal shared specs.

Status — Pre‑restart checks (done)
- Pushed to selfhost branch: recent fixes are on record
  - PHI JSON format unified to `values[]` (no `incoming` in output)
    - commit: 5e7bc9ea
  - CLI `--entry` wired directly to VM entry resolution (Strict `Main.main`, CLI override)
    - commit: 7c500bae
  - README and smokes updated to prefer `hako` CLI; `nyash` kept as alias (deprecation banner only)
    - commits: 57151b1c, a4fe896c
  - Rust VM array output/prints stabilization (ArrayBox + collect_prints)
    - commits: c2e4eeae, 41f0cf6b

Resume after restart
- Build
  - `cargo build --release`
  - Binary: `target/release/hako`（nyash は互換エイリアスがある環境も）
- Quick smokes (optional)
  - `SMOKES_ENABLE_ENTRY=1 NYASH_DISABLE_PLUGINS=1 tools/smokes/v2/profiles/quick/core/cli_entry_ok.sh`
  - Run entry‑gated cases with the env gate ON
- Representative strict runs
  - `./target/release/hako --backend vm apps/APP/main.nyash`
  - Alternate entry: `./target/release/hako --backend vm --entry App.main apps/APP/main.nyash`

Notes
- PHI JSON is unified to `values[]`; emitters must not output `incoming`. Readers accept `values` primarily and may accept legacy `incoming` for compatibility.
- CLI naming: バイナリは `hako`。`nyash` は環境により互換で利用可能。
- Entry smokes are gated via `SMOKES_ENABLE_ENTRY=1` by design.

Next actions
- [x] Add/update CURRENT_TASK for selfhost (this file)
- [x] Append a short PHI `values` spec to reference docs
- [x] Enable commit/push for selfhost branches (local hooks updated)
- [ ] If HEAD points to a WASM‑prefixed branch, switch back to the selfhost/main branch before feature work (no code change here).

Audit trail (relevant commits)
- 5e7bc9ea — llvm_py(phi): PHI 'values' 形式完全統一（selfhost移植完了）
- 7c500bae — runner: add EntryResolveBox + CLI --entry (Strict Main.main; CLI override). Docs: entry policy
- 57151b1c / a4fe896c — CLI/docs: add `hako` bin; prefer `hako`; keep `nyash` alias
- c2e4eeae / 41f0cf6b — VM/runner: ArrayBox/collect_prints の出力安定化

Updated: 2025‑10‑01
