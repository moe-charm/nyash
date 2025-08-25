/*!
 * VM BoxCall Dispatch - method calls on boxes
 */

use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox};
use super::vm::{VM, VMError, VMValue};

impl VM {
    /// Call a method on a Box - simplified version of interpreter method dispatch
    pub(super) fn call_box_method_impl(&self, box_value: Box<dyn NyashBox>, method: &str, _args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, VMError> {
        // ResultBox (NyashResultBox - new)
        if let Some(result_box) = box_value.as_any().downcast_ref::<crate::boxes::result::NyashResultBox>() {
            match method {
                "is_ok" | "isOk" => { return Ok(result_box.is_ok()); }
                "get_value" | "getValue" => { return Ok(result_box.get_value()); }
                "get_error" | "getError" => { return Ok(result_box.get_error()); }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // ResultBox (box_trait::ResultBox - legacy)
        {
            #[allow(deprecated)]
            if let Some(result_box_legacy) = box_value.as_any().downcast_ref::<crate::box_trait::ResultBox>() {
                match method {
                    "is_ok" | "isOk" => { return Ok(result_box_legacy.is_ok()); }
                    "get_value" | "getValue" => { return Ok(result_box_legacy.get_value()); }
                    "get_error" | "getError" => { return Ok(result_box_legacy.get_error()); }
                    _ => return Ok(Box::new(VoidBox::new())),
                }
            }
        }

        // Generic fallback: toString for any Box type
        if method == "toString" {
            return Ok(Box::new(StringBox::new(box_value.to_string_box().value)));
        }

        // StringBox methods
        if let Some(string_box) = box_value.as_any().downcast_ref::<StringBox>() {
            match method {
                "length" | "len" => { return Ok(Box::new(IntegerBox::new(string_box.value.len() as i64))); }
                "toString" => { return Ok(Box::new(StringBox::new(string_box.value.clone()))); }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // ArrayBox methods (minimal set)
        if let Some(array_box) = box_value.as_any().downcast_ref::<crate::boxes::array::ArrayBox>() {
            match method {
                "push" => { if let Some(v) = _args.get(0) { return Ok(array_box.push(v.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: push(value) requires 1 arg"))); }
                "pop" => { return Ok(array_box.pop()); },
                "length" | "len" => { return Ok(array_box.length()); },
                "get" => { if let Some(i) = _args.get(0) { return Ok(array_box.get(i.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: get(index) requires 1 arg"))); }
                "set" => { if _args.len() >= 2 { return Ok(array_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); } return Ok(Box::new(StringBox::new("Error: set(index, value) requires 2 args"))); }
                "remove" => { if let Some(i) = _args.get(0) { return Ok(array_box.remove(i.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: remove(index) requires 1 arg"))); }
                "contains" => { if let Some(v) = _args.get(0) { return Ok(array_box.contains(v.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: contains(value) requires 1 arg"))); }
                "indexOf" => { if let Some(v) = _args.get(0) { return Ok(array_box.indexOf(v.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: indexOf(value) requires 1 arg"))); }
                "clear" => { return Ok(array_box.clear()); },
                "join" => { if let Some(sep) = _args.get(0) { return Ok(array_box.join(sep.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: join(sep) requires 1 arg"))); }
                "sort" => { return Ok(array_box.sort()); },
                "reverse" => { return Ok(array_box.reverse()); },
                "slice" => { if _args.len() >= 2 { return Ok(array_box.slice(_args[0].clone_or_share(), _args[1].clone_or_share())); } return Ok(Box::new(StringBox::new("Error: slice(start, end) requires 2 args"))); }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // MapBox methods (minimal set)
        if let Some(map_box) = box_value.as_any().downcast_ref::<crate::boxes::map_box::MapBox>() {
            match method {
                "set" => { if _args.len() >= 2 { return Ok(map_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())); } return Ok(Box::new(StringBox::new("Error: set(key, value) requires 2 args"))); }
                "get" => { if let Some(k) = _args.get(0) { return Ok(map_box.get(k.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: get(key) requires 1 arg"))); }
                "has" => { if let Some(k) = _args.get(0) { return Ok(map_box.has(k.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: has(key) requires 1 arg"))); }
                "delete" | "remove" => { if let Some(k) = _args.get(0) { return Ok(map_box.delete(k.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: delete(key) requires 1 arg"))); }
                "keys" => { return Ok(map_box.keys()); },
                "values" => { return Ok(map_box.values()); },
                "size" => { return Ok(map_box.size()); },
                "clear" => { return Ok(map_box.clear()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // SocketBox methods (minimal + timeouts)
        if let Some(sock) = box_value.as_any().downcast_ref::<crate::boxes::socket_box::SocketBox>() {
            match method {
                "bind" => { if _args.len() >= 2 { return Ok(sock.bind(_args[0].clone_or_share(), _args[1].clone_or_share())); } return Ok(Box::new(StringBox::new("Error: bind(address, port) requires 2 args"))); }
                "listen" => { if let Some(b) = _args.get(0) { return Ok(sock.listen(b.clone_or_share())); } return Ok(Box::new(StringBox::new("Error: listen(backlog) requires 1 arg"))); }
                "accept" => { return Ok(sock.accept()); },
                "acceptTimeout" | "accept_timeout" => { if let Some(ms) = _args.get(0) { return Ok(sock.accept_timeout(ms.clone_or_share())); } return Ok(Box::new(crate::box_trait::VoidBox::new())); }
                "connect" => { if _args.len() >= 2 { return Ok(sock.connect(_args[0].clone_or_share(), _args[1].clone_or_share())); } return Ok(Box::new(StringBox::new("Error: connect(address, port) requires 2 args"))); }
                "read" => { return Ok(sock.read()); },
                "recvTimeout" | "recv_timeout" => { if let Some(ms) = _args.get(0) { return Ok(sock.recv_timeout(ms.clone_or_share())); } return Ok(Box::new(StringBox::new(""))); }
                "write" => { if let Some(d) = _args.get(0) { return Ok(sock.write(d.clone_or_share())); } return Ok(Box::new(crate::box_trait::BoolBox::new(false))); }
                "close" => { return Ok(sock.close()); },
                "isServer" | "is_server" => { return Ok(sock.is_server()); },
                "isConnected" | "is_connected" => { return Ok(sock.is_connected()); },
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // IntegerBox methods
        if let Some(integer_box) = box_value.as_any().downcast_ref::<IntegerBox>() { match method { "toString" => { return Ok(Box::new(StringBox::new(integer_box.value.to_string()))); }, "abs" => { return Ok(Box::new(IntegerBox::new(integer_box.value.abs()))); }, _ => return Ok(Box::new(VoidBox::new())), } }

        // BoolBox methods
        if let Some(bool_box) = box_value.as_any().downcast_ref::<BoolBox>() { match method { "toString" => { return Ok(Box::new(StringBox::new(bool_box.value.to_string()))); }, _ => return Ok(Box::new(VoidBox::new())), } }

        // Default: return void for any unrecognized box type or method
        Ok(Box::new(VoidBox::new()))
    }

    /// Debug helper for BoxCall tracing (enabled via NYASH_VM_DEBUG_BOXCALL=1)
    pub(super) fn debug_log_boxcall(&self, recv: &VMValue, method: &str, args: &[Box<dyn NyashBox>], stage: &str, result: Option<&VMValue>) {
        if std::env::var("NYASH_VM_DEBUG_BOXCALL").ok().as_deref() == Some("1") {
            let recv_ty = match recv {
                VMValue::BoxRef(arc) => arc.type_name().to_string(),
                VMValue::Integer(_) => "Integer".to_string(),
                VMValue::Float(_) => "Float".to_string(),
                VMValue::Bool(_) => "Bool".to_string(),
                VMValue::String(_) => "String".to_string(),
                VMValue::Future(_) => "Future".to_string(),
                VMValue::Void => "Void".to_string(),
            };
            let args_desc: Vec<String> = args.iter().map(|a| a.type_name().to_string()).collect();
            if let Some(res) = result {
                let res_ty = match res {
                    VMValue::BoxRef(arc) => format!("BoxRef({})", arc.type_name()),
                    VMValue::Integer(_) => "Integer".to_string(),
                    VMValue::Float(_) => "Float".to_string(),
                    VMValue::Bool(_) => "Bool".to_string(),
                    VMValue::String(_) => "String".to_string(),
                    VMValue::Future(_) => "Future".to_string(),
                    VMValue::Void => "Void".to_string(),
                };
                eprintln!("[VM-BOXCALL][{}] recv_ty={} method={} argc={} args={:?} => result_ty={}", stage, recv_ty, method, args.len(), args_desc, res_ty);
            } else {
                eprintln!("[VM-BOXCALL][{}] recv_ty={} method={} argc={} args={:?}", stage, recv_ty, method, args.len(), args_desc);
            }
        }
    }
}
