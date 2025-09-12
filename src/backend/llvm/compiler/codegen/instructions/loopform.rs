use inkwell::{
    basic_block::BasicBlock,
    values::{BasicValueEnum, FunctionValue, PhiValue},
};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{
    function::MirFunction,
    instruction::MirInstruction,
    BasicBlockId,
    ValueId,
};

use super::builder_cursor::BuilderCursor;
use super::super::types::to_bool;

/// LoopForm scaffolding — fixed block layout for while/loop normalization
pub struct LoopFormContext<'ctx> {
    pub header: BasicBlock<'ctx>,
    pub body: BasicBlock<'ctx>,
    pub dispatch: BasicBlock<'ctx>,
    pub latch: BasicBlock<'ctx>,
    pub exit: BasicBlock<'ctx>,
    pub loop_id: u32,
}

impl<'ctx> LoopFormContext<'ctx> {
    /// Create a new LoopForm block set under `function` with a readable name prefix.
    pub fn new(
        codegen: &CodegenContext<'ctx>,
        function: FunctionValue<'ctx>,
        loop_id: u32,
        prefix: &str,
    ) -> Self {
        let header = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_header", prefix, loop_id));
        let body = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_body", prefix, loop_id));
        let dispatch = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_dispatch", prefix, loop_id));
        let latch = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_latch", prefix, loop_id));
        let exit = codegen
            .context
            .append_basic_block(function, &format!("{}_lf{}_exit", prefix, loop_id));
        Self { header, body, dispatch, latch, exit, loop_id }
    }
}

/// Lower a while-like loop using LoopForm shape (Phase 1: scaffold only).
/// - condition: MIR value producing i1/i64 truthy
/// - body_mir: MIR instructions of loop body
/// Note: In Phase 1, this function is not invoked by default lowering; it is a gated scaffold.
pub fn lower_while_loopform<'ctx, 'b>(
    codegen: &CodegenContext<'ctx>,
    cursor: &mut BuilderCursor<'ctx, 'b>,
    func: &MirFunction,
    llvm_func: FunctionValue<'ctx>,
    condition: &ValueId,
    _body_mir: &[MirInstruction],
    loop_id: u32,
    prefix: &str,
    header_bid: BasicBlockId,
    body_bb: BasicBlockId,
    after_bb: BasicBlockId,
    bb_map: &std::collections::HashMap<BasicBlockId, BasicBlock<'ctx>>,
    vmap: &std::collections::HashMap<ValueId, BasicValueEnum<'ctx>>,
    // Registry to allow later body→dispatch wiring (simple bodies)
    registry: &mut std::collections::HashMap<BasicBlockId, (BasicBlock<'ctx>, PhiValue<'ctx>, PhiValue<'ctx>, BasicBlock<'ctx>)>,
    body_to_header: &mut std::collections::HashMap<BasicBlockId, BasicBlockId>,
) -> Result<bool, String> {
    let enabled = std::env::var("NYASH_ENABLE_LOOPFORM").ok().as_deref() == Some("1");
    if !enabled { return Ok(false); }

    // Create LoopForm fixed blocks under the same function
    let lf = LoopFormContext::new(codegen, llvm_func, loop_id, prefix);

    // Header: evaluate condition and branch to body (for true) or dispatch (for false)
    let cond_v = *vmap.get(condition).ok_or("loopform: condition value missing")?;
    let cond_i1 = to_bool(codegen.context, cond_v, &codegen.builder)?;
    cursor.emit_term(header_bid, |b| {
        b.build_conditional_branch(cond_i1, lf.body, lf.dispatch)
            .map_err(|e| e.to_string())
            .unwrap();
    });

    // Body: currently pass-through to original body block (non-invasive Phase 1)
    let orig_body = *bb_map.get(&body_bb).ok_or("loopform: body bb missing")?;
    cursor.with_block(body_bb, lf.body, |c| {
        c.emit_term(body_bb, |b| {
            b.build_unconditional_branch(orig_body)
                .map_err(|e| e.to_string())
                .unwrap();
        });
    });

    // Dispatch: create PHIs (tag i8, payload i64) and switch(tag)
    // For now, only header(false) contributes (Break=1); body path does not reach dispatch in Phase 1 wiring.
    let orig_after = *bb_map.get(&after_bb).ok_or("loopform: after bb missing")?;
    let header_llbb = *bb_map.get(&header_bid).ok_or("loopform: header bb missing")?;
    let (tag_phi, payload_phi) = cursor.with_block(after_bb, lf.dispatch, |c| {
        let i8t = codegen.context.i8_type();
        let i64t = codegen.context.i64_type();
        let tag_ty: inkwell::types::BasicTypeEnum = i8t.into();
        let tag_phi = codegen
            .builder
            .build_phi(tag_ty, "lf_tag")
            .map_err(|e| e.to_string())
            .unwrap();
        let payload_ty: inkwell::types::BasicTypeEnum = i64t.into();
        let payload_phi = codegen
            .builder
            .build_phi(payload_ty, "lf_payload")
            .map_err(|e| e.to_string())
            .unwrap();
        let tag_break = i8t.const_int(1, false);
        let payload_zero = i64t.const_zero();
        tag_phi.add_incoming(&[(&tag_break, header_llbb)]);
        payload_phi.add_incoming(&[(&payload_zero, header_llbb)]);
        let tag_iv = tag_phi.as_basic_value().into_int_value();
        c.emit_term(after_bb, |b| {
            b.build_switch(tag_iv, lf.exit, &[(i8t.const_int(0, false), lf.latch)])
                .map_err(|e| e.to_string())
                .unwrap();
        });
        (tag_phi, payload_phi)
    });

    // Register for simple body→dispatch wiring later (at body terminator lowering time)
    registry.insert(header_bid, (lf.dispatch, tag_phi, payload_phi, lf.latch));
    body_to_header.insert(body_bb, header_bid);

    // Latch: optionally jump back to header (gated), otherwise keep unreachable to avoid header pred増
    codegen.builder.position_at_end(lf.latch);
    if std::env::var("NYASH_LOOPFORM_LATCH2HEADER").ok().as_deref() == Some("1") {
        codegen
            .builder
            .build_unconditional_branch(header_llbb)
            .map_err(|e| e.to_string())
            .unwrap();
    } else {
        let _ = codegen.builder.build_unreachable();
    }
    // Exit: to original after
    codegen.builder.position_at_end(lf.exit);
    codegen
        .builder
        .build_unconditional_branch(orig_after)
        .map_err(|e| e.to_string())
        .unwrap();

    if std::env::var("NYASH_CLI_VERBOSE").ok().as_deref() == Some("1") {
        eprintln!(
            "[LoopForm] wired header->(body/dispatch) and pass-through to then/else (lf_id={})",
            loop_id
        );
    }
    Ok(true)
}

/// LoopForm header PHI normalization: when enabling latch→header, header gains an extra LLVM
/// predecessor (latch) that is not represented in MIR predecessors. To satisfy LLVM's verifier,
/// ensure every PHI in the header has an incoming for the latch. For Phase 1, we conservatively
/// wire a typed zero as the incoming value for the latch.
pub fn normalize_header_phis_for_latch<'ctx>(
    codegen: &CodegenContext<'ctx>,
    header_bid: BasicBlockId,
    latch_bb: BasicBlock<'ctx>,
    phis: &[(ValueId, PhiValue<'ctx>, Vec<(BasicBlockId, ValueId)>)],
) -> Result<(), String> {
    use inkwell::types::BasicTypeEnum as BT;
    let _ = header_bid; // reserved for future diagnostics
    for (_dst, phi, _inputs) in phis {
        let bt = phi.as_basic_value().get_type();
        let z = match bt {
            BT::IntType(it) => it.const_zero().into(),
            BT::FloatType(ft) => ft.const_zero().into(),
            BT::PointerType(pt) => pt.const_zero().into(),
            _ => return Err("unsupported phi type for latch incoming".to_string()),
        };
        match z {
            BasicValueEnum::IntValue(iv) => phi.add_incoming(&[(&iv, latch_bb)]),
            BasicValueEnum::FloatValue(fv) => phi.add_incoming(&[(&fv, latch_bb)]),
            BasicValueEnum::PointerValue(pv) => phi.add_incoming(&[(&pv, latch_bb)]),
            _ => return Err("unsupported zero value kind for latch incoming".to_string()),
        }
    }
    Ok(())
}
