use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, mir::{MirCompiler, MirInstruction}, box_trait::IntegerBox};
use nyash_rust::mir::passes::method_id_inject::inject_method_ids;
use std::{fs, process};
use serde_json::json;

impl NyashRunner {
    /// Execute LLVM mode (split)
    pub(crate) fn execute_llvm_mode(&self, filename: &str) {
        // Initialize plugin host so method_id injection can resolve plugin calls
        crate::runner_plugin_init::init_bid_plugins();

        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };

        // Parse to AST
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        // Compile to MIR
        let mut mir_compiler = MirCompiler::new();
        let compile_result = match mir_compiler.compile(ast) {
            Ok(result) => result,
            Err(e) => { eprintln!("❌ MIR compilation error: {}", e); process::exit(1); }
        };

        println!("📊 MIR Module compiled successfully!");
        println!("📊 Functions: {}", compile_result.module.functions.len());

        // Inject method_id for BoxCall/PluginInvoke where resolvable (by-id path)
        #[allow(unused_mut)]
        let mut module = compile_result.module.clone();
        let injected = inject_method_ids(&mut module);
        if injected > 0 && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[LLVM] method_id injected: {} places", injected);
        }

        // If explicit object path is requested, emit object only
        if let Ok(out_path) = std::env::var("NYASH_LLVM_OBJ_OUT") {
            #[cfg(feature = "llvm")]
            {
                // Harness path (optional): if NYASH_LLVM_USE_HARNESS=1, try Python/llvmlite first.
                let use_harness = std::env::var("NYASH_LLVM_USE_HARNESS").ok().as_deref() == Some("1");
                if use_harness {
                    if let Some(parent) = std::path::Path::new(&out_path).parent() { let _ = std::fs::create_dir_all(parent); }
                    let py = which::which("python3").ok();
                    if let Some(py3) = py {
                        let harness = std::path::Path::new("tools/llvmlite_harness.py");
                        if harness.exists() {
                            // 1) Emit MIR(JSON) to a temp file
                            let tmp_dir = std::path::Path::new("tmp");
                            let _ = std::fs::create_dir_all(tmp_dir);
                            let mir_json_path = tmp_dir.join("nyash_harness_mir.json");
                            if let Err(e) = emit_mir_json_for_harness(&module, &mir_json_path) {
                                eprintln!("❌ MIR JSON emit error: {}", e);
                                process::exit(1);
                            }
                            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!("[Runner/LLVM] using llvmlite harness → {} (mir={})", out_path, mir_json_path.display());
                            }
                            // 2) Run harness with --in/--out（失敗時は即エラー）
                            let status = std::process::Command::new(py3)
                                .args([harness.to_string_lossy().as_ref(), "--in", &mir_json_path.display().to_string(), "--out", &out_path])
                                .status().map_err(|e| format!("spawn harness: {}", e)).unwrap();
                            if !status.success() {
                                eprintln!("❌ llvmlite harness failed (status={})", status.code().unwrap_or(-1));
                                process::exit(1);
                            }
                            // Verify
                            match std::fs::metadata(&out_path) {
                                Ok(meta) if meta.len() > 0 => {
                                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                        eprintln!("[LLVM] object emitted by harness: {} ({} bytes)", out_path, meta.len());
                                    }
                                    return;
                                }
                                _ => {
                                    eprintln!("❌ harness output not found or empty: {}", out_path);
                                    process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("❌ harness script not found: {}", harness.display());
                            process::exit(1);
                        }
                    }
                    eprintln!("❌ python3 not found in PATH. Install Python 3 to use the harness.");
                    process::exit(1);
                } else {
                    use nyash_rust::backend::llvm_compile_to_object;
                    // Ensure parent directory exists for the object file
                    if let Some(parent) = std::path::Path::new(&out_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[Runner/LLVM] emitting object to {} (cwd={})", out_path, std::env::current_dir().map(|p| p.display().to_string()).unwrap_or_default());
                    }
                    if let Err(e) = llvm_compile_to_object(&module, &out_path) {
                        eprintln!("❌ LLVM object emit error: {}", e);
                        process::exit(1);
                    }
                }
                // Verify object presence and size (>0)
                match std::fs::metadata(&out_path) {
                    Ok(meta) => {
                        if meta.len() == 0 {
                            eprintln!("❌ LLVM object is empty: {}", out_path);
                            process::exit(1);
                        }
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[LLVM] object emitted: {} ({} bytes)", out_path, meta.len());
                        }
                    }
                    Err(e) => {
                        // Try one immediate retry by writing through the compiler again (rare FS lag safeguards)
                        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                            eprintln!("[Runner/LLVM] object not found after emit, retrying once: {} ({})", out_path, e);
                        }
                        if let Err(e2) = nyash_rust::backend::llvm_compile_to_object(&module, &out_path) {
                            eprintln!("❌ LLVM object emit error (retry): {}", e2);
                            process::exit(1);
                        }
                        match std::fs::metadata(&out_path) {
                            Ok(meta2) => {
                                if meta2.len() == 0 {
                                    eprintln!("❌ LLVM object is empty (after retry): {}", out_path);
                                    process::exit(1);
                                }
                                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                    eprintln!("[LLVM] object emitted after retry: {} ({} bytes)", out_path, meta2.len());
                                }
                            }
                            Err(e3) => {
                                eprintln!("❌ LLVM object not found after emit (retry): {} ({})", out_path, e3);
                                process::exit(1);
                            }
                        }
                    }
                }
                return;
            }
            #[cfg(not(feature = "llvm"))]
            {
                eprintln!("❌ LLVM backend not available (object emit).");
                process::exit(1);
            }
        }

        // Execute via LLVM backend (mock or real)
        #[cfg(feature = "llvm")]
        {
            use nyash_rust::backend::llvm_compile_and_execute;
            let temp_path = "nyash_llvm_temp";
            match llvm_compile_and_execute(&module, temp_path) {
                Ok(result) => {
                    if let Some(int_result) = result.as_any().downcast_ref::<IntegerBox>() {
                        let exit_code = int_result.value;
                        println!("✅ LLVM execution completed!");
                        println!("📊 Exit code: {}", exit_code);
                        process::exit(exit_code as i32);
                    } else {
                        println!("✅ LLVM execution completed (non-integer result)!");
                        println!("📊 Result: {}", result.to_string_box().value);
                    }
                },
                Err(e) => { eprintln!("❌ LLVM execution error: {}", e); process::exit(1); }
            }
        }
        #[cfg(not(feature = "llvm"))]
        {
            println!("🔧 Mock LLVM Backend Execution:");
            println!("   Build with --features llvm for real compilation.");
            if let Some(main_func) = module.functions.get("Main.main") {
                for (_bid, block) in &main_func.blocks {
                    for inst in &block.instructions {
                        match inst {
                            MirInstruction::Return { value: Some(_) } => { println!("✅ Mock exit code: 42"); process::exit(42); }
                            MirInstruction::Return { value: None } => { println!("✅ Mock exit code: 0"); process::exit(0); }
                            _ => {}
                        }
                    }
                }
            }
            println!("✅ Mock exit code: 0");
            process::exit(0);
        }
    }
}

fn emit_mir_json_for_harness(module: &nyash_rust::mir::MirModule, path: &std::path::Path) -> Result<(), String> {
    use nyash_rust::mir::{MirInstruction as I, BinaryOp as B, CompareOp as C};
    // Build JSON structure expected by python builder: { functions: [ { name, params, blocks: [ { id, instructions: [ ... ] } ] } ] }
    let mut funs = Vec::new();
    for (name, f) in &module.functions {
        let mut blocks = Vec::new();
        let mut ids: Vec<_> = f.blocks.keys().copied().collect();
        ids.sort();
        for bid in ids {
            if let Some(bb) = f.blocks.get(&bid) {
                let mut insts = Vec::new();
                // PHI first（オプション）
                for inst in &bb.instructions {
                        if let I::Phi { dst, inputs } = inst {
                        let incoming: Vec<_> = inputs.iter().map(|(b, v)| json!([v.as_u32(), b.as_u32()])).collect();
                        insts.push(json!({"op":"phi","dst": dst.as_u32(), "incoming": incoming}));
                        }
                }
                // Non-PHI
                for inst in &bb.instructions {
                    match inst {
                        I::Const { dst, value } => {
                            let (t, val) = match value {
                                nyash_rust::mir::ConstValue::Integer(i) => ("i64", json!(i)),
                                nyash_rust::mir::ConstValue::Float(fv) => ("f64", json!(fv)),
                                nyash_rust::mir::ConstValue::Bool(b) => ("i64", json!(if *b {1} else {0})),
                                nyash_rust::mir::ConstValue::String(s) => ("string", json!(s)),
                                nyash_rust::mir::ConstValue::Null => ("void", json!(0)),
                                nyash_rust::mir::ConstValue::Void => ("void", json!(0)),
                            };
                            insts.push(json!({"op":"const","dst": dst.as_u32(), "value": {"type": t, "value": val}}));
                        }
                        I::BinOp { dst, op, lhs, rhs } => {
                            let op_s = match op { B::Add=>"+",B::Sub=>"-",B::Mul=>"*",B::Div=>"/",B::Mod=>"%",B::BitAnd=>"&",B::BitOr=>"|",B::BitXor=>"^",B::Shl=>"<<",B::Shr=>">>",B::And=>"&",B::Or=>"|"};
                            insts.push(json!({"op":"binop","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()}));
                        }
                        I::Compare { dst, op, lhs, rhs } => {
                            let op_s = match op { C::Lt=>"<", C::Le=>"<=", C::Gt=>">", C::Ge=>">=", C::Eq=>"==", C::Ne=>"!=" };
                            insts.push(json!({"op":"compare","operation": op_s, "lhs": lhs.as_u32(), "rhs": rhs.as_u32(), "dst": dst.as_u32()}));
                        }
                        I::Call { dst, func, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"call","func": func.as_u32(), "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                        }
                        I::ExternCall { dst, iface_name, method_name, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            // Map known interfaces to NyRT symbols
                            let func_name = if iface_name == "env.console" {
                                format!("nyash.console.{}", method_name)
                            } else { format!("{}.{}", iface_name, method_name) };
                            insts.push(json!({"op":"externcall","func": func_name, "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                        }
                        I::BoxCall { dst, box_val, method, args, .. } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"boxcall","box": box_val.as_u32(), "method": method, "args": args_a, "dst": dst.map(|d| d.as_u32())}));
                        }
                        I::NewBox { dst, box_type, args } => {
                            let args_a: Vec<_> = args.iter().map(|v| json!(v.as_u32())).collect();
                            insts.push(json!({"op":"newbox","type": box_type, "args": args_a, "dst": dst.as_u32()}));
                        }
                        I::Branch { condition, then_bb, else_bb } => {
                            insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()}));
                        }
                        I::Jump { target } => {
                            insts.push(json!({"op":"jump","target": target.as_u32()}));
                        }
                        I::Return { value } => {
                            insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())}));
                        }
                        _ => { /* skip non-essential ops for initial harness */ }
                    }
                }
                // Terminator (if present)
                if let Some(term) = &bb.terminator {
                    match term {
                        I::Return { value } => insts.push(json!({"op":"ret","value": value.map(|v| v.as_u32())})),
                        I::Jump { target } => insts.push(json!({"op":"jump","target": target.as_u32()})),
                        I::Branch { condition, then_bb, else_bb } => insts.push(json!({"op":"branch","cond": condition.as_u32(), "then": then_bb.as_u32(), "else": else_bb.as_u32()})),
                        _ => {}
                    }
                }
                blocks.push(json!({"id": bid.as_u32(), "instructions": insts}));
            }
        }
        funs.push(json!({"name": name, "params": [], "blocks": blocks}));
    }
    let root = json!({"functions": funs});
    std::fs::write(path, serde_json::to_string_pretty(&root).unwrap()).map_err(|e| format!("write mir json: {}", e))
}
