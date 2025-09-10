use super::LLVMCompiler;
use crate::box_trait::{BoolBox, IntegerBox, NyashBox, StringBox};
use crate::boxes::{function_box::FunctionBox, math_box::FloatBox, null_box::NullBox};
use crate::mir::function::MirModule;
use crate::mir::instruction::{BinaryOp, ConstValue, MirInstruction};
use std::collections::HashMap;

include!("mock_impl.in.rs");
