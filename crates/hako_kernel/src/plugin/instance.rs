// ---- Instance field helpers for LLVM lowering (handle-based) ----
// Exported as: nyash.instance.get_field_h(i64 handle, i8* name) -> i64
#[export_name = "nyash.instance.get_field_h"]
pub extern "C" fn nyash_instance_get_field_h(handle: i64, name: *const i8) -> i64 {
    if handle <= 0 || name.is_null() {
        return 0;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(name) };
    let Ok(field) = name.to_str() else { return 0 };
    if let Some(obj) = nyash_rust::runtime::host_handles::get(handle as u64) {
        if let Some(inst) = obj
            .as_any()
            .downcast_ref::<nyash_rust::instance_v2::InstanceBox>()
        {
            if let Some(shared) = inst.get_field(field) {
                let arc: std::sync::Arc<dyn nyash_rust::box_trait::NyashBox> =
                    std::sync::Arc::from(shared);
                let h = nyash_rust::runtime::host_handles::to_handle_arc(arc);
                return h as i64;
            }
        }
    }
    0
}

// Exported as: nyash.instance.set_field_h(i64 handle, i8* name, i64 val_h) -> i64
#[export_name = "nyash.instance.set_field_h"]
pub extern "C" fn nyash_instance_set_field_h(handle: i64, name: *const i8, val_h: i64) -> i64 {
    if handle <= 0 || name.is_null() {
        return 0;
    }
    let name = unsafe { std::ffi::CStr::from_ptr(name) };
    let Ok(field) = name.to_str() else { return 0 };
    if let Some(obj) = nyash_rust::runtime::host_handles::get(handle as u64) {
        if let Some(inst) = obj
            .as_any()
            .downcast_ref::<nyash_rust::instance_v2::InstanceBox>()
        {
            if val_h > 0 {
                if let Some(val) = nyash_rust::runtime::host_handles::get(val_h as u64) {
                    let shared: nyash_rust::box_trait::SharedNyashBox = std::sync::Arc::clone(&val);
                    let _ = inst.set_field(field, shared);
                    return 0;
                }
            }
        }
    }
    0
}
