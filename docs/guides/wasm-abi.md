WASM ABI for Hakorune (Draft)

Scope
- Minimal ABI to let Hakorune boxes (written in .hako) operate over WASM linear memory.
- Keep core small and portable; let boxes own policies (bounds, growth, copying).

Principles
- Box-first: Array/Map/String are implemented in Hakorune; core only offers memory ops.
- Single extern_call surface: boxes call nykernel.*; host provides implementation.
- Fail-Fast: boxes validate inputs; nykernel functions do not validate.

ABI (required exports)
- nykernel.malloc(size: i64) -> i64
- nykernel.load_i64(addr: i64) -> i64
- nykernel.store_i64(addr: i64, val: i64) -> void

Box responsibilities
- Track ptr/len/cap (for arrays) and run-time checks.
- Resize by allocating a new region and copying existing elements.
- Throw on out-of-bounds or invalid state; no silent fallback.

Host responsibilities
- Provide nykernel.* implementations per backend:
  - WASM: implemented in Rust crate nykernel-wasm (wasm32 target).
  - VM/LLVM dev: stub map nykernel.* to safe host functions (dev only).

Testing
- Start with VM stubs for extern_call("nykernel.*", ...). WASM linking is opt-in.
- Add small smokes for push/get/resize; keep gated (env flag) until wiring completes.

Notes
- Pointers are i64 (byte offsets). Alignment: 8 for i64 arrays.
- Future ops (optional): memcpy, memset; keep ABI minimal until needed.

