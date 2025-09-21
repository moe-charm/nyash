// Minimal shim to keep build/link path ready for a future yyjson backend.
// This file does not currently implement parsing; the Rust side still
// uses serde_json for actual parsing until the real yyjson integration lands.

#include <stdint.h>
#include <stddef.h>
#include "yyjson/yyjson.h"

// Parse JSON via yyjson and return 0 on success; non-zero error code on failure.
int nyash_json_shim_parse(const char *text, size_t len) {
    yyjson_read_err err;
    yyjson_doc *doc = yyjson_read_opts(text, len, 0, NULL, &err);
    if (!doc) return (int)err.code;
    yyjson_doc_free(doc);
    return 0;
}

// Full wrappers exported with stable names for Rust FFI
void *nyjson_parse_doc(const char *text, size_t len, int *out_err_code) {
    yyjson_read_err err;
    yyjson_doc *doc = yyjson_read_opts(text, len, 0, NULL, &err);
    if (!doc) {
        if (out_err_code) *out_err_code = (int)err.code;
        return NULL;
    }
    if (out_err_code) *out_err_code = 0;
    return (void *)doc;
}
void nyjson_doc_free(void *doc) {
    if (doc) yyjson_doc_free((yyjson_doc *)doc);
}
void *nyjson_doc_root(void *doc) {
    if (!doc) return NULL;
    return (void *)yyjson_doc_get_root((yyjson_doc *)doc);
}

int nyjson_is_null(void *v)  { return v && yyjson_is_null((yyjson_val *)v); }
int nyjson_is_bool(void *v)  { return v && yyjson_is_bool((yyjson_val *)v); }
int nyjson_is_int(void *v)   { return v && yyjson_is_sint((yyjson_val *)v); }
int nyjson_is_real(void *v)  { return v && yyjson_is_real((yyjson_val *)v); }
int nyjson_is_str(void *v)   { return v && yyjson_is_str((yyjson_val *)v); }
int nyjson_is_arr(void *v)   { return v && yyjson_is_arr((yyjson_val *)v); }
int nyjson_is_obj(void *v)   { return v && yyjson_is_obj((yyjson_val *)v); }

int nyjson_get_bool_val(void *v) {
    if (!v || !yyjson_is_bool((yyjson_val *)v)) return 0;
    return yyjson_get_bool((yyjson_val *)v);
}
long long nyjson_get_sint_val(void *v) {
    if (!v || !yyjson_is_sint((yyjson_val *)v)) return 0;
    return (long long)yyjson_get_sint((yyjson_val *)v);
}
const char *nyjson_get_str_val(void *v) {
    if (!v || !yyjson_is_str((yyjson_val *)v)) return NULL;
    return yyjson_get_str((yyjson_val *)v);
}

size_t nyjson_arr_size_val(void *v) {
    if (!v || !yyjson_is_arr((yyjson_val *)v)) return 0;
    return yyjson_arr_size((yyjson_val *)v);
}
void *nyjson_arr_get_val(void *v, size_t idx) {
    if (!v || !yyjson_is_arr((yyjson_val *)v)) return NULL;
    return (void *)yyjson_arr_get((yyjson_val *)v, idx);
}

size_t nyjson_obj_size_val(void *v) {
    if (!v || !yyjson_is_obj((yyjson_val *)v)) return 0;
    return yyjson_obj_size((yyjson_val *)v);
}
void *nyjson_obj_get_key(void *v, const char *key) {
    if (!v || !yyjson_is_obj((yyjson_val *)v) || !key) return NULL;
    return (void *)yyjson_obj_get((yyjson_val *)v, key);
}
