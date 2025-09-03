// NyRT Host Reverse-Call C ABI (Phase 12)
// Minimal header for plugins → host calls using TLV-encoded arguments.
//
// Exports:
//  - nyrt_host_call_name: call method by name
//  - nyrt_host_call_slot: call by numeric slot (stable ABI)
//
// TLV Tags (subset):
//  tag=1  -> bool (1 byte)
//  tag=2  -> i32  (4 bytes, LE)
//  tag=3  -> i64/u64 (8 bytes, LE)
//  tag=5  -> f64 (8 bytes, LE)
//  tag=6/7-> string (utf8)
//  tag=8  -> PluginHandle (type_id:u32, instance_id:u32)
//  tag=9  -> HostHandle (u64) for user/builtin boxes
//
// Slots (subset):
//  InstanceBox: 1(getField), 2(setField), 3(has), 4(size)
//  ArrayBox:    100(get), 101(set), 102(len)
//  MapBox:      200(size), 201(len), 202(has), 203(get), 204(set)
//  StringBox:   300(len)

#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Call a method by name on a HostHandle receiver.
// Arguments are TLV-encoded values; output is a single TLV value.
// Returns 0 on success.
int32_t nyrt_host_call_name(uint64_t handle,
                            const uint8_t* method_ptr, size_t method_len,
                            const uint8_t* args_ptr, size_t args_len,
                            uint8_t* out_ptr, size_t* out_len);

// Call by stable numeric slot (preferred path for performance and diagnostics).
int32_t nyrt_host_call_slot(uint64_t handle, uint64_t selector_id,
                            const uint8_t* args_ptr, size_t args_len,
                            uint8_t* out_ptr, size_t* out_len);

#ifdef __cplusplus
}
#endif

