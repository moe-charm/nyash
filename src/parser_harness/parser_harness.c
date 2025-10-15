#include "parser_harness.h"
#include <stdlib.h>
#include <string.h>

static HakoParseResult* alloc_result(void) {
    HakoParseResult* r = (HakoParseResult*)malloc(sizeof(HakoParseResult));
    if (!r) return NULL;
    r->abi_version = 1;
    r->struct_size = (uint32_t)sizeof(HakoParseResult);
    r->success = 0;
    r->stmt_count = 0;
    r->kind = NULL;
    r->error_msg = NULL;
    return r;
}

HakoParseResult* parse_source_dual(const char* source_utf8, HakoParseMode mode) {
    HakoParseResult* r = alloc_result();
    if (!r) return NULL;
    uint32_t sc_r = 0, sc_h = 0;
    const char *kind_r = NULL, *err_r = NULL;
    const char *kind_h = NULL, *err_h = NULL;

    if (mode == HAKO_PARSER_MODE_RUST) {
        int ok = hako_parse_source_rust_header(source_utf8, &sc_r, &kind_r, &err_r);
        r->success = (ok != 0);
        r->stmt_count = sc_r;
        r->kind = kind_r;
        r->error_msg = err_r;
        return r;
    }
    if (mode == HAKO_PARSER_MODE_HAKO) {
        int ok = hako_parse_source_hako_header(source_utf8, &sc_h, &kind_h, &err_h);
        r->success = (ok != 0);
        r->stmt_count = sc_h;
        r->kind = kind_h;
        r->error_msg = err_h;
        return r;
    }
    if (mode == HAKO_PARSER_MODE_BOTH) {
        int ok_r = hako_parse_source_rust_header(source_utf8, &sc_r, &kind_r, &err_r);
        int ok_h = hako_parse_source_hako_header(source_utf8, &sc_h, &kind_h, &err_h);
        if (ok_r && ok_h) {
            if (sc_r == sc_h && kind_r && kind_h && strcmp(kind_r, kind_h) == 0) {
                r->success = 1;
                r->stmt_count = sc_r;
                r->kind = kind_r; /* prefer rust */
                if (kind_h && kind_h != kind_r) hako_rust_free_cstr(kind_h);
                if (err_r) hako_rust_free_cstr(err_r);
                if (err_h) hako_rust_free_cstr(err_h);
                return r;
            } else {
                /* mismatch: synthesize error */
                const char* prefix = "mismatch: stmt ";
                char buf[128];
                snprintf(buf, sizeof(buf), "mismatch: stmt rust=%u hako=%u", sc_r, sc_h);
                r->error_msg = hako_rust_make_cstring(buf);
                r->success = 0;
                r->stmt_count = sc_r;
                r->kind = kind_r;
                if (kind_h && kind_h != kind_r) hako_rust_free_cstr(kind_h);
                if (err_r) hako_rust_free_cstr(err_r);
                if (err_h) hako_rust_free_cstr(err_h);
                return r;
            }
        } else {
            /* choose whichever error is present */
            if (!ok_r && err_r) {
                r->error_msg = err_r;
                if (err_h) hako_rust_free_cstr(err_h);
            } else if (!ok_h && err_h) {
                r->error_msg = err_h;
                if (err_r) hako_rust_free_cstr(err_r);
            }
            r->success = 0;
            r->stmt_count = ok_r ? sc_r : sc_h;
            r->kind = ok_r ? kind_r : kind_h;
            return r;
        }
    }
    /* invalid mode */
    r->success = 0;
    r->error_msg = hako_rust_make_cstring("invalid parser mode");
    return r;
}

void free_parse_result(HakoParseResult* result) {
    if (!result) return;
    if (result->kind) hako_rust_free_cstr(result->kind);
    if (result->error_msg) hako_rust_free_cstr(result->error_msg);
    free(result);
}

