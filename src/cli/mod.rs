/*!
 * CLI Argument Parsing Module - Nyash Command Line Interface (split)
 */

mod args;
mod groups;
mod utils;


/// Command-line configuration structure
#[derive(Debug, Clone)]
pub struct CliConfig {
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
    pub emit_cfg: Option<String>,
    pub cli_verbose: bool,
    pub run_task: Option<String>,
    pub load_ny_plugins: bool,
    pub parser_ny: bool,
    pub ny_parser_pipe: bool,
    pub json_file: Option<String>,
    pub gc_mode: Option<String>,
    pub build_path: Option<String>,
    pub build_app: Option<String>,
    pub build_out: Option<String>,
    pub build_aot: Option<String>,
    pub build_profile: Option<String>,
    pub build_target: Option<String>,
    pub cli_usings: Vec<String>,
    pub entry: Option<String>,
    pub emit_mir_json: Option<String>,
    pub emit_exe: Option<String>,
    pub emit_exe_nyrt: Option<String>,
    pub emit_exe_libs: Option<String>,
    pub macro_expand_child: Option<String>,
    pub dump_expanded_ast_json: bool,
    pub macro_ctx_json: Option<String>,
    // Optional explicit entry (dotted), e.g., "Main.main"
    pub entry: Option<String>,
}

pub use groups::{BackendConfig, BuildConfig, CliGroups, DebugConfig, EmitConfig, InputConfig, JitConfig, ParserPipeConfig};

impl CliConfig {
    pub fn parse() -> Self { args::parse() }

    pub fn as_groups(&self) -> CliGroups {
        CliGroups {
            input: InputConfig { file: self.file.clone(), cli_usings: self.cli_usings.clone(), entry: self.entry.clone() },
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
            macro_ctx_json: None,
            entry: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_debug_fuel() {
        assert_eq!(super::utils::parse_debug_fuel("unlimited"), None);
        assert_eq!(super::utils::parse_debug_fuel("1000"), Some(1000));
        assert_eq!(super::utils::parse_debug_fuel("invalid"), None);
    }
    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.backend, "interpreter");
        assert_eq!(config.iterations, 10);
    }
}
