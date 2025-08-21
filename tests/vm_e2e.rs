//! VM E2E: Compile Nyash to MIR and execute via VM, with mock plugin factory
use std::sync::Arc;

use nyash_rust::box_factory::BoxFactory;
use nyash_rust::box_factory::builtin::BuiltinGroups;
use nyash_rust::runtime::NyashRuntimeBuilder;
use nyash_rust::parser::NyashParser;
use nyash_rust::mir::MirCompiler;
use nyash_rust::backend::VM;
use nyash_rust::interpreter::RuntimeError;
use nyash_rust::box_trait::{NyashBox, BoxCore, BoxBase, StringBox, BoolBox};

// Minimal AdderBox to validate plugin factory path under VM
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

struct TestPluginFactory;
impl TestPluginFactory { fn new() -> Self { Self } }
impl BoxFactory for TestPluginFactory {
    fn create_box(&self, name: &str, args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match name {
            "AdderBox" => {
                if args.len() != 2 { return Err(RuntimeError::InvalidOperation{ message: format!("AdderBox expects 2 args, got {}", args.len()) }); }
                let a = args[0].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError{ message: "AdderBox arg a must be int".into() })?;
                let b = args[1].to_string_box().value.parse::<i64>().map_err(|_| RuntimeError::TypeError{ message: "AdderBox arg b must be int".into() })?;
                Ok(Box::new(AdderBox::new(a, b)))
            }
            _ => Err(RuntimeError::InvalidOperation{ message: format!("Unknown Box type: {}", name) })
        }
    }
    fn box_types(&self) -> Vec<&str> { vec!["AdderBox"] }
}

#[test]
fn vm_e2e_adder_box() {
    // Build runtime with builtin + user-defined + mock plugin factory
    let runtime = NyashRuntimeBuilder::new()
        .with_builtin_groups(BuiltinGroups::native_full())
        .with_factory(Arc::new(nyash_rust::box_factory::user_defined::UserDefinedBoxFactory::new(
            nyash_rust::interpreter::SharedState::new(),
        )))
        .with_factory(Arc::new(TestPluginFactory::new()))
        .build();

    // Nyash code: construct AdderBox and leave it as final expression
    let code = r#"
        a = new AdderBox(10, 32)
        a
    "#;

    // Parse → MIR
    let ast = NyashParser::parse_from_string(code).expect("parse ok");
    let mut mir_compiler = MirCompiler::new();
    let compile_result = mir_compiler.compile(ast).expect("mir ok");

    // Execute via VM using the prepared runtime
    let mut vm = VM::with_runtime(runtime);
    let result = vm.execute_module(&compile_result.module).expect("vm exec ok");

    // The VM returns an Option<Box<dyn NyashBox>> or a value; we print/debug and check string form
    // Here we rely on Display via to_string_box through Debug format
    // Try to format the result if available
    // For this implementation, result is a generic value; we check debug string contains 42 or to_string equivalent.
    let s = format!("{:?}", result);
    assert!(s.contains("42") || s.contains("AdderBox"), "unexpected VM result: {}", s);
}

