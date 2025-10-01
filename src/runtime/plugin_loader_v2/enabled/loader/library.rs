use super::specs;
use super::util::dbg_on;
use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};
use crate::config::nyash_toml_v2::LibraryDefinition;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn load_all_plugins(loader: &PluginLoaderV2) -> BidResult<()> {
    let config = loader.config.as_ref().ok_or(BidError::PluginError)?;
    for (lib_name, lib_def) in &config.libraries {
        load_plugin(loader, lib_name, lib_def)?;
    }
    for (plugin_name, root) in &config.plugins {
        load_plugin_from_root(loader, plugin_name, root)?;
    }
    super::singletons::prebirth_singletons(loader)?;
    Ok(())
}

pub(super) fn load_plugin(
    loader: &PluginLoaderV2,
    lib_name: &str,
    lib_def: &LibraryDefinition,
) -> BidResult<()> {
    let base = Path::new(&lib_def.path);
    let candidates = candidate_paths(base);
    let mut lib_path = candidates.iter().find(|p| p.exists()).cloned();
    if lib_path.is_none() {
        if let Some(cfg) = &loader.config {
            for candidate in &candidates {
                if let Some(fname) = candidate.file_name().and_then(|s| s.to_str()) {
                    if let Some(resolved) = cfg.resolve_plugin_path(fname) {
                        let pb = PathBuf::from(resolved);
                        if pb.exists() {
                            lib_path = Some(pb);
                            break;
                        }
                    }
                }
            }
        }
    }
    let lib_path = lib_path.unwrap_or_else(|| base.to_path_buf());
    if dbg_on() {
        eprintln!(
            "[PluginLoaderV2] load_plugin: lib='{}' path='{}'",
            lib_name,
            lib_path.display()
        );
    }
    let lib = unsafe { Library::new(&lib_path) }.map_err(|_| BidError::PluginError)?;
    let lib_arc = Arc::new(lib);

    unsafe {
        if let Ok(init_sym) =
            lib_arc.get::<Symbol<unsafe extern "C" fn() -> i32>>(b"nyash_plugin_init\0")
        {
            let _ = init_sym();
        }
    }

    let loaded = super::super::types::LoadedPluginV2 {
        _lib: lib_arc.clone(),
        box_types: lib_def.boxes.clone(),
        typeboxes: HashMap::new(),
        init_fn: None,
    };
    loader
        .plugins
        .write()
        .map_err(|_| BidError::PluginError)?
        .insert(lib_name.to_string(), Arc::new(loaded));

    for box_type in &lib_def.boxes {
        let sym_name = format!("nyash_typebox_{}\0", box_type);
        unsafe {
            if let Ok(tb_sym) =
                lib_arc.get::<Symbol<&super::super::types::NyashTypeBoxFfi>>(sym_name.as_bytes())
            {
                specs::record_typebox_spec(loader, lib_name, box_type, &*tb_sym)?;
            } else if dbg_on() {
                eprintln!(
                    "[PluginLoaderV2] NOTE: TypeBox symbol not found for {}.{} (symbol='{}'). Migrate plugin to Nyash ABI v2 to enable per-Box dispatch.",
                    lib_name,
                    box_type,
                    sym_name.trim_end_matches('\0')
                );
            }
        }
        // Optional: probe Final ABI (env-gated) — no behavior change when absent
        if crate::config::env::plugin_abi_final() {
            let final_sym = format!("nyash_typebox_final_{}\0", box_type);
            unsafe {
                if let Ok(f_sym) = lib_arc.get::<Symbol<&super::super::types::NyashTypeBoxFinalFfi>>(
                    final_sym.as_bytes(),
                ) {
                    let _ = specs::record_typebox_final_spec(loader, lib_name, box_type, &*f_sym);
                    if crate::config::env::plugin_meta() && super::util::dbg_on() {
                        eprintln!(
                            "[PluginLoaderV2] Final ABI available for {}.{} (symbol='{}')",
                            lib_name,
                            box_type,
                            final_sym.trim_end_matches('\0')
                        );
                    }
                } else if crate::config::env::plugin_meta() && super::util::dbg_on() {
                    eprintln!(
                        "[PluginLoaderV2] Final ABI not found for {}.{} (looked for '{}')",
                        lib_name,
                        box_type,
                        final_sym.trim_end_matches('\0')
                    );
                }
            }
        }
    }

    Ok(())
}

pub(super) fn load_plugin_from_root(
    _loader: &PluginLoaderV2,
    _plugin_name: &str,
    _root: &str,
) -> BidResult<()> {
    Ok(())
}

fn candidate_paths(base: &Path) -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if cfg!(target_os = "windows") {
        candidates.push(base.with_extension("dll"));
        if let Some(file) = base.file_name().and_then(|s| s.to_str()) {
            if file.starts_with("lib") {
                let mut alt = base.to_path_buf();
                let alt_file = file.trim_start_matches("lib");
                alt.set_file_name(alt_file);
                candidates.push(alt.with_extension("dll"));
            }
        }
    } else if cfg!(target_os = "macos") {
        candidates.push(base.with_extension("dylib"));
    } else {
        candidates.push(base.with_extension("so"));
    }
    candidates
}
