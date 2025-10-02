use std::collections::HashMap;
use std::sync::OnceLock;

use crate::mir::externs::registry::{self, ExternCallSpec};

#[derive(Debug, Clone)]
pub struct WasmExternSignature {
    pub module: String,
    pub name: String,
    pub params: Vec<String>,
    pub result: Option<String>,
}

struct WasmExternAdapterBox {
    overrides: HashMap<(String, String), (String, String)>,
}

fn adapter() -> &'static WasmExternAdapterBox {
    static ADAPTER: OnceLock<WasmExternAdapterBox> = OnceLock::new();
    ADAPTER.get_or_init(|| {
        let overrides = HashMap::new();
        WasmExternAdapterBox { overrides }
    })
}

/// Resolve an extern call into WASM signature information.
pub fn resolve_signature(iface: &str, method: &str) -> Option<WasmExternSignature> {
    let spec = registry::registry().get(iface, method)?;
    adapter().resolve(spec)
}

/// Enumerate all known extern signatures (useful for collecting imports).
pub fn all_signatures() -> Vec<WasmExternSignature> {
    registry::registry()
        .iter()
        .filter_map(|(_, spec)| adapter().resolve(spec))
        .collect()
}

impl WasmExternAdapterBox {
    fn resolve(&self, spec: &ExternCallSpec) -> Option<WasmExternSignature> {
        let key = (spec.interface.clone(), spec.method.clone());
        let (module, name) = self
            .overrides
            .get(&key)
            .cloned()
            .unwrap_or_else(|| default_name(spec));

        let params = spec
            .args
            .iter()
            .map(|ty| mir_type_to_wasm_param(ty))
            .collect();
        let result = match spec.returns {
            crate::mir::MirType::Void => None,
            crate::mir::MirType::Unknown => None,
            _ => Some(mir_type_to_wasm_result(&spec.returns)),
        };

        Some(WasmExternSignature {
            module,
            name,
            params,
            result,
        })
    }
}

fn default_name(spec: &ExternCallSpec) -> (String, String) {
    let mut parts: Vec<&str> = spec.interface.split('.').collect();
    if parts.is_empty() {
        return ("nyrt".to_string(), spec.method.clone());
    }
    let module = parts.first().unwrap().to_string();
    parts.remove(0);
    if parts.is_empty() {
        return (module, spec.method.clone());
    }
    let mut name_parts: Vec<String> = parts.into_iter().map(|s| s.to_string()).collect();
    name_parts.push(spec.method.clone());
    let name = name_parts.join("_");
    (module, name)
}

fn mir_type_to_wasm_param(_ty: &crate::mir::MirType) -> String {
    // 現在の Mini-VM/WASM 実装ではハンドル/整数/BooI をすべて i32 で受け渡す。
    "i32".to_string()
}

fn mir_type_to_wasm_result(ty: &crate::mir::MirType) -> String {
    match ty {
        crate::mir::MirType::Void | crate::mir::MirType::Unknown => "".to_string(),
        _ => "i32".to_string(),
    }
}
