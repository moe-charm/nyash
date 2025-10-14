/*!
 * Runner dispatch helpers — execute MIR module and report result
 */

use super::*;
use crate::runner::json_v0_bridge;
use nyash_rust::parser::NyashParser;
use std::{fs, process};
#[cfg(feature = "parser-c-abi")]
use std::ffi::CString;
#[cfg(feature = "parser-c-abi")]
use std::os::raw::c_char;

#[cfg(feature = "parser-c-abi")]
#[repr(C)]
struct HakoParseResult {
    abi_version: u32,
    struct_size: u32,
    success: u32,
    stmt_count: u32,
    kind: *const c_char,
    error_msg: *const c_char,
}
#[cfg(feature = "parser-c-abi")]
#[allow(non_camel_case_types)]
#[repr(i32)]
enum HakoParseMode { RUST=0, HAKO=1, BOTH=2 }
#[cfg(feature = "parser-c-abi")]
extern "C" {
    fn parse_source_dual(src: *const c_char, mode: HakoParseMode) -> *mut HakoParseResult;
    fn free_parse_result(r: *mut HakoParseResult);
}
#[cfg(feature = "parser-c-abi")]
fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() { return String::new(); }
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

/// Thin file dispatcher: select backend and delegate to mode executors
pub(crate) fn execute_file_with_backend(runner: &NyashRunner, filename: &str) {
    crate::runner_plugin_init::init_bid_plugins();
    // Deprecation: warn on legacy .nyash extension (prefer .hako)
    if filename.ends_with(".nyash") {
        eprintln!("[deprecate] The .nyash extension is deprecated; please use .hako (see docs/guides/naming-and-extensions.md).");
    }
    // Selfhost pipeline (Ny -> JSON v0) behind env gate
    if std::env::var("NYASH_USE_NY_COMPILER").ok().as_deref() == Some("1") {
        if runner.try_run_selfhost_pipeline(filename) {
            return;
        } else {
            crate::cli_v!("[ny-compiler] fallback to default path (MVP unavailable for this input)");
        }
    }

    // Optional: Stage-4 header parity check via C-ABI harness (no-op unless requested)
    #[cfg(feature = "parser-c-abi")]
    {
        if let Some(mode_s) = std::env::var("SMOKES_PARSER_MODE").ok() {
            let mode = match mode_s.as_str() { "both" => Some(HakoParseMode::BOTH), "hako" => Some(HakoParseMode::HAKO), _ => None };
            if let Some(m) = mode {
                if let Ok(code) = fs::read_to_string(filename) {
                    if let Ok(ccode) = CString::new(code) {
                        unsafe {
                            let res = parse_source_dual(ccode.as_ptr(), m);
                            if !res.is_null() {
                                let r = &*res;
                                if r.success == 0 {
                                    eprintln!("[parser-header] parity check failed: {}", cstr_to_string(r.error_msg));
                                }
                                free_parse_result(res);
                            }
                        }
                    }
                }
            }
        }
    }

    // Direct v0 bridge (raw JSON) when requested via env (dev-only)
    if std::env::var("NYASH_JSON_V0_DIRECT").ok().as_deref() == Some("1") {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        match json_v0_bridge::parse_json_v0_to_module(&code) {
            Ok(module) => {
                crate::cli_v!("🚀 Nyash MIR Interpreter - (json_v0 direct) Executing file: {} 🚀", filename);
                runner.execute_mir_module(&module);
                return;
            }
            Err(e) => {
                eprintln!("❌ Direct JSON v0 parse error: {}", e);
                process::exit(1);
            }
        }
    }

    // Direct v0 bridge when requested via CLI/env (source v0 mini-grammar)
    let groups = runner.config.as_groups();
    let use_ny_parser = groups.parser.parser_ny
        || std::env::var("NYASH_USE_NY_PARSER").ok().as_deref() == Some("1");
    if use_ny_parser {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        match json_v0_bridge::parse_source_v0_to_module(&code) {
            Ok(module) => {
                crate::cli_v!("🚀 Nyash MIR Interpreter - (parser=ny) Executing file: {} 🚀", filename);
                runner.execute_mir_module(&module);
                return;
            }
            Err(e) => {
                eprintln!("❌ Direct bridge parse error: {}", e);
                process::exit(1);
            }
        }
    }

    // AST dump mode
    if groups.debug.dump_ast {
        println!("🧠 Nyash AST Dump - Processing file: {}", filename);
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("❌ Error reading file {}: {}", filename, e);
                process::exit(1);
            }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("❌ Parse error: {}", e);
                process::exit(1);
            }
        };
        // Optional macro expansion dump (no-op expansion for now)
        let ast2 = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, true);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else {
            ast.clone()
        };
        println!("{:#?}", ast2);
        return;
    }

    // Dump expanded AST as JSON v0 and exit
    if runner.config.dump_expanded_ast_json {
        let code = match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => { eprintln!("❌ Error reading file {}: {}", filename, e); process::exit(1); }
        };
        let ast = match NyashParser::parse_from_string(&code) {
            Ok(ast) => ast,
            Err(e) => { eprintln!("❌ Parse error: {}", e); process::exit(1); }
        };
        let expanded = if crate::r#macro::enabled() {
            let a = crate::r#macro::maybe_expand_and_dump(&ast, false);
            crate::runner::modes::macro_child::normalize_core_pass(&a)
        } else { ast };
        let j = crate::r#macro::ast_json::ast_to_json(&expanded);
        println!("{}", j.to_string());
        return;
    }

    // MIR dump/verify
    if groups.debug.dump_mir || groups.debug.verify_mir {
        crate::cli_v!("🚀 Nyash MIR Compiler - Processing file: {} 🚀", filename);
        runner.execute_mir_mode(filename);
        return;
    }

    // WASM / AOT (feature-gated)
    if groups.compile_wasm {
        #[cfg(feature = "wasm-backend")]
        {
            super::modes::wasm::execute_wasm_mode(runner, filename);
            return;
        }
        #[cfg(not(feature = "wasm-backend"))]
        {
            eprintln!("❌ WASM backend not available. Please rebuild with: cargo build --features wasm-backend");
            process::exit(1);
        }
    }
    if groups.compile_native {
        #[cfg(feature = "cranelift-jit")]
        {
            runner.execute_aot_mode(filename);
            return;
        }
        #[cfg(not(feature = "cranelift-jit"))]
        {
            eprintln!("❌ Native AOT compilation requires Cranelift. Please rebuild: cargo build --features cranelift-jit");
            process::exit(1);
        }
    }

    // Backend selection (normalize; fallback unknown/empty to "vm")
    let mut backend_norm = groups.backend.backend.trim().to_string();
    if backend_norm.is_empty() || backend_norm == "." { backend_norm = "vm".to_string(); }
    match backend_norm.as_str() {
        "mir" => {
            let eng = std::env::var("HAKO_NYVM_ENGINE").ok().unwrap_or_else(|| "hakorune".to_string());
            if eng.eq_ignore_ascii_case("mini") {
                crate::cli_v!("🚀 Nyash Mini‑VM (MIR Interpreter) - Executing file: {} 🚀", filename);
                runner.execute_mir_mode(filename);
            } else {
                crate::cli_v!("🚀 Hakorune VM (Ny) bridge - Executing file: {} 🚀", filename);
                if let Err(e) = runner.execute_hakorune_vm_bridge(filename) {
                    eprintln!("❌ nyvm(hakorune) bridge error: {}", e);
                    std::process::exit(1);
                }
            }
        },
        "vm" => {
            crate::cli_v!("🚀 Nyash VM Backend - Executing file: {} 🚀", filename);
            // Unified VM engine (Phase‑A): default=fallback
            runner.execute_vm_engine(filename);
        }
        #[cfg(feature = "cranelift-jit")]
        "jit-direct" => {
            crate::cli_v!("⚡ Nyash JIT-Direct Backend - Executing file: {} ⚡", filename);
            #[cfg(feature = "cranelift-jit")]
            {
                // Use independent JIT-direct runner method (no VM execute loop)
                runner.run_file_jit_direct(filename);
            }
            #[cfg(not(feature = "cranelift-jit"))]
            {
                eprintln!("❌ Cranelift backend not available. Please rebuild with: cargo build --features cranelift-jit");
                process::exit(1);
            }
        }
        "llvm" => {
            crate::cli_v!("⚡ Nyash LLVM Backend - Executing file: {} ⚡", filename);
            runner.execute_llvm_mode(filename);
        }
        other => {
            eprintln!("[warn] Unknown backend: {} → falling back to 'vm'", other);
            runner.execute_vm_engine(filename);
        }
    }
}

impl NyashRunner {
    /// Execute VM engine from in-memory source (no wrapper temp files)
    pub(crate) fn execute_vm_engine_from_source(&self, source_inline: &str, pseudo_filename: &str) {
        use crate::runner::vm_pipeline;
        use std::process;

        let result = (|| {
            // Stage 1: (skipped) load from file → we already have inline source
            let source = source_inline.to_string();

            // Stage 2: Resolve `using` and preludes based on pseudo filename context
            let (source, preludes, aliases) =
                vm_pipeline::resolve_preludes_and_aliases(self, &source, pseudo_filename)?;

            // Stage 3: Parse main source and merge with preludes
            let ast = vm_pipeline::parse_and_merge_ast(&source, preludes)?;

            // Stage 4: Apply macros and desugar aliases
            let ast = vm_pipeline::process_ast_macros_and_aliases(ast, &aliases)?;

            // Stage 5: Register user-defined boxes from the AST
            vm_pipeline::register_user_boxes_from_ast(&ast);

            // Stage 6: Compile to MIR and execute
            vm_pipeline::compile_and_execute_mir(ast, self.config.no_optimize)
        })();

        if let Err(e) = result {
            eprintln!("❌ Pipeline error: {}", e);
            process::exit(1);
        }
    }
    /// Compile Nyash file to MIR and execute via selected VM engine (unified entry)
    pub(crate) fn execute_vm_engine(&self, filename: &str) {
        use crate::runner::vm_pipeline;
        use std::process;

        let result = (|| {
            // Stage 1: Load and preprocess source
            let source = vm_pipeline::load_and_preprocess_source(filename)?;

            // Stage 2: Resolve `using` and preludes
            let (source, preludes, aliases) =
                vm_pipeline::resolve_preludes_and_aliases(self, &source, filename)?;

            // Stage 3: Parse main source and merge with preludes
            let ast = vm_pipeline::parse_and_merge_ast(&source, preludes)?;

            // Stage 4: Apply macros and desugar aliases
            let ast = vm_pipeline::process_ast_macros_and_aliases(ast, &aliases)?;

            // Stage 5: Register user-defined boxes from the AST
            vm_pipeline::register_user_boxes_from_ast(&ast);

            // Stage 6: Compile to MIR and execute
            vm_pipeline::compile_and_execute_mir(ast, self.config.no_optimize)
        })();

        if let Err(e) = result {
            eprintln!("❌ Pipeline error: {}", e);
            process::exit(1);
        }
    }
    pub(crate) fn execute_mir_module(&self, module: &crate::mir::MirModule) {
        // If CLI requested MIR JSON emit, write to file and exit immediately.
        let groups = self.config.as_groups();
        if let Some(path) = groups.emit.emit_mir_json.as_ref() {
            let p = std::path::Path::new(path);
            if let Err(e) = crate::runner::mir_json_emit::emit_mir_json_for_harness_bin(module, p) {
                eprintln!("❌ MIR JSON emit error: {}", e);
                std::process::exit(1);
            }
            println!("MIR JSON written: {}", p.display());
            std::process::exit(0);
        }
        // If CLI requested EXE emit, generate JSON then invoke ny-llvmc to link NyRT and exit.
        if let Some(exe_out) = groups.emit.emit_exe.as_ref() {
            if let Err(e) = crate::runner::modes::common_util::exec::ny_llvmc_emit_exe_bin(
                module,
                exe_out,
                groups.emit.emit_exe_nyrt.as_deref(),
                groups.emit.emit_exe_libs.as_deref(),
            ) {
                eprintln!("❌ {}", e);
                std::process::exit(1);
            }
            println!("EXE written: {}", exe_out);
            std::process::exit(0);
        }
        use crate::backend::MirInterpreter;
        use crate::box_trait::{BoolBox, IntegerBox, StringBox};
        #[cfg(feature = "legacy-boxes")]
        use crate::boxes::FloatBox;
        use crate::mir::MirType;

        let mut interp = MirInterpreter::new();
        match interp.execute_module(module) {
            Ok(result) => {
                println!("✅ MIR interpreter execution completed!");
                if let Some(func) = module.functions.get("main") {
                    let (ety, sval) = match &func.signature.return_type {
                        MirType::Float => {
                            #[cfg(feature = "legacy-boxes")]
                            {
                                if let Some(fb) = result.as_any().downcast_ref::<FloatBox>() {
                                    ("Float", format!("{}", fb.value))
                                } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                    ("Float", format!("{}", ib.value as f64))
                                } else {
                                    ("Float", result.to_string_box().value)
                                }
                            }
                            #[cfg(not(feature = "legacy-boxes"))]
                            {
                                if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                    ("Float", format!("{}", ib.value as f64))
                                } else {
                                    ("Float", result.to_string_box().value)
                                }
                            }
                        }
                        MirType::Integer => {
                            if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Integer", ib.value.to_string())
                            } else {
                                ("Integer", result.to_string_box().value)
                            }
                        }
                        MirType::Bool => {
                            if let Some(bb) = result.as_any().downcast_ref::<BoolBox>() {
                                ("Bool", bb.value.to_string())
                            } else if let Some(ib) = result.as_any().downcast_ref::<IntegerBox>() {
                                ("Bool", (ib.value != 0).to_string())
                            } else {
                                ("Bool", result.to_string_box().value)
                            }
                        }
                        MirType::String => {
                            if let Some(sb) = result.as_any().downcast_ref::<StringBox>() {
                                ("String", sb.value.clone())
                            } else {
                                ("String", result.to_string_box().value)
                            }
                        }
                        _ => (result.type_name(), result.to_string_box().value),
                    };
                    println!("ResultType(MIR): {}", ety);
                    println!("Result: {}", sval);
                } else {
                    println!("Result: {:?}", result);
                }
            }
            Err(e) => {
                eprintln!("❌ MIR interpreter error: {}", e);
                std::process::exit(1);
            }
        }
    }
}


impl NyashRunner {
    pub(crate) fn execute_hakorune_vm_bridge(&self, filename: &str) -> Result<(), String> {
        use crate::runner::vm_pipeline;
        let (module_json, _ret_ty) = {
            let source = vm_pipeline::load_and_preprocess_source(filename).map_err(|e| e.to_string())?;
            let (source, preludes, aliases) = vm_pipeline::resolve_preludes_and_aliases(self, &source, filename).map_err(|e| e.to_string())?;
            let ast = vm_pipeline::parse_and_merge_ast(&source, preludes).map_err(|e| e.to_string())?;
            let ast = vm_pipeline::process_ast_macros_and_aliases(ast, &aliases).map_err(|e| e.to_string())?;
            let mut mirc = crate::mir::MirCompiler::with_options(!self.config.no_optimize);
            let cr = mirc.compile(ast).map_err(|e| format!("MIR compile error: {}", e))?;
            let tf = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
            let p = tf.path().to_path_buf();
            crate::runner::mir_json_emit::emit_mir_json_for_harness(&cr.module, &p).map_err(|e| e.to_string())?;
            let json = std::fs::read_to_string(&p).map_err(|e| e.to_string())?;
            let json_for_bridge = match serde_json::from_str::<serde_json::Value>(&json) {
                Ok(root) => {
                    if let Some(funcs) = root.get("functions").and_then(|v| v.as_array()) {
                        if let Some(first) = funcs.get(0) {
                            serde_json::to_string(first).unwrap_or(json.clone())
                        } else { json.clone() }
                    } else { json.clone() }
                }
                Err(_) => json.clone(),
            };
            (json_for_bridge, cr.module.functions.get("main").map(|f| f.signature.return_type.clone()))
        };
        
        // Escape JSON via serde_json to embed safely in Ny source
        let esc = serde_json::to_string(&module_json).map_err(|e| e.to_string())?;
        let wrapper = format!(
            "using \"selfhost/hakorune-vm/hakorune_vm_core.hako\" as HakoruneVmCore
static box Main {{
  main() {{
    local j = {}
    return HakoruneVmCore.run(j)
  }}
}}
",
            esc
        );

        self.execute_vm_engine_from_source(&wrapper, "<nyvm-bridge>");
        Ok(())
    }
}
