/*!
 * CLI Argument Parsing Module - Nyash Command Line Interface
 *
 * This module handles all command-line argument parsing using clap,
 * separating CLI concerns from the main execution logic.
 */

use clap::{Arg, ArgMatches, Command};
use serde_json;

/// Command-line configuration structure
#[derive(Debug, Clone)]
pub struct CliConfig {
    // File input (Nyash source)
    pub file: Option<String>,
    pub debug_fuel: Option<usize>,
    pub dump_ast: bool,
    pub dump_mir: bool,
    pub verify_mir: bool,
    pub mir_verbose: bool,
    pub mir_verbose_effects: bool,
    pub no_optimize: bool,
    pub backend: String,
    pub compile_wasm: bool,
    pub compile_native: bool,
    pub output_file: Option<String>,
    pub benchmark: bool,
    pub iterations: u32,
    pub vm_stats: bool,
    pub vm_stats_json: bool,
    // JIT controls
    pub jit_exec: bool,
    pub jit_stats: bool,
    pub jit_stats_json: bool,
    pub jit_dump: bool,
    pub jit_events: bool,
    pub jit_events_compile: bool,
    pub jit_events_runtime: bool,
    pub jit_events_path: Option<String>,
    pub jit_threshold: Option<u32>,
    pub jit_phi_min: bool,
    pub jit_hostcall: bool,
    pub jit_handle_debug: bool,
    pub jit_native_f64: bool,
    pub jit_native_bool: bool,
    pub jit_only: bool,
    pub jit_direct: bool,
    // DOT emit helper
    pub emit_cfg: Option<String>,
    // Verbose CLI
    pub cli_verbose: bool,
    // Tasks
    pub run_task: Option<String>,
    // Ny script plugins enumeration (opt-in)
    pub load_ny_plugins: bool,
    // Parser choice: 'ny' (direct v0 bridge) when true, otherwise default rust
    pub parser_ny: bool,
    // Phase-15: JSON IR v0 bridge
    pub ny_parser_pipe: bool,
    pub json_file: Option<String>,
    // GC mode (dev; forwarded to env as NYASH_GC_MODE)
    pub gc_mode: Option<String>,
    // Build system (MVP)
    pub build_path: Option<String>,
    pub build_app: Option<String>,
    pub build_out: Option<String>,
    pub build_aot: Option<String>,
    pub build_profile: Option<String>,
    pub build_target: Option<String>,
    // Using (CLI)
    pub cli_usings: Vec<String>,
    // Emit MIR JSON to a file and exit (bridge mode)
    pub emit_mir_json: Option<String>,
    // Emit native executable via ny-llvmc (crate) and exit
    pub emit_exe: Option<String>,
    pub emit_exe_nyrt: Option<String>,
    pub emit_exe_libs: Option<String>,
    // Macro child (sandbox) mode
    pub macro_expand_child: Option<String>,
    // Dump expanded AST as JSON and exit
    pub dump_expanded_ast_json: bool,
}

/// Grouped views (Phase 1: non-breaking). These structs provide a categorized
/// lens over the flat CliConfig without changing public fields.
#[derive(Debug, Clone)]
pub struct InputConfig {
    pub file: Option<String>,
    pub cli_usings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub debug_fuel: Option<usize>,
    pub dump_ast: bool,
    pub dump_mir: bool,
    pub verify_mir: bool,
    pub mir_verbose: bool,
    pub mir_verbose_effects: bool,
    pub cli_verbose: bool,
}

#[derive(Debug, Clone)]
pub struct BackendConfig {
    pub backend: String,
    // VM
    pub vm_stats: bool,
    pub vm_stats_json: bool,
    // JIT
    pub jit: JitConfig,
}

#[derive(Debug, Clone)]
pub struct JitConfig {
    pub exec: bool,
    pub stats: bool,
    pub stats_json: bool,
    pub dump: bool,
    pub events: bool,
    pub events_compile: bool,
    pub events_runtime: bool,
    pub events_path: Option<String>,
    pub threshold: Option<u32>,
    pub phi_min: bool,
    pub hostcall: bool,
    pub handle_debug: bool,
    pub native_f64: bool,
    pub native_bool: bool,
    pub only: bool,
    pub direct: bool,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub path: Option<String>,
    pub app: Option<String>,
    pub out: Option<String>,
    pub aot: Option<String>,
    pub profile: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmitConfig {
    pub emit_cfg: Option<String>,
    pub emit_mir_json: Option<String>,
    pub emit_exe: Option<String>,
    pub emit_exe_nyrt: Option<String>,
    pub emit_exe_libs: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParserPipeConfig {
    pub parser_ny: bool,
    pub ny_parser_pipe: bool,
    pub json_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CliGroups {
    pub input: InputConfig,
    pub debug: DebugConfig,
    pub backend: BackendConfig,
    pub build: BuildConfig,
    pub emit: EmitConfig,
    pub parser: ParserPipeConfig,
    pub gc_mode: Option<String>,
    pub compile_wasm: bool,
    pub compile_native: bool,
    pub output_file: Option<String>,
    pub benchmark: bool,
    pub iterations: u32,
    pub run_task: Option<String>,
    pub load_ny_plugins: bool,
}

impl CliConfig {
    /// Parse command-line arguments and return configuration
    pub fn parse() -> Self {
        // Pre-process raw argv to capture trailing script args after '--'
        let argv: Vec<String> = std::env::args().collect();
        if let Some(pos) = argv.iter().position(|s| s == "--") {
            // Everything after '--' is script args
            let script_args: Vec<String> = argv.iter().skip(pos + 1).cloned().collect();
            if !script_args.is_empty() {
                if let Ok(json) = serde_json::to_string(&script_args) {
                    std::env::set_var("NYASH_SCRIPT_ARGS_JSON", json);
                }
            }
            // Only parse CLI args up to '--'
            let matches = Self::build_command()
                .try_get_matches_from(&argv[..pos])
                .unwrap_or_else(|e| e.exit());
            Self::from_matches(&matches)
        } else {
            let matches = Self::build_command().get_matches();
            Self::from_matches(&matches)
        }
    }

    /// Non-breaking grouped view for downstream code to gradually adopt.
    pub fn as_groups(&self) -> CliGroups {
        CliGroups {
            input: InputConfig { file: self.file.clone(), cli_usings: self.cli_usings.clone() },
            debug: DebugConfig {
                debug_fuel: self.debug_fuel,
                dump_ast: self.dump_ast,
                dump_mir: self.dump_mir,
                verify_mir: self.verify_mir,
                mir_verbose: self.mir_verbose,
                mir_verbose_effects: self.mir_verbose_effects,
                cli_verbose: self.cli_verbose,
            },
            backend: BackendConfig {
                backend: self.backend.clone(),
                vm_stats: self.vm_stats,
                vm_stats_json: self.vm_stats_json,
                jit: JitConfig {
                    exec: self.jit_exec,
                    stats: self.jit_stats,
                    stats_json: self.jit_stats_json,
                    dump: self.jit_dump,
                    events: self.jit_events,
                    events_compile: self.jit_events_compile,
                    events_runtime: self.jit_events_runtime,
                    events_path: self.jit_events_path.clone(),
                    threshold: self.jit_threshold,
                    phi_min: self.jit_phi_min,
                    hostcall: self.jit_hostcall,
                    handle_debug: self.jit_handle_debug,
                    native_f64: self.jit_native_f64,
                    native_bool: self.jit_native_bool,
                    only: self.jit_only,
                    direct: self.jit_direct,
                },
            },
            build: BuildConfig {
                path: self.build_path.clone(),
                app: self.build_app.clone(),
                out: self.build_out.clone(),
                aot: self.build_aot.clone(),
                profile: self.build_profile.clone(),
                target: self.build_target.clone(),
            },
            emit: EmitConfig {
                emit_cfg: self.emit_cfg.clone(),
                emit_mir_json: self.emit_mir_json.clone(),
                emit_exe: self.emit_exe.clone(),
                emit_exe_nyrt: self.emit_exe_nyrt.clone(),
                emit_exe_libs: self.emit_exe_libs.clone(),
            },
            parser: ParserPipeConfig {
                parser_ny: self.parser_ny,
                ny_parser_pipe: self.ny_parser_pipe,
                json_file: self.json_file.clone(),
            },
            gc_mode: self.gc_mode.clone(),
            compile_wasm: self.compile_wasm,
            compile_native: self.compile_native,
            output_file: self.output_file.clone(),
            benchmark: self.benchmark,
            iterations: self.iterations,
            run_task: self.run_task.clone(),
            load_ny_plugins: self.load_ny_plugins,
        }
    }

    /// Build the clap Command structure
    fn build_command() -> Command {
        Command::new("nyash")
            .version("1.0")
            .author("Claude Code <claude@anthropic.com>")
            .about("🦀 Nyash Programming Language - Everything is Box in Rust! 🦀")
            .arg(
                Arg::new("file")
                    .help("Nyash file to execute")
                    .value_name("FILE")
                    .index(1)
            )
            .arg(
                Arg::new("macro-expand-child")
                    .long("macro-expand-child")
                    .value_name("FILE")
                    .help("Macro sandbox child: read AST JSON v0 from stdin, expand using Nyash macro file, write AST JSON v0 to stdout (PoC)")
            )
            .arg(
                Arg::new("dump-expanded-ast-json")
                    .long("dump-expanded-ast-json")
                    .help("Dump AST after macro expansion as JSON v0 and exit")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("gc")
                    .long("gc")
                    .value_name("{auto,rc+cycle,minorgen,stw,rc,off}")
                    .help("Select GC mode (default: rc+cycle)")
            )
            .arg(
                Arg::new("parser")
                    .long("parser")
                    .value_name("{rust|ny}")
                    .help("Choose parser: 'rust' (default) or 'ny' (direct v0 bridge)")
            )
            .arg(
                Arg::new("ny-parser-pipe")
                    .long("ny-parser-pipe")
                    .help("Read Ny JSON IR v0 from stdin and execute via MIR Interpreter")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("json-file")
                    .long("json-file")
                    .value_name("FILE")
                    .help("Read Ny JSON IR v0 from a file and execute via MIR Interpreter")
            )
            .arg(
                Arg::new("emit-mir-json")
                    .long("emit-mir-json")
                    .value_name("FILE")
                    .help("Emit MIR JSON v0 to file (validation-friendly) and exit")
            )
            .arg(
                Arg::new("emit-exe")
                    .long("emit-exe")
                    .value_name("FILE")
                    .help("Emit native executable via ny-llvmc (crate) and exit")
            )
            .arg(
                Arg::new("emit-exe-nyrt")
                    .long("emit-exe-nyrt")
                    .value_name("DIR")
                    .help("Directory containing libnyrt.a (used with --emit-exe)")
            )
            .arg(
                Arg::new("emit-exe-libs")
                    .long("emit-exe-libs")
                    .value_name("FLAGS")
                    .help("Extra linker flags for ny-llvmc when emitting executable")
            )
            .arg(
                Arg::new("stage3")
                    .long("stage3")
                    .help("Enable Stage-3 syntax acceptance for selfhost parser (sets NYASH_NY_COMPILER_STAGE3=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("ny-compiler-args")
                    .long("ny-compiler-args")
                    .value_name("ARGS")
                    .help("Pass additional args to selfhost child compiler (equivalent to NYASH_NY_COMPILER_CHILD_ARGS)")
            )
            .arg(
                Arg::new("using")
                    .long("using")
                    .value_name("SPEC")
                    .help("Register a using entry (e.g., 'ns as Alias' or '\"apps/foo.nyash\" as Foo'). Repeatable.")
                    .action(clap::ArgAction::Append)
            )

            .arg(
                Arg::new("debug-fuel")
                    .long("debug-fuel")
                    .value_name("ITERATIONS")
                    .help("Set parser debug fuel limit (default: 100000, 'unlimited' for no limit)")
                    .default_value("100000")
            )
            .arg(
                Arg::new("dump-ast")
                    .long("dump-ast")
                    .help("Dump parsed AST and exit")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("profile")
                    .long("profile")
                    .value_name("{lite|dev|ci|strict}")
                    .help("Set execution profile: lite (macros OFF), dev (macros ON), ci (macros ON+strict), strict (macros ON+strict). Default run behaves like dev for macros.")
            )
            .arg(
                Arg::new("expand")
                    .long("expand")
                    .help("Macro: enable macro engine and dump expansion traces (sets NYASH_MACRO_ENABLE=1, NYASH_MACRO_TRACE=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("macro-preexpand")
                    .long("macro-preexpand")
                    .help("Self-host: pre-expand macros before MIR compile (sets NYASH_MACRO_SELFHOST_PRE_EXPAND=1). Requires NYASH_USE_NY_COMPILER=1 and NYASH_VM_USE_PY=1.")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("macro-preexpand-auto")
                    .long("macro-preexpand-auto")
                    .help("Self-host: pre-expand macros in auto mode (sets NYASH_MACRO_SELFHOST_PRE_EXPAND=auto). Requires NYASH_USE_NY_COMPILER=1 and NYASH_VM_USE_PY=1.")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("macro-top-level-allow")
                    .long("macro-top-level-allow")
                    .help("Allow top-level static MacroBoxSpec.expand(json[,ctx]) without BoxDeclaration (sets NYASH_MACRO_TOPLEVEL_ALLOW=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("macro-profile")
                    .long("macro-profile")
                    .value_name("{dev|ci-fast|strict}")
                    .help("Convenience: configure macro envs for dev/ci-fast/strict. Non-breaking; can be overridden by explicit envs/flags.")
            )
            .arg(
                Arg::new("run-tests")
                    .long("run-tests")
                    .help("Run tests: enable macro engine and inject test harness (functions starting with 'test_')")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("test-filter")
                    .long("test-filter")
                    .value_name("SUBSTR")
                    .help("Only run tests whose name contains SUBSTR (with --run-tests)")
            )
            .arg(
                Arg::new("test-entry")
                    .long("test-entry")
                    .value_name("{wrap|override}")
                    .help("When --run-tests and a main exists: wrap (run tests then call original) or override (replace main with test harness). Default: keep original (no harness). Use with --run-tests.")
            )
            .arg(
                Arg::new("test-return")
                    .long("test-return")
                    .value_name("{tests|original}")
                    .help("When --run-tests with --test-entry wrap: choose harness return policy (tests: return failures count; original: return original main()'s result)")
            )
            .arg(
                Arg::new("dump-mir")
                    .long("dump-mir")
                    .help("Dump MIR (Mid-level Intermediate Representation) instead of executing")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("verify")
                    .long("verify")
                    .help("Verify MIR integrity and exit")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("mir-verbose")
                    .long("mir-verbose")
                    .help("Show verbose MIR output with statistics")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("mir-verbose-effects")
                    .long("mir-verbose-effects")
                    .help("Show per-instruction effect category (pure/readonly/side)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("no-optimize")
                    .long("no-optimize")
                    .help("Disable MIR optimizer passes (dump raw Builder MIR)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("backend")
                    .long("backend")
                    .value_name("BACKEND")
                    .help("Choose execution backend: 'vm' (default), 'llvm', or 'interpreter' (legacy)")
                    .default_value("vm")
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .help("Verbose CLI output (sets NYASH_CLI_VERBOSE=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("compile-wasm")
                    .long("compile-wasm")
                    .help("Compile to WebAssembly (WAT/WASM). Requires --features wasm-backend")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("compile-native")
                    .long("compile-native")
                    .help("Compile to native executable (AOT). Requires --features cranelift-jit")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("aot")
                    .long("aot")
                    .help("Short form of --compile-native")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .value_name("FILE")
                    .help("Output file (for WASM compilation or AOT executable)")
            )
            .arg(
                Arg::new("benchmark")
                    .long("benchmark")
                    .help("Run performance benchmarks across all backends")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("iterations")
                    .long("iterations")
                    .value_name("COUNT")
                    .help("Number of iterations for benchmarks (default: 10)")
                    .default_value("10")
            )
            .arg(
                Arg::new("vm-stats")
                    .long("vm-stats")
                    .help("Enable VM instruction statistics (equivalent to NYASH_VM_STATS=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("vm-stats-json")
                    .long("vm-stats-json")
                    .help("Output VM statistics in JSON format")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-exec")
                    .long("jit-exec")
                    .help("Enable JIT execution where available (NYASH_JIT_EXEC=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-stats")
                    .long("jit-stats")
                    .help("Print JIT compilation/execution statistics (NYASH_JIT_STATS=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-stats-json")
                    .long("jit-stats-json")
                    .help("Output JIT statistics in JSON format (NYASH_JIT_STATS_JSON=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-dump")
                    .long("jit-dump")
                    .help("Dump JIT lowering summary (NYASH_JIT_DUMP=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-events")
                    .long("jit-events")
                    .help("Emit JIT events as JSONL (NYASH_JIT_EVENTS=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-events-compile")
                    .long("jit-events-compile")
                    .help("Emit compile-time (lower) JIT events (NYASH_JIT_EVENTS_COMPILE=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-events-runtime")
                    .long("jit-events-runtime")
                    .help("Emit runtime JIT events (NYASH_JIT_EVENTS_RUNTIME=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-events-path")
                    .long("jit-events-path")
                    .value_name("FILE")
                    .help("Write JIT events JSONL to file (NYASH_JIT_EVENTS_PATH)")
            )
            .arg(
                Arg::new("jit-threshold")
                    .long("jit-threshold")
                    .value_name("N")
                    .help("Set hotness threshold for JIT compilation (NYASH_JIT_THRESHOLD)")
            )
            .arg(
                Arg::new("jit-phi-min")
                    .long("jit-phi-min")
                    .help("Enable minimal PHI path for branches (NYASH_JIT_PHI_MIN=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-hostcall")
                    .long("jit-hostcall")
                    .help("Enable JIT hostcall bridge for Array/Map (NYASH_JIT_HOSTCALL=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-handle-debug")
                    .long("jit-handle-debug")
                    .help("Print JIT handle allocation debug logs (NYASH_JIT_HANDLE_DEBUG=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-native-f64")
                    .long("jit-native-f64")
                    .help("Enable native f64 ABI path in JIT (NYASH_JIT_NATIVE_F64=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-native-bool")
                    .long("jit-native-bool")
                    .help("Enable native bool ABI path in JIT (NYASH_JIT_NATIVE_BOOL=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-only")
                    .long("jit-only")
                    .help("Run JIT only (no VM fallback). Fails if JIT is unavailable (NYASH_JIT_ONLY=1)")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("jit-direct")
                    .long("jit-direct")
                    .help("Run program via independent JIT engine (no VM interpreter/executor). Requires --features cranelift-jit")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("emit-cfg")
                    .long("emit-cfg")
                    .value_name("DOT_FILE")
                    .help("Emit JIT CFG as DOT to file (equivalent to setting NYASH_JIT_DOT)")
            )
            .arg(
                Arg::new("run-task")
                    .long("run-task")
                    .value_name("NAME")
                    .help("Run a named task defined in nyash.toml [tasks]")
            )
            .arg(
                Arg::new("load-ny-plugins")
                    .long("load-ny-plugins")
                    .help("Opt-in: read [ny_plugins] from nyash.toml and load scripts in order")
                    .action(clap::ArgAction::SetTrue)
            )
            // Build system (MVP)
            .arg(
                Arg::new("build")
                    .long("build")
                    .value_name("PATH")
                    .help("Build AOT executable using nyash.toml at PATH (MVP)")
            )
            .arg(
                Arg::new("build-app")
                    .long("app")
                    .value_name("FILE")
                    .help("Entry Nyash script for --build (e.g., apps/hello/main.nyash)")
            )
            .arg(
                Arg::new("build-out")
                    .long("out")
                    .value_name("FILE")
                    .help("Output executable name for --build (default: app/app.exe)")
            )
            .arg(
                Arg::new("build-aot")
                    .long("build-aot")
                    .value_name("{cranelift|llvm}")
                    .help("AOT backend for --build (default: cranelift)")
            )
            .arg(
                Arg::new("build-profile")
                    .long("profile")
                    .value_name("{release|debug}")
                    .help("Cargo profile for --build (default: release)")
            )
            .arg(
                Arg::new("build-target")
                    .long("target")
                    .value_name("TRIPLE")
                    .help("Target triple for --build (e.g., x86_64-pc-windows-msvc)")
            )
    }

    /// Convert ArgMatches to CliConfig
    fn from_matches(matches: &ArgMatches) -> Self {
        // Stage-3 gate: when specified via CLI, set env for selfhost child
        if matches.get_flag("stage3") {
            std::env::set_var("NYASH_NY_COMPILER_STAGE3", "1");
        }
        // Side-effect: forward child args for selfhost compiler via env
        if let Some(a) = matches.get_one::<String>("ny-compiler-args") {
            std::env::set_var("NYASH_NY_COMPILER_CHILD_ARGS", a);
        }
        let cfg = Self {
            file: matches.get_one::<String>("file").cloned(),
            debug_fuel: parse_debug_fuel(matches.get_one::<String>("debug-fuel").unwrap()),
            dump_ast: matches.get_flag("dump-ast"),
            dump_mir: matches.get_flag("dump-mir"),
            verify_mir: matches.get_flag("verify"),
            mir_verbose: matches.get_flag("mir-verbose"),
            mir_verbose_effects: matches.get_flag("mir-verbose-effects"),
            no_optimize: matches.get_flag("no-optimize"),
            backend: matches.get_one::<String>("backend").unwrap().clone(),
            compile_wasm: matches.get_flag("compile-wasm"),
            compile_native: matches.get_flag("compile-native") || matches.get_flag("aot"),
            output_file: matches.get_one::<String>("output").cloned(),
            benchmark: matches.get_flag("benchmark"),
            iterations: matches
                .get_one::<String>("iterations")
                .unwrap()
                .parse()
                .unwrap_or(10),
            vm_stats: matches.get_flag("vm-stats"),
            vm_stats_json: matches.get_flag("vm-stats-json"),
            jit_exec: matches.get_flag("jit-exec"),
            jit_stats: matches.get_flag("jit-stats"),
            jit_stats_json: matches.get_flag("jit-stats-json"),
            jit_dump: matches.get_flag("jit-dump"),
            jit_events: matches.get_flag("jit-events"),
            jit_events_compile: matches.get_flag("jit-events-compile"),
            jit_events_runtime: matches.get_flag("jit-events-runtime"),
            jit_events_path: matches.get_one::<String>("jit-events-path").cloned(),
            jit_threshold: matches
                .get_one::<String>("jit-threshold")
                .and_then(|s| s.parse::<u32>().ok()),
            jit_phi_min: matches.get_flag("jit-phi-min"),
            jit_hostcall: matches.get_flag("jit-hostcall"),
            jit_handle_debug: matches.get_flag("jit-handle-debug"),
            jit_native_f64: matches.get_flag("jit-native-f64"),
            jit_native_bool: matches.get_flag("jit-native-bool"),
            emit_cfg: matches.get_one::<String>("emit-cfg").cloned(),
            jit_only: matches.get_flag("jit-only"),
            jit_direct: matches.get_flag("jit-direct"),
            cli_verbose: matches.get_flag("verbose"),
            run_task: matches.get_one::<String>("run-task").cloned(),
            load_ny_plugins: matches.get_flag("load-ny-plugins"),
            gc_mode: matches.get_one::<String>("gc").cloned(),
            parser_ny: matches
                .get_one::<String>("parser")
                .map(|s| s == "ny")
                .unwrap_or(false),
            ny_parser_pipe: matches.get_flag("ny-parser-pipe"),
            json_file: matches.get_one::<String>("json-file").cloned(),
            // Build system (MVP)
            build_path: matches.get_one::<String>("build").cloned(),
            build_app: matches.get_one::<String>("build-app").cloned(),
            build_out: matches.get_one::<String>("build-out").cloned(),
            build_aot: matches.get_one::<String>("build-aot").cloned(),
            build_profile: matches.get_one::<String>("build-profile").cloned(),
            build_target: matches.get_one::<String>("build-target").cloned(),
            cli_usings: matches
                .get_many::<String>("using")
                .map(|v| v.cloned().collect())
                .unwrap_or_else(|| Vec::new()),
            emit_mir_json: matches.get_one::<String>("emit-mir-json").cloned(),
            emit_exe: matches.get_one::<String>("emit-exe").cloned(),
            emit_exe_nyrt: matches.get_one::<String>("emit-exe-nyrt").cloned(),
            emit_exe_libs: matches.get_one::<String>("emit-exe-libs").cloned(),
            macro_expand_child: matches.get_one::<String>("macro-expand-child").cloned(),
            dump_expanded_ast_json: matches.get_flag("dump-expanded-ast-json"),
        };
        // Macro debug gate
        if matches.get_flag("expand") {
            std::env::set_var("NYASH_MACRO_ENABLE", "1");
            std::env::set_var("NYASH_MACRO_TRACE", "1");
        }
        // Profile mapping (non-breaking; users can override afterwards)
        if let Some(p) = matches.get_one::<String>("profile") {
            match p.as_str() {
                "lite" => {
                    std::env::set_var("NYASH_MACRO_ENABLE", "0");
                    std::env::set_var("NYASH_MACRO_STRICT", "0");
                    std::env::set_var("NYASH_MACRO_TRACE", "0");
                }
                "dev" => {
                    std::env::set_var("NYASH_MACRO_ENABLE", "1");
                    std::env::set_var("NYASH_MACRO_STRICT", "1");
                    std::env::set_var("NYASH_MACRO_TRACE", "0");
                }
                "ci" | "strict" => {
                    std::env::set_var("NYASH_MACRO_ENABLE", "1");
                    std::env::set_var("NYASH_MACRO_STRICT", "1");
                    std::env::set_var("NYASH_MACRO_TRACE", "0");
                }
                _ => {}
            }
        }
        if matches.get_flag("run-tests") {
            std::env::set_var("NYASH_MACRO_ENABLE", "1");
            std::env::set_var("NYASH_TEST_RUN", "1");
            if let Some(f) = matches.get_one::<String>("test-filter") {
                std::env::set_var("NYASH_TEST_FILTER", f);
            }
            if let Some(entry) = matches.get_one::<String>("test-entry") {
                let v = entry.as_str();
                if v == "wrap" || v == "override" {
                    std::env::set_var("NYASH_TEST_ENTRY", v);
                }
            }
            if let Some(ret) = matches.get_one::<String>("test-return") {
                let v = ret.as_str();
                if v == "tests" || v == "original" {
                    std::env::set_var("NYASH_TEST_RETURN", v);
                }
            }
        }
        // Self-host macro pre-expand gate (CLI convenience)
        if matches.get_flag("macro-preexpand") {
            std::env::set_var("NYASH_MACRO_SELFHOST_PRE_EXPAND", "1");
        }
        if matches.get_flag("macro-preexpand-auto") {
            std::env::set_var("NYASH_MACRO_SELFHOST_PRE_EXPAND", "auto");
        }
        if matches.get_flag("macro-top-level-allow") {
            std::env::set_var("NYASH_MACRO_TOPLEVEL_ALLOW", "1");
        }
        if let Some(p) = matches.get_one::<String>("macro-profile") {
            let p = p.as_str();
            match p {
                "dev" | "ci-fast" | "strict" => {
                    // Minimal, non-invasive mapping; users can still override.
                    std::env::set_var("NYASH_MACRO_ENABLE", "1");
                    std::env::set_var("NYASH_MACRO_STRICT", "1");
                    std::env::set_var("NYASH_MACRO_TOPLEVEL_ALLOW", "0");
                    std::env::set_var("NYASH_MACRO_SELFHOST_PRE_EXPAND", "auto");
                }
                _ => {}
            }
        }
        cfg
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            file: None,
            debug_fuel: Some(100000),
            dump_ast: false,
            dump_mir: false,
            verify_mir: false,
            mir_verbose: false,
            mir_verbose_effects: false,
            no_optimize: false,
            backend: "interpreter".to_string(),
            compile_wasm: false,
            compile_native: false,
            output_file: None,
            benchmark: false,
            iterations: 10,
            vm_stats: false,
            vm_stats_json: false,
            jit_exec: false,
            jit_stats: false,
            jit_stats_json: false,
            jit_dump: false,
            jit_events: false,
            jit_events_compile: false,
            jit_events_runtime: false,
            jit_events_path: None,
            jit_threshold: None,
            jit_phi_min: false,
            jit_hostcall: false,
            jit_handle_debug: false,
            jit_native_f64: false,
            jit_native_bool: false,
            emit_cfg: None,
            jit_only: false,
            jit_direct: false,
            cli_verbose: false,
            run_task: None,
            load_ny_plugins: false,
            gc_mode: None,
            parser_ny: false,
            ny_parser_pipe: false,
            json_file: None,
            build_path: None,
            build_app: None,
            build_out: None,
            build_aot: None,
            build_profile: None,
            build_target: None,
            cli_usings: Vec::new(),
            emit_mir_json: None,
            emit_exe: None,
            emit_exe_nyrt: None,
            emit_exe_libs: None,
            macro_expand_child: None,
            dump_expanded_ast_json: false,
        }
    }
}

/// Parse debug fuel value ("unlimited" or numeric)
fn parse_debug_fuel(value: &str) -> Option<usize> {
    if value == "unlimited" {
        None // No limit
    } else {
        value.parse::<usize>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_debug_fuel() {
        assert_eq!(parse_debug_fuel("unlimited"), None);
        assert_eq!(parse_debug_fuel("1000"), Some(1000));
        assert_eq!(parse_debug_fuel("invalid"), None);
    }

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.backend, "interpreter");
        assert_eq!(config.iterations, 10);
    }
}
