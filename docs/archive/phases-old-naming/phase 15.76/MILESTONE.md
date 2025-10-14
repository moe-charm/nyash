# Phase 15.76 — extern_c / Bootstrap FFI Milestones

Status: Week‑1 (extern_c MVP) — done

Scope
- Introduce a minimal, safe dynamic FFI surface to bootstrap the self‑host path.
- Keep default strict (deny by default). Open only what we need per profile.

What’s shipped (Week‑1)
- Syntax: `extern_c "symbol"(args)` → AST `ExternCCall`.
- MIR: `Call{ callee=Extern("ffi.dynamic.<symbol>") }` (unified MirCall).
- VM: dynamic FFI for 0/1/2 args (CString → i64), deny‑by‑default with a small allowlist.
- Allowlist (compiled‑in, MVP): `getpid`, `strlen`, `system`.
- Dev override: `HAKO_FFI_ALLOW_ALL=1`.
- Smokes: quick‑selfhost `extern_c_{getpid,strlen,system}_vm.sh`.

Next (Week‑2) — Config‑backed allowlist + native backend (no behavior change by default)
- Keep compiled‑in defaults for safety; allow opt‑in expansion via config.
- Sources and precedence (highest → lowest):
  1) CLI (future) `--ffi-allow <csv>` (optional)
  2) ENV `HAKO_FFI_ALLOW_LIST=foo,bar`
  3) Project config `hako.toml`:
     ```toml
     [ffi.dynamic]
     allow = ["strlen", "getpid", "system"]
     ```
  4) Compiled‑in defaults (MVP set above)

Design notes
- Registry: a tiny adapter that merges the 3 layers above into a HashSet at VM init.
- Security: default = strict deny; config only broadens the set. No silent fallbacks.
- Effects: keep conservative IO unless a symbol is explicitly annotated later.
- Observability: `HAKO_DEBUG_FFI=1` to print resolved library + symbol at call time (dev only).

DoD (Week‑2)
- ENV `HAKO_FFI_ALLOW_LIST` respected; merging with defaults. (DONE)
- VM dynamically locates `libllvm_backend` and resolves custom symbols. (DONE)
- Optional TOML `[ffi.dynamic].allow` respected; merging with env. (NEXT)
- Smokes: one case proving ENV expansion; deny case continues to fail fast. (ADDED, runner整備は後続)
- Docs: extern_c + frozen-toolchain updated with allowlist/lib search. (DONE)

Week‑3 (complete)
- Added `llvm_compile_mir_to_ll`（.ll 出力） in `libs/llvm_backend` and harness `--emit-ll`.
- Added minimal link wrapper `tools/aot/link_with_clang.sh`（開発専用）。
- 固定化: 凍結EXEミント手順（docs/guides/frozen-toolchain.md）を具体化しチェックリスト更新。

Later
- Native lib `libllvm_backend` and AOT hooks per 15.76 TODO.
- Optional per‑symbol effects table (read/IO/control) in config; VM enforces plus verifier hints.
- Per‑profile presets (quick/plugins/ci) via `tools/smokes/v2/configs/env/*.env` or profile overlays.

Risks & Mitigations
- Over‑permissive configs leaking into CI → keep defaults strict; profiles must opt‑in.
- Platform libc resolution differences → we pin known C runtimes (libc.so.6/libSystem/ucrtbase/msvcrt).

Rollback
- Flip back to compiled‑in allowlist by ignoring ENV/TOML (guard behind a feature flag when wiring, default ON).
Goal Line (DoD)
- Syntax/MIR/VM: `extern_c` 受理 → `Callee::Extern("ffi.dynamic.*")` → VM動的FFI（0/1/2, CString→i64, Fail‑Fast）
- 既定はDeny（最小Allow: getpid/strlen/system）。
- 設定ベース許可: ENV `HAKO_FFI_ALLOW_LIST`, TOML `[ffi.dynamic].allow` をマージ（`HAKO_FFI_ALLOW_ALL` は開発専用）
- バックエンドlib: `libs/llvm_backend` を `cargo build -p llvm_backend` で生成。`llvm_compile_mir_to_object` を提供。
- VMは lib 探索（`target/release`, `$NYASH_ROOT/target/release`, `.` , `HAKO_FFI_LIB_PATHS`）。
- AOT導線（最小）: `--emit-mir-json` → extern_c で `.o` → clangでリンク。
- Docs: extern_c / frozen-toolchain / milestone の整備。
