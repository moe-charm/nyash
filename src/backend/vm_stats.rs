/*!
 * VM Stats & Diagnostics - prints per-instruction counters and timing
 *
 * Responsibility:
 * - Aggregate instruction counts (by MIR opcode)
 * - Print text or JSON summary based on env vars
 * - Stay side-effect free w.r.t. VM execution semantics
 */

use super::vm::VM;

impl VM {
    /// Print simple VM execution statistics when enabled via env var
    pub(super) fn maybe_print_stats(&mut self) {
        let enabled = std::env::var("NYASH_VM_STATS").ok().map(|v| v != "0").unwrap_or(false);
        if !enabled { return; }

        let elapsed_ms = self.exec_start.map(|t| t.elapsed().as_secs_f64() * 1000.0).unwrap_or(0.0);
        let mut items: Vec<(&str, usize)> = self.instr_counter.iter().map(|(k,v)| (*k, *v)).collect();
        items.sort_by(|a,b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let total: usize = items.iter().map(|(_,v)| *v).sum();

        let json_enabled = std::env::var("NYASH_VM_STATS_JSON").ok().map(|v| v != "0").unwrap_or(false)
            || std::env::var("NYASH_VM_STATS_FORMAT").map(|v| v == "json").unwrap_or(false);

        if json_enabled {
            let counts_obj: std::collections::BTreeMap<&str, usize> = self.instr_counter.iter().map(|(k,v)| (*k, *v)).collect();
            let top20: Vec<_> = items.iter().take(20).map(|(op,cnt)| {
                serde_json::json!({ "op": op, "count": cnt })
            }).collect();
            let now_ms = {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
            };
            let payload = serde_json::json!({
                "total": total,
                "elapsed_ms": elapsed_ms,
                "counts": counts_obj,
                "top20": top20,
                "timestamp_ms": now_ms
            });
            match serde_json::to_string_pretty(&payload) {
                Ok(s) => println!("{}", s),
                Err(_) => println!("{{\"total\":{},\"elapsed_ms\":{:.3}}}", total, elapsed_ms),
            }
        } else {
            println!("\n🧮 VM Stats: {} instructions in {:.3} ms", total, elapsed_ms);
            for (k, v) in items.into_iter().take(20) {
                println!("  {:>10}: {:>8}", k, v);
            }
        }
    }

    /// Print a concise unified JIT summary alongside VM stats when enabled
    pub(super) fn maybe_print_jit_unified_stats(&self) {
        // Show when either JIT stats requested or VM stats are on
        let jit_enabled = std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1");
        let vm_enabled = std::env::var("NYASH_VM_STATS").ok().map(|v| v != "0").unwrap_or(false);
        let jit_json = std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1");
        if !jit_enabled && !vm_enabled { return; }
        if let Some(jm) = &self.jit_manager {
            // Gather basic counters
            let sites = jm.sites();
            let compiled = jm.compiled_count();
            let total_hits: u64 = jm.total_hits();
            let ok = jm.exec_ok_count();
            let tr = jm.exec_trap_count();
            let total_exec = ok + tr;
            let fb_rate = if total_exec > 0 { (tr as f64) / (total_exec as f64) } else { 0.0 };
            let handles = crate::jit::rt::handles::len();
            let cfg = crate::jit::config::current();
            let caps = crate::jit::config::probe_capabilities();
            let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig { "b1_bool" } else { "i64_bool" };
            let b1_norm = crate::jit::rt::b1_norm_get();
            let ret_b1_hints = crate::jit::rt::ret_bool_hint_get();
            if jit_json {
                let payload = serde_json::json!({
                    "version": 1,
                    "sites": sites,
                    "compiled": compiled,
                    "hits": total_hits,
                    "exec_ok": ok,
                    "trap": tr,
                    "fallback_rate": fb_rate,
                    "handles": handles,
                    "abi_mode": abi_mode,
                    "abi_b1_enabled": cfg.native_bool_abi,
                    "abi_b1_supported": caps.supports_b1_sig,
                    "b1_norm_count": b1_norm,
                    "ret_bool_hint_count": ret_b1_hints,
                    "phi_total_slots": crate::jit::rt::phi_total_get(),
                    "phi_b1_slots": crate::jit::rt::phi_b1_get(),
                    "top5": jm.top_hits(5).into_iter().map(|(name, hits, compiled, handle)| {
                        serde_json::json!({
                            "name": name,
                            "hits": hits,
                            "compiled": compiled,
                            "handle": handle
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| String::from("{}")));
            } else {
                eprintln!("[JIT] summary: sites={} compiled={} hits={} exec_ok={} trap={} fallback_rate={:.2} handles={}",
                    sites, compiled, total_hits, ok, tr, fb_rate, handles);
                eprintln!("          abi_mode={} b1_norm_count={} ret_bool_hint_count={}", abi_mode, b1_norm, ret_b1_hints);
            }
        }
    }
}
