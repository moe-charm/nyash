# Phase 9.75f: ビルトインBox動的ライブラリ分離アーキテクチャ

## 🎯 目的
- ビルド時間を2分→15秒に短縮
- バイナリサイズを15MB→2MBに削減
- Box単位での独立開発を可能に

## 📋 Gemini先生からのアドバイス

### 1. **C ABI + libloading が最も安定**
```rust
#[no_mangle]
extern "C" fn nyash_file_read(path: *const c_char) -> *mut c_char {
    // 実装
}
```

### 2. **段階的移行戦略**
- Phase 1: インタープリターでExternCall直接実行
- Phase 2: FileBox/ConsoleBoxをプラグイン化
- Phase 3: 残りのBox順次移行

### 3. **メモリ管理の重要性**
- 所有権ルールを明確に
- データ生成側が解放関数も提供
- Arc<RwLock>は直接共有不可→ハンドルパターン使用

## 🚀 実装計画

### Step 1: インタープリターExternCall（即実装可能）
```rust
// interpreter/expressions.rs
impl NyashInterpreter {
    fn execute_extern_call(&mut self, 
        iface: &str, 
        method: &str, 
        args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        match (iface, method) {
            ("env.file", "read") => {
                // 直接実行
            }
        }
    }
}
```

### Step 2: プラグインAPI定義
```rust
#[repr(C)]
pub struct PluginAPI {
    pub version: u32,
    pub name: *const c_char,
    pub methods: *const MethodTable,
}
```

### Step 3: ワークスペース構成
```toml
[workspace]
members = [
    "nyash-core",       # 2MB
    "nyash-plugin-api", # 共通API
    "plugins/io",       # FileBox, ConsoleBox
    "plugins/web",      # CanvasBox
]
```

## ⚠️ 注意点
- プラグイン間の直接依存は避ける
- セキュリティ考慮（信頼できるソースのみ）
- クロスプラットフォーム対応（.so/.dll/.dylib）