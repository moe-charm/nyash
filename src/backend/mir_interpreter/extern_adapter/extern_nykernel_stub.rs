// extern_nykernel_stub.rs — nykernel.* dev stub (opt-in)
use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nykernel.malloc(size: i64) -> i64 (byte address)
    map.insert(("nykernel".into(), "malloc".into()), |args: &[VMValue]| {
        let size = args
            .get(0)
            .map(crate::runtime::nykernel_stub::vmvalue_to_i64)
            .unwrap_or(0);
        crate::runtime::nykernel_stub::malloc_bytes(size)
    });

    // nykernel.load_i64(addr: i64) -> i64
    map.insert(("nykernel".into(), "load_i64".into()), |args: &[VMValue]| {
        let addr = args
            .get(0)
            .map(crate::runtime::nykernel_stub::vmvalue_to_i64)
            .unwrap_or(0);
        crate::runtime::nykernel_stub::load_i64(addr)
    });

    // nykernel.store_i64(addr: i64, value: i64) -> void
    map.insert(("nykernel".into(), "store_i64".into()), |args: &[VMValue]| {
        if args.len() < 2 {
            return Err(VMError::InvalidInstruction("need 2 args".into()));
        }
        let addr = crate::runtime::nykernel_stub::vmvalue_to_i64(&args[0]);
        let val = crate::runtime::nykernel_stub::vmvalue_to_i64(&args[1]);
        crate::runtime::nykernel_stub::store_i64(addr, val)
    });
}

