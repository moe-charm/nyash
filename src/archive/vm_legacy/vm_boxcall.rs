/*!
 * VM BoxCall Dispatch
 *
 * Purpose: Method calls on Box values (dispatch by concrete box type)
 * Responsibilities: call_box_method_impl, BoxCall debug logging
 * Key APIs: call_box_method_impl, debug_log_boxcall
 * Typical Callers: vm_instructions::execute_boxcall / VM::call_unified_method
 */

use super::vm::{VMError, VMValue, VM};
use crate::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox, VoidBox};

impl VM {
    /// Call a method on a Box - simplified version of interpreter method dispatch
    pub(super) fn call_box_method_impl(
        &self,
        box_value: Box<dyn NyashBox>,
        method: &str,
        _args: Vec<Box<dyn NyashBox>>,
    ) -> Result<Box<dyn NyashBox>, VMError> {
        // PluginBoxV2: delegate to unified plugin host (BID-FFI v1)
        if let Some(pbox) = box_value
            .as_any()
            .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
        {
            if std::env::var("NYASH_DEBUG_PLUGIN").ok().as_deref() == Some("1") {
                eprintln!(
                    "[VM][BoxCall→PluginInvoke] {}.{} inst_id={}",
                    pbox.box_type, method, pbox.inner.instance_id
                );
            }
            let host = crate::runtime::get_global_plugin_host();
            let host = host.read().unwrap();
            match host.invoke_instance_method(
                &pbox.box_type,
                method,
                pbox.inner.instance_id,
                &_args,
            ) {
                Ok(Some(val)) => {
                    return Ok(val);
                }
                Ok(None) => {
                    return Ok(Box::new(crate::box_trait::VoidBox::new()));
                }
                Err(e) => {
                    return Err(VMError::InvalidInstruction(format!(
                        "PluginInvoke failed via BoxCall: {}.{} ({})",
                        pbox.box_type,
                        method,
                        e.message()
                    )));
                }
            }
        }

        // MathBox methods (minimal set used in 10.9)
        if let Some(math) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::math_box::MathBox>()
        {
            match method {
                "min" => {
                    if _args.len() >= 2 {
                        return Ok(math.min(_args[0].clone_or_share(), _args[1].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: min(a, b) requires 2 args")));
                }
                "max" => {
                    if _args.len() >= 2 {
                        return Ok(math.max(_args[0].clone_or_share(), _args[1].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: max(a, b) requires 2 args")));
                }
                "abs" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(math.abs(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: abs(x) requires 1 arg")));
                }
                "sin" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(math.sin(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: sin(x) requires 1 arg")));
                }
                "cos" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(math.cos(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: cos(x) requires 1 arg")));
                }
                _ => {
                    return Ok(Box::new(VoidBox::new()));
                }
            }
        }
        // ResultBox (NyashResultBox - new)
        if let Some(result_box) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::result::NyashResultBox>()
        {
            match method {
                "is_ok" | "isOk" => {
                    return Ok(result_box.is_ok());
                }
                "get_value" | "getValue" => {
                    return Ok(result_box.get_value());
                }
                "get_error" | "getError" => {
                    return Ok(result_box.get_error());
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // Legacy box_trait::ResultBox path removed (migration complete)

        // Generic fallback: toString for any Box type
        if method == "toString" {
            return Ok(Box::new(StringBox::new(box_value.to_string_box().value)));
        }

        // JitConfigBox methods
        if let Some(jcb) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::jit_config_box::JitConfigBox>()
        {
            match method {
                "get" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(jcb
                            .get_flag(&k.to_string_box().value)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("get(name) requires 1 arg")));
                }
                "set" => {
                    if _args.len() >= 2 {
                        let k = _args[0].to_string_box().value;
                        let v = _args[1].to_string_box().value;
                        let on = matches!(v.as_str(), "1" | "true" | "True" | "on" | "ON");
                        return Ok(jcb
                            .set_flag(&k, on)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("set(name, bool) requires 2 args")));
                }
                "getThreshold" => {
                    return Ok(jcb.get_threshold());
                }
                "setThreshold" => {
                    if let Some(v) = _args.get(0) {
                        let iv = v.to_string_box().value.parse::<i64>().unwrap_or(0);
                        return Ok(jcb
                            .set_threshold(iv)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("setThreshold(n) requires 1 arg")));
                }
                "apply" => {
                    return Ok(jcb.apply());
                }
                "toJson" => {
                    return Ok(jcb.to_json());
                }
                "fromJson" => {
                    if let Some(s) = _args.get(0) {
                        return Ok(jcb
                            .from_json(&s.to_string_box().value)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("fromJson(json) requires 1 arg")));
                }
                "enable" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(jcb
                            .set_flag(&k.to_string_box().value, true)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("enable(name) requires 1 arg")));
                }
                "disable" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(jcb
                            .set_flag(&k.to_string_box().value, false)
                            .unwrap_or_else(|e| Box::new(StringBox::new(e.to_string()))));
                    }
                    return Ok(Box::new(StringBox::new("disable(name) requires 1 arg")));
                }
                "summary" => {
                    return Ok(jcb.summary());
                }
                _ => {
                    return Ok(Box::new(VoidBox::new()));
                }
            }
        }

        // JitPolicyBox methods
        if let Some(jpb) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::jit_policy_box::JitPolicyBox>()
        {
            match method {
                "get" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(jpb.get_flag(&k.to_string_box().value));
                    }
                    return Ok(Box::new(StringBox::new("get(name) requires 1 arg")));
                }
                "set" => {
                    if _args.len() >= 2 {
                        let k = _args[0].to_string_box().value;
                        let v = _args[1].to_string_box().value;
                        let on = matches!(v.as_str(), "1" | "true" | "True" | "on" | "ON");
                        return Ok(jpb.set_flag(&k, on));
                    }
                    return Ok(Box::new(StringBox::new("set(name, bool) requires 2 args")));
                }
                "setWhitelistCsv" | "set_whitelist_csv" => {
                    if let Some(s) = _args.get(0) {
                        return Ok(jpb.set_whitelist_csv(&s.to_string_box().value));
                    }
                    return Ok(Box::new(StringBox::new(
                        "setWhitelistCsv(csv) requires 1 arg",
                    )));
                }
                "addWhitelist" | "add_whitelist" => {
                    if let Some(s) = _args.get(0) {
                        return Ok(jpb.add_whitelist(&s.to_string_box().value));
                    }
                    return Ok(Box::new(StringBox::new(
                        "addWhitelist(name) requires 1 arg",
                    )));
                }
                "clearWhitelist" | "clear_whitelist" => {
                    return Ok(jpb.clear_whitelist());
                }
                "enablePreset" | "enable_preset" => {
                    if let Some(s) = _args.get(0) {
                        return Ok(jpb.enable_preset(&s.to_string_box().value));
                    }
                    return Ok(Box::new(StringBox::new(
                        "enablePreset(name) requires 1 arg",
                    )));
                }
                _ => {
                    return Ok(Box::new(VoidBox::new()));
                }
            }
        }

        // JitStatsBox methods
        if let Some(jsb) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::jit_stats_box::JitStatsBox>()
        {
            match method {
                "toJson" => {
                    return Ok(jsb.to_json());
                }
                "top5" => {
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.top_hits(5);
                        let arr: Vec<serde_json::Value> = v
                            .into_iter()
                            .map(|(name, hits, compiled, handle)| {
                                serde_json::json!({
                                    "name": name,
                                    "hits": hits,
                                    "compiled": compiled,
                                    "handle": handle
                                })
                            })
                            .collect();
                        let s = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string());
                        return Ok(Box::new(StringBox::new(s)));
                    }
                    return Ok(Box::new(StringBox::new("[]")));
                }
                "summary" => {
                    let cfg = crate::jit::config::current();
                    let caps = crate::jit::config::probe_capabilities();
                    let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig {
                        "b1_bool"
                    } else {
                        "i64_bool"
                    };
                    let b1_norm = crate::jit::rt::b1_norm_get();
                    let ret_b1 = crate::jit::rt::ret_bool_hint_get();
                    let mut payload = serde_json::json!({
                        "abi_mode": abi_mode,
                        "abi_b1_enabled": cfg.native_bool_abi,
                        "abi_b1_supported": caps.supports_b1_sig,
                        "b1_norm_count": b1_norm,
                        "ret_bool_hint_count": ret_b1,
                        "top5": [],
                        "perFunction": []
                    });
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.top_hits(5);
                        let top5: Vec<serde_json::Value> = v.into_iter().map(|(name, hits, compiled, handle)| serde_json::json!({
                            "name": name, "hits": hits, "compiled": compiled, "handle": handle
                        })).collect();
                        let perf = jm.per_function_stats();
                        let per_arr: Vec<serde_json::Value> = perf.into_iter().map(|(name, phi_t, phi_b1, rb, hits, compiled, handle)| serde_json::json!({
                            "name": name, "phi_total": phi_t, "phi_b1": phi_b1, "ret_bool_hint": rb, "hits": hits, "compiled": compiled, "handle": handle
                        })).collect();
                        if let Some(obj) = payload.as_object_mut() {
                            obj.insert("top5".to_string(), serde_json::Value::Array(top5));
                            obj.insert(
                                "perFunction".to_string(),
                                serde_json::Value::Array(per_arr),
                            );
                        }
                    }
                    let s =
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
                    return Ok(Box::new(StringBox::new(s)));
                }
                "perFunction" | "per_function" => {
                    if let Some(jm) = &self.jit_manager {
                        let v = jm.per_function_stats();
                        let arr: Vec<serde_json::Value> = v
                            .into_iter()
                            .map(|(name, phi_t, phi_b1, rb, hits, compiled, handle)| {
                                serde_json::json!({
                                    "name": name,
                                    "phi_total": phi_t,
                                    "phi_b1": phi_b1,
                                    "ret_bool_hint": rb,
                                    "hits": hits,
                                    "compiled": compiled,
                                    "handle": handle,
                                })
                            })
                            .collect();
                        let s =
                            serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".to_string());
                        return Ok(Box::new(StringBox::new(s)));
                    }
                    return Ok(Box::new(StringBox::new("[]")));
                }
                _ => {
                    return Ok(Box::new(VoidBox::new()));
                }
            }
        }

        // StringBox methods
        if let Some(string_box) = box_value.as_any().downcast_ref::<StringBox>() {
            match method {
                "length" | "len" => {
                    return Ok(Box::new(IntegerBox::new(string_box.value.len() as i64)));
                }
                "toString" => {
                    return Ok(Box::new(StringBox::new(string_box.value.clone())));
                }
                "substring" => {
                    if _args.len() >= 2 {
                        let s = match _args[0].to_string_box().value.parse::<i64>() {
                            Ok(v) => v.max(0) as usize,
                            Err(_) => 0,
                        };
                        let e = match _args[1].to_string_box().value.parse::<i64>() {
                            Ok(v) => v.max(0) as usize,
                            Err(_) => string_box.value.chars().count(),
                        };
                        return Ok(string_box.substring(s, e));
                    }
                    return Ok(Box::new(VoidBox::new()));
                }
                "concat" => {
                    if let Some(arg0) = _args.get(0) {
                        let out = format!("{}{}", string_box.value, arg0.to_string_box().value);
                        return Ok(Box::new(StringBox::new(out)));
                    }
                    return Ok(Box::new(VoidBox::new()));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // ArrayBox methods (minimal set)
        if let Some(array_box) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::array::ArrayBox>()
        {
            match method {
                "push" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(array_box.push(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: push(value) requires 1 arg",
                    )));
                }
                "pop" => {
                    return Ok(array_box.pop());
                }
                "length" | "len" => {
                    return Ok(array_box.length());
                }
                "get" => {
                    if let Some(i) = _args.get(0) {
                        return Ok(array_box.get(i.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: get(index) requires 1 arg")));
                }
                "set" => {
                    if _args.len() >= 2 {
                        return Ok(
                            array_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())
                        );
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: set(index, value) requires 2 args",
                    )));
                }
                "remove" => {
                    if let Some(i) = _args.get(0) {
                        return Ok(array_box.remove(i.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: remove(index) requires 1 arg",
                    )));
                }
                "contains" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(array_box.contains(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: contains(value) requires 1 arg",
                    )));
                }
                "indexOf" => {
                    if let Some(v) = _args.get(0) {
                        return Ok(array_box.indexOf(v.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: indexOf(value) requires 1 arg",
                    )));
                }
                "clear" => {
                    return Ok(array_box.clear());
                }
                "join" => {
                    if let Some(sep) = _args.get(0) {
                        return Ok(array_box.join(sep.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: join(sep) requires 1 arg")));
                }
                "sort" => {
                    return Ok(array_box.sort());
                }
                "reverse" => {
                    return Ok(array_box.reverse());
                }
                "slice" => {
                    if _args.len() >= 2 {
                        return Ok(
                            array_box.slice(_args[0].clone_or_share(), _args[1].clone_or_share())
                        );
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: slice(start, end) requires 2 args",
                    )));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // MapBox methods (minimal set)
        if let Some(map_box) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::map_box::MapBox>()
        {
            match method {
                "set" => {
                    if _args.len() >= 2 {
                        return Ok(
                            map_box.set(_args[0].clone_or_share(), _args[1].clone_or_share())
                        );
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: set(key, value) requires 2 args",
                    )));
                }
                "get" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(map_box.get(k.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: get(key) requires 1 arg")));
                }
                "has" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(map_box.has(k.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("Error: has(key) requires 1 arg")));
                }
                "delete" | "remove" => {
                    if let Some(k) = _args.get(0) {
                        return Ok(map_box.delete(k.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: delete(key) requires 1 arg",
                    )));
                }
                "keys" => {
                    return Ok(map_box.keys());
                }
                "values" => {
                    return Ok(map_box.values());
                }
                "size" => {
                    return Ok(map_box.size());
                }
                "clear" => {
                    return Ok(map_box.clear());
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // TaskGroupBox methods (scaffold → instance内の所有Futureに対して実行)
        if box_value
            .as_any()
            .downcast_ref::<crate::boxes::task_group_box::TaskGroupBox>()
            .is_some()
        {
            let mut owned = box_value;
            if let Some(tg) = (&mut *owned)
                .as_any_mut()
                .downcast_mut::<crate::boxes::task_group_box::TaskGroupBox>()
            {
                match method {
                    "cancelAll" | "cancel_all" => {
                        return Ok(tg.cancelAll());
                    }
                    "joinAll" | "join_all" => {
                        let ms = _args
                            .get(0)
                            .map(|a| a.to_string_box().value.parse::<i64>().unwrap_or(2000));
                        return Ok(tg.joinAll(ms));
                    }
                    _ => {
                        return Ok(Box::new(VoidBox::new()));
                    }
                }
            }
            return Ok(Box::new(VoidBox::new()));
        }

        // P2PBox methods (minimal)
        if let Some(p2p) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::p2p_box::P2PBox>()
        {
            match method {
                "send" => {
                    if _args.len() >= 2 {
                        return Ok(p2p.send(_args[0].clone_or_share(), _args[1].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: send(to, intent) requires 2 args",
                    )));
                }
                "on" => {
                    if _args.len() >= 2 {
                        return Ok(p2p.on(_args[0].clone_or_share(), _args[1].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: on(intent, handler) requires 2 args",
                    )));
                }
                "onOnce" | "on_once" => {
                    if _args.len() >= 2 {
                        return Ok(
                            p2p.on_once(_args[0].clone_or_share(), _args[1].clone_or_share())
                        );
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: onOnce(intent, handler) requires 2 args",
                    )));
                }
                "off" => {
                    if _args.len() >= 1 {
                        return Ok(p2p.off(_args[0].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: off(intent) requires 1 arg",
                    )));
                }
                "getLastFrom" => {
                    return Ok(p2p.get_last_from());
                }
                "getLastIntentName" => {
                    return Ok(p2p.get_last_intent_name());
                }
                "debug_nodes" | "debugNodes" => {
                    return Ok(p2p.debug_nodes());
                }
                "getNodeId" | "getId" => {
                    return Ok(p2p.get_node_id());
                }
                "getTransportType" | "transport" => {
                    return Ok(p2p.get_transport_type());
                }
                "isReachable" => {
                    if let Some(n) = _args.get(0) {
                        return Ok(p2p.is_reachable(n.clone_or_share()));
                    }
                    return Ok(Box::new(BoolBox::new(false)));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // SocketBox methods (minimal + timeouts)
        if let Some(sock) = box_value
            .as_any()
            .downcast_ref::<crate::boxes::socket_box::SocketBox>()
        {
            match method {
                "bind" => {
                    if _args.len() >= 2 {
                        return Ok(sock.bind(_args[0].clone_or_share(), _args[1].clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: bind(address, port) requires 2 args",
                    )));
                }
                "listen" => {
                    if let Some(b) = _args.get(0) {
                        return Ok(sock.listen(b.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: listen(backlog) requires 1 arg",
                    )));
                }
                "accept" => {
                    return Ok(sock.accept());
                }
                "acceptTimeout" | "accept_timeout" => {
                    if let Some(ms) = _args.get(0) {
                        return Ok(sock.accept_timeout(ms.clone_or_share()));
                    }
                    return Ok(Box::new(crate::box_trait::VoidBox::new()));
                }
                "connect" => {
                    if _args.len() >= 2 {
                        return Ok(
                            sock.connect(_args[0].clone_or_share(), _args[1].clone_or_share())
                        );
                    }
                    return Ok(Box::new(StringBox::new(
                        "Error: connect(address, port) requires 2 args",
                    )));
                }
                "read" => {
                    return Ok(sock.read());
                }
                "recvTimeout" | "recv_timeout" => {
                    if let Some(ms) = _args.get(0) {
                        return Ok(sock.recv_timeout(ms.clone_or_share()));
                    }
                    return Ok(Box::new(StringBox::new("")));
                }
                "write" => {
                    if let Some(d) = _args.get(0) {
                        return Ok(sock.write(d.clone_or_share()));
                    }
                    return Ok(Box::new(crate::box_trait::BoolBox::new(false)));
                }
                "close" => {
                    return Ok(sock.close());
                }
                "isServer" | "is_server" => {
                    return Ok(sock.is_server());
                }
                "isConnected" | "is_connected" => {
                    return Ok(sock.is_connected());
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // IntegerBox methods
        if let Some(integer_box) = box_value.as_any().downcast_ref::<IntegerBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(integer_box.value.to_string())));
                }
                "abs" => {
                    return Ok(Box::new(IntegerBox::new(integer_box.value.abs())));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // BoolBox methods
        if let Some(bool_box) = box_value.as_any().downcast_ref::<BoolBox>() {
            match method {
                "toString" => {
                    return Ok(Box::new(StringBox::new(bool_box.value.to_string())));
                }
                _ => return Ok(Box::new(VoidBox::new())),
            }
        }

        // Default: return void for any unrecognized box type or method
        Ok(Box::new(VoidBox::new()))
    }

    /// Debug helper for BoxCall tracing (enabled via NYASH_VM_DEBUG_BOXCALL=1)
    pub(super) fn debug_log_boxcall(
        &self,
        recv: &VMValue,
        method: &str,
        args: &[Box<dyn NyashBox>],
        stage: &str,
        result: Option<&VMValue>,
    ) {
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
                eprintln!(
                    "[VM-BOXCALL][{}] recv_ty={} method={} argc={} args={:?} => result_ty={}",
                    stage,
                    recv_ty,
                    method,
                    args.len(),
                    args_desc,
                    res_ty
                );
            } else {
                eprintln!(
                    "[VM-BOXCALL][{}] recv_ty={} method={} argc={} args={:?}",
                    stage,
                    recv_ty,
                    method,
                    args.len(),
                    args_desc
                );
            }
        }
    }
}
