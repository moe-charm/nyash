/*!
 Minimal CLI entry point for Nyash.
 Delegates to the library crate (`nyash_rust`) for all functionality.
*/

use nyash_rust::cli::CliConfig;
use nyash_rust::config::env as env_config;
use nyash_rust::runner::NyashRunner;
use nyash_rust::runner::modes::common_util::exec;

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

    // Tools helpers: --which / --doctor tools
    if let Some(tool) = &config.which_tool {
        match which_tool(tool) {
            Some((p, origin)) => { println!("{} ({})", p, origin); std::process::exit(0); },
            None => { eprintln!("not found: {}", tool); std::process::exit(1); }
        }
    }
    if config.doctor_tools {
        let items = ["plugin-tester", "llvm-harness", "ny-llvmc"];
        for it in items.iter() {
            match which_tool(it) {
                Some((p, origin)) => println!("[tools] {:<14} => {} ({})", it, p, origin),
                None => println!("[tools] {:<14} => MISSING", it),
            }
        }
        std::process::exit(0);
    }

    // Create and run the execution coordinator
    let runner = NyashRunner::new(config);
    runner.run();
}

fn which_tool(name: &str) -> Option<(String, &'static str)> {
    use std::path::PathBuf;
    let cwd = std::env::current_dir().ok();
    match name {
        "plugin-tester" => {
            if let Some(mut p) = cwd.clone() { p.push("tools/plugin-tester/target/release/plugin-tester"); if p.exists() { return Some((p.display().to_string(), "workspace")); } }
            which::which("plugin-tester").ok().map(|p| (p.display().to_string(), "PATH"))
        }
        "llvm-harness" => {
            if let Some(mut p) = cwd.clone() { p.push("tools/llvmlite_harness.py"); if p.exists() { return Some((p.display().to_string(), "workspace")); } }
            if let Some(p0) = cwd { let mut q = PathBuf::from(p0); q.push("apps/llvm/harness.py"); if q.exists() { return Some((q.display().to_string(), "workspace")); } }
            None
        }
        "ny-llvmc" => {
            let p = exec::resolve_ny_llvmc();
            if p.exists() { Some((p.display().to_string(), "resolver")) } else { None }
        }
        other => {
            which::which(other).ok().map(|p| (p.display().to_string(), "PATH"))
        }
    }
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
