# nykernel-wasm (draft)

Minimal WASM-side ABI for Hakorune boxes.

Exports (WASM):
- `nykernel_malloc(size: i64) -> i64`
- `nykernel_load_i64(addr: i64) -> i64`
- `nykernel_store_i64(addr: i64, val: i64)`

Notes
- Simple bump allocator; not production grade.
- Leave validation to Hakorune boxes (Fail-Fast at the caller).
- Not wired in workspace by default; build with `--target wasm32-unknown-unknown`.

