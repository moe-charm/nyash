use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationType { SelfRecursion, TailCall, PureInline, Escape }

#[derive(Debug, Clone)]
pub struct OptimizationEvent { pub location: (String, usize) }

#[derive(Debug, Default)]
pub struct OptimizationReporterBox { metrics: HashMap<OptimizationType, Vec<OptimizationEvent>> }

impl OptimizationReporterBox {
    pub fn new() -> Self { Default::default() }
    pub fn record(&mut self, ty: OptimizationType, location: (String, usize)) {
        self.metrics.entry(ty).or_default().push(OptimizationEvent { location });
    }
    pub fn to_json(&self) -> String {
        let mut out = Vec::new();
        for (ty, evs) in &self.metrics {
            let kind = match ty { OptimizationType::SelfRecursion => "selfrec", OptimizationType::TailCall => "tail", OptimizationType::PureInline => "pure", OptimizationType::Escape => "escape" };
            for e in evs {
                out.push(serde_json::json!({"kind": kind, "fn": e.location.0, "idx": e.location.1 }));
            }
        }
        serde_json::to_string(&out).unwrap_or_else(|_| "[]".to_string())
    }
}

