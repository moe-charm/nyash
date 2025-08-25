# ビルトイン⇔プラグインBox相互変換システム
Status: Proposal
Created: 2025-08-25
Priority: High
Related: Everything is Box哲学の現実的実装

## 概要

ビルトインBoxとプラグインBoxを自由に相互変換できるシステムの提案。
同一ソースコードから両形態を自動生成し、用途に応じて使い分ける。

## 基本アーキテクチャ

```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ Box定義     │ ──→  │ 変換層      │ ──→  │ 生成物      │
│ (単一ソース)│      │ (自動生成)  │      │ (両形態)    │
└─────────────┘      └─────────────┘      └─────────────┘
     ↓                      ↓                      ↓
 #[nyash_box]          nyash-box-gen         .rlib + .so
```

## 実装設計

### 1. 統一Box定義フォーマット

```rust
// boxes/string_box/src/lib.rs
#[nyash_box(
    id = "string",
    version = "1.0.0",
    capabilities = ["core", "serializable"]
)]
pub struct StringBox {
    value: String,
}

#[nyash_box_impl]
impl StringBox {
    #[nyash_constructor]
    pub fn new(s: &str) -> Self {
        Self { value: s.to_string() }
    }
    
    #[nyash_method]
    pub fn length(&self) -> i64 {
        self.value.len() as i64
    }
    
    #[nyash_method(name = "toString")]
    pub fn to_string(&self) -> String {
        self.value.clone()
    }
}
```

### 2. 自動生成される形態

#### ビルトイン版（静的リンク用）
```rust
// 自動生成: target/builtin/string_box.rs
use crate::box_trait::{NyashBox, BoxCore};

impl BoxCore for StringBox {
    fn box_id(&self) -> u64 { /* 自動生成 */ }
    fn fmt_box(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StringBox({})", self.value)
    }
}

impl NyashBox for StringBox {
    fn type_name(&self) -> &'static str { "StringBox" }
    
    fn to_string_box(&self) -> StringBox {
        StringBox::new(&self.value)
    }
    
    // メソッドディスパッチ
    fn call_method(&self, method: &str, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, String> {
        match method {
            "length" => Ok(Box::new(IntegerBox::new(self.length()))),
            "toString" => Ok(Box::new(StringBox::new(&self.to_string()))),
            _ => Err(format!("Unknown method: {}", method))
        }
    }
}
```

#### プラグイン版（動的ロード用）
```rust
// 自動生成: target/plugin/string_box_plugin.rs
use std::ffi::{c_char, c_void};

#[repr(C)]
pub struct StringBoxFFI {
    handle: u64,
}

#[no_mangle]
pub extern "C" fn nyash_box_register(host: *const NyHostV1) -> *const NyBoxV1 {
    // ホストAPIの保存とBox登録
}

#[no_mangle]
pub extern "C" fn string_new(s: *const c_char) -> StringBoxFFI {
    let s = unsafe { CStr::from_ptr(s).to_string_lossy() };
    let box_instance = StringBox::new(&s);
    // ハンドル管理に登録して返す
}

#[no_mangle]
pub extern "C" fn string_length(handle: StringBoxFFI) -> i64 {
    // ハンドルから実体を取得してメソッド呼び出し
}
```

### 3. ビルドシステム

#### nyash-box-gen ツール
```toml
# nyash-box-gen/Cargo.toml
[package]
name = "nyash-box-gen"
version = "0.1.0"

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

#### 使用方法
```bash
# 両形態を生成
nyash-box-gen generate \
    --input boxes/string_box/src/lib.rs \
    --builtin-out target/builtin/string_box.rs \
    --plugin-out target/plugin/string_box_plugin.rs

# ディレクトリ一括処理
nyash-box-gen batch \
    --input-dir boxes/ \
    --builtin-dir target/builtin/ \
    --plugin-dir target/plugin/
```

### 4. ビルド設定

```toml
# Nyash本体のCargo.toml
[features]
# 個別Box制御
static-string-box = []
static-integer-box = []
static-console-box = []

# 一括制御
all-static = ["static-string-box", "static-integer-box", ...]
all-dynamic = []
hybrid-performance = ["static-string-box", "static-integer-box"]  # よく使うものだけ静的
```

### 5. ランタイムでの透過的利用

```rust
// src/runtime/box_loader.rs
pub struct BoxLoader {
    static_boxes: HashMap<&'static str, fn() -> Box<dyn NyashBox>>,
    plugin_loader: PluginLoaderV2,
}

impl BoxLoader {
    pub fn load_box(&self, name: &str) -> Result<Box<dyn NyashBox>, Error> {
        // 静的版が有効なら優先
        #[cfg(feature = "static-string-box")]
        if name == "StringBox" {
            return Ok(Box::new(StringBox::new("")));
        }
        
        // なければプラグインから
        self.plugin_loader.load_box(name)
    }
}
```

## 高度な機能

### 1. ホットスワップ開発モード
```rust
// 開発時のみ有効
#[cfg(feature = "dev-mode")]
impl BoxLoader {
    pub fn reload_box(&mut self, name: &str) -> Result<(), Error> {
        // プラグインを再ロード（ビルトインは不可）
        self.plugin_loader.reload(name)
    }
}
```

### 2. パフォーマンスプロファイリング
```rust
// 両形態の性能を比較
nyash-box-gen benchmark \
    --box string \
    --iterations 1000000 \
    --compare static vs dynamic
```

### 3. 移行ツール
```rust
// 既存ビルトインBoxをプラグイン化
nyash-box-gen migrate \
    --from src/boxes/old_string_box.rs \
    --to boxes/string_box/src/lib.rs \
    --infer-annotations
```

## 利点

1. **柔軟性**: 同一コードで両形態を維持
2. **パフォーマンス**: 必要に応じて静的リンク選択可
3. **開発効率**: プラグインでホットリロード開発
4. **互換性**: 既存コードへの影響最小
5. **段階的移行**: Boxごとに個別に移行可能

## 実装ロードマップ

### Phase 1: 基礎実装（1週間）
- [ ] nyash-box-genの基本実装
- [ ] マクロ（#[nyash_box]等）の定義
- [ ] 単純なBoxでのプロトタイプ

### Phase 2: 統合（2週間）
- [ ] BoxLoaderの実装
- [ ] ビルドシステムとの統合
- [ ] 既存Boxの移行

### Phase 3: 最適化（1週間）
- [ ] ベンチマークシステム
- [ ] ホットリロード機能
- [ ] デバッグサポート

## まとめ

このシステムにより、「Everything is Box」の理想を保ちながら、
現実的なパフォーマンスと開発効率を両立できる。
純粋主義と実用主義の最適なバランスを実現する。