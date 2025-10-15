# VM Plugin Integration仕様書

Note: Terminology updated — “Nyash ABI” is now referred to as “Hako ABI (formerly Nyash ABI)”.

## Policy & Lifecycle — Final Rules (Phase 15.7)

### Strict Plugin Policy（HAKO_PLUGIN_POLICY=force）

- plugins が ON かつ対象 Box に Plugin provider が存在する場合、VM ルーターは builtin へのフォールバックを禁止して Fail‑Fast します。
- 未実装・未知メソッドは即時エラーとなり、原因が隠蔽されません。
- 代表エラー: `plugin strict: builtin fallback disabled for MapBox.noSuchMethod(0 args)`
- 推奨運用:
  - plugins プロファイルやCIの一部で Strict を有効化（フォールバック検出）。
  - 互換や観測を優先する quick プロファイルでは `auto` を維持。

### Length 系の統一（String/Array）

- String/Array の `size/len/length` は Extern に正規化して実装を一本化します。
  - String: `Extern("nyrt.string.length")` — 受けは String 値（BoxRef の場合は文字列化）
  - Array:  `Extern("nyrt.array.size")` — 受けは Array（HostHandle 経路: slot 102）
- Builder 側:
  - 受けの materialize は EmitGuard（finalize_call_operands）で一度だけ実施し、正規化（normalize_*）では再materializeしない。
  - これにより、未定義 ValueId の生成（use-before-def）を防止。
- VM 側:
  - Method(String/Array).length 系は早期Externに橋渡しする（安全弁）。
  - Extern 実装は HostHandle/legacy の双方を吸収する。

### SetBox — Extern 経路（Map ベース）

Set は Map の意味論（Eq/Hash/決定性）を再利用する。VM は Extern("nyrt.set.*") で受け、内部的に Map に委譲する。

Extern I/O（最小）
- `nyrt.set.add(recv:Set, v:any) -> Void`（NullBox）
- `nyrt.set.remove(recv:Set, v:any) -> Void`（NullBox）
- `nyrt.set.has(recv:Set, v:any) -> Bool`
- `nyrt.set.size(recv:Set) -> i64`
- `nyrt.set.clear(recv:Set) -> Void`（NullBox）
- `nyrt.set.toArray(recv:Set) -> Array`

実装方針
- HostHandle あり: `recv` を Map の HostHandle として扱い、`set/get/has/size/clear/keys` を利用（Unit 値で add/remove を表現）。
- legacy 互換: 内部 MapBox を保持する SetBox でも同一の外部 Extern を経由（ABI 安定）。
- Strict（policy=force）下でも挙動は同一（フォールバック禁止）。
 - プロバイダ: `plugins/nyash-set-plugin` が `SetBox` を提供。`hako.toml`/`nyash.toml` の `[libraries."libnyash_set_plugin.so".SetBox]` で type_id とメソッドIDを定義。


## Capabilities (Policy Hooks)

See `docs/reference/plugin-system/capabilities.md` for capability bit definitions (IO/NET/ENV/TIME/...).
- Deterministic runs deny IO/NET boxes.
- Plugins should set `NyashTypeBoxFfi.capabilities` appropriately.


- Plugin Policy: default ON (auto). If no plugins are configured in hako.toml/nyash.toml, nothing is loaded (no side‑effects). CI などで完全遮断したい場合のみ `NYASH_DISABLE_PLUGINS=1` を使う。
- Creation: `new T(args…)` is always followed by `birth(me,args…)` by VM. When `birth` is not implemented, it is treated as no‑op (idempotent). Builder の auto‑birth は既定OFF。
- Plugin Init: two idempotent stages are allowed (optional)
  - Load‑time: `nyash_plugin_init()` called once per library when present
  - First‑birth: plugin may call `ensure_ready()` guarded by `Once`
- Provider Resolution: single order — `PluginProvider(T) → BuiltinProvider(T) → Registry/Fallback(T) → error`. Before resolving, the registry performs on‑demand re‑probe for `T` to avoid timing issues.
  - プラグイン設定で `boxes = [T]` が宣言されている場合は plugin-only とみなし、プラグイン経路が失敗したらそのままエラーを返す（ビルトインフォールバック禁止）。
  - `HAKO_PLUGIN_ON_STRICT=1`（互換: `NYASH_PLUGIN_ON_STRICT=1`）を指定すると、最終フォールバックも抑止して Fail-Fast する。
- Boot Disabled Non‑cache: boot() no longer caches “disabled” as success (allows later retry when policy flips to ON). Operationally we run with policy=auto by default so this path is rarely used.
- Stage-2 handles: `Map.keys()/values()` は既定で HostHandle(ArrayBox) を返す（Phase 15.7+）。
  - 互換フラグ `NYASH_PLUGIN_MAP_ARRAY_HANDLE` は移行期の歴史的フラグ（未設定でも有効）。
  - values() の要素には PluginHandle(tag=8)（例: ArrayBox）が含まれ得る。Host 側は tag=8/9 を decode し、グローバルハンドルキャッシュで identity を再利用する。
  - Router の文字列シム（keysS/valuesS→Array 正規化）は撤退済み（Phase 15.7+）。


## 📦 Phase 15.7–15.75 Structural Boxes（HostHandleRouter フェーズイン）

- `src/runtime/method_router_box/method_ref.rs` が methodRef 疑似メソッドを担当。VM ルーターは最初にここへ委譲し、型チェックと CallableBox 生成を一箇所で行う。 (詳細: docs/reference/plugin-system/callable-box-guide.md)
- `src/runtime/method_router_box/map_callable.rs` に Map.call/Map.callAsync の糖衣実装を隔離。プラグインは get/set 群だけ実装すれば良く、call 系は VM 側で一貫化。
- `src/runtime/codec/codec_box.rs` は TLV エンコード/デコードの単一窓口。Host/Plugin ハンドル、コア Box の扱いをここで統制し、plugin_ffi_common と同じポリシーを維持する。
- ディレクトリ README (`src/runtime/codec/README.md`) で境界の責務を明示。将来 helper を増やす場合もこの箱を経由する。

### HostHandleRouter（段階導入）

- 入口: `src/runtime/host_handle_router/mod.rs` に HostHandle 経由メソッドを slot で受ける薄いルーターを配置。
- 対応スロット（15.75 現在）:
  - ArrayBox: `len → 102`
  - MapBox: `size → 200`, `has → 202`, `get → 203`, `set → 204`
  - StringBox: `len → 300`
- VM からの強制経路（開発用）
  - `NYASH_MAP_FORCE_HOST=1` で Map.size/has/get/set を HostHandleRouter に強制。
  - `NYASH_ARRAY_FORCE_HOST=1` で Array.size/get/set を HostHandleRouter に強制（`NYASH_ARRAY_SIZE_FORCE_HOST` は互換）。
  - `NYASH_STRING_SIZE_FORCE_HOST=1` で String.size/len を HostHandleRouter に強制。
- 目的: VM 内蔵の per‑type 分岐（型名ハードコード/ダウンキャスト）を段階撤退し、ABI 境界を一本化すること。
- 互換性: 内蔵/外付け（動的）いずれも `Arc<dyn NyashBox>` を HostHandle 経由で扱うため、パッケージ方式に依存しない。

## 🎯 概要

NyashのVMバックエンドとプラグインシステム（BID-FFI v1）の統合に関する技術仕様。Everything is Box哲学に基づき、**すべてのBox型（ビルトイン、ユーザー定義、プラグイン）**をVMで統一的に扱えるようにする。

## ⚠️ **現在のVM実装の重大な問題**

1. **ユーザー定義Box未対応** - NewBoxで文字列を返すだけ
2. **birth/finiライフサイクル欠落** - コンストラクタ・デストラクタが呼ばれない
3. **メソッド呼び出しハードコード** - 新メソッド追加が困難

これらを解決し、インタープリターと同等の統一処理を実現する。

## 🏗️ アーキテクチャ

### 統一Box管理モデル

```
┌─────────────────────────────────────────────────┐
│                  Nyash VM                       │
├─────────────────────────────────────────────────┤
│  VMValue                                        │
│  ├─ Integer(i64)     ← 基本型は直接保持       │
│  ├─ String(String)                             │
│  ├─ Bool(bool)                                 │
│  └─ BoxRef(Arc<dyn NyashBox>) ← 複雑型全般    │
├─────────────────────────────────────────────────┤
│  統一Box管理層                                  │
│  ├─ BoxFactory       : 統一Box作成             │
│  ├─ ScopeTracker     : ライフサイクル管理      │
│  └─ MethodDispatcher : 統一メソッド呼び出し    │
├─────────────────────────────────────────────────┤
│  変換レイヤー                                   │
│  ├─ to_nyash_box()   : VMValue → Box          │
│  └─ from_nyash_box() : Box → VMValue          │
├─────────────────────────────────────────────────┤
│  プラグインローダー (PluginLoaderV2)           │
│  └─ BID-FFI v1プロトコルで通信                │
└─────────────────────────────────────────────────┘
```

### VM構造体の完全形

```rust
pub struct VM {
    // 既存フィールド
    registers: HashMap<RegisterId, VMValue>,
    memory: HashMap<MemoryLocation, VMValue>,
    
    // 統一Box管理（新規）
    box_factory: Arc<BoxFactory>,           // 統一Box作成
    plugin_loader: Option<Arc<PluginLoaderV2>>, // プラグイン
    scope_tracker: ScopeTracker,            // finiライフサイクル
    box_declarations: Arc<RwLock<HashMap<String, BoxDeclaration>>>, // ユーザー定義Box
}
```

## 📊 VMValue拡張仕様

### 型定義

```rust
pub enum VMValue {
    // 基本型（既存）
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Future(FutureBox),
    Void,
    
    // 拡張型（新規）
    BoxRef(Arc<dyn NyashBox>),
}
```

### 変換規則

#### NyashBox → VMValue

1. **基本型の最適化**
   - IntegerBox → VMValue::Integer（値を直接保持）
   - StringBox → VMValue::String（値を直接保持）
   - BoolBox → VMValue::Bool（値を直接保持）

2. **複雑型の参照保持**
   - PluginBoxV2 → VMValue::BoxRef
   - ユーザー定義Box → VMValue::BoxRef
   - その他のBox → VMValue::BoxRef

#### VMValue → NyashBox

1. **基本型の再Box化**
   - VMValue::Integer → IntegerBox::new()
   - VMValue::String → StringBox::new()
   - VMValue::Bool → BoolBox::new()

2. **参照型のクローン**
   - VMValue::BoxRef → Arc::clone_box()

## 🔄 MIR命令の処理

### NewBox命令の統一実装

```rust
MirInstruction::NewBox { dst, box_type, args } => {
    // 🌟 統一Box作成プロセス
    
    // Step 1: 引数を評価してNyashBoxに変換
    let nyash_args: Vec<Box<dyn NyashBox>> = args.iter()
        .map(|id| self.get_value(*id)?.to_nyash_box())
        .collect::<Result<Vec<_>, _>>()?;
    
    // Step 2: BoxFactory経由で統一作成
    let new_box = self.box_factory.create_box(box_type, &nyash_args)?;
    
    // Step 3: birth実行（ユーザー定義Boxの場合）
    if let Some(instance) = new_box.as_any().downcast_ref::<InstanceBox>() {
        // birthコンストラクタを検索
        let birth_key = format!("birth/{}", args.len());
        
        if let Some(box_decl) = self.box_declarations.read().unwrap().get(&instance.class_name) {
            if let Some(constructor) = box_decl.constructors.get(&birth_key) {
                // birthメソッドを実行
                self.push_scope(); // 新しいスコープ
                self.set_variable("me", new_box.clone()); // me をバインド
                
                // コンストラクタ本体を実行
                let result = self.execute_constructor(constructor, nyash_args)?;
                
                self.pop_scope(); // スコープ終了
            }
        }
    }
    
    // Step 4: プラグインBoxのbirth実行
    #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
    if new_box.as_any().downcast_ref::<PluginBoxV2>().is_some() {
        // プラグインのbirthは既にcreate_box内で実行済み
    }
    
    // Step 5: スコープ追跡に登録（fini用）
    self.scope_tracker.register_box(new_box.clone());
    
    // Step 6: VMValueに変換して格納
    let vm_value = VMValue::from_nyash_box(new_box);
    self.set_value(*dst, vm_value);
}
```

### BoxCall命令の統一処理

```rust
MirInstruction::BoxCall { dst, box_val, method, args, effects } => {
    let box_vm_value = self.get_value(*box_val)?;
    
    // 統一的なメソッド呼び出し
    let result = match &box_vm_value {
        // 基本型の最適化パス
        VMValue::String(s) => {
            self.call_string_method_optimized(s, method, args)?
        },
        VMValue::Integer(i) => {
            self.call_integer_method_optimized(i, method, args)?
        },
        
        // BoxRef経由の汎用パス
        VMValue::BoxRef(arc_box) => {
            let nyash_args = convert_args_to_nyash(args);
            self.call_box_method_generic(arc_box.as_ref(), method, nyash_args)?
        },
        
        _ => return Err(VMError::TypeError("Not a box type"))
    };
    
    if let Some(dst_id) = dst {
        self.set_value(*dst_id, result);
    }
}
```

### ExternCall命令の実装

```rust
MirInstruction::ExternCall { dst, iface_name, method_name, args, effects } => {
    match (iface_name.as_str(), method_name.as_str()) {
        // プラグインBox作成
        ("plugin", "new") => {
            let box_type = self.get_value(args[0])?.to_string();
            let ctor_args = self.convert_args_to_nyash(&args[1..])?;
            
            if let Some(loader) = &self.plugin_loader {
                let plugin_box = loader.create_box(&box_type, ctor_args)?;
                let vm_value = VMValue::from_nyash_box(plugin_box);
                
                if let Some(dst_id) = dst {
                    self.set_value(*dst_id, vm_value);
                }
            }
        },
        
        // 既存のconsole.log等
        ("env.console", "log") => {
            // 既存の処理
        },
        
        _ => {
            println!("ExternCall stub: {}.{}", iface_name, method_name);
        }
    }
}
```

## 🔧 メモリ管理

### 参照カウント管理

1. **BoxRefの作成時**
   - Arc::fromでBox<dyn NyashBox>をArc<dyn NyashBox>に変換
   - 参照カウント = 1

2. **BoxRefのクローン時**
   - Arc::cloneで参照カウント増加
   - 軽量なポインタコピー

3. **BoxRefの破棄時**
   - 参照カウント減少
   - 0になったら自動解放

### スコープとライフタイム

```rust
// VMのスコープ管理
impl VM {
    fn exit_scope(&mut self) {
        // BoxRefを含むレジスタがクリアされると
        // 参照カウントが自動的に減少
        self.registers.clear();
    }
}
```

## 📈 パフォーマンス最適化

### 基本型の直接処理

```rust
// 最適化されたStringメソッド呼び出し
fn call_string_method_optimized(&self, s: &str, method: &str, args: &[ValueId]) 
    -> Result<VMValue, VMError> {
    match method {
        "length" => Ok(VMValue::Integer(s.len() as i64)),
        "substring" => {
            // 引数を直接整数として取得（Box化を回避）
            let start = self.get_value(args[0])?.to_i64()?;
            let end = self.get_value(args[1])?.to_i64()?;
            Ok(VMValue::String(s[start..end].to_string()))
        },
        _ => {
            // 未知のメソッドは汎用パスへ
            let string_box = Box::new(StringBox::new(s));
            self.call_box_method_generic(&*string_box, method, args)
        }
    }
}
```

### プラグイン呼び出しの最適化

1. **メソッドIDキャッシュ**
   - 頻繁に呼ばれるメソッドのIDをキャッシュ
   - 文字列比較を回避

2. **TLV変換の遅延評価**
   - 必要になるまでTLV変換を遅延
   - 基本型は直接渡す

## 🧪 テスト戦略

### 単体テスト

```rust
#[test]
fn test_vm_plugin_box_creation() {
    let plugin_loader = create_test_plugin_loader();
    let mut vm = VM::new_with_plugins(plugin_loader);
    
    // FileBoxの作成
    let result = vm.execute_extern_call(
        "plugin", "new", 
        vec!["FileBox", "test.txt"]
    );
    
    assert!(matches!(result, Ok(VMValue::BoxRef(_))));
}
```

### 統合テスト

```nyash
// VMで実行されるNyashコード
local file = new FileBox("output.txt")
file.write("VM Plugin Test")
local content = file.read()
assert(content == "VM Plugin Test")
```

### パフォーマンステスト

```rust
#[bench]
fn bench_plugin_method_call(b: &mut Bencher) {
    let vm = setup_vm_with_plugins();
    let file_box = create_file_box(&vm);
    
    b.iter(|| {
        vm.call_box_method(&file_box, "write", &["test"])
    });
}
```

## 🚨 エラーハンドリング

### プラグイン関連エラー

```rust
pub enum VMError {
    // 既存のエラー
    TypeError(String),
    RuntimeError(String),
    
    // プラグイン関連（新規）
    PluginNotFound(String),
    PluginMethodError { 
        plugin: String, 
        method: String, 
        error: String 
    },
    PluginInitError(String),
}
```

### エラー伝播

```rust
// プラグインエラーをVMエラーに変換
impl From<PluginError> for VMError {
    fn from(err: PluginError) -> Self {
        match err {
            PluginError::MethodNotFound(m) => {
                VMError::PluginMethodError { 
                    plugin: "unknown".to_string(),
                    method: m,
                    error: "Method not found".to_string()
                }
            },
            // ... 他のエラー変換
        }
    }
}
```

## 📊 メトリクスとモニタリング

### パフォーマンスメトリクス

- プラグイン呼び出し回数
- 平均呼び出し時間
- TLV変換オーバーヘッド
- メモリ使用量

### デバッグ情報

```rust
// デバッグモードでの詳細ログ
if cfg!(debug_assertions) {
    eprintln!("VM: Calling plugin method {}.{}", box_type, method);
    eprintln!("VM: Args: {:?}", args);
    eprintln!("VM: Result: {:?}", result);
}
```

## 🔄 ライフサイクル管理

### スコープ管理とfini呼び出し

```rust
pub struct ScopeTracker {
    scopes: Vec<Scope>,
}

pub struct Scope {
    boxes: Vec<(u64, Arc<dyn NyashBox>)>,  // (id, box)
    variables: HashMap<String, VMValue>,     // ローカル変数
}

impl VM {
    /// スコープ開始
    fn push_scope(&mut self) {
        self.scope_tracker.scopes.push(Scope::new());
    }
    
    /// スコープ終了時の自動fini呼び出し
    fn pop_scope(&mut self) -> Result<(), VMError> {
        if let Some(scope) = self.scope_tracker.scopes.pop() {
            // 逆順でfiniを呼ぶ（作成順と逆）
            for (_, box_ref) in scope.boxes.iter().rev() {
                self.call_fini_if_needed(box_ref)?;
            }
        }
        Ok(())
    }
    
    /// 統一fini呼び出し
    fn call_fini_if_needed(&mut self, box_ref: &Arc<dyn NyashBox>) -> Result<(), VMError> {
        match box_ref.type_name() {
            // ユーザー定義Box
            name if self.box_declarations.read().unwrap().contains_key(name) => {
                if let Some(instance) = box_ref.as_any().downcast_ref::<InstanceBox>() {
                    // finiメソッドが定義されているか確認
                    if let Some(box_decl) = self.box_declarations.read().unwrap().get(name) {
                        if let Some(fini_method) = box_decl.methods.get("fini") {
                            // finiを実行
                            self.set_variable("me", box_ref.clone_box());
                            self.execute_method(fini_method.clone())?;
                        }
                    }
                }
            },
            
            // プラグインBox
            #[cfg(all(feature = "plugins", not(target_arch = "wasm32")))]
            _ if box_ref.as_any().downcast_ref::<PluginBoxV2>().is_some() => {
                if let Some(plugin) = box_ref.as_any().downcast_ref::<PluginBoxV2>() {
                    plugin.call_fini();
                }
            },
            
            // ビルトインBox（将来finiサポート予定）
            _ => {
                // 現在ビルトインBoxはfiniなし
                // 将来的にはStringBox等もfini対応
            }
        }
        Ok(())
    }
}
```

### ライフサイクルの完全性

```nyash
// 🌟 すべてのBoxが同じライフサイクル

{  // スコープ開始
    local str = new StringBox("hello")      // birth（引数1つ）
    local user = new UserBox("Alice", 25)   // birth（引数2つ）
    local file = new FileBox("test.txt")    // birth（引数1つ）
    
    // 使用
    str.length()
    user.greet()
    file.write("data")
    
}  // スコープ終了 → 自動的にfini呼び出し
   // file.fini() → user.fini() → str.fini() の順
```

## 🎯 統一の利点

### 1. **シンプルな実装**
- すべてのBox型が同じコードパスを通る
- 特殊ケースの削減
- バグの温床排除

### 2. **拡張性**
- 新しいBox型追加が容易
- プラグインも同じ扱い
- 将来の機能追加も簡単

### 3. **パフォーマンス**
- 基本型は最適化パス維持
- 必要時のみBoxRef使用
- メソッドディスパッチの効率化

---

**最終更新**: 2025-08-21  
**関連文書**: 
- [BID-FFI v1 実装仕様書](./bid-ffi-v1-actual-specification.md)
- [Phase 9.78a VM Plugin Integration](../../予定/native-plan/issues/phase_9_78a_vm_plugin_integration.md)
- [Phase 9.78a 深層分析](../../予定/native-plan/issues/phase_9_78a_vm_plugin_integration_deep_analysis.md)
 - [nyash.toml v2.1: BoxRef仕様](../plugin-system/nyash-toml-v2_1-spec.md)

### 付録: 引数エンコード（v2.1 追加）
- TLVタグ: 1=Bool, 2=I32, 3=I64, 4=F32, 5=F64, 6=String, 7=Bytes, 8=Handle(BoxRef)
- BoxRef payload(tag=8): `type_id:u32` + `instance_id:u32`（LE, 8バイト）
- `nyash.toml` の `args` で `{ kind="box", category="plugin" }` を指定したとき、Loaderは `tag=8` を使用

### 返り値（v2.2）
- プラグインが `tag=8` を返した場合、Loaderは `type_id` からBox型名を逆引きし `PluginBoxV2` を構築
- 同一ライブラリでなくてもOK（構成ファイル全体から探索）

## Hako ABI Notes — StringBox（plugin‑on 経路）

- 目的: plugin‑on 環境で、受けがホスト String の場合でも TypeBox v2 の StringBox へ正しく橋渡しする。
- 受けが String のときの呼び出し順序（VM → プラグイン）
  - size/length/indexOf/lastIndexOf/substring/charAt などの BoxCall に対して、VM は一時的にプラグイン側の StringBox を作成して呼び出す。
  - 初期化は `birth(s: String)` を優先し、未実装の場合は `fromUtf8(s: String|Bytes)` を利用する。
  - `size()` 呼び出しは内部で `length()` に正規化される（コレクション API を size 統一で見せつつ、ABI は length を保持）。

### StringBox（TypeBox v2）最小 API

- length(0) -> i64（size の別名）
- isEmpty(0) -> bool
- substring(2) -> String
- indexOf(1..2) -> i64
- lastIndexOf(1..2) -> i64
- charAt(1) -> String
- fromUtf8(1) -> Handle(StringBox)（新規作成）

### 引数エンコード（TLV）と Plugin Handle

- VM→プラグインの引数エンコードは以下の順序で行う。
  1) PluginBoxV2 は `tag=8 (type_id:u32, instance_id:u32)` として渡す（ハンドル）。
  2) 数値は i64、文字列は UTF‑8（tag=6）、バイト列は tag=7。
  3) 上記に当てはまらない Box は `toString()` を UTF‑8 として渡す（暫定）。
- これにより、`Map.set("k", array)` → `Map.get("k")` で、ArrayBox の「実体同一（identity）」が保持される。

### 返り値の型復元

- プラグインが `tag=8` を返した場合、Loader は `type_id` → `box_type` をメタデータから逆引きし、`PluginBoxV2 { box_type, instance_id }` を生成する。
  - これにより `MapBox.get()` の戻りが ArrayBox であっても、後続の `a2.size()` は正しく ArrayBox へルーティングされる。
