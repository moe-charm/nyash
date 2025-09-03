#[cfg(test)]
mod tests {
    use crate::runtime::host_handles;
    use crate::runtime::host_api;

    #[test]
    fn host_reverse_call_map_slots() {
        // Build a MapBox and turn it into a HostHandle
        let map = std::sync::Arc::new(crate::boxes::map_box::MapBox::new()) as std::sync::Arc<dyn crate::box_trait::NyashBox>;
        let h = host_handles::to_handle_arc(map);

        // TLV args: key="k", val=42
        let mut tlv = crate::runtime::plugin_ffi_common::encode_tlv_header(2);
        crate::runtime::plugin_ffi_common::encode::string(&mut tlv, "k");
        crate::runtime::plugin_ffi_common::encode::i64(&mut tlv, 42);

        // set: slot 204
        let mut out = vec![0u8; 256];
        let mut out_len = out.len();
        let code = unsafe { host_api::nyrt_host_call_slot(h, 204, tlv.as_ptr(), tlv.len(), out.as_mut_ptr(), &mut out_len) };
        assert_eq!(code, 0);

        // size: slot 200
        let mut out2 = vec![0u8; 256];
        let mut out2_len = out2.len();
        let code2 = unsafe { host_api::nyrt_host_call_slot(h, 200, std::ptr::null(), 0, out2.as_mut_ptr(), &mut out2_len) };
        assert_eq!(code2, 0);
        if let Some((tag, _sz, payload)) = crate::runtime::plugin_ffi_common::decode::tlv_first(&out2[..out2_len]) {
            assert_eq!(tag, 3, "size returns i64 tag (3)");
            let n = crate::runtime::plugin_ffi_common::decode::u64(payload).unwrap_or(0);
            assert_eq!(n, 1, "after set, size should be 1");
        } else {
            panic!("no TLV output from size");
        }
    }
}

