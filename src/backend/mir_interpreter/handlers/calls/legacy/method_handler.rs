//! Method call handling with receiver resolution
//!
//! Provides complex receiver recovery logic for Method calls,
//! including Copy-based materialization and fallback strategies.

use super::super::super::*;
use crate::backend::mir_interpreter::VmConfig;

impl MirInterpreter {
    /// Handle Method callee: resolve receiver and execute method call
    pub(super) fn handle_method_call_legacy(
        &mut self,
        box_name: &str,
        method: &str,
        receiver: Option<ValueId>,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
                if method == "birth" && crate::config::env::cli_verbose() && !crate::config::env::cli_quiet() {
                    eprintln!("[vm-call] invoking birth() via method call");
                }
                if let Some(recv_id) = receiver {
                    // Built-in arity check (Fail-Fast)
                    {
                        let arity = args.len();
                        let mut type_name: Option<String> = None;
                        if let Ok(v) = self.reg_load(recv_id) {
                            match v {
                                VMValue::String(_) => { type_name = Some("StringBox".to_string()); },
                                VMValue::BoxRef(bx) => {
                                    let tn = bx.type_name().to_string();
                                    if matches!(tn.as_str(), "ArrayBox" | "MapBox" | "StringBox") { type_name = Some(tn); }
                                },
                                _ => {}
                            }
                        }
                        if let Some(tn) = type_name {
                            if method != "birth" {
                                if crate::runtime::type_registry::resolve_typebox_by_name(&tn).is_some() {
                                    if crate::runtime::type_registry::resolve_slot_by_name(&tn, method, arity).is_none() {
                                        // Prefer runtime registry; if absent, use core_box_methods fallback for a helpful message
                                        let known_rt = crate::runtime::type_registry::known_arities_for(&tn, method);
                                        let known_core = crate::common::core_box_methods::known_arities_for(&tn, method)
                                            .map(|s| s.iter().map(|&u| u as u8).collect::<Vec<u8>>());
                                        if let Some(known) = known_rt.or(known_core) {
                                            if !known.is_empty() {
                                                let known_usize: Vec<usize> = known.iter().map(|x| (*x) as usize).collect();
                                                return Err(VMError::InvalidInstruction(
                                                    crate::common::diagnostics::msg::no_method_arity(&tn, method, arity, &known_usize)
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Fail-Fast: forbid operations on unborn InstanceBox until birth()
                    if method != "birth" {
                        self.check_unborn_guard(recv_id)?;
                    }
                    // LocalSSA for receiver: prefer materialized id within current block
                    let recv_id = self.materialize_recv_in_current_block(recv_id);
                    // Primary: load receiver by id. If undefined, attempt a best-effort
                    // recovery by resolving a local Copy(dst := recv_id) in the same block,
                    // then fall back to arg[0] or error.
                    let recv_val = match self.reg_load(recv_id) {
                        Ok(v) => v,
                        Err(e) => {
                            // Try: find a preceding Copy in the current block with src=recv_id
                            let mut recovered: Option<VMValue> = None;
                            if let (Some(fn_name), Some(bb_id)) = (self.cur_fn.clone(), self.last_block) {
                                if let Some(fun) = self.functions.get(&fn_name) {
                                    if let Some(bb) = fun.blocks.get(&bb_id) {
                                        for inst in &bb.instructions {
                                            if let crate::mir::MirInstruction::Copy { dst, src } = inst {
                                                // Pattern A: we copied into recv_id just before the call (dst == recv_id)
                                                if *dst == recv_id {
                                                    if let Ok(v2) = self.reg_load(*src) {
                                                        recovered = Some(v2);
                                                        break;
                                                    }
                                                }
                                                // Pattern B: we copied from recv_id into a local tmp (src == recv_id)
                                                if *src == recv_id {
                                                    if let Ok(v2) = self.reg_load(*dst) {
                                                        recovered = Some(v2);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if let Some(v) = recovered {
                                v
                            } else {
                                // Autoload guard (plugins profile): when using kind="dylib" autoload is active,
                                // try a best-effort recovery by scanning current registers for a BoxRef whose
                                // type matches the expected receiver box (e.g., CounterBox/FixtureBox).
                                if std::env::var("NYASH_USING_DYLIB_AUTOLOAD").ok().as_deref() == Some("1") {
                                    let mut found: Option<VMValue> = None;
                                    for (_id, val) in self.regs.iter() {
                                        if let VMValue::BoxRef(bx) = val {
                                            // Match plugin-backed boxes by inner box_type when available
                                            if let Some(pb) = bx.as_any().downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>() {
                                                if pb.box_type == *box_name { found = Some(val.clone()); break; }
                                            } else if bx.type_name() == box_name {
                                                // Builtin boxes expose their concrete type name
                                                found = Some(val.clone()); break;
                                            }
                                        }
                                    }
                                    if let Some(v) = found { v } else {
                                        // Fallbacks (dev-only/tolerant modes)
                                        let cfg = VmConfig::global();
                                        let tolerate = cfg.recv_arg_fallback || cfg.tolerate_void;
                                        if tolerate {
                                            if let Some(a0) = args.get(0) { self.reg_load(*a0)? } else { return Err(e); }
                                        } else {
                                            // Narrow, behavior-preserving rescue: for ParserBox.* inside ParserBox.* functions,
                                            // fallback receiver to the `me` parameter of the current function.
                                            if box_name == "ParserBox" {
                                                if let Some(cur) = &self.cur_fn {
                                                    if cur.starts_with("ParserBox.") {
                                                        if let Some(fun) = self.functions.get(cur) {
                                                            if let Some(me_vid) = fun.params.first() {
                                                                if let Ok(mev) = self.reg_load(*me_vid) { return Ok(mev); }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // Minimal safety: common pure methods with undefined recv return neutral defaults
                                            if method == "length" { return Ok(VMValue::Integer(0)); }
                                            return Err(e);
                                        }
                                    }
                                } else {
                                    // Dev fallback: use args[0] as surrogate when enabled
                                    let cfg = VmConfig::global();
                                    let tolerate = cfg.recv_arg_fallback || cfg.tolerate_void;
                                    if tolerate {
                                        if let Some(a0) = args.get(0) { self.reg_load(*a0)? } else { return Err(e); }
                                    } else {
                                        // Narrow, behavior-preserving rescue: for ParserBox.* inside ParserBox.* functions,
                                        // fallback receiver to the `me` parameter of the current function.
                                        if box_name == "ParserBox" {
                                            if let Some(cur) = &self.cur_fn {
                                                if cur.starts_with("ParserBox.") {
                                                    if let Some(fun) = self.functions.get(cur) {
                                                        if let Some(me_vid) = fun.params.first() {
                                                            if let Ok(mev) = self.reg_load(*me_vid) { return Ok(mev); }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        // Minimal safety: common pure methods with undefined recv
                                        // return neutral defaults instead of crashing (length -> 0).
                                        if method == "length" { return Ok(VMValue::Integer(0)); }
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    };
                    let dev_trace = VmConfig::global().general_trace;
                    // Fast bridge for builtin boxes (Array) and common methods.
                    // Preserve legacy semantics when plugins are absent.
                    if let VMValue::BoxRef(bx) = &recv_val {
                        if let Some(arr) = bx
                            .as_any()
                            .downcast_ref::<crate::boxes::array::ArrayBox>()
                        {
                            if let Some(res) =
                                self.box_array_fastpath(arr, method, args)
                            {
                                return res;
                            }
                        }
                        if let Some(map) = bx
                            .as_any()
                            .downcast_ref::<crate::boxes::map_box::MapBox>()
                        {
                            if let Some(res) =
                                self.box_map_fastpath(map, method, args)
                            {
                                return res;
                            }
                        }
                        if let Some(s) = bx
                            .as_any()
                            .downcast_ref::<crate::boxes::string_box::StringBox>()
                        {
                            if let Some(res) =
                                self.box_string_fastpath(s, method, args)
                            {
                                return res;
                            }
                        }
                    }
                    // Fast bridge for VMValue::String (primitive string)
                    if let VMValue::String(s_inner) = &recv_val {
                        // Convert to StringBox temporarily for fastpath
                        let temp_sbox = crate::boxes::string_box::StringBox::new(s_inner.clone());
                        if let Some(res) = self.box_string_fastpath(&temp_sbox, method, args) {
                            return res;
                        }
                    }
                    // Minimal bridge for birth(): delegate to BoxCall handler and return Void
                    if method == "birth" {
                        let _ = self.handle_box_call(None, recv_id, method, args)?;
                        return Ok(VMValue::Void);
                    }
                    let is_kw = method == "keyword_to_token_type";
                    if dev_trace && is_kw {
                        let a0 = args.get(0).and_then(|id| self.reg_load(*id).ok());
                        eprintln!("[vm-trace] mcall {} argv0={:?}", method, a0);
                    }
                    let out = self.execute_method_call(&recv_val, method, args)?;
                    if dev_trace && is_kw {
                        eprintln!("[vm-trace] mret  {} -> {:?}", method, out);
                    }
                    Ok(out)
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method call missing receiver for {}",
                        method
                    )))
                }
    }
}
