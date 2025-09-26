use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct AotConfigBox {
    pub base: BoxBase,
    // staging fields (apply() writes to env)
    pub output_file: Option<String>,
    pub emit_obj_out: Option<String>,
    pub plugin_paths: Option<String>,
}

impl AotConfigBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
            output_file: None,
            emit_obj_out: None,
            plugin_paths: None,
        }
    }
}

impl BoxCore for AotConfigBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AotConfigBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for AotConfigBox {
    fn to_string_box(&self) -> StringBox {
        self.summary()
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<AotConfigBox>())
    }
    fn type_name(&self) -> &'static str {
        "AotConfigBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(Self {
            base: self.base.clone(),
            output_file: self.output_file.clone(),
            emit_obj_out: self.emit_obj_out.clone(),
            plugin_paths: self.plugin_paths.clone(),
        })
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl AotConfigBox {
    pub fn set_output(&mut self, path: &str) -> Box<dyn NyashBox> {
        self.output_file = Some(path.to_string());
        Box::new(VoidBox::new())
    }
    pub fn set_obj_out(&mut self, path: &str) -> Box<dyn NyashBox> {
        self.emit_obj_out = Some(path.to_string());
        Box::new(VoidBox::new())
    }
    pub fn set_plugin_paths(&mut self, paths: &str) -> Box<dyn NyashBox> {
        self.plugin_paths = Some(paths.to_string());
        Box::new(VoidBox::new())
    }
    pub fn clear(&mut self) -> Box<dyn NyashBox> {
        self.output_file = None;
        self.emit_obj_out = None;
        self.plugin_paths = None;
        Box::new(VoidBox::new())
    }

    /// Apply staged config to environment for CLI/runner consumption
    pub fn apply(&self) -> Box<dyn NyashBox> {
        if let Some(p) = &self.output_file {
            std::env::set_var("NYASH_AOT_OUT", p);
        }
        if let Some(p) = &self.emit_obj_out {
            std::env::set_var("NYASH_AOT_OBJECT_OUT", p);
        }
        if let Some(p) = &self.plugin_paths {
            std::env::set_var("NYASH_PLUGIN_PATHS", p);
        }
        Box::new(VoidBox::new())
    }

    pub fn summary(&self) -> StringBox {
        let s = format!(
            "output={} obj_out={} plugin_paths={}",
            self.output_file
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
            self.emit_obj_out
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
            self.plugin_paths
                .clone()
                .unwrap_or_else(|| "<none>".to_string()),
        );
        StringBox::new(s)
    }
}
