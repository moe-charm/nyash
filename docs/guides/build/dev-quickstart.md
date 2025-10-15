# Developer Quickstart

This quickstart summarizes the most common build/run/test flows when working on Nyash.

See also
- Self‑hosting one‑pager: `docs/how-to/self-hosting.md`

## Build
- VM/JIT (Cranelift): `cargo build --release --features cranelift-jit`
- LLVM AOT: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`

## Run
- Quick VM run: `./target/release/nyash --backend vm apps/APP/main.nyash`
- LLVM emit+link helper: `tools/build_llvm.sh apps/APP/main.nyash -o app`

## Smokes
- End‑to‑end LLVM smokes: `./tools/llvm_smoke.sh release`
  - Use env toggles like `NYASH_LLVM_VINVOKE_RET_SMOKE=1`

## Syntax Sugar (Phase 12.7)
- Control via env: `NYASH_SYNTAX_SUGAR_LEVEL=none|basic|full`
- Or via `nyash.toml`:
  ```toml
  [syntax]
  sugar_level = "none" # or "basic" / "full"
  ```

## MIR Migration Notes
- Implementation currently enforces Core‑15 during migration with tests in place.
- Core‑13 is the final minimal kernel target. The final flip documentation is tracked under:
  `docs/development/roadmap/mir/core-13/step-50/`.

## Testing
- Rust unit tests: `cargo test`
- Targeted: e.g., tokenizer/sugar config `cargo test --lib sugar_basic_test -- --nocapture`

## Acceptance Checklist (Feature Additions Pause)
- cargo check (workspace) passes
- Representative smokes are green:
  - PyVM smokes: `tools/pyvm_stage2_smoke.sh`
  - LLVM harness smokes: `tools/llvm_smoke.sh release`
  - Macro goldens: `tools/ci_check_golden.sh`
- No spec changes (compat maintained); changes are minimal and local
