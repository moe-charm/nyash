# 統一TypeBox API リファレンス

## 📋 目次

1. [基本構造体](#基本構造体)
2. [関数ポインタ仕様](#関数ポインタ仕様)
3. [NyValue型システム](#nyvalue型システム)
4. [機能フラグ](#機能フラグ)
5. [エラーハンドリング](#エラーハンドリング)
6. [プラグイン間連携](#プラグイン間連携)

## 基本構造体

### NyashTypeBox

```c
typedef struct {
    uint32_t abi_tag;        // 必須: 0x54594258 ('TYBX')
    uint16_t version;        // APIバージョン（現在: 1）
    uint16_t struct_size;    // sizeof(NyashTypeBox)
    const char* name;        // Box型名（NULL終端）
    
    // 基本操作
    void* (*create)(void* args);
    void (*destroy)(void* self);
    
    // 高速ディスパッチ
    uint32_t (*resolve)(const char* name);
    NyResult (*invoke_id)(void* self, uint32_t method_id, 
                         NyValue* args, int argc);
    
    // 互換性
    void* (*method)(void* self, const char* name, 
                   void** args, int argc);
    
    // メタ情報
    const char* (*get_type_info)(void);
    uint64_t capabilities;
    
    // 予約済み
    void* reserved[4];
} NyashTypeBox;
```

## 関数ポインタ仕様

### create
```c
void* (*create)(void* args);
```
- **目的**: 新しいBoxインスタンスを生成
- **引数**: `args` - 初期化パラメータ（型依存）
- **戻り値**: 生成されたインスタンスのポインタ
- **所有権**: 呼び出し元が所有（destroy必須）

### destroy
```c
void (*destroy)(void* self);
```
- **目的**: Boxインスタンスを破棄
- **引数**: `self` - 破棄するインスタンス
- **注意**: NULLチェックは呼び出し元の責任

### resolve
```c
uint32_t (*resolve)(const char* name);
```
- **目的**: メソッド名を数値IDに変換
- **引数**: `name` - メソッド名
- **戻り値**: メソッドID（0 = 未知のメソッド）
- **用途**: JIT最適化、キャッシング

### invoke_id
```c
NyResult (*invoke_id)(void* self, uint32_t method_id, 
                     NyValue* args, int argc);
```
- **目的**: 高速メソッド呼び出し
- **引数**:
  - `self` - 対象インスタンス
  - `method_id` - resolveで取得したID
  - `args` - 引数配列
  - `argc` - 引数数
- **戻り値**: NyResult（成功/エラー）

### method（互換用）
```c
void* (*method)(void* self, const char* name, 
               void** args, int argc);
```
- **目的**: 従来互換のメソッド呼び出し
- **注意**: 新規実装では非推奨

### get_type_info
```c
const char* (*get_type_info)(void);
```
- **目的**: 型情報をJSON形式で返す
- **戻り値**: JSON文字列（静的メモリ）
- **形式例**:
```json
{
  "methods": [
    {"name": "length", "id": 1, "returns": "int"},
    {"name": "concat", "id": 2, "returns": "string"}
  ]
}
```

## NyValue型システム

### 基本構造
```c
typedef struct __attribute__((aligned(16))) {
    uint64_t type_tag;   // 型識別子
    union {
        int64_t  i64;    // 整数
        double   f64;    // 浮動小数点
        void*    ptr;    // ポインタ
        uint64_t bits;   // ビットパターン
    } payload;
} NyValue;
```

### 型タグ定義
```c
#define NYVAL_NULL      0x00
#define NYVAL_BOOL      0x01
#define NYVAL_INT       0x02
#define NYVAL_FLOAT     0x03
#define NYVAL_STRING    0x04
#define NYVAL_BOX       0x05
#define NYVAL_ARRAY     0x06
#define NYVAL_MAP       0x07
```

### ヘルパー関数
```c
// 値生成
NyValue ny_value_null(void);
NyValue ny_value_bool(bool val);
NyValue ny_value_int(int64_t val);
NyValue ny_value_float(double val);
NyValue ny_value_string(const char* str);
NyValue ny_value_box(void* box, NyashTypeBox* type);

// 値取得
bool ny_is_null(NyValue val);
bool ny_to_bool(NyValue val);
int64_t ny_to_int(NyValue val);
double ny_to_float(NyValue val);
const char* ny_to_string(NyValue val);
void* ny_to_box(NyValue val);
```

## 機能フラグ

### 基本フラグ
```c
#define NYASH_CAP_THREAD_SAFE    (1 << 0)  // スレッドセーフ
#define NYASH_CAP_ASYNC_SAFE     (1 << 1)  // async/await対応
#define NYASH_CAP_REENTRANT      (1 << 2)  // 再入可能
#define NYASH_CAP_PARALLELIZABLE (1 << 3)  // 並列実行可能
#define NYASH_CAP_PURE           (1 << 4)  // 副作用なし
#define NYASH_CAP_DETERMINISTIC  (1 << 5)  // 決定的動作
```

### 拡張フラグ
```c
#define NYASH_CAP_GPU_ACCEL      (1 << 8)  // GPU実行可能
#define NYASH_CAP_SIMD_OPTIMIZED (1 << 9)  // SIMD最適化済み
#define NYASH_CAP_LAZY_EVAL      (1 << 10) // 遅延評価対応
```

## エラーハンドリング

### NyResult構造体
```c
typedef struct {
    int status;      // 0 = 成功、非0 = エラー
    NyValue value;   // 戻り値（成功時）
    const char* error_msg;  // エラーメッセージ（エラー時）
} NyResult;
```

### ステータスコード
```c
#define NY_OK                0
#define NY_ERROR_GENERIC    -1
#define NY_ERROR_NULL_PTR   -2
#define NY_ERROR_TYPE       -3
#define NY_ERROR_BOUNDS     -4
#define NY_ERROR_NOT_FOUND  -5
#define NY_ERROR_MEMORY     -6
```

### ヘルパー関数
```c
NyResult ny_result_ok(NyValue val);
NyResult ny_result_error(const char* msg);
bool ny_result_is_ok(NyResult res);
bool ny_result_is_error(NyResult res);
```

## プラグイン間連携

### TypeBox取得
```c
NyashTypeBox* ny_host_get_typebox(const char* name);
```
- **目的**: 他のプラグインのTypeBoxを取得
- **引数**: `name` - Box型名
- **戻り値**: TypeBoxポインタ（NULL = 未登録）

### 使用例
```c
// MapBoxがArrayBoxを生成する例
NyResult map_keys(void* self, uint32_t method_id, 
                 NyValue* args, int argc) {
    MapBoxImpl* map = (MapBoxImpl*)self;
    
    // ArrayBoxを取得
    NyashTypeBox* array_type = ny_host_get_typebox("ArrayBox");
    if (!array_type) {
        return ny_result_error("ArrayBox not found");
    }
    
    // 新しいArrayBoxを生成
    void* array = array_type->create(NULL);
    
    // キーを追加
    for (int i = 0; i < map->key_count; i++) {
        NyValue key = ny_value_string(map->keys[i]);
        array_type->invoke_id(array, 1, &key, 1); // push
    }
    
    return ny_result_ok(ny_value_box(array, array_type));
}
```

## 所有権ルール

### 基本原則
1. **create**の戻り値 → 呼び出し元が**destroy**責任
2. **invoke_id**の引数 → プラグインは**借用**（変更/解放禁止）
3. **invoke_id**の戻り値 → 呼び出し元が所有

### 文字列の扱い
- 引数の文字列：コピー必須（借用のため）
- 戻り値の文字列：新規確保（呼び出し元が解放）

### メモリ管理ベストプラクティス
```c
// 良い例：文字列をコピー
void* string_concat(void* self, NyValue* args, int argc) {
    char* str1 = (char*)self;
    char* str2 = ny_to_string(args[0]);
    
    // 新しいメモリを確保
    size_t len = strlen(str1) + strlen(str2) + 1;
    char* result = malloc(len);
    snprintf(result, len, "%s%s", str1, str2);
    
    return ny_value_string(result);
}
```

## バージョニング

### APIバージョン
- **現在**: 1
- **互換性**: 上位互換を維持
- **チェック**: `version`フィールドで確認

### 構造体サイズ
- **用途**: 前方互換性の確保
- **チェック**: `struct_size >= sizeof(NyashTypeBox)`

### 将来の拡張
- `reserved`配列を使用
- 新フィールドは構造体末尾に追加
- 既存フィールドの意味は変更禁止