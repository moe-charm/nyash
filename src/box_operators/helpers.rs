//! Helper functions and type conversion utilities for Box operators
//!
//! This module contains utility functions used across the operator system,
//! primarily for type conversion and validation.

use crate::box_trait::{NyashBox, StringBox};

/// Concatenate two boxes by converting both to strings
///
/// This function provides the fallback behavior for addition operations
/// when type-specific arithmetic is not available - it converts both
/// operands to strings and concatenates them.
///
/// # Arguments
///
/// * `left` - The left operand
/// * `right` - The right operand
///
/// # Returns
///
/// A StringBox containing the concatenated string representation
/// of both operands.
///
/// # Example
///
/// ```rust
/// let left = IntegerBox::new(42);
/// let right = BoolBox::new(true);
/// let result = concat_result(&left, &right);
/// // result will be StringBox("42true")
/// ```
#[inline]
pub fn concat_result(left: &dyn NyashBox, right: &dyn NyashBox) -> Box<dyn NyashBox> {
    let l = left.to_string_box();
    let r = right.to_string_box();
    Box::new(StringBox::new(format!("{}{}", l.value, r.value)))
}

/// Check if a repetition count is within safe limits
///
/// This function validates that string repetition operations stay within
/// reasonable bounds to prevent memory exhaustion attacks.
///
/// # Arguments
///
/// * `times` - The number of repetitions requested
///
/// # Returns
///
/// `true` if the repetition count is safe (0-10,000), `false` otherwise.
///
/// # Example
///
/// ```rust
/// assert!(can_repeat(5));     // OK
/// assert!(can_repeat(10000)); // OK (at limit)
/// assert!(!can_repeat(10001)); // Too many
/// assert!(!can_repeat(-1));   // Negative
/// ```
#[inline]
pub fn can_repeat(times: i64) -> bool {
    (0..=10_000).contains(&times)
}