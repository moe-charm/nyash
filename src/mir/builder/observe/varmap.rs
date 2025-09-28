//! VarMapTrace — dev-only variable_map quick line helper
//!
//! Prints a short one-liner with names bound to a given ValueId (receiver),
//! and a small summary of the current variable_map size.

use crate::mir::builder::MirBuilder;
use crate::mir::ValueId;

/// Emit a single line showing which variable names currently bind to `recv`.
/// Guarded by `NYASH_VARMAP_TRACE=1` (or true/on/yes).
pub fn emit_recv_names(builder: &MirBuilder, recv: ValueId, where_tag: &str) {
    if !super::common::trace_enabled("NYASH_VARMAP_TRACE") { return; }
    let mut names: Vec<String> = Vec::new();
    for (k, v) in builder.variable_map.iter() {
        if *v == recv { names.push(k.clone()); }
    }
    names.sort();
    let total = builder.variable_map.len();
    super::common::eprintln_tag(
        "varmap",
        &format!(
            "tag={} recv=%{} names={:?} map_size={}",
            where_tag, recv.0, names, total
        ),
    );
}

