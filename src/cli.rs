/*!
 * CLI Argument Parsing Module - Nyash Command Line Interface
 * 
 * This module handles all command-line argument parsing using clap,
 * separating CLI concerns from the main execution logic.
 */

use clap::{Arg, Command, ArgMatches, Subcommand};

/// Command-line configuration structure
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub command: NyashCommand,
}

/// Top-level commands supported by Nyash
#[derive(Debug, Clone)]
pub enum NyashCommand {
    /// Run a Nyash file
    Run {
        file: Option<String>,
        debug_fuel: Option<usize>,
        dump_mir: bool,
        verify_mir: bool,
        mir_verbose: bool,
        backend: String,
        compile_wasm: bool,
        compile_native: bool,
        output_file: Option<String>,
        benchmark: bool,
        iterations: u32,
    },
    /// BID (Box Interface Definition) tools
    Bid {
        subcommand: BidSubcommand,
    },
}

/// BID subcommands
#[derive(Debug, Clone)]
pub enum BidSubcommand {
    /// Generate code from BID files
    Gen {
        target: String,
        bid_file: String,
        output_dir: Option<String>,
        force: bool,
        dry_run: bool,
    },
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
            .subcommand_required(false)
            .arg_required_else_help(false)
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
                    .help("Choose execution backend: 'interpreter' (default), 'vm', or 'llvm'")
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
            .subcommand(
                Command::new("bid")
                    .about("BID (Box Interface Definition) tools")
                    .subcommand_required(true)
                    .subcommand(
                        Command::new("gen")
                            .about("Generate code from BID files")
                            .arg(
                                Arg::new("target")
                                    .long("target")
                                    .short('t')
                                    .value_name("TARGET")
                                    .help("Target platform: wasm, vm, llvm, ts, py")
                                    .required(true)
                                    .value_parser(["wasm", "vm", "llvm", "ts", "py"])
                            )
                            .arg(
                                Arg::new("bid-file")
                                    .help("BID YAML file to process")
                                    .value_name("BID_FILE")
                                    .required(true)
                                    .index(1)
                            )
                            .arg(
                                Arg::new("out")
                                    .long("out")
                                    .short('o')
                                    .value_name("DIR")
                                    .help("Output directory (default: out/<target>/<name>/)")
                            )
                            .arg(
                                Arg::new("force")
                                    .long("force")
                                    .help("Overwrite existing files")
                                    .action(clap::ArgAction::SetTrue)
                            )
                            .arg(
                                Arg::new("dry-run")
                                    .long("dry-run")
                                    .help("Preview generated files without writing")
                                    .action(clap::ArgAction::SetTrue)
                            )
                    )
            )
    }

    /// Convert ArgMatches to CliConfig
    fn from_matches(matches: &ArgMatches) -> Self {
        // Check if this is a BID subcommand
        if let Some(("bid", bid_matches)) = matches.subcommand() {
            let subcommand = match bid_matches.subcommand() {
                Some(("gen", gen_matches)) => BidSubcommand::Gen {
                    target: gen_matches.get_one::<String>("target").unwrap().clone(),
                    bid_file: gen_matches.get_one::<String>("bid-file").unwrap().clone(),
                    output_dir: gen_matches.get_one::<String>("out").cloned(),
                    force: gen_matches.get_flag("force"),
                    dry_run: gen_matches.get_flag("dry-run"),
                },
                _ => unreachable!("clap should prevent this"),
            };
            
            return Self {
                command: NyashCommand::Bid { subcommand },
            };
        }
        
        // Default to Run command
        Self {
            command: NyashCommand::Run {
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
            },
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
            command: NyashCommand::Run {
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
            },
        };
        
        match config.command {
            NyashCommand::Run { backend, iterations, .. } => {
                assert_eq!(backend, "interpreter");
                assert_eq!(iterations, 10);
            },
            _ => panic!("Expected Run command"),
        }
    }
}