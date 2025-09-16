use std::collections::HashMap;

use inkwell::{basic_block::BasicBlock, builder::Builder};

use crate::mir::BasicBlockId;

/// Track per-block open/closed state and centralize terminator emission.
pub struct BuilderCursor<'ctx, 'b> {
    pub builder: &'b Builder<'ctx>,
    closed_by_bid: HashMap<BasicBlockId, bool>,
    cur_bid: Option<BasicBlockId>,
    cur_llbb: Option<BasicBlock<'ctx>>,
}

impl<'ctx, 'b> BuilderCursor<'ctx, 'b> {
    pub fn new(builder: &'b Builder<'ctx>) -> Self {
        Self {
            builder,
            closed_by_bid: HashMap::new(),
            cur_bid: None,
            cur_llbb: None,
        }
    }

    /// Temporarily switch to another block, run body, then restore previous position/state.
    pub fn with_block<R>(
        &mut self,
        bid: BasicBlockId,
        bb: BasicBlock<'ctx>,
        body: impl FnOnce(&mut BuilderCursor<'ctx, 'b>) -> R,
    ) -> R {
        let prev_bid = self.cur_bid;
        let prev_bb = self.cur_llbb;
        // Preserve previous closed state
        let prev_closed = prev_bid.and_then(|id| self.closed_by_bid.get(&id).copied());
        // Preserve target block closed state and restore after
        let tgt_closed_before = self.closed_by_bid.get(&bid).copied();

        self.at_end(bid, bb);
        let r = body(self);

        // Restore prior insertion point/state
        if let Some(pbb) = prev_bb {
            self.builder.position_at_end(pbb);
        }
        self.cur_bid = prev_bid;
        self.cur_llbb = prev_bb;
        if let (Some(pid), Some(closed)) = (prev_bid, prev_closed) {
            self.closed_by_bid.insert(pid, closed);
        }
        if let Some(closed) = tgt_closed_before {
            self.closed_by_bid.insert(bid, closed);
        } else {
            // If previously unknown, keep it marked as closed if a terminator exists
            let has_term = unsafe { bb.get_terminator() }.is_some();
            self.closed_by_bid.insert(bid, has_term);
        }
        r
    }

    pub fn at_end(&mut self, bid: BasicBlockId, bb: BasicBlock<'ctx>) {
        self.cur_bid = Some(bid);
        self.cur_llbb = Some(bb);
        // Mark closed if LLVM already has a terminator in this block
        let has_term = unsafe { bb.get_terminator() }.is_some();
        self.closed_by_bid.insert(bid, has_term);
        self.builder.position_at_end(bb);
    }

    pub fn position_at_end(&self, bb: BasicBlock<'ctx>) {
        self.builder.position_at_end(bb);
    }

    pub fn assert_open(&self, bid: BasicBlockId) {
        if let Some(closed) = self.closed_by_bid.get(&bid) {
            assert!(
                !closed,
                "attempt to insert into closed block {}",
                bid.as_u32()
            );
        }
    }

    pub fn emit_instr<T>(&mut self, bid: BasicBlockId, f: impl FnOnce(&Builder<'ctx>) -> T) -> T {
        self.assert_open(bid);
        // Extra hard guard: check actual LLVM block state before inserting
        if let Some(bb) = self.cur_llbb {
            if unsafe { bb.get_terminator() }.is_some() {
                panic!("post-terminator insert detected in bb {}", bid.as_u32());
            }
        }
        f(self.builder)
    }

    pub fn emit_term(&mut self, bid: BasicBlockId, f: impl FnOnce(&Builder<'ctx>)) {
        self.assert_open(bid);
        f(self.builder);
        // After emitting a terminator, assert the current basic block now has one
        if let Some(bb) = self.cur_llbb {
            assert!(
                unsafe { bb.get_terminator() }.is_some(),
                "expected terminator in bb {}",
                bid.as_u32()
            );
        }
        self.closed_by_bid.insert(bid, true);
    }
}
