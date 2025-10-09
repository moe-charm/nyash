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

fn env_truthy(key: &str) -> bool {
    matches!(std::env::var(key).ok().as_deref(), Some("1"|"true"|"on"|"yes"))
}

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
            // StringBox removed — use plugin provider
            "IntegerBox" => builtin_impls::integer_box::create(args),

            // Phase 2.3: DELETE when BoolBox plugin is created
            "BoolBox" => builtin_impls::bool_box::create(args),

            // ArrayBox/MapBox removed — use plugin providers

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
        ]
    }

    fn is_builtin_factory(&self) -> bool {
        true
    }
}
