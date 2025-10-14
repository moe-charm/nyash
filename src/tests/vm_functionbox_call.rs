#![cfg(feature = "legacy-boxes")]
use crate::mir::{MirModule, MirFunction, FunctionSignature, MirInstruction, EffectMask, BasicBlockId, ConstValue};
use crate::backend::VM;
use crate::backend::vm::VMValue;
use crate::boxes::function_box::{FunctionBox, ClosureEnv};
use crate::box_trait::NyashBox;

#[test]
fn vm_call_functionbox_returns_42() {
    // Build FunctionBox: function(a) { return a + 1 }
    let params = vec!["a".to_string()];
    let body = vec![
        crate::ast::ASTNode::Return {
            value: Some(Box::new(crate::ast::ASTNode::BinaryOp {
                left: Box::new(crate::ast::ASTNode::Variable { name: "a".to_string(), span: crate::ast::Span::unknown() }),
                operator: crate::ast::BinaryOperator::Add,
                right: Box::new(crate::ast::ASTNode::Literal { value: crate::ast::LiteralValue::Integer(1), span: crate::ast::Span::unknown() }),
                span: crate::ast::Span::unknown(),
            })),
            span: crate::ast::Span::unknown(),
        }
    ];
    let fun = FunctionBox::with_env(params, body, ClosureEnv::new());

    // Build MIR: arg=41; res = call func_id(arg); return res
    let sig = FunctionSignature { name: "main".into(), params: vec![], return_type: crate::mir::MirType::Integer, effects: EffectMask::PURE };
    let mut f = MirFunction::new(sig, BasicBlockId::new(0));
    let bb = f.entry_block;
    // Reserve an id for function value (we'll inject VMValue::BoxRef later)
    let func_id = f.next_value_id();
    // arg const
    let arg = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Const { dst: arg, value: ConstValue::Integer(41) });
    // call
    let res = f.next_value_id();
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Call { dst: Some(res), func: func_id, callee: Some(crate::mir::definitions::Callee::Value(func_id)), args: vec![arg], effects: EffectMask::PURE });
    f.get_block_mut(bb).unwrap().add_instruction(MirInstruction::Return { value: Some(res) });

    let mut m = MirModule::new("vm_funbox".into());
    m.add_function(f.clone());

    // Prepare VM and inject FunctionBox into func_id
    let mut vm = VM::new();
    let arc_fun: std::sync::Arc<dyn NyashBox> = std::sync::Arc::from(Box::new(fun) as Box<dyn NyashBox>);
    vm.set_value(func_id, VMValue::BoxRef(arc_fun));

    let out = vm.execute_module(&m).expect("vm exec");
    assert_eq!(out.to_string_box().value, "42");
}
