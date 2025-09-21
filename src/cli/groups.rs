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
pub struct BackendConfig {
    pub backend: String,
    pub vm_stats: bool,
    pub vm_stats_json: bool,
    pub jit: JitConfig,
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

