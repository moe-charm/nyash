Build Logs — Quieting Guidelines (Phase 15.5)

Scope
- Reduce noisy warnings in routine builds while keeping important diagnostics visible.

Key changes (implemented)
- Stub historical feature: add `cranelift-jit = []` to workspace `Cargo.toml` so `#[cfg(feature = "cranelift-jit")]` guards no longer trigger `unexpected cfg` warnings.
- Plugin profiles: remove `[profile.release]` sections from plugin crates (`plugins/*/Cargo.toml`). Cargo ignores non-root profiles and emits warnings; profiles should be defined at the workspace root only.
- LLVM harness notes: the runner sets `NYASH_LLVM_SANITIZE_EMPTY_PHI=1` by default (unless explicitly set to `0`) to drop malformed empty PHIs and group PHIs at block heads in IR text before verification.
- Smoke scripts: use `NYASH_NYRT_SILENT_RESULT=1` to suppress the runtime’s final `Result: <n>` line when comparing program output lines.

Recommended usage
- Builds: prefer `cargo build -q` (already used in scripts) to reduce cargo chatter.
- Tests/Smokes: when golden-comparing program prints, add `NYASH_NYRT_SILENT_RESULT=1` so only script prints appear.

Backout/Notes
- The stub feature is a no-op and OFF by default; it can be removed once archived cfg paths are deleted.
- If you need per-crate profile tuning, migrate the relevant `[profile.*]` keys to the workspace `Cargo.toml`.

