# Nyash ABI プラグインシステム

## 📦 概要

Nyash ABIは、**C ABI + TypeBoxをベースに構築された高度なプラグインシステム**です。C ABIの限界を超えて、より豊富な機能と言語間相互運用を提供します。

## 🏗️ アーキテクチャ

```
基本層：C ABI（シンプル・高速・安定）
   ↓
拡張層：TypeBox（プラグイン間連携）
   ↓
高度層：Nyash ABI（言語間相互運用・拡張性）
```

Nyash ABIは、C ABIとTypeBoxの上に構築されているため、既存のC ABIプラグインとの互換性を保ちながら、より高度な機能を提供できます。

## 🎯 特徴

### C ABIからの進化点

1. **セレクターベースの高速ディスパッチ**
   - 文字列比較ではなく、事前計算されたハッシュ値を使用
   - メソッド呼び出しの高速化

2. **NyashValue型による統一的な値表現**
   - 16バイトに最適化された値表現
   - インライン値サポート（小さな整数やboolは直接格納）

3. **言語間相互運用**
   - Python、Go、JavaScript等との連携が可能
   - 共通のオブジェクトモデルを提供

4. **高度なメモリ管理**
   - アトミック参照カウント
   - 弱参照による循環参照回避

## 📝 実装例

### 基本的なNyash ABIプラグイン

```c
#include "nyash_abi.h"

// Nyash ABIオブジェクト構造体
typedef struct {
    nyash_obj_header header;  // 共通ヘッダー（参照カウント等）
    int value;                // カスタムデータ
} CounterBox;

// メソッドディスパッチャー（セレクターベース）
nyash_status counter_call(
    nyash_ctx* ctx,
    void* self,
    nyash_selector selector,
    nyash_value* args,
    size_t arg_count,
    nyash_value* result
) {
    CounterBox* counter = (CounterBox*)self;
    
    // セレクターに基づいて高速ディスパッチ
    switch(selector) {
        case NYASH_SEL_INCREMENT:  // 事前計算されたハッシュ値
            counter->value++;
            *result = nyash_make_int(counter->value);
            return NYASH_OK;
            
        case NYASH_SEL_GET_VALUE:
            *result = nyash_make_int(counter->value);
            return NYASH_OK;
            
        default:
            return NYASH_ERROR_METHOD_NOT_FOUND;
    }
}
```

### NyashValue - 統一的な値表現

```c
// 16バイトに最適化された値構造（JIT/LLVM最適化を考慮）
typedef struct __attribute__((aligned(16))) {
    uint64_t type_id;     // 型識別子
    uint64_t payload;     // ポインタまたはインライン値
    uint64_t metadata;    // フラグ・追加情報
} nyash_value;

// インライン値の例
nyash_value nyash_make_int(int64_t value) {
    return (nyash_value){
        .type_id = NYASH_TYPE_INTEGER,
        .payload = (uint64_t)value,
        .metadata = NYASH_TAG_SMALL_INT  // インライン整数タグ
    };
}

// Boxオブジェクトの例
nyash_value nyash_make_box(void* box_ptr) {
    return (nyash_value){
        .type_id = ((nyash_obj_header*)box_ptr)->type_id,
        .payload = (uint64_t)box_ptr,
        .metadata = NYASH_TAG_POINTER  // ヒープポインタタグ
    };
}
```

## 🌐 言語間相互運用

### Python連携

```python
# Python側のNyash ABIラッパー
import nyash

# Nyashプラグインをロード
counter = nyash.load_plugin("counter.so")

# セレクターベースの呼び出し
result = counter.call("increment")  # 内部でセレクターに変換
print(f"Counter value: {result}")
```

### Go連携

```go
// Go側のNyash ABIバインディング
package main

import "github.com/nyash/go-bindings"

func main() {
    counter := nyash.LoadPlugin("counter.so")
    
    // 型安全な呼び出し
    value, err := counter.Call("increment")
    if err == nil {
        fmt.Printf("Counter value: %d\n", value.(int64))
    }
}
```

## 🚀 Nyash ABIがTypeBoxとして実装される仕組み

Nyash ABIの革新的な点は、**ABIそのものがTypeBoxとして実装される**ことです：

```c
// Nyash ABIプロバイダーもTypeBox（C ABI）として提供
typedef struct {
    // TypeBox標準ヘッダ
    uint32_t abi_tag;           // 'NABI'
    const char* name;           // "NyashABIProvider"
    void* (*create)(void);      // ABIプロバイダ生成
    
    // Nyash ABI専用操作
    struct {
        nyash_status (*call)(nyash_ctx*, void* obj, nyash_selector, ...);
        void (*retain)(void* obj);
        void (*release)(void* obj);
    } nyash_ops;
} NyashABITypeBox;
```

これにより：
1. **段階的移行**: C ABIプラグインからNyash ABIへの移行が容易
2. **相互運用**: C ABIとNyash ABIプラグインが同じシステムで共存
3. **セルフホスティング**: 最終的にNyash自身でNyashを実装可能

## 💡 いつNyash ABIを使うべきか？

### Nyash ABIが最適な場合
- ✅ **他言語との相互運用**が必要（Python/Go/JS等）
- ✅ **高度なメソッドディスパッチ**が必要（セレクター方式）
- ✅ **複雑な型システム**を扱う
- ✅ **将来の拡張性**を重視

### C ABIで十分な場合
- ✅ シンプルな機能のみ必要
- ✅ 最高速度を求める（直接関数呼び出し）
- ✅ 既存のC/C++ライブラリの単純なラップ

## 📊 3つのABIの比較

| 特徴 | C ABI | C ABI + TypeBox | Nyash ABI |
|------|-------|-----------------|-----------|
| シンプルさ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 速度 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| プラグイン間連携 | ❌ | ✅ | ✅ |
| 言語間連携 | ⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| 拡張性 | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

## 📚 まとめ

Nyash ABIは「**C ABI + TypeBoxの上に構築された高度なプラグインシステム**」です。

- C ABIの安定性とシンプルさを継承
- TypeBoxによるプラグイン間連携をサポート
- セレクター方式による高速メソッドディスパッチ
- 言語間相互運用による無限の可能性

**高度な機能や将来の拡張性が必要ならNyash ABI**を選びましょう！