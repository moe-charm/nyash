/*!
 Minimal CLI entry point for Nyash.
 Delegates to the library crate (`nyash_rust`) for all functionality.
*/

use nyash_rust::cli::CliConfig;
use nyash_rust::config::env as env_config;
use nyash_rust::runner::NyashRunner;

/// Thin entry point - delegates to CLI parsing and runner execution
fn main() {
    // Bootstrap brand env aliases (HAKU_/HRN_ → NYASH_)
    env_config::alias_prefixes_bootstrap();
    // Bootstrap env overrides from nyash.toml/hakorune.toml [env] early (管理棟)
    env_config::bootstrap_from_toml_env();
    // Deprecation note when invoked as `nyash` (use `hako`) unless JSON-only
    if let Ok(exe) = std::env::current_exe() {
        if let Some(name) = exe.file_name().and_then(|s| s.to_str()) {
            if name == "nyash" && std::env::var("NYASH_JSON_ONLY").ok().as_deref() != Some("1") {
                eprintln!("[deprecate] CLI name 'nyash' is deprecated; use 'hako' instead.");
            }
        }
    }
    // Parse command-line arguments
    let config = CliConfig::parse();

    // Create and run the execution coordinator
    let runner = NyashRunner::new(config);
    runner.run();
}

#[cfg(test)]
mod tests {
    
    use nyash_rust::box_trait::{NyashBox, StringBox};

    #[test]
    fn test_main_functionality() {
        // Smoke: library module path wiring works
        let string_box = StringBox::new("test".to_string());
        assert_eq!(string_box.to_string_box().value, "test");
        let _ = (); // CLI wiring exists via nyash_rust::cli
    }
}
