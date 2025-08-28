# Technical Details

## 1. ライフサイクル設計の詳細

### 1.1 所有権の森（Ownership Forest）

```
   A (strong owner)
   ├── B (strong)
   │   └── D (strong)
   └── C (strong)
       └── E (weak) → B
```

**不変条件**:
- 各ノードの強参照in-degreeは最大1
- weakエッジは所有権に関与しない
- サイクルはweakエッジでのみ許可

### 1.2 決定的破棄順序

```rust
// ScopeTrackerの実装
impl ScopeTracker {
    fn exit_scope(&mut self) {
        // LIFO順で破棄
        while let Some(box_id) = self.scope_stack.pop() {
            if let Some(pbox) = self.get_box(box_id) {
                // finiを呼ぶ
                pbox.call_fini();
            }
        }
    }
}
```

## 2. プラグインシステムの実装

### 2.1 TLVエンコーディング

```rust
// Type定義
const TLV_BOOL: u16 = 1;    // 1 byte: 0 or 1  
const TLV_I64: u16 = 3;     // 8 bytes: little endian
const TLV_STRING: u16 = 4;  // n bytes: UTF-8
const TLV_HANDLE: u16 = 8;  // 8 bytes: type_id(4) + instance_id(4)
const TLV_VOID: u16 = 9;    // 0 bytes

// エンコード例
fn encode_string(s: &str) -> Vec<u8> {
    let mut buf = vec![];
    buf.extend(&TLV_STRING.to_le_bytes());
    buf.extend(&(s.len() as u16).to_le_bytes());
    buf.extend(s.as_bytes());
    buf
}
```

### 2.2 プラグイン実装例（StringBox）

```rust
// StringBoxプラグイン
struct StringInstance {
    data: String,
}

static INSTANCES: Lazy<Mutex<HashMap<u32, StringInstance>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

#[no_mangle]
extern "C" fn nyash_plugin_invoke(
    type_id: u32,
    method_id: u32,
    instance_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => {
            // 新しいインスタンス作成
            let inst = StringInstance {
                data: String::new(),
            };
            let id = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
            INSTANCES.lock().unwrap().insert(id, inst);
            write_tlv_handle(type_id, id, result, result_len)
        }
        METHOD_LENGTH => {
            // 文字列長を返す
            let instances = INSTANCES.lock().unwrap();
            if let Some(inst) = instances.get(&instance_id) {
                let len = inst.data.len() as i64;
                write_tlv_i64(len, result, result_len)
            } else {
                NYB_E_INVALID_HANDLE
            }
        }
        METHOD_FINI => {
            // インスタンス破棄（Dropが自動実行）
            INSTANCES.lock().unwrap().remove(&instance_id);
            NYB_OK
        }
        _ => NYB_E_INVALID_METHOD,
    }
}
```

## 3. JIT統合

### 3.1 プラグイン呼び出しのJITコンパイル

```rust
// LowerCoreでのプラグイン呼び出し生成
fn lower_box_call(&mut self, method: &str, args: &[ValueId]) {
    if let Ok(h) = plugin_host.resolve_method("StringBox", method) {
        // スタックに引数を積む
        self.builder.emit_param_i64(receiver_idx);
        for arg in args {
            self.push_value(arg);
        }
        
        // プラグイン呼び出し命令を生成
        self.builder.emit_plugin_invoke(
            h.type_id,
            h.method_id, 
            args.len() + 1,
            has_return
        );
    }
}
```

### 3.2 Craneliftでのコード生成

```rust
// nyash_plugin_invoke3_i64 シムの実装
extern "C" fn nyash_plugin_invoke3_i64(
    type_id: i64,
    method_id: i64,
    argc: i64,
    a0: i64,  // receiver (param index)
    a1: i64,  // arg1
    a2: i64,  // arg2
) -> i64 {
    // レシーバーをVMの引数から解決
    let instance_id = resolve_instance_from_vm_args(a0);
    
    // TLVエンコードで引数準備
    let mut tlv_args = encode_tlv_header(argc - 1);
    if argc >= 2 { encode_i64(&mut tlv_args, a1); }
    if argc >= 3 { encode_i64(&mut tlv_args, a2); }
    
    // プラグイン呼び出し
    let mut result = [0u8; 32];
    let mut result_len = result.len();
    let rc = invoke_plugin(
        type_id as u32,
        method_id as u32,
        instance_id,
        &tlv_args,
        &mut result,
        &mut result_len
    );
    
    // 結果をデコードして返す
    decode_tlv_i64(&result[..result_len]).unwrap_or(0)
}
```

## 4. GCオン/オフ等価性

### 4.1 アノテーション処理

```rust
enum BoxAnnotation {
    MustDrop,   // @must_drop - 即時破棄必須
    GCable,     // @gcable - GC可能（遅延OK）
}

impl GarbageCollector {
    fn should_collect(&self, box_type: &BoxType) -> bool {
        match box_type.annotation {
            BoxAnnotation::MustDrop => false,  // GC対象外
            BoxAnnotation::GCable => self.gc_enabled,
        }
    }
}
```

### 4.2 観測不可能性の保証

```rust
// テストケース：GCオン/オフで同じ結果
#[test]
fn test_gc_equivalence() {
    let output_gc_on = run_with_gc(true, "test.nyash");
    let output_gc_off = run_with_gc(false, "test.nyash");
    
    assert_eq!(output_gc_on.io_trace, output_gc_off.io_trace);
    assert_eq!(output_gc_on.result, output_gc_off.result);
}
```

## 5. 静的リンクによる最適化

### 5.1 AOT変換

```bash
# NyashからCLIFへ
nyashc --emit-clif program.nyash > program.clif

# CLIFからオブジェクトファイルへ  
cranelift-objdump program.clif -o program.o

# 静的リンク（PLT回避）
cc program.o -static \
   -L. -lnyrt \
   -lnyplug_array \
   -lnyplug_string \
   -o program
```

### 5.2 パフォーマンス比較

```
動的リンク（PLT経由）: ~15ns/call
静的リンク（直接呼出）: ~2ns/call
インライン展開後: ~0.5ns/call
```