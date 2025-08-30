//! Introspection helpers for MIR instruction set
//!
//! Migration note:
//! - Historically we synced to a canonical 26-instruction doc list.
//! - We are migrating to Core-15 for enforcement and tests. During migration,
//!   docs may still list 26; tests should use `core15_instruction_names()`.

/// Returns the legacy canonical list of core MIR instruction names (26 items).
/// This list matched docs/reference/mir/INSTRUCTION_SET.md under "Core Instructions".
pub fn core_instruction_names() -> &'static [&'static str] {
    &[
        "Const",
        "Copy",
        "Load",
        "Store",
        "UnaryOp",
        "BinOp",
        "Compare",
        "Jump",
        "Branch",
        "Phi",
        "Return",
        "Call",
        "ExternCall",
        "BoxCall",
        "NewBox",
        "ArrayGet",
        "ArraySet",
        "RefNew",
        "RefGet",
        "RefSet",
        "Await",
        "Print",
        "TypeOp",
        "WeakRef",
        "Barrier",
    ]
}

/// Returns the Core-15 instruction names used for the minimal kernel enforcement.
/// This list is implementation-driven for migration stage; docs may differ temporarily.
pub fn core15_instruction_names() -> &'static [&'static str] {
    &[
        // 値/計算
        "Const",
        "UnaryOp",
        "BinOp",
        "Compare",
        "TypeOp",
        // メモリ
        "Load",
        "Store",
        // 制御
        "Jump",
        "Branch",
        "Return",
        "Phi",
        // 呼び出し/Box
        "Call",
        "NewBox",
        "BoxCall",
        "ExternCall",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::collections::BTreeSet;

    // Core-15 enforcement: only count check; names are implementation-defined during migration.
    #[test]
    fn core15_instruction_count_is_15() {
        let impl_names = core15_instruction_names();
        assert_eq!(impl_names.len(), 15, "Core-15 must contain exactly 15 instructions");
        // basic sanity: includes a few key ops
        let set: BTreeSet<_> = impl_names.iter().copied().collect();
        for must in ["Const", "BinOp", "Return", "ExternCall"] { assert!(set.contains(must), "missing '{}'", must); }
    }
}
