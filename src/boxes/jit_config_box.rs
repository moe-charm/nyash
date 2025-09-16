use crate::box_trait::{BoolBox, BoxBase, BoxCore, IntegerBox, NyashBox, StringBox, VoidBox};
use crate::interpreter::RuntimeError;
use crate::jit::config::JitConfig;
use std::any::Any;
use std::sync::RwLock;

#[derive(Debug)]
pub struct JitConfigBox {
    base: BoxBase,
    pub config: RwLock<JitConfig>,
}

impl JitConfigBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            config: RwLock::new(JitConfig::from_env()),
        }
    }
    /// Update internal config flags from runtime capability probe
    pub fn from_runtime_probe(&self) -> Box<dyn NyashBox> {
        let caps = crate::jit::config::probe_capabilities();
        let mut cfg = self.config.write().unwrap();
        if cfg.native_bool_abi && !caps.supports_b1_sig {
            cfg.native_bool_abi = false;
        }
        Box::new(VoidBox::new())
    }
    pub fn set_flag(&self, name: &str, on: bool) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut cfg = self.config.write().unwrap();
        match name {
            "exec" => cfg.exec = on,
            "stats" => cfg.stats = on,
            "stats_json" => cfg.stats_json = on,
            "dump" => cfg.dump = on,
            "phi_min" => cfg.phi_min = on,
            "hostcall" => cfg.hostcall = on,
            "handle_debug" => cfg.handle_debug = on,
            "native_f64" => cfg.native_f64 = on,
            "native_bool" => cfg.native_bool = on,
            "bool_abi" | "native_bool_abi" => cfg.native_bool_abi = on,
            "ret_b1" | "ret_bool_b1" => cfg.ret_bool_b1 = on,
            "relax_numeric" | "hostcall_relax_numeric" => cfg.relax_numeric = on,
            _ => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown flag: {}", name),
                })
            }
        }
        Ok(Box::new(VoidBox::new()))
    }
    pub fn get_flag(&self, name: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let cfg = self.config.read().unwrap();
        let b = match name {
            "exec" => cfg.exec,
            "stats" => cfg.stats,
            "stats_json" => cfg.stats_json,
            "dump" => cfg.dump,
            "phi_min" => cfg.phi_min,
            "hostcall" => cfg.hostcall,
            "handle_debug" => cfg.handle_debug,
            "native_f64" => cfg.native_f64,
            "native_bool" => cfg.native_bool,
            "bool_abi" | "native_bool_abi" => cfg.native_bool_abi,
            "ret_b1" | "ret_bool_b1" => cfg.ret_bool_b1,
            "relax_numeric" | "hostcall_relax_numeric" => cfg.relax_numeric,
            _ => {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("Unknown flag: {}", name),
                })
            }
        };
        Ok(Box::new(BoolBox::new(b)))
    }
    pub fn set_threshold(&self, v: i64) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut cfg = self.config.write().unwrap();
        if v <= 0 {
            cfg.threshold = None;
        } else {
            cfg.threshold = Some(v as u32);
        }
        Ok(Box::new(VoidBox::new()))
    }
    pub fn get_threshold(&self) -> Box<dyn NyashBox> {
        let cfg = self.config.read().unwrap();
        Box::new(IntegerBox::new(
            cfg.threshold.map(|v| v as i64).unwrap_or(0),
        ))
    }
    pub fn to_json(&self) -> Box<dyn NyashBox> {
        let cfg = self.config.read().unwrap();
        let val = serde_json::json!({
            "exec": cfg.exec,
            "stats": cfg.stats,
            "stats_json": cfg.stats_json,
            "dump": cfg.dump,
            "threshold": cfg.threshold,
            "phi_min": cfg.phi_min,
            "hostcall": cfg.hostcall,
            "handle_debug": cfg.handle_debug,
            "native_f64": cfg.native_f64,
            "native_bool": cfg.native_bool,
            "native_bool_abi": cfg.native_bool_abi,
            "ret_bool_b1": cfg.ret_bool_b1,
            "relax_numeric": cfg.relax_numeric,
        });
        Box::new(StringBox::new(val.to_string()))
    }
    pub fn from_json(&self, s: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        let mut cfg = self.config.write().unwrap();
        let v: serde_json::Value =
            serde_json::from_str(s).map_err(|e| RuntimeError::InvalidOperation {
                message: format!("Invalid JSON: {}", e),
            })?;
        if let Some(b) = v.get("exec").and_then(|x| x.as_bool()) {
            cfg.exec = b;
        }
        if let Some(b) = v.get("stats").and_then(|x| x.as_bool()) {
            cfg.stats = b;
        }
        if let Some(b) = v.get("stats_json").and_then(|x| x.as_bool()) {
            cfg.stats_json = b;
        }
        if let Some(b) = v.get("dump").and_then(|x| x.as_bool()) {
            cfg.dump = b;
        }
        if let Some(n) = v.get("threshold").and_then(|x| x.as_u64()) {
            cfg.threshold = Some(n as u32);
        }
        if let Some(b) = v.get("phi_min").and_then(|x| x.as_bool()) {
            cfg.phi_min = b;
        }
        if let Some(b) = v.get("hostcall").and_then(|x| x.as_bool()) {
            cfg.hostcall = b;
        }
        if let Some(b) = v.get("handle_debug").and_then(|x| x.as_bool()) {
            cfg.handle_debug = b;
        }
        if let Some(b) = v.get("native_f64").and_then(|x| x.as_bool()) {
            cfg.native_f64 = b;
        }
        if let Some(b) = v.get("native_bool").and_then(|x| x.as_bool()) {
            cfg.native_bool = b;
        }
        if let Some(b) = v.get("native_bool_abi").and_then(|x| x.as_bool()) {
            cfg.native_bool_abi = b;
        }
        if let Some(b) = v.get("ret_bool_b1").and_then(|x| x.as_bool()) {
            cfg.ret_bool_b1 = b;
        }
        if let Some(b) = v.get("relax_numeric").and_then(|x| x.as_bool()) {
            cfg.relax_numeric = b;
        }
        Ok(Box::new(VoidBox::new()))
    }
    pub fn apply(&self) -> Box<dyn NyashBox> {
        let cfg = self.config.read().unwrap().clone();
        // Apply to env for CLI parity
        cfg.apply_env();
        // Also set global current JIT config for hot paths (env-less)
        crate::jit::config::set_current(cfg);
        Box::new(VoidBox::new())
    }
    pub fn summary(&self) -> Box<dyn NyashBox> {
        let cfg = self.config.read().unwrap();
        let s = format!(
            "exec={} stats={} json={} dump={} thr={:?} phi_min={} hostcall={} hdbg={} f64={} bool={} relax_numeric={}",
            cfg.exec, cfg.stats, cfg.stats_json, cfg.dump, cfg.threshold,
            cfg.phi_min, cfg.hostcall, cfg.handle_debug, cfg.native_f64, cfg.native_bool, cfg.relax_numeric
        );
        Box::new(StringBox::new(s))
    }
}

impl BoxCore for JitConfigBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitConfigBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitConfigBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(self.summary().to_string_box().value)
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitConfigBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitConfigBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        let cfg = self.config.read().unwrap().clone();
        Box::new(JitConfigBox {
            base: self.base.clone(),
            config: RwLock::new(cfg),
        })
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
