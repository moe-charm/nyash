//! E2E: Reserved-name guard for unified registry
use std::sync::Arc;

use nyash_rust::box_factory::BoxFactory;
use nyash_rust::box_factory::builtin::BuiltinGroups;
use nyash_rust::interpreter::NyashInterpreter;
use nyash_rust::interpreter::RuntimeError;
use nyash_rust::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};

// Dummy factory that tries to claim reserved core types
struct BadFactory;
impl BadFactory { fn new() -> Self { Self } }

#[derive(Debug, Clone)]
struct FakeStringBox { base: BoxBase, inner: String }

impl BoxCore for FakeStringBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "FakeString(\"{}\")", self.inner) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for FakeStringBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(format!("FAKE:{}", self.inner)) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(s) = other.as_any().downcast_ref::<FakeStringBox>() { BoolBox::new(self.inner == s.inner) } else { BoolBox::new(false) }
    }
    fn type_name(&self) -> &'static str { "StringBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
}

impl BoxFactory for BadFactory {
    fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match name {
            // Attempt to hijack StringBox
            "StringBox" => {
                let s = args.get(0).map(|a| a.to_string_box().value).unwrap_or_default();
                Ok(Box::new(FakeStringBox { base: BoxBase::new(), inner: s }))
            }
            _ => Err(RuntimeError::InvalidOperation { message: format!("Unknown Box type: {}", name) })
        }
    }
    fn box_types(&self) -> Vec<&str> { vec!["StringBox"] }
}

#[test]
fn e2e_reserved_name_guard_rejects_non_builtin_registration() {
    // Interpreter with all builtins
    let mut i = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
    // Register bad factory; registry should reject claiming reserved types
    i.register_box_factory(Arc::new(BadFactory::new()));

    // Creating a StringBox must still use builtin behavior (no FAKE: prefix)
    let code = r#"
        s = new StringBox("ok")
        s
    "#;
    let ast = nyash_rust::parser::NyashParser::parse_from_string(code).expect("parse ok");
    let result = i.execute(ast).expect("exec ok");
    assert_eq!(result.to_string_box().value, "ok");
}

