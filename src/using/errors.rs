//! Error helpers for using resolver (placeholder)

#[derive(thiserror::Error, Debug)]
pub enum UsingError {
    #[error("failed to read nyash.toml: {0}")]
    ReadToml(String),
    #[error("invalid nyash.toml format: {0}")]
    ParseToml(String),
    #[error("workspace dependency cycle detected: {0}")]
    Cycle(String),
    #[error("workspace missing dependency: {module} → {dep} ({req})")]
    MissingDep { module: String, dep: String, req: String },
    #[error("workspace namespace conflict: {ns} has multiple paths: {paths}")]
    Conflict { ns: String, paths: String },
}
