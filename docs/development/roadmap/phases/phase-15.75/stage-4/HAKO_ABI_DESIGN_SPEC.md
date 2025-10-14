# Hako ABI Design Specification - Stage 4

**Phase 15.75 Stage 4**: Parser の Hako ABI (プラグインシステム) 統合設計

**作成日**: 2025-10-14
**ステータス**: 設計完了、実装待ち

---

## 🎯 **Hako ABI 層の位置づけ**

### **3層アーキテクチャにおける役割**

```
┌─────────────────────────────────────────┐
│ Hakorune コード (.hako)                  │
│ - BoxCall("ParserBox", "parse")          │
│ - 通常のBox呼び出しと同じ                │
└─────────────────────────────────────────┘
           ↓ VM Runtime (execute_boxcall)
┌─────────────────────────────────────────┐
│ TypeRegistry                             │
│ - "ParserBox" を解決                     │
│ - typebox->resolve("parse") → method_id  │
│ - 引数を NyValue[] に変換                │
└─────────────────────────────────────────┘
           ↓ invoke_id(method_id, NyValue[], argc)
┌─────────────────────────────────────────┐
│ 【Hako ABI層 - このドキュメント】       │  ⭐
│ NyashTypeBox 実装 (C言語)               │
│ - ParserTypeBox 構造体                   │
│ - invoke_id 実装（method_id判定）       │
│ - NyValue ↔ C型 変換（TLV）             │
│ (100-200行、C言語)                      │
└─────────────────────────────────────────┘
           ↓ C関数呼び出し
┌─────────────────────────────────────────┐
│ C ABI層（C_ABI_DESIGN_SPEC.md参照）     │
│ - parse_source_dual()                   │
│ - ParseResult 構造体                     │
│ (100-200行、C言語)                      │
└─────────────────────────────────────────┘
           ↓ 内部で呼び出し
┌─────────────────────────────────────────┐
│ Rust Parser（既存、縮小対象）           │
└─────────────────────────────────────────┘
```

### **Hako ABI層の責務**

| 責務 | 詳細 | 実装 |
|------|------|------|
| **TypeBox登録** | ParserBox を TypeRegistry に登録 | NyashTypeBox構造体 |
| **メソッド解決** | "parse" → method_id 変換 | resolve関数ポインタ |
| **高速ディスパッチ** | method_id でメソッド呼び出し | invoke_id関数ポインタ |
| **TLV変換** | NyValue ↔ C型 変換 | エンコード/デコード関数 |
| **C ABI呼び出し** | parse_source_dual() を呼び出し | invoke_id内部実装 |

---

## 📚 **既存ドキュメント（必読！）**

Stage 4 実装前に以下を参照してください：

### **Hako ABI コアドキュメント**
- 📄 [hako-abi-collections.md](../../../../reference/abi/hako-abi-collections.md) - Hako ABI 概要（StringBox/ArrayBox/MapBox例）
- 📄 [ABI_INDEX.md](../../../../reference/abi/ABI_INDEX.md) - ABI 統合インデックス
- 📄 [NYASH_ABI_MIN_CORE.md](../../../../reference/abi/NYASH_ABI_MIN_CORE.md) - 最小コアABI仕様
- 📄 [typebox-api-reference.md](../../../roadmap/phases/phase-12/specs/typebox-api-reference.md) - TypeBox API完全リファレンス

### **Phase 12 プラグインシステム**
- 📄 [Phase 12 README](../../../roadmap/phases/phase-12/README.md) - TypeBox統一設計
- 📄 [unified-typebox-abi.md](../../../roadmap/phases/phase-12/unified-typebox-abi.md) - 統一ABI実装計画

---

## 🔧 **NyashTypeBox 構造体設計**

### **基本構造体（Phase 12 仕様ベース）**

```c
// src/parser_harness/parser_typebox.h

#include <stdint.h>

// NyValue: 16B タグ付きユニオン
typedef struct {
    uint32_t tag;        // 型タグ（1=Bool, 2=I32, 3=I64, 6=String, etc.）
    uint32_t reserved;
    union {
        int64_t  i64;
        double   f64;
        void*    ptr;
        uint64_t bits;
    } payload;
} NyValue;

// NyResult: 戻り値 + エラー
typedef struct {
    int32_t status;           // 0=成功、非0=エラー
    NyValue value;            // 戻り値
    const char* error_msg;    // エラーメッセージ
} NyResult;

// ParserTypeBox: Hako ABI実装
typedef struct {
    uint32_t abi_tag;         // 0x54594258 ('TYBX')
    uint16_t version;         // 1
    uint16_t struct_size;     // sizeof(NyashTypeBox)
    const char* name;         // "ParserBox"

    // メソッド解決
    uint32_t (*resolve)(const char* method_name);

    // 高速ディスパッチ
    int32_t (*invoke_id)(
        uint32_t type_id,
        uint32_t method_id,
        uint32_t instance_id,
        const uint8_t* args_tlv,
        size_t args_len,
        uint8_t* result_tlv,
        size_t* result_len
    );

    // 互換性
    uint64_t capabilities;
    void* reserved[4];
} NyashTypeBox;
```

---

## 🔨 **実装例（C言語）**

### **ファイル構成**

```
src/parser_harness/
├── parser_typebox.h         # NyashTypeBox定義
├── parser_typebox.c         # NyashTypeBox実装
├── tlv_codec.h              # TLVエンコード/デコード
├── tlv_codec.c              # TLV実装
└── registration.c           # TypeRegistry登録
```

### **1. メソッド解決（resolve）**

```c
// src/parser_harness/parser_typebox.c

#include "parser_typebox.h"
#include <string.h>

// メソッドID定義
#define METHOD_PARSE_DUAL   1
#define METHOD_GET_VERSION  2

// メソッド名 → ID 変換
uint32_t parser_resolve(const char* method_name) {
    if (strcmp(method_name, "parse_dual") == 0) {
        return METHOD_PARSE_DUAL;
    }
    if (strcmp(method_name, "get_version") == 0) {
        return METHOD_GET_VERSION;
    }
    return 0;  // 未知のメソッド
}
```

### **2. 高速ディスパッチ（invoke_id）**

```c
// src/parser_harness/parser_typebox.c

#include "tlv_codec.h"
#include "../c_abi/parser_c_abi.h"  // parse_source_dual()

int32_t parser_invoke_id(
    uint32_t type_id,
    uint32_t method_id,
    uint32_t instance_id,
    const uint8_t* args_tlv,
    size_t args_len,
    uint8_t* result_tlv,
    size_t* result_len
) {
    switch (method_id) {
        case METHOD_PARSE_DUAL: {
            // TLVデコード：args[0] = source (String), args[1] = mode (I32)
            const char* source = NULL;
            int32_t mode = 0;

            if (!tlv_decode_string(args_tlv, args_len, 0, &source)) {
                return -1;  // デコードエラー
            }
            if (!tlv_decode_i32(args_tlv, args_len, 1, &mode)) {
                return -1;
            }

            // C ABI層呼び出し
            ParseResult c_result = {0};
            int ret = parse_source_dual(source, (ParserMode)mode, &c_result);

            if (ret != 0) {
                // エラー結果をTLVエンコード
                tlv_encode_error(result_tlv, result_len, c_result.error_msg);
                free_parse_result(&c_result);
                return ret;
            }

            // 成功結果をTLVエンコード（MapBox形式）
            TlvEncoder enc;
            tlv_encoder_init(&enc, result_tlv, *result_len);

            tlv_encoder_start_map(&enc);
            tlv_encoder_add_string(&enc, "version", c_result.version);
            tlv_encoder_add_string(&enc, "kind", c_result.kind);
            tlv_encoder_add_i32(&enc, "stmt_count", c_result.stmt_count);
            tlv_encoder_add_bool(&enc, "success", c_result.success);
            tlv_encoder_end_map(&enc);

            *result_len = tlv_encoder_size(&enc);

            free_parse_result(&c_result);
            return 0;
        }

        case METHOD_GET_VERSION: {
            // バージョン文字列を返す
            tlv_encode_string(result_tlv, result_len, "0.1.0");
            return 0;
        }

        default:
            return -1;  // 未知のメソッド
    }
}
```

### **3. TypeBox構造体初期化**

```c
// src/parser_harness/parser_typebox.c

NyashTypeBox g_parser_typebox = {
    .abi_tag = 0x54594258,  // 'TYBX'
    .version = 1,
    .struct_size = sizeof(NyashTypeBox),
    .name = "ParserBox",
    .resolve = parser_resolve,
    .invoke_id = parser_invoke_id,
    .capabilities = 0,  // 機能フラグ（必要に応じて設定）
    .reserved = {0}
};
```

### **4. TypeRegistry 登録**

```c
// src/parser_harness/registration.c

#include "parser_typebox.h"

// Rust側のFFI関数
extern void nyash_runtime_register_typebox(NyashTypeBox* typebox);

// 初期化時に呼び出される
void register_parser_typebox(void) {
    nyash_runtime_register_typebox(&g_parser_typebox);
}
```

---

## 📦 **TLV エンコード/デコード**

### **TLVフォーマット**

Hako ABIはTLV (Tag-Length-Value) エンコーディングを使用：

```
TLV形式:
┌────────┬────────┬─────────────┐
│ Tag(1) │ Len(4) │ Value(Len)  │
└────────┴────────┴─────────────┘

Tag定義:
1  = Bool (1 byte)
2  = I32 (4 bytes)
3  = I64 (8 bytes)
5  = F64 (8 bytes)
6  = String (UTF-8, null終端不要)
7  = String (null終端あり)
8  = PluginHandle (type_id + instance_id)
9  = HostHandle (u64)
```

### **デコード例**

```c
// src/parser_harness/tlv_codec.c

#include "tlv_codec.h"
#include <string.h>

bool tlv_decode_string(
    const uint8_t* tlv,
    size_t tlv_len,
    int arg_index,
    const char** out_str
) {
    size_t offset = 0;
    int current_index = 0;

    while (offset < tlv_len) {
        uint8_t tag = tlv[offset++];
        uint32_t len = *(uint32_t*)(tlv + offset);
        offset += 4;

        if (current_index == arg_index) {
            if (tag != 6 && tag != 7) {
                return false;  // 型不一致
            }

            // 文字列をコピー（nullターミネート）
            char* str = malloc(len + 1);
            memcpy(str, tlv + offset, len);
            str[len] = '\0';
            *out_str = str;
            return true;
        }

        offset += len;
        current_index++;
    }

    return false;  // インデックス範囲外
}

bool tlv_decode_i32(
    const uint8_t* tlv,
    size_t tlv_len,
    int arg_index,
    int32_t* out_i32
) {
    size_t offset = 0;
    int current_index = 0;

    while (offset < tlv_len) {
        uint8_t tag = tlv[offset++];
        uint32_t len = *(uint32_t*)(tlv + offset);
        offset += 4;

        if (current_index == arg_index) {
            if (tag != 2) {
                return false;  // 型不一致
            }

            *out_i32 = *(int32_t*)(tlv + offset);
            return true;
        }

        offset += len;
        current_index++;
    }

    return false;
}
```

### **エンコード例**

```c
// src/parser_harness/tlv_codec.c

typedef struct {
    uint8_t* buffer;
    size_t capacity;
    size_t offset;
} TlvEncoder;

void tlv_encoder_init(TlvEncoder* enc, uint8_t* buf, size_t cap) {
    enc->buffer = buf;
    enc->capacity = cap;
    enc->offset = 0;
}

void tlv_encoder_add_string(TlvEncoder* enc, const char* key, const char* value) {
    size_t len = strlen(value);

    // Tag (6 = String)
    enc->buffer[enc->offset++] = 6;

    // Length (4 bytes)
    *(uint32_t*)(enc->buffer + enc->offset) = (uint32_t)len;
    enc->offset += 4;

    // Value
    memcpy(enc->buffer + enc->offset, value, len);
    enc->offset += len;
}

void tlv_encoder_add_i32(TlvEncoder* enc, const char* key, int32_t value) {
    // Tag (2 = I32)
    enc->buffer[enc->offset++] = 2;

    // Length
    *(uint32_t*)(enc->buffer + enc->offset) = 4;
    enc->offset += 4;

    // Value
    *(int32_t*)(enc->buffer + enc->offset) = value;
    enc->offset += 4;
}

size_t tlv_encoder_size(TlvEncoder* enc) {
    return enc->offset;
}
```

---

## 🔗 **Rust側との統合**

### **TypeRegistry登録（Rust側）**

```rust
// src/runtime/type_registry.rs

use std::ffi::CStr;
use std::os::raw::c_char;

#[repr(C)]
pub struct NyashTypeBoxFfi {
    pub abi_tag: u32,
    pub version: u16,
    pub struct_size: u16,
    pub name: *const c_char,
    pub resolve: Option<extern "C" fn(*const c_char) -> u32>,
    pub invoke_id: Option<extern "C" fn(
        u32, u32, u32,
        *const u8, usize,
        *mut u8, *mut usize
    ) -> i32>,
    pub capabilities: u64,
    pub reserved: [*mut u8; 4],
}

// FFI経由で登録
#[no_mangle]
pub extern "C" fn nyash_runtime_register_typebox(typebox: *const NyashTypeBoxFfi) {
    unsafe {
        let name = CStr::from_ptr((*typebox).name).to_str().unwrap();

        // TypeRegistryに登録
        TYPE_REGISTRY.lock().unwrap().register(name, typebox);
    }
}
```

### **execute_boxcall での呼び出し**

```rust
// src/backend/mir_interpreter/handlers/boxcall.rs

pub fn execute_boxcall(
    vm: &mut VmState,
    recv: BoxRef,
    method: &str,
    args: &[Value]
) -> Result<Value, RuntimeError> {
    // TypeRegistry から TypeBox 取得
    let typebox = TYPE_REGISTRY.lock().unwrap()
        .get(&recv.type_name)
        .ok_or(RuntimeError::UnknownType)?;

    // メソッド名 → method_id 変換
    let method_id = unsafe {
        (typebox.resolve.unwrap())(
            CString::new(method).unwrap().as_ptr()
        )
    };

    if method_id == 0 {
        return Err(RuntimeError::UnknownMethod);
    }

    // 引数を TLV エンコード
    let mut args_tlv = Vec::new();
    for arg in args {
        encode_value_to_tlv(arg, &mut args_tlv)?;
    }

    // invoke_id 呼び出し
    let mut result_tlv = vec![0u8; 4096];
    let mut result_len = result_tlv.len();

    let ret = unsafe {
        (typebox.invoke_id.unwrap())(
            recv.type_id,
            method_id,
            recv.instance_id,
            args_tlv.as_ptr(),
            args_tlv.len(),
            result_tlv.as_mut_ptr(),
            &mut result_len
        )
    };

    if ret != 0 {
        return Err(RuntimeError::InvokeFailed);
    }

    // 結果を TLV デコード
    decode_tlv_to_value(&result_tlv[..result_len])
}
```

---

## 🧪 **テスト戦略**

### **単体テスト（C側）**

```c
// src/parser_harness/tests/test_parser_typebox.c

#include <assert.h>
#include "parser_typebox.h"

void test_resolve() {
    uint32_t id = parser_resolve("parse_dual");
    assert(id == 1);

    id = parser_resolve("unknown_method");
    assert(id == 0);
}

void test_invoke_parse_dual() {
    // TLVエンコード：["static box Test {}", 0]
    uint8_t args_tlv[256];
    size_t args_len = 0;

    tlv_encode_string(&args_tlv[args_len], &args_len, "static box Test {}");
    tlv_encode_i32(&args_tlv[args_len], &args_len, 0);  // mode=rust

    // invoke_id 呼び出し
    uint8_t result_tlv[4096];
    size_t result_len = sizeof(result_tlv);

    int ret = parser_invoke_id(
        0, 1, 0,  // type_id, method_id=parse_dual, instance_id
        args_tlv, args_len,
        result_tlv, &result_len
    );

    assert(ret == 0);

    // 結果検証
    // （TLVデコードして version/kind/stmt_count を確認）
}

int main() {
    test_resolve();
    test_invoke_parse_dual();
    return 0;
}
```

### **統合テスト（Rust側）**

```rust
// tests/parser_typebox_integration.rs

#[test]
fn test_parser_typebox_registration() {
    register_parser_typebox();

    let registry = TYPE_REGISTRY.lock().unwrap();
    assert!(registry.get("ParserBox").is_some());
}

#[test]
fn test_boxcall_parse_dual() {
    let source = "static box Test { main() { return 42 } }";

    let result = execute_boxcall(
        &mut vm,
        BoxRef { type_name: "ParserBox", instance_id: 0 },
        "parse_dual",
        &[Value::String(source.to_string()), Value::Integer(0)]
    ).unwrap();

    // 結果検証
    assert!(result.is_map());
    let map = result.as_map().unwrap();
    assert_eq!(map.get("version"), Some(&Value::String("0.1".to_string())));
}
```

---

## 📁 **ファイル配置**

### **推奨ディレクトリ構造**

```
src/parser_harness/
├── parser_typebox.h         # NyashTypeBox定義
├── parser_typebox.c         # NyashTypeBox実装
├── tlv_codec.h              # TLVエンコード/デコード
├── tlv_codec.c              # TLV実装
├── registration.c           # TypeRegistry登録
└── tests/
    ├── test_parser_typebox.c
    └── test_tlv_codec.c

build.rs の追記:
- parser_typebox.c のコンパイル
- tlv_codec.c のコンパイル
- registration.c のコンパイル
```

---

## 🎯 **実装優先順位**

### **Phase-A (MVP、2日間)**

| 優先度 | 実装内容 | 実装量 | ファイル |
|--------|----------|--------|----------|
| **P0** | NyashTypeBox構造体定義 | 30行 | `parser_typebox.h` |
| **P0** | TLVデコード（String, I32） | 60行 | `tlv_codec.c` |
| **P0** | TLVエンコード（Map） | 40行 | `tlv_codec.c` |
| **P1** | parser_resolve 実装 | 20行 | `parser_typebox.c` |
| **P1** | parser_invoke_id 実装 | 80行 | `parser_typebox.c` |
| **P2** | TypeRegistry登録 | 30行 | `registration.c` |
| **P2** | Rust側統合 | 50行 | Rust側実装 |

**合計**: 約310行（予定200-300行内、少しオーバー）

### **Phase-B（Stage 5以降）**

- 完全なTLVエンコーダ（全型対応）
- エラーハンドリング強化
- パフォーマンス最適化

---

## 🔧 **環境変数**

### **SMOKES_PARSER_MODE**

（C_ABI_DESIGN_SPEC.md と同じ）

| 値 | 動作 | 用途 |
|----|------|------|
| **rust** (デフォルト) | Rust Parserのみ使用 | 通常実行 |
| **hako** | Hakorune Parserのみ使用 | Hakorune Parser単体テスト |
| **both** | 両方実行＋比較 | デュアル検証モード |

### **NYASH_ABI_TRACE**

| 値 | 動作 |
|----|------|
| **0** (デフォルト) | トレース無効 |
| **1** | invoke_id 呼び出しトレース |

```bash
# トレース有効化
NYASH_ABI_TRACE=1 ./target/release/hako test.nyash

# 出力例:
# [ABI] invoke_id: type=ParserBox, method=parse_dual, args=[String, I32]
# [ABI] invoke_id: result=Map(version=0.1, kind=Program, stmt_count=1)
```

---

## 🚨 **エラーハンドリング**

### **エラー種類と対応**

| エラー種類 | 対応 | 実装 |
|-----------|------|------|
| **TLVデコードエラー** | -1 を返す | tlv_decode_* |
| **未知のメソッド** | method_id=0 | parser_resolve |
| **C ABI呼び出し失敗** | エラーTLVエンコード | parser_invoke_id |
| **型不一致** | -3 を返す | TLVデコード |

---

## 🔄 **Rollback対応**

### **Hako ABI層のRollback**

**Level 1: 機能無効化（1分）**
```bash
# TypeRegistry登録をスキップ
# → ParserBox が未登録 → Rustフォールバック
export NYASH_DISABLE_PARSER_PLUGIN=1
cargo build --release
```

**Level 2: ファイル削除（5分）**
```bash
rm -rf src/parser_harness/
# build.rs から parser_harness エントリ削除
cargo build --release
```

**Level 3: Full Rollback（C ABI含む、30分）**
（C_ABI_DESIGN_SPEC.md と同じ）

---

## ✅ **受け入れ基準（DoD）**

### **Hako ABI層完了条件**

- [ ] NyashTypeBox構造体定義完了
- [ ] parser_resolve 実装完了
- [ ] parser_invoke_id 実装完了（parse_dual のみ）
- [ ] TLVエンコード/デコード実装完了（String, I32, Map）
- [ ] TypeRegistry登録成功
- [ ] Rust側統合完了（execute_boxcall 経由呼び出し）
- [ ] 単体テスト3件以上PASS
- [ ] 統合テスト3件以上PASS
- [ ] quick-selfhost 170/185 PASS維持（最重要！）
- [ ] コード行数 200-300行以内

---

## 📚 **関連ドキュメント**

### **Stage 4関連**
- [C_ABI_DESIGN_SPEC.md](./C_ABI_DESIGN_SPEC.md) - C ABI層設計（下層）
- [TECHNICAL_REQUIREMENTS.md](./TECHNICAL_REQUIREMENTS.md) - 技術要件
- [SCHEDULE.md](./SCHEDULE.md) - 実装スケジュール
- [RISK_ANALYSIS.md](./RISK_ANALYSIS.md) - リスク分析

### **Hako ABI / TypeBox 仕様**
- [hako-abi-collections.md](../../../../reference/abi/hako-abi-collections.md) ⭐必読
- [ABI_INDEX.md](../../../../reference/abi/ABI_INDEX.md) - ABI統合インデックス
- [NYASH_ABI_MIN_CORE.md](../../../../reference/abi/NYASH_ABI_MIN_CORE.md) - 最小コアABI
- [typebox-api-reference.md](../../../roadmap/phases/phase-12/specs/typebox-api-reference.md) - TypeBox API完全リファレンス

### **Phase 12 プラグインシステム**
- [Phase 12 README](../../../roadmap/phases/phase-12/README.md) - TypeBox統一設計
- [unified-typebox-abi.md](../../../roadmap/phases/phase-12/unified-typebox-abi.md) - 統一ABI実装計画

---

## 💡 **実装のヒント**

### **開発順序（推奨）**

1. **NyashTypeBox 構造体定義**（parser_typebox.h）
2. **TLVデコード実装**（tlv_codec.c）- String, I32のみ
3. **parser_resolve 実装**（parser_typebox.c）
4. **parser_invoke_id 実装**（parser_typebox.c）- parse_dual のみ
5. **TLVエンコード実装**（tlv_codec.c）- Map のみ
6. **TypeRegistry登録**（registration.c）
7. **Rust側統合**（execute_boxcall 修正）

### **デバッグTips**

```bash
# TLVエンコード/デコード確認
NYASH_ABI_TRACE=1 ./target/release/hako test.nyash

# invoke_id トレース
NYASH_DEBUG_INVOKE=1 ./target/release/hako test.nyash
```

### **最初のテスト**

**最小限の成功ケース**:
```bash
# C側単体テスト
gcc -o test_parser_typebox \
    src/parser_harness/tests/test_parser_typebox.c \
    src/parser_harness/parser_typebox.c \
    src/parser_harness/tlv_codec.c \
    -I src/parser_harness
./test_parser_typebox
# 期待: All tests passed
```

---

**作成者**: Claude (Sonnet 4.5)
**作成日**: 2025-10-14
**最終更新**: 2025-10-14（修正版）
**ステータス**: 設計完了、実装待ち
