use crate::mir::BasicBlockId;
use crate::backend::vm::VMValue;

/// Control flow result from instruction execution
pub(crate) enum ControlFlow {
    Continue,
    Jump(BasicBlockId),
    Return(VMValue),
}
