use crate::bid::{BidError, BidResult, LoadedPlugin};
use crate::bid::tlv::{TlvEncoder, TlvDecoder};
use crate::bid::types::BidTag;
use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};
use std::any::Any;
use std::fmt;

/// 汎用プラグインBoxインスタンス
/// 任意のプラグインBoxを統一的に扱える
pub struct GenericPluginBox {
    base: BoxBase,
    plugin: &'static LoadedPlugin,
    instance_id: u32,
    box_name: String,
}

impl GenericPluginBox {
    /// 汎用的なプラグインBoxを作成（birth呼び出し）
    pub fn birth(plugin: &'static LoadedPlugin, box_name: String) -> BidResult<Self> {
        eprintln!("🔍 GenericPluginBox::birth for '{}'", box_name);
        
        // birthメソッド（method_id = 0）を呼び出し
        let mut out = Vec::new();
        plugin.handle.invoke(plugin.type_id, 0, 0, &[], &mut out)?;
        
        // インスタンスIDを取得
        let instance_id = if out.len() == 4 {
            u32::from_le_bytes([out[0], out[1], out[2], out[3]])
        } else {
            return Err(BidError::InvalidArgs);
        };
        
        eprintln!("✅ Created {} instance with ID: {}", box_name, instance_id);
        
        Ok(Self {
            base: BoxBase::new(),
            plugin,
            instance_id,
            box_name,
        })
    }
    
    /// 汎用メソッド呼び出し
    pub fn call_method(&self, method_name: &str, args: &[u8]) -> BidResult<Vec<u8>> {
        eprintln!("🔍 GenericPluginBox::call_method '{}' on {}", method_name, self.box_name);
        
        // プラグインからメソッドIDを動的取得
        match self.plugin.find_method(method_name) {
            Ok(Some((method_id, _signature))) => {
                eprintln!("✅ Found method '{}' with ID: {}", method_name, method_id);
                
                let mut out = Vec::new();
                self.plugin.handle.invoke(
                    self.plugin.type_id,
                    method_id,
                    self.instance_id,
                    args,
                    &mut out
                )?;
                
                Ok(out)
            }
            Ok(None) => {
                eprintln!("❌ Method '{}' not found in {}", method_name, self.box_name);
                Err(BidError::InvalidArgs)
            }
            Err(e) => Err(e)
        }
    }
}

impl Drop for GenericPluginBox {
    fn drop(&mut self) {
        // finiメソッド（method_id = u32::MAX）を呼び出し
        let _ = self.plugin.handle.invoke(
            self.plugin.type_id,
            u32::MAX,
            self.instance_id,
            &[],
            &mut Vec::new(),
        );
    }
}

impl BoxCore for GenericPluginBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(plugin)", self.box_name)
    }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl NyashBox for GenericPluginBox {
    fn to_string_box(&self) -> StringBox {
        StringBox::new(format!("{}(plugin)", self.box_name))
    }
    
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(other_plugin) = other.as_any().downcast_ref::<GenericPluginBox>() {
            BoolBox::new(
                self.box_name == other_plugin.box_name &&
                self.instance_id == other_plugin.instance_id
            )
        } else {
            BoolBox::new(false)
        }
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // 新しいインスタンスを作成
        if let Some(reg) = crate::bid::registry::global() {
            if let Some(plugin) = reg.get_by_name(&self.box_name) {
                if let Ok(new_box) = GenericPluginBox::birth(plugin, self.box_name.clone()) {
                    return Box::new(new_box);
                }
            }
        }
        Box::new(StringBox::new("<plugin clone failed>"))
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}

impl fmt::Debug for GenericPluginBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenericPluginBox({}, instance={})", self.box_name, self.instance_id)
    }
}

impl fmt::Display for GenericPluginBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_box(f)
    }
}