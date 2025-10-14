# Stage‑4 (ChatGPT Plan) — QUICKSTART

Goal
- Land a thin C ABI harness (2 functions), export 2 Rust externs, and wire a minimal both-mode smoke that compares only header fields.

Day 1 — C harness + build wiring
- Files to add:
  - `src/parser_harness/parser_harness.h`
  - `src/parser_harness/parser_harness.c`
- Build integration (feature-gated):
  - `build.rs` add:
    ```rust
    #[cfg(feature = "parser-c-abi")]
    fn main() { cc::Build::new().file("src/parser_harness/parser_harness.c").include("src/parser_harness").compile("parser_harness"); }
    #[cfg(not(feature = "parser-c-abi"))]
    fn main() {}
    ```
  - `Cargo.toml` add feature `parser-c-abi = []` and `cc = "*"` (build-dependency).

Day 1 — C ABI (minimal)
- Header (see C_ABI_MIN_SPEC.md):
  - `HakoParseMode { RUST=0, HAKO=1, BOTH=2 }`
  - `HakoParseResult { abi_version, struct_size, success, stmt_count, kind, error_msg }`
  - `parse_source_dual(const char*, HakoParseMode) -> HakoParseResult*`
  - `free_parse_result(HakoParseResult*)`
- Implementation:
  - For BOTH: call `parse_source_rust` and `parse_source_hako`, compare `stmt_count/kind`, synthesize error on mismatch.
  - Strings are heap-owned; must be freed in `free_parse_result`.

Day 2 — Rust externs + façade wiring
- Rust exports (temporary location near front parser façade):
  - `#[no_mangle] extern "C" fn parse_source_rust(src: *const c_char) -> *mut HakoParseResult { ... }`
  - `#[no_mangle] extern "C" fn parse_source_hako(src: *const c_char) -> *mut HakoParseResult { /* stub initially */ }`
  - Both fill the minimal header; `parse_source_hako` returns success=0, error_msg="not-implemented" for MVP.
- Runner wiring (feature-gated):
  - If `SMOKES_PARSER_MODE=both|hako`, call C harness; else call existing rust path.

Smoke (one test)
- Add: `tools/smokes/v2/profiles/quick-selfhost/parser_facade_both_min_header_vm.sh`
  - Sets `SMOKES_PARSER_MODE=both` and feeds a tiny program (with semicolons).
  - Checks: success=1, version/kind equal, stmt_count equal.

Build/Run
- Plugin-only unaffected. For C harness line:
  - `cargo build --features parser-c-abi`
  - `tools/smokes/v2/run.sh --profile quick-selfhost --filter parser_facade_both_min_header_vm.sh`

Rollback
- Disable feature: `cargo build` (no harness compiled). No code removal required.

