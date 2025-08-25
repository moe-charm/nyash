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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use std::collections::BTreeSet;

    // Ensure docs/reference/mir/INSTRUCTION_SET.md and implementation list stay in perfect sync (26 items)
    #[test]
    fn mir26_doc_and_impl_are_in_sync() {
        // 1) Read the canonical list from docs
        let doc_path = Path::new("docs/reference/mir/INSTRUCTION_SET.md");
        let content = fs::read_to_string(doc_path)
            .expect("Failed to read docs/reference/mir/INSTRUCTION_SET.md");

        let mut in_core = false;
        let mut doc_names: Vec<String> = Vec::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("## Core Instructions") {
                in_core = true;
                continue;
            }
            if in_core && line.starts_with("## ") { // stop at next section (Meta)
                break;
            }
            if in_core {
                if let Some(rest) = line.strip_prefix("- ") {
                    // Strip annotations like （...） or (...) and trailing spaces
                    let name = rest
                        .split(|c: char| c.is_whitespace() || c == '（' || c == '(')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !name.is_empty() {
                        doc_names.push(name.to_string());
                    }
                }
            }
        }

        // 2) Implementation list
        let impl_names = core_instruction_names();
        // Keep the source-of-truth synced: names and counts must match
        assert_eq!(doc_names.len(), impl_names.len(), "Doc and impl must list the same number of core instructions");

        // 3) Compare as sets (order agnostic)
        let doc_set: BTreeSet<_> = doc_names.iter().map(|s| s.as_str()).collect();
        let impl_set: BTreeSet<_> = impl_names.iter().copied().collect();
        assert_eq!(doc_set, impl_set, "MIR core instruction names must match docs exactly");
    }
}
