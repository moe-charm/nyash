use std::collections::HashMap;

use inkwell::{
    basic_block::BasicBlock,
    values::{BasicValueEnum as BVE, FloatValue, IntValue, PointerValue},
};

use crate::backend::llvm::context::CodegenContext;
use crate::mir::{function::MirFunction, BasicBlockId, ValueId};

use super::{builder_cursor::BuilderCursor, Resolver};

pub type LlResult<T> = Result<T, String>;

/// Per-function lowering context that centralizes access to codegen utilities and
/// enforces Resolver-only value access.
pub struct LowerFnCtx<'ctx, 'b> {
    pub codegen: &'ctx CodegenContext<'ctx>,
    pub func: &'b MirFunction,
    pub cursor: &'b mut BuilderCursor<'ctx, 'b>,
    pub resolver: &'b mut Resolver<'ctx>,
    pub vmap: &'b mut HashMap<ValueId, BVE<'ctx>>,
    pub bb_map: &'b HashMap<BasicBlockId, BasicBlock<'ctx>>,
    pub preds: &'b HashMap<BasicBlockId, Vec<BasicBlockId>>,
    pub block_end_values: &'b HashMap<BasicBlockId, HashMap<ValueId, BVE<'ctx>>>,
    // Optional extras commonly needed by some paths
    pub box_type_ids: Option<&'b HashMap<String, i64>>,
    pub const_strs: Option<&'b HashMap<ValueId, String>>,
    // Dev flag: extra runtime assertions
    pub dev_checks: bool,
}

impl<'ctx, 'b> LowerFnCtx<'ctx, 'b> {
    pub fn new(
        codegen: &'ctx CodegenContext<'ctx>,
        func: &'b MirFunction,
        cursor: &'b mut BuilderCursor<'ctx, 'b>,
        resolver: &'b mut Resolver<'ctx>,
        vmap: &'b mut HashMap<ValueId, BVE<'ctx>>,
        bb_map: &'b HashMap<BasicBlockId, BasicBlock<'ctx>>,
        preds: &'b HashMap<BasicBlockId, Vec<BasicBlockId>>,
        block_end_values: &'b HashMap<BasicBlockId, HashMap<ValueId, BVE<'ctx>>>,
    ) -> Self {
        let dev_checks = std::env::var("NYASH_DEV_CHECKS").ok().as_deref() == Some("1");
        Self {
            codegen,
            func,
            cursor,
            resolver,
            vmap,
            bb_map,
            preds,
            block_end_values,
            box_type_ids: None,
            const_strs: None,
            dev_checks,
        }
    }

    pub fn with_box_type_ids(mut self, ids: &'b HashMap<String, i64>) -> Self {
        self.box_type_ids = Some(ids);
        self
    }

    pub fn with_const_strs(mut self, m: &'b HashMap<ValueId, String>) -> Self {
        self.const_strs = Some(m);
        self
    }

    #[inline]
    pub fn ensure_i64(&mut self, blk: &BlockCtx<'ctx>, v: ValueId) -> LlResult<IntValue<'ctx>> {
        self.cursor.assert_open(blk.cur_bid);
        self.resolver.resolve_i64(
            self.codegen,
            self.cursor,
            blk.cur_bid,
            v,
            self.bb_map,
            self.preds,
            self.block_end_values,
            self.vmap,
        )
    }

    #[inline]
    pub fn ensure_ptr(&mut self, blk: &BlockCtx<'ctx>, v: ValueId) -> LlResult<PointerValue<'ctx>> {
        self.cursor.assert_open(blk.cur_bid);
        self.resolver.resolve_ptr(
            self.codegen,
            self.cursor,
            blk.cur_bid,
            v,
            self.bb_map,
            self.preds,
            self.block_end_values,
            self.vmap,
        )
    }

    #[inline]
    pub fn ensure_f64(&mut self, blk: &BlockCtx<'ctx>, v: ValueId) -> LlResult<FloatValue<'ctx>> {
        self.cursor.assert_open(blk.cur_bid);
        self.resolver.resolve_f64(
            self.codegen,
            self.cursor,
            blk.cur_bid,
            v,
            self.bb_map,
            self.preds,
            self.block_end_values,
            self.vmap,
        )
    }
}

/// Per-basic-block context to keep insertion site and block identity together.
pub struct BlockCtx<'ctx> {
    pub cur_bid: BasicBlockId,
    pub cur_llbb: BasicBlock<'ctx>,
}

impl<'ctx> BlockCtx<'ctx> {
    pub fn new(cur_bid: BasicBlockId, cur_llbb: BasicBlock<'ctx>) -> Self {
        Self { cur_bid, cur_llbb }
    }
}
