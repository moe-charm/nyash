use super::specs;
use super::util::dbg_on;
use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};
use crate::config::nyash_toml_v2::LibraryDefinition;
use libloading::{Library, Symbol};
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
            "[PluginLoaderV2] load_plugin: lib='{}' path='{}' boxes={:?}",
            lib_name,
            lib_path.display(),
            &lib_def.boxes
        );
    }
    let lib = unsafe { Library::new(&lib_path) }.map_err(|err| {
        if super::util::dbg_on() {
            eprintln!("[PluginLoaderV2] load_plugin dlopen failed lib='{}' path='{}': {}", lib_name, lib_path.display(), err);
        }
        BidError::PluginError
    })?;
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
    };
    loader
        .plugins
        .write()
        .map_err(|_| BidError::PluginError)?
        .insert(lib_name.to_string(), Arc::new(loaded));

    for box_type in &lib_def.boxes {
        let sym_name = format!("nyash_typebox_{}\0", box_type);
        unsafe {
            if super::util::dbg_on() {
                eprintln!("[PluginLoaderV2] DEBUG typebox probe start {}.{}", lib_name, box_type);
            }
            match lib_arc.get::<Symbol<&super::super::types::NyashTypeBoxFfi>>(sym_name.as_bytes()) {
                Ok(tb_sym) => {
                    if super::util::dbg_on() {
                        eprintln!("[PluginLoaderV2] DEBUG typebox ok {}.{}", lib_name, box_type);
                    }
                    super::util::dbg_once(
                        &format!("typebox_present:{}:{}", lib_name, box_type),
                        &format!(
                            "[PluginLoaderV2] TypeBox present for {}.{} (symbol='{}')",
                            lib_name,
                            box_type,
                            sym_name.trim_end_matches('\0')
                        ),
                    );
                    specs::record_typebox_spec(loader, lib_name, box_type, &*tb_sym)?;
                }
                Err(err) => {
                    if super::util::dbg_on() {
                        eprintln!("[PluginLoaderV2] TypeBox lookup failed for {}.{} (symbol='{}'): {}", lib_name, box_type, sym_name.trim_end_matches('\0'), err);
                    }
                    super::util::dbg_once(
                        &format!("typebox_missing:{}:{}", lib_name, box_type),
                        &format!(
                            "[PluginLoaderV2] NOTE: TypeBox symbol not found for {}.{} (symbol='{}'). Migrate plugin to Nyash ABI v2 to enable per-Box dispatch.",
                            lib_name,
                            box_type,
                            sym_name.trim_end_matches('\0')
                        ),
                    );
                    // Attempt a unified re-probe path (records invoke id if available later)
                    let _ = super::metadata::probe_and_record_typebox_invoke(loader, lib_name, box_type);
                }
            }
        }
        // Opportunistically ingest nyash_box.toml/hako_box.toml located near the library path
        // to populate type_id and method ids even when a central nyash.toml is not fully loaded.
        {
            if let Some(spec_path) = crate::runtime::spec_ingest_box::find_near_spec(&lib_path, 5) {
                super::util::dbg_once(
                    &format!("spec_probe:{}:{}", lib_name, spec_path.display()),
                    &format!(
                        "[PluginLoaderV2] spec ingest: probing {} for {} boxes {:?}",
                        spec_path.display(),
                        lib_name,
                        &lib_def.boxes
                    ),
                );
                // Ingest via SpecIngestBox facade to decouple callers from specs module
                crate::runtime::spec_ingest_box::ingest_from_path(
                    loader,
                    lib_name,
                    &lib_def.boxes,
                    &spec_path,
                );
                // If spec ingest didn't provide type_id, record defaults for core boxes
                if let Some(spec_cur) = super::specs::get_spec(loader, lib_name, box_type) {
                    if spec_cur.type_id.is_none() {
                        let default_tid = match box_type.as_str() {
                            "ArrayBox" => Some(12u32),
                            "MapBox" => Some(11u32),
                            "StringBox" => Some(13u32),
                            _ => None,
                        };
                        if let Some(tid) = default_tid {
                            if let Ok(mut map) = loader.box_specs.write() {
                                let key = (lib_name.to_string(), box_type.to_string());
                                let entry = map.entry(key).or_insert_with(|| spec_cur.clone());
                                entry.type_id = Some(tid);
                                super::util::dbg_once(&format!("spec_tid_fallback:{}:{}", lib_name, box_type), &format!("[PluginLoaderV2] spec ingest: default type_id={} recorded for {}.{}", tid, lib_name, box_type));
                            }
                        }
                    }
                }

            } else {
                super::util::dbg_once(
                    &format!("spec_near_missing:{}", lib_path.display()),
                    &format!(
                        "[PluginLoaderV2] spec ingest: no nyash_box/hako_box next to '{}' (searched up to 5 parents)",
                        lib_path.display()
                    ),
                );
                // Even when spec is missing, record known type_id defaults for core boxes
                let default_tid = match box_type.as_str() {
                    "ArrayBox" => Some(12u32),
                    "MapBox" => Some(11u32),
                    "StringBox" => Some(13u32),
                    _ => None,
                };
                if let Some(tid) = default_tid {
                    if let Ok(mut map) = loader.box_specs.write() {
                        let key = (lib_name.to_string(), box_type.to_string());
                        let entry = map.entry(key).or_default();
                        if entry.type_id.is_none() {
                            entry.type_id = Some(tid);
                            super::util::dbg_once(&format!("spec_tid_default:{}:{}", lib_name, box_type), &format!("[PluginLoaderV2] recorded default type_id={} for {}.{} (no near-spec)", tid, lib_name, box_type));
                        }
                    }
                }

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

/// Public (crate) entry to ingest from a spec path — thin wrapper to keep `specs` private.
#[allow(dead_code)]
pub(crate) fn ingest_from_spec_path(
    loader: &super::PluginLoaderV2,
    lib_name: &str,
    box_names: &[String],
    spec_path: &Path,
) {
    specs::ingest_box_specs_from_nyash_box(loader, lib_name, box_names, spec_path);
}
