use crate::mir::builder::MirBuilder;
use crate::mir::definitions::call_unified::Callee;
use crate::mir::ValueId;

// Normalize Set operations (Map-backed) into Extern("nyrt.set.*")
// Supports Method and ModuleFunction forms:
// - Method(receiver, method in {add/remove/has} with 1 arg) → Extern("nyrt.set.{m}") args=[recv,arg0]
// - Method(receiver, method in {size/clear/toArray} with 0 arg) → Extern("nyrt.set.{m}") args=[recv]
// - ModuleFunction("SetBox.{m}/N") accordingly
pub fn normalize_set_call(
    _builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    // Already normalized
    if let Callee::Extern(name) = callee {
        if name.starts_with("nyrt.set.") { return false; }
    }

    // Helper: rewrite to extern with provided args
    fn rewrite(callee: &mut Callee, args: &mut Vec<ValueId>, name: &str, new_args: Vec<ValueId>) -> bool {
        *callee = Callee::Extern(name.to_string());
        args.clear();
        args.extend(new_args);
        true
    }

    // Method form — restrict to SetBox receiver to avoid hijacking MapBox methods
    if let Callee::Method { method, receiver: Some(r), .. } = callee.clone() {
        // Guard: only when receiver is SetBox, or when test env allows MapBox to use Set externs
        let is_set = _builder
            .origin_get(r)
            .map(|s| s == "SetBox")
            .unwrap_or_else(|| matches!(_builder.value_types.get(&r), Some(crate::mir::MirType::Box(b)) if b == "SetBox"));
        let allow_map = std::env::var("HAKO_SET_ON_MAP").ok().as_deref() == Some("1");
        let is_map = _builder
            .origin_get(r)
            .map(|s| s == "MapBox")
            .unwrap_or_else(|| matches!(_builder.value_types.get(&r), Some(crate::mir::MirType::Box(b)) if b == "MapBox"));
        if !(is_set || (allow_map && is_map)) { return false; }
        match method.as_str() {
            "add" | "remove" | "has" => {
                if args.len() == 1 {
                    let recv_local = r; // already materialized
                    let a0 = args[0];
                    let extern_name = format!("nyrt.set.{}", method);
                    return rewrite(callee, args, &extern_name, vec![recv_local, a0]);
                }
            }
            "size" | "clear" | "toArray" => {
                if args.is_empty() {
                    let recv_local = r; // already materialized
                    let extern_name = format!("nyrt.set.{}", method);
                    return rewrite(callee, args, &extern_name, vec![recv_local]);
                }
            }
            _ => {}
        }
    }

    // ModuleFunction form: "SetBox.method/N"
    if let Callee::ModuleFunction(name) = callee.clone() {
        if name.starts_with("SetBox.") {
            // Extract method and arity suffix
            if let Some((m, arity)) = name[7..].split_once('/') { // after "SetBox."
                match (m, arity) {
                    ("add", "2") | ("remove", "2") | ("has", "2") => {
                        if args.len() == 2 {
                            let recv_local = args[0];
                            let a0 = args[1];
                            let extern_name = format!("nyrt.set.{}", m);
                            return rewrite(callee, args, &extern_name, vec![recv_local, a0]);
                        }
                    }
                    ("size", "1") | ("clear", "1") | ("toArray", "1") => {
                        if args.len() == 1 {
                            let recv_local = args[0];
                            let extern_name = format!("nyrt.set.{}", m);
                            return rewrite(callee, args, &extern_name, vec![recv_local]);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    false
}
