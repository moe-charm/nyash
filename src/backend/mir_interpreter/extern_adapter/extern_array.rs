use std::collections::HashMap;
use crate::backend::vm_types::{VMError, VMValue};

fn array_size(args: &[VMValue]) -> Result<VMValue, VMError> {
    if args.is_empty() {
        return Err(VMError::InvalidInstruction(
            "nyrt.array.size requires receiver".into(),
        ));
    }
    match &args[0] {
        VMValue::BoxRef(b) => {
            #[cfg(feature = "legacy-boxes")]
            if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                let n = arr.len();
                return Ok(VMValue::Integer(n as i64));
            }
            let hh = crate::runtime::host_handles::to_handle_arc(b.clone());
            let mut out_buf = vec![0u8; 64];
            let mut out_len: usize = out_buf.len();
            let rc = crate::runtime::host_api::nyrt_host_call_slot(
                hh,
                crate::runtime::host_handle_router::consts::ARRAY_SIZE,
                std::ptr::null(),
                0,
                out_buf.as_mut_ptr(),
                &mut out_len,
            );
            if rc == 0 && out_len >= 6 {
                if let Some((tag, _sz, payload)) =
                    crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len])
                {
                    if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        return Ok(v);
                    }
                }
            }
            Ok(VMValue::Integer(0))
        }
        _ => Err(VMError::TypeError(
            "nyrt.array.size expects ArrayBox".into(),
        )),
    }
}

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.array.size(recv:Array) -> i64
    map.insert(("nyrt.array".into(), "size".into()), array_size as super::HandlerFn);
    // Alias: nyrt.array.length → size
    map.insert(("nyrt.array".into(), "length".into()), array_size as super::HandlerFn);
}
