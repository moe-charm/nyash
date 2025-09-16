/*!
 * MIR pass: Inject method_id into BoxCall/PluginInvoke by resolving receiver box type.
 *
 * - Tracks NewBox dst -> box_type and propagates through simple copies.
 * - For BoxCall with missing method_id, resolves using:
 *     1) PluginHost config (plugins) when available
 *     2) Builtin slot registry (ArrayBox, StringBox, etc.)
 * - For PluginInvoke, rewrites to BoxCall with resolved method_id when possible.
 *
 * Scope: minimal dataflow (direct NewBox and Copy propagation). Phi/complex flows are TODO.
 */

use crate::mir::{MirInstruction as I, MirModule, ValueId};

pub fn inject_method_ids(module: &mut MirModule) -> usize {
    use crate::mir::slot_registry::resolve_slot_by_type_name;
    use std::collections::HashMap;

    // Try to access plugin host (optional in builds without plugins)
    let host = crate::runtime::get_global_plugin_host();
    let host_guard = host.read().ok();

    let mut injected = 0usize;

    for (_fname, func) in module.functions.iter_mut() {
        // Track simple value origins: ValueId -> type_name
        let mut origin: HashMap<ValueId, String> = HashMap::new();

        // Single forward pass is sufficient for NewBox/Copy cases
        for (_bid, block) in func.blocks.iter_mut() {
            for inst in block.instructions.iter_mut() {
                match inst {
                    I::NewBox { dst, box_type, .. } => {
                        origin.insert(*dst, box_type.clone());
                    }
                    I::Copy { dst, src } => {
                        if let Some(bt) = origin.get(src).cloned() {
                            origin.insert(*dst, bt);
                        }
                    }
                    I::BoxCall {
                        box_val,
                        method,
                        method_id,
                        ..
                    } => {
                        if method_id.is_none() {
                            if let Some(bt) = origin.get(box_val).cloned() {
                                // First try plugin host if available, else builtin slots
                                let mid_u16 = if let Some(h) = host_guard.as_ref() {
                                    // Try resolve via plugin config (may fail for builtins)
                                    match h.resolve_method(&bt, method) {
                                        Ok(mh) => Some(mh.method_id as u16),
                                        Err(_) => resolve_slot_by_type_name(&bt, method),
                                    }
                                } else {
                                    resolve_slot_by_type_name(&bt, method)
                                };
                                if let Some(mid) = mid_u16 {
                                    *method_id = Some(mid);
                                    injected += 1;
                                }
                            }
                        }
                    }
                    I::PluginInvoke {
                        dst,
                        box_val,
                        method,
                        args,
                        effects,
                    } => {
                        if let Some(bt) = origin.get(box_val).cloned() {
                            // Resolve id as above
                            let mid_u16 = if let Some(h) = host_guard.as_ref() {
                                match h.resolve_method(&bt, method) {
                                    Ok(mh) => Some(mh.method_id as u16),
                                    Err(_) => resolve_slot_by_type_name(&bt, method),
                                }
                            } else {
                                resolve_slot_by_type_name(&bt, method)
                            };
                            if let Some(mid) = mid_u16 {
                                // Rewrite to BoxCall with method_id
                                *inst = I::BoxCall {
                                    dst: dst.take(),
                                    box_val: *box_val,
                                    method: method.clone(),
                                    method_id: Some(mid),
                                    args: args.clone(),
                                    effects: *effects,
                                };
                                injected += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    injected
}
