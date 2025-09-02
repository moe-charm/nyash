# 統一TypeBox ABIへの移行ガイド

## 📋 概要

このガイドでは、既存のC ABIプラグインやTypeBoxプラグインを、新しい統一TypeBox ABIに移行する方法を説明します。

**良いニュース**：既存のプラグインは**そのまま動作し続けます**！段階的に移行できます。

## 🎯 移行のメリット

1. **パフォーマンス向上**：最大33倍の高速化（JIT最適化時）
2. **プラグイン間連携**：他のBoxを自由に作成・使用可能
3. **将来性**：async/await、並列実行、GPU対応への道
4. **保守性向上**：統一された1つの形式

## 🔄 移行パターン

### パターン1: 既存のC ABIプラグインから移行

#### Before（旧C ABI）
```c
// 旧形式：3つの関数をエクスポート
void* string_create(const char* initial) {
    return strdup(initial);
}

void* string_method(void* self, const char* method, 
                   void** args, int argc) {
    if (strcmp(method, "length") == 0) {
        int* result = malloc(sizeof(int));
        *result = strlen((char*)self);
        return result;
    }
    return NULL;
}

void string_destroy(void* self) {
    free(self);
}
```

#### After（統一TypeBox）
```c
#include "nyash_typebox.h"

// メソッドIDを定義（高速化のため）
#define METHOD_LENGTH 1

// resolve関数を追加
uint32_t string_resolve(const char* name) {
    if (strcmp(name, "length") == 0) return METHOD_LENGTH;
    return 0;
}

// 高速版メソッドを追加
NyResult string_invoke_id(void* self, uint32_t id, 
                         NyValue* args, int argc) {
    switch (id) {
        case METHOD_LENGTH:
            return ny_result_ok(ny_value_int(strlen((char*)self)));
        default:
            return ny_result_error("Unknown method");
    }
}

// TypeBox構造体として統合
const NyashTypeBox nyash_typebox_StringBox = {
    .abi_tag = 0x54594258,  // 'TYBX'
    .version = 1,
    .struct_size = sizeof(NyashTypeBox),
    .name = "StringBox",
    
    // 既存の関数をそのまま使用
    .create = (void*)string_create,
    .destroy = string_destroy,
    .method = string_method,  // 互換性のため残す
    
    // 新規追加
    .resolve = string_resolve,
    .invoke_id = string_invoke_id,
    
    .capabilities = NYASH_CAP_THREAD_SAFE,
    .reserved = {0}
};
```

### パターン2: 最小限の移行（互換モード）

既存のコードをほぼ変更せずに、TypeBox形式でラップ：

```c
// 既存の関数はそのまま
extern void* my_create(const char*);
extern void* my_method(void*, const char*, void**, int);
extern void my_destroy(void*);

// ラッパーを追加するだけ
const NyashTypeBox nyash_typebox_MyBox = {
    .abi_tag = 0x54594258,
    .version = 1,
    .struct_size = sizeof(NyashTypeBox),
    .name = "MyBox",
    
    .create = (void*)my_create,
    .destroy = my_destroy,
    .method = my_method,
    
    // 高速版は後で追加可能
    .resolve = NULL,
    .invoke_id = NULL,
    
    .capabilities = 0,
    .reserved = {0}
};
```

## 🛠️ 移行ツール

### 自動変換ツール

```bash
# 既存のC ABIプラグインを分析して、TypeBoxラッパーを生成
nyash-plugin migrate --input=old_plugin.c --output=new_plugin.c

# 生成されるコード例：
# - TypeBox構造体定義
# - resolve関数のスケルトン
# - invoke_id関数のスケルトン
```

### 検証ツール

```bash
# 移行後のプラグインが正しく動作するかチェック
nyash-plugin validate new_plugin.so

# 出力例：
# ✅ ABI tag: OK (TYBX)
# ✅ Version: OK (1)
# ✅ Basic functions: OK
# ⚠️ Performance functions: Not implemented (optional)
# ✅ Thread safety: Declared
```

## 📊 段階的移行戦略

### Step 1: 互換ラッパー（1日）
- 最小限の変更でTypeBox形式に
- 既存の機能はそのまま維持
- テストがすべてパス

### Step 2: メソッドID化（1週間）
- resolve関数を実装
- 頻出メソッドにIDを割り当て
- 10-20%の性能向上

### Step 3: 高速実装（2週間）
- invoke_id関数を実装
- NyValue形式に対応
- 3-5倍の性能向上

### Step 4: 最適化（1ヶ月）
- JITフレンドリーな実装に
- インラインキャッシング対応
- 最大33倍の性能向上

## 🚨 注意事項

### メモリ管理
- NyValueは**ホスト管理**のメモリを使用
- 戻り値の所有権ルールに注意：
  - `NYASH_OWN_TRANSFER`: 呼び出し元が解放責任
  - `NYASH_OWN_BORROW`: プラグインが管理、触らない
  - `NYASH_OWN_CLONE`: コピーして返す

### スレッド安全性
- `NYASH_CAP_THREAD_SAFE`フラグを正しく設定
- グローバル状態を避ける
- 必要ならmutexで保護

### エラーハンドリング
```c
// 旧：NULLを返す
return NULL;

// 新：明示的なエラー
return ny_result_error("Invalid argument");
```

## 💡 ベストプラクティス

### 1. メソッドIDは列挙型で管理
```c
enum {
    METHOD_NONE = 0,
    METHOD_LENGTH = 1,
    METHOD_TO_UPPER = 2,
    METHOD_CONCAT = 3,
    // ...
};
```

### 2. 型情報を提供
```c
const char* my_get_type_info(void) {
    return "{"
        "\"methods\": ["
        "  {\"name\": \"length\", \"returns\": \"int\"},"
        "  {\"name\": \"toUpper\", \"returns\": \"string\"}"
        "]"
    "}";
}
```

### 3. プラグイン間連携を活用
```c
// 他のBoxを使う例
NyashTypeBox* array_type = ny_host_get_typebox("ArrayBox");
void* array = array_type->create(NULL);
```

## 📅 移行スケジュール

| フェーズ | 期間 | 内容 |
|---------|------|------|
| 現在 | - | 既存プラグインはそのまま動作 |
| Phase 1 | 3ヶ月 | 新規プラグインは統一形式推奨 |
| Phase 2 | 6ヶ月 | 移行ツール・ガイド充実 |
| Phase 3 | 9ヶ月 | パフォーマンス最適化 |
| Phase 4 | 12ヶ月 | 旧形式を非推奨に |

## 🆘 サポート

### ドキュメント
- [統一TypeBox ABI仕様](./unified-typebox-abi.md)
- [API詳細リファレンス](./specs/typebox-api-reference.md)
- [サンプルコード集](./examples/)

### コミュニティ
- Discord: #plugin-dev チャンネル
- GitHub: Issues/Discussionsで質問歓迎

### 移行支援
- 移行の相談・レビュー受付中
- 大規模プラグインの移行支援あり

## 🎯 まとめ

統一TypeBox ABIへの移行は：
- ✅ **段階的**：急ぐ必要なし
- ✅ **互換性重視**：既存コードを保護
- ✅ **ツール充実**：自動化でラクラク
- ✅ **大きなメリット**：性能・機能・将来性

**今すぐ始められる小さな一歩から！**