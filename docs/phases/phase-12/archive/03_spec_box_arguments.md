# 埋め込みVMでの箱引数サポート

## 🎯 結論：完全にサポート可能

現在のMIR/VM/JIT/プラグインすべてで**箱は箱を引数にできる**仕組みが確立されており、埋め込みVMでも同じパターンで実装可能です。

## 📊 各層での箱引数の扱い

### 1. Nyashスクリプトレベル
```nyash
// 箱を引数に取るメソッド
box Processor {
    process(inputBox, configBox) {
        // inputBoxもconfigBoxも箱インスタンス
        local data = inputBox.getData()
        local settings = configBox.getSettings()
        return me.transform(data, settings)
    }
}
```

### 2. MIRレベル
```
; box1.process(box2, box3)
%4 = BoxCall %1.process(%2, %3)
```

### 3. 埋め込みVMでの実装

#### バイトコード表現
```c
// BoxCall命令: [op][dst][recv][method_id:2][argc][args...]
// 例: box1.process(box2, box3)
0x20  // OP_BOXCALL
0x04  // dst: %4
0x01  // recv: %1 (box1)
0x00 0x10  // method_id: 16 (process)
0x02  // argc: 2
0x02  // arg[0]: %2 (box2)
0x03  // arg[1]: %3 (box3)
```

#### 実行時処理
```c
// 埋め込みVMでの箱引数処理
int nyvm_execute_boxcall(NyashEmbeddedVM* vm, ...) {
    // レシーバー取得
    NyVMValue* recv = &vm->values[recv_idx];
    
    // 引数をTLVエンコード
    for (int i = 0; i < argc; i++) {
        NyVMValue* arg = &vm->values[args[i]];
        
        if (arg->type == NYVM_TYPE_HANDLE) {
            // 箱引数はハンドルとしてエンコード
            uint32_t type_id = get_type_id_from_handle(arg->value.h);
            uint32_t inst_id = get_instance_id_from_handle(arg->value.h);
            encode_handle(&tlv, type_id, inst_id);
        } else {
            // プリミティブ値
            encode_primitive(&tlv, arg);
        }
    }
    
    // C ABIプラグイン呼び出し
    return nyash_plugin_invoke(...);
}
```

## 🔄 ハンドル管理の詳細

### 1. ハンドルレジストリ
```c
// グローバルハンドルテーブル
typedef struct {
    uint32_t type_id;      // Box型ID
    uint32_t instance_id;  // インスタンスID
    uint8_t flags;         // GC/所有権フラグ
} HandleEntry;

static HandleEntry g_handles[MAX_HANDLES];

// 新しい箱インスタンスの登録
uint64_t register_box_handle(uint32_t type_id, uint32_t instance_id) {
    uint64_t handle = allocate_handle();
    g_handles[handle] = (HandleEntry){
        .type_id = type_id,
        .instance_id = instance_id,
        .flags = HANDLE_OWNED
    };
    return handle;
}
```

### 2. プラグイン間の箱共有
```c
// プラグインAが箱を返す
int plugin_a_create_box(uint8_t* result, size_t* result_len) {
    // 新しい箱を作成
    uint32_t type_id = 100;  // CustomBox
    uint32_t inst_id = create_instance();
    
    // TLVエンコード
    encode_handle(result, type_id, inst_id);
    return 0;
}

// プラグインBが箱を受け取る
int plugin_b_process(const uint8_t* args, size_t args_len) {
    // TLVデコード
    uint32_t type_id, inst_id;
    decode_handle(args, &type_id, &inst_id);
    
    // 箱を使用
    process_box(type_id, inst_id);
    return 0;
}
```

## 💡 重要なポイント

### 1. 型安全性
- ハンドルには型情報（type_id）が含まれる
- 実行時の型チェックが可能

### 2. 所有権管理
- ハンドルは参照カウント or GC管理
- プラグイン間で安全に共有

### 3. 相互運用性
- ネイティブBox ↔ スクリプトBox間で透過的
- 同じハンドル機構を使用

## 🎯 結論

埋め込みVMでも：
1. **箱は箱を引数に取れる**（ハンドル経由）
2. **型情報を保持**（type_id）
3. **プラグイン間で共有可能**（instance_id）
4. **C ABIと完全互換**（TLVエンコード）

これにより、Nyashスクリプトで書いた高度なBoxコンポジションも、C ABIプラグインとして動作します！