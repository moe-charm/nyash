//! VTable codegen stubs (WASM v2)
//!
//! - 将来的にTypeRegistryから静的テーブル/インライン分岐を生成
//! - 現段階はプレースホルダ

#![cfg(feature = "wasm-backend")]

/// 生成結果のメタ情報（雛形）
pub struct GeneratedVTableInfo {
    pub types: usize,
    pub methods: usize,
}

pub fn generate_tables() -> GeneratedVTableInfo {
    // 未実装: TypeRegistry::resolve_typebox_by_name()/methods を走査して集計
    GeneratedVTableInfo {
        types: 0,
        methods: 0,
    }
}
