use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct AotCompilerBox { base: BoxBase }

impl AotCompilerBox { pub fn new() -> Self { Self { base: BoxBase::new() } } }

impl BoxCore for AotCompilerBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "AotCompilerBox") }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl NyashBox for AotCompilerBox {
    fn to_string_box(&self) -> StringBox { StringBox::new("AotCompilerBox".to_string()) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox { BoolBox::new(other.as_any().is::<AotCompilerBox>()) }
    fn type_name(&self) -> &'static str { "AotCompilerBox" }
    fn clone_box(&self) -> Box<dyn NyashBox> { Box::new(Self { base: self.base.clone() }) }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

impl AotCompilerBox {
    /// Compile a .nyash file into native EXE by re-invoking current binary with --compile-native.
    /// Returns combined stdout/stderr as String.
    pub fn compile(&self, file: &str, out: &str) -> Box<dyn NyashBox> {
        let mut cmd = match std::env::current_exe() {
            Ok(p) => std::process::Command::new(p),
            Err(e) => return Box::new(StringBox::new(format!("ERR: current_exe(): {}", e)))
        };
        // Propagate relevant envs (AOT/JIT observe)
        let mut c = cmd.arg("--backend").arg("vm") // ensures runner path
                      .arg("--compile-native")
                      .arg("-o").arg(out)
                      .arg(file)
                      .envs(std::env::vars());
        match c.output() {
            Ok(o) => {
                let mut s = String::new();
                s.push_str(&String::from_utf8_lossy(&o.stdout));
                s.push_str(&String::from_utf8_lossy(&o.stderr));
                if !o.status.success() {
                    s = format!("AOT FAILED (code={}):\n{}", o.status.code().unwrap_or(-1), s);
                }
                Box::new(StringBox::new(s))
            }
            Err(e) => Box::new(StringBox::new(format!("ERR: spawn compile-native: {}", e)))
        }
    }
}

