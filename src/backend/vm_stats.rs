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
}

