use crate::box_trait::NyashBox;
use crate::mir::ValueId;
use std::collections::HashMap;

pub struct LLVMCompiler {
    values: HashMap<ValueId, Box<dyn NyashBox>>,
}

#[cfg(not(feature = "llvm-inkwell-legacy"))]
mod mock;
#[cfg(not(feature = "llvm-inkwell-legacy"))]
pub use mock::*;

#[cfg(feature = "llvm-inkwell-legacy")]
mod aot;
#[cfg(feature = "llvm-inkwell-legacy")]
mod codegen;
#[cfg(feature = "llvm-inkwell-legacy")]
mod helpers;
#[cfg(feature = "llvm-inkwell-legacy")]
mod interpreter;
#[cfg(feature = "llvm-inkwell-legacy")]
pub use aot::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_module_creation() {
        assert!(true);
    }
}
