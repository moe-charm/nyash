// Function lowering for method bodies
use super::super::{MirBuilder, MirFunction, MirInstruction, MirType, ValueId};
use crate::ast::ASTNode;
use crate::mir::builder::calls::function_lowering;

impl MirBuilder {
    // Lower a box method into a standalone MIR function (with `me` parameter)
    pub(in super::super) fn lower_method_as_function(
        &mut self,
        func_name: String,
        box_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        let signature = function_lowering::prepare_method_signature(
            func_name,
            &box_name,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        // Per-function metadata scope: ValueId は関数ごとにリセットされるため、
        // value_types / value_origin_newbox は関数単位で分離する（交差汚染防止）。
        let saved_types = std::mem::take(&mut self.value_types);
        let saved_origin = self.origin_snapshot();
        let saved_value_gen = self.value_gen.clone();
        let saved_singletons = std::mem::take(&mut self.current_fn_singletons);

        let result = (|| -> Result<(), String> {
            self.value_gen.reset();
            self.current_function = Some(function);
            self.current_block = Some(entry);
            self.ensure_block_exists(entry)?;
            let mut me_origin: Option<ValueId> = None;
            if let Some(ref mut f) = self.current_function {
                f.metadata
                    .optimization_hints
                    .push("static_singleton_me".to_string());
                let me_id = self.value_gen.next();
                me_origin = Some(me_id);
                f.params.push(me_id);
                self.variable_map.insert("me".to_string(), me_id);
                for p in &params {
                    let pid = self.value_gen.next();
                    f.params.push(pid);
                    self.variable_map.insert(p.clone(), pid);
                }
            }
            if let Some(me_id) = me_origin {
                self.origin_register(me_id, box_name.clone());
            }
            let program_ast = function_lowering::wrap_in_program(body);
            let _last = self.build_expression(program_ast)?;
            if !returns_value && !self.is_current_block_terminated() {
                let void_val = crate::mir::builder::emission::constant::emit_void(self);
                self.emit_instruction(MirInstruction::Return {
                    value: Some(void_val),
                })?;
            }
            if let Some(ref mut f) = self.current_function {
                if returns_value
                    && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
                {
                    let mut inferred: Option<MirType> = None;
                    'search: for (_bid, bb) in f.blocks.iter() {
                        for inst in bb.instructions.iter() {
                            if let MirInstruction::Return { value: Some(v) } = inst {
                                if let Some(mt) = self.value_types.get(v).cloned() {
                                    inferred = Some(mt);
                                    break 'search;
                                }
                            }
                        }
                        if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break;
                            }
                        }
                    }
                    if let Some(mt) = inferred {
                        f.signature.return_type = mt;
                    }
                }
            }
            let finalized_function = self.current_function.take().unwrap();
            if let Some(ref mut module) = self.current_module {
                module.add_function(finalized_function);
            }
            Ok(())
        })();

        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        // Drop per-function metadata and restore outer scope
        self.value_types = saved_types;
        self.origin_restore(saved_origin);
        self.value_gen = saved_value_gen;
        self.current_fn_singletons = saved_singletons;
        result
    }

    // Lower a static method body into a standalone MIR function (no `me` parameter)
    pub(in super::super) fn lower_static_method_as_function(
        &mut self,
        func_name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    ) -> Result<(), String> {
        // Derive static box context from function name prefix, e.g., "BoxName.method/N"
        let saved_static_ctx = self.current_static_box.clone();
        let mut static_box_name: Option<String> = None;
        if let Some(pos) = func_name.find('.') {
            let box_name = &func_name[..pos];
            if !box_name.is_empty() {
                // Canonicalize alias-alias (X_X -> X)
                let canon = if let Some((a, b)) = box_name.split_once('_') {
                    if a == b { a } else { box_name }
                } else {
                    box_name
                };
                let canon = canon.to_string();
                self.current_static_box = Some(canon.clone());
                static_box_name = Some(canon);
            }
        }
        let box_name_for_signature = static_box_name
            .clone()
            .or_else(|| self.current_static_box.clone())
            .unwrap_or_else(|| "StaticBox".to_string());
        let signature = function_lowering::prepare_static_method_signature(
            func_name,
            &box_name_for_signature,
            &params,
            &body,
        );
        let returns_value = !matches!(signature.return_type, MirType::Void);
        let entry = self.block_gen.next();
        let function = MirFunction::new(signature, entry);
        let saved_function = self.current_function.take();
        let saved_block = self.current_block.take();
        let saved_var_map = std::mem::take(&mut self.variable_map);
        let saved_types = std::mem::take(&mut self.value_types);
        let saved_origin = self.origin_snapshot();
        let saved_value_gen = self.value_gen.clone();
        let saved_singletons = std::mem::take(&mut self.current_fn_singletons);

        let result = (|| -> Result<(), String> {
            self.value_gen.reset();
            self.current_function = Some(function);
            self.current_block = Some(entry);
            self.ensure_block_exists(entry)?;
            let mut me_origin: Option<ValueId> = None;
            if let Some(ref mut f) = self.current_function {
                let me_id = self.value_gen.next();
                me_origin = Some(me_id);
                f.params.push(me_id);
                self.variable_map.insert("me".to_string(), me_id);
                for p in &params {
                    let pid = self.value_gen.next();
                    f.params.push(pid);
                    self.variable_map.insert(p.clone(), pid);
                }
            }
            if let (Some(me_id), Some(ref cls)) = (me_origin, static_box_name.as_ref()) {
                self.origin_register(me_id, cls.to_string());
            }
            let program_ast = function_lowering::wrap_in_program(body);
            let _last = self.build_expression(program_ast)?;
            if !returns_value {
                if let Some(ref mut f) = self.current_function {
                    if let Some(block) = f.get_block(self.current_block.unwrap()) {
                        if !block.is_terminated() {
                            let void_val = crate::mir::builder::emission::constant::emit_void(self);
                            self.emit_instruction(MirInstruction::Return {
                                value: Some(void_val),
                            })?;
                        }
                    }
                }
            }
            if let Some(ref mut f) = self.current_function {
                if returns_value
                    && matches!(f.signature.return_type, MirType::Void | MirType::Unknown)
                {
                    let mut inferred: Option<MirType> = None;
                    'search: for (_bid, bb) in f.blocks.iter() {
                        for inst in bb.instructions.iter() {
                            if let MirInstruction::Return { value: Some(v) } = inst {
                                if let Some(mt) = self.value_types.get(v).cloned() {
                                    inferred = Some(mt);
                                    break 'search;
                                }
                            }
                        }
                        if let Some(MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                            if let Some(mt) = self.value_types.get(v).cloned() {
                                inferred = Some(mt);
                                break;
                            }
                        }
                    }
                    if let Some(mt) = inferred {
                        f.signature.return_type = mt;
                    }
                }
            }
            let finalized = self.current_function.take().unwrap();
            if let Some(ref mut module) = self.current_module {
                module.add_function(finalized);
            }
            Ok(())
        })();

        self.current_function = saved_function;
        self.current_block = saved_block;
        self.variable_map = saved_var_map;
        self.value_types = saved_types;
        self.origin_restore(saved_origin);
        self.value_gen = saved_value_gen;
        self.current_fn_singletons = saved_singletons;
        self.current_static_box = saved_static_ctx;
        result
    }
}
