//! host_slot.rs — Shared HostHandle slot invocation helper
//!
//! Responsibility
//! - Execute HostHandle router slots in a single place (builtin/plugin routers共通)。
//! - Handle TLV encode/decode, ERR_BUF_SMALL リトライ、Void 戻りの共通化。

use crate::backend::vm_types::VMValue;
use crate::box_trait::NyashBox;
use crate::runtime::host_handle_router::consts;
use crate::runtime::host_handles::HostHandle;

use super::tables::{HostSlotRoute, HostSlotRoutes};

pub fn invoke(hh: HostHandle, route: HostSlotRoute, args: &[VMValue]) -> Option<VMValue> {
    if route.arity != args.len() {
        return None;
    }
    let mut argv: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
    for v in args {
        argv.push(v.to_nyash_box());
    }
    let tlv_args = crate::runtime::plugin_ffi_common::encode_args(&argv);
    let mut out_buf = vec![0u8; 64];
    let mut out_len: usize = out_buf.len();
    let mut attempted_resize = false;
    loop {
        let rc = crate::runtime::host_api::nyrt_host_call_slot(
            hh,
            route.slot,
            if tlv_args.is_empty() {
                std::ptr::null()
            } else {
                tlv_args.as_ptr()
            },
            tlv_args.len(),
            out_buf.as_mut_ptr(),
            &mut out_len,
        );
        if rc == consts::ERR_BUF_SMALL && route.returns_value && !attempted_resize {
            let new_cap = out_len.max(out_buf.len().saturating_mul(2)).min(4096);
            if new_cap > out_buf.len() {
                out_buf.resize(new_cap, 0);
                out_len = out_buf.len();
                attempted_resize = true;
                continue;
            }
        }
        if rc != 0 {
            if route.returns_value
                && crate::runtime::env_gate_box::bool_any(&[
                    "HAKO_HOSTHANDLE_TEST_RET_MISMATCH",
                    "NYASH_HOSTHANDLE_TEST_RET_MISMATCH",
                ])
            {
                println!("hosthandle-test rc={}", rc);
                return Some(VMValue::Integer(rc as i64));
            }
            return None;
        }
        break;
    }
        if route.returns_value {
            if out_len >= 6 {
                if let Some((tag, _sz, payload)) =
                    crate::runtime::plugin_ffi_common::decode::tlv_first(&out_buf[..out_len])
                {
                    if let Some(v) = crate::runtime::host_api::vmvalue_from_tlv(tag, payload) {
                        let norm = normalize_host_value(v);
                        if std::env::var("NYASH_DEBUG_HOST_SLOT").ok().as_deref() == Some("1") {
                            eprintln!(
                                "[debug:host_slot] route={} slot={} -> {:?}",
                                route.names.first().copied().unwrap_or("<unknown>"),
                                route.slot,
                                norm
                            );
                        }
                        return Some(norm);
                    }
                }
            }
            None
        } else {
        Some(VMValue::Void)
    }
}

pub fn try_invoke_arc(
    bx: &std::sync::Arc<dyn NyashBox>,
    routes: HostSlotRoutes,
    method: &str,
    args: &[VMValue],
) -> Option<(HostSlotRoute, VMValue)> {
    if let Some(route) = routes.pick(method, args.len()) {
        let hh = crate::runtime::host_handles::to_handle_arc(bx.clone());
        if let Some(v) = invoke(hh, route, args) {
            return Some((route, v));
        }
    }
    None
}

fn normalize_host_value(value: VMValue) -> VMValue {
    if let VMValue::BoxRef(bx) = &value {
        if let Some(hh) = bx
            .as_any()
            .downcast_ref::<crate::runtime::host_handle_box::HostHandleBox>()
        {
            if let Some(real) = crate::runtime::host_handles::get(hh.id) {
                return VMValue::BoxRef(real);
            }
        }
    }
    value
}
