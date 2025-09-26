/*!
 * TypeBox ABI (Tier-0 雛形)
 *
 * 目的:
 * - Phase 12 で導入する Nyash ABI (vtable) の型定義と最小構造を先置きするための雛形。
 * - 現段階では参照型と関数ポインタの骨組みのみ提供し、実呼び出しは行わない（常に未処理）。
 */

/// Nyash ABI における最小の値タグ（雛形）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NyrtTag {
    Void,
    I64,
    F64,
    Bool,
    String,
    Handle,
}

/// Nyash ABI の値表現（雛形）
#[derive(Debug, Clone)]
pub struct NyrtValue {
    pub tag: NyrtTag,
    pub i64_: i64,
}

impl NyrtValue {
    pub fn void() -> Self {
        Self {
            tag: NyrtTag::Void,
            i64_: 0,
        }
    }
    pub fn i64(v: i64) -> Self {
        Self {
            tag: NyrtTag::I64,
            i64_: v,
        }
    }
}

/// Nyash ABI のメソッド関数ポインタ（雛形）
pub type NyrtMethodFn = fn(instance: u64, argc: usize, argv: *const NyrtValue) -> NyrtValue;

/// スロット定義（雛形）
#[derive(Clone, Copy)]
pub struct MethodEntry {
    pub name: &'static str,
    pub arity: u8,
    pub slot: u16,
}

/// TypeBox（雛形）: 各型の静的メタデータ（スロット一覧付き）
pub struct TypeBox {
    pub type_name: &'static str,
    pub methods: &'static [MethodEntry],
}

impl TypeBox {
    pub const fn new(type_name: &'static str) -> Self {
        Self {
            type_name,
            methods: &[],
        }
    }
    pub const fn new_with(type_name: &'static str, methods: &'static [MethodEntry]) -> Self {
        Self { type_name, methods }
    }
}
