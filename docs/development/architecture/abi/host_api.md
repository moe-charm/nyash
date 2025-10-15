NyRT Host Reverse-Call API (Phase 12)

Purpose
- Provide a stable ABI for plugins to call back into the host to operate on HostHandles.
- Enable vtable/slot-based dispatch for Array/Map/String/Instance with GC-barrier correctness.

Exports
- `nyrt_host_call_name(handle, method, args_tlv) -> out_tlv`
- `nyrt_host_call_slot(handle, slot, args_tlv) -> out_tlv` (preferred)

TLV Tags (subset)
- 1: bool
- 2: i32
- 3: i64/u64 (8 bytes LE)
- 5: f64
- 6/7: string (utf8)
- 8: PluginHandle (type_id:u32, instance_id:u32)
- 9: HostHandle (u64) for user/builtin boxes

Slots
- InstanceBox: 1(getField), 2(setField), 3(has), 4(size)
- ArrayBox: 100(get), 101(set), 102(len)
- MapBox: 200(size), 201(len), 202(has), 203(get), 204(set)
- StringBox: 300(len)

Barriers
- Mutators (Instance.setField, Array.set, Map.set) invoke a write barrier using TLS-bound current VM.
- JIT must bind `set_current_vm/clear_current_vm` around host-bridge calls; VM does this at the JIT boundary.

Header
- See `include/nyrt_host_api.h` for C prototypes and TLV summary.

