use super::{EffectMask, FunctionSignature, MirFunction, MirInstruction, MirModule, MirType, ValueId, BasicBlockId};
use crate::ast::ASTNode;

// Lifecycle routines extracted from builder.rs
impl super::MirBuilder {
    /// Unified declaration indexing (Phase A): collect symbols before lowering
    /// - user_defined_boxes: non-static Box names (for NewBox birth() skip)
    /// - static_method_index: name -> [(BoxName, arity)] (for bare-call fallback)
    fn index_declarations(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program { statements, .. } => {
                for st in statements {
                    self.index_declarations(st);
                }
            }
            ASTNode::BoxDeclaration { name, methods, is_static, .. } => {
                if !*is_static {
                    self.user_defined_boxes.insert(name.clone());
                } else {
                    for (mname, mast) in methods {
                        if let ASTNode::FunctionDeclaration { params, .. } = mast {
                            self.method_index.register_static_method(
                                mname.clone(),
                                name.clone(),
                                params.len()
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub(super) fn prepare_module(&mut self) -> Result<(), String> {
        let module = MirModule::new("main".to_string());
        let main_signature = FunctionSignature {
            name: "main".to_string(),
            params: vec![],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };

        let entry_block = self.block_gen.next();
        let mut main_function = MirFunction::new(main_signature, entry_block);
        main_function.metadata.is_entry_point = true;

        self.current_module = Some(module);
        self.current_function = Some(main_function);
        self.current_block = Some(entry_block);
        self.clear_current_fn_singletons();

        // Hint: scope enter at function entry (id=0 for main)
        self.hint_scope_enter(0);

        if std::env::var("NYASH_BUILDER_SAFEPOINT_ENTRY")
            .ok()
            .as_deref()
            == Some("1")
        {
            self.emit_instruction(MirInstruction::Safepoint)?;
        }

        Ok(())
    }

    pub(super) fn lower_root(&mut self, ast: ASTNode) -> Result<ValueId, String> {
        // Pre-index static methods to enable safe fallback for bare calls in using-prepended code
        let snapshot = ast.clone();
        // Phase A: collect declarations in one pass (symbols available to lowering)
        self.index_declarations(&snapshot);
        // Phase B: top-level program lowering with declaration-first pass
        match ast {
            ASTNode::Program { statements, .. } => {
                use crate::ast::ASTNode as N;
                // First pass: lower declarations (static boxes except Main, and instance boxes)
                let mut main_static: Option<(String, std::collections::HashMap<String, ASTNode>)> = None;
                for st in &statements {
                    if let N::BoxDeclaration {
                        name,
                        methods,
                        is_static,
                        fields,
                        constructors,
                        weak_fields,
                        ..
                    } = st
                    {
                        if *is_static {
                            if name == "Main" {
                                main_static = Some((name.clone(), methods.clone()));
                            } else {
                                // Lower all static methods into standalone functions: BoxName.method/Arity
                                for (mname, mast) in methods.iter() {
                                    if let N::FunctionDeclaration { params, body, .. } = mast {
                                        let func_name = format!("{}.{}{}", name, mname, format!("/{}", params.len()));
                                        self.lower_static_method_as_function(func_name, params.clone(), body.clone())?;
                                        self.method_index.register_static_method(
                                            mname.clone(),
                                            name.clone(),
                                            params.len()
                                        );
                                    }
                                }
                            }
                        } else {
                            // Instance box: register type and lower instance methods/ctors as functions
                            self.user_defined_boxes.insert(name.clone());
                            self.build_box_declaration(
                                name.clone(),
                                methods.clone(),
                                fields.clone(),
                                weak_fields.clone(),
                            )?;
                            for (ctor_key, ctor_ast) in constructors.iter() {
                                if let N::FunctionDeclaration { params, body, .. } = ctor_ast {
                                    // Keep constructor function name as "Box.birth/N" where ctor_key already encodes arity.
                                    // ctor_key format comes from parser as "birth/<arity>".
                                    let func_name = format!("{}.{}", name, ctor_key);
                                    self.lower_method_as_function(
                                        func_name,
                                        name.clone(),
                                        params.clone(),
                                        body.clone(),
                                    )?;
                                }
                            }
                            for (mname, mast) in methods.iter() {
                                if let N::FunctionDeclaration { params, body, is_static, .. } = mast {
                                    if !*is_static {
                                        let func_name = format!("{}.{}{}", name, mname, format!("/{}", params.len()));
                                        // Index instance method for rewrite gating
                                        self.method_index.register_instance_method(
                                            name.clone(),
                                            mname.clone(),
                                            params.len()
                                        );
                                        self.lower_method_as_function(
                                            func_name,
                                            name.clone(),
                                            params.clone(),
                                            body.clone(),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }

                // Second pass: lower Main.main body as Program (entry). If absent, fall back to sequential block.
                if let Some((box_name, methods)) = main_static {
                    self.build_static_main_box(box_name, methods)
                } else {
                    // Fallback: sequential lowering (keeps legacy behavior for scripts without Main)
                    self.cf_block(statements)
                }
            }
            other => self.build_expression(other),
        }
    }

    pub(super) fn finalize_module(
        &mut self,
        result_value: ValueId,
    ) -> Result<MirModule, String> {
        // Hint: scope leave at function end (id=0 for main)
        self.hint_scope_leave(0);
        if let Some(block_id) = self.current_block {
            if let Some(ref mut function) = self.current_function {
                if let Some(block) = function.get_block_mut(block_id) {
                    if !block.is_terminated() {
                        block.add_instruction(MirInstruction::Return {
                            value: Some(result_value),
                        });
                    }
                    if let Some(mt) = self.value_types.get(&result_value).cloned() {
                        function.signature.return_type = mt;
                    }
                }
            }
        }

        let mut module = self.current_module.take().unwrap();
        let mut function = self.current_function.take().unwrap();
        function.metadata.value_types = self.value_types.clone();
        if matches!(
            function.signature.return_type,
            super::MirType::Void | super::MirType::Unknown
        ) {
            let mut inferred: Option<super::MirType> = None;
            'outer: for (_bid, bb) in function.blocks.iter() {
                for inst in bb.instructions.iter() {
                    if let super::MirInstruction::Return { value: Some(v) } = inst {
                        if let Some(mt) = self.value_types.get(v).cloned() {
                            inferred = Some(mt);
                            break 'outer;
                        }
                        if let Some(mt) = crate::mir::phi_core::if_phi::infer_type_from_phi(
                            &function,
                            *v,
                            &self.value_types,
                        ) {
                            inferred = Some(mt);
                            break 'outer;
                        }
                    }
                }
                if let Some(super::MirInstruction::Return { value: Some(v) }) = &bb.terminator {
                    if let Some(mt) = self.value_types.get(v).cloned() {
                        inferred = Some(mt);
                        break;
                    }
                    if let Some(mt) = crate::mir::phi_core::if_phi::infer_type_from_phi(
                        &function,
                        *v,
                        &self.value_types,
                    ) {
                        inferred = Some(mt);
                        break;
                    }
                }
            }
            if let Some(mt) = inferred {
                function.signature.return_type = mt;
            }
        }
        // Dev-only verify: NewBox → birth() invariant (warn if missing)
        if crate::config::env::using_is_dev() {
            let mut warn_count = 0usize;
            for (_bid, bb) in function.blocks.iter() {
                let insns = &bb.instructions;
                let mut idx = 0usize;
                while idx < insns.len() {
                    if let MirInstruction::NewBox { dst, box_type, args, .. } = &insns[idx] {
                        // Skip StringBox (literal optimization path)
                        if box_type != "StringBox" {
                            let expect_tail = format!("{}.birth/{}", box_type, args.len());
                            // Look ahead up to 3 instructions for either BoxCall("birth") on dst or Global(expect_tail)
                            let mut ok = false;
                            let mut j = idx + 1;
                            let mut last_const_name: Option<String> = None;
                            // Allow up to 4 ops ahead to tolerate a receiver copy (dst -> tmp) before birth()
                            let mut recv_alias: Option<super::ValueId> = None;
                            while j < insns.len() && j <= idx + 4 {
                                match &insns[j] {
                                    MirInstruction::BoxCall { box_val, method, .. } => {
                                        // accept birth on original dst or its immediate copy alias
                                        let target = if let Some(a) = recv_alias { a } else { *dst };
                                        if method == "birth" && *box_val == target { ok = true; break; }
                                    }
                                    MirInstruction::Const { value, .. } => {
                                        if let super::ConstValue::String(s) = value { last_const_name = Some(s.clone()); }
                                    }
                                    MirInstruction::Copy { dst: d, src: s } => {
                                        if s == dst { recv_alias = Some(*d); }
                                    }
                                    MirInstruction::Call { callee, args, .. } => {
                                        // Accept unified ModuleFunction("Class.birth/N") with me == dst (or alias)
                                        if let Some(ref c) = callee {
                                            if let crate::mir::definitions::Callee::ModuleFunction(ref fname) = c {
                                                if fname == &expect_tail {
                                                    if let Some(first) = args.get(0) {
                                                        let target = if let Some(a) = recv_alias { a } else { *dst };
                                                        if *first == target { ok = true; break; }
                                                    }
                                                }
                                            }
                                        }
                                        // Legacy: If immediately preceded by matching Const String, accept
                                        if let Some(prev) = last_const_name.as_ref() {
                                            if prev == &expect_tail { ok = true; break; }
                                        }
                                    }
                                    _ => {}
                                }
                                j += 1;
                            }
                            if !ok {
                                if !crate::config::env::cli_quiet() {
                                    eprintln!(
                                        "[warn] dev verify: NewBox {} at v{} not followed by birth() call (expect {})",
                                        box_type,
                                        dst,
                                        expect_tail
                                    );
                                    if std::env::var("NYASH_WARN_JSON").ok().as_deref() == Some("1") {
                                        eprintln!(
                                            "{}",
                                            crate::common::diagnostics::dev_verify_newbox_missing_birth(
                                                box_type,
                                                &format!("v{}", dst.0),
                                                &expect_tail,
                                            )
                                        );
                                    }
                                }
                                warn_count += 1;
                            }
                        }
                    }
                    idx += 1;
                }
            }
            if warn_count > 0 && !crate::config::env::cli_quiet() {
                eprintln!("[warn] dev verify: NewBox→birth invariant warnings: {}", warn_count);
                if std::env::var("NYASH_WARN_JSON").ok().as_deref() == Some("1") {
                    eprintln!(
                        "{}",
                        crate::common::diagnostics::dev_verify_birth_invariant_summary(warn_count)
                    );
                }
            }
        }

        // Normalize: ensure every basic block ends with a terminator (ret/jump/branch/throw)
        // This guards against unterminated blocks observed in rc paths.
        {
            use crate::mir::{MirInstruction, types::MirType};
            let mut fun = function.clone();
            // Collect block ids first to satisfy borrow rules
            let ids: Vec<_> = fun.blocks.keys().cloned().collect();
            for bid in ids {
                let mut need_fix = false;
                if let Some(bb) = fun.get_block(bid) {
                    if !bb.is_terminated() {
                        need_fix = true;
                    }
                }
                if need_fix {
                    match fun.signature.return_type {
                        MirType::Void | MirType::Unknown => {
                            if let Some(bbm) = fun.get_block_mut(bid) {
                                bbm.add_instruction(MirInstruction::Return { value: None });
                            }
                        }
                        _ => {
                            // Try to return the last defined value in the block; otherwise return 0
                            let mut last_dst: Option<crate::mir::ValueId> = None;
                            if let Some(bb) = fun.get_block(bid) {
                                for inst in bb.instructions.iter().rev() {
                                    if let Some(d) = inst.dst_value() { last_dst = Some(d); break; }
                                }
                            }
                            let ret_val = if let Some(d) = last_dst {
                                d
                            } else {
                                crate::mir::function_emission::emit_const_integer(&mut fun, bid, 0)
                            };
                            crate::mir::function_emission::emit_return_value(&mut fun, bid, ret_val);
                        }
                    }
                }
            }
            // Recompute CFG after edits
            fun.update_cfg();
            function = fun;
        }

        module.add_function(function);

        // Dev stub: provide condition_fn when missing to satisfy predicate calls in JSON lexers
        // Returns integer 1 (truthy) and accepts one argument (unused).
        if module.functions.get("condition_fn").is_none() {
            let sig = FunctionSignature {
                name: "condition_fn".to_string(),
                params: vec![MirType::Integer], // accept one i64-like arg
                return_type: MirType::Integer,
                effects: EffectMask::PURE,
            };
            let entry = BasicBlockId::new(0);
            let mut f = MirFunction::new(sig, entry);
            // parameter slot (unused in body)
            let _param = f.next_value_id();
            f.params.push(_param);
            // body: const 1; return it（FunctionEmissionBox を使用）
            let one = crate::mir::function_emission::emit_const_integer(&mut f, entry, 1);
            crate::mir::function_emission::emit_return_value(&mut f, entry, one);
            module.add_function(f);
        }

        
        // Final normalization: ensure every block in every function has a terminator
        {
            use crate::mir::{MirInstruction, types::MirType};
            // Collect function names to avoid borrow issues
            let fnames: Vec<String> = module.functions.keys().cloned().collect();
            for fname in fnames {
                if let Some(fun) = module.functions.get_mut(&fname) {
                    let ids: Vec<_> = fun.blocks.keys().cloned().collect();
                    for bid in ids {
                        let need_fix = fun.get_block(bid).map(|bb| !bb.is_terminated()).unwrap_or(false);
                        if need_fix {
                            match fun.signature.return_type {
                                MirType::Void | MirType::Unknown => {
                                    if let Some(bbm) = fun.get_block_mut(bid) {
                                        bbm.add_instruction(MirInstruction::Return { value: None });
                                    }
                                }
                                _ => {
                                    let mut last_dst: Option<crate::mir::ValueId> = None;
                                    if let Some(bb) = fun.get_block(bid) {
                                        for inst in bb.instructions.iter().rev() {
                                            if let Some(d) = inst.dst_value() { last_dst = Some(d); break; }
                                        }
                                    }
                                    let ret_val = if let Some(d) = last_dst {
                                        d
                                    } else {
                                        crate::mir::function_emission::emit_const_integer(fun, bid, 0)
                                    };
                                    crate::mir::function_emission::emit_return_value(fun, bid, ret_val);
                                }
                            }
                        }
                    }
                    fun.update_cfg();
                }
            }
        }

Ok(module)
    }
}
