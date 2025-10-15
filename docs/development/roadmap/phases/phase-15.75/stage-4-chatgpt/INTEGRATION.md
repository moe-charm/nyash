# Integration Guide (Stage‑4 ChatGPT Plan)

Build (feature-gated)
- Cargo features: add `parser-c-abi = []` (default OFF).
- build.rs:
  ```rust
  #[cfg(feature = "parser-c-abi")]
  fn main() {
      cc::Build::new()
          .file("src/parser_harness/parser_harness.c")
          .include("src/parser_harness")
          .compile("parser_harness");
  }
  #[cfg(not(feature = "parser-c-abi"))]
  fn main() {}
  ```

Rust externs (thin adapters)
- Place near parser façade (temporary):
  ```rust
  #[repr(C)]
  pub struct HakoParseResult {
      pub abi_version: u32,
      pub struct_size: u32,
      pub success: u32,
      pub stmt_count: u32,
      pub kind: *const std::os::raw::c_char,
      pub error_msg: *const std::os::raw::c_char,
  }

  #[no_mangle]
  pub extern "C" fn parse_source_rust(src: *const std::os::raw::c_char) -> *mut HakoParseResult { /* fill minimal header */ }

  #[no_mangle]
  pub extern "C" fn parse_source_hako(src: *const std::os::raw::c_char) -> *mut HakoParseResult { /* MVP: not-implemented */ }
  ```

Runner wiring (env-driven)
- If `SMOKES_PARSER_MODE=both|hako` and feature `parser-c-abi` is enabled, call C harness; else keep current rust path.
- Both-mode only compares header fields (version/kind/stmt_count).

Smoke (one test)
- `tools/smokes/v2/profiles/quick-selfhost/parser_facade_both_min_header_vm.sh`:
  - Sets `SMOKES_PARSER_MODE=both`
  - Asserts success and header equality

Rollback
- Disable feature: `cargo build` (no harness compiled)
- No code removal; harness is additive and optional

