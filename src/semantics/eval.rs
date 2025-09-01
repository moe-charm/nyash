/*!
 * MIR evaluator (skeleton)
 *
 * Walks a MIR function and calls into a Semantics implementation.
 * PoC: minimal matching; not wired yet.
 */

#![allow(dead_code)]

use std::collections::HashMap;
use crate::mir::{function::MirFunction, ValueId, instruction::{MirInstruction, ConstValue}};
use super::Semantics;

pub struct MirInterpreter<S: Semantics> {
    vals: HashMap<ValueId, S::Val>,
}

impl<S: Semantics> MirInterpreter<S> {
    pub fn new() -> Self { Self { vals: HashMap::new() } }

    pub fn eval_function(&mut self, f: &MirFunction, sem: &mut S) {
        // Very small PoC: iterate blocks in numeric order; ignore control flow for now.
        let mut blocks: Vec<_> = f.blocks.iter().collect();
        blocks.sort_by_key(|(id, _)| id.as_u32());
        for (_bid, bb) in blocks {
            for inst in &bb.instructions {
                match inst {
                    MirInstruction::Const { dst, value } => {
                        let v = match value {
                            ConstValue::Integer(i) => sem.const_i64(*i),
                            ConstValue::Float(x) => sem.const_f64(*x),
                            ConstValue::Bool(b) => sem.const_bool(*b),
                            ConstValue::Null => sem.const_null(),
                            ConstValue::String(s) => sem.const_str(s),
                        };
                        self.vals.insert(*dst, v);
                    }
                    MirInstruction::Return { value } => {
                        let rv = value.and_then(|vid| self.vals.get(&vid).cloned());
                        sem.ret(rv);
                        return;
                    }
                    _ => { /* later */ }
                }
            }
        }
    }
}

