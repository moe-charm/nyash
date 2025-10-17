use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::StringBox;
#[cfg(feature = "legacy-boxes")]
use crate::box_trait::IntegerBox;

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // env.local.get(key:String) -> String
    map.insert(("env.local".into(), "get".into()), |args: &[VMValue]| {
        if args.len() < 1 { return Err(VMError::InvalidInstruction("env.local.get requires 1 arg".into())); }
        let key = match &args[0] {
            VMValue::String(s) => s.clone(),
            VMValue::BoxRef(b) => b.to_string_box().value,
            v => v.to_string(),
        };
        let val = std::env::var(&key).unwrap_or_else(|_| "".to_string());
        Ok(VMValue::String(val))
    });

    // nyash.json.canonicalize_h(json:String) -> String
    map.insert(("nyash.json".into(), "canonicalize_h".into()), |args: &[VMValue]| {
        if args.len() < 1 {
            return Err(VMError::InvalidInstruction(
                "nyash.json.canonicalize_h requires 1 arg".into(),
            ));
        }
        let src = match &args[0] {
            VMValue::String(s) => s.clone(),
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                {
                    if let Some(arr) = b.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
                        if arr.len() > 0 {
                            let first = arr.get(Box::new(IntegerBox::new(0)));
                            first.to_string_box().value
                        } else {
                            b.to_string_box().value
                        }
                    } else {
                        b.to_string_box().value
                    }
                }
                #[cfg(not(feature = "legacy-boxes"))]
                {
                    b.to_string_box().value
                }
            }
            v => v.to_string(),
        };

        #[cfg(feature = "host-anchors")]
        {
            let handle_in = crate::runtime::host_handles::to_handle_box(Box::new(StringBox::new(src.clone())));
            let handle_out = crate::runtime::host_api_anchors::nyash_json_canonicalize_h(handle_in as i64);
            crate::runtime::host_handles::release(handle_in);

            if handle_out != 0 {
                let arc = crate::runtime::host_handles::get(handle_out as u64);
                crate::runtime::host_handles::release(handle_out as u64);

                if let Some(bx) = arc.and_then(|a| a.as_ref().as_any().downcast_ref::<StringBox>().cloned()) {
                    return Ok(VMValue::String(bx.value));
                }
            }
        }

        match serde_json::from_str::<serde_json::Value>(&src) {
            Ok(v) => {
                let s = crate::common::json_canonical::to_canonical_string(&v);
                Ok(VMValue::String(s))
            }
            Err(_) => Ok(VMValue::String(src)),
        }
    });
}
