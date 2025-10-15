use std::collections::HashMap;
use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    // add(recv:Set, v:any) -> Void
    map.insert(("nyrt.set".into(), "add".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.set.add requires (recv, value)".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![ args[1].to_nyash_box() ];
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "add", pb.inner.instance_id, &argv);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    let _ = sb.add(args[1].to_nyash_box());
                    return Ok(VMValue::Void);
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![
                            args[1].to_nyash_box(),
                            Box::new(crate::box_trait::VoidBox::new()),
                        ];
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "set", pb.inner.instance_id, &argv);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let _ = mapb.set(args[1].to_nyash_box(), Box::new(crate::boxes::null_box::NullBox::new()));
                    return Ok(VMValue::Void);
                }
                Err(VMError::TypeError("nyrt.set.add expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.add expects Set(Map-backed) receiver".into())),
        }
    });

    // remove(recv:Set, v:any) -> Void
    map.insert(("nyrt.set".into(), "remove".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.set.remove requires (recv, value)".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![ args[1].to_nyash_box() ];
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "remove", pb.inner.instance_id, &argv);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    let _ = sb.remove(args[1].to_nyash_box());
                    return Ok(VMValue::Void);
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![ args[1].to_nyash_box() ];
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "remove", pb.inner.instance_id, &argv);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let _ = mapb.delete(args[1].to_nyash_box());
                    return Ok(VMValue::Void);
                }
                Err(VMError::TypeError("nyrt.set.remove expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.remove expects Set(Map-backed) receiver".into())),
        }
    });

    // has(recv:Set, v:any) -> Bool
    map.insert(("nyrt.set".into(), "has".into()), |args: &[VMValue]| {
        if args.len() < 2 { return Err(VMError::InvalidInstruction("nyrt.set.has requires (recv, value)".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![ args[1].to_nyash_box() ];
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "has", pb.inner.instance_id, &argv)
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Bool(false)));
                        return Ok(out.unwrap_or(VMValue::Bool(false)));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    let out = sb.has(args[1].to_nyash_box());
                    return Ok(VMValue::from_nyash_box(out));
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let hh = crate::runtime::host_handles::to_handle_arc(b.clone());
                        let args_boxes = vec![args[1].to_nyash_box()];
                        let tlv = crate::runtime::plugin_ffi_common::encode_args(&args_boxes);
                        let mut out_buf = vec![0u8; 64];
                        let mut out_len: usize = out_buf.len();
                        let rc = crate::runtime::host_api::nyrt_host_call_slot(
                            hh,
                            crate::runtime::host_handle_router::consts::MAP_HAS,
                            tlv.as_ptr(),
                            tlv.len(),
                            out_buf.as_mut_ptr(),
                            &mut out_len,
                        );
                        if rc == 0 && out_len >= 6 {
                            if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len]) {
                                if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) { return Ok(v); }
                            }
                        }
                        let argv: Vec<Box<dyn crate::box_trait::NyashBox>> = vec![ args[1].to_nyash_box() ];
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "has", pb.inner.instance_id, &argv)
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Bool(false)));
                        return Ok(out.unwrap_or(VMValue::Bool(false)));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let out = mapb.has(args[1].to_nyash_box());
                    return Ok(VMValue::from_nyash_box(out));
                }
                Err(VMError::TypeError("nyrt.set.has expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.has expects Set(Map-backed) receiver".into())),
        }
    });

    // size(recv:Set) -> i64
    map.insert(("nyrt.set".into(), "size".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.set.size requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "size", pb.inner.instance_id, &[])
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Integer(0)));
                        return Ok(out.unwrap_or(VMValue::Integer(0)));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    return Ok(VMValue::from_nyash_box(sb.size()))
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "size", pb.inner.instance_id, &[])
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Integer(0)));
                        return Ok(out.unwrap_or(VMValue::Integer(0)));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.size()));
                }
                Err(VMError::TypeError("nyrt.set.size expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.size expects Set(Map-backed) receiver".into())),
        }
    });

    // clear(recv:Set) -> Void
    map.insert(("nyrt.set".into(), "clear".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.set.clear requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "clear", pb.inner.instance_id, &[]);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    let _ = sb.clear();
                    return Ok(VMValue::Void);
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let _ = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "clear", pb.inner.instance_id, &[]);
                        return Ok(VMValue::Void);
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    let _ = mapb.clear();
                    return Ok(VMValue::Void);
                }
                Err(VMError::TypeError("nyrt.set.clear expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.clear expects Set(Map-backed) receiver".into())),
        }
    });

    // toArray(recv:Set) -> Array (keys)
    map.insert(("nyrt.set".into(), "toArray".into()), |args: &[VMValue]| {
        if args.is_empty() { return Err(VMError::InvalidInstruction("nyrt.set.toArray requires receiver".into())); }
        match &args[0] {
            VMValue::BoxRef(b) => {
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "SetBox" {
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("SetBox", "toArray", pb.inner.instance_id, &[])
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Void));
                        return Ok(out.unwrap_or(VMValue::Void));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(sb) = b.as_any().downcast_ref::<crate::boxes::set_box::SetBox>() {
                    return Ok(VMValue::from_nyash_box(sb.toArray()));
                }
                if let Some(pb) = b.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                    if pb.box_type == "MapBox" {
                        let out = crate::runtime::plugin_host_box::invoke_instance_method("MapBox", "keys", pb.inner.instance_id, &[])
                            .map(|opt| opt.map(crate::backend::vm::VMValue::from_nyash_box))
                            .unwrap_or(Some(VMValue::Void));
                        return Ok(out.unwrap_or(VMValue::Void));
                    }
                }
                #[cfg(feature = "legacy-boxes")]
                if let Some(mapb) = b.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
                    return Ok(VMValue::from_nyash_box(mapb.keys()));
                }
                Err(VMError::TypeError("nyrt.set.toArray expects Set(Map-backed) receiver".into()))
            }
            _ => Err(VMError::TypeError("nyrt.set.toArray expects Set(Map-backed) receiver".into())),
        }
    });
}
