//! E2E test for unified registry with a mock plugin factory
use std::sync::Arc;

use nyash_rust::box_factory::BoxFactory;
use nyash_rust::box_factory::builtin::BuiltinGroups;
use nyash_rust::interpreter::{NyashInterpreter, SharedState, RuntimeError};
use nyash_rust::runtime::NyashRuntimeBuilder;
use nyash_rust::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};

// ---------- Mock plugin boxes ----------

#[derive(Debug, Clone)]
struct EchoBox { base: BoxBase, msg: String }

impl EchoBox { fn new(msg: String) -> Self { Self { base: BoxBase::new(), msg } } }

impl BoxCore for EchoBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EchoBox(\"{}\")", self.msg)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for EchoBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(self.msg.clone()) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(e) = other.as_any().downcast_ref::<EchoBox>() { BoolBox::new(self.msg == e.msg) } else { BoolBox::new(false) }
    }
    fn type_name(&self) -> &'static str { "EchoBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
}

#[derive(Debug, Clone)]
struct AdderBox { base: BoxBase, sum: i64 }
impl AdderBox { fn new(a: i64, b: i64) -> Self { Self { base: BoxBase::new(), sum: a + b } } }

impl BoxCore for AdderBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "AdderBox(sum={})", self.sum) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for AdderBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(self.sum.to_string()) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(a) = other.as_any().downcast_ref::<AdderBox>() { BoolBox::new(self.sum == a.sum) } else { BoolBox::new(false) }
    }
    fn type_name(&self) -> &'static str { "AdderBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
    fn share_box(&self) -> Box<dyn NyashBox> { Box::new(self.clone()) }
}

// ---------- Mock plugin factory ----------

struct TestPluginFactory;
impl TestPluginFactory { fn new() -> Self { Self } }

impl BoxFactory for TestPluginFactory {
    fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match name {
            "EchoBox" => {
                let msg = args.get(0).map(|a| a.to_string_box().value).unwrap_or_else(|| "".to_string());
                Ok(Box::new(EchoBox::new(msg)))
            }
            "AdderBox" => {
                if args.len() != 2 { return Err(RuntimeError::InvalidOperation{ message: format!("AdderBox expects 2 args, got {}", args.len()) }); }
                let a = args[0].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError{ message: "AdderBox arg a must be int".into() })?;
                let b = args[1].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError{ message: "AdderBox arg b must be int".into() })?;
                Ok(Box::new(AdderBox::new(a, b)))
            }
            _ => Err(RuntimeError::InvalidOperation{ message: format!("Unknown Box type: {}", name) })
        }
    }

    fn box_types(&self) -> Vec<&str> { vec!["EchoBox", "AdderBox"] }
}

// ---------- E2E tests ----------

fn build_interpreter_with_test_plugin() -> NyashInterpreter {
    // Start with a standard interpreter (native_full)
    let mut interp = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
    // Inject our mock plugin factory into the interpreter's private registry
    interp.register_box_factory(Arc::new(TestPluginFactory::new()));
    interp
}

#[test]
fn e2e_create_echo_box_and_return_string() {
    let mut i = build_interpreter_with_test_plugin();
    let code = r#"
        e = new EchoBox("hi")
        e
    "#;
    let ast = nyash_rust::parser::NyashParser::parse_from_string(code).expect("parse ok");
    let result = i.execute(ast).expect("exec ok");
    assert_eq!(result.to_string_box().value, "hi");
}

#[test]
fn e2e_create_adder_box_and_return_sum() {
    let mut i = build_interpreter_with_test_plugin();
    let code = r#"
        a = new AdderBox(10, 32)
        a
    "#;
    let ast = nyash_rust::parser::NyashParser::parse_from_string(code).expect("parse ok");
    let result = i.execute(ast).expect("exec ok");
    assert_eq!(result.to_string_box().value, "42");
}
