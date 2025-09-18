# C ABI プラグインシステム

## 📦 概要

C ABIは、Nyashの**基本プラグインシステム**です。C言語の標準的な関数呼び出し規約を使用し、高速かつ軽量な実装を提供します。

## 🎯 特徴

### シンプルさ
- **たった3つの関数**で完全なプラグインが作れる
- 複雑な型システムや継承は不要
- C言語の知識があれば誰でも書ける

### 高速性
- 直接的な関数ポインタ呼び出し
- オーバーヘッドがほぼゼロ
- JITコンパイラとの相性も抜群

### 安定性
- C ABIは数十年の実績がある標準規格
- プラットフォーム間での互換性が高い
- バージョン間の互換性も維持しやすい

## 📝 基本実装

### 1. 最小限のプラグイン（3つの関数）

```c
#include <string.h>
#include <stdlib.h>

// 1. create関数：Boxを作成
void* string_create(const char* initial_value) {
    char* str = malloc(strlen(initial_value) + 1);
    strcpy(str, initial_value);
    return str;
}

// 2. method関数：メソッドを実行
void* string_method(void* self, const char* method_name, void** args, int arg_count) {
    char* str = (char*)self;
    
    if (strcmp(method_name, "length") == 0) {
        int* result = malloc(sizeof(int));
        *result = strlen(str);
        return result;
    }
    
    if (strcmp(method_name, "toUpperCase") == 0) {
        char* upper = malloc(strlen(str) + 1);
        for (int i = 0; str[i]; i++) {
            upper[i] = toupper(str[i]);
        }
        upper[strlen(str)] = '\0';
        return upper;
    }
    
    return NULL;  // メソッドが見つからない
}

// 3. destroy関数：メモリを解放
void string_destroy(void* self) {
    free(self);
}
```

### 2. プラグイン設定（nyash.toml）

```toml
[[plugins]]
name = "StringBox"
path = "./string_plugin.so"
type = "c-abi"
version = 1

# メソッドの戻り値型を指定
[plugins.methods]
length = { returns = "integer" }
toUpperCase = { returns = "string" }
```

## 🔧 TLV（Type-Length-Value）形式

### 構造化データのやり取り

C ABIでは、複雑なデータをTLV形式でやり取りします：

```c
// TLVヘッダー
typedef struct {
    uint8_t type;     // 1=bool, 2=i64, 3=f64, 5=string, 6=handle
    uint32_t length;  // データ長
    // この後にデータが続く
} TLVHeader;

// 複数の値を返す例
void* math_stats(void* self, const char* method_name, void** args, int arg_count) {
    if (strcmp(method_name, "calculate") == 0) {
        // 3つの値を返す: [min, max, average]
        size_t total_size = sizeof(TLVHeader) * 3 + sizeof(double) * 3;
        uint8_t* buffer = malloc(total_size);
        uint8_t* ptr = buffer;
        
        // min値
        TLVHeader* hdr1 = (TLVHeader*)ptr;
        hdr1->type = 3;  // f64
        hdr1->length = sizeof(double);
        ptr += sizeof(TLVHeader);
        *(double*)ptr = 1.0;
        ptr += sizeof(double);
        
        // 以下同様にmax値、average値を追加...
        
        return buffer;
    }
    return NULL;
}
```

## 🚀 TypeBox：C ABIの拡張メカニズム

### C ABIの限界を超える

C ABIだけでは、プラグイン間でBoxを生成することができません。例えば、MapBoxがArrayBoxを返したい場合、MapBoxはArrayBoxの実装を知らないため直接作成できません。

この問題を解決するのが**TypeBox**です。

### TypeBoxとは？

TypeBoxは「**型情報をBoxとして扱う**」という概念です。型の生成方法をBoxとして渡すことで、プラグイン間の連携を可能にします。

```c
// TypeBox構造体：型情報をBoxとして扱う
typedef struct {
    uint32_t abi_tag;       // 'TYBX' (0x54594258)
    const char* name;       // "ArrayBox"
    void* (*create)(void);  // Box生成関数
} TypeBox;

// MapBox.keys()の実装 - ArrayBoxのTypeBoxを引数で受け取る
void* map_keys(void* self, void* array_type_box) {
    MapBox* map = (MapBox*)self;
    TypeBox* array_type = (TypeBox*)array_type_box;
    
    // 検証
    if (!array_type || array_type->abi_tag != 0x54594258) {
        return NULL;
    }
    
    // ArrayBoxを生成（TypeBox経由）
    void* array = array_type->create();
    if (!array) return NULL;
    
    // MapのキーをArrayに追加
    // （ArrayBoxのメソッドは別途C API経由で呼ぶ）
    
    return array;
}
```

### TypeBoxの利点

1. **プラグイン間の疎結合**: 直接的な依存関係なし
2. **型安全性**: abi_tagによる検証
3. **拡張可能**: 新しいBox型の追加が容易
4. **シンプル**: 構造体1つ、関数ポインタ1つ

## 💡 いつC ABIを使うべきか？

### C ABIが最適な場合
- ✅ **シンプルな機能**を追加したい
- ✅ **高速性**が重要
- ✅ **安定性**を重視する
- ✅ 既存のC/C++ライブラリをラップしたい

### C ABIでは難しい場合
- ❌ 複雑な継承関係が必要
- ❌ 他言語（Python/Go等）との相互運用
- ❌ 動的な型変換が頻繁に必要

これらの場合は、C ABI + TypeBoxをベースに構築された**Nyash ABI**を検討してください。

## 📊 まとめ

C ABIは「**シンプル・高速・安定**」の3拍子が揃った、Nyashの基本プラグインシステムです。

- 最小3関数で実装可能
- オーバーヘッドがほぼゼロ
- TypeBoxによる拡張で、プラグイン間連携も可能

**迷ったらC ABIから始める**のが正解です！