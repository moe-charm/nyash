/*!
 * BoxIndex — minimal view over aliases and plugin box types
 *
 * Purpose: allow using/namespace resolver to make decisions that depend
 * on plugin-visible type names (e.g., enforcing strict prefix rules) and
 * to surface aliases defined in nyash.toml/env.
 */

use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use std::sync::RwLock;

#[derive(Clone, Default)]
pub struct BoxIndex {
    pub aliases: HashMap<String, String>,
    pub plugin_boxes: HashSet<String>,
    pub plugin_meta: HashMap<String, PluginMeta>,
    pub plugins_require_prefix_global: bool,
}

impl BoxIndex {
    pub fn build_current() -> Self {
        // aliases from nyash.toml and env
        let mut aliases: HashMap<String, String> = HashMap::new();
        if let Ok(text) = std::fs::read_to_string("nyash.toml") {
            if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                if let Some(alias_tbl) = doc.get("aliases").and_then(|v| v.as_table()) {
                    for (k, v) in alias_tbl.iter() {
                        if let Some(target) = v.as_str() { aliases.insert(k.to_string(), target.to_string()); }
                    }
                }
            }
        }
        if let Ok(raw) = std::env::var("NYASH_ALIASES") {
            for ent in raw.split(',') {
                if let Some((k,v)) = ent.split_once('=') {
                    let k = k.trim(); let v = v.trim();
                    if !k.is_empty() && !v.is_empty() { aliases.insert(k.to_string(), v.to_string()); }
                }
            }
        }

        // plugin box types (best-effort; may be empty if host not initialized yet)
        let mut plugin_boxes: HashSet<String> = HashSet::new();
        let mut plugin_meta: HashMap<String, PluginMeta> = HashMap::new();
        let mut plugins_require_prefix_global = false;

        // Read per-plugin meta and global flags from nyash.toml when available
        if let Ok(text) = std::fs::read_to_string("nyash.toml") {
            if let Ok(doc) = toml::from_str::<toml::Value>(&text) {
                if let Some(plugins_tbl) = doc.get("plugins").and_then(|v| v.as_table()) {
                    // Global switch: [plugins].require_prefix = true
                    if let Some(v) = plugins_tbl.get("require_prefix").and_then(|v| v.as_bool()) {
                        plugins_require_prefix_global = v;
                    }
                    for (k, v) in plugins_tbl.iter() {
                        // Skip non-table entries (string entries are plugin roots)
                        if let Some(t) = v.as_table() {
                            let prefix = t.get("prefix").and_then(|x| x.as_str()).map(|s| s.to_string());
                            let require_prefix = t.get("require_prefix").and_then(|x| x.as_bool()).unwrap_or(false);
                            let expose_short_names = t.get("expose_short_names").and_then(|x| x.as_bool()).unwrap_or(true);
                            plugin_meta.insert(k.clone(), PluginMeta { prefix, require_prefix, expose_short_names });
                        }
                    }
                }
            }
        }
        let host = crate::runtime::get_global_plugin_host();
        if let Ok(h) = host.read() {
            if let Some(cfg) = h.config_ref() {
                for (_lib, def) in &cfg.libraries {
                    for bt in &def.boxes { plugin_boxes.insert(bt.clone()); }
                }
            }
        }

        Self { aliases, plugin_boxes, plugin_meta, plugins_require_prefix_global }
    }

    pub fn is_known_plugin_short(name: &str) -> bool {
        // Prefer global index view
        if GLOBAL.read().ok().map(|g| g.plugin_boxes.contains(name)).unwrap_or(false) {
            return true;
        }
        // Env override list
        if let Ok(raw) = std::env::var("NYASH_KNOWN_PLUGIN_SHORTNAMES") {
            let set: HashSet<String> = raw.split(',').map(|s| s.trim().to_string()).collect();
            if set.contains(name) { return true; }
        }
        // Minimal fallback set
        const KNOWN: &[&str] = &[
            "ArrayBox","MapBox","StringBox","ConsoleBox","FileBox","PathBox","MathBox","IntegerBox","TOMLBox"
        ];
        KNOWN.iter().any(|k| *k == name)
    }
}

// Global BoxIndex view (rebuilt on-demand)
static GLOBAL: Lazy<RwLock<BoxIndex>> = Lazy::new(|| RwLock::new(BoxIndex::default()));

// Global resolve cache (keyed by tgt|base|strict|paths)
static RESOLVE_CACHE: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn refresh_box_index() {
    let next = BoxIndex::build_current();
    if let Ok(mut w) = GLOBAL.write() { *w = next; }
}

pub fn get_box_index() -> BoxIndex {
    GLOBAL.read().ok().map(|g| g.clone()).unwrap_or_default()
}

pub fn cache_get(key: &str) -> Option<String> {
    RESOLVE_CACHE.read().ok().and_then(|m| m.get(key).cloned())
}

pub fn cache_put(key: &str, value: String) {
    if let Ok(mut m) = RESOLVE_CACHE.write() { m.insert(key.to_string(), value); }
}

pub fn cache_clear() {
    if let Ok(mut m) = RESOLVE_CACHE.write() { m.clear(); }
}

#[derive(Clone, Debug, Default)]
pub struct PluginMeta {
    pub prefix: Option<String>,
    pub require_prefix: bool,
    pub expose_short_names: bool,
}

pub fn get_plugin_meta(plugin: &str) -> Option<PluginMeta> {
    GLOBAL.read().ok().and_then(|g| g.plugin_meta.get(plugin).cloned())
}
