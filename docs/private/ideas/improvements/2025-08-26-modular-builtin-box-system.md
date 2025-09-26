# モジュラービルトインBoxシステム
Status: Pending
Created: 2025-08-26
Priority: Medium
Related: ビルトインBox管理の柔軟性向上

## 現状の問題
- ビルトインBoxは本体にハードコードされており、不要な機能も常に含まれる
- プラグインBoxはWASMでは動作しない（.so/.dll依存）
- 特殊用途のBox（MidnightBox等）が常にビルドに含まれる

## 提案：3層アーキテクチャ

```
┌─────────────────────────────────────┐
│        Core Builtin Boxes           │ ← 絶対必要（String, Integer等）
├─────────────────────────────────────┤
│     Optional Builtin Modules        │ ← 条件付きコンパイル（MidnightBox等）
├─────────────────────────────────────┤
│        Plugin Boxes (BID)           │ ← 外部プラグイン（FileBox等）
└─────────────────────────────────────┘
```

## 実装案

### 1. BuiltinModuleトレイト
```rust
pub trait BuiltinModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn create_box(&self, args: Vec<Box<dyn NyashBox>>) -> Option<Box<dyn NyashBox>>;
    fn required_features(&self) -> Vec<&'static str> { vec![] }
}

pub struct BuiltinModuleRegistry {
    modules: HashMap<String, Box<dyn BuiltinModule>>,
    enabled: HashSet<String>,
}
```

### 2. 設定ファイルベース管理
```toml
# nyash.toml
[builtin_modules]
core = ["StringBox", "IntegerBox", "BoolBox", "ArrayBox", "MapBox"]

[builtin_modules.optional]
midnight = { enabled = true, features = ["testnet"] }
egui = { enabled = false }
webcanvas = { enabled = true, target = "wasm32" }
```

### 3. Cargo features統合
```toml
[features]
default = ["core-boxes"]

# オプショナルモジュール
builtin-midnight = ["web-sys", "js-sys", "wasm-bindgen"]
builtin-egui = ["egui", "eframe"]
builtin-webcanvas = ["web-sys/CanvasRenderingContext2d"]

# プリセット
full = ["builtin-midnight", "builtin-egui", "builtin-webcanvas", "plugins"]
minimal = ["core-boxes"]
wasm-web = ["core-boxes", "builtin-webcanvas", "builtin-midnight"]
```

### 4. 動的登録マクロ
```rust
builtin_module! {
    MidnightModule,
    condition = cfg!(feature = "builtin-midnight")
}
```

## 利点
1. **バイナリサイズ削減**: 必要な機能のみ含める
2. **ビルド時間短縮**: 不要な依存関係を除外
3. **WASM対応**: Rust WASM機能をフル活用可能
4. **設定の柔軟性**: ビルド時/実行時の両方で制御
5. **後方互換性**: 既存のコードは変更不要

## 使用例

```nyash
// 条件付き実行
if hasBox("MidnightBox") {
    local voting = new VotingApp()
    voting.run()
} else {
    print("MidnightBox is not available in this build")
}
```

## ビルドコマンド例
```bash
# 最小構成
cargo build --release

# Midnight対応版
cargo build --release --features builtin-midnight

# フル機能版
cargo build --release --features full

# WASM Web版
cargo build --target wasm32-unknown-unknown --features wasm-web
```

## 実装タイミング
- Phase 10以降の最適化フェーズで検討
- プラグインシステムの成熟後に実装
- パフォーマンス問題が顕在化した場合に優先度上げ

## 関連課題
- プラグインシステムとの統合
- 設定ファイル管理の標準化
- ドキュメント自動生成（有効なBoxの一覧）