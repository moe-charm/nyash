// HakoRune CLI shim (alias of nyash)
// Non-breaking: delegates to nyash_rust runner with branding env aliases.

use nyash_rust::cli::CliConfig;
use nyash_rust::config::env as env_config;
use nyash_rust::runner::NyashRunner;

fn main() {
    // Accept HAKU_/HRN_ env aliases, then load [env] from nyash.toml/hakorune.toml
    env_config::alias_prefixes_bootstrap();
    env_config::bootstrap_from_toml_env();

    // Parse CLI and run as usual
    let config = CliConfig::parse();
    let runner = NyashRunner::new(config);
    runner.run();
}

