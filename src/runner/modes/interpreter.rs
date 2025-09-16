use crate::interpreter::NyashInterpreter;
use crate::parser::NyashParser;
use crate::runner::pipeline::{resolve_using_target, UsingContext};
use crate::runner_plugin_init;
use std::{fs, process};

/// Execute Nyash file with interpreter
pub fn execute_nyash_file(filename: &str, debug_fuel: Option<usize>) {
    // Read the file
    let code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file {}: {}", filename, e);
            process::exit(1);
        }
    };
    // Initialize plugin host and mappings (idempotent)
    if std::env::var("NYASH_DISABLE_PLUGINS").ok().as_deref() != Some("1") {
        runner_plugin_init::init_bid_plugins();
        crate::runner::box_index::refresh_box_index();
    }

    println!("📝 File contents:\n{}", code);
    println!("\n🚀 Parsing and executing...\n");

    // Test: immediate file creation (use relative path to avoid sandbox issues)
    std::fs::create_dir_all("development/debug_hang_issue").ok();
    std::fs::write("development/debug_hang_issue/test.txt", "START").ok();

    // Optional: using pre-processing (strip lines and register modules)
    let enable_using = std::env::var("NYASH_ENABLE_USING").ok().as_deref() == Some("1");
    let mut code_ref: std::borrow::Cow<'_, str> = std::borrow::Cow::Borrowed(&code);
    if enable_using {
        let mut out = String::with_capacity(code.len());
        let mut used_names: Vec<(String, Option<String>)> = Vec::new();
        for line in code.lines() {
            let t = line.trim_start();
            if t.starts_with("using ") {
                let rest0 = t.strip_prefix("using ").unwrap().trim();
                let rest0 = rest0.strip_suffix(';').unwrap_or(rest0).trim();
                let (target, alias) = if let Some(pos) = rest0.find(" as ") {
                    (
                        rest0[..pos].trim().to_string(),
                        Some(rest0[pos + 4..].trim().to_string()),
                    )
                } else {
                    (rest0.to_string(), None)
                };
                let is_path = target.starts_with('"')
                    || target.starts_with("./")
                    || target.starts_with('/')
                    || target.ends_with(".nyash");
                if is_path {
                    let path = target.trim_matches('"').to_string();
                    let name = alias.clone().unwrap_or_else(|| {
                        std::path::Path::new(&path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("module")
                            .to_string()
                    });
                    used_names.push((name, Some(path)));
                } else {
                    used_names.push((target, alias));
                }
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        // Resolve and register
        let using_ctx = UsingContext {
            using_paths: vec!["apps".into(), "lib".into(), ".".into()],
            pending_modules: vec![],
            aliases: std::collections::HashMap::new(),
        };
        let strict = std::env::var("NYASH_USING_STRICT").ok().as_deref() == Some("1");
        let verbose = std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1");
        let ctx_dir = std::path::Path::new(filename).parent();
        for (ns_or_alias, alias_or_path) in used_names {
            if let Some(path) = alias_or_path {
                let sb = crate::box_trait::StringBox::new(path);
                crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
            } else {
                match resolve_using_target(
                    &ns_or_alias,
                    false,
                    &using_ctx.pending_modules,
                    &using_ctx.using_paths,
                    &using_ctx.aliases,
                    ctx_dir,
                    strict,
                    verbose,
                ) {
                    Ok(value) => {
                        let sb = crate::box_trait::StringBox::new(value);
                        crate::runtime::modules_registry::set(ns_or_alias, Box::new(sb));
                    }
                    Err(e) => {
                        eprintln!("❌ using: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        code_ref = std::borrow::Cow::Owned(out);
    }

    // Parse the code with debug fuel limit
    eprintln!("🔍 DEBUG: Starting parse with fuel: {:?}...", debug_fuel);
    let ast = match NyashParser::parse_from_string_with_fuel(&*code_ref, debug_fuel) {
        Ok(ast) => {
            eprintln!("🔍 DEBUG: Parse completed, AST created");
            ast
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            process::exit(1);
        }
    };

    eprintln!("🔍 DEBUG: About to print parse success message...");
    println!("✅ Parse successful!");
    eprintln!("🔍 DEBUG: Parse success message printed");

    // Debug log file write
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("development/debug_hang_issue/debug_trace.log")
    {
        use std::io::Write;
        let _ = writeln!(file, "=== MAIN: Parse successful ===");
        let _ = file.flush();
    }

    eprintln!("🔍 DEBUG: Creating interpreter...");

    // Execute the AST
    let mut interpreter = NyashInterpreter::new();
    eprintln!("🔍 DEBUG: Starting execution...");
    match interpreter.execute(ast) {
        Ok(result) => {
            println!("✅ Execution completed successfully!");
            println!("Result: {}", result.to_string_box().value);
            // Structured concurrency: best-effort join of spawned tasks at program end
            let join_ms: u64 = std::env::var("NYASH_JOIN_ALL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2000);
            crate::runtime::global_hooks::join_all_registered_futures(join_ms);
        }
        Err(e) => {
            // Use enhanced error reporting with source context
            eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
            process::exit(1);
        }
    }
}
