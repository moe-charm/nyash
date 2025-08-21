//! E2E: Interop between builtin and mock plugin boxes via MapBox storage
use std::sync::Arc;

use nyash_rust::box_factory::BoxFactory;
use nyash_rust::box_factory::builtin::BuiltinGroups;
use nyash_rust::interpreter::{NyashInterpreter, RuntimeError};
use nyash_rust::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};

#[derive(Debug, Clone)]
struct EchoBox { base: BoxBase, msg: String }
impl EchoBox { fn new(msg: String) -> Self { Self { base: BoxBase::new(), msg } } }

impl BoxCore for EchoBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { None }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "EchoBox(\"{}\")", self.msg) }
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

struct TestPluginFactory;
impl TestPluginFactory { fn new() -> Self { Self } }
impl BoxFactory for TestPluginFactory {
    fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match name {
            "EchoBox" => {
                let msg = args.get(0).map(|a| a.to_string_box().value).unwrap_or_else(|| "".to_string());
                Ok(Box::new(EchoBox::new(msg)))
            }
            _ => Err(RuntimeError::InvalidOperation{ message: format!("Unknown Box type: {}", name) })
        }
    }
    fn box_types(&self) -> Vec<&str> { vec!["EchoBox"] }
}

fn new_interpreter_with_factory() -> NyashInterpreter {
    let mut i = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
    i.register_box_factory(Arc::new(TestPluginFactory::new()));
    i
}

#[test]
fn e2e_interop_mapbox_store_plugin_box() {
    let mut i = new_interpreter_with_factory();
    let code = r#"
        m = new MapBox()
        e = new EchoBox("ok")
        m.set("k", e)
        v = m.get("k")
        v
    "#;
    let ast = nyash_rust::parser::NyashParser::parse_from_string(code).expect("parse ok");
    let result = i.execute(ast).expect("exec ok");
    assert_eq!(result.to_string_box().value, "ok");
}

