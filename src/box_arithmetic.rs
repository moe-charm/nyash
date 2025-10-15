/*!
 * Box Operations - Binary and unary operations between boxes
 *
 * This module has been refactored into individual files for better maintainability.
 * Each arithmetic operation now has its own dedicated file in the arithmetic/ subdirectory.
 */

// Re-export all arithmetic operations from the dedicated arithmetic module
#[cfg(feature = "legacy-boxes")]
pub use crate::boxes::arithmetic::{AddBox, SubtractBox, MultiplyBox, DivideBox, ModuloBox, CompareBox};
