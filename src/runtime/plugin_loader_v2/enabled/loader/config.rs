use super::PluginLoaderV2;
use crate::bid::{BidError, BidResult};

pub(super) fn load_config(loader: &mut PluginLoaderV2, config_path: &str) -> BidResult<()> {
    let canonical = std::fs::canonicalize(config_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| config_path.to_string());
    loader.config_path = Some(canonical.clone());
    // Read once; cache both structured config and raw TOML value
    let content = std::fs::read_to_string(&canonical).map_err(|_| BidError::PluginError)?;
    loader.cached_toml = toml::from_str::<toml::Value>(&content).ok();
    loader.config = crate::config::nyash_toml_v2::NyashConfigV2::from_str(&content)
        .map(Some)
        .map_err(|_| BidError::PluginError)?;
    if let Some(cfg) = loader.config.as_ref() {
        let mut labels: Vec<String> = Vec::new();
        for (_lib, def) in &cfg.libraries {
            for bt in &def.boxes {
                labels.push(format!("BoxRef:{}", bt));
            }
        }
        crate::runtime::cache_versions::bump_many(&labels);
    }
    Ok(())
}
