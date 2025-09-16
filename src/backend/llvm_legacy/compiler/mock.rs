use crate::box_trait::{IntegerBox, NyashBox};
use crate::mir::function::MirModule;
use std::collections::HashMap;

pub struct LLVMCompiler {
    values: HashMap<crate::mir::ValueId, Box<dyn NyashBox>>,
}

impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }

    pub fn compile_module(&self, _mir: &MirModule, _out: &str) -> Result<(), String> {
        // Mock: pretend emitted
        Ok(())
    }

    pub fn compile_and_execute(
        &mut self,
        _mir: &MirModule,
        _out: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        Ok(Box::new(IntegerBox::new(0)))
    }
}
