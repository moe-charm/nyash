// Basic Box Unification: re-export canonical IntegerBox
// Canonical implementation lives in `crate::box_trait::IntegerBox`.
// This module re-exports it so both `crate::box_trait::IntegerBox` and
// `crate::boxes::integer_box::IntegerBox` refer to the same runtime type.

pub use crate::box_trait::IntegerBox;
