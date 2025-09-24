//! Using specification models (skeleton)

#[derive(Debug, Clone)]
pub enum UsingTarget {
    /// Logical package name (to be resolved via nyash.toml)
    Package(String),
    /// Source file path (absolute or relative)
    SourcePath(String),
    /// Dynamic library path (plugin)
    DylibPath(String),
}

#[derive(Debug, Clone)]
pub struct UsingSpec {
    pub target: UsingTarget,
    pub alias: Option<String>,
    pub expose: Option<Vec<String>>, // planned
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageKind {
    Package,
    Dylib,
}

impl PackageKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dylib" => PackageKind::Dylib,
            _ => PackageKind::Package,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsingPackage {
    pub kind: PackageKind,
    pub path: String,
    pub main: Option<String>,
    pub bid: Option<String>,
}

