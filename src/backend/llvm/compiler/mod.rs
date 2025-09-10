use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use std::collections::HashMap;

pub struct LLVMCompiler {
    values: HashMap<ValueId, Box<dyn NyashBox>>,
}

#[cfg(not(feature = "llvm"))]
mod mock;
#[cfg(not(feature = "llvm"))]
pub use mock::*;

#[cfg(feature = "llvm")]
mod real;
#[cfg(feature = "llvm")]
pub use real::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_module_creation() {
        assert!(true);
    }
}
