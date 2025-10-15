use crate::mir::MirModule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    Specified,
    MainMain,
    TopLevel,
    ModuleMain,
}

#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub name: String,
    pub kind: EntryKind,
}

fn list_candidates_from_keys<'a, I>(keys: I) -> Vec<String>
where I: Iterator<Item=&'a String>
{
    let mut cands: Vec<String> = Vec::new();
    for k in keys {
        if k == "Main.main" || k == "main" || k.ends_with(".main") {
            cands.push(k.clone());
        }
    }
    cands.sort();
    cands.dedup();
    cands
}

pub fn resolve_entry_for_module(module: &MirModule, cli_entry: Option<&str>) -> Result<ResolvedEntry, String> {
    // CLI override
    if let Some(name) = cli_entry {
        if module.functions.contains_key(name) {
            return Ok(ResolvedEntry { name: name.to_string(), kind: EntryKind::Specified });
        }
        let cands = list_candidates_from_keys(module.functions.keys());
        return Err(format!("entry not found: {} (candidates: {})", name, if cands.is_empty() { "<none>".to_string() } else { cands.join(", ") }));
    }
    // Strict default: only Main.main accepted
    if module.functions.contains_key("Main.main") {
        return Ok(ResolvedEntry { name: "Main.main".to_string(), kind: EntryKind::MainMain });
    }
    // No implicit top-level adoption; report helpful error
    let cands = list_candidates_from_keys(module.functions.keys());
    Err(format!("strict entry resolution failed: missing Main.main (candidates: {})",
        if cands.is_empty() { "<none>".to_string() } else { cands.join(", ") }))
}

// Library crate variant (nyash_rust)
pub fn resolve_entry_for_module_lib(module: &nyash_rust::mir::MirModule, cli_entry: Option<&str>) -> Result<ResolvedEntry, String> {
    if let Some(name) = cli_entry {
        if module.functions.contains_key(name) {
            return Ok(ResolvedEntry { name: name.to_string(), kind: EntryKind::Specified });
        }
        let cands = list_candidates_from_keys(module.functions.keys());
        return Err(format!("entry not found: {} (candidates: {})", name, if cands.is_empty() { "<none>".to_string() } else { cands.join(", ") }));
    }
    if module.functions.contains_key("Main.main") {
        return Ok(ResolvedEntry { name: "Main.main".to_string(), kind: EntryKind::MainMain });
    }
    let cands = list_candidates_from_keys(module.functions.keys());
    Err(format!("strict entry resolution failed: missing Main.main (candidates: {})",
        if cands.is_empty() { "<none>".to_string() } else { cands.join(", ") }))
}
