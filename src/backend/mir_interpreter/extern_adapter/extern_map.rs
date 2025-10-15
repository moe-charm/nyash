use std::collections::HashMap;
use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // nyrt.map.size(recv:Map) -> i64
    map.insert(("nyrt.map".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.size requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(map) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let n = map.get_data().read().unwrap().len();
                    return Ok(VMValue::Integer(hako_core_map::size(n)));
                }
                // plugin path fallbacks intentionally minimal
                Ok(VMValue::Integer(0))
            }
            _ => Err(VMError::TypeError("nyrt.map.size expects MapBox".into())),
        }
    });

    // nyrt.map.keys(recv:Map) -> Array
    map.insert(("nyrt.map".into(), "keys".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.keys requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.keys()));
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::TypeError("nyrt.map.keys expects MapBox".into())),
        }
    });

    // nyrt.map.values(recv:Map) -> Array
    map.insert(("nyrt.map".into(), "values".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.map.values requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.values()));
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::TypeError("nyrt.map.values expects MapBox".into())),
        }
    });
}

