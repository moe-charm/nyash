# LLVM Backend via Hako ABI — File Handoff (Draft)

Purpose
- Define a small, stable boundary to drive LLVM from Hakorune using a file handoff.
- Make the execution path easy to swap later (llvmlite → pure‑Rust) without touching call sites.

Principles
- Single source of truth: MIR(JSON v0) with `{ kind: "MIR", schema_version: "1.0" }`.
- File I/O only: callers write JSON to a file; backend reads it and emits an object/exe file.
- Fail‑Fast: non‑zero exit or missing outputs are errors.

Proposed Hako ABI (minimal)
- `LlvmBackend.compile_obj(json_path: String) -> String`  // returns obj path, or throws error
- `LlvmBackend.link_exe(obj_path: String, out_path: String, libs?: String) -> Bool`  // true on success

Reference implementation (current)
- Harness: Python llvmlite (tools/llvmlite_harness.py) compiles MIR(JSON) → object (.o)
- Wrapper: ny-llvmc (Rust) invokes the harness and then links with libhako_kernel.a
- AOT helper: tools/build_llvm.sh orchestrates CLI, object emission and linking

Roadmap
- Phase A: keep using ny-llvmc + llvmlite, formalize the file API and add fallbacks/timeouts.
- Phase B: replace harness with pure‑Rust backend; retain the same file ABI and method names.

Related docs
- docs/reference/mir/json-v0-schema.md

Async considerations
- The LLVM handoff remains file‑based; async does not change the ABI.
- When producing EXEs for async tests, prefer ny‑llvmc (crate route). The llvmlite harness is allowed to SKIP advanced async (spawn) until builder parity lands.
- Determinism: keep async scheduling within the process (no OS threads required); file handoff is still synchronous at compile‑time.
