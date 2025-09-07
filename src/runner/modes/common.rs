use super::super::NyashRunner;
use crate::runner::json_v0_bridge;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter};
// Use the library crate's plugin init module rather than the bin crate root
use nyash_rust::runner_plugin_init;
use std::{fs, process};

// limited directory walk: add matching files ending with .nyash and given leaf name
fn suggest_in_base(base: &str, leaf: &str, out: &mut Vec<String>) {
    use std::fs;
    fn walk(dir: &std::path::Path, leaf: &str, out: &mut Vec<String>, depth: usize) {
        if depth == 0 || out.len() >= 5 { return; }
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let path = e.path();
                if path.is_dir() {
                    walk(&path, leaf, out, depth - 1);
                    if out.len() >= 5 { return; }
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "nyash" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem == leaf {
                                out.push(path.to_string_lossy().to_string());
                                if out.len() >= 5 { return; }
                            }
                        }
                    }
                }
            }
        }
    }
    let p = std::path::Path::new(base);
    walk(p, leaf, out, 4);
}

impl NyashRunner {
    /// File-mode dispatcher (thin wrapper around backend/mode selection)
    pub(crate) fn run_file(&self, filename: &str) {
        // Direct v0 bridge when requested via CLI/env
        let use_ny_parser = self.config.parser_ny || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
        if use_ny_parser {
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            match json_v0_bridge::parse_source_v0_to_module(&code) {
                Ok(module) => {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        println!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                    }
                    self.execute_mir_module(&module);
                    return;
                }
                Err(e) => { eprintln!("❌ Direct bridge parse error: {}", e); process::exit(1); }
            }
        }
        // AST dump mode
        if self.config.dump_ast {
            println!("🧠 Nyash AST Dump - Processing file: {}", filename);
            let code = match fs::read_to_string(filename) {
                Ok(content) => content,
                Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
            };
            let ast = match NyashParser::parse_from_string(&code) {
                Ok(ast) => ast,
                Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
            };
            println!("{:#?}", ast);
            return;
        }

        // MIR dump/verify
        if self.config.dump_mir || self.config.verify_mir {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                println!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
            }
            self.execute_mir_mode(filename);
            return;
        }

        // WASM / AOT (feature-gated)
        if self.config.compile_wasm {
            #[cfg(feature = "wasm-backend")]
            { self.execute_wasm_mode(filename); return; }
            #[cfg(not(feature = "wasm-backend"))]
            { eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend"); process::exit(1); }
        }
        if self.config.compile_native {
            #[cfg(feature = "cranelift-jit")]
            { self.execute_aot_mode(filename); return; }
            #[cfg(not(feature = "cranelift-jit"))]
            { eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit"); process::exit(1); }
        }

        // Backend selection
        match self.config.backend.as_str() {
            "mir" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🚀 Nyash MIR Interpreter - Executing file: {} 🚀", filename);
                }
                self.execute_mir_interpreter_mode(filename);
            }
            "vm" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
                }
                self.execute_vm_mode(filename);
            }
            "cranelift" => {
                #[cfg(feature = "cranelift-jit")]
                {
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        println!("⚙️  Nyash Cranelift JIT - Executing file: {}", filename);
                    }
                    self.execute_cranelift_mode(filename);
                }
                #[cfg(not(feature = "cranelift-jit"))]
                {
                    eprintln!("❌ Cranelift backend not available. Please rebuild with: cargo build --features cranelift-jit");
                    process::exit(1);
                }
            }
            "llvm" => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
                }
                self.execute_llvm_mode(filename);
            }
            _ => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                    println!("🦀 Nyash Rust Implementation - Executing file: {} 🦀", filename);
                    if let Some(fuel) = self.config.debug_fuel {
                        println!("🔥 Debug fuel limit: {} iterations", fuel);
                    } else {
                        println!("🔥 Debug fuel limit: unlimited");
                    }
                    println!("====================================================");
                }
                self.execute_nyash_file(filename);
            }
        }
    }

    /// Execute Nyash file with interpreter (common helper)
    pub(crate) fn execute_nyash_file(&self, filename: &str) {
        let quiet_pipe = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1");
        // Ensure plugin host and provider mappings are initialized (idempotent)
        if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
            // Call via lib crate to avoid referring to the bin crate root
            runner_plugin_init::init_bid_plugins();
        }
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") && !quiet_pipe {
            println!("📝 File contents:\n{}", code);
            println!("\n🚀 Parsing and executing...\n");
        }

        // Optional Phase-15: strip `using` lines (gate) for minimal acceptance
        let mut code_ref: &str = &code;
        let enable_using = std::env::var("NYASH_ENABLE_USING").ok().as_deref() == Some("1");
        let cleaned_code_owned;
        if enable_using {
            let mut out = String::with_capacity(code.len());
            let mut used_names: Vec<(String, Option<String>)> = Vec::new();
            for line in code.lines() {
                let t = line.trim_start();
                if t.starts_with("using ") {
                    // Skip `using ns` or `using ns as alias` lines (MVP)
                    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[using] stripped line: {}", line);
                    }
                    // Parse namespace or path and optional alias
                    let rest0 = t.strip_prefix("using ").unwrap().trim();
                    // allow trailing semicolon
                    let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                    // Split alias
                    let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                        (rest0[..pos].trim().to_string(), Some(rest0[pos+4..].trim().to_string()))
                    } else { (rest0.to_string(), None) };
                    // If quoted or looks like relative/absolute path, treat as path; else as namespace
                    let is_path = target.starts_with('"') || target.starts_with("./") || target.starts_with('/') || target.ends_with(".nyash");
                    if is_path {
                        let mut path = target.trim_matches('"').to_string();
                        // existence check and strict handling
                        let missing = !std::path::Path::new(&path).exists();
                        if missing {
                            if std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1") {
                                eprintln!("❌ using: path not found: {}", path);
                                std::process::exit(1);
                            } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                                eprintln!("[using] path not found (continuing): {}", path);
                            }
                        }
                        // choose alias or derive from filename stem
                        let name = alias.clone().unwrap_or_else(|| {
                            std::path::Path::new(&path)
                                .file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string()
                        });
                        // register alias only (path-backed)
                        used_names.push((name, Some(path)));
                    } else {
                        used_names.push((target, alias));
                    }
                    continue;
                }
                out.push_str(line);
                out.push('\n');
            }
            cleaned_code_owned = out;
            code_ref = &cleaned_code_owned;

            // Register modules into minimal registry with best-effort path resolution
            for (ns_or_alias, alias_or_path) in used_names {
                // alias_or_path Some(path) means this entry was a direct path using
                if let Some(path) = alias_or_path {
                    let sb = crate::box_trait::StringBox::new(path);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                } else {
                    let rel = format!("apps/{}.nyash", ns_or_alias.replace('.', "/"));
                    let exists = std::path::Path::new(&rel).exists();
                    if !exists && std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                        eprintln!("[using] unresolved namespace '{}'; tried '{}'. Hint: add @module {}={} or --module {}={}", ns_or_alias, rel, ns_or_alias, rel, ns_or_alias, rel);
                        // naive candidates by suffix within common bases
                        let leaf = ns_or_alias.split('.').last().unwrap_or(&ns_or_alias);
                        let mut cands: Vec<String> = Vec::new();
                        suggest_in_base("apps", leaf, &mut cands);
                        if cands.len() < 5 { suggest_in_base("lib", leaf, &mut cands); }
                        if cands.len() < 5 { suggest_in_base(".", leaf, &mut cands); }
                        if !cands.is_empty() {
                            eprintln!("[using] candidates: {}", cands.join(", "));
                        }
                    }
                    let path_or_ns = if exists { rel } else { ns_or_alias.clone() };
                    let sb = crate::box_trait::StringBox::new(path_or_ns);
                    crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                }
            }
        }

        // Parse the code with debug fuel limit
        eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", self.config.debug_fuel);
        let ast = match NyashParser::parse_from_string_with_fuel(code_ref, self.config.debug_fuel) {
            Ok(ast) => { eprintln!("🔍 DEBUG: Parse completed, AST created"); ast },
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };

        // Stage-0: import loader (top-level only) — resolve path and register in modules registry
        if let nyash_rust::ast::ASTNode::Program { statements, .. } = &ast {
            for st in statements {
                if let nyash_rust::ast::ASTNode::ImportStatement { path, alias, .. } = st {
                    // resolve path relative to current file if not absolute
                    let mut p = std::path::PathBuf::from(path);
                    if p.is_relative() {
                        if let Some(dir) = std::path::Path::new(filename).parent() {
                            p = dir.join(&p);
                        }
                    }
                    let exists = p.exists();
                    if !exists {
                        if std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1") {
                            eprintln!("❌ import: path not found: {} (from {})", p.display(), filename);
                            process::exit(1);
                        } else if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") || std::env::var("NYASH_IMPORT_TRACE").ok().as_deref() == Some("1") {
                            eprintln!("[import] path not found (continuing): {}", p.display());
                        }
                    }
                    let key = if let Some(a) = alias { a.clone() } else {
                        std::path::Path::new(path)
                            .file_stem().and_then(|s| s.to_str())
                            .unwrap_or(path)
                            .to_string()
                    };
                    let value = if exists { p.to_string_lossy().to_string() } else { path.clone() };
                    let sb = nyash_rust::box_trait::StringBox::new(value);
                    nyash_rust::runtime::modules_registry::set(key, Box::new(sb));
                }
            }
        }

        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") && !quiet_pipe {
            println!("✅ Parse successful!");
        }

        // Execute the AST
        let mut interpreter = NyashInterpreter::new();
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") && !quiet_pipe {
                    println!("✅ Execution completed successfully!");
                }
                // Normalize display via semantics: prefer numeric, then string, then fallback
                let disp = {
                    // Special-case: plugin IntegerBox → call .get to fetch numeric value
                    if let Some(p) = result.as_any().downcast_ref::<nyash_rust::runtime::plugin_loader_v2::PluginBoxV2>() {
                        if p.box_type == "IntegerBox" {
                            // Scope the lock strictly to this block
                            let fetched = {
                                let host = nyash_rust::runtime::get_global_plugin_host();
                                let res = if let Ok(ro) = host.read() {
                                    if let Ok(Some(vb)) = ro.invoke_instance_method("IntegerBox", "get", p.instance_id(), &[]) {
                                        if let Some(ib) = vb.as_any().downcast_ref::<nyash_rust::box_trait::IntegerBox>() {
                                            Some(ib.value.to_string())
                                        } else {
                                            Some(vb.to_string_box().value)
                                        }
                                    } else { None }
                                } else { None };
                                res
                            };
                            if let Some(s) = fetched { s } else {
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                    .map(|i| i.to_string())
                                    .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                    .unwrap_or_else(|| result.to_string_box().value)
                            }
                        } else {
                            nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                .map(|i| i.to_string())
                                .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                                .unwrap_or_else(|| result.to_string_box().value)
                        }
                    } else {
                        nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            .map(|i| i.to_string())
                            .or_else(|| nyash_rust::runtime::semantics::coerce_to_string(result.as_ref()))
                            .unwrap_or_else(|| result.to_string_box().value)
                    }
                };
                if !quiet_pipe { println!("Result: {}", disp); }
            },
            Err(e) => {
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }
}
