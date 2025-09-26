//! GC mode selection (user-facing)
use crate::config::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcMode {
    RcCycle,
    Minorgen,
    STW,
    Rc,
    Off,
}

impl GcMode {
    pub fn from_env() -> Self {
        match env::gc_mode().to_ascii_lowercase().as_str() {
            "auto" | "rc+cycle" => GcMode::RcCycle,
            "minorgen" => GcMode::Minorgen,
            "stw" => GcMode::STW,
            "rc" => GcMode::Rc,
            "off" => GcMode::Off,
            _ => GcMode::RcCycle,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            GcMode::RcCycle => "rc+cycle",
            GcMode::Minorgen => "minorgen",
            GcMode::STW => "stw",
            GcMode::Rc => "rc",
            GcMode::Off => "off",
        }
    }
}

