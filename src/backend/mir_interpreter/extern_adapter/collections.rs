use std::sync::Arc;

use crate::backend::vm_types::{VMError, VMValue};
use crate::box_trait::NyashBox;
use crate::runtime::host_handle_box::HostHandleBox;

/// Resolve HostHandle indirections so Array receivers are concrete boxes.
pub fn materialize_array_receiver(bx: &Arc<dyn NyashBox>) -> Result<Arc<dyn NyashBox>, VMError> {
    let mut current = bx.clone();
    let mut hop = 0usize;
    loop {
        if let Some(hhb) = current.as_any().downcast_ref::<HostHandleBox>() {
            let next = crate::runtime::host_handles::get(hhb.id).ok_or_else(|| {
                VMError::InvalidInstruction("HostHandle receiver was released".into())
            })?;
            current = next;
            hop += 1;
            if hop > 8 {
                return Err(VMError::InvalidInstruction(
                    "HostHandle receiver resolution exceeded limit".into(),
                ));
            }
            continue;
        }
        return Ok(current);
    }
}

/// Normalize array-like VMValue results so HostHandleBox wrappers become concrete boxes.
pub fn normalize_array_value(value: VMValue) -> Result<VMValue, VMError> {
    if let VMValue::BoxRef(ref arc) = value {
        if arc.as_any().downcast_ref::<HostHandleBox>().is_some() {
            let resolved = materialize_array_receiver(arc)?;
            return Ok(VMValue::BoxRef(resolved));
        }
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_array_value_unwraps_host_handle_box() {
        let _ = crate::runtime::init_global_plugin_host("nyash.toml");
        let maybe = crate::runtime::type_registry::create_core_box("ArrayBox", &[]);
        if maybe.is_none() { eprintln!("[skip] ArrayBox core box not registered"); return; }
        let array_box = match maybe.unwrap() { Ok(b) => b, Err(e) => { eprintln!("[skip] create ArrayBox failed: {:?}", e); return; } };
        let array: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(array_box);
        let handle = crate::runtime::host_handles::to_handle_arc(array.clone());
        let host_box: std::sync::Arc<dyn NyashBox> =
            std::sync::Arc::new(crate::runtime::host_handle_box::HostHandleBox::new(handle));
        let value = VMValue::BoxRef(host_box);

        let normalized = normalize_array_value(value).expect("normalization should succeed");

        crate::runtime::host_handles::release(handle);

        match normalized {
            VMValue::BoxRef(real) => {
                assert_eq!(real.type_name(), array.type_name());
                assert!(
                    std::sync::Arc::ptr_eq(&real, &array),
                    "normalization should preserve the original ArrayBox Arc"
                );
            }
            other => panic!("expected BoxRef after normalization, got {:?}", other),
        }
    }

    #[test]
    fn materialize_array_receiver_passthrough_for_concrete_box() {
        let _ = crate::runtime::init_global_plugin_host("nyash.toml");
        let maybe = crate::runtime::type_registry::create_core_box("ArrayBox", &[]);
        if maybe.is_none() { eprintln!("[skip] ArrayBox core box not registered"); return; }
        let array_box = match maybe.unwrap() { Ok(b) => b, Err(e) => { eprintln!("[skip] create ArrayBox failed: {:?}", e); return; } };
        let array: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(array_box);
        let resolved = materialize_array_receiver(&array).expect("should succeed");
        assert!(
            std::sync::Arc::ptr_eq(&resolved, &array),
            "materialize_array_receiver should return same Arc for concrete ArrayBox"
        );
    }
}
