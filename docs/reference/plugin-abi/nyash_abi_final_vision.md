# Nyash ABI 最終形態 (Final Vision)

**作成日**: 2025-09-30
**更新日**: 2025-09-30（ChatGPT Pro提案統合）
**ステータス**: 設計ビジョン（長期目標）
**現行版**: v2 ([nyash_abi_v2.md](nyash_abi_v2.md))
**Phase 12**: 青写真 ([phase-12/unified-typebox-abi.md](../../development/roadmap/phases/phase-12/unified-typebox-abi.md))

---

## 🎯 設計哲学：美しく・綺麗で・機能も十分

この文書は、Nyash ABIの**最終到達点**を描きます。
Phase 12の青写真と**ChatGPT Pro「6アーキタイプ」提案**を統合し、**シンプルさを保ちつつ完全な機能性**を実現する設計です。

### 核心原則

1. **型安全性**: コンパイル時に型エラーを検出
2. **ゼロコスト抽象化**: 抽象層のオーバーヘッドなし
3. **完全なメタデータ**: リフレクション・ツール統合
4. **能力ベース設計**: スレッド安全性・非同期・GPU等の宣言的表現（ChatGPT Pro）
5. **境界強化**: プラグインBox境界での効果・契約検証（ChatGPT Pro）
6. **拡張可能性**: 将来の機能追加に対応

### 🆕 ChatGPT Pro提案の核心理念

**「プラグインBoxの境界を強くする」**

- ✅ プラグインBox（FileBox、P2PBox等）: 能力・効果・契約で境界強化
- ❌ ユーザーBox（Dog、EnhancedP2P等）: 変更なし（信頼済みコード）

**理由**:
- **信頼境界**: プラグイン=外部バイナリ（信頼できない） vs ユーザーBox=自分のコード（信頼済み）
- **能力の意味**: プラグインだけが実際のシステムコール実行（ファイルI/O、ネット通信等）
- **ユーザー体験**: 構文変更なし、より安全になるだけ

---

## 📋 TypeBox FFI構造体（最終形態）

```c
// ============================================
// Nyash TypeBox ABI - Final Vision
// Phase 12 + ChatGPT Pro 6アーキタイプ統合版
// ============================================

// --- 値の抽象型 ---
typedef struct {
    uint8_t tag;      // 1=bool, 2=i32, 3=i64, 4=f32, 5=f64,
                      // 6=string, 7=bytes, 8=handle, 9=null
    union {
        bool b;
        int32_t i32;
        int64_t i64;
        float f32;
        double f64;
        struct { const char* ptr; size_t len; } str;
        struct { const uint8_t* ptr; size_t len; } bytes;
        uint32_t handle;  // ← プラグインBox・ユーザーBoxどちらも同じhandle
    } value;
} NyValue;

// --- 関数結果型 ---
typedef struct {
    int32_t status;    // 0=success, <0=error code
    NyValue result;    // 成功時の戻り値
    const char* error; // エラー時のメッセージ（NULL可）
} NyResult;

// --- メソッドメタデータ ---
typedef struct {
    uint32_t method_id;
    const char* name;           // "concat", "add", etc.
    const char* signature;      // "(StringBox) -> StringBox"
    const char* description;    // "文字列を連結します"
    uint8_t arg_count;
    const char* arg_types[8];   // ["StringBox", ...]
    const char* return_type;    // "StringBox"
} NyMethodMeta;

// --- 型情報（JSON形式） ---
typedef struct {
    const char* json;  // 完全な型情報をJSONで返却
    size_t len;
} NyTypeInfo;

// ============================================
// ChatGPT Pro提案: 能力・効果・契約システム
// ============================================

// --- 能力フラグ定義 ---
#define NY_CAP_THREAD_SAFE   (1ULL << 0)  // スレッド安全
#define NY_CAP_ASYNC_SAFE    (1ULL << 1)  // 非同期コンテキスト安全
#define NY_CAP_IMMUTABLE     (1ULL << 2)  // 不変型
#define NY_CAP_CLONE         (1ULL << 3)  // Clone可能
#define NY_CAP_SEND          (1ULL << 4)  // スレッド間送信可能
#define NY_CAP_SYNC          (1ULL << 5)  // スレッド間共有可能
#define NY_CAP_DETERMINISTIC (1ULL << 6)  // 決定的実行保証
#define NY_CAP_GPU_ACCEL     (1ULL << 8)  // GPU加速対応
#define NY_CAP_SIMD_ACCEL    (1ULL << 9)  // SIMD最適化対応
#define NY_CAP_ZERO_COPY     (1ULL << 10) // ゼロコピー可能

// --- 効果宣言（Nyash/E: Effect & Capability First） ---
typedef struct {
    uint32_t method_id;
    const char** effects;     // ["io.read", "mem.alloc", NULL]
    size_t effect_count;
} NyMethodEffect;

// --- 契約（Nyash/V: Verified by Construction） ---
typedef struct {
    uint32_t method_id;
    const char* precondition;   // "len(path) > 0 && is_valid_path(path)"
    const char* postcondition;  // "result != null || error != null"
} NyMethodContract;

// --- Component Model情報（Nyash/C: Component Model Native） ---
typedef struct {
    const char* wit_schema;    // WIT形式のインターフェース定義
    size_t wit_schema_len;
    uint16_t min_version;      // 最小互換バージョン
    uint16_t max_version;      // 最大互換バージョン
} NyComponentInfo;

// ============================================
// TypeBox 最終形態（完全版）
// ============================================
typedef struct {
    // === コア識別 ===
    uint32_t abi_tag;        // 'TYFN' (0x5459464E) - TypeBox Final
    uint16_t version;        // バージョン (例: 100 = v1.0.0)
    uint16_t struct_size;    // 構造体サイズ（前方互換性）
    const char* name;        // Box型名："StringBox"

    // === 基本操作 ===
    // インスタンス作成（NyValue配列で引数受け取り）
    NyResult (*create)(const NyValue* args, size_t argc);

    // インスタンス破棄（手動破棄が必要な場合のみ）
    void (*destroy)(uint32_t instance_id);

    // === 高速メソッドディスパッチ ===
    // メソッド名→IDの解決（起動時1回のみ）
    uint32_t (*resolve)(const char* method_name);

    // メソッド呼び出し（高速パス）
    NyResult (*invoke)(uint32_t instance_id,
                       uint32_t method_id,
                       const NyValue* args,
                       size_t argc);

    // === メタデータ取得 ===
    // メソッド情報取得（IDE補完・デバッガー用）
    const NyMethodMeta* (*get_method_meta)(uint32_t method_id);

    // メソッド一覧取得
    size_t (*get_method_count)(void);
    const NyMethodMeta** (*get_all_methods)(void);

    // 完全な型情報（JSON形式）
    NyTypeInfo (*get_type_info)(void);

    // ============================================
    // ChatGPT Pro提案: プラグインBox境界強化
    // ============================================

    // === 能力・効果（Nyash/E） ===
    const char** required_capabilities;  // プラグインが要求する能力
                                        // 例: ["fs.read", "net.out", NULL]
    size_t required_cap_count;

    const NyMethodEffect* method_effects;  // メソッドごとの効果宣言
    size_t method_effect_count;

    // === 契約（Nyash/V） ===
    const NyMethodContract* contracts;   // 事前/事後条件
    size_t contract_count;

    // === Component Model（Nyash/C） ===
    const NyComponentInfo* component_info;  // WIT互換情報（NULL可）

    // === 能力フラグ ===
    uint64_t capabilities;   // 能力フラグのビットマスク

    // === 将来拡張用 ===
    void* reserved[4];       // ABI互換性維持用
} NyashTypeBoxFinal;
```

---

## 🎨 設計の美しさ

### 1. 型安全な値表現

```c
// ❌ 生のバイト列（v2方式）
uint8_t* raw_bytes;  // 型情報なし、デバッグ困難

// ✅ NyValue抽象型（最終形態）
NyValue value;
value.tag = 3;          // i64型
value.value.i64 = 42;   // 型安全な代入

// ✅ プラグインBox・ユーザーBoxの垣根なし
value.tag = 8;                    // handle型
value.value.handle = instance_id; // どちらのBoxも同じhandle型
```

### 2. エラーハンドリングの明確化

```c
// ❌ エラーコードのみ（情報不足）
int32_t status = invoke(...);
if (status < 0) {
    // エラー原因不明
}

// ✅ NyResult型（詳細なエラー情報）
NyResult result = typebox->invoke(instance_id, method_id, args, argc);
if (result.status < 0) {
    fprintf(stderr, "Error: %s (code: %d)\n", result.error, result.status);
    // エラー原因明確
}
```

### 3. メタデータによる開発体験向上

```rust
// プラグイン側実装例（C）
const NyMethodMeta STRING_CONCAT_META = {
    .method_id = 1,
    .name = "concat",
    .signature = "(StringBox) -> StringBox",
    .description = "文字列を連結します",
    .arg_count = 1,
    .arg_types = {"StringBox"},
    .return_type = "StringBox"
};

const NyMethodMeta* get_method_meta(uint32_t method_id) {
    if (method_id == 1) return &STRING_CONCAT_META;
    return NULL;
}
```

---

## 🧹 綺麗な設計

### resolve/invokeの二段階ディスパッチ（method()削除維持）

Phase 12ではmethod()フォールバックが含まれていましたが、**最終形態では完全削除**します。

**理由**:
1. **シンプルさ**: 呼び出しパスが1つだけ
2. **性能**: 文字列比較のオーバーヘッドなし
3. **型安全性**: method_id経由で静的解析可能

```c
// ❌ Phase 12のmethod()フォールバック
void* (*method)(void* self, const char* name, void** args, int argc);
// 問題: 文字列比較遅い、型情報なし、デバッグ困難

// ✅ 最終形態: resolve + invoke のみ
uint32_t method_id = typebox->resolve("concat");  // 起動時1回
NyResult result = typebox->invoke(instance_id, method_id, args, argc);  // 実行時高速
```

### ゼロコスト抽象化

```c
// NyValue → TLV変換（最適化で消える）
static inline void ny_value_encode_tlv(const NyValue* val, uint8_t* buf) {
    buf[0] = val->tag;
    memcpy(buf + 1, &val->value, sizeof(val->value));
}

// 最適化後: 直接TLV生成（オーバーヘッドなし）
// コンパイラがインライン展開 → NyValueレイヤーが消失
```

---

## 🚀 機能の充実（ChatGPT Pro統合）

### 1. 能力ベース型システム（Nyash/E）

**目的**: プラグインBox境界での最小権限強制

```rust
// プラグイン宣言（nyash.toml経由）
NyashTypeBoxFinal file_box = {
    .name = "FileBox",
    .required_capabilities = (const char*[]){"fs.read", "fs.write", NULL},
    .required_cap_count = 2,
    .capabilities = NY_CAP_THREAD_SAFE,
    // ...
};

// Rust実行器側でプラグインロード時検証
impl PluginLoader {
    fn load(&mut self, typebox: &NyashTypeBoxFinal) -> Result<()> {
        // 能力トークン検証（プラグインのみ）
        for cap in typebox.required_capabilities() {
            if !self.runtime_caps.has(cap) {
                return Err(format!("Plugin {} missing capability: {}",
                                   typebox.name(), cap));
            }
        }
        Ok(())
    }
}
```

**ユーザーBoxへの影響**: ❌ なし（能力チェックはプラグイン境界のみ）

### 2. 効果トレース（Nyash/E）

**目的**: プラグインBoxメソッドが引き起こす副作用の可視化

```c
// プラグイン側で効果宣言
static const NyMethodEffect FILE_EFFECTS[] = {
    {.method_id = 1, .effects = (const char*[]){"fs.open", "mem.alloc", NULL}, .effect_count = 2},
    {.method_id = 2, .effects = (const char*[]){"io.read", "mem.alloc", NULL}, .effect_count = 2},
    {.method_id = 3, .effects = (const char*[]){"io.write", NULL}, .effect_count = 1},
};

const NyashTypeBoxFinal TypeBox_FileBox = {
    .method_effects = FILE_EFFECTS,
    .method_effect_count = 3,
    // ...
};
```

```rust
// 実行器側で効果トレース
impl PluginInvoker {
    fn invoke_with_trace(&self, method_id: u32, args: &[NyValue]) -> NyResult {
        // 効果トレース（開発時のみ）
        if env::var("NYASH_TRACE_EFFECTS").is_ok() {
            if let Some(effects) = self.get_method_effects(method_id) {
                eprintln!("[TRACE] {}.{}() effects: {:?}",
                         self.box_name, self.method_name(method_id), effects);
            }
        }

        // 通常の呼び出し
        self.typebox.invoke(self.instance_id, method_id, args, args.len())
    }
}
```

**ユーザーBoxへの影響**: ❌ なし（効果はプラグイン境界で発生）

### 3. 契約検証（Nyash/V）

**目的**: プラグインBox境界での引数・戻り値検証

```c
// プラグイン側で契約宣言
static const NyMethodContract FILE_CONTRACTS[] = {
    {
        .method_id = 1,  // open
        .precondition = "len(path) > 0 && is_valid_path(path)",
        .postcondition = "is_valid_handle(result) || error != null"
    },
    {
        .method_id = 2,  // read
        .precondition = "is_open(self) && len > 0",
        .postcondition = "len(result) <= len || error != null"
    },
};

const NyashTypeBoxFinal TypeBox_FileBox = {
    .contracts = FILE_CONTRACTS,
    .contract_count = 2,
    // ...
};
```

```rust
// 実行器側で契約チェック
impl PluginInvoker {
    fn invoke_with_contract(&self, method_id: u32, args: &[NyValue]) -> NyResult {
        // 事前条件チェック（プラグイン境界のみ）
        if let Some(contract) = self.get_contract(method_id) {
            if !self.eval_precondition(contract, args) {
                return NyResult::contract_violation(
                    format!("Precondition failed: {}", contract.precondition)
                );
            }
        }

        // 実際の呼び出し
        let result = self.typebox.invoke(self.instance_id, method_id, args, args.len());

        // 事後条件チェック
        if let Some(contract) = self.get_contract(method_id) {
            if !self.eval_postcondition(contract, &result) {
                return NyResult::contract_violation(
                    format!("Postcondition failed: {}", contract.postcondition)
                );
            }
        }

        result
    }
}
```

**ユーザーBoxへの影響**: ❌ なし（契約はプラグイン境界のみ）

### 4. Component Model統合（Nyash/C）

**目的**: WIT互換でエコシステム統合

```c
// WIT定義（プラグイン開発者が提供）
static const char FILE_WIT[] =
"interface file {\n"
"  open: func(path: string) -> result<handle, error>\n"
"  read: func(h: handle, len: u32) -> result<list<u8>, error>\n"
"  write: func(h: handle, data: list<u8>) -> result<u32, error>\n"
"}\n";

static const NyComponentInfo FILE_COMPONENT_INFO = {
    .wit_schema = FILE_WIT,
    .wit_schema_len = sizeof(FILE_WIT) - 1,
    .min_version = 100,  // v1.0.0
    .max_version = 199,  // v1.99.99
};

const NyashTypeBoxFinal TypeBox_FileBox = {
    .component_info = &FILE_COMPONENT_INFO,
    // ...
};
```

**ユーザーBoxへの影響**: ❌ なし（WIT情報はプラグインのみ）

### 5. JSON型情報（完全リフレクション）

```json
// get_type_info()の返却例（プラグインBoxのみ）
{
  "name": "FileBox",
  "version": "1.0.0",
  "abi_version": 100,
  "capabilities": {
    "thread_safe": true,
    "async_safe": false,
    "required": ["fs.read", "fs.write"]
  },
  "methods": [
    {
      "id": 1,
      "name": "open",
      "signature": "(string) -> result<handle, error>",
      "effects": ["fs.open", "mem.alloc"],
      "contract": {
        "pre": "len(path) > 0",
        "post": "is_valid_handle(result) || error != null"
      }
    }
  ],
  "component": {
    "wit_schema": "interface file { ... }",
    "min_version": "1.0.0",
    "max_version": "1.99.99"
  }
}
```

---

## 📊 完全性の指標

| 機能 | v2 | Phase 12 | 最終形態 |
|------|----|---------:|--------:|
| 型安全性 | 60% | 90% | **95%** |
| エラー情報 | 30% | 70% | **95%** |
| メタデータ | 0% | 50% | **100%** |
| 能力判定 | 0% | 70% | **100%** ⭐ |
| 効果トレース | 0% | 0% | **100%** ⭐ |
| 契約検証 | 0% | 0% | **100%** ⭐ |
| リフレクション | 0% | 30% | **100%** |
| ツール統合 | 20% | 60% | **100%** |
| Component Model | 0% | 0% | **100%** ⭐ |
| 最適化ヒント | 10% | 50% | **100%** |
| 後方互換性 | 100% | 80% | **100%** ⭐ |
| シンプルさ | 100% | 60% | **85%** |

⭐ = ChatGPT Pro提案による強化

---

## 🎯 Phase 12 vs ChatGPT Pro統合版

| 項目 | Phase 12 | 最終形態 | 理由 |
|------|----------|----------|------|
| method()フォールバック | ✅ あり | ❌ なし | シンプルさ優先 |
| NyValue型 | ✅ あり | ✅ あり | 型安全性 |
| NyResult型 | ❌ なし | ✅ あり | エラー情報充実 |
| get_method_meta() | ❌ なし | ✅ あり | 開発体験向上 |
| get_all_methods() | ❌ なし | ✅ あり | ツール統合 |
| JSON型情報 | ✅ 簡易版 | ✅ 完全版 | リフレクション完全対応 |
| capabilities活用 | ⚠️ 定義のみ | ✅ 完全実装 | 最適化・安全性 |
| **required_capabilities** | ❌ なし | ✅ あり | ⭐ 能力チェック |
| **method_effects** | ❌ なし | ✅ あり | ⭐ 効果トレース |
| **contracts** | ❌ なし | ✅ あり | ⭐ 契約検証 |
| **component_info** | ❌ なし | ✅ あり | ⭐ WIT互換 |

⭐ = ChatGPT Pro提案による追加機能

---

## 🛠️ 実装例

### プラグイン側（C言語） - ChatGPT Pro統合版

```c
// plugins/filebox/filebox.c
#include "nyash_abi_final.h"

// --- 能力・効果・契約宣言（ChatGPT Pro） ---
static const char* REQUIRED_CAPS[] = {"fs.read", "fs.write", NULL};

static const NyMethodEffect METHOD_EFFECTS[] = {
    {.method_id = 1, .effects = (const char*[]){"fs.open", "mem.alloc", NULL}, .effect_count = 2},
    {.method_id = 2, .effects = (const char*[]){"io.read", "mem.alloc", NULL}, .effect_count = 2},
};

static const NyMethodContract METHOD_CONTRACTS[] = {
    {.method_id = 1, .precondition = "len(path) > 0", .postcondition = "is_valid_handle(result)"},
    {.method_id = 2, .precondition = "is_open(self)", .postcondition = "len(result) >= 0"},
};

// --- WIT互換情報（ChatGPT Pro） ---
static const char WIT_SCHEMA[] =
"interface file {\n"
"  open: func(path: string) -> result<handle, error>\n"
"  read: func(h: handle, len: u32) -> result<list<u8>, error>\n"
"}\n";

static const NyComponentInfo COMPONENT_INFO = {
    .wit_schema = WIT_SCHEMA,
    .wit_schema_len = sizeof(WIT_SCHEMA) - 1,
    .min_version = 100,
    .max_version = 199,
};

// --- メソッドメタデータ ---
static const NyMethodMeta METHOD_OPEN = {
    .method_id = 1,
    .name = "open",
    .signature = "(string) -> result<handle, error>",
    .description = "ファイルを開きます",
    .arg_count = 1,
    .arg_types = {"string"},
    .return_type = "handle"
};

static const NyMethodMeta METHOD_READ = {
    .method_id = 2,
    .name = "read",
    .signature = "(u32) -> result<bytes, error>",
    .description = "ファイルから読み込みます",
    .arg_count = 1,
    .arg_types = {"u32"},
    .return_type = "bytes"
};

static const NyMethodMeta* ALL_METHODS[] = {&METHOD_OPEN, &METHOD_READ, NULL};

// --- 実装 ---
static NyResult file_create(const NyValue* args, size_t argc) {
    // 引数チェック（契約検証は実行器が行うので簡易チェックのみ）
    if (argc != 1 || args[0].tag != 6) {  // 6 = string
        return (NyResult){
            .status = -1,
            .error = "Expected 1 string argument"
        };
    }

    uint32_t instance_id = allocate_file_instance(args[0].value.str.ptr);
    return (NyResult){
        .status = 0,
        .result = {.tag = 8, .value.handle = instance_id}
    };
}

static void file_destroy(uint32_t instance_id) {
    free_file_instance(instance_id);
}

static uint32_t file_resolve(const char* name) {
    if (strcmp(name, "open") == 0) return 1;
    if (strcmp(name, "read") == 0) return 2;
    return 0;
}

static NyResult file_invoke(uint32_t instance_id, uint32_t method_id,
                            const NyValue* args, size_t argc) {
    FileInstance* file = get_file_instance(instance_id);

    switch (method_id) {
        case 1: {  // open
            if (argc != 1 || args[0].tag != 6) {
                return (NyResult){.status = -1, .error = "Invalid argument"};
            }
            int fd = open(args[0].value.str.ptr, O_RDONLY);
            if (fd < 0) {
                return (NyResult){.status = -2, .error = strerror(errno)};
            }
            file->fd = fd;
            return (NyResult){.status = 0, .result = {.tag = 8, .value.handle = instance_id}};
        }
        case 2: {  // read
            if (argc != 1 || args[0].tag != 2) {  // 2 = i32
                return (NyResult){.status = -1, .error = "Invalid argument"};
            }
            uint32_t len = args[0].value.i32;
            uint8_t* buf = malloc(len);
            ssize_t n = read(file->fd, buf, len);
            if (n < 0) {
                free(buf);
                return (NyResult){.status = -3, .error = strerror(errno)};
            }
            return (NyResult){
                .status = 0,
                .result = {.tag = 7, .value.bytes = {.ptr = buf, .len = n}}
            };
        }
        default:
            return (NyResult){.status = -4, .error = "Unknown method"};
    }
}

static const NyMethodMeta* file_get_method_meta(uint32_t method_id) {
    if (method_id == 1) return &METHOD_OPEN;
    if (method_id == 2) return &METHOD_READ;
    return NULL;
}

static size_t file_get_method_count(void) {
    return 2;
}

static const NyMethodMeta** file_get_all_methods(void) {
    return ALL_METHODS;
}

static NyTypeInfo file_get_type_info(void) {
    static const char type_info_json[] =
    "{"
    "  \"name\": \"FileBox\","
    "  \"version\": \"1.0.0\","
    "  \"capabilities\": {\"required\": [\"fs.read\", \"fs.write\"]},"
    "  \"methods\": ["
    "    {\"id\": 1, \"name\": \"open\", \"effects\": [\"fs.open\", \"mem.alloc\"]},"
    "    {\"id\": 2, \"name\": \"read\", \"effects\": [\"io.read\", \"mem.alloc\"]}"
    "  ]"
    "}";
    return (NyTypeInfo){
        .json = type_info_json,
        .len = sizeof(type_info_json) - 1
    };
}

// --- TypeBox エクスポート（ChatGPT Pro統合版） ---
__attribute__((visibility("default")))
const NyashTypeBoxFinal TypeBox_FileBox = {
    .abi_tag = 0x5459464E,  // 'TYFN'
    .version = 100,         // v1.0.0
    .struct_size = sizeof(NyashTypeBoxFinal),
    .name = "FileBox",

    .create = file_create,
    .destroy = file_destroy,
    .resolve = file_resolve,
    .invoke = file_invoke,

    .get_method_meta = file_get_method_meta,
    .get_method_count = file_get_method_count,
    .get_all_methods = file_get_all_methods,
    .get_type_info = file_get_type_info,

    // === ChatGPT Pro: 境界強化 ===
    .required_capabilities = REQUIRED_CAPS,
    .required_cap_count = 2,

    .method_effects = METHOD_EFFECTS,
    .method_effect_count = 2,

    .contracts = METHOD_CONTRACTS,
    .contract_count = 2,

    .component_info = &COMPONENT_INFO,

    .capabilities = NY_CAP_THREAD_SAFE,

    .reserved = {NULL, NULL, NULL, NULL}
};
```

### Rust実行器側（ChatGPT Pro統合版）

```rust
// src/runtime/plugin_loader_final/mod.rs
use std::ffi::{CStr, c_char};

#[repr(C)]
pub struct NyValue {
    tag: u8,
    value: NyValueUnion,
}

#[repr(C)]
union NyValueUnion {
    b: bool,
    i32_val: i32,
    i64_val: i64,
    f32_val: f32,
    f64_val: f64,
    str_val: NyString,
    bytes_val: NyBytes,
    handle: u32,
}

#[repr(C)]
struct NyString {
    ptr: *const c_char,
    len: usize,
}

#[repr(C)]
pub struct NyResult {
    status: i32,
    result: NyValue,
    error: *const c_char,
}

#[repr(C)]
pub struct NyMethodMeta {
    method_id: u32,
    name: *const c_char,
    signature: *const c_char,
    description: *const c_char,
    arg_count: u8,
    arg_types: [*const c_char; 8],
    return_type: *const c_char,
}

#[repr(C)]
pub struct NyMethodEffect {
    method_id: u32,
    effects: *const *const c_char,
    effect_count: usize,
}

#[repr(C)]
pub struct NyMethodContract {
    method_id: u32,
    precondition: *const c_char,
    postcondition: *const c_char,
}

#[repr(C)]
pub struct NyComponentInfo {
    wit_schema: *const c_char,
    wit_schema_len: usize,
    min_version: u16,
    max_version: u16,
}

#[repr(C)]
pub struct NyashTypeBoxFinal {
    abi_tag: u32,
    version: u16,
    struct_size: u16,
    name: *const c_char,

    create: Option<extern "C" fn(*const NyValue, usize) -> NyResult>,
    destroy: Option<extern "C" fn(u32)>,
    resolve: Option<extern "C" fn(*const c_char) -> u32>,
    invoke: Option<extern "C" fn(u32, u32, *const NyValue, usize) -> NyResult>,

    get_method_meta: Option<extern "C" fn(u32) -> *const NyMethodMeta>,
    get_method_count: Option<extern "C" fn() -> usize>,
    get_all_methods: Option<extern "C" fn() -> *const *const NyMethodMeta>,
    get_type_info: Option<extern "C" fn() -> NyTypeInfo>,

    // ChatGPT Pro: 境界強化
    required_capabilities: *const *const c_char,
    required_cap_count: usize,
    method_effects: *const NyMethodEffect,
    method_effect_count: usize,
    contracts: *const NyMethodContract,
    contract_count: usize,
    component_info: *const NyComponentInfo,

    capabilities: u64,
    reserved: [*const (); 4],
}

impl NyashTypeBoxFinal {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.name).to_str().unwrap() }
    }

    pub fn has_capability(&self, cap: u64) -> bool {
        (self.capabilities & cap) != 0
    }

    // ChatGPT Pro: 能力取得
    pub fn required_capabilities(&self) -> Vec<String> {
        if self.required_capabilities.is_null() {
            return vec![];
        }
        unsafe {
            let mut caps = vec![];
            let mut i = 0;
            loop {
                let cap_ptr = *self.required_capabilities.add(i);
                if cap_ptr.is_null() {
                    break;
                }
                caps.push(CStr::from_ptr(cap_ptr).to_str().unwrap().to_string());
                i += 1;
            }
            caps
        }
    }

    // ChatGPT Pro: 効果取得
    pub fn get_method_effects(&self, method_id: u32) -> Option<Vec<String>> {
        if self.method_effects.is_null() {
            return None;
        }
        unsafe {
            let effects_slice = std::slice::from_raw_parts(
                self.method_effects, self.method_effect_count
            );
            for effect in effects_slice {
                if effect.method_id == method_id {
                    let mut effects_vec = vec![];
                    for i in 0..effect.effect_count {
                        let eff_ptr = *effect.effects.add(i);
                        effects_vec.push(CStr::from_ptr(eff_ptr).to_str().unwrap().to_string());
                    }
                    return Some(effects_vec);
                }
            }
            None
        }
    }

    // ChatGPT Pro: 契約取得
    pub fn get_contract(&self, method_id: u32) -> Option<ContractInfo> {
        if self.contracts.is_null() {
            return None;
        }
        unsafe {
            let contracts_slice = std::slice::from_raw_parts(
                self.contracts, self.contract_count
            );
            for contract in contracts_slice {
                if contract.method_id == method_id {
                    return Some(ContractInfo {
                        precondition: CStr::from_ptr(contract.precondition)
                            .to_str().unwrap().to_string(),
                        postcondition: CStr::from_ptr(contract.postcondition)
                            .to_str().unwrap().to_string(),
                    });
                }
            }
            None
        }
    }
}

pub struct ContractInfo {
    pub precondition: String,
    pub postcondition: String,
}

// ChatGPT Pro: プラグインロード時検証
impl PluginLoader {
    pub fn load(&mut self, typebox: &NyashTypeBoxFinal) -> Result<(), String> {
        // 能力チェック（プラグインのみ）
        for cap in typebox.required_capabilities() {
            if !self.runtime_caps.has(&cap) {
                return Err(format!(
                    "Plugin {} requires capability '{}' which is not available",
                    typebox.name(), cap
                ));
            }
        }

        // 効果トレース有効化
        if std::env::var("NYASH_TRACE_EFFECTS").is_ok() {
            eprintln!("[PLUGIN] {} loaded with effects tracing enabled", typebox.name());
        }

        Ok(())
    }
}

// ChatGPT Pro: 契約付き呼び出し
impl PluginInvoker {
    pub fn invoke_with_contract(
        &self,
        method_id: u32,
        args: &[NyValue]
    ) -> Result<NyValue, String> {
        // 事前条件チェック（簡易版）
        if let Some(contract) = self.typebox.get_contract(method_id) {
            if std::env::var("NYASH_CHECK_CONTRACTS").is_ok() {
                eprintln!("[CONTRACT] Precondition: {}", contract.precondition);
                // TODO: 実際の条件評価
            }
        }

        // 効果トレース
        if let Some(effects) = self.typebox.get_method_effects(method_id) {
            if std::env::var("NYASH_TRACE_EFFECTS").is_ok() {
                eprintln!("[EFFECT] {}.method_{} effects: {:?}",
                         self.typebox.name(), method_id, effects);
            }
        }

        // 実際の呼び出し
        let invoke_fn = self.typebox.invoke.ok_or("No invoke function")?;
        let result = invoke_fn(self.instance_id, method_id, args.as_ptr(), args.len());

        if result.status < 0 {
            let error_msg = if result.error.is_null() {
                format!("Error code: {}", result.status)
            } else {
                unsafe { CStr::from_ptr(result.error).to_str().unwrap().to_string() }
            };
            return Err(error_msg);
        }

        // 事後条件チェック（簡易版）
        if let Some(contract) = self.typebox.get_contract(method_id) {
            if std::env::var("NYASH_CHECK_CONTRACTS").is_ok() {
                eprintln!("[CONTRACT] Postcondition: {}", contract.postcondition);
                // TODO: 実際の条件評価
            }
        }

        Ok(result.result)
    }
}
```

---

## 🎯 ユーザーBoxは完全に変更なし

### ユーザーコード（影響ゼロ）

```nyash
// ユーザー定義Box - 何も変わらない！
box Dog {
    init { name, breed }

    birth(dogName, dogBreed) {
        me.name = dogName
        me.breed = dogBreed
    }

    bark() {
        print(me.name + " says woof!")  // ← print()はプラグイン境界でチェック
    }
}

// プラグインBoxからのデリゲート - 何も変わらない！
box EnhancedP2P from P2PBox {
    init { extraFeatures }

    birth(nodeId, transport) {
        from P2PBox.birth(nodeId, transport)  // ← プラグイン境界でチェック
        me.extraFeatures = new ArrayBox()
    }

    sendMessage(msg) {
        // P2PBox.send()を呼ぶ際にプラグイン境界でチェックされる
        me.send(msg)
    }
}

// 使用例 - 何も変わらない！
local dog = new Dog("Buddy", "Labrador")
dog.bark()

local node = new EnhancedP2P("node1", "tcp")
node.sendMessage("Hello")
```

### 信頼境界の図解

```
┌─────────────────────────────────────┐
│  Nyashユーザーコード（信頼済み）      │
│                                     │
│  box Dog { ... }                   │
│  box EnhancedP2P from P2PBox { ... }│
│                                     │
│  ❌ チェックなし（信頼済みコード）    │
└─────────────────────────────────────┘
              │
              │ call method
              ↓
┌═════════════════════════════════════┐
│  🛡️ プラグイン境界（信頼境界）       │
│                                     │
│  ✅ 能力チェック                     │
│  ✅ 効果トレース                     │
│  ✅ 契約検証                         │
└═════════════════════════════════════┘
              │
              ↓
┌─────────────────────────────────────┐
│  プラグインBox（外部バイナリ）        │
│                                     │
│  FileBox / P2PBox / ConsoleBox etc. │
│                                     │
│  ⚠️ 信頼できない（要検証）            │
└─────────────────────────────────────┘
```

---

## 📚 nyash.toml拡張スキーマ（ChatGPT Pro統合版）

```toml
# ============================================
# nyash.toml - ChatGPT Pro統合版
# ============================================

# === 既存セクション（変更なし）===
[plugin]
name = "FileBox"
version = "1.0.0"
description = "ファイル操作Box"

# === ChatGPT Pro: 能力・効果（新規）===
[plugin.capabilities]
# プラグインが要求する能力（起動時チェック）
required = ["fs.read", "fs.write", "mem.alloc"]

[plugin.effects]
# メソッドごとの効果宣言（トレース・検証用）
"open" = ["fs.open", "mem.alloc"]
"read" = ["io.read", "mem.alloc"]
"write" = ["io.write"]
"close" = ["fs.close", "mem.free"]

# === ChatGPT Pro: 契約（新規）===
[plugin.contracts]
# メソッドごとの事前/事後条件
[plugin.contracts.open]
pre = "len(path) > 0 && is_valid_path(path)"
post = "is_valid_handle(result) || error != null"

[plugin.contracts.read]
pre = "is_open(self) && len > 0"
post = "len(result) <= len || error != null"

[plugin.contracts.write]
pre = "is_open(self) && len(data) > 0"
post = "result == len(data) || error != null"

# === ChatGPT Pro: Component Model（新規、オプション）===
[plugin.component]
# WIT互換情報（オプション）
wit_file = "file.wit"  # 外部ファイル参照
min_version = "1.0.0"
max_version = "1.99.99"

# === 既存: メソッド情報（拡張）===
[plugin.methods]
[plugin.methods.open]
args = [{type = "string", name = "path"}]
return = {type = "result", ok = "handle", err = "error"}
description = "ファイルを開きます"

[plugin.methods.read]
args = [{type = "handle", name = "self"}, {type = "u32", name = "len"}]
return = {type = "result", ok = "bytes", err = "error"}
description = "ファイルから読み込みます"
```

---

## 🎉 結論：最終形態の特徴

### ✅ 美しさ

- **型安全**: NyValue/NyResultで完全な型情報
- **エラー明確**: ステータスコード + メッセージ
- **メタデータ充実**: メソッド情報・JSON型情報
- **プラグイン/ユーザーBoxの垣根なし**: 同じhandle型で統一 ⭐

### ✅ 綺麗さ

- **method()削除**: resolve/invokeの二段階のみ
- **シンプルな呼び出しパス**: 1つの明確な経路
- **ゼロコスト抽象化**: オーバーヘッドなし
- **ユーザーコード変更なし**: 構文・API完全互換 ⭐

### ✅ 機能性

- **完全リフレクション**: JSON型情報で全情報取得
- **能力ベース設計**: プラグイン境界での最小権限強制 ⭐
- **効果トレース**: 副作用の可視化・デバッグ支援 ⭐
- **契約検証**: 事前/事後条件で境界安全性 ⭐
- **Component Model**: WIT互換でエコシステム統合 ⭐
- **ツール統合**: IDE・デバッガー・CLI完全対応

⭐ = ChatGPT Pro提案による強化

---

## 📚 関連ドキュメント

- **現行版**: [nyash_abi_v2.md](nyash_abi_v2.md) - 現在の実装
- **Phase 12**: [unified-typebox-abi.md](../../development/roadmap/phases/phase-12/unified-typebox-abi.md) - 青写真
- **ChatGPT Pro提案**: 6アーキタイプ（Effect & Capability / Deterministic / Verified / Differentiable / Component Model / Metaobject）

---

**この設計は「とにかくシンプルによせていきたい」哲学 + ChatGPT Pro「プラグイン境界強化」戦略を完全統合します** 🎯

**段階導入計画**:
- **Phase A（3-6ヶ月）**: Effect & Capability + 契約 + Component Model（高ROI）
- **Phase B（6-12ヶ月）**: Deterministic + Metaobject（追加機能）
- **Phase C（12ヶ月以降）**: Differentiable（別ABI拡張）

すべて**後方互換**で段階導入可能！ 🚀