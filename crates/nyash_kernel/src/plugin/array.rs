// ---- Array helpers for LLVM lowering (handle-based) ----
// Exported as: nyash_array_get_h(i64 handle, i64 idx) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_get_h(handle: i64, idx: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, runtime::host_handles as handles};
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[ARR] get_h(handle={}, idx={})", handle, idx);
    }
    if handle <= 0 || idx < 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            let val = arr.get(Box::new(IntegerBox::new(idx)));
            if let Some(ib) = val.as_any().downcast_ref::<IntegerBox>() {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    eprintln!("[ARR] get_h => {}", ib.value);
                }
                return ib.value;
            }
        }
    }
    0
}

// Exported as: nyash_array_set_h(i64 handle, i64 idx, i64 val) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_set_h(handle: i64, idx: i64, val: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, runtime::host_handles as handles};
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[ARR] set_h(handle={}, idx={}, val={})", handle, idx, val);
    }
    if handle <= 0 || idx < 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            let i = idx as usize;
            let len = arr.len();
            if i < len {
                let _ = arr.set(
                    Box::new(IntegerBox::new(idx)),
                    Box::new(IntegerBox::new(val)),
                );
            } else if i == len {
                let _ = arr.push(Box::new(IntegerBox::new(val)));
            } else {
                // Do nothing for gaps (keep behavior conservative)
            }
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[ARR] set_h done; size now {}", arr.len());
            }
            return 0;
        }
    }
    0
}

// Exported as: nyash_array_push_h(i64 handle, i64 val) -> i64 (returns new length)
#[no_mangle]
pub extern "C" fn nyash_array_push_h(handle: i64, val: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        runtime::host_handles as handles,
    };
    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!("[ARR] push_h(handle={}, val={})", handle, val);
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            // If val is handle, try to use it; otherwise treat as integer
            let vbox: Box<dyn NyashBox> = if val > 0 {
                if let Some(o) = handles::get(val as u64) {
                    o.clone_box()
                } else {
                    Box::new(IntegerBox::new(val))
                }
            } else {
                Box::new(IntegerBox::new(val))
            };
            let _ = arr.push(vbox);
            let len = arr.len() as i64;
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!("[ARR] push_h -> len {}", len);
            }
            return len;
        }
    }
    0
}

// Exported as: nyash_array_length_h(i64 handle) -> i64
#[no_mangle]
pub extern "C" fn nyash_array_length_h(handle: i64) -> i64 {
    use nyash_rust::runtime::host_handles as handles;
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(arr) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::array::ArrayBox>()
        {
            return arr.len() as i64;
        }
    }
    0
}

// --- AOT ObjectModule dotted-name aliases (Array) ---
// Provide dotted symbol names expected by ObjectBuilder lowering, forwarding to existing underscored exports.
#[export_name = "nyash.array.get_h"]
pub extern "C" fn nyash_array_get_h_alias(handle: i64, idx: i64) -> i64 {
    nyash_array_get_h(handle, idx)
}

#[export_name = "nyash.array.set_h"]
pub extern "C" fn nyash_array_set_h_alias(handle: i64, idx: i64, val: i64) -> i64 {
    nyash_array_set_h(handle, idx, val)
}

#[export_name = "nyash.array.push_h"]
pub extern "C" fn nyash_array_push_h_alias(handle: i64, val: i64) -> i64 {
    nyash_array_push_h(handle, val)
}

#[export_name = "nyash.array.len_h"]
pub extern "C" fn nyash_array_len_h_alias(handle: i64) -> i64 {
    nyash_array_length_h(handle)
}
