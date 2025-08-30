# 埋め込みVMでのBox処理設計

## 🎯 核心：MIRレベルでのBox処理を再現

### 現在のMIR/VMでのBox処理フロー

```
1. MIR: BoxCall/PluginInvoke命令
   ↓
2. VM: ValueId → VMValue変換
   ↓
3. VMValue → Box<dyn NyashBox> or TLVエンコード
   ↓
4. メソッド実行
   ↓
5. 結果をVMValueに戻す
```

## 📊 埋め込みVMの設計

### 1. 軽量VMValue定義

```c
// 埋め込みVM用の値表現
typedef enum {
    NYVM_TYPE_INT,
    NYVM_TYPE_FLOAT,
    NYVM_TYPE_BOOL,
    NYVM_TYPE_STRING,
    NYVM_TYPE_HANDLE,  // Box参照（ハンドル）
    NYVM_TYPE_VOID
} NyVMType;

typedef struct {
    NyVMType type;
    union {
        int64_t i;
        double f;
        uint8_t b;
        struct { const char* data; size_t len; } s;
        uint64_t h;  // ハンドル（Box参照）
    } value;
} NyVMValue;
```

### 2. MIRバイトコード形式

```c
// BoxCall命令のエンコード
enum {
    OP_BOXCALL = 0x20,
    OP_PLUGIN_INVOKE = 0x21,
    // ...
};

// BoxCall: [op:1] [dst:1] [box_val:1] [method_id:2] [argc:1] [args...]
// 例: BoxCall %2 = %1.toString()
// → 0x20 0x02 0x01 0x00 0x00 0x00
```

### 3. 埋め込みVMでのBoxCall実行

```c
int nyvm_execute_boxcall(
    NyashEmbeddedVM* vm,
    uint8_t dst,
    uint8_t box_val,
    uint16_t method_id,
    uint8_t argc,
    uint8_t* args
) {
    // 1. レシーバー取得
    NyVMValue* recv = &vm->values[box_val];
    
    // 2. プリミティブ型の場合
    if (recv->type != NYVM_TYPE_HANDLE) {
        // プリミティブ→TLV変換
        uint8_t tlv_buf[256];
        size_t tlv_len = encode_primitive_to_tlv(recv, tlv_buf);
        
        // 組み込み実装を呼び出し
        return call_builtin_method(recv->type, method_id, tlv_buf, tlv_len);
    }
    
    // 3. Box（ハンドル）の場合
    uint64_t handle = recv->value.h;
    
    // 引数をTLVエンコード
    uint8_t args_tlv[1024];
    size_t args_len = 0;
    for (int i = 0; i < argc; i++) {
        NyVMValue* arg = &vm->values[args[i]];
        args_len += encode_value_to_tlv(arg, &args_tlv[args_len]);
    }
    
    // 4. プラグイン呼び出し（C ABI）
    uint8_t result[4096];
    size_t result_len = sizeof(result);
    
    int rc = nyash_plugin_invoke(
        get_type_id_from_handle(handle),
        method_id,
        get_instance_id_from_handle(handle),
        args_tlv, args_len,
        result, &result_len
    );
    
    // 5. 結果をVMValueに変換
    if (rc == 0) {
        decode_tlv_to_value(result, result_len, &vm->values[dst]);
    }
    
    return rc;
}
```

## 🔄 Box引数の処理

### 現在のVM（Rust）
```rust
// VMValue → Box<dyn NyashBox>変換
let val = self.get_value(*arg)?;
Ok(val.to_nyash_box())
```

### 埋め込みVM（C）
```c
// NyVMValue → TLVエンコード
switch (value->type) {
    case NYVM_TYPE_INT:
        encode_i64(tlv, value->value.i);
        break;
    case NYVM_TYPE_HANDLE:
        encode_handle(tlv, 
            get_type_id_from_handle(value->value.h),
            get_instance_id_from_handle(value->value.h)
        );
        break;
    // ...
}
```

## 💡 実装のポイント

### 1. ハンドル管理
```c
// グローバルハンドルテーブル
typedef struct {
    uint32_t type_id;
    uint32_t instance_id;
    void* native_ptr;  // 実際のBoxポインタ（必要な場合）
} HandleEntry;

static HandleEntry g_handles[MAX_HANDLES];
static uint64_t g_next_handle = 1;

uint64_t register_handle(uint32_t type_id, uint32_t instance_id) {
    uint64_t h = g_next_handle++;
    g_handles[h].type_id = type_id;
    g_handles[h].instance_id = instance_id;
    return h;
}
```

### 2. 組み込みメソッド
```c
// 頻出メソッドは埋め込みVMに直接実装
int call_builtin_method(NyVMType type, uint16_t method_id, ...) {
    switch (type) {
        case NYVM_TYPE_INT:
            if (method_id == 0) { // toString
                // 整数→文字列変換
            }
            break;
        // ...
    }
}
```

### 3. プラグインとの統合
```c
// 生成されるCコード
extern "C" int32_t nyplug_mybox_invoke(...) {
    // MIRバイトコード実行
    NyashEmbeddedVM vm;
    nyvm_init(&vm, BYTECODE, sizeof(BYTECODE));
    
    // 引数をVMスタックに設定
    nyvm_decode_args(&vm, args, args_len);
    
    // メソッド実行
    nyvm_execute_method(&vm, method_id);
    
    // 結果をTLVエンコード
    return nyvm_encode_result(&vm, result, result_len);
}
```

## 🎯 結論

埋め込みVMは：
1. **MIRのBoxCall/PluginInvoke命令を忠実に実装**
2. **TLVエンコード/デコードでC ABIと通信**
3. **ハンドルでBox参照を管理**
4. **頻出処理は最適化実装**

これにより、Nyashスクリプトで書いたプラグインも、ネイティブプラグインと同じC ABIで動作します！