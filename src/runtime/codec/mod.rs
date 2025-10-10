//! TLV Codec Box — boundary for plugin FFI TLV policy
//!
//! Responsibility
//! - Provide a thin, testable facade over TLV encode/decode used across VM and PluginLoader.
//! - Centralize handle-first encoding policy and primitive fallbacks.
//! - Keep behavior identical to existing helpers in `plugin_ffi_common` (calls delegate here).
//!
//! Notes
//! - This crate does not own ABI constants; it mirrors `plugin_ffi_common` behavior.
//! - Keep minimal to avoid runtime cost; no dynamic dispatch on hot paths.

use crate::box_trait::NyashBox;
use crate::runtime::plugin_loader_v2::PluginBoxV2;

#[derive(Default, Clone, Debug)]
pub struct TlvCodecBox;

impl TlvCodecBox {
    pub fn encode_empty_args(&self) -> Vec<u8> {
        vec![1u8, 0, 0, 0]
    }

    pub fn encode_header(&self, argc: u16) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4);
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&argc.to_le_bytes());
        buf
    }

    pub fn encode_args(&self, args: &[Box<dyn NyashBox>]) -> Vec<u8> {
        let mut buf = self.encode_header(args.len() as u16);
        for a in args {
            if let Some(pb) = a.as_any().downcast_ref::<PluginBoxV2>() {
                crate::runtime::plugin_ffi_common::encode::plugin_handle(
                    &mut buf,
                    pb.inner.type_id,
                    pb.instance_id(),
                );
            } else if crate::runtime::type_registry::is_core_box(a.type_name()) {
                let h = crate::runtime::host_handles::to_handle_box(a.clone_box());
                crate::runtime::plugin_ffi_common::encode::host_handle(&mut buf, h);
            } else if let Some(hb) = a.as_any().downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>() {
                    crate::runtime::plugin_ffi_common::encode::host_handle(&mut buf, hb.id);
            } else if let Some(i) = crate::runtime::semantics::coerce_to_i64(a.as_ref()) {
                crate::runtime::plugin_ffi_common::encode::i64(&mut buf, i);
            } else if let Some(s) = crate::runtime::semantics::coerce_to_string(a.as_ref()) {
                crate::runtime::plugin_ffi_common::encode::string(&mut buf, &s);
            } else {
                crate::runtime::plugin_ffi_common::encode::string(
                    &mut buf,
                    &a.to_string_box().value,
                );
            }
        }
        buf
    }

    pub fn decode_first<'a>(&self, buf: &'a [u8]) -> Option<(u8, usize, &'a [u8])> {
        crate::runtime::plugin_ffi_common::decode::tlv_first(buf)
    }

    pub fn decode_nth<'a>(&self, buf: &'a [u8], n: usize) -> Option<(u8, usize, &'a [u8])> {
        crate::runtime::plugin_ffi_common::decode::tlv_nth(buf, n)
    }

    pub fn decode_plugin_handle(&self, payload: &[u8]) -> Option<(u32, u32)> {
        crate::runtime::plugin_ffi_common::decode::plugin_handle(payload)
    }

    pub fn encode_plugin_handle(&self, buf: &mut Vec<u8>, type_id: u32, instance_id: u32) {
        crate::runtime::plugin_ffi_common::encode::plugin_handle(buf, type_id, instance_id)
    }
}

#[cfg(test)]
mod tests {
    use super::TlvCodecBox;
    use crate::box_trait::{NyashBox, IntegerBox, StringBox};
    use crate::runtime::plugin_ffi_common;

    #[test]
    fn codec_encode_header_and_decode_first() {
        let c = TlvCodecBox::default();
        let mut buf = c.encode_header(0);
        // Append a small string TLV via plugin_ffi_common for simplicity
        crate::runtime::plugin_ffi_common::encode::string(&mut buf, "hi");
        let (tag, sz, payload) = c.decode_first(&buf).unwrap();
        assert_eq!(tag, 6); // TAG_STRING
        assert_eq!(sz as usize, payload.len());
        assert_eq!(crate::runtime::plugin_ffi_common::decode::string(payload), "hi");
    }

    #[test]
    fn codec_encode_args_mixed_int_and_string() {
        let c = TlvCodecBox::default();
        let args: Vec<Box<dyn NyashBox>> = vec![
            Box::new(IntegerBox::new(42)) as Box<dyn NyashBox>,
            Box::new(StringBox::new("ok")) as Box<dyn NyashBox>,
        ];
        let buf = c.encode_args(&args);
        // First entry should be I64 (tag=3) with value 42
        let (t0, _s0, p0) = c.decode_nth(&buf, 0).unwrap();
        assert_eq!(t0, 3);
        let v0 = crate::runtime::plugin_ffi_common::decode::u64(p0).unwrap();
        assert_eq!(v0, 42);
        // Second entry should be STRING (tag=6) with payload "ok"
        let (t1, _s1, p1) = c.decode_nth(&buf, 1).unwrap();
        assert_eq!(t1, 6);
        let s1 = crate::runtime::plugin_ffi_common::decode::string(p1);
        assert_eq!(s1, "ok");
    }

    #[test]
    fn codec_handle_roundtrip_tag8() {
        let c = TlvCodecBox::default();
        let mut buf = c.encode_header(1);
        // Encode a plugin handle (type_id=12, instance_id=345)
        c.encode_plugin_handle(&mut buf, 12, 345);
        let (tag, sz, payload) = c.decode_first(&buf).unwrap();
        assert_eq!(tag, 8); // TAG_HANDLE
        assert_eq!(sz, 8);
        let (tid, iid) = c.decode_plugin_handle(payload).unwrap();
        assert_eq!(tid, 12);
        assert_eq!(iid, 345);
    }
}
