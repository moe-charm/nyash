use crate::abi::NyashTypeBoxFfi;
use crate::consts::*;
use crate::ffi::{self, slice};
use crate::state::{self, RequestState, ResponseState};
use crate::tlv;
use std::collections::HashMap;
use std::io::Write as IoWrite;

include!("request_impl.rs");
