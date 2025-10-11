//! FFI bridge for plugin method invocation and TLV encoding/decoding

use crate::bid::{BidError, BidResult};
use crate::box_trait::{NyashBox, StringBox};
use crate::boxes::{BufferBox, FloatBox};
use crate::runtime::plugin_loader_v2::enabled::loader::util::dbg_on;
use crate::runtime::plugin_loader_v2::enabled::PluginLoaderV2;

impl PluginLoaderV2 {
    /// Invoke a method on a plugin instance with TLV encoding/decoding
    pub fn invoke_instance_method(
        &self,
        box_type: &str,
        method_name: &str,
        instance_id: u32,
        args: &[Box<dyn NyashBox>],
    ) -> BidResult<Option<Box<dyn NyashBox>>> {
        let trace_fx = crate::config::env::trace_effects();
        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] invoke_instance_method ENTRY: box_type={} method={} instance_id={} argc={}",
                box_type, method_name, instance_id, args.len()
            );
        }
        if trace_fx {
            eprintln!(
                "{{\"kind\":\"plugin_call\",\"box\":\"{}\",\"method\":\"{}\",\"argc\":{}}}",
                box_type, method_name, args.len()
            );
        }
        // Resolve (lib_name, type_id) either from config or cached specs
        if dbg_on() {
            eprintln!("[PluginLoaderV2] About to call resolve_type_info for {}", box_type);
        }
        let (lib_name, type_id) = match resolve_type_info(self, box_type) {
            Ok(res) => res,
            Err(e) => {
                if dbg_on() {
                    eprintln!("[PluginLoaderV2] ERR: resolve_type_info failed for {}: {:?}", box_type, e);
                }
                return Err(e);
            }
        };

        // Resolve method id via config or TypeBox resolve()
        let method_id = match self.resolve_method_id(box_type, method_name) {
            Ok(mid) => mid,
            Err(e) => {
                if dbg_on() {
                    eprintln!(
                        "[PluginLoaderV2] ERR: method resolve failed for {}.{}: {:?}",
                        box_type, method_name, e
                    );
                }
                return Err(BidError::InvalidMethod);
            }
        };

        // Get plugin handle
        let plugins = self.plugins.read().map_err(|_| BidError::PluginError)?;
        let _plugin = plugins.get(&lib_name).ok_or(BidError::PluginError)?;

        // Phase A: If Final ABI is available for this box and env enabled, log its presence.
        if crate::config::env::plugin_abi_final() && crate::config::env::plugin_meta() {
            if let Ok(map) = self.box_specs.read() {
                if let Some(spec) = map.get(&(lib_name.clone(), box_type.to_string())) {
                    if spec.final_invoke.is_some() && dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] Final ABI detected for {}.{} (Phase A: using v2 invoke)",
                            lib_name, box_type
                        );
                    }
                }
            }
        }

        // Phase A: If Final ABI is enabled and present for this box, prefer it
        if crate::config::env::plugin_abi_final() {
            if let Ok(map) = self.box_specs.read() {
                if let Some(spec) = map.get(&(lib_name.clone(), box_type.to_string())) {
                    if let Some(final_invoke) = spec.final_invoke {
                        // Encode arguments to NyValueFfi (copy semantics for strings/bytes)
                        let mut owned_bufs: Vec<Vec<u8>> = Vec::new();
                        let values = encode_args_final(args, &mut owned_bufs);
                        let mut out = crate::runtime::plugin_loader_v2::enabled::types::NyResultFfi {
                            status: -1,
                            tag: 0,
                            ptr: std::ptr::null(),
                            len: 0,
                        };
                        let code = (final_invoke)(
                            type_id,
                            method_id,
                            instance_id,
                            values.as_ptr(),
                            values.len(),
                            &mut out,
                        );
                        if dbg_on() {
                            eprintln!(
                                "[PluginLoaderV2] final call {}.{}: code={} status={} tag={}",
                                box_type, method_name, code, out.status, out.tag
                            );
                        }
                        let ret_box: Box<dyn NyashBox> = decode_result_final(box_type, &out);
                        if trace_fx {
                            eprintln!(
                                "{{\"kind\":\"plugin_ret\",\"box\":\"{}\",\"method\":\"{}\",\"tag\":\"{}\"}}",
                                box_type, method_name, ret_box.type_name()
                            );
                        }
                        return Ok(Some(ret_box));
                    }
                }
            }
        }

        // Prefer invoking via the existing handle's invoke_fn when available
        let tlv = crate::runtime::codec::TlvCodecBox::default().encode_args(args);
        if dbg_on() {
            let hex_args = tlv.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
            eprintln!("[PluginLoaderV2] call args tlv: {}", hex_args);
        }
        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] call {}.{}: type_id={} method_id={} instance_id={}",
                box_type, method_name, type_id, method_id, instance_id
            );
        }
        let inv_fn = if let Some(h) = super::types::find_handle_by_instance(instance_id) {
            Some(h.invoke_fn)
        } else {
            None
        };
        let invoke_fn = inv_fn.unwrap_or(super::super::nyash_plugin_invoke_v2_shim);
        let (code, out_len, out) = super::host_bridge::invoke_alloc(
            invoke_fn,
            type_id,
            method_id,
            instance_id,
            &tlv,
        );
        if dbg_on() {
            eprintln!(
                "[PluginLoaderV2] call result {}.{}: code={} out_len={}",
                box_type, method_name, code, out_len
            );
        }
        let ret = decode_tlv_result(box_type, &out[..out_len]);
        if trace_fx {
            let tag = match &ret {
                Ok(Some(b)) => b.type_name().to_string(),
                Ok(None) => "<none>".to_string(),
                Err(_) => "<error>".to_string(),
            };
            eprintln!(
                "{{\"kind\":\"plugin_ret\",\"box\":\"{}\",\"method\":\"{}\",\"tag\":\"{}\"}}",
                box_type, method_name, tag
            );
        }
        ret
    }
}

/// Resolve type information for a box
fn resolve_type_info(loader: &PluginLoaderV2, box_type: &str) -> BidResult<(String, u32)> {
    if dbg_on() {
        eprintln!("[PluginLoaderV2] resolve_type_info: box_type={} config={}", box_type, loader.config.is_some());
    }
    if let Some(cfg) = loader.config.as_ref() {
        let cfg_path = loader.config_path_str();
        if dbg_on() {
            eprintln!("[PluginLoaderV2] resolve_type_info: trying config path {}", cfg_path);
        }
        let cfg_content = match std::fs::read_to_string(cfg_path) {
            Ok(s) => s,
            Err(e) => {
                if dbg_on() {
                    eprintln!("[PluginLoaderV2] ERR: failed to read {}: {:?}", cfg_path, e);
                }
                return Err(BidError::PluginError);
            }
        };
        let toml_value: toml::Value = match toml::from_str(&cfg_content) {
            Ok(v) => v,
            Err(e) => {
                if dbg_on() {
                    eprintln!("[PluginLoaderV2] ERR: failed to parse {}: {:?}", cfg_path, e);
                }
                return Err(BidError::PluginError);
            }
        };

        if let Some((lib_name, _)) = cfg.find_library_for_box(box_type) {
            if let Some(bc) = cfg.get_box_config(lib_name, box_type, &toml_value) {
                if dbg_on() {
                    eprintln!("[PluginLoaderV2] resolve_type_info(cfg): {} -> lib={} type_id={}", box_type, lib_name, bc.type_id);
                }
                return Ok((lib_name.to_string(), bc.type_id));
            } else {
                let key = (lib_name.to_string(), box_type.to_string());
                let map = loader.box_specs.read().map_err(|_| BidError::PluginError)?;
                let tid = map
                    .get(&key)
                    .and_then(|s| s.type_id)
                    .ok_or(BidError::InvalidType)?;
                if dbg_on() {
                    eprintln!("[PluginLoaderV2] resolve_type_info(cfg+spec): {} -> lib={} type_id={} (spec)", box_type, lib_name, tid);
                }
                return Ok((lib_name.to_string(), tid));
            }
        }
    } else {
        let map = loader.box_specs.read().map_err(|_| BidError::PluginError)?;
        if let Some(((lib, _), spec)) = map.iter().find(|((_, bt), _)| bt == box_type) {
            let tid = spec.type_id.ok_or(BidError::InvalidType)?;
            if dbg_on() {
                eprintln!("[PluginLoaderV2] resolve_type_info(spec): {} -> lib={} type_id={} (spec)", box_type, lib, tid);
            }
            return Ok((lib.clone(), tid));
        }
    }
    Err(BidError::InvalidType)
}

/// Decode TLV result into a NyashBox
fn decode_tlv_result(box_type: &str, data: &[u8]) -> BidResult<Option<Box<dyn NyashBox>>> {
    if let Some((tag, _sz, payload)) =
        crate::runtime::plugin_ffi_common::decode::tlv_first(data)
    {
        let bx: Box<dyn NyashBox> = match tag {
            1 => Box::new(crate::box_trait::BoolBox::new(
                crate::runtime::plugin_ffi_common::decode::bool(payload).unwrap_or(false),
            )),
            2 => Box::new(crate::box_trait::IntegerBox::new(
                crate::runtime::plugin_ffi_common::decode::i32(payload).unwrap_or(0) as i64,
            )),
            3 => {
                // i64 payload
                if payload.len() == 8 {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(payload);
                    Box::new(crate::box_trait::IntegerBox::new(i64::from_le_bytes(b)))
                } else {
                    Box::new(crate::box_trait::IntegerBox::new(0))
                }
            }
            5 => {
                let x = crate::runtime::plugin_ffi_common::decode::f64(payload).unwrap_or(0.0);
                Box::new(crate::boxes::FloatBox::new(x))
            }
            6 | 7 => {
                let s = crate::runtime::plugin_ffi_common::decode::string(payload);
                Box::new(crate::box_trait::StringBox::new(s))
            }
            8 => {
                // Plugin handle (type_id, instance_id)
                if let Some((ret_type, inst)) =
                    crate::runtime::plugin_ffi_common::decode::plugin_handle(payload)
                {
                    // Resolve real box type name and prefer HostHandle for ArrayBox to unify identity route
                    let real_bt = {
                        let loader = super::super::get_global_loader_v2();
                        let x = if let Ok(ro) = loader.read() {
                            ro.metadata_for_type_id(ret_type)
                                .map(|m| m.box_type)
                                .unwrap_or_else(|| box_type.to_string())
                        } else {
                            box_type.to_string()
                        };
                        x
                    };
                    let invoke_fn = super::super::nyash_plugin_invoke_v2_shim;

                    // If this is an ArrayBox handle, convert to HostHandleBox so downstream
                    // routes can use HostHandleRouter (and preserve identity via registry).
                    if real_bt == "ArrayBox" {
                        let pbox = super::types::make_plugin_box_v2(real_bt.clone(), ret_type, inst, invoke_fn);
                        let arc: std::sync::Arc<dyn crate::box_trait::NyashBox> =
                            std::sync::Arc::new(pbox);
                        let h = crate::runtime::host_handles::to_handle_arc(arc);
                        return Ok(Some(Box::new(
                            crate::runtime::host_handle_box::HostHandleBox::new(h),
                        )));
                    }

                    // Fallback: return PluginBoxV2 as-is for other types
                    let inner = super::types::get_or_create_handle(
                        ret_type,
                        inst,
                        super::super::nyash_plugin_invoke_v2_shim,
                        None,
                    );
                    Box::new(super::types::PluginBoxV2 { box_type: real_bt, inner })
                } else {
                    Box::new(crate::box_trait::VoidBox::new())
                }
            }
            9 => {
                // Host handle (u64) → return HostHandleBox; VM 層で実体 Arc に解決
                if let Some(u) = crate::runtime::plugin_ffi_common::decode::u64(payload) {
                    Box::new(crate::runtime::host_handle_box::HostHandleBox::new(u))
                } else {
                    Box::new(crate::box_trait::VoidBox::new())
                }
            }
            _ => Box::new(crate::box_trait::VoidBox::new()),
        };
        return Ok(Some(bx));
    }
    Ok(None)
}

// ---- Final ABI encode/decode (Phase A minimal) ----
use crate::runtime::plugin_loader_v2::enabled::types::{NyResultFfi, NyValueFfi};

fn encode_args_final(
    args: &[Box<dyn NyashBox>],
    owned: &mut Vec<Vec<u8>>,
) -> Vec<NyValueFfi> {
    let mut out = Vec::with_capacity(args.len());
    for a in args {
        // Bool
        if let Some(b) = a.as_any().downcast_ref::<crate::box_trait::BoolBox>() {
            let v = if b.value { 1u64 } else { 0u64 };
            out.push(NyValueFfi { tag: 1, reserved: 0, data0: v, data1: 0, ptr: std::ptr::null(), len: 0 });
            continue;
        }
        // Integer
        if let Some(i) = a.as_any().downcast_ref::<crate::box_trait::IntegerBox>() {
            out.push(NyValueFfi { tag: 3, reserved: 0, data0: i.value as u64, data1: 0, ptr: std::ptr::null(), len: 0 });
            continue;
        }
        // Float
        if let Some(f) = a.as_any().downcast_ref::<FloatBox>() {
            let bits = f.value.to_bits();
            out.push(NyValueFfi { tag: 5, reserved: 0, data0: bits, data1: 0, ptr: std::ptr::null(), len: 0 });
            continue;
        }
        // String
        if let Some(s) = a.as_any().downcast_ref::<crate::box_trait::StringBox>() {
            let bytes = s.value.as_bytes().to_vec();
            let len = bytes.len();
            owned.push(bytes);
            let buf_ptr = owned.last().unwrap().as_ptr();
            out.push(NyValueFfi { tag: 6, reserved: 0, data0: len as u64, data1: 0, ptr: buf_ptr, len });
            continue;
        }
        // Bytes (BufferBox)
        if let Some(buf) = a.as_any().downcast_ref::<BufferBox>() {
            let bytes = buf.to_vec();
            let len = bytes.len();
            owned.push(bytes);
            let buf_ptr = owned.last().unwrap().as_ptr();
            out.push(NyValueFfi { tag: 7, reserved: 0, data0: len as u64, data1: 0, ptr: buf_ptr, len });
            continue;
        }
        // PluginBoxV2: pass type_id/instance_id
        if let Some(pb) = a.as_any().downcast_ref::<super::types::PluginBoxV2>() {
            out.push(NyValueFfi { tag: 8, reserved: 0, data0: pb.inner.type_id as u64, data1: pb.inner.instance_id as u64, ptr: std::ptr::null(), len: 0 });
            continue;
        }
        // Fallback: stringify
        let s = a.to_string_box().value.into_bytes();
        let len = s.len();
        owned.push(s);
        let ptr = owned.last().unwrap().as_ptr();
        out.push(NyValueFfi { tag: 6, reserved: 0, data0: len as u64, data1: 0, ptr, len });
    }
    out
}

fn decode_result_final(_box_type: &str, r: &NyResultFfi) -> Box<dyn NyashBox> {
    if r.status != 0 {
        return Box::new(crate::box_trait::VoidBox::new());
    }
    // Decode by tag. Phase A': support bool/int/float/string/bytes
    match r.tag {
        // Bool: expect 1 byte payload (0/1). Missing payload => false
        1 => {
            if !r.ptr.is_null() && r.len >= 1 {
                let b = unsafe { *r.ptr } != 0;
                Box::new(crate::box_trait::BoolBox::new(b))
            } else {
                Box::new(crate::box_trait::BoolBox::new(false))
            }
        }
        // Integer (i64 LE): expect 8 bytes
        3 => {
            if !r.ptr.is_null() && r.len == 8 {
                let slice = unsafe { std::slice::from_raw_parts(r.ptr, 8) };
                let mut b = [0u8; 8];
                b.copy_from_slice(slice);
                let v = i64::from_le_bytes(b);
                Box::new(crate::box_trait::IntegerBox::new(v))
            } else {
                Box::new(crate::box_trait::IntegerBox::new(0))
            }
        }
        // Float (f64 bits LE): expect 8 bytes
        5 => {
            if !r.ptr.is_null() && r.len == 8 {
                let slice = unsafe { std::slice::from_raw_parts(r.ptr, 8) };
                let mut b = [0u8; 8];
                b.copy_from_slice(slice);
                let bits = u64::from_le_bytes(b);
                let f = f64::from_bits(bits);
                Box::new(FloatBox::new(f))
            } else {
                Box::new(FloatBox::new(0.0))
            }
        }
        // String (utf8)
        6 => {
            if !r.ptr.is_null() && r.len > 0 {
                let s = unsafe { std::slice::from_raw_parts(r.ptr, r.len) };
                let st = String::from_utf8_lossy(s).to_string();
                Box::new(StringBox::new(st))
            } else {
                Box::new(StringBox::new(""))
            }
        }
        // Bytes → BufferBox
        7 => {
            if !r.ptr.is_null() && r.len > 0 {
                let s = unsafe { std::slice::from_raw_parts(r.ptr, r.len) };
                Box::new(BufferBox::from_vec(s.to_vec()))
            } else {
                Box::new(BufferBox::from_vec(Vec::new()))
            }
        }
        _ => Box::new(crate::box_trait::VoidBox::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_final_string() {
        let data = b"abc";
        let r = NyResultFfi { status: 0, tag: 6, ptr: data.as_ptr(), len: data.len() };
        let b = decode_result_final("StringBox", &r);
        assert_eq!(b.as_any().downcast_ref::<StringBox>().unwrap().value, "abc");
    }

    #[test]
    fn decode_final_bool() {
        let one = [1u8];
        let r = NyResultFfi { status: 0, tag: 1, ptr: one.as_ptr(), len: one.len() };
        let b = decode_result_final("BoolBox", &r);
        assert!(b.as_any().downcast_ref::<crate::box_trait::BoolBox>().unwrap().value);
    }

    #[test]
    fn decode_final_int() {
        let v: i64 = 42;
        let bytes = v.to_le_bytes();
        let r = NyResultFfi { status: 0, tag: 3, ptr: bytes.as_ptr(), len: bytes.len() };
        let b = decode_result_final("IntegerBox", &r);
        assert_eq!(b.as_any().downcast_ref::<crate::box_trait::IntegerBox>().unwrap().value, 42);
    }

    #[test]
    fn decode_final_float() {
        let f: f64 = 3.5;
        let bytes = f.to_bits().to_le_bytes();
        let r = NyResultFfi { status: 0, tag: 5, ptr: bytes.as_ptr(), len: bytes.len() };
        let b = decode_result_final("FloatBox", &r);
        let fb = b.as_any().downcast_ref::<FloatBox>().unwrap();
        assert!((fb.value - 3.5).abs() < 1e-12);
    }

    #[test]
    fn decode_final_bytes() {
        let buf = [1u8, 2, 3, 4];
        let r = NyResultFfi { status: 0, tag: 7, ptr: buf.as_ptr(), len: buf.len() };
        let b = decode_result_final("BufferBox", &r);
        let bb = b.as_any().downcast_ref::<BufferBox>().unwrap();
        assert_eq!(bb.len(), 4);
    }

    #[test]
    fn encode_final_args_basic_types() {
        // Prepare args of various types
        let mut owned: Vec<Vec<u8>> = Vec::new();
        let args: Vec<Box<dyn NyashBox>> = vec![
            Box::new(crate::box_trait::BoolBox::new(true)),
            Box::new(crate::box_trait::IntegerBox::new(42)),
            Box::new(FloatBox::new(1.5)),
            Box::new(StringBox::new("hi".to_string())),
            Box::new(BufferBox::from_vec(vec![1u8, 2, 3])),
        ];
        let vals = encode_args_final(&args, &mut owned);
        assert_eq!(vals.len(), 5);
        assert_eq!(vals[0].tag, 1); // bool
        assert_eq!(vals[1].tag, 3); // i64
        assert_eq!(vals[2].tag, 5); // f64
        assert_eq!(vals[3].tag, 6); // string
        assert_eq!(vals[4].tag, 7); // bytes
        // Basic sanity for payload lengths
        assert_eq!(vals[0].len, 0);
        assert_eq!(vals[1].len, 0);
        assert_eq!(vals[2].len, 0);
        assert!(vals[3].len > 0);
        assert!(vals[4].len > 0);
    }
}
