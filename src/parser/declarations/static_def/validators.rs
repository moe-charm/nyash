//! Static box validators (placeholder for symmetry)
#![allow(dead_code)]

use crate::parser::{NyashParser, ParseError};

pub(crate) struct StaticValidators;

impl StaticValidators {
    #[allow(dead_code)]
    pub(crate) fn validate(_p: &mut NyashParser) -> Result<(), ParseError> { Ok(()) }
}

