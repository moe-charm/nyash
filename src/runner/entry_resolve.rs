use crate::mir::MirModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind { MainMain, Specified }

#[derive(Debug, Clone)]
pub struct ResolvedEntry {
    pub name: String,
    pub kind: EntryKind,
    pub candidates: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum EntryError {
    MissingMainStrict { candidates: Vec<String> },
    CliSpecifiedNotFound { name: String, candidates: Vec<String> },
}

fn collect_candidates(module: &MirModule) -> Vec<String> {
    let mut cands: Vec<String> = module
        .functions
        .keys()
        .filter(|k| k.ends_with(".main") || k.ends_with(".main/0") || k.as_str() == "main")
        .map(|s| s.clone())
        .collect();
    cands.sort();
    cands
}

pub fn resolve_entry(module: &MirModule, cli_entry: Option<&str>) -> Result<ResolvedEntry, EntryError> {
    let candidates = collect_candidates(module);
    if let Some(e) = cli_entry {
        let target = if module.functions.contains_key(e) {
            e.to_string()
        } else {
            let with0 = format!("{}/0", e);
            if module.functions.contains_key(&with0) { with0 } else { e.to_string() }
        };
        if module.functions.contains_key(&target) {
            return Ok(ResolvedEntry { name: target, kind: EntryKind::Specified, candidates });
        } else {
            return Err(EntryError::CliSpecifiedNotFound { name: e.to_string(), candidates });
        }
    }
    if module.functions.contains_key("Main.main") {
        return Ok(ResolvedEntry { name: "Main.main".to_string(), kind: EntryKind::MainMain, candidates });
    }
    if module.functions.contains_key("Main.main/0") {
        return Ok(ResolvedEntry { name: "Main.main/0".to_string(), kind: EntryKind::MainMain, candidates });
    }
    Err(EntryError::MissingMainStrict { candidates })
}
