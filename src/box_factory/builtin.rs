/*!
 * Builtin Box Factory
 *
 * Provides constructors for core builtin Box types so that the unified
 * registry can create them without relying on plugins.
 * Priority order in UnifiedBoxRegistry remains: builtin > user > plugin.
 */

use super::BoxFactory;
use crate::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox};
use crate::interpreter::RuntimeError;

/// Factory for builtin Box types
pub struct BuiltinBoxFactory;

impl BuiltinBoxFactory {
    pub fn new() -> Self { Self }
}

impl BoxFactory for BuiltinBoxFactory {
    fn create_box(
        &self,
        name: &str,
        args: &[Box<dyn NyashBox>],
    ) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match name {
            // Primitive wrappers
            "StringBox" => {
                if let Some(arg0) = args.get(0) {
                    if let Some(sb) = arg0.as_any().downcast_ref::<StringBox>() {
                        return Ok(Box::new(StringBox::new(&sb.value)));
                    }
                }
                Ok(Box::new(StringBox::new("")))
            }
            "IntegerBox" => {
                if let Some(arg0) = args.get(0) {
                    if let Some(ib) = arg0.as_any().downcast_ref::<IntegerBox>() {
                        return Ok(Box::new(IntegerBox::new(ib.value)));
                    }
                }
                Ok(Box::new(IntegerBox::new(0)))
            }
            "BoolBox" => {
                if let Some(arg0) = args.get(0) {
                    if let Some(bb) = arg0.as_any().downcast_ref::<BoolBox>() {
                        return Ok(Box::new(BoolBox::new(bb.value)));
                    }
                }
                Ok(Box::new(BoolBox::new(false)))
            }

            // Collections and common boxes
            "ArrayBox" => Ok(Box::new(crate::boxes::array::ArrayBox::new())),
            "MapBox" => Ok(Box::new(crate::boxes::map_box::MapBox::new())),
            "ConsoleBox" => Ok(Box::new(crate::boxes::console_box::ConsoleBox::new())),
            "NullBox" => Ok(Box::new(crate::boxes::null_box::NullBox::new())),

            // Leave other types to other factories (user/plugin)
            _ => Err(RuntimeError::InvalidOperation { message: format!("Unknown Box type: {}", name) }),
        }
    }

    fn box_types(&self) -> Vec<&str> {
        vec![
            // Primitive wrappers
            "StringBox", "IntegerBox", "BoolBox",
            // Collections/common
            "ArrayBox", "MapBox", "ConsoleBox", "NullBox",
        ]
    }

    fn is_builtin_factory(&self) -> bool { true }
}
