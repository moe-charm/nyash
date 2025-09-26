use crate::ast::ASTNode;
use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug)]
pub struct ClosureEnv {
    pub me_value: Option<Weak<dyn NyashBox>>, // Weak me (upgrade at call)
    pub captures: HashMap<String, Box<dyn NyashBox>>, // P1: by-value captures
}

impl ClosureEnv {
    pub fn new() -> Self {
        Self {
            me_value: None,
            captures: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionBox {
    pub params: Vec<String>,
    pub body: Vec<ASTNode>,
    pub env: ClosureEnv,
    base: BoxBase,
}

impl FunctionBox {
    pub fn new(params: Vec<String>, body: Vec<ASTNode>) -> Self {
        Self {
            params,
            body,
            env: ClosureEnv::new(),
            base: BoxBase::new(),
        }
    }
    pub fn with_env(params: Vec<String>, body: Vec<ASTNode>, env: ClosureEnv) -> Self {
        Self {
            params,
            body,
            env,
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for FunctionBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FunctionBox(params={}, body={})",
            self.params.len(),
            self.body.len()
        )
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for FunctionBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!(
            "FunctionBox(params={}, captures={}, body={})",
            self.params.len(),
            self.env.captures.len(),
            self.body.len()
        ))
    }
    fn type_name(&self) -> &'static str {
        "FunctionBox"
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(o) = other.as_any().downcast_ref::<FunctionBox>() {
            BoolBox::new(self.box_id() == o.box_id())
        } else {
            BoolBox::new(false)
        }
    }
}

impl Clone for ClosureEnv {
    fn clone(&self) -> Self {
        let me_value = self.me_value.as_ref().map(|w| Weak::clone(w));
        let mut captures: HashMap<String, Box<dyn NyashBox>> = HashMap::new();
        for (k, v) in &self.captures {
            captures.insert(k.clone(), v.clone_box());
        }
        Self { me_value, captures }
    }
}

impl Clone for FunctionBox {
    fn clone(&self) -> Self {
        Self {
            params: self.params.clone(),
            body: self.body.clone(),
            env: self.env.clone(),
            base: BoxBase::new(),
        }
    }
}
