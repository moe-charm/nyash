#[cfg(test)]
mod tests {
    use crate::runtime::host_api_box;
    use crate::runtime::host_handles;

    #[test]
    fn host_reverse_call_map_slots() {
        // Build a MapBox and turn it into a HostHandle
        let map: std::sync::Arc<dyn crate::box_trait::NyashBox> = {
            #[cfg(feature = "legacy-boxes")]
            {
                std::sync::Arc::new(crate::boxes::map_box::MapBox::new())
                    as std::sync::Arc<dyn crate::box_trait::NyashBox>
            }
            #[cfg(not(feature = "legacy-boxes"))]
            {
                let bx = crate::runtime::plugin_host_box::create_box("MapBox", &[])
                    .expect("plugin MapBox create");
                std::sync::Arc::from(bx)
            }
        };
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
