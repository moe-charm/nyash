# Phase 9.75f-2: Math/Time系Box動的ライブラリ化（第二段階）

## 🎯 目的
- FileBox成功を受けて、Math/Random/Time系を動的化
- 複数Boxの単一ライブラリ化パターン検証
- ビルド時間追加短縮（目標: さらに30秒短縮）

## 📋 実装計画

### Step 1: プラグイン構成
```toml
# plugins/nyash-math/Cargo.toml
[package]
name = "nyash-math"
crate-type = ["cdylib"]

[dependencies]
rand = "0.8"
chrono = "0.4"
```

### Step 2: 統合プラグインAPI
```rust
// plugins/nyash-math/src/lib.rs
#[repr(C)]
pub struct MathPlugin {
    magic: u32,
    version: u32,
    // 複数Box型を1つのプラグインで提供
    box_types: *const BoxTypeInfo,
    box_count: usize,
}

#[repr(C)]
pub struct BoxTypeInfo {
    name: *const c_char,        // "MathBox", "RandomBox", etc.
    constructor: extern "C" fn() -> *mut c_void,
    methods: *const MethodInfo,
    method_count: usize,
}

#[repr(C)]
pub struct MethodInfo {
    name: *const c_char,
    func: extern "C" fn(*mut c_void, *const c_void) -> *mut c_void,
}

// プラグイン初期化
#[no_mangle]
extern "C" fn nyash_plugin_init() -> *const MathPlugin {
    static BOX_TYPES: &[BoxTypeInfo] = &[
        BoxTypeInfo {
            name: c"MathBox",
            constructor: math_box_new,
            methods: &MATH_METHODS,
            method_count: MATH_METHODS.len(),
        },
        BoxTypeInfo {
            name: c"RandomBox",
            constructor: random_box_new,
            methods: &RANDOM_METHODS,
            method_count: RANDOM_METHODS.len(),
        },
        BoxTypeInfo {
            name: c"TimeBox",
            constructor: time_box_new,
            methods: &TIME_METHODS,
            method_count: TIME_METHODS.len(),
        },
    ];
    
    Box::into_raw(Box::new(MathPlugin {
        magic: 0x4E594153,
        version: 1,
        box_types: BOX_TYPES.as_ptr(),
        box_count: BOX_TYPES.len(),
    }))
}
```

### Step 3: メソッド実装
```rust
// MathBox methods
static MATH_METHODS: &[MethodInfo] = &[
    MethodInfo { name: c"sin", func: math_sin },
    MethodInfo { name: c"cos", func: math_cos },
    MethodInfo { name: c"sqrt", func: math_sqrt },
    MethodInfo { name: c"pow", func: math_pow },
];

extern "C" fn math_sin(_self: *mut c_void, args: *const c_void) -> *mut c_void {
    // 引数をFloatBoxとして解釈
    // sin計算
    // 結果をFloatBoxとして返す
}

// RandomBox methods
static RANDOM_METHODS: &[MethodInfo] = &[
    MethodInfo { name: c"int", func: random_int },
    MethodInfo { name: c"float", func: random_float },
    MethodInfo { name: c"choice", func: random_choice },
];

// TimeBox methods  
static TIME_METHODS: &[MethodInfo] = &[
    MethodInfo { name: c"now", func: time_now },
    MethodInfo { name: c"format", func: time_format },
    MethodInfo { name: c"add", func: time_add },
];
```

### Step 4: 改良されたプラグインローダー
```rust
// src/interpreter/plugin_loader.rs
pub struct PluginRegistry {
    plugins: HashMap<String, LoadedPlugin>,
    box_registry: HashMap<String, BoxTypeEntry>,
}

struct LoadedPlugin {
    library: Library,
    info: PluginInfo,
}

struct BoxTypeEntry {
    plugin_name: String,
    type_info: BoxTypeInfo,
}

impl PluginRegistry {
    pub fn load_plugin(&mut self, name: &str, path: &str) -> Result<(), Error> {
        let lib = unsafe { Library::new(path)? };
        
        // プラグイン初期化
        let init_fn: Symbol<unsafe extern "C" fn() -> *const MathPlugin> = 
            unsafe { lib.get(b"nyash_plugin_init")? };
        let plugin_info = unsafe { &*init_fn() };
        
        // Box型を登録
        for i in 0..plugin_info.box_count {
            let box_info = unsafe { &*plugin_info.box_types.add(i) };
            let box_name = unsafe { CStr::from_ptr(box_info.name).to_string_lossy() };
            
            self.box_registry.insert(
                box_name.to_string(),
                BoxTypeEntry {
                    plugin_name: name.to_string(),
                    type_info: *box_info,
                }
            );
        }
        
        self.plugins.insert(name.to_string(), LoadedPlugin { library: lib, info: *plugin_info });
        Ok(())
    }
}
```

### Step 5: インタープリター統合
```rust
// src/interpreter/objects.rs
impl NyashInterpreter {
    fn execute_new_dynamic(&mut self, box_name: &str, args: Vec<Box<dyn NyashBox>>) 
        -> Result<Box<dyn NyashBox>, RuntimeError> {
        
        let registry = self.plugin_registry.read().unwrap();
        
        if let Some(entry) = registry.box_registry.get(box_name) {
            // 動的ライブラリ経由でコンストラクタ呼び出し
            let handle = unsafe { (entry.type_info.constructor)() };
            
            Ok(Box::new(DynamicBoxProxy {
                handle,
                type_name: box_name.to_string(),
                type_info: entry.type_info.clone(),
            }))
        } else {
            Err(RuntimeError::UndefinedBox { name: box_name.to_string() })
        }
    }
}
```

## 🎯 成功条件
1. ✅ Math/Random/Timeの全メソッドが動的ライブラリ経由で動作
2. ✅ 1つのプラグインで複数Box型を提供
3. ✅ ビルド時間がさらに30秒短縮
4. ✅ プラグイン遅延ロード（使用時のみ）
5. ✅ 静的版と同等のパフォーマンス

## 📊 ベンチマーク項目
- Math演算1000回のオーバーヘッド
- Random生成のスループット
- Time操作のレイテンシ
- プラグインロード時間（初回/キャッシュ後）

## 🔮 将来の拡張
- プラグイン自動検出（plugins/ディレクトリスキャン）
- バージョン管理とアップグレード
- プラグイン間依存関係