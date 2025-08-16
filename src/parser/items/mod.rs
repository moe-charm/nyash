/*!
 * Parser Items Module
 * 
 * Top-level item declarations:
 * - Global variables
 * - Function declarations
 * - Static declarations (functions and boxes)
 */

pub mod global_vars;
pub mod functions;
pub mod static_items;

// Re-export for convenience
pub use global_vars::*;
pub use functions::*;
pub use static_items::*;