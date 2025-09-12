use std::collections::HashMap;

use inkwell::values::{BasicValueEnum as BVE, IntValue};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{BasicBlockId, ValueId};

use super::builder_cursor::BuilderCursor;
use super::flow::localize_to_i64;

/// Minimal per-function resolver caches. Caches localized i64 values per (block,value) to avoid
/// redundant PHIs and casts when multiple users in the same block request the same MIR value.
pub struct Resolver<'ctx> {
    i64_locals: HashMap<(BasicBlockId, ValueId), IntValue<'ctx>>,
}

impl<'ctx> Resolver<'ctx> {
    pub fn new() -> Self {
        Self { i64_locals: HashMap::new() }
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
        block_end_values: &std::collections::HashMap<BasicBlockId, std::collections::HashMap<ValueId, BVE<'ctx>>>,
        vmap: &std::collections::HashMap<ValueId, BVE<'ctx>>,
    ) -> Result<IntValue<'ctx>, String> {
        if let Some(iv) = self.i64_locals.get(&(cur_bid, vid)).copied() {
            return Ok(iv);
        }
        let iv = localize_to_i64(codegen, cursor, cur_bid, vid, bb_map, preds, block_end_values, vmap)?;
        self.i64_locals.insert((cur_bid, vid), iv);
        Ok(iv)
    }
}

