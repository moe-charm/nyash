// Nyash Plugin ABI v2 — TypeBox header (minimal)
// Notes:
//  - This is the minimal, production-friendly subset used in Phase‑15.
//  - Forward compatibility: hosts validate struct_size and version; fields may
//    be extended in future minor revisions without breaking existing plugins.
//  - See docs/reference/plugin-abi/nyash_abi_v2.md for the spec and roadmap.
#ifndef NYASH_ABI_V2_H
#define NYASH_ABI_V2_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Magic and version
#define NYASH_TYPEBOX_ABI_TAG 0x54594258u /* 'TYBX' */
#define NYASH_ABI_VERSION     1u

// Error codes
#define NYB_OK            0
#define NYB_E_SHORT      -1
#define NYB_E_TYPE       -2
#define NYB_E_METHOD     -3
#define NYB_E_ARGS       -4
#define NYB_E_PLUGIN     -5
#define NYB_E_HANDLE     -8

// TLV tags (version=1)
#define NYTLV_BOOL   1
#define NYTLV_I32    2
#define NYTLV_I64    3
#define NYTLV_F32    4
#define NYTLV_F64    5
#define NYTLV_STRING 6
#define NYTLV_BYTES  7
#define NYTLV_HANDLE 8   /* type_id:u32 + instance_id:u32 */
#define NYTLV_HOST   9   /* host handle: u64 */

typedef int32_t (*ny_invoke_id_fn)(
    uint32_t instance_id,
    uint32_t method_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* out,
    size_t* out_len
);

typedef uint32_t (*ny_resolve_fn)(const char* method_name);

typedef struct NyashTypeBoxFfi {
    uint32_t abi_tag;
    uint16_t version;
    uint16_t struct_size;
    const char* name;   /* C string with trailing NUL */
    ny_resolve_fn resolve;      /* optional */
    ny_invoke_id_fn invoke_id;  /* required */
    uint64_t capabilities;      /* reserved, set 0 */
} NyashTypeBoxFfi;

#ifdef __cplusplus
}
#endif

#endif /* NYASH_ABI_V2_H */
