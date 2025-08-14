/*!
 * CLI Argument Parsing Module - Nyash Command Line Interface
 * 
 * This module handles all command-line argument parsing using clap,
 * separating CLI concerns from the main execution logic.
 */

use clap::{Arg, Command, ArgMatches};

/// Command-line configuration structure
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub file: Option<String>,
    pub debug_fuel: Option<usize>,
    pub dump_mir: bool,
    pub verify_mir: bool,
    pub mir_verbose: bool,
    pub backend: String,
    pub compile_wasm: bool,
    pub compile_native: bool,
    pub output_file: Option<String>,
    pub benchmark: bool,
    pub iterations: u32,
}

impl CliConfig {
    /// Parse command-line arguments and return configuration
    pub fn parse() -> Self {
        let matches = Self::build_command().get_matches();
        Self::from_matches(&matches)
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
                Arg::new("debug-fuel")
                    .long("debug-fuel")
                    .value_name("ITERATIONS")
                    .help("Set parser debug fuel limit (default: 100000, 'unlimited' for no limit)")
                    .default_value("100000")
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
                Arg::new("backend")
                    .long("backend")
                    .value_name("BACKEND")
                    .help("Choose execution backend: 'interpreter' (default) or 'vm'")
                    .default_value("interpreter")
            )
            .arg(
                Arg::new("compile-wasm")
                    .long("compile-wasm")
                    .help("Compile to WebAssembly (WAT format) instead of executing")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("compile-native")
                    .long("compile-native")
                    .help("Compile to native AOT executable using wasmtime precompilation")
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
    }

    /// Convert ArgMatches to CliConfig
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            file: matches.get_one::<String>("file").cloned(),
            debug_fuel: parse_debug_fuel(matches.get_one::<String>("debug-fuel").unwrap()),
            dump_mir: matches.get_flag("dump-mir"),
            verify_mir: matches.get_flag("verify"),
            mir_verbose: matches.get_flag("mir-verbose"),
            backend: matches.get_one::<String>("backend").unwrap().clone(),
            compile_wasm: matches.get_flag("compile-wasm"),
            compile_native: matches.get_flag("compile-native") || matches.get_flag("aot"),
            output_file: matches.get_one::<String>("output").cloned(),
            benchmark: matches.get_flag("benchmark"),
            iterations: matches.get_one::<String>("iterations").unwrap().parse().unwrap_or(10),
        }
    }
}

/// Parse debug fuel value ("unlimited" or numeric)
fn parse_debug_fuel(value: &str) -> Option<usize> {
    if value == "unlimited" {
        None  // No limit
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
        // This test would require mocking clap's behavior
        // For now, we just ensure the structure is valid
        let config = CliConfig {
            file: None,
            debug_fuel: Some(100000),
            dump_mir: false,
            verify_mir: false,
            mir_verbose: false,
            backend: "interpreter".to_string(),
            compile_wasm: false,
            compile_native: false,
            output_file: None,
            benchmark: false,
            iterations: 10,
        };
        
        assert_eq!(config.backend, "interpreter");
        assert_eq!(config.iterations, 10);
    }
}