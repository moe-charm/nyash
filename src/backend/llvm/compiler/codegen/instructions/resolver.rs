use std::collections::HashMap;

use inkwell::values::PointerValue;
use inkwell::values::{BasicValueEnum as BVE, IntValue};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};

use super::builder_cursor::BuilderCursor;
use super::flow::localize_to_i64;

/// Minimal per-function resolver caches. Caches localized i64 values per (block,value) to avoid
/// redundant PHIs and casts when multiple users in the same block request the same MIR value.
pub struct Resolver<'ctx> {
    i64_locals: HashMap<(BasicBlockId, ValueId), IntValue<'ctx>>,
    ptr_locals: HashMap<(BasicBlockId, ValueId), PointerValue<'ctx>>,
    f64_locals: HashMap<(BasicBlockId, ValueId), inkwell::values::FloatValue<'ctx>>,
}

impl<'ctx> Resolver<'ctx> {
    pub fn new() -> Self {
        Self {
            i64_locals: HashMap::new(),
            ptr_locals: HashMap::new(),
            f64_locals: HashMap::new(),
        }
    }

    /// Resolve a MIR value as an i64 dominating the current block.
    /// Strategy: if present in cache, return it; otherwise localize via sealed snapshots and cache.
    pub fn resolve_i64<'b>(
        &mut self,
        codegen: &CodegenContext<'ctx>,
        cursor: &mut BuilderCursor<'ctx, 'b>,
        cur_bid: BasicBlockId,
        vid: ValueId,
        bb_map: &std::collections::HashMap<BasicBlockId, inkwell::basic_block::BasicBlock<'ctx>>,
        preds: &std::collections::HashMap<BasicBlockId, Vec<BasicBlockId>>,
        block_end_values: &std::collections::HashMap<
            BasicBlockId,
            std::collections::HashMap<ValueId, BVE<'ctx>>,
        >,
        vmap: &std::collections::HashMap<ValueId, BVE<'ctx>>,
    ) -> Result<IntValue<'ctx>, String> {
        if let Some(iv) = self.i64_locals.get(&(cur_bid, vid)).copied() {
            return Ok(iv);
        }
        let iv = localize_to_i64(
            codegen,
            cursor,
            cur_bid,
            vid,
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        self.i64_locals.insert((cur_bid, vid), iv);
        Ok(iv)
    }

    /// Resolve a MIR value as an i8* pointer dominating the current block.
    pub fn resolve_ptr<'b>(
        &mut self,
        codegen: &CodegenContext<'ctx>,
        cursor: &mut BuilderCursor<'ctx, 'b>,
        cur_bid: BasicBlockId,
        vid: ValueId,
        bb_map: &std::collections::HashMap<BasicBlockId, inkwell::basic_block::BasicBlock<'ctx>>,
        preds: &std::collections::HashMap<BasicBlockId, Vec<BasicBlockId>>,
        block_end_values: &std::collections::HashMap<
            BasicBlockId,
            std::collections::HashMap<ValueId, BVE<'ctx>>,
        >,
        vmap: &std::collections::HashMap<ValueId, BVE<'ctx>>,
    ) -> Result<PointerValue<'ctx>, String> {
        if let Some(pv) = self.ptr_locals.get(&(cur_bid, vid)).copied() {
            return Ok(pv);
        }
        // Avoid using current vmap directly to keep dominance safe under multiple predecessors.
        // Strategy: localize as i64 (dominance-safe PHI), then convert to i8* in current block.
        let i8p = codegen.context.ptr_type(inkwell::AddressSpace::from(0));
        let iv = localize_to_i64(
            codegen,
            cursor,
            cur_bid,
            vid,
            bb_map,
            preds,
            block_end_values,
            vmap,
        )?;
        let pv = cursor
            .emit_instr(cur_bid, |b| b.build_int_to_ptr(iv, i8p, "loc_i2p_dom"))
            .map_err(|e| e.to_string())?;
        self.ptr_locals.insert((cur_bid, vid), pv);
        Ok(pv)
    }

    /// Resolve a MIR value as an f64 dominating the current block.
    pub fn resolve_f64<'b>(
        &mut self,
        codegen: &CodegenContext<'ctx>,
        cursor: &mut BuilderCursor<'ctx, 'b>,
        cur_bid: BasicBlockId,
        vid: ValueId,
        bb_map: &std::collections::HashMap<BasicBlockId, inkwell::basic_block::BasicBlock<'ctx>>,
        preds: &std::collections::HashMap<BasicBlockId, Vec<BasicBlockId>>,
        block_end_values: &std::collections::HashMap<
            BasicBlockId,
            std::collections::HashMap<ValueId, BVE<'ctx>>,
        >,
        vmap: &std::collections::HashMap<ValueId, BVE<'ctx>>,
    ) -> Result<inkwell::values::FloatValue<'ctx>, String> {
        if let Some(fv) = self.f64_locals.get(&(cur_bid, vid)).copied() {
            return Ok(fv);
        }
        // Avoid using current vmap directly to keep dominance safe under multiple predecessors.
        let f64t = codegen.context.f64_type();
        let cur_llbb = *bb_map.get(&cur_bid).ok_or("cur bb missing")?;
        let pred_list = preds.get(&cur_bid).cloned().unwrap_or_default();
        let saved_ip = codegen.builder.get_insert_block();
        if let Some(first) = cur_llbb.get_first_instruction() {
            codegen.builder.position_before(&first);
        } else {
            codegen.builder.position_at_end(cur_llbb);
        }
        let phi = codegen
            .builder
            .build_phi(f64t, &format!("loc_f64_{}", vid.as_u32()))
            .map_err(|e| e.to_string())?;
        if pred_list.is_empty() {
            // No predecessor: conservatively zero（vmap には依存しない）
            let z = f64t.const_zero();
            phi.add_incoming(&[(&z, cur_llbb)]);
        } else {
            for p in &pred_list {
                let pred_bb = *bb_map.get(p).ok_or("pred bb missing")?;
                let base = block_end_values
                    .get(p)
                    .and_then(|m| m.get(&vid).copied())
                    .unwrap_or_else(|| f64t.const_zero().into());
                let mut coerced = f64t.const_zero();
                cursor.with_block(*p, pred_bb, |c| {
                    let term = unsafe { pred_bb.get_terminator() };
                    if let Some(t) = term {
                        codegen.builder.position_before(&t);
                    } else {
                        c.position_at_end(pred_bb);
                    }
                    coerced = match base {
                        BVE::FloatValue(fv) => fv,
                        BVE::IntValue(iv) => codegen
                            .builder
                            .build_signed_int_to_float(iv, f64t, "loc_i2f_p")
                            .map_err(|e| e.to_string())
                            .unwrap(),
                        BVE::PointerValue(_) => f64t.const_zero(),
                        _ => f64t.const_zero(),
                    };
                });
                phi.add_incoming(&[(&coerced, pred_bb)]);
            }
        }
        if let Some(bb) = saved_ip {
            codegen.builder.position_at_end(bb);
        }
        let out = phi.as_basic_value().into_float_value();
        self.f64_locals.insert((cur_bid, vid), out);
        Ok(out)
    }
}
