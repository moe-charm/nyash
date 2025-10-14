#ifndef HAKO_PARSER_HARNESS_H
#define HAKO_PARSER_HARNESS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HAKO_PARSER_MODE_RUST = 0,
    HAKO_PARSER_MODE_HAKO = 1,
    HAKO_PARSER_MODE_BOTH = 2
} HakoParseMode;

typedef struct HakoParseResult {
    uint32_t abi_version;
    uint32_t struct_size;
    uint32_t success;      /* 1=ok, 0=error */
    uint32_t stmt_count;   /* minimal comparable stat */
    const char* kind;      /* e.g., "Program" */
    const char* error_msg; /* nullable */
} HakoParseResult;

/* Rust-side helpers (implemented in src/front/parser_layer/c_abi_bridge.rs) */
int hako_parse_source_rust_header(const char* src_utf8, uint32_t* out_stmt_count,
                                  const char** out_kind, const char** out_error);
int hako_parse_source_hako_header(const char* src_utf8, uint32_t* out_stmt_count,
                                  const char** out_kind, const char** out_error);
const char* hako_rust_make_cstring(const char* src_utf8);
void hako_rust_free_cstr(const char* s);

/* C harness API */
HakoParseResult* parse_source_dual(const char* source_utf8, HakoParseMode mode);
void free_parse_result(HakoParseResult* result);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* HAKO_PARSER_HARNESS_H */

