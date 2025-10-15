#![cfg(feature = "legacy-boxes")]
use crate::box_factory::RuntimeError;
use crate::box_trait::NyashBox;

pub fn create(_args: &[Box<dyn NyashBox>]) -> Result<Box<dyn NyashBox>, RuntimeError> {
    Ok(Box::new(crate::boxes::set_box::SetBox::new()))
}

