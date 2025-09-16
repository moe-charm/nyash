use std::{env, fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SugarLevel {
    None,
    Basic,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SugarConfig {
    pub level: SugarLevel,
}

impl Default for SugarConfig {
    fn default() -> Self {
        Self {
            level: SugarLevel::None,
        }
    }
}

impl SugarConfig {
    pub fn from_env_or_toml(path: impl AsRef<Path>) -> Self {
        // 1) env override
        if let Ok(s) = env::var("NYASH_SYNTAX_SUGAR_LEVEL") {
            return Self {
                level: parse_level(&s),
            };
        }
        // 2) toml [syntax].sugar_level
        let path = path.as_ref();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(val) = toml::from_str::<toml::Value>(&content) {
                if let Some(table) = val.get("syntax").and_then(|v| v.as_table()) {
                    if let Some(level_str) = table.get("sugar_level").and_then(|v| v.as_str()) {
                        return Self {
                            level: parse_level(level_str),
                        };
                    }
                }
            }
        }
        // 3) default
        Self::default()
    }
}

fn parse_level(s: &str) -> SugarLevel {
    match s.to_ascii_lowercase().as_str() {
        "basic" => SugarLevel::Basic,
        "full" => SugarLevel::Full,
        _ => SugarLevel::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn env_precedence_over_toml() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("nyash.toml");
        fs::write(&file, "[syntax]\nsugar_level='full'\n").unwrap();
        env::set_var("NYASH_SYNTAX_SUGAR_LEVEL", "basic");
        let cfg = SugarConfig::from_env_or_toml(&file);
        env::remove_var("NYASH_SYNTAX_SUGAR_LEVEL");
        assert_eq!(cfg.level, SugarLevel::Basic);
    }

    #[test]
    fn toml_level_when_env_absent() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("nyash.toml");
        fs::write(&file, "[syntax]\nsugar_level='basic'\n").unwrap();
        let cfg = SugarConfig::from_env_or_toml(&file);
        assert_eq!(cfg.level, SugarLevel::Basic);
    }

    #[test]
    fn default_none_on_missing_or_invalid() {
        // Ensure env override is not present for this test
        env::remove_var("NYASH_SYNTAX_SUGAR_LEVEL");
        let dir = tempdir().unwrap();
        let file = dir.path().join("nyash.toml");
        fs::write(&file, "[syntax]\nsugar_level='unknown'\n").unwrap();
        let cfg = SugarConfig::from_env_or_toml(&file);
        assert_eq!(cfg.level, SugarLevel::None);
        let cfg2 = SugarConfig::from_env_or_toml(dir.path().join("missing.toml"));
        assert_eq!(cfg2.level, SugarLevel::None);
    }
}
