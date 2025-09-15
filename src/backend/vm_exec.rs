/*!
 * VM Execution Loop (extracted from vm.rs)
 *
 * Contains the high-level execution entrypoints and per-instruction dispatch glue:
 * - VM::execute_module
 * - VM::execute_function
 * - VM::call_function_by_name
 * - VM::execute_instruction (delegates to backend::dispatch)
 * - VM::print_cache_stats_summary (stats helper)
 *
 * Behavior and public APIs are preserved. This is a pure move/refactor.
 */

use crate::mir::{MirModule, MirFunction, MirInstruction, BasicBlockId};
use crate::box_trait::NyashBox;
use super::{vm::VM, vm::VMError, vm::VMValue};
use crate::backend::vm_control_flow::ControlFlow;

impl VM {
    /// Execute a MIR module
    pub fn execute_module(&mut self, module: &MirModule) -> Result<Box<dyn NyashBox>, VMError> {
        self.module = Some(module.clone());
        if std::env::var("NYASH_REG_DUMP").ok().as_deref() == Some("1") {
            crate::runtime::type_meta::dump_registry();
        }
        self.instr_counter.clear();
        self.exec_start = Some(std::time::Instant::now());
        let main_function = module
            .get_function("main")
            .ok_or_else(|| VMError::InvalidInstruction("No main function found".to_string()))?;
        let result = self.execute_function(main_function)?;
        self.maybe_print_stats();
        self.maybe_print_jit_unified_stats();
        if crate::config::env::vm_pic_stats() {
            self.print_cache_stats_summary();
        }
        if let Some(jm) = &self.jit_manager { jm.print_summary(); }
        {
            let lvl = crate::config::env::gc_trace_level();
            if lvl > 0 {
                if let Some((sp, rd, wr)) = self.runtime.gc.snapshot_counters() {
                    eprintln!("[GC] counters: safepoints={} read_barriers={} write_barriers={}", sp, rd, wr);
                }
                let roots_total = self.scope_tracker.root_count_total();
                let root_regions = self.scope_tracker.root_regions();
                let field_slots: usize = self.object_fields.values().map(|m| m.len()).sum();
                eprintln!(
                    "[GC] mock_mark: roots_total={} regions={} object_field_slots={}",
                    roots_total, root_regions, field_slots
                );
                if lvl >= 2 { self.gc_print_roots_breakdown(); }
                if lvl >= 3 { self.gc_print_reachability_depth2(); }
            }
        }
        Ok(result.to_nyash_box())
    }

    pub(super) fn print_cache_stats_summary(&self) {
        let sites_poly = self.boxcall_poly_pic.len();
        let entries_poly: usize = self.boxcall_poly_pic.values().map(|v| v.len()).sum();
        let avg_entries = if sites_poly > 0 {
            (entries_poly as f64) / (sites_poly as f64)
        } else {
            0.0
        };
        let sites_mono = self.boxcall_pic_funcname.len();
        let hits_total: u64 = self.boxcall_pic_hits.values().map(|v| *v as u64).sum();
        let vt_entries = self.boxcall_vtable_funcname.len();
        eprintln!(
            "[VM] PIC/VT summary: poly_sites={} avg_entries={:.2} mono_sites={} hits_total={} vt_entries={} | hits: vt={} poly={} mono={} generic={}",
            sites_poly,
            avg_entries,
            sites_mono,
            hits_total,
            vt_entries,
            self.boxcall_hits_vtable,
            self.boxcall_hits_poly_pic,
            self.boxcall_hits_mono_pic,
            self.boxcall_hits_generic
        );
        let mut hits: Vec<(&String, &u32)> = self.boxcall_pic_hits.iter().collect();
        hits.sort_by(|a, b| b.1.cmp(a.1));
        for (i, (k, v)) in hits.into_iter().take(5).enumerate() {
            eprintln!("  #{} {} hits={}", i + 1, k, v);
        }
    }

    /// Call a MIR function by name with VMValue arguments
    pub(super) fn call_function_by_name(
        &mut self,
        func_name: &str,
        args: Vec<VMValue>,
    ) -> Result<VMValue, VMError> {
        self.enter_root_region();
        self.pin_roots(args.iter());
        let module_ref = self
            .module
            .as_ref()
            .ok_or_else(|| VMError::InvalidInstruction("No active module".to_string()))?;
        let function_ref = module_ref
            .get_function(func_name)
            .ok_or_else(|| VMError::InvalidInstruction(format!("Function '{}' not found", func_name)))?;
        let function = function_ref.clone();

        let saved_values = std::mem::take(&mut self.values);
        let saved_current_function = self.current_function.clone();
        let saved_current_block = self.frame.current_block;
        let saved_previous_block = self.previous_block;
        let saved_pc = self.frame.pc;
        let saved_last_result = self.frame.last_result;

        for (i, param_id) in function.params.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                self.set_value(*param_id, arg.clone());
            }
        }
        if let Some(first) = function.params.get(0) {
            if let Some((class_part, _rest)) = func_name.split_once('.') {
                self.object_class.insert(*first, class_part.to_string());
                self.object_internal.insert(*first);
            }
        }

        let result = self.execute_function(&function);

        self.values = saved_values;
        self.current_function = saved_current_function;
        self.frame.current_block = saved_current_block;
        self.previous_block = saved_previous_block;
        self.frame.pc = saved_pc;
        self.frame.last_result = saved_last_result;
        self.scope_tracker.leave_root_region();
        result
    }

    /// Execute a single function
    pub(super) fn execute_function(&mut self, function: &MirFunction) -> Result<VMValue, VMError> {
        use crate::box_trait::{StringBox, IntegerBox, BoolBox, VoidBox};
        use crate::runtime::global_hooks;
        use crate::instance_v2::InstanceBox;
        use super::control_flow;

        self.current_function = Some(function.signature.name.clone());
        if let Some(jm) = &mut self.jit_manager {
            if let Ok(s) = std::env::var("NYASH_JIT_THRESHOLD") {
                if let Ok(t) = s.parse::<u32>() { if t > 0 { jm.set_threshold(t); } }
            }
            jm.record_entry(&function.signature.name);
            let _ = jm.maybe_compile(&function.signature.name, function);
            if jm.is_compiled(&function.signature.name)
                && std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1")
            {
                if let Some(h) = jm.handle_of(&function.signature.name) {
                    eprintln!(
                        "[JIT] dispatch would go to handle={} for {} (stub)",
                        h, function.signature.name
                    );
                }
            }
        }

        self.loop_executor.initialize();
        self.scope_tracker.push_scope();
        global_hooks::push_task_scope();

        let args_vec: Vec<VMValue> = function
            .params
            .iter()
            .filter_map(|pid| self.get_value(*pid).ok())
            .collect();
        if std::env::var("NYASH_JIT_EXEC").ok().as_deref() == Some("1") {
            let jit_only = std::env::var("NYASH_JIT_ONLY").ok().as_deref() == Some("1");
            self.enter_root_region();
            self.pin_roots(args_vec.iter());
            if let Some(compiled) = self
                .jit_manager
                .as_ref()
                .map(|jm| jm.is_compiled(&function.signature.name))
            {
                if compiled {
                    crate::runtime::host_api::set_current_vm(self as *mut _);
                    let jit_val = if let Some(jm_mut) = self.jit_manager.as_mut() {
                        jm_mut.execute_compiled(
                            &function.signature.name,
                            &function.signature.return_type,
                            &args_vec,
                        )
                    } else {
                        None
                    };
                    crate::runtime::host_api::clear_current_vm();
                    if let Some(val) = jit_val {
                        self.leave_root_region();
                        self.scope_tracker.pop_scope();
                        global_hooks::pop_task_scope();
                        return Ok(val);
                    } else if std::env::var("NYASH_JIT_STATS").ok().as_deref() == Some("1")
                        || std::env::var("NYASH_JIT_TRAP_LOG").ok().as_deref() == Some("1")
                    {
                        eprintln!("[JIT] fallback: VM path taken for {}", function.signature.name);
                        if jit_only {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            global_hooks::pop_task_scope();
                            return Err(VMError::InvalidInstruction(format!(
                                "JIT-only enabled and JIT trap occurred for {}",
                                function.signature.name
                            )));
                        }
                    }
                } else if jit_only {
                    if let Some(jm_mut) = self.jit_manager.as_mut() {
                        let _ = jm_mut.maybe_compile(&function.signature.name, function);
                    }
                    if self
                        .jit_manager
                        .as_ref()
                        .map(|jm| jm.is_compiled(&function.signature.name))
                        .unwrap_or(false)
                    {
                        crate::runtime::host_api::set_current_vm(self as *mut _);
                        let jit_val = if let Some(jm_mut) = self.jit_manager.as_mut() {
                            jm_mut.execute_compiled(
                                &function.signature.name,
                                &function.signature.return_type,
                                &args_vec,
                            )
                        } else {
                            None
                        };
                        crate::runtime::host_api::clear_current_vm();
                        if let Some(val) = jit_val {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            global_hooks::pop_task_scope();
                            return Ok(val);
                        } else {
                            self.leave_root_region();
                            self.scope_tracker.pop_scope();
                            global_hooks::pop_task_scope();
                            return Err(VMError::InvalidInstruction(format!(
                                "JIT-only mode: compiled execution failed for {}",
                                function.signature.name
                            )));
                        }
                    } else {
                        self.leave_root_region();
                        self.scope_tracker.pop_scope();
                        global_hooks::pop_task_scope();
                        return Err(VMError::InvalidInstruction(format!(
                            "JIT-only mode: function {} not compiled",
                            function.signature.name
                        )));
                    }
                }
            }
        }

        let mut current_block = function.entry_block;
        self.frame.current_block = Some(current_block);
        self.frame.pc = 0;
        let mut should_return: Option<VMValue> = None;
        let mut next_block: Option<BasicBlockId> = None;

        loop {
            // Reset per-block control-flow decisions to avoid carrying over stale state
            // from a previous block (which could cause infinite loops on if/return).
            should_return = None;
            next_block = None;
            if let Some(block) = function.blocks.get(&current_block) {
                for instruction in &block.instructions {
                    match self.execute_instruction(instruction)? {
                        ControlFlow::Continue => continue,
                        ControlFlow::Jump(target) => {
                            next_block = Some(target);
                            break;
                        }
                        ControlFlow::Return(value) => {
                            should_return = Some(value);
                            break;
                        }
                    }
                }
                // Execute terminator if present and we haven't decided control flow yet
                if should_return.is_none() && next_block.is_none() {
                    if let Some(term) = &block.terminator {
                        match self.execute_instruction(term)? {
                            ControlFlow::Continue => {},
                            ControlFlow::Jump(target) => { next_block = Some(target); },
                            ControlFlow::Return(value) => { should_return = Some(value); },
                        }
                    }
                }
            } else {
                return Err(VMError::InvalidBasicBlock(format!(
                    "Basic block {:?} not found",
                    current_block
                )));
            }

            if let Some(return_value) = should_return {
                self.scope_tracker.pop_scope();
                global_hooks::pop_task_scope();
                return Ok(return_value);
            } else if let Some(target) = next_block {
                control_flow::record_transition(
                    &mut self.previous_block,
                    &mut self.loop_executor,
                    current_block,
                    target,
                )
                .ok();
                current_block = target;
            } else {
                self.scope_tracker.pop_scope();
                global_hooks::pop_task_scope();
                return Ok(VMValue::Void);
            }
        }
    }

    /// Execute a single instruction
    pub(super) fn execute_instruction(&mut self, instruction: &MirInstruction) -> Result<ControlFlow, VMError> {
        let debug_global = std::env::var("NYASH_VM_DEBUG").ok().as_deref() == Some("1");
        let debug_exec = debug_global || std::env::var("NYASH_VM_DEBUG_EXEC").ok().as_deref() == Some("1");
        if debug_exec { eprintln!("[VM] execute_instruction: {:?}", instruction); }
        self.record_instruction(instruction);
        super::dispatch::execute_instruction(self, instruction, debug_global)
    }
}
