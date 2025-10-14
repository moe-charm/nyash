# Minimal C ABI Spec (Stage‑4 ChatGPT Plan)

Stability
- Use a fixed header with `abi_version` and `struct_size` for forward/backward compatibility.
- All strings are UTF‑8, heap-allocated by the callee, and freed by `free_parse_result`.

C Types
```
#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

typedef enum {
    HAKO_PARSER_MODE_RUST = 0,
    HAKO_PARSER_MODE_HAKO = 1,
    HAKO_PARSER_MODE_BOTH = 2
} HakoParseMode;

typedef struct HakoParseResult {
    uint32_t abi_version;   /* must be 1 */
    uint32_t struct_size;   /* sizeof(HakoParseResult) at build time */
    uint32_t success;       /* 1=ok, 0=error */
    uint32_t stmt_count;    /* minimal stat for parity */
    const char* kind;       /* e.g., "Program" (owned by result) */
    const char* error_msg;  /* nullable; owned by result */
} HakoParseResult;

/* Returns heap-allocated result (caller must free with free_parse_result) */
HakoParseResult* parse_source_dual(const char* source_utf8, HakoParseMode mode);

/* Frees result + owned strings (kind, error_msg) */
void free_parse_result(HakoParseResult* result);

#ifdef __cplusplus
}
#endif
```

Semantics
- RUST: run Rust parser; fill header; `error_msg=NULL` on success.
- HAKO: run Hakorune parser; same contract. Initially, allowed to return success=0 with `"not-implemented"`.
- BOTH: call both; if either fails, return the error. If both succeed, compare `stmt_count` and `kind`; on mismatch set `success=0` with a short diagnostic string (e.g., `"mismatch: stmt rust=3 hako=2"`).

Memory
- Allocate `HakoParseResult` and any non-NULL strings via `malloc`.
- `free_parse_result` must free `kind`, `error_msg` if non-NULL, then `result`.

Versioning
- `abi_version=1` for the first release.
- Future fields must be appended; callers ignore unknown tail by checking `struct_size`.

