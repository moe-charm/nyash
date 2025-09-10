use nyash_rust::backend::vm::VMValue;
use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;

// Internal helpers to avoid nested mutable borrows across closures
pub(crate) fn nyrt_encode_from_legacy_at(buf: &mut Vec<u8>, pos: usize) {
    nyash_rust::jit::rt::with_legacy_vm_args(|args| {
        if let Some(v) = args.get(pos) {
            match v {
                VMValue::String(s) => {
                    nyash_rust::runtime::plugin_ffi_common::encode::string(buf, s)
                }
                VMValue::Integer(i) => nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, *i),
                VMValue::Float(f) => nyash_rust::runtime::plugin_ffi_common::encode::f64(buf, *f),
                VMValue::Bool(b) => nyash_rust::runtime::plugin_ffi_common::encode::bool(buf, *b),
                VMValue::BoxRef(b) => {
                    if let Some(bufbox) = b
                        .as_any()
                        .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
                    {
                        nyash_rust::runtime::plugin_ffi_common::encode::bytes(
                            buf,
                            &bufbox.to_vec(),
                        );
                        return;
                    }
                    if let Some(p) = b.as_any().downcast_ref::<PluginBoxV2>() {
                        let host = nyash_rust::runtime::get_global_plugin_host();
                        if let Ok(hg) = host.read() {
                            if p.box_type == "StringBox" {
                                if let Ok(Some(sb)) = hg.invoke_instance_method(
                                    "StringBox",
                                    "toUtf8",
                                    p.instance_id(),
                                    &[],
                                ) {
                                    if let Some(s) = sb
                                        .as_any()
                                        .downcast_ref::<nyash_rust::box_trait::StringBox>()
                                    {
                                        nyash_rust::runtime::plugin_ffi_common::encode::string(
                                            buf, &s.value,
                                        );
                                        return;
                                    }
                                }
                            } else if p.box_type == "IntegerBox" {
                                if let Ok(Some(ibx)) = hg.invoke_instance_method(
                                    "IntegerBox",
                                    "get",
                                    p.instance_id(),
                                    &[],
                                ) {
                                    if let Some(i) = ibx
                                        .as_any()
                                        .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                    {
                                        nyash_rust::runtime::plugin_ffi_common::encode::i64(
                                            buf, i.value,
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                        nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                            buf,
                            p.inner.type_id,
                            p.instance_id(),
                        );
                    } else {
                        let s = b.to_string_box().value;
                        nyash_rust::runtime::plugin_ffi_common::encode::string(buf, &s)
                    }
                }
                _ => {}
            }
        }
    });
}

pub(crate) fn nyrt_encode_arg_or_legacy(buf: &mut Vec<u8>, val: i64, pos: usize) {
    use nyash_rust::jit::rt::handles;
    if val > 0 {
        if let Some(obj) = handles::get(val as u64) {
            if let Some(bufbox) = obj
                .as_any()
                .downcast_ref::<nyash_rust::boxes::buffer::BufferBox>()
            {
                nyash_rust::runtime::plugin_ffi_common::encode::bytes(buf, &bufbox.to_vec());
                return;
            }
            if let Some(p) = obj.as_any().downcast_ref::<PluginBoxV2>() {
                let host = nyash_rust::runtime::get_global_plugin_host();
                if let Ok(hg) = host.read() {
                    if p.box_type == "StringBox" {
                        if let Ok(Some(sb)) =
                            hg.invoke_instance_method("StringBox", "toUtf8", p.instance_id(), &[])
                        {
                            if let Some(s) = sb
                                .as_any()
                                .downcast_ref::<nyash_rust::box_trait::StringBox>()
                            {
                                nyash_rust::runtime::plugin_ffi_common::encode::string(
                                    buf, &s.value,
                                );
                                return;
                            }
                        }
                    } else if p.box_type == "IntegerBox" {
                        if let Ok(Some(ibx)) =
                            hg.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[])
                        {
                            if let Some(i) = ibx
                                .as_any()
                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                            {
                                nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, i.value);
                                return;
                            }
                        }
                    }
                }
                nyash_rust::runtime::plugin_ffi_common::encode::plugin_handle(
                    buf,
                    p.inner.type_id,
                    p.instance_id(),
                );
                return;
            }
        }
    }
    let before = buf.len();
    nyrt_encode_from_legacy_at(buf, pos);
    if buf.len() == before {
        nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, val);
    }
}
