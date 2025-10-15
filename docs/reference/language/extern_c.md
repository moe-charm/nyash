# extern_c — Dynamic FFI (15.76 MVP)

Purpose
- Minimal foreign function call from Hakorune code for bootstrap.
- Syntax: `extern_c "symbol" (args...)` where args are converted to C strings.

Lowering
- Builder emits `Call { callee = Extern("ffi.dynamic.<symbol>") }`.
- Effects: IO (conservative default).

VM Semantics
- Whitelisted dynamic calls only (Stage‑1): `getpid`, `strlen`, `system`.
- Arity: 0/1/2. Arguments are converted via `.to_string()` → `CString`.
- Return type: `i64` (return value is wrapped as Integer).
- Denied symbols produce `InvalidInstruction: ffi: symbol not allowed`.

Examples
```
static box Main {
  main() {
    local n; n = extern_c "strlen" ("hello");
    print(n);      // 5
    return 0;
  }
}
```

LLVM backend helpers (optional)
- The workspace provides a native library `libllvm_backend` exposing:
  - `llvm_compile_mir_to_object(mir_json, out_o)` — emits an object file
  - `llvm_compile_mir_to_ll(mir_json, out_ll)` — emits LLVM IR (.ll)

Example (.ll emit):
```
static box Main {
  main() {
    local rc; rc = extern_c "llvm_compile_mir_to_ll" ("build/program.mir.json", "build/program.ll");
    return rc;
  }
}
```

Notes
- Linux loads from `libc.so.6` (macOS: `libSystem.B.dylib`, Windows: `ucrtbase.dll|msvcrt.dll`).
- Allow expansion via env: `HAKO_FFI_ALLOW_LIST=foo,bar` merges into allowlist.
- Dev override: `HAKO_FFI_ALLOW_ALL=1` bypasses the list (not recommended in CI).

TOML
- You can add permanent project entries in `hako.toml` (or `nyash.toml`):
```
[ffi.dynamic]
allow = ["strlen", "getpid", "system", "llvm_compile_mir_to_object"]
```
Resolution precedence (top wins): CLI (future) → ENV → TOML → compiled‑in minimal.
