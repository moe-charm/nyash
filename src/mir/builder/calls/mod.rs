/*!
 * Call System Module Organization
 *
 * Refactored from monolithic builder_calls.rs (879 lines)
 * Split into focused modules following Single Responsibility Principle
 */

// Core types
pub mod call_target;

// Resolution system
pub mod method_resolution;

// External calls
pub mod extern_calls;

// Special handlers
pub mod special_handlers;

// Function lowering
pub mod function_lowering;

// Unified call system
pub mod call_unified;

// Re-export commonly used items




