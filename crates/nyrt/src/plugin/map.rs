// --- AOT ObjectModule dotted-name exports (Map) ---
// Provide dotted symbol names expected by ObjectBuilder lowering for MapBox operations.
// size: (handle) -> i64
#[export_name = "nyash.map.size_h"]
pub extern "C" fn nyash_map_size_h(handle: i64) -> i64 {
    use nyash_rust::jit::rt::handles;
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
            return h as i64;
        }
    }
    0
}

// get_hh: (map_handle, key_handle) -> value_handle
#[export_name = "nyash.map.get_hh"]
pub extern "C" fn nyash_map_get_hh(handle: i64, key_h: i64) -> i64 {
    use nyash_rust::{box_trait::NyashBox, jit::rt::handles};
    if handle <= 0 || key_h <= 0 {
        return 0;
    }
    if let (Some(obj), Some(key)) = (handles::get(handle as u64), handles::get(key_h as u64)) {
        if let Some(map) = obj
            .as_any()
            .downcast_ref::<nyash_rust::boxes::map_box::MapBox>()
        {
            let v = map.get(key.clone_box());
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
            return 0;
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
            let v = map.get(kbox);
            // Consider present if not VoidBox
            let present = !v.as_any().is::<nyash_rust::box_trait::VoidBox>();
            return if present { 1 } else { 0 };
        }
    }
    0
}
