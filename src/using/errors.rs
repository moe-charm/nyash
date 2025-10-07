//! Error helpers for using resolver (placeholder)

#[derive(thiserror::Error, Debug)]
pub enum UsingError {
    #[error("failed to read nyash.toml: {0}")]
    ReadToml(String),
    #[error("invalid nyash.toml format: {0}")]
    ParseToml(String),
    #[error("workspace dependency cycle detected: {0}")]
    Cycle(String),
}
