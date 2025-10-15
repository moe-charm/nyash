#![doc = "このファイルは vm_ops 層の責務を定義します"]
// vm_ops は命令族ごとの薄い箱（Box）境界を提供する層です。
// ここでは VM 本体（mir_interpreter）から呼ばれる最小APIのみを公開し、
// 具体実装は family ごとのモジュールに委譲します。

pub const LAYER_NAME: &str = "vm_ops";
pub const ALLOWED_IMPORTS: &[&str] = &[
    // VM型・エラー
    "backend::vm_types",
    // MIR 種別（CompareOp 等）
    "mir",
    // TypeRegistry 参照（arity/slot）
    "runtime::type_registry",
];
pub const FORBIDDEN_IMPORTS: &[&str] = &[
    // 実行器の詳細（直接依存を避ける）
    "backend::mir_interpreter::handlers",
];
