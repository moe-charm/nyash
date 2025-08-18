use crate::bid::{BidError, BidResult, LoadedPlugin};
use crate::bid::tlv::{TlvEncoder, TlvDecoder};
use crate::bid::types::BidTag;
use crate::bid::metadata::{NyashMethodInfo, NyashPluginInfo};
use crate::box_trait::{NyashBox, StringBox, BoolBox, BoxCore, BoxBase};

/// Minimal plugin-backed instance that manages birth/fini lifecycle
pub struct PluginBoxInstance<'a> {
    pub plugin: &'a LoadedPlugin,
    pub instance_id: u32,
}

impl<'a> PluginBoxInstance<'a> {
    /// Create a new instance by invoking METHOD_BIRTH (0)
    pub fn birth(plugin: &'a LoadedPlugin) -> BidResult<Self> {
        let mut out = Vec::new();
        plugin.handle.invoke(plugin.type_id, 0, 0, &[], &mut out)?;
        // Expect TLV encoding of handle or instance id; current prototype returns raw u32
        let instance_id = if out.len() == 4 {
            u32::from_le_bytes([out[0], out[1], out[2], out[3]])
        } else {
            // Try to decode TLV handle (future-proof)
            return Err(BidError::invalid_args());
        };
        Ok(Self { plugin, instance_id })
    }

    // Method IDs are fixed for FileBox in BID-1 prototype:
    // 1=open, 2=read, 3=write, 4=close

    pub fn open(&self, path: &str, mode: &str) -> BidResult<()> {
        let method = 1; // open
        let mut enc = TlvEncoder::new();
        enc.encode_string(path)?;
        enc.encode_string(mode)?;
        let args = enc.finish();
        let mut out = Vec::new();
        self.plugin.handle.invoke(self.plugin.type_id, method, self.instance_id, &args, &mut out)?;
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> BidResult<i32> {
        let method = 3; // write
        let mut enc = TlvEncoder::new();
        enc.encode_bytes(data)?;
        let args = enc.finish();
        let mut out = Vec::new();
        self.plugin.handle.invoke(self.plugin.type_id, method, self.instance_id, &args, &mut out)?;
        let mut dec = TlvDecoder::new(&out)?;
        if let Some((tag, payload)) = dec.decode_next()? {
            if tag != BidTag::I32 { return Err(BidError::invalid_type()); }
            return Ok(TlvDecoder::decode_i32(payload)?);
        }
        Err(BidError::plugin_error())
    }

    pub fn read(&self, size: usize) -> BidResult<Vec<u8>> {
        let method = 2; // read
        let mut enc = TlvEncoder::new();
        enc.encode_i32(size as i32)?;
        let args = enc.finish();
        let mut out = Vec::new();
        self.plugin.handle.invoke(self.plugin.type_id, method, self.instance_id, &args, &mut out)?;
        let mut dec = TlvDecoder::new(&out)?;
        if let Some((tag, payload)) = dec.decode_next()? {
            if tag != BidTag::Bytes { return Err(BidError::invalid_type()); }
            return Ok(payload.to_vec());
        }
        Err(BidError::plugin_error())
    }

    pub fn close(&self) -> BidResult<()> {
        let method = 4; // close
        let mut enc = TlvEncoder::new();
        enc.encode_void()?;
        let args = enc.finish();
        let mut out = Vec::new();
        self.plugin.handle.invoke(self.plugin.type_id, method, self.instance_id, &args, &mut out)?;
        Ok(())
    }
}

impl<'a> Drop for PluginBoxInstance<'a> {
    fn drop(&mut self) {
        // METHOD_FINI = u32::MAX
        let _ = self.plugin.handle.invoke(
            self.plugin.type_id,
            u32::MAX,
            self.instance_id,
            &[],
            &mut Vec::new(),
        );
    }
}

/// NyashBox implementation wrapping a BID plugin FileBox instance
pub struct PluginFileBox {
    base: BoxBase,
    inner: PluginBoxInstance<'static>,
    path: String,
}

impl PluginFileBox {
    pub fn new(plugin: &'static LoadedPlugin, path: String) -> BidResult<Self> {
        let inst = PluginBoxInstance::birth(plugin)?;
        // Open with read-write by default (compat with built-in)
        inst.open(&path, "rw")?;
        Ok(Self { base: BoxBase::new(), inner: inst, path })
    }

    pub fn read_bytes(&self, size: usize) -> BidResult<Vec<u8>> { self.inner.read(size) }
    pub fn write_bytes(&self, data: &[u8]) -> BidResult<i32> { self.inner.write(data) }
    pub fn close(&self) -> BidResult<()> { self.inner.close() }

    /// 汎用メソッド呼び出し（動的ディスパッチ）
    pub fn call_method(&self, method_name: &str, args: &[u8]) -> BidResult<Vec<u8>> {
        eprintln!("🔍 call_method: method_name='{}', args_len={}", method_name, args.len());
        
        // プラグインからメソッドIDを動的取得
        match self.inner.plugin.find_method(method_name) {
            Ok(Some((method_id, signature))) => {
                eprintln!("🔍 Found method '{}': ID={}, signature=0x{:08X}", method_name, method_id, signature);
                let mut out = Vec::new();
                match self.inner.plugin.handle.invoke(
                    self.inner.plugin.type_id,
                    method_id,
                    self.inner.instance_id,
                    args,
                    &mut out
                ) {
                    Ok(()) => {
                        eprintln!("🔍 Plugin invoke succeeded, output_len={}", out.len());
                        Ok(out)
                    }
                    Err(e) => {
                        eprintln!("🔍 Plugin invoke failed: {:?}", e);
                        Err(e)
                    }
                }
            }
            Ok(None) => {
                eprintln!("🔍 Method '{}' not found in plugin", method_name);
                Err(BidError::invalid_args()) // メソッドが見つからない
            }
            Err(e) => {
                eprintln!("🔍 Error looking up method '{}': {:?}", method_name, e);
                Err(e)
            }
        }
    }

    /// プラグインのメソッド一覧を取得
    pub fn get_available_methods(&self) -> BidResult<Vec<(u32, String, u32)>> {
        self.inner.plugin.get_methods()
    }
}

impl BoxCore for PluginFileBox {
    fn box_id(&self) -> u64 { self.base.id }
    fn parent_type_id(&self) -> Option<std::any::TypeId> { self.base.parent_type_id }
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "FileBox({}) [plugin]", self.path)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl NyashBox for PluginFileBox {
    fn to_string_box(&self) -> StringBox { StringBox::new(format!("FileBox({})", self.path)) }
    fn equals(&self, other: &dyn NyashBox) -> BoolBox {
        if let Some(of) = other.as_any().downcast_ref::<PluginFileBox>() {
            BoolBox::new(self.path == of.path)
        } else { BoolBox::new(false) }
    }
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // Create a new plugin-backed instance to the same path
        if let Some(reg) = crate::bid::registry::global() {
            if let Some(plugin) = reg.get_by_name("FileBox") {
                if let Ok(newb) = PluginFileBox::new(plugin, self.path.clone()) {
                    return Box::new(newb);
                }
            }
        }
        Box::new(StringBox::new("<plugin clone failed>"))
    }
    fn share_box(&self) -> Box<dyn NyashBox> { self.clone_box() }
}

impl std::fmt::Debug for PluginFileBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PluginFileBox(path={})", self.path)
    }
}
