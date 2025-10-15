use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationType {
    SelfRecursion,
    TailCall,
    PureInline,
    Escape,
}

#[derive(Debug, Clone, Default)]
pub struct OptimizationConfigBox {
    pub self_rec_direct: bool,
    pub tail_call: bool,
    pub pure_inline: bool,
    pub escape_analysis: bool,
    pub trace_enabled: bool,
}

impl OptimizationConfigBox {
    pub fn from_env() -> Self {
        fn on(k: &str) -> bool {
            matches!(env::var(k).ok().as_deref(), Some("1") | Some("true") | Some("on"))
        }
        let hints = env::var("NYASH_MIR_HINTS").unwrap_or_default().to_lowercase();
        let has = |key: &str| hints.contains(key) || hints == "all";
        Self {
            self_rec_direct: on("NYASH_MIR_SELFREC_DIRECT"),
            tail_call: on("NYASH_LLVM_TAILCALL") || has("tail"),
            pure_inline: has("pure"),
            escape_analysis: has("escape"),
            trace_enabled: on("NYASH_MIR_OPTIMIZE_TRACE"),
        }
    }

    pub fn is_enabled(&self, ty: OptimizationType) -> bool {
        match ty {
            OptimizationType::SelfRecursion => self.self_rec_direct,
            OptimizationType::TailCall => self.tail_call,
            OptimizationType::PureInline => self.pure_inline,
            OptimizationType::Escape => self.escape_analysis,
        }
    }
}

