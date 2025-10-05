//! newbox.rs — NewBox handler extraction

use super::super::*;
use crate::box_trait::NyashBox;

impl MirInterpreter {
    pub(crate) fn handle_new_box(
        &mut self,
        dst: ValueId,
        box_type: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // Provider Lock guard (受け口・既定は挙動不変)
        if let Err(e) = crate::runtime::provider_lock::guard_before_new_box(box_type) {
            return Err(VMError::InvalidInstruction(e));
        }
        let mut converted: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for vid in args {
            converted.push(self.reg_load(*vid)?.to_nyash_box());
        }
        let reg = crate::runtime::unified_registry::get_global_unified_registry();
        let created = reg
            .lock()
            .unwrap()
            .create_box(box_type, &converted)
            .map_err(|e| {
                VMError::InvalidInstruction(format!("NewBox {} failed: {}", box_type, e))
            })?;
        // Store created instance first so 'me' can be passed to birth
        let created_vm = VMValue::from_nyash_box(created);
        self.regs.insert(dst, created_vm.clone());

        // Centralized lifecycle observation (contracts + traces)
        self.lifecycle_observe_new(dst, box_type, args.len());

        // Dev-only: optional auto birth after NewBox to unblock selfhost paths
        // Guarded by NYASH_VM_AUTO_BIRTH_DEV=1. In production, builders must
        // materialize explicit birth calls.
        if super::super::VmConfig::global().auto_birth_dev {
            // Dev: call birth with the same args that were provided to NewBox
            // This covers user-defined boxes that rely on birth parameters
            let _ = self.handle_box_call(None, dst, "birth", args);
        }

        // C++-style constructor mode (interim): optionally invoke ModuleFunction
        // "Class.birth/N" immediately after NewBox, using fully qualified name.
        // Guarded by NYASH_VM_AUTO_BIRTH_CPP=1. This simulates the future MIR
        // NewBox{auto_birth} semantics without changing the MIR yet.
        //
        // Everything is Box: unified lifecycle rule for all boxes (core/plugin/user).
        // If birth/N exists in function table → call it. Otherwise → no-op.
        // No hardcoded special rules for built-in boxes.
        if super::super::VmConfig::global().auto_birth_cpp {
            // Compose fully-qualified birth name and invoke via ModuleFunction path.
            // me + args
            let mut bargs: Vec<super::super::ValueId> = Vec::with_capacity(1 + args.len());
            bargs.push(dst);
            bargs.extend(args.iter().copied());
            let name = format!("{}.birth/{}", box_type, args.len());
            if self.functions.contains_key(&name) {
                let _ = self.handle_callee_module_function(&name, &bargs);
            } else {
                // No such birth function; treat as no-op (applies to built-in boxes)
            }
        }

        // Note: productionでは birth の自動呼び出しは行わない。
        // 正しい設計は Builder が NewBox 後に明示的に birth 呼び出しを生成すること。
        
        // Contracts: if no birth method exists globally, mark as born immediately
        // to satisfy lifecycle for builtin/plugin boxes that don't require birth.
        let birth_name = format!("{}.birth/{}", box_type, args.len());
        if !self.functions.contains_key(&birth_name) {
            let key = self.object_key_for(dst);
            self.contracts_born.insert(key);
            if crate::config::env::check_contracts() {
                eprintln!(
                    "{{\"kind\":\"contracts_born_nobirth\",\"class\":\"{}\",\"key\":{}}}",
                    box_type, key
                );
            }
        }
Ok(())
    }
}
