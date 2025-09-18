use crate::backend::vm::VMValue;
use crate::mir::BasicBlockId;

/// Control flow result from instruction execution
pub(crate) enum ControlFlow {
    Continue,
    Jump(BasicBlockId),
    Return(VMValue),
}
