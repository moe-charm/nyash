//! Static trait implementations for Box operators
//!
//! This module contains all static trait implementations (NyashAdd, NyashSub, etc.)
//! for basic Box types. It uses both macro-generated implementations for numeric
//! types and manual implementations for special cases.

use crate::box_trait::{BoolBox, IntegerBox, StringBox};
use crate::boxes::FloatBox;
use crate::operator_traits::{NyashAdd, NyashMul};
use crate::impl_static_numeric_ops;

// ===== Macro-generated static implementations =====

/// Static numeric operations for IntegerBox
///
/// Generates implementations for: Add, Sub, Mul, Div with zero-division error handling
impl_static_numeric_ops!(IntegerBox, 0);

/// Static numeric operations for FloatBox
///
/// Generates implementations for: Add, Sub, Mul, Div with zero-division error handling
impl_static_numeric_ops!(FloatBox, 0.0);

// ===== Manual static implementations for special cases =====

/// StringBox + StringBox -> StringBox (concatenation)
impl NyashAdd<StringBox> for StringBox {
    type Output = StringBox;

    fn add(self, rhs: StringBox) -> Self::Output {
        StringBox::new(format!("{}{}", self.value, rhs.value))
    }
}

/// StringBox * IntegerBox -> StringBox (repetition)
impl NyashMul<IntegerBox> for StringBox {
    type Output = StringBox;

    fn mul(self, rhs: IntegerBox) -> Self::Output {
        if rhs.value >= 0 && rhs.value <= 10000 {
            // Safety limit to prevent memory exhaustion
            StringBox::new(self.value.repeat(rhs.value as usize))
        } else {
            StringBox::new(String::new()) // Empty string for invalid repetition
        }
    }
}

/// BoolBox + BoolBox -> IntegerBox (logical OR as addition)
impl NyashAdd<BoolBox> for BoolBox {
    type Output = IntegerBox;

    fn add(self, rhs: BoolBox) -> Self::Output {
        let result = (self.value as i64) + (rhs.value as i64);
        IntegerBox::new(result)
    }
}

// Note: Additional static implementations can be added here as needed
// for cross-type operations or special Box types