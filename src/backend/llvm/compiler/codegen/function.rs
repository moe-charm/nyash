use super::*;

pub(super) fn lower_one_function<'ctx>(
    codegen: &CodegenContext<'ctx>,
    llvm_func: FunctionValue<'ctx>,
    func: &crate::mir::function::MirFunction,
    name: &str,
    box_type_ids: &HashMap<String, i64>,
    llvm_funcs: &HashMap<String, FunctionValue<'ctx>>,
) -> Result<(), String> {
    // Create basic blocks (prefix names with function label to avoid any ambiguity)
    let fn_label = sanitize_symbol(name);
    let (mut bb_map, entry_bb) =
        instructions::create_basic_blocks(codegen, llvm_func, func, &fn_label);
    let mut cursor = instructions::builder_cursor::BuilderCursor::new(&codegen.builder);
    cursor.at_end(func.entry_block, entry_bb);
    let mut vmap: HashMap<ValueId, BasicValueEnum> = HashMap::new();
    let mut allocas: HashMap<ValueId, PointerValue> = HashMap::new();
    let entry_builder = codegen.context.create_builder();
    entry_builder.position_at_end(entry_bb);
    let mut alloca_elem_types: HashMap<ValueId, BasicTypeEnum> = HashMap::new();
    let mut phis_by_block: HashMap<
        crate::mir::BasicBlockId,
        Vec<(ValueId, PhiValue, Vec<(crate::mir::BasicBlockId, ValueId)>)>,
    > = HashMap::new();
    // Snapshot of values at the end of each basic block (for sealed-SSA PHI wiring)
    let mut block_end_values: HashMap<crate::mir::BasicBlockId, HashMap<ValueId, BasicValueEnum>> =
        HashMap::new();
    // Build successors and predecessors map (for optional sealed-SSA PHI wiring)
    let mut succs: HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>> =
        HashMap::new();
    for (bid, block) in &func.blocks {
        let v: Vec<crate::mir::BasicBlockId> = block.successors.iter().copied().collect();
        succs.insert(*bid, v);
    }
    let mut preds: HashMap<crate::mir::BasicBlockId, Vec<crate::mir::BasicBlockId>> =
        HashMap::new();
    for (b, ss) in &succs {
        for s in ss {
            preds.entry(*s).or_default().push(*b);
        }
    }
    // Track sealed blocks to know when all preds of a successor are sealed
    let mut sealed_blocks: std::collections::HashSet<crate::mir::BasicBlockId> =
        std::collections::HashSet::new();
    // Bind parameters
    for (i, pid) in func.params.iter().enumerate() {
        if let Some(av) = llvm_func.get_nth_param(i as u32) {
            vmap.insert(*pid, av);
        }
    }
    // Gather block order once for fallthrough handling
    let block_ids: Vec<crate::mir::BasicBlockId> = func.block_ids().into_iter().collect();

    // Precreate phis
    for bid in &block_ids {
        let bb = *bb_map.get(bid).ok_or("missing bb in map")?;
        codegen.builder.position_at_end(bb);
        let block = func.blocks.get(bid).unwrap();
        for inst in block
            .instructions
            .iter()
            .take_while(|i| matches!(i, MirInstruction::Phi { .. }))
        {
            if let MirInstruction::Phi { dst, inputs } = inst {
                let mut phi_ty: Option<BasicTypeEnum> = None;
                if let Some(mt) = func.metadata.value_types.get(dst) {
                    phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                } else if let Some((_, iv)) = inputs.first() {
                    if let Some(mt) = func.metadata.value_types.get(iv) {
                        phi_ty = Some(map_mirtype_to_basic(codegen.context, mt));
                    }
                }
                let phi_ty = phi_ty.unwrap_or_else(|| codegen.context.i64_type().into());
                let phi = codegen
                    .builder
                    .build_phi(phi_ty, &format!("phi_{}", dst.as_u32()))
                    .map_err(|e| e.to_string())?;
                vmap.insert(*dst, phi.as_basic_value());
                phis_by_block
                    .entry(*bid)
                    .or_default()
                    .push((*dst, phi, inputs.clone()));
                if std::env::var("NYASH_LLVM_TRACE_PHI").ok().as_deref() == Some("1") {
                    let ty_str = phi
                        .as_basic_value()
                        .get_type()
                        .print_to_string()
                        .to_string();
                    let mut pairs: Vec<String> = Vec::new();
                    for (pb, vid) in inputs {
                        pairs.push(format!("({}->{})", pb.as_u32(), vid.as_u32()));
                    }
                    eprintln!(
                        "[PHI:new] fn={} bb={} dst={} ty={} inputs={}",
                        fn_label,
                        bid.as_u32(),
                        dst.as_u32(),
                        ty_str,
                        pairs.join(",")
                    );
                }
            }
        }
    }

    // Map of const strings for Call resolution
    let const_strs = utils::build_const_str_map(func);

    // Lower body
    let mut loopform_loop_id: u32 = 0;
    // Default sealed-SSA ON unless explicitly disabled with NYASH_LLVM_PHI_SEALED=0
    let sealed_mode = std::env::var("NYASH_LLVM_PHI_SEALED").ok().as_deref() != Some("0");
    // LoopForm registry (per-function lowering; gated)
    let mut loopform_registry: HashMap<
        crate::mir::BasicBlockId,
        (
            inkwell::basic_block::BasicBlock,
            PhiValue,
            PhiValue,
            inkwell::basic_block::BasicBlock,
        ),
    > = HashMap::new();
    let mut loopform_body_to_header: HashMap<crate::mir::BasicBlockId, crate::mir::BasicBlockId> =
        HashMap::new();
    // Per-function Resolver for dominance-safe value access (i64 minimal)
    let mut resolver = instructions::Resolver::new();
    for (bi, bid) in block_ids.iter().enumerate() {
        let bb = *bb_map.get(bid).unwrap();
        // Use cursor to position at BB start for lowering
        cursor.at_end(*bid, bb);
        if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
            eprintln!("[LLVM] lowering bb={}", bid.as_u32());
        }
        let block = func.blocks.get(bid).unwrap();
        let mut defined_in_block: std::collections::HashSet<ValueId> =
            std::collections::HashSet::new();
        for inst in &block.instructions {
            match inst {
                MirInstruction::NewBox {
                    dst,
                    box_type,
                    args,
                } => {
                    instructions::lower_newbox(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        &mut vmap,
                        *dst,
                        box_type,
                        args,
                        box_type_ids,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    defined_in_block.insert(*dst);
                }
                MirInstruction::Const { dst, value } => {
                    let bval = match value {
                        ConstValue::Integer(i) => {
                            codegen.context.i64_type().const_int(*i as u64, true).into()
                        }
                        ConstValue::Float(f) => codegen.context.f64_type().const_float(*f).into(),
                        ConstValue::Bool(b) => codegen
                            .context
                            .bool_type()
                            .const_int(*b as u64, false)
                            .into(),
                        ConstValue::String(s) => {
                            // Hoist string creation to entry block to dominate all uses.
                            let entry_term = unsafe { entry_bb.get_terminator() };
                            if let Some(t) = entry_term {
                                entry_builder.position_before(&t);
                            } else {
                                entry_builder.position_at_end(entry_bb);
                            }
                            let gv = entry_builder
                                .build_global_string_ptr(s, "str")
                                .map_err(|e| e.to_string())?;
                            let len = codegen.context.i32_type().const_int(s.len() as u64, false);
                            let rt = codegen.context.ptr_type(inkwell::AddressSpace::from(0));
                            let fn_ty = rt.fn_type(
                                &[
                                    codegen
                                        .context
                                        .ptr_type(inkwell::AddressSpace::from(0))
                                        .into(),
                                    codegen.context.i32_type().into(),
                                ],
                                false,
                            );
                            let callee = codegen
                                .module
                                .get_function("nyash_string_new")
                                .unwrap_or_else(|| {
                                    codegen.module.add_function("nyash_string_new", fn_ty, None)
                                });
                            let call = entry_builder
                                .build_call(
                                    callee,
                                    &[gv.as_pointer_value().into(), len.into()],
                                    "strnew",
                                )
                                .map_err(|e| e.to_string())?;
                            call.try_as_basic_value()
                                .left()
                                .ok_or("nyash_string_new returned void".to_string())?
                        }
                        ConstValue::Null => codegen
                            .context
                            .ptr_type(inkwell::AddressSpace::from(0))
                            .const_zero()
                            .into(),
                        ConstValue::Void => codegen.context.i64_type().const_zero().into(),
                    };
                    vmap.insert(*dst, bval);
                    defined_in_block.insert(*dst);
                }
                MirInstruction::Call {
                    dst,
                    func: callee,
                    args,
                    ..
                } => {
                    instructions::lower_call(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &mut vmap,
                        dst,
                        callee,
                        args,
                        &const_strs,
                        llvm_funcs,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    if let Some(d) = dst {
                        defined_in_block.insert(*d);
                    }
                }
                MirInstruction::BoxCall {
                    dst,
                    box_val,
                    method,
                    method_id,
                    args,
                    effects: _,
                } => {
                    instructions::lower_boxcall(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &mut vmap,
                        dst,
                        box_val,
                        method,
                        method_id,
                        args,
                        box_type_ids,
                        &entry_builder,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    if let Some(d) = dst {
                        defined_in_block.insert(*d);
                    }
                }
                MirInstruction::ExternCall {
                    dst,
                    iface_name,
                    method_name,
                    args,
                    effects: _,
                } => {
                    instructions::lower_externcall(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &mut vmap,
                        dst,
                        iface_name,
                        method_name,
                        args,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    if let Some(d) = dst {
                        defined_in_block.insert(*d);
                    }
                }
                MirInstruction::UnaryOp { dst, op, operand } => {
                    instructions::lower_unary(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &mut vmap,
                        *dst,
                        op,
                        operand,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    defined_in_block.insert(*dst);
                }
                MirInstruction::BinOp { dst, op, lhs, rhs } => {
                    instructions::lower_binop(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &mut vmap,
                        *dst,
                        op,
                        lhs,
                        rhs,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    defined_in_block.insert(*dst);
                }
                MirInstruction::Compare { dst, op, lhs, rhs } => {
                    let out = instructions::lower_compare(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &vmap,
                        op,
                        lhs,
                        rhs,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                    vmap.insert(*dst, out);
                    defined_in_block.insert(*dst);
                }
                MirInstruction::Store { value, ptr } => {
                    instructions::lower_store(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        &vmap,
                        &mut allocas,
                        &mut alloca_elem_types,
                        value,
                        ptr,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                }
                MirInstruction::Load { dst, ptr } => {
                    instructions::lower_load(
                        codegen,
                        &mut cursor,
                        *bid,
                        &mut vmap,
                        &mut allocas,
                        &mut alloca_elem_types,
                        dst,
                        ptr,
                    )?;
                    defined_in_block.insert(*dst);
                }
                MirInstruction::Phi { .. } => { /* precreated */ }
                _ => { /* ignore others */ }
            }
            // Snapshot end-of-block values
            let mut snap: HashMap<ValueId, BasicValueEnum> = HashMap::new();
            for vid in &defined_in_block {
                if let Some(v) = vmap.get(vid).copied() {
                    snap.insert(*vid, v);
                }
            }
            block_end_values.insert(*bid, snap);
        }
        // Terminator handling
        if let Some(term) = &block.terminator {
            cursor.at_end(*bid, bb);
            match term {
                MirInstruction::Return { value } => {
                    instructions::term_emit_return(
                        codegen,
                        &mut cursor,
                        &mut resolver,
                        *bid,
                        func,
                        &vmap,
                        value,
                        &bb_map,
                        &preds,
                        &block_end_values,
                    )?;
                }
                MirInstruction::Jump { target } => {
                    let mut handled = false;
                    if std::env::var("NYASH_ENABLE_LOOPFORM").ok().as_deref() == Some("1")
                        && std::env::var("NYASH_LOOPFORM_BODY2DISPATCH")
                            .ok()
                            .as_deref()
                            == Some("1")
                    {
                        if let Some(hdr) = loopform_body_to_header.get(bid) {
                            if hdr == target {
                                if let Some((dispatch_bb, tag_phi, payload_phi, _latch_bb)) =
                                    loopform_registry.get(hdr)
                                {
                                    let i8t = codegen.context.i8_type();
                                    let i64t = codegen.context.i64_type();
                                    let pred_llbb =
                                        *bb_map.get(bid).ok_or("loopform: body llbb missing")?;
                                    let z = i8t.const_zero();
                                    let pz = i64t.const_zero();
                                    tag_phi.add_incoming(&[(&z, pred_llbb)]);
                                    payload_phi.add_incoming(&[(&pz, pred_llbb)]);
                                    cursor.emit_term(*bid, |b| {
                                        b.build_unconditional_branch(*dispatch_bb).unwrap();
                                    });
                                    handled = true;
                                }
                            }
                        }
                    }
                    if !handled {
                        instructions::term_emit_jump(
                            codegen,
                            &mut cursor,
                            *bid,
                            target,
                            &bb_map,
                            &phis_by_block,
                        )?;
                    }
                }
                MirInstruction::Branch {
                    condition,
                    then_bb,
                    else_bb,
                } => {
                    let mut handled_by_loopform = false;
                    if std::env::var("NYASH_ENABLE_LOOPFORM").ok().as_deref() == Some("1") {
                        let mut is_back = |start: crate::mir::BasicBlockId| -> u8 {
                            if let Some(b) = func.blocks.get(&start) {
                                if let Some(crate::mir::instruction::MirInstruction::Jump {
                                    target,
                                }) = &b.terminator
                                {
                                    if target == bid {
                                        return 1;
                                    }
                                    if let Some(b2) = func.blocks.get(target) {
                                        if let Some(
                                            crate::mir::instruction::MirInstruction::Jump {
                                                target: t2,
                                            },
                                        ) = &b2.terminator
                                        {
                                            if t2 == bid {
                                                return 2;
                                            }
                                        }
                                    }
                                }
                            }
                            0
                        };
                        let d_then = is_back(*then_bb);
                        let d_else = is_back(*else_bb);
                        let choose_body = if d_then > 0 && d_else == 0 {
                            Some((*then_bb, *else_bb))
                        } else if d_else > 0 && d_then == 0 {
                            Some((*else_bb, *then_bb))
                        } else if d_then > 0 && d_else > 0 {
                            if d_then <= d_else {
                                Some((*then_bb, *else_bb))
                            } else {
                                Some((*else_bb, *then_bb))
                            }
                        } else {
                            None
                        };
                        if let Some((body_sel, after_sel)) = choose_body {
                            let body_block = func.blocks.get(&body_sel).unwrap();
                            handled_by_loopform = instructions::lower_while_loopform(
                                codegen,
                                &mut cursor,
                                &mut resolver,
                                func,
                                llvm_func,
                                condition,
                                &body_block.instructions,
                                loopform_loop_id,
                                &fn_label,
                                *bid,
                                body_sel,
                                after_sel,
                                &bb_map,
                                &vmap,
                                &preds,
                                &block_end_values,
                                &mut loopform_registry,
                                &mut loopform_body_to_header,
                            )?;
                            loopform_loop_id = loopform_loop_id.wrapping_add(1);
                        }
                    }
                    if !handled_by_loopform {
                        let cond_norm = instructions::normalize_branch_condition(func, condition);
                        instructions::term_emit_branch(
                            codegen,
                            &mut cursor,
                            &mut resolver,
                            *bid,
                            &cond_norm,
                            then_bb,
                            else_bb,
                            &bb_map,
                            &phis_by_block,
                            &vmap,
                            &preds,
                            &block_end_values,
                        )?;
                    }
                }
                _ => {
                    cursor.at_end(*bid, bb);
                    if let Some(next_bid) = block_ids.get(bi + 1) {
                        instructions::term_emit_jump(
                            codegen,
                            &mut cursor,
                            *bid,
                            next_bid,
                            &bb_map,
                            &phis_by_block,
                        )?;
                    } else {
                        let entry_first = func.entry_block;
                        instructions::term_emit_jump(
                            codegen,
                            &mut cursor,
                            *bid,
                            &entry_first,
                            &bb_map,
                            &phis_by_block,
                        )?;
                    }
                }
            }
        } else {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "[LLVM] no terminator in MIR for bb={} (fallback)",
                    bid.as_u32()
                );
            }
            cursor.at_end(*bid, bb);
            if let Some(next_bid) = block_ids.get(bi + 1) {
                instructions::term_emit_jump(
                    codegen,
                    &mut cursor,
                    *bid,
                    next_bid,
                    &bb_map,
                    &phis_by_block,
                )?;
            } else {
                let entry_first = func.entry_block;
                instructions::term_emit_jump(
                    codegen,
                    &mut cursor,
                    *bid,
                    &entry_first,
                    &bb_map,
                    &phis_by_block,
                )?;
            }
        }
        if unsafe { bb.get_terminator() }.is_none() {
            if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
                eprintln!(
                    "[LLVM] extra guard inserting fallback for bb={}",
                    bid.as_u32()
                );
            }
            cursor.at_end(*bid, bb);
            if let Some(next_bid) = block_ids.get(bi + 1) {
                instructions::term_emit_jump(
                    codegen,
                    &mut cursor,
                    *bid,
                    next_bid,
                    &bb_map,
                    &phis_by_block,
                )?;
            } else {
                let entry_first = func.entry_block;
                instructions::term_emit_jump(
                    codegen,
                    &mut cursor,
                    *bid,
                    &entry_first,
                    &bb_map,
                    &phis_by_block,
                )?;
            }
        }
        if sealed_mode {
            instructions::flow::seal_block(
                codegen,
                &mut cursor,
                func,
                *bid,
                &succs,
                &bb_map,
                &phis_by_block,
                &block_end_values,
            )?;
            sealed_blocks.insert(*bid);
        }
    }
    if std::env::var("NYASH_ENABLE_LOOPFORM").ok().as_deref() == Some("1")
        && std::env::var("NYASH_LOOPFORM_LATCH2HEADER").ok().as_deref() == Some("1")
    {
        for (hdr_bid, (_dispatch_bb, _tag_phi, _payload_phi, latch_bb)) in &loopform_registry {
            if let Some(phis) = phis_by_block.get(hdr_bid) {
                instructions::normalize_header_phis_for_latch(codegen, *hdr_bid, *latch_bb, phis)?;
            }
        }
        instructions::dev_check_dispatch_only_phi(&phis_by_block, &loopform_registry);
    }
    for bb in llvm_func.get_basic_blocks() {
        if unsafe { bb.get_terminator() }.is_none() {
            codegen.builder.position_at_end(bb);
            let _ = codegen.builder.build_unreachable();
        }
    }
    if !llvm_func.verify(true) {
        if std::env::var("NYASH_LLVM_DUMP_ON_FAIL").ok().as_deref() == Some("1") {
            let ir = codegen.module.print_to_string().to_string();
            let dump_dir = std::path::Path::new("tmp");
            let _ = std::fs::create_dir_all(dump_dir);
            let dump_path = dump_dir.join(format!("llvm_fail_{}.ll", sanitize_symbol(name)));
            let _ = std::fs::write(&dump_path, ir);
            eprintln!("[LLVM] wrote IR dump: {}", dump_path.display());
        }
        return Err(format!("Function verification failed: {}", name));
    }
    Ok(())
}
