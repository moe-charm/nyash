# Phase 9.75f-3: 基本型動的化実験（第三段階・実験的）

## 🎯 目的
- String/Integer/Bool/Nullまで動的化する実験
- "Everything is Plugin"哲学の究極形
- ビルドを5秒以下にする野心的目標

## ⚠️ 警告
これは**実験的機能**です。以下のリスクがあります：
- 起動時間の増加（基本型ロード）
- デバッグの複雑化
- パフォーマンスオーバーヘッド

## 📋 実装計画

### Step 1: 最小コア定義
```rust
// nyash-core/src/minimal_core.rs
// 本当に必要な最小限のみ残す
pub trait MinimalBox: Send + Sync {
    fn type_id(&self) -> u64;
    fn as_ptr(&self) -> *const c_void;
}

// FFI境界用の最小構造
#[repr(C)]
pub struct FFIValue {
    type_id: u64,
    data_ptr: *mut c_void,
    vtable: *const FFIVTable,
}

#[repr(C)]
pub struct FFIVTable {
    drop: extern "C" fn(*mut c_void),
    clone: extern "C" fn(*const c_void) -> *mut c_void,
    to_string: extern "C" fn(*const c_void) -> *mut c_char,
}
```

### Step 2: 基本型プラグイン
```rust
// plugins/nyash-core-types/src/lib.rs
#[no_mangle]
extern "C" fn nyash_create_string(data: *const c_char) -> FFIValue {
    let s = unsafe { CStr::from_ptr(data).to_string_lossy().to_string() };
    let boxed = Box::new(StringData { value: s });
    
    FFIValue {
        type_id: STRING_TYPE_ID,
        data_ptr: Box::into_raw(boxed) as *mut c_void,
        vtable: &STRING_VTABLE,
    }
}

static STRING_VTABLE: FFIVTable = FFIVTable {
    drop: string_drop,
    clone: string_clone,
    to_string: string_to_string,
};

extern "C" fn string_drop(ptr: *mut c_void) {
    unsafe { Box::from_raw(ptr as *mut StringData); }
}

// Integer, Bool, Null も同様に実装
```

### Step 3: 起動時プリロード
```rust
// src/main.rs
fn initialize_core_plugins() -> Result<(), Error> {
    let registry = PLUGIN_REGISTRY.write().unwrap();
    
    // 基本型は起動時に必ずロード
    #[cfg(feature = "dynamic-core")]
    {
        registry.preload_plugin("core-types", "./plugins/libnyash_core_types.so")?;
        
        // 基本操作をキャッシュ
        registry.cache_constructor("StringBox");
        registry.cache_constructor("IntegerBox");
        registry.cache_constructor("BoolBox");
        registry.cache_constructor("NullBox");
    }
    
    Ok(())
}
```

### Step 4: リテラル処理の最適化
```rust
// src/interpreter/expressions/literals.rs
impl NyashInterpreter {
    fn evaluate_string_literal(&mut self, value: &str) -> Result<Box<dyn NyashBox>, RuntimeError> {
        #[cfg(feature = "static-core")]
        {
            Ok(Box::new(StringBox::new(value)))
        }
        
        #[cfg(feature = "dynamic-core")]
        {
            // キャッシュされたコンストラクタを使用
            let constructor = self.cached_constructors.get("StringBox").unwrap();
            let ffi_value = unsafe {
                constructor(CString::new(value)?.as_ptr())
            };
            
            Ok(Box::new(FFIBoxWrapper::new(ffi_value)))
        }
    }
}
```

### Step 5: JITライクな最適化
```rust
// src/interpreter/optimizer.rs
struct DynamicCallOptimizer {
    // よく使われる操作をインライン化
    hot_paths: HashMap<String, fn(&[FFIValue]) -> FFIValue>,
}

impl DynamicCallOptimizer {
    fn optimize_hot_path(&mut self, op: &str, count: usize) {
        if count > HOT_THRESHOLD {
            match op {
                "StringBox.concat" => {
                    // 頻繁に呼ばれる操作は専用パス
                    self.hot_paths.insert(op.to_string(), optimized_string_concat);
                }
                _ => {}
            }
        }
    }
}
```

## 🎯 実験的機能

### --dynamic-all フラグ
```bash
# 通常起動（基本型は静的）
./nyash program.nyash

# 完全動的モード（実験）
./nyash --dynamic-all program.nyash

# プロファイリングモード
./nyash --dynamic-all --profile program.nyash
```

### プラグイン統計
```
Plugin Load Statistics:
  core-types: 2.3ms (cached)
  math: 0.8ms (lazy)
  file: 1.2ms (on-demand)
  
Method Call Overhead:
  StringBox.concat: +15ns (optimized)
  IntegerBox.add: +12ns (optimized)
  FileBox.read: +3ns (already dynamic)
```

## 📊 ベンチマーク目標
- Hello Worldの起動: < 10ms（プラグインロード込み）
- 基本演算オーバーヘッド: < 20ns
- ビルド時間: 5秒以下
- バイナリサイズ: 500KB以下

## 🔮 超実験的アイデア

### ホットリロード
```rust
// 開発中にプラグインを再読み込み
./nyash --watch-plugins program.nyash
```

### WASM プラグイン
```rust
// プラグインもWASMで記述可能に
registry.load_wasm_plugin("custom-box.wasm")?;
```

### 分散プラグイン
```rust
// ネットワーク経由でプラグインロード（危険！）
registry.load_remote_plugin("https://plugins.nyash.dev/crypto-box")?;
```

## ⚠️ 既知の課題
1. **デバッグ体験**: スタックトレースが複雑化
2. **エラーメッセージ**: プラグイン境界でのエラーが分かりにくい
3. **セキュリティ**: 任意のプラグインロードは危険
4. **互換性**: プラグインABIバージョン管理が必要

## 📝 まとめ
Phase 9.75f-3は**純粋な実験**です。実用性より「どこまでできるか」の探求。
成功すれば革新的、失敗しても学びは大きい。