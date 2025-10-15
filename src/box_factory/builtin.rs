/*!
 * Builtin Box Factory (Phase 15.5: Transitioning to "Everything is Plugin")
 *
 * ⚠️ MIGRATION IN PROGRESS: Phase 15.5 Core Box Unification
 * 🎯 Goal: Remove builtin priority, make all Boxes plugin-based
 * 📋 Current: builtin > user > plugin (PROBLEMATIC)
 * 🚀 Target: plugin > user > builtin_compat (Phase 1) → plugin-only (Phase 3)
 *
 * Implementation Strategy:
 * - Phase 0: ✅ Separate implementations to builtin_impls/ (easy deletion)
 * - Phase 1: 🚧 Add strict_plugin_first policy + access guards
 * - Phase 2: 🔄 Delete builtin_impls/ files one by one
 * - Phase 3: ❌ Delete BuiltinBoxFactory entirely
 */

use super::BoxFactory;
use super::RuntimeError;
use crate::box_trait::NyashBox;

// Separated implementations (Phase 0: ✅ Complete)
#[cfg(feature = "legacy-boxes")]
use super::builtin_impls;

/// Factory for builtin Box types
pub struct BuiltinBoxFactory;

impl BuiltinBoxFactory {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "legacy-boxes")]
impl BoxFactory for BuiltinBoxFactory {
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Phase 0: ✅ Route to separated implementations (easy deletion)
        match name {
            // StringBox removed — use plugin provider
            "IntegerBox" => builtin_impls::integer_box::create(args),

            // Phase 2.3: DELETE when BoolBox plugin is created
            "BoolBox" => builtin_impls::bool_box::create(args),

            // ArrayBox/MapBox removed — use plugin providers
            // SetBox is a thin wrapper around MapBox (legacy helper)
            "SetBox" => builtin_impls::set_box::create(args),

            // Phase 2.6: DELETE LAST (critical for logging)
            "ConsoleBox" => builtin_impls::console_box::create(args),

            // Special: Keep vs Delete discussion needed
            "NullBox" => builtin_impls::null_box::create(args),

            // Leave other types to other factories (user/plugin)
            _ => Err(RuntimeError::InvalidOperation {
                message: format!("Unknown Box type: {}", name),
            }),
        }
    }

    fn box_types(&self) -> Vec<&str> {
        vec![
            // Primitive wrappers
            "IntegerBox",
            "BoolBox",
            // Collections/common
            "ConsoleBox",
            "NullBox",
            "SetBox",
        ]
    }

    fn is_builtin_factory(&self) -> bool {
        true
    }
}

#[cfg(not(feature = "legacy-boxes"))]
impl BoxFactory for BuiltinBoxFactory {
    fn create_box(
        &self,
        _name: &str,
        _args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        Err(RuntimeError::InvalidOperation { message: "builtin boxes disabled (legacy-boxes OFF)".to_string() })
    }

    fn box_types(&self) -> Vec<&str> { vec![] }
    fn is_builtin_factory(&self) -> bool { false }
}
