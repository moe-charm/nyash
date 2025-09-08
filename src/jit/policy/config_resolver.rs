use std::sync::{OnceLock};

use crate::config::nyash_toml_v2::NyashConfigV2;

#[derive(Clone, Debug)]
pub struct MethodInfo {
    pub type_id: u32,
    pub method_id: u32,
    pub returns_result: bool,
}

static CFG_CACHE: OnceLock<Option<NyashConfigV2>> = OnceLock::new();
static RAW_TOML: OnceLock<Option<toml::Value>> = OnceLock::new();

fn load_config() -> (Option<&'static NyashConfigV2>, Option<&'static toml::Value>) {
    let cfg = CFG_CACHE.get_or_init(|| {
        let path = std::env::var("NYASH_CONFIG").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "nyash.toml".to_string());
        NyashConfigV2::from_file(&path).ok()
    });
    let raw = RAW_TOML.get_or_init(|| {
        let path = std::env::var("NYASH_CONFIG").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "nyash.toml".to_string());
        std::fs::read_to_string(&path).ok().and_then(|s| toml::from_str::<toml::Value>(&s).ok())
    });
    (cfg.as_ref(), raw.as_ref())
}

pub fn resolve_method_from_config(box_type: &str, method: &str) -> Option<MethodInfo> {
    let (cfg, raw) = load_config();
    let cfg = cfg?; let raw = raw?;
    let (lib_name, _lib) = cfg.find_library_for_box(box_type)?;
    let box_conf = cfg.get_box_config(lib_name, box_type, raw)?;
    let m = box_conf.methods.get(method)?;
    Some(MethodInfo { type_id: box_conf.type_id, method_id: m.method_id, returns_result: m.returns_result })
}

