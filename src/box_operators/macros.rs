//! Macro definitions for Box operator implementations
//!
//! This module contains the macro system for generating static trait implementations
//! for numeric Box types (IntegerBox, FloatBox, etc.)

// Note: The traits and OperatorError are used within the macro expansion,
// so they appear unused to the compiler but are actually required.

/// Generate static numeric operator implementations for a given Box type
///
/// This macro creates implementations of NyashAdd, NyashSub, NyashMul, and NyashDiv
/// for the specified type, with built-in error handling (e.g., division by zero).
///
/// # Arguments
///
/// * `$ty` - The Box type to implement operators for (e.g., IntegerBox, FloatBox)
/// * `$zero` - The zero value for the type (e.g., 0 for integers, 0.0 for floats)
///
/// # Example
///
/// ```rust
/// impl_static_numeric_ops!(IntegerBox, 0);
/// impl_static_numeric_ops!(FloatBox, 0.0);
/// ```
#[macro_export]
macro_rules! impl_static_numeric_ops {
    ($ty:ty, $zero:expr) => {
        impl crate::operator_traits::NyashAdd<$ty> for $ty {
            type Output = $ty;
            fn add(self, rhs: $ty) -> Self::Output {
                < $ty >::new(self.value + rhs.value)
            }
        }

        impl crate::operator_traits::NyashSub<$ty> for $ty {
            type Output = $ty;
            fn sub(self, rhs: $ty) -> Self::Output {
                < $ty >::new(self.value - rhs.value)
            }
        }

        impl crate::operator_traits::NyashMul<$ty> for $ty {
            type Output = $ty;
            fn mul(self, rhs: $ty) -> Self::Output {
                < $ty >::new(self.value * rhs.value)
            }
        }

        impl crate::operator_traits::NyashDiv<$ty> for $ty {
            type Output = Result<$ty, crate::operator_traits::OperatorError>;
            fn div(self, rhs: $ty) -> Self::Output {
                if rhs.value == $zero {
                    Err(crate::operator_traits::OperatorError::DivisionByZero)
                } else {
                    Ok(< $ty >::new(self.value / rhs.value))
                }
            }
        }
    };
}

// Re-export the macro for external use
pub use impl_static_numeric_ops;