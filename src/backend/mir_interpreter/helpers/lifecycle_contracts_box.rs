//! LifecycleContractsBox — VM-side facade to unify contract emissions
//!
//! Responsibility
//! - Single entry for NewBox/birth contract observation and default born-no-birth logic.
//! - Keeps logic out of call-sites; no behavior change.

use super::super::*;

pub struct LifecycleContractsBox;

impl LifecycleContractsBox {
    pub fn mark_new(vm: &mut MirInterpreter, dst: ValueId, box_type: &str, argc: usize) {
        vm.lifecycle_observe_new(dst, box_type, argc);
    }

    pub fn mark_birth(vm: &mut MirInterpreter, recv_val: ValueId, argc_birth: usize) {
        vm.lifecycle_contracts_birth(recv_val, argc_birth);
    }

    /// If no birth/N lowered function exists, mark born immediately and emit guard log when enabled.
    pub fn born_if_no_birth(vm: &mut MirInterpreter, dst: ValueId, box_type: &str, argc: usize) {
        let birth_name = format!("{}.birth/{}", box_type, argc);
        if !vm.functions.contains_key(&birth_name) {
            let key = vm.object_key_for(dst);
            vm.contracts_born.insert(key);
            if crate::config::env::check_contracts() {
                eprintln!(
                    "{{\"kind\":\"contracts_born_nobirth\",\"class\":\"{}\",\"key\":{}}}",
                    box_type, key
                );
            }
        }
    }
}

