/*!
 * Interpreter Objects Module (mod)
 *
 * Split into submodules:
 * - ops.rs: instantiation (execute_new) and helpers
 * - methods.rs: constructor-related methods
 * - fields.rs: declarations, inheritance, generics utilities
 */

use super::*;

mod fields;
mod methods;
mod ops;
