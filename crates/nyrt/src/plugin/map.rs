// --- AOT ObjectModule dotted-name exports (Map) ---
// Provide dotted symbol names expected by ObjectBuilder lowering for MapBox operations.
// size: (handle) -> i64
#[export_name = "nyash.map.size_h"]
pub extern "C" fn nyash_map_size_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
    if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
        eprintln!("[MAP] size_h(handle={})", handle);
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            if let Some(ib) = map
                .size()
                .as_any()
                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
            {
                if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
                    eprintln!("[MAP] size_h => {}", ib.value);
                }
                return ib.value;
            }
        }
    }
    0
}

// get_h: (map_handle, key_i64) -> value_handle
#[export_name = "nyash.map.get_h"]
pub extern "C" fn nyash_map_get_h(handle: i64, key: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
        eprintln!("[MAP] get_h(handle={}, key={})", handle, key);
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = Box::new(IntegerBox::new(key));
            let v = map.get(kbox);
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(v);
            let h = handles::to_handle(arc);
            if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
                eprintln!("[MAP] get_h => handle {}", h);
            }
            return h as i64;
        }
    }
    0
}

// get_hh: (map_handle, key_handle) -> value_handle
#[export_name = "nyash.map.get_hh"]
pub extern "C" fn nyash_map_get_hh(handle: i64, key_any: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let key_box: Box<dyn NyashBox> = if key_any > 0 {
                if let Some(k) = handles::get(key_any as u64) {
                    k.clone_box()
                } else {
                    Box::new(IntegerBox::new(key_any))
                }
            } else {
                Box::new(IntegerBox::new(key_any))
            };
            let v = map.get(key_box);
            let arc: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(v);
            let h = handles::to_handle(arc);
            return h as i64;
        }
    }
    0
}

// set_h: (map_handle, key_i64, val) -> i64 (ignored/0)
#[export_name = "nyash.map.set_h"]
pub extern "C" fn nyash_map_set_h(handle: i64, key: i64, val: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
        eprintln!("[MAP] set_h(handle={}, key={}, val={})", handle, key, val);
    }
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = Box::new(IntegerBox::new(key));
            let vbox: Box<dyn NyashBox> = if val > 0 {
                if let Some(o) = handles::get(val as u64) {
                    o.clone_box()
                } else {
                    Box::new(IntegerBox::new(val))
                }
            } else {
                Box::new(IntegerBox::new(val))
            };
            let _ = map.set(kbox, vbox);
            if std::env::var("NYASH_LLVM_MAP_DEBUG").ok().as_deref() == Some("1") {
                let sz = map
                    .size()
                    .as_any()
                    .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                    .map(|i| i.value)
                    .unwrap_or(-1);
                eprintln!("[MAP] set_h done; size now {}", sz);
            }
            return 0;
        }
    }
    0
}

// set_hh: (map_handle, key_any: handle or i64, val_any: handle or i64) -> i64
#[export_name = "nyash.map.set_hh"]
pub extern "C" fn nyash_map_set_hh(handle: i64, key_any: i64, val_any: i64) -> i64 {
    use nyash_rust::{
        box_trait::{IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = if key_any > 0 {
                if let Some(k) = handles::get(key_any as u64) {
                    k.clone_box()
                } else {
                    Box::new(IntegerBox::new(key_any))
                }
            } else {
                Box::new(IntegerBox::new(key_any))
            };
            let vbox: Box<dyn NyashBox> = if val_any > 0 {
                if let Some(v) = handles::get(val_any as u64) {
                    v.clone_box()
                } else {
                    Box::new(IntegerBox::new(val_any))
                }
            } else {
                Box::new(IntegerBox::new(val_any))
            };
            let _ = map.set(kbox, vbox);
            return 0;
        }
    }
    0
}

// has_hh: (map_handle, key_any: handle or i64) -> i64 (0/1)
#[export_name = "nyash.map.has_hh"]
pub extern "C" fn nyash_map_has_hh(handle: i64, key_any: i64) -> i64 {
    use nyash_rust::{
        box_trait::{BoolBox, IntegerBox, NyashBox},
        jit::rt::handles,
    };
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox: Box<dyn NyashBox> = if key_any > 0 {
                if let Some(k) = handles::get(key_any as u64) {
                    k.clone_box()
                } else {
                    Box::new(IntegerBox::new(key_any))
                }
            } else {
                Box::new(IntegerBox::new(key_any))
            };
            let v = map.has(kbox);
            if let Some(b) = v.as_any().downcast_ref::<BoolBox>() {
                return if b.value { 1 } else { 0 };
            }
        }
    }
    0
}

// has_h: (map_handle, key_i64) -> i64 (0/1)
#[export_name = "nyash.map.has_h"]
pub extern "C" fn nyash_map_has_h(handle: i64, key: i64) -> i64 {
    use nyash_rust::{box_trait::IntegerBox, jit::rt::handles};
    if handle <= 0 {
        return 0;
    }
    if let Some(obj) = handles::get(handle as u64) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let kbox = Box::new(IntegerBox::new(key));
            let v = map.has(kbox);
            if let Some(b) = v.as_any().downcast_ref::<nyash_rust::box_trait::BoolBox>() {
                return if b.value { 1 } else { 0 };
            }
        }
    }
    0
}
