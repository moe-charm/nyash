/*!
 Hako CLI entry point — thin wrapper delegating to library crate.
*/

use nyash_rust::cli::CliConfig;
use nyash_rust::config::env as env_config;
use nyash_rust::runner::NyashRunner;

fn main() {
    env_config::bootstrap_from_toml_env();
    let config = CliConfig::parse();
    let runner = NyashRunner::new(config);
    runner.run();
}
