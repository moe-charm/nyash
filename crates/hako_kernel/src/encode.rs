// Plugin-First architecture encoding system
// Simplified encoding that works directly with plugins and handles

use nyash_rust::runtime::plugin_loader_v2::PluginBoxV2;

/// Simplified encoding for Plugin-First architecture (replaces legacy VM encoding)
pub(crate) fn nyrt_encode_from_legacy_at(_buf: &mut Vec<u8>, _pos: usize) {
    // No-op: Plugin-First architecture handles encoding directly through unified plugin system
}

/// Simplified encoding for Plugin-First architecture (replaces legacy encoding)
pub(crate) fn nyrt_encode_arg_or_legacy(buf: &mut Vec<u8>, val: i64, _pos: usize) {
    use nyash_rust::runtime::host_handles as handles;
    // Handle direct values and plugin objects, bypass legacy VM fallback
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
    // Fallback: encode as i64 for non-plugin objects
    nyash_rust::runtime::plugin_ffi_common::encode::i64(buf, val);
}
