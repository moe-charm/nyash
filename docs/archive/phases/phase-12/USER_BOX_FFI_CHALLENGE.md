# ユーザーBoxをC ABIで渡す技術的課題

## 🚨 現在の実装の問題点

### 1. ユーザーBoxの内部構造

```rust
pub struct InstanceBox {
    // フィールドはHashMapで管理
    pub fields_ng: Arc<Mutex<HashMap<String, NyashValue>>>,
    
    // メソッドはASTノードのまま！
    pub methods: Arc<HashMap<String, ASTNode>>,
}
```

**問題**: これらはRust固有の構造で、C ABIで直接渡せない

### 2. C ABIが期待する構造

```c
typedef struct {
    // 固定的な関数ポインタが必要
    void* (*create)(void* args);
    void (*destroy)(void* self);
    NyResult (*invoke_id)(void* self, uint32_t method_id, 
                         NyValue* args, int argc);
} NyashTypeBox;
```

## 🔧 必要な変換層

### 1. メソッドのコンパイル

```rust
// 現在: ASTNode（インタープリター実行）
methods: HashMap<String, ASTNode>

// 必要: 関数ポインタ or トランポリン
methods: HashMap<u32, fn(self, args) -> Result>
```

### 2. フィールドアクセスの標準化

```c
// C側から見えるインターフェース
typedef struct {
    void* (*get_field)(void* self, const char* name);
    void (*set_field)(void* self, const char* name, void* value);
} FieldAccessor;
```

### 3. トランポリン関数の生成

```rust
// ユーザーBoxごとに自動生成する必要がある
extern "C" fn user_box_invoke(
    self_ptr: *mut c_void,
    method_id: u32,
    args: *mut NyValue,
    argc: c_int
) -> NyResult {
    // 1. self_ptrからInstanceBoxを復元
    let instance = unsafe { &mut *(self_ptr as *mut InstanceBox) };
    
    // 2. method_idからメソッド名を解決
    let method_name = resolve_method_name(method_id);
    
    // 3. ASTNodeを取得
    let ast = instance.methods.get(&method_name)?;
    
    // 4. インタープリターで実行（遅い！）
    let result = interpreter.execute_method(instance, ast, args);
    
    // 5. 結果をC ABIに変換
    to_ny_result(result)
}
```

## 🚀 解決策の提案

### 案1: JITコンパイル（理想的だが複雑）

```rust
// ユーザーBox登録時にJITコンパイル
fn register_user_box(spec: &BoxSpec) -> TypeBox {
    let compiled_methods = jit_compile_methods(&spec.methods);
    
    TypeBox {
        invoke_id: |self, id, args| {
            compiled_methods[id](self, args)
        }
    }
}
```

### 案2: インタープリタートランポリン（現実的）

```rust
// グローバルなインタープリター参照を保持
static INTERPRETER: OnceCell<Arc<Interpreter>> = OnceCell::new();

extern "C" fn universal_user_box_invoke(
    handle: u64,  // ハンドル経由
    method_id: u32,
    args: *mut NyValue,
    argc: c_int
) -> NyResult {
    // ハンドルからBoxを取得
    let registry = HANDLE_REGISTRY.read();
    let instance = registry.get(handle)?;
    
    // インタープリター経由で実行
    INTERPRETER.get().unwrap().invoke_method(
        instance, method_id, args, argc
    )
}
```

### 案3: ハイブリッドアプローチ（段階的）

1. **Phase 1**: インタープリタートランポリン（すぐ実装可能）
2. **Phase 2**: 頻繁に呼ばれるメソッドをキャッシュ
3. **Phase 3**: AOT時にネイティブコード生成

## 📊 パフォーマンスへの影響

```
ビルトインBox呼び出し:     1-3ns
プラグインBox呼び出し:     10-15ns
ユーザーBox（トランポリン）: 100-200ns
ユーザーBox（JIT後）:      15-20ns
```

## 🎯 実装優先順位

1. **最小実装**（1週間）
   - ハンドル経由のトランポリン
   - グローバルインタープリター参照
   - 基本的なメソッド呼び出し

2. **最適化**（2週間）
   - メソッドIDキャッシュ
   - 引数変換の効率化
   - エラーハンドリング

3. **高速化**（1ヶ月）
   - 簡易JITコンパイル
   - AOT対応
   - ネイティブコード生成

## 結論

ユーザーBoxをC ABIで渡すには、**インタープリター実行をトランポリン関数でラップ**する必要があります。これは性能上のオーバーヘッドがありますが、段階的に最適化可能です。