use crate::mir::instruction::{ConstValue, MirInstruction};
use crate::mir::ValueId;
use std::collections::HashMap;

pub(super) fn sanitize_symbol(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '.' | '/' | '-' => '_',
            other => other,
        })
        .collect()
}

pub(super) fn build_const_str_map(
    f: &crate::mir::function::MirFunction,
) -> HashMap<ValueId, String> {
    let mut m = HashMap::new();
    for bid in f.block_ids() {
        if let Some(b) = f.blocks.get(&bid) {
            for inst in &b.instructions {
                if let MirInstruction::Const {
                    dst,
                    value: ConstValue::String(s),
                } = inst
                {
                    m.insert(*dst, s.clone());
                }
            }
            if let Some(MirInstruction::Const {
                dst,
                value: ConstValue::String(s),
            }) = &b.terminator
            {
                m.insert(*dst, s.clone());
            }
        }
    }
    m
}
