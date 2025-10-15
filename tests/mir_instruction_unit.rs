use nyash_rust::mir::{BinaryOp, ConstValue, Effect, EffectMask, MirInstruction, ValueId};

#[test]
fn test_const_instruction() {
    let dst = ValueId::new(0);
    let inst = MirInstruction::Const {
        dst,
        value: ConstValue::Integer(42),
    };
    assert_eq!(inst.dst_value(), Some(dst));
    assert!(inst.used_values().is_empty());
    assert!(inst.effects().is_pure());
}

#[test]
fn test_binop_instruction() {
    let dst = ValueId::new(0);
    let lhs = ValueId::new(1);
    let rhs = ValueId::new(2);
    let inst = MirInstruction::BinOp {
        dst,
        op: BinaryOp::Add,
        lhs,
        rhs,
    };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![lhs, rhs]);
    assert!(inst.effects().is_pure());
}

#[test]
fn test_call_instruction() {
    let dst = ValueId::new(0);
    let func = ValueId::new(1);
    let arg1 = ValueId::new(2);
    let arg2 = ValueId::new(3);
    let inst = MirInstruction::Call {
        dst: Some(dst),
        func,
        callee: None,
        args: vec![arg1, arg2],
        effects: EffectMask::IO,
    };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![func, arg1, arg2]);
    assert_eq!(inst.effects(), EffectMask::IO);
}

#[test]
fn test_ref_new_instruction() {
    let dst = ValueId::new(0);
    let box_val = ValueId::new(1);
    let inst = MirInstruction::RefNew { dst, box_val };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![box_val]);
    assert!(inst.effects().is_pure());
}

#[test]
fn test_ref_get_instruction() {
    let dst = ValueId::new(0);
    let reference = ValueId::new(1);
    let field = "name".to_string();
    let inst = MirInstruction::RefGet {
        dst,
        reference,
        field,
    };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![reference]);
    assert!(!inst.effects().is_pure());
    assert!(inst.effects().contains(Effect::ReadHeap));
}

#[test]
fn test_ref_set_instruction() {
    let reference = ValueId::new(0);
    let field = "value".to_string();
    let value = ValueId::new(1);
    let inst = MirInstruction::RefSet {
        reference,
        field,
        value,
    };
    assert_eq!(inst.dst_value(), None);
    assert_eq!(inst.used_values(), vec![reference, value]);
    assert!(!inst.effects().is_pure());
    assert!(inst.effects().contains(Effect::WriteHeap));
}

#[test]
fn test_weak_new_instruction() {
    let dst = ValueId::new(0);
    let box_val = ValueId::new(1);
    let inst = MirInstruction::WeakNew { dst, box_val };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![box_val]);
    assert!(inst.effects().is_pure());
}

#[test]
fn test_weak_load_instruction() {
    let dst = ValueId::new(0);
    let weak_ref = ValueId::new(1);
    let inst = MirInstruction::WeakLoad { dst, weak_ref };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![weak_ref]);
    assert!(!inst.effects().is_pure());
    assert!(inst.effects().contains(Effect::ReadHeap));
}

#[test]
fn test_barrier_instructions() {
    let ptr = ValueId::new(0);
    let read_barrier = MirInstruction::BarrierRead { ptr };
    assert_eq!(read_barrier.dst_value(), None);
    assert_eq!(read_barrier.used_values(), vec![ptr]);
    assert!(read_barrier.effects().contains(Effect::Barrier));
    assert!(read_barrier.effects().contains(Effect::ReadHeap));

    let write_barrier = MirInstruction::BarrierWrite { ptr };
    assert_eq!(write_barrier.dst_value(), None);
    assert_eq!(write_barrier.used_values(), vec![ptr]);
    assert!(write_barrier.effects().contains(Effect::Barrier));
    assert!(write_barrier.effects().contains(Effect::WriteHeap));
}

#[test]
fn test_extern_call_instruction() {
    let dst = ValueId::new(0);
    let arg1 = ValueId::new(1);
    let arg2 = ValueId::new(2);
    let inst = MirInstruction::ExternCall {
        dst: Some(dst),
        iface_name: "env.console".to_string(),
        method_name: "log".to_string(),
        args: vec![arg1, arg2],
        effects: EffectMask::IO,
    };
    assert_eq!(inst.dst_value(), Some(dst));
    assert_eq!(inst.used_values(), vec![arg1, arg2]);
    assert_eq!(inst.effects(), EffectMask::IO);

    let void_inst = MirInstruction::ExternCall {
        dst: None,
        iface_name: "env.canvas".to_string(),
        method_name: "fillRect".to_string(),
        args: vec![arg1],
        effects: EffectMask::IO,
    };
    assert_eq!(void_inst.dst_value(), None);
    assert_eq!(void_inst.used_values(), vec![arg1]);
}
