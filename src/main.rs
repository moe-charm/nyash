/*!
 * Nyash Rust Implementation - Everything is Box in Memory Safe Rust
 * 
 * This is the main entry point for the Rust implementation of Nyash,
 * demonstrating the "Everything is Box" philosophy with Rust's ownership system.
 * 
 * The main function serves as a thin entry point that delegates to the CLI
 * and runner modules for actual processing.
 */

// Core modules
pub mod box_trait;
pub mod boxes;
pub mod box_factory; // 🏭 Unified Box Factory Architecture (Phase 9.78)
pub mod stdlib;
pub mod environment;
pub mod tokenizer;
pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod instance_v2; // 🎯 Phase 9.78d: Simplified InstanceBox implementation
pub mod channel_box;
pub mod finalization;
pub mod exception_box;
pub mod method_box;
pub mod operator_traits;
pub mod box_arithmetic; // 🚀 Moved from box_trait.rs for better organization
pub mod value; // 🔥 NyashValue Revolutionary System
pub mod type_box;  // 🌟 TypeBox revolutionary system
pub mod messaging; // 🌐 P2P Communication Infrastructure
pub mod transport; // 🌐 P2P Communication Infrastructure

// 🚀 MIR Infrastructure
pub mod mir;

// 🚀 Backend Infrastructure  
pub mod backend;

// 📊 Performance Benchmarks
pub mod benchmarks;

// 🚀 Refactored modules for better organization
pub mod cli;
pub mod runner;

// BID-FFI / Plugin System (prototype)
pub mod bid;

// Configuration system
pub mod config;

// Runtime system (plugins, registry, etc.)
pub mod runtime;

use nyash_rust::cli::CliConfig;
use runner::NyashRunner;

/// Thin entry point - delegates to CLI parsing and runner execution
fn main() {
    // Parse command-line arguments
    let config = CliConfig::parse();
    
    // Create and run the execution coordinator
    let runner = NyashRunner::new(config);
    runner.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use box_trait::{StringBox, BoxCore};
    
    #[test]
    fn test_main_functionality() {
        // This test ensures the module structure is correct
        let string_box = StringBox::new("test".to_string());
        assert_eq!(string_box.to_string_box().value, "test");
    }
}
