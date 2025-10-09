//! Test harness system for Nyash/Hakorune
//!
//! Provides automatic test runner injection for test_* functions/methods.
//!
//! ## Usage
//! Set `NYASH_TEST_RUN=1` or `NYASH_RUN_TESTS=1` to enable test harness.
//!
//! ## Environment Variables
//! - `NYASH_TEST_ARGS_JSON`: JSON map of test arguments
//! - `NYASH_TEST_FILTER`: Substring filter for test names
//! - `NYASH_TEST_FORCE=1`: Force harness even if main() exists
//! - `NYASH_TEST_ENTRY=wrap|override`: Control main() handling
//! - `NYASH_TEST_RETURN=tests|original`: Control return value
//! - `NYASH_TEST_ARGS_DEFAULTS=1`: Use zero defaults for missing args

mod json_args;
mod harness;

use nyash_rust::ASTNode;

/// Inject test harness into AST if NYASH_TEST_RUN=1
pub fn maybe_inject_test_harness(ast: &ASTNode) -> ASTNode {
    let run_tests = std::env::var("NYASH_TEST_RUN").ok().as_deref() == Some("1")
        || std::env::var("NYASH_RUN_TESTS").ok().as_deref() == Some("1");
    if !run_tests {
        return ast.clone();
    }

    let args_map = json_args::parse_args_map_from_env();
    harness::inject_test_harness(ast, &args_map)
}
