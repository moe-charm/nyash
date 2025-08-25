use std::fs;
use std::path::Path;
use nyash_rust::mir::instruction_introspection;

// Compare the source-of-truth doc and implementation core instruction names.
#[test]
fn mir_instruction_set_doc_sync() {
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
        if in_core && line.starts_with("## ") { // stop at next section (e.g., Meta)
            break;
        }
        if in_core {
            if let Some(rest) = line.strip_prefix("- ") {
                // Each bullet is a name, possibly with annotations like （...） → strip anything after a space/paren
                let name = rest.split(|c: char| c.is_whitespace() || c == '（' || c == '(')
                               .next().unwrap_or("").trim();
                if !name.is_empty() {
                    doc_names.push(name.to_string());
                }
            }
        }
    }

    assert_eq!(doc_names.len(), 26, "Doc must list exactly 26 core instructions");

    // 2) Implementation list
    let impl_names = instruction_introspection::core_instruction_names();

    // Compare as sets (order-agnostic)
    let doc_set: std::collections::BTreeSet<_> = doc_names.iter().map(|s| s.as_str()).collect();
    let impl_set: std::collections::BTreeSet<_> = impl_names.iter().copied().collect();

    assert_eq!(doc_set, impl_set, "MIR core instruction names must match docs exactly");
}
