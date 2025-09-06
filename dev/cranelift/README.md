Cranelift JIT/AOT Dev

Focus: Cranelift JIT‑direct and AOT (object emit + link with `libnyrt.a`).

Quick AOT Smoke

- Build core (once):
  - `cargo build --release --features cranelift-jit`
- Lower to object + link with NyRT:
  - `NYASH_DISABLE_PLUGINS=1 tools/aot_smoke_cranelift.sh apps/smokes/jit_aot_string_min.nyash app_str`
- Run app:
  - `./app_str`

Useful env toggles

- `NYASH_JIT_DUMP=1`: show JIT lowering summary
- `NYASH_JIT_TRACE_LOCAL=1`: trace local slot loads/stores
- `NYASH_JIT_TRACE_RET=1`: trace return path
- `NYASH_JIT_TRACE_LEN=1`: trace string/any len thunks
- `NYASH_JIT_DISABLE_LEN_CONST=1`: disable early const‑fold for String.length

Notes

- For AOT linking: requires `libnyrt.a` from `crates/nyrt` (built by `cargo build --release`).
- Use `target/aot_objects/` as scratch; keep per‑experiment subfolders if needed.

