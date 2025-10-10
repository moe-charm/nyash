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
        // ProviderBox first: single boundary for Plugin→Registry→Embedded resolution
        {
            let mut converted_pb: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
            for vid in args { converted_pb.push(self.reg_load(*vid)?.to_nyash_box()); }
            crate::runtime::provider_box::ensure_loaded(None);
            if let Ok(created) = crate::runtime::provider_box::new_box(box_type, &converted_pb) {
                let created_vm = VMValue::from_nyash_box(created);
                self.regs.insert(dst, created_vm.clone());
                if let VMValue::BoxRef(arc_box) = &created_vm { self.scope.register_box(arc_box.clone()); }
                crate::backend::mir_interpreter::helpers::lifecycle_contracts_box::LifecycleContractsBox::mark_new(self, dst, box_type, args.len());
                // Always attempt birth; ignore missing method
                let _ = self.handle_box_call(None, dst, "birth", args);
                crate::backend::mir_interpreter::helpers::lifecycle_contracts_box::LifecycleContractsBox::born_if_no_birth(self, dst, box_type, args.len());
                return Ok(());
            }
        }
        // ProviderBox path above is the single source of truth for Plugin→Registry→Embedded.

        // Forbid instantiation of flow/static entry boxes (dev guard):
        // Heuristic: when flow is enabled and a `<Class>.main/0` exists but there is
        // no `<Class>.birth/N`, treat the class as a flow/static box and reject `new`.
        if let Some(msg) = crate::guards::entry_guard::forbid_new(box_type) {
            return Err(VMError::InvalidInstruction(msg));
        }
// User-defined Box fast-path: when module contains methods/ctors for this
        // class (e.g., "Class.birth/N"), construct an InstanceBox directly to
        // avoid builtin shadowing (ArrayBox/MapBox reserved types).
        let has_user_birth = self
            .functions
            .contains_key(&format!("{}.birth/{}", box_type, args.len()));
        let has_user_any = if has_user_birth {
            true
        } else {
            // Light probe: any function starting with "Class." indicates a user box
            self.functions
                .keys()
                .any(|k| k.starts_with(&format!("{}.", box_type)))
        };
        if has_user_any {
            let inst = crate::instance_v2::InstanceBox::new(
                box_type.to_string(),
                Vec::new(),
                std::collections::HashMap::new(),
            );
            let created_vm = VMValue::from_nyash_box(Box::new(inst));
            self.regs.insert(dst, created_vm.clone());
            if let VMValue::BoxRef(arc_box) = &created_vm {
                self.scope.register_box(arc_box.clone());
            }
            self.lifecycle_observe_new(dst, box_type, args.len());
            // Invoke birth() via instance path (idempotent for user boxes by policy)
            let _ = self.handle_box_call(None, dst, "birth", args);
            return Ok(());
        }
        let mut converted: Vec<Box<dyn NyashBox>> = Vec::with_capacity(args.len());
        for vid in args {
            converted.push(self.reg_load(*vid)?.to_nyash_box());
        }
        // plugin-on では unified_registry（builtin）へのフォールバックを抑止して一貫化
        let plugin_on = crate::runtime::env_gate_box::plugin_policy_on() && !crate::runtime::env_gate_box::plugins_disabled();
        if plugin_on {
            return Err(VMError::InvalidInstruction(format!(
                "NewBox {} failed: plugin-on policy forbids builtin fallback",
                box_type
            )));
        }
        let reg = crate::runtime::unified_registry::get_global_unified_registry();
        let created = reg.lock().unwrap().create_box(box_type, &converted).map_err(|e| {
            VMError::InvalidInstruction(format!("NewBox {} failed: {}", box_type, e))
        })?;
        // Store created instance first so 'me' can be passed to birth
        let created_vm = VMValue::from_nyash_box(created);
        self.regs.insert(dst, created_vm.clone());
        // Register BoxRef into current function scope for fini-on-scope-exit
        if let VMValue::BoxRef(arc_box) = &created_vm {
            self.scope.register_box(arc_box.clone());
        }

        // Centralized lifecycle observation (contracts + traces)
        crate::backend::mir_interpreter::helpers::lifecycle_contracts_box::LifecycleContractsBox::mark_new(self, dst, box_type, args.len());

        // Always attempt birth after creation; ignore missing method to keep no-op semantics.
        let _ = self.handle_box_call(None, dst, "birth", args);

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
        crate::backend::mir_interpreter::helpers::lifecycle_contracts_box::LifecycleContractsBox::born_if_no_birth(self, dst, box_type, args.len());
Ok(())
    }
}
