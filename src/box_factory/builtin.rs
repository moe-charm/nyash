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
use super::builtin_impls;

/// Factory for builtin Box types
pub struct BuiltinBoxFactory;

impl BuiltinBoxFactory {
    pub fn new() -> Self {
        Self
    }
}

impl BoxFactory for BuiltinBoxFactory {
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        // Phase 0: ✅ Route to separated implementations (easy deletion)
        match name {
            // Phase 2.1-2.2: DELETE when plugins are confirmed working
            "StringBox" => builtin_impls::string_box::create(args),
            "IntegerBox" => builtin_impls::integer_box::create(args),

            // Phase 2.3: DELETE when BoolBox plugin is created
            "BoolBox" => builtin_impls::bool_box::create(args),

            // Phase 2.4-2.5: DELETE when collection plugins confirmed
            "ArrayBox" => builtin_impls::array_box::create(args),
            "MapBox" => builtin_impls::map_box::create(args),

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
            "StringBox",
            "IntegerBox",
            "BoolBox",
            // Collections/common
            "ArrayBox",
            "MapBox",
            "ConsoleBox",
            "NullBox",
        ]
    }

    fn is_builtin_factory(&self) -> bool {
        true
    }
}
