#[cfg(feature = "aot-plan-import")]
#[test]
fn import_plan_v1_min_and_run_vm() {
    // Use the embedded minimal plan JSON
    let plan = include_str!("../../tools/aot_plan/samples/plan_v1_min.json");
    let module = crate::mir::aot_plan_import::import_from_str(plan).expect("import plan v1");

    // Execute via VM; expect string "42"
    let mut vm = crate::backend::vm::VM::new();
    let out = vm.execute_module(&module).expect("vm exec");
    assert_eq!(out.to_string_box().value, "42");
}
