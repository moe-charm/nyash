use super::super::NyashRunner;
use crate::runner::json_v0_bridge;
#[cfg(feature = "interpreter-legacy")]
use nyash_rust::{interpreter::NyashInterpreter, parser::NyashParser};
#[cfg(not(feature = "interpreter-legacy"))]
use nyash_rust::parser::NyashParser;
// Use the library crate's plugin init module rather than the bin crate root
use crate::cli_v;
use crate::runner::pipeline::{resolve_using_target, suggest_in_base};
use crate::runner::trace::cli_verbose;
use std::io::Read;
use std::process::Stdio;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{fs, process};

// (moved) suggest_in_base is now in runner/pipeline.rs

#[cfg(feature = "interpreter-legacy")]
impl NyashRunner {
    // legacy run_file_legacy removed (was commented out)

    /// Helper: run PyVM harness over a MIR module, returning the exit code
    #[cfg(feature = "pyvm-bridge")]
    fn run_pyvm_harness(
        &self,
        module: &nyash_rust::mir::MirModule,
        tag: &str,
    ) -> Result<i32, String> {
        super::common_util::pyvm::run_pyvm_harness(module, tag, self.config.as_groups().input.entry.as_deref())
    }
    #[cfg(not(feature = "pyvm-bridge"))]
    fn run_pyvm_harness(
        &self,
        _module: &nyash_rust::mir::MirModule,
        _tag: &str,
    ) -> Result<i32, String> {
        Err("pyvm-bridge feature disabled".to_string())
    }

    /// Helper: try external selfhost compiler EXE to parse Ny -> JSON v0 and return MIR module
    /// Returns Some(module) on success, None on failure (timeout/invalid output/missing exe)
    fn exe_try_parse_json_v0(
        &self,
        filename: &str,
        timeout_ms: u64,
    ) -> Option<nyash_rust::mir::MirModule> {
        super::common_util::selfhost_exe::exe_try_parse_json_v0(filename, timeout_ms)
    }

    /// Phase-15.3: Attempt Ny compiler pipeline (Ny -> JSON v0 via Ny program), then execute MIR
    pub(crate) fn try_run_ny_compiler_pipeline(&self, filename: &str) -> bool {
        // Delegate to centralized selfhost pipeline to avoid drift
        self.try_run_selfhost_pipeline(filename)
    }

    /// Execute Nyash file with interpreter (common helper)
    pub(crate) fn execute_nyash_file(&self, filename: &str) {
        let quiet_pipe = std::env::var("NYASH_JSON_ONLY").ok().as_deref() == Some("1");
        // Ensure runtime and plugins are initialized via unified helper (idempotent)
        let groups = self.config.as_groups();
        self.init_runtime_and_plugins(&groups);
        // Read the file
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        if crate::config::env::cli_verbose() && !quiet_pipe {
            println!("📝 File contents:\n{}", code);
            println!("\n🚀 Parsing and executing...\n");
        }

        // Using handling: AST-based prelude collection (unified resolver)
        let mut code_ref: &str = &code;
        let cleaned_code_owned;
        let prelude_asts: Vec<nyash_rust::ast::ASTNode>;

        if crate::config::env::enable_using() {
            let options = crate::runner::modes::common_util::resolve::UsingResolveOptions {
                allow_skip_ast_merge: quiet_pipe,  // Allow quiet pipes to skip AST merge
                collect_alias_names: false,
            };

            match crate::runner::modes::common_util::resolve::resolve_using_with_preludes(
                self, &code, filename, options
            ) {
                Ok(result) => {
                    cleaned_code_owned = result.cleaned_code;
                    code_ref = &cleaned_code_owned;
                    prelude_asts = result.prelude_asts;
                }
                Err(e) => {
                    eprintln!("❌ Pipeline error: `using` resolution error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            prelude_asts = Vec::new();
        }
        // Optional dev sugar: @name[:T] = expr → local name[:T] = expr (line-head only)
        let preexpanded_owned;
        {
            let s = crate::runner::modes::common_util::resolve::preexpand_at_local(code_ref);
            preexpanded_owned = s;
            code_ref = &preexpanded_owned;
        }

        // Parse the code with debug fuel limit
        let groups = self.config.as_groups();
        eprintln!(
            "🔍 DEBUG: Starting parse with fuel: {:?}...",
            groups.debug.debug_fuel
        );
        let main_ast =
            match NyashParser::parse_from_string_with_fuel(code_ref, groups.debug.debug_fuel) {
                Ok(ast) => {
                    eprintln!("🔍 DEBUG: Parse completed, AST created");
                    ast
                }
                Err(e) => {
                    eprintln!("❌ Parse error: {}", e);
                    process::exit(1);
                }
            };
        // When using AST prelude mode, combine prelude ASTs + main AST into one Program
        let ast = if !prelude_asts.is_empty() {
            crate::runner::modes::common_util::resolve::merge_prelude_asts_with_main(prelude_asts, &main_ast)
        } else { main_ast };

        // Optional: dump AST statement kinds for quick diagnostics
        if std::env::var("NYASH_AST_DUMP").ok().as_deref() == Some("1") {
            use nyash_rust::ast::ASTNode;
            eprintln!("[ast] dump start");
            if let ASTNode::Program { statements, .. } = &ast {
                for (i, st) in statements.iter().enumerate().take(50) {
                    let kind = match st {
                        ASTNode::BoxDeclaration {
                            is_static, name, ..
                        } => {
                            if *is_static {
                                format!("StaticBox({})", name)
                            } else {
                                format!("Box({})", name)
                            }
                        }
                        ASTNode::FunctionDeclaration { name, .. } => format!("FuncDecl({})", name),
                        ASTNode::FunctionCall { name, .. } => format!("FuncCall({})", name),
                        ASTNode::MethodCall { method, .. } => format!("MethodCall({})", method),
                        ASTNode::ScopeBox { .. } => "ScopeBox".to_string(),
                        ASTNode::ImportStatement { path, .. } => format!("Import({})", path),
                        ASTNode::UsingStatement { namespace_name, .. } => {
                            format!("Using({})", namespace_name)
                        }
                        _ => format!("{:?}", st),
                    };
                    eprintln!("[ast] {}: {}", i, kind);
                }
            }
            eprintln!("[ast] dump end");
        }

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
                        if crate::config::env::using_strict() {
                            eprintln!(
                                "❌ import: path not found: {} (from {})",
                                p.display(),
                                filename
                            );
                            process::exit(1);
                        } else if crate::config::env::cli_verbose()
                            || crate::config::env::import_trace()
                        {
                            eprintln!("[import] path not found (continuing): {}", p.display());
                        }
                    }
                    let key = if let Some(a) = alias {
                        a.clone()
                    } else {
                        std::path::Path::new(path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(path)
                            .to_string()
                    };
                    let value = if exists {
                        p.to_string_lossy().to_string()
                    } else {
                        path.clone()
                    };
                    let sb = nyash_rust::box_trait::StringBox::new(value);
                    nyash_rust::runtime::modules_registry::set(key, Box::new(sb));
                }
            }
        }

        if crate::config::env::cli_verbose() && !quiet_pipe {
            if crate::config::env::cli_verbose() {
                println!("✅ Parse successful!");
            }
        }

        // Execute the AST
        let mut interpreter = NyashInterpreter::new();
        eprintln!("🔍 DEBUG: Starting execution...");
        match interpreter.execute(ast) {
            Ok(result) => {
                if crate::config::env::cli_verbose() && !quiet_pipe {
                    if crate::config::env::cli_verbose() {
                        println!("✅ Execution completed successfully!");
                    }
                }
                // Normalize display via semantics: prefer numeric, then string, then fallback
                let disp = {
                    // Special-case: plugin IntegerBox → call .get to fetch numeric value
                    if let Some(p) = result
                        .as_any()
                        .downcast_ref::<nyash_rust::runtime::plugin_loader_v2::PluginBoxV2>(
                    ) {
                        if p.box_type == "IntegerBox" {
                            // Scope the lock strictly to this block
                            let fetched = {
                                let host = nyash_rust::runtime::get_global_plugin_host();
                                let res = if let Ok(ro) = host.read() {
                                    if let Ok(Some(vb)) = ro.invoke_instance_method(
                                        "IntegerBox",
                                        "get",
                                        p.instance_id(),
                                        &[],
                                    ) {
                                        if let Some(ib) =
                                            vb.as_any()
                                                .downcast_ref::<nyash_rust::box_trait::IntegerBox>()
                                        {
                                            Some(ib.value.to_string())
                                        } else {
                                            Some(vb.to_string_box().value)
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                res
                            };
                            if let Some(s) = fetched {
                                s
                            } else {
                                nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                    .map(|i| i.to_string())
                                    .or_else(|| {
                                        nyash_rust::runtime::semantics::coerce_to_string(
                                            result.as_ref(),
                                        )
                                    })
                                    .unwrap_or_else(|| result.to_string_box().value)
                            }
                        } else {
                            nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                                .map(|i| i.to_string())
                                .or_else(|| {
                                    nyash_rust::runtime::semantics::coerce_to_string(
                                        result.as_ref(),
                                    )
                                })
                                .unwrap_or_else(|| result.to_string_box().value)
                        }
                    } else {
                        nyash_rust::runtime::semantics::coerce_to_i64(result.as_ref())
                            .map(|i| i.to_string())
                            .or_else(|| {
                                nyash_rust::runtime::semantics::coerce_to_string(result.as_ref())
                            })
                            .unwrap_or_else(|| result.to_string_box().value)
                    }
                };
                if !quiet_pipe {
                    println!("Result: {}", disp);
                }
            }
            Err(e) => {
                eprintln!("❌ Runtime error:\n{}", e.detailed_message(Some(&code)));
                process::exit(1);
            }
        }
    }
}

#[cfg(not(feature = "interpreter-legacy"))]
impl NyashRunner {
    /// Interpreter backend is disabled in default builds. Use `--backend vm` or `--backend llvm`.
    pub(crate) fn execute_nyash_file(&self, _filename: &str) {
        eprintln!("❌ Interpreter backend (AST) is disabled. Build with --features interpreter-legacy to enable, or use --backend vm/llvm.");
        std::process::exit(1);
    }
}
