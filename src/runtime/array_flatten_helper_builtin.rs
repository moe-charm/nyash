use crate::backend::vm_types::VMValue;

#[cfg(feature = "legacy-boxes")]
pub fn is_array(v: &VMValue) -> bool {
    if let VMValue::BoxRef(bx) = v {
        return bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>().is_some();
    }
    false
}

#[cfg(not(feature = "legacy-boxes"))]
pub fn is_array(_: &VMValue) -> bool { false }

#[cfg(feature = "legacy-boxes")]
pub fn get_len(v: &VMValue) -> Option<usize> {
    if let VMValue::BoxRef(bx) = v {
        if let Some(a) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            return Some(a.items.read().unwrap().len());
        }
    }
    None
}

#[cfg(not(feature = "legacy-boxes"))]
pub fn get_len(_: &VMValue) -> Option<usize> { None }

#[cfg(feature = "legacy-boxes")]
pub fn get_element(v: &VMValue, i: usize) -> Option<VMValue> {
    if let VMValue::BoxRef(bx) = v {
        if let Some(a) = bx.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            let guard = a.items.read().unwrap();
            return Some(VMValue::from_nyash_box(guard[i].clone_box()));
        }
    }
    None
}

#[cfg(not(feature = "legacy-boxes"))]
pub fn get_element(_: &VMValue, _: usize) -> Option<VMValue> { None }

