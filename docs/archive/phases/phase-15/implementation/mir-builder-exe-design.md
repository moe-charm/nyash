# MIR Builder EXE Design (Phase 15 — EXE‑First)

Purpose: define a standalone MIR Builder executable that takes Nyash JSON IR (v0/v1) and produces native outputs (object/executable), independent of the Rust Runner/VM. This aligns Phase‑15 with an EXE‑first delivery pipeline.

Goals
- Accept JSON IR from stdin or file and validate semantics (minimal passes).
- Emit: object (.o/.obj), LLVM IR (.ll), or final executable by linking with NyRT.
- Operate without the Rust VM path; suitable for CLI and scripted pipelines.
- Keep the boundary simple and observable (stdout diagnostics, exit codes).

CLI Interface (proposed)
- Basic form
  - `ny_mir_builder [--in <file>|--stdin] [--emit {obj|exe|ll|json}] -o <out> [options]`
  - Defaults: `--stdin`, `--emit obj`, `-o target/aot_objects/a.o`
- Options
  - `--in <file>`: Input JSON IR file (v0/v1). If omitted, read stdin.
  - `--emit {obj|exe|ll|json}`: Output kind. `json` emits validated/normalized IR for roundtrip.
  - `-o <path>`: Output path (object/exe/IR). Default under `target/aot_objects`.
  - `--target <triple>`: Target triple override (default: host).
  - `--nyrt <path>`: NyRT static runtime directory (for `--emit exe`).
  - `--plugin-config <path>`: `nyash.toml` path resolution for boxes/plugins.
  - `--quiet`: Minimize logs; only errors to stderr.
  - `--validate-only`: Parse+validate IR; no codegen.
  - `--verify-llvm`: Run LLVM verifier on generated IR (when `--emit {obj|exe}`).

Exit Codes
- `0` on success; >0 on error. Validation errors produce human‑readable messages on stderr.

Input IR
- JSON v0 (current Bridge spec). Unknown fields are ignored; `meta.usings` is accepted.
- Future JSON v1 (additive) must remain backward compatible; builder performs normalization.

Outputs
- `--emit obj`: Native object file. Uses LLVM harness internally.
- `--emit ll`: Dumps LLVM IR for diagnostics.
- `--emit exe`: Produces a self‑contained executable by linking the object with NyRT.
- `--emit json`: Emits normalized MIR JSON (post‑validation) for roundtrip tests.

Packaging Forms
- CLI executable: `ny_mir_builder` (primary).
- Optional shared lib: `libny_mir_builder` exposing a minimal C ABI for embedding.

C ABI Sketch (optional library form)
```c
// Input: JSON IR bytes. Output: newly allocated buffer with object bytes.
// Caller frees via ny_free_buf.
int ny_mir_to_obj(const uint8_t* json, size_t len,
                  const char* target_triple,
                  uint8_t** out_buf, size_t* out_len);

// Convenience linker: object → exe (returns 0=ok).
int ny_obj_to_exe(const uint8_t* obj, size_t len,
                  const char* nyrt_dir, const char* out_path);

void ny_free_buf(void* p);
```

Internal Architecture
- Frontend
  - JSON IR parser → AST/CFG structures compatible with existing MIR builder expectations.
  - Validation passes: control‑flow well‑formedness, PHI consistency (incoming edges), type sanity for BoxCall/ExternCall minimal set.
- Codegen
  - LLVM harness path (current primary). Environment fallback via `LLVM_SYS_180/181_PREFIX`.
  - Option flag `NYASH_LLVM_FEATURE=llvm|llvm-inkwell-legacy` maintained for transitional builds.
- Linking (`--emit exe`)
  - Use `cc` with `-L <nyrt>/target/release -lnyrt` (static preferred) + platform libs `-lpthread -ldl -lm` (Unix) or Win equivalents.
  - Search `nyash.toml` near the output exe and current CWD (same heuristic as NyRT runtime) to initialize plugins at runtime.

Integration Points
- Parser EXE → MIR Builder EXE
  - `./nyash_compiler <in.nyash> | ny_mir_builder --stdin --emit obj -o a.o`
  - Compose with link step for quick end‑to‑end: `... --emit exe -o a.out`
- Runner (future option)
  - `NYASH_USE_NY_COMPILER_EXE=1`: Runner spawns parser EXE; optionally chain into MIR Builder EXE for AOT.
  - Fallback to in‑proc Bridge when EXE pipeline fails.

Logging & Observability
- Default: single‑line summaries on success, detailed errors on failure.
- `--quiet` to suppress non‑essential logs; `--verify-llvm` to force verification.
- Print `OK obj:<path>` / `OK exe:<path>` on success (stable for scripts).

Security & Sandboxing
- No arbitrary file writes beyond `-o` and temp dirs.
- Deny network; fail fast on malformed JSON.

Platform Notes
- Windows: `.obj` + link with MSVC or lld‑link; prefer bundling `nyrt` artifacts.
- macOS/Linux: `.o` + `cc` link; RPATH/loader path considerations documented.

Incremental Plan
1) CLI skeleton: stdin/file → validate → `--emit json/ll` (dry path) + golden tests。
2) Hook LLVM harness: `--emit obj` for const/arith/branch/ret subset。
3) Linker integration: `--emit exe` with NyRT static lib; add platform matrices。
4) Parity suite: run produced EXE against known cases (strings/collections minimal)。
5) Packaging: `tools/build_mir_builder_exe.sh` + smoke `tools/mir_builder_exe_smoke.sh`。

Reference Artifacts
- `tools/build_llvm.sh`: current harness flow used as a baseline for emission and link steps。
- `crates/nyrt`: runtime interface and plugin host initialization heuristics。

