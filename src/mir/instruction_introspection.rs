//! Introspection helpers for the MIR14 instruction set

/// Return the canonical list of MIR14 instruction names.
pub fn mir14_instruction_names() -> &'static [&'static str] {
    &[
        // values / arithmetic
        "Const",
        "UnaryOp",
        "BinOp",
        "Compare",
        "TypeOp",
        // memory
        "Load",
        "Store",
        // control flow
        "Jump",
        "Branch",
        "Return",
        "Phi",
        // boxes / external
        "NewBox",
        "BoxCall",
        "ExternCall",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn mir14_instruction_count_is_14() {
        let names = mir14_instruction_names();
        assert_eq!(
            names.len(),
            14,
            "MIR14 must contain exactly 14 instructions"
        );
        let set: BTreeSet<_> = names.iter().copied().collect();
        for must in ["Const", "UnaryOp", "BoxCall"] {
            assert!(set.contains(must), "missing '{}'", must);
        }
    }
}
