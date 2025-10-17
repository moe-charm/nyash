#[cfg(all(test, feature = "plugins"))]
mod tests {
    use crate::runtime::host_api_box;
    use crate::runtime::host_handles;

    #[test]
    fn host_reverse_call_map_slots() {
        // Build a MapBox and turn it into a HostHandle
        let map: std::sync::Arc<dyn crate::box_trait::NyashBox> = {
            let _ = crate::runtime::init_global_plugin_host("nyash.toml");
            #[cfg(feature = "legacy-boxes")]
            {
                std::sync::Arc::new(crate::boxes::map_box::MapBox::new())
                    as std::sync::Arc<dyn crate::box_trait::NyashBox>
            }
            #[cfg(not(feature = "legacy-boxes"))]
            {
                match crate::runtime::plugin_host_box::create_box("MapBox", &[]) {
                    Ok(bx) => std::sync::Arc::from(bx),
                    Err(_e) => {
                        eprintln!("[skip] plugin MapBox create failed");
                        return;
                    }
                }
            }
        };
        // If plugin host is not ready, skip this test (dev-only stability)
        if let Some(p) = map.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
            if p.box_type != "MapBox" {
                eprintln!("[skip] plugin MapBox not available");
                return;
            }
        }
        let h = host_handles::to_handle_arc(map);

        // TLV args: key="k", val=42
        let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(2);
        crate::runtime::plugin_ffi_common::encode::string(&mut tlv, "k");
        crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, 42);

        // set: slot 204
        let code = host_api_box::call_slot_grow(h, 204, &tlv);
        assert!(code.is_ok());

        // size: slot 200
        let out2 = host_api_box::call_slot_grow(h, 200, &[]).expect("size ok");
        if let Some((tag, _sz, payload)) =
            crate::runtime::plugin_ffi_common::decode::tlv_first(&out2)
        {
            assert_eq!(tag, 3, "size returns i64 tag (3)");
            let n = crate::runtime::plugin_ffi_common::decode::u64(payload).unwrap_or(0);
            assert_eq!(n, 1, "after set, size should be 1");
        } else {
            panic!("no TLV output from size");
        }
    }
}
