#![cfg(feature = "jit-direct-only")]
use super::*;

impl NyashRunner {
    /// Run a file through independent JIT engine (no VM execute loop)
    pub(crate) fn run_file_jit_direct(&self, filename: &str) {
        use nyash_rust::{mir::MirCompiler, parser::NyashParser};
        use std::fs;
        let emit_err = |phase: &str, code: &str, msg: &str| {
            if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1")
                || std::env::var("NYASH_JIT_ERROR_JSON").ok().as_deref() == Some("1")
            {
                let payload = serde_json::json!({
                    "kind": "jit_direct_error",
                    "phase": phase,
                    "code": code,
                    "message": msg,
                    "file": filename,
                });
                println!("{}", payload.to_string());
            } else {
                eprintln!("[JIT-direct][{}][{}] {}", phase, code, msg);
            }
        };
        let code = match fs::read_to_string(filename) {
            Ok(s) => s,
            Err(e) => { emit_err("read_file", "IO", &format!("{}", e)); std::process::exit(1); }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(a) => a,
            Err(e) => { emit_err("parse", "SYNTAX", &format!("{}", e)); std::process::exit(1); }
        };
        let mut mc = MirCompiler::new();
        let cr = match mc.compile(ast) {
            Ok(m) => m,
            Err(e) => { emit_err("mir", "MIR_COMPILE", &format!("{}", e)); std::process::exit(1); }
        };
        let func = match cr.module.functions.get("main") {
            Some(f) => f,
            None => { emit_err("mir", "NO_MAIN", "No main function found"); std::process::exit(1); }
        };

        // Refuse write-effects in jit-direct when policy.read_only
        {
            use nyash_rust::mir::effect::Effect;
            let policy = nyash_rust::jit::policy::current();
            let mut writes = 0usize;
            for (_bbid, bb) in func.blocks.iter() {
                for inst in bb.instructions.iter() {
                    let mask = inst.effects();
                    if mask.contains(Effect::WriteHeap) { writes += 1; }
                }
                if let Some(term) = &bb.terminator {
                    if term.effects().contains(Effect::WriteHeap) { writes += 1; }
                }
            }
            if policy.read_only && writes > 0 {
                emit_err("policy","WRITE_EFFECTS", &format!("write-effects detected ({} ops). jit-direct is read-only at this stage.", writes));
                std::process::exit(1);
            }
        }

        // PHI-min config for jit-direct
        {
            let mut cfg = nyash_rust::jit::config::current();
            cfg.phi_min = true;
            nyash_rust::jit::config::set_current(cfg);
        }
        // minimal runtime hooks
        {
            let rt = nyash_rust::runtime::NyashRuntime::new();
            nyash_rust::runtime::global_hooks::set_from_runtime(&rt);
        }
        let mut engine = nyash_rust::jit::engine::JitEngine::new();
        match engine.compile_function("main", func) {
            Some(h) => {
                nyash_rust::jit::events::emit("compile", &func.signature.name, Some(h), None, serde_json::json!({}));
                // parse NYASH_JIT_ARGS
                let mut jit_args: Vec<nyash_rust::jit::abi::JitValue> = Vec::new();
                if let Ok(s) = std::env::var("NYASH_JIT_ARGS") { for raw in s.split(',') { let t = raw.trim(); if t.is_empty() { continue; } let v = if let Some(rest) = t.strip_prefix("i:") { rest.parse::<i64>().ok().map(nyash_rust::jit::abi::JitValue::I64) } else if let Some(rest) = t.strip_prefix("f:") { rest.parse::<f64>().ok().map(nyash_rust::jit::abi::JitValue::F64) } else if let Some(rest) = t.strip_prefix("b:") { let b = matches!(rest, "1"|"true"|"True"|"TRUE"); Some(nyash_rust::jit::abi::JitValue::Bool(b)) } else if let Some(rest) = t.strip_prefix("h:") { rest.parse::<u64>().ok().map(nyash_rust::jit::abi::JitValue::Handle) } else if t.eq_ignore_ascii_case("true") || t == "1" { Some(nyash_rust::jit::abi::JitValue::Bool(true)) } else if t.eq_ignore_ascii_case("false") || t == "0" { Some(nyash_rust::jit::abi::JitValue::Bool(false)) } else if let Ok(iv) = t.parse::<i64>() { Some(nyash_rust::jit::abi::JitValue::I64(iv)) } else if let Ok(fv) = t.parse::<f64>() { Some(nyash_rust::jit::abi::JitValue::F64(fv)) } else { None }; if let Some(jv) = v { jit_args.push(jv); } } }
                // coerce to MIR signature
                use nyash_rust::mir::MirType;
                let expected = &func.signature.params;
                if expected.len() != jit_args.len() { emit_err("args","COUNT_MISMATCH", &format!("expected={}, passed={}", expected.len(), jit_args.len())); eprintln!("Hint: set NYASH_JIT_ARGS as comma-separated values, e.g., i:42,f:3.14,b:true"); std::process::exit(1); }
                let mut coerced: Vec<nyash_rust::jit::abi::JitValue> = Vec::with_capacity(jit_args.len());
                for (exp, got) in expected.iter().zip(jit_args.iter()) {
                    let cv = match exp {
                        MirType::Integer => match got { nyash_rust::jit::abi::JitValue::I64(v)=>nyash_rust::jit::abi::JitValue::I64(*v), nyash_rust::jit::abi::JitValue::F64(f)=>nyash_rust::jit::abi::JitValue::I64(*f as i64), nyash_rust::jit::abi::JitValue::Bool(b)=>nyash_rust::jit::abi::JitValue::I64(if *b {1}else{0}), _=>nyash_rust::jit::abi::adapter::from_jit_value(got) },
                        MirType::Float => match got { nyash_rust::jit::abi::JitValue::F64(v)=>nyash_rust::jit::abi::JitValue::F64(*v), nyash_rust::jit::abi::JitValue::I64(i)=>nyash_rust::jit::abi::JitValue::F64(*i as f64), _=>nyash_rust::jit::abi::adapter::from_jit_value(got) },
                        MirType::Bool => match got { nyash_rust::jit::abi::JitValue::Bool(b)=>nyash_rust::jit::abi::JitValue::Bool(*b), nyash_rust::jit::abi::JitValue::I64(i)=>nyash_rust::jit::abi::JitValue::Bool(*i!=0), _=>nyash_rust::jit::abi::adapter::from_jit_value(got) },
                        _ => nyash_rust::jit::abi::adapter::from_jit_value(got),
                    };
                    coerced.push(cv);
                }
                match engine.execute_function(h, &coerced) {
                    Some(v) => {
                        let ret_ty = &func.signature.return_type;
                        let vmv = match (ret_ty, v) {
                            (MirType::Bool, nyash_rust::jit::abi::JitValue::Bool(b)) => nyash_rust::backend::vm::VMValue::Bool(b),
                            (MirType::Float, nyash_rust::jit::abi::JitValue::F64(f)) => nyash_rust::backend::vm::VMValue::Float(f),
                            (MirType::Integer, nyash_rust::jit::abi::JitValue::I64(i)) => nyash_rust::backend::vm::VMValue::Integer(i),
                            (_, v) => nyash_rust::jit::abi::adapter::from_jit_value(&v),
                        };
                        println!("✅ JIT-direct execution completed successfully!");
                        let (ety, sval) = match (ret_ty, &vmv) {
                            (MirType::Bool, nyash_rust::backend::vm::VMValue::Bool(b)) => ("Bool", b.to_string()),
                            (MirType::Float, nyash_rust::backend::vm::VMValue::Float(f)) => ("Float", format!("{}", f)),
                            (MirType::Integer, nyash_rust::backend::vm::VMValue::Integer(i)) => ("Integer", i.to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Integer(i)) => ("Integer", i.to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Float(f)) => ("Float", format!("{}", f)),
                            (_, nyash_rust::backend::vm::VMValue::Bool(b)) => ("Bool", b.to_string()),
                            (_, nyash_rust::backend::vm::VMValue::String(s)) => ("String", s.clone()),
                            (_, nyash_rust::backend::vm::VMValue::BoxRef(arc)) => ("BoxRef", arc.type_name().to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Future(_)) => ("Future", "<future>".to_string()),
                            (_, nyash_rust::backend::vm::VMValue::Void) => ("Void", "void".to_string()),
                        };
                        println!("ResultType(MIR): {}", ety);
                        println!("Result: {}", sval);
                        if std::env::var("NYASH_JIT_STATS_JSON").ok().as_deref() == Some("1") {
                            let cfg = nyash_rust::jit::config::current();
                            let caps = nyash_rust::jit::config::probe_capabilities();
                            let (phi_t, phi_b1, ret_b) = engine.last_lower_stats();
                            let abi_mode = if cfg.native_bool_abi && caps.supports_b1_sig { "b1_bool" } else { "i64_bool" };
                            let payload = serde_json::json!({
                                "version": 1,
                                "function": func.signature.name,
                                "abi_mode": abi_mode,
                                "abi_b1_enabled": cfg.native_bool_abi,
                                "abi_b1_supported": caps.supports_b1_sig,
                                "b1_norm_count": nyash_rust::jit::rt::b1_norm_get(),
                                "ret_bool_hint_count": nyash_rust::jit::rt::ret_bool_hint_get(),
                                "phi_total_slots": phi_t,
                                "phi_b1_slots": phi_b1,
                                "ret_bool_hint_used": ret_b,
                            });
                            println!("{}", payload.to_string());
                        }
                    }
                    None => {
                        nyash_rust::jit::events::emit("fallback", &func.signature.name, Some(h), None, serde_json::json!({"reason":"trap_or_missing"}));
                        emit_err("execute", "TRAP_OR_MISSING", "execution failed (trap or missing handle)");
                        std::process::exit(1);
                    }
                }
            }
            None => {
                emit_err("compile", "UNAVAILABLE", "Build with --features cranelift-jit");
                std::process::exit(1);
            }
        }
    }
}
