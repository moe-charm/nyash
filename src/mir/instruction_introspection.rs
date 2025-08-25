//! Introspection helpers for MIR instruction set
//! This module enumerates the canonical 26 core instruction names to sync with docs.

/// Returns the canonical list of core MIR instruction names (26 items).
/// This list must match docs/reference/mir/INSTRUCTION_SET.md under "Core Instructions".
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

