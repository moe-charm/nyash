/*!
 * MIR Builder Module - Modular AST to MIR conversion
 * 
 * This module contains the refactored MIR builder split into focused sub-modules:
 * 
 * - `core`: Core builder functionality and instruction emission
 * - `expressions`: Expression AST node conversion 
 * - `statements`: Statement AST node conversion
 * - `control_flow`: Control flow constructs (if, loop, try-catch)
 * - `box_handlers`: Box-related operations (new, declarations)
 */

pub mod core;
pub mod expressions;
pub mod statements; 
pub mod control_flow;
pub mod box_handlers;

// Re-export the main builder struct and key functionality
pub use self::core::MirBuilder;

// Re-export commonly used types from the parent module
pub use super::{
    MirInstruction, BasicBlock, BasicBlockId, MirFunction, MirModule,
    FunctionSignature, ValueId, ConstValue, BinaryOp, UnaryOp, CompareOp,
    MirType, EffectMask, Effect, BasicBlockIdGenerator, ValueIdGenerator, TypeOpKind
};
