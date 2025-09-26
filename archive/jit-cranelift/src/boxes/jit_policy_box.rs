use crate::box_trait::{BoolBox, BoxBase, BoxCore, NyashBox, StringBox, VoidBox};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct JitPolicyBox {
    base: BoxBase,
}

impl JitPolicyBox {
    pub fn new() -> Self {
        Self {
            base: BoxBase::new(),
        }
    }
}

impl BoxCore for JitPolicyBox {
    fn box_id(&self) -> u64 {
        self.base.id
    }
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id
    }
    fn fmt_box(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JitPolicyBox")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl NyashBox for JitPolicyBox {
    fn to_string_box(&self) -> StringBox {
        let p = crate::jit::policy::current();
        let s = format!(
            "read_only={} whitelist={}",
            p.read_only,
            p.hostcall_whitelist.join(",")
        );
        StringBox::new(s)
    }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        BoolBox::new(other.as_any().is::<JitPolicyBox>())
    }
    fn type_name(&self) -> &'static str {
        "JitPolicyBox"
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(Self {
            base: self.base.clone(),
        })
    }
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

// Methods (exposed via VM dispatch):
impl JitPolicyBox {
    pub fn set_flag(&self, name: &str, on: bool) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        match name {
            "read_only" | "readonly" => cur.read_only = on,
            _ => return Box::new(StringBox::new(format!("Unknown flag: {}", name))),
        }
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }
    pub fn get_flag(&self, name: &str) -> Box<dyn NyashBox> {
        let cur = crate::jit::policy::current();
        let v = match name {
            "read_only" | "readonly" => cur.read_only,
            _ => false,
        };
        Box::new(BoolBox::new(v))
    }
    pub fn set_whitelist_csv(&self, csv: &str) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        cur.hostcall_whitelist = csv
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }

    pub fn add_whitelist(&self, name: &str) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        if !cur.hostcall_whitelist.iter().any(|s| s == name) {
            cur.hostcall_whitelist.push(name.to_string());
        }
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }

    pub fn clear_whitelist(&self) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        cur.hostcall_whitelist.clear();
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }

    pub fn enable_preset(&self, name: &str) -> Box<dyn NyashBox> {
        let mut cur = crate::jit::policy::current();
        match name {
            // 最小: Array.push_h のみ許可（読み取り以外は変えない）
            "mutating_minimal" | "mutating_array_push" => {
                let id = crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H;
                if !cur.hostcall_whitelist.iter().any(|s| s == id) {
                    cur.hostcall_whitelist.push(id.to_string());
                }
            }
            // 例: Map.set_h も追加許可（必要に応じて拡張）
            "mutating_map_set" => {
                let id = crate::jit::r#extern::collections::SYM_MAP_SET_H;
                if !cur.hostcall_whitelist.iter().any(|s| s == id) {
                    cur.hostcall_whitelist.push(id.to_string());
                }
            }
            // よく使う: Array.push_h + Array.set_h + Map.set_h を許可
            "mutating_common" => {
                let ids = [
                    crate::jit::r#extern::collections::SYM_ARRAY_PUSH_H,
                    crate::jit::r#extern::collections::SYM_ARRAY_SET_H,
                    crate::jit::r#extern::collections::SYM_MAP_SET_H,
                ];
                for id in ids {
                    if !cur.hostcall_whitelist.iter().any(|s| s == id) {
                        cur.hostcall_whitelist.push(id.to_string());
                    }
                }
            }
            _ => {
                return Box::new(StringBox::new(format!("Unknown preset: {}", name)));
            }
        }
        crate::jit::policy::set_current(cur);
        Box::new(VoidBox::new())
    }
}
