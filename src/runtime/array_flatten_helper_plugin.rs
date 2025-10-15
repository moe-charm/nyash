use crate::backend::vm_types::VMValue;

pub fn is_array(v: &VMValue) -> bool {
    if let VMValue::BoxRef(bx) = v {
        if let Some(p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            return p.box_type == "ArrayBox";
        }
    }
    false
}

pub fn get_len(v: &VMValue) -> usize {
    if let VMValue::BoxRef(bx) = v {
        if let Some(_p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            let mut tmp_interp = crate::backend::mir_interpreter::MirInterpreter::new();
            if let Ok(VMValue::Integer(sz)) = crate::runtime::method_router_box::route(&mut tmp_interp, v, "size", &[]) {
                return sz as usize;
            }
        }
    }
    0
}

pub fn get_element(v: &VMValue, i: usize) -> VMValue {
    if let VMValue::BoxRef(bx) = v {
        if let Some(_p) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            let mut tmp_interp = crate::backend::mir_interpreter::MirInterpreter::new();
            if let Ok(val) = crate::runtime::method_router_box::route(&mut tmp_interp, v, "get", &[VMValue::Integer(i as i64)]) {
                return val;
            }
        }
    }
    v.clone()
}

