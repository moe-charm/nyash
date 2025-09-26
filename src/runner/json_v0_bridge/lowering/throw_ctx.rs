use crate::mir::{BasicBlockId, MirFunction, MirInstruction, ValueId};
use std::cell::RefCell;

thread_local! {
    static THROW_CTX: RefCell<Option<ThrowCtx>> = RefCell::new(None);
}

#[derive(Clone, Debug)]
pub(super) struct ThrowCtx {
    pub(super) catch_bb: BasicBlockId,
    pub(super) incoming: Vec<(BasicBlockId, ValueId)>,
}

impl ThrowCtx {
    fn new(catch_bb: BasicBlockId) -> Self {
        Self { catch_bb, incoming: Vec::new() }
    }
}

pub(super) fn set(catch_bb: BasicBlockId) {
    THROW_CTX.with(|slot| {
        *slot.borrow_mut() = Some(ThrowCtx::new(catch_bb));
    });
}

pub(super) fn take() -> Option<ThrowCtx> {
    THROW_CTX.with(|slot| slot.borrow_mut().take())
}

pub(super) fn is_active() -> bool {
    THROW_CTX.with(|slot| slot.borrow().is_some())
}

/// Record a throw from `from_bb` with value `exc_val`. Sets terminator Jump to catch and
/// appends predecessor+value to the incoming list. Returns the catch block id if active.
pub(super) fn record_throw(f: &mut MirFunction, from_bb: BasicBlockId, exc_val: ValueId) -> Option<BasicBlockId> {
    THROW_CTX.with(|slot| {
        if let Some(ctx) = slot.borrow_mut().as_mut() {
            let target = ctx.catch_bb;
            if let Some(bb) = f.get_block_mut(from_bb) {
                bb.set_terminator(MirInstruction::Jump { target });
            }
            if let Some(succ) = f.get_block_mut(target) {
                succ.add_predecessor(from_bb);
            }
            ctx.incoming.push((from_bb, exc_val));
            Some(target)
        } else {
            None
        }
    })
}
