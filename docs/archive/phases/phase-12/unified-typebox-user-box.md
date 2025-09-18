# 統一TypeBox ABI - ユーザー定義Box統合

## 🎯 概要

統一TypeBox ABIの最終形：**ユーザー定義BoxもC ABIプラグインとして扱える！**

## 🚀 革命的な統合アーキテクチャ

### 提案されたAPI

```c
// ユーザーBoxもプラグインとして動的登録
NyashTypeBox* register_user_box(const char* name, 
                                NyashBoxMethods* methods) {
    // ユーザーBoxに一時的なtype_idを割り当て
    // メソッドテーブルを登録
    // → プラグインと同じように扱える
}
```

### 利点

1. **完全な統一性**: すべてのBox（ビルトイン、プラグイン、ユーザー定義）が同じABIで扱える
2. **透過的な相互運用**: ユーザーBox ↔ プラグインBox間の自由な受け渡し
3. **動的拡張性**: 実行時に新しい型を追加可能
4. **既存コードとの互換性**: 現在のプラグインシステムと完全互換

## 📐 技術設計

### 統一TypeBox構造体（拡張版）

```c
typedef struct NyashTypeBox {
    // === ヘッダー情報 ===
    uint32_t abi_tag;        // 'TYBX'
    uint16_t version;        // 2 (ユーザーBox対応)
    uint16_t flags;          // HOST_MANAGED | THREAD_SAFE等
    
    // === 型情報 ===
    uint32_t type_id;        // 動的割り当て（高ビットで区別）
    const char* name;        // "UserDefinedBox"
    
    // === 基本操作 ===
    void* (*create)(void* args);
    void (*destroy)(void* self);
    void (*retain)(void* self);   // 参照カウント増加
    void (*release)(void* self);  // 参照カウント減少
    
    // === メソッドディスパッチ ===
    uint32_t (*resolve)(const char* name);
    NyResult (*invoke_id)(void* self, uint32_t method_id, 
                         NyValue* args, int argc);
    
    // === メタ情報（オプション）===
    size_t instance_size;    // インスタンスサイズ
    void* metadata;          // 追加メタデータ
} NyashTypeBox;
```

### Type ID管理

```c
// 高ビットでホスト/プラグイン区別
#define HOST_TYPE_FLAG    0x80000000
#define PLUGIN_TYPE_FLAG  0x40000000
#define BUILTIN_TYPE_FLAG 0x00000000

// 例
uint32_t user_box_id = HOST_TYPE_FLAG | 0x00001234;
uint32_t plugin_box_id = PLUGIN_TYPE_FLAG | 0x00005678;
uint32_t builtin_box_id = BUILTIN_TYPE_FLAG | 0x00000001;
```

## 🔧 実装詳細

### メモリ管理

```c
typedef struct {
    atomic_uint ref_count;     // アトミック参照カウント
    uint32_t finalizer_id;     // クリーンアップ用ID
    void (*destroy)(void*);    // デストラクタ
} NyashBoxHeader;

// すべてのBoxインスタンスの先頭に配置
typedef struct {
    NyashBoxHeader header;
    // ... Box固有のデータ
} UserBoxInstance;
```

### ユーザーBox登録の流れ

```nyash
// Nyashでユーザー定義Box
box Person {
    init { name, age }
    
    greet() {
        return "Hello, I'm " + me.name
    }
}

// 内部で自動的に以下が実行される：
// 1. Personボックスのメソッドテーブル生成
// 2. register_user_box("Person", methods)でC ABIに登録
// 3. type_id割り当て（例: 0x80001234）
// 4. プラグインから呼び出し可能に！
```

### プラグインからの利用

```c
// プラグイン側でユーザーBoxを受け取る
NyResult process_person(void* self, uint32_t method_id, 
                       NyValue* args, int argc) {
    // args[0]がPersonBoxのhandle
    if (args[0].type_tag == TAG_HANDLE) {
        uint32_t type_id = extract_type_id(args[0]);
        uint32_t instance_id = extract_instance_id(args[0]);
        
        // ホストに問い合わせてメソッド呼び出し
        NyValue result = ny_host_invoke(type_id, instance_id, 
                                       "greet", NULL, 0);
        
        // 結果を処理
        return ny_result_ok(result);
    }
}
```

## 🛡️ セキュリティと安全性

### 多層防御アプローチ

1. **信頼レベル別処理**
   ```c
   if (is_trusted_source(box_source)) {
       // 直接登録（高速パス）
       register_user_box_direct(name, methods);
   } else {
       // Wasmサンドボックス内で実行
       register_user_box_sandboxed(name, methods);
   }
   ```

2. **メソッドポインタ検証**
   ```c
   bool validate_method_pointer(void* ptr) {
       // 実行可能メモリ領域をチェック
       return is_executable_memory(ptr);
   }
   ```

3. **権限ベースアクセス制御**
   ```c
   #define CAP_FILE_ACCESS   (1 << 0)
   #define CAP_NET_ACCESS    (1 << 1)
   #define CAP_SPAWN_THREAD  (1 << 2)
   
   // 登録時に権限を指定
   register_user_box_with_caps(name, methods, 
                              CAP_FILE_ACCESS);
   ```

## 📊 パフォーマンス考慮

### ベンチマーク予測
```
ネイティブメソッド呼び出し     : 1ns
プラグイン内メソッド呼び出し   : 15ns  
ユーザーBox（同一プロセス）     : 20ns
ユーザーBox（Wasmサンドボックス）: 100ns
```

### 最適化戦略

1. **インラインキャッシング**: 頻繁に呼ばれるユーザーBoxメソッドをキャッシュ
2. **JIT最適化**: 型が確定したらde-virtualization
3. **バッチ処理**: 複数メソッド呼び出しをまとめて実行

## 🔄 段階的実装計画

### Phase 1: 基本実装（1ヶ月）
- `register_user_box` APIの実装
- 基本的な参照カウント管理
- シンプルなメソッドディスパッチ

### Phase 2: GC統合（2ヶ月）
- GCとの協調動作
- 循環参照の検出と解決
- WeakRef対応

### Phase 3: セキュリティ強化（3ヶ月）
- Wasmサンドボックス統合
- 権限管理システム
- 監査ログ機能

### Phase 4: 最適化（6ヶ月）
- JIT統合
- インラインキャッシング
- プロファイルベース最適化

## 🌟 Everything is Boxの完成

この実装により：
- ✅ **すべてのBoxが平等に**: ビルトイン、プラグイン、ユーザー定義の区別なし
- ✅ **完全な相互運用性**: どのBoxも自由に受け渡し可能
- ✅ **動的な拡張性**: 実行時に新しい型を追加可能
- ✅ **最高のパフォーマンス**: JIT最適化で高速実行

**「Everything is Box」哲学が、言語の境界を越えて完全に実現されます！**

## 📚 参考資料

- [AI先生たちの技術的検討](./ai-consultation-unified-typebox.md)
- [統一TypeBox ABI仕様](./unified-typebox-abi.md)
- [移行ガイド](./migration-guide.md)