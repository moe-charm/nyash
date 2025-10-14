# C ABI層 設計仕様書

**Phase 15.75 Phase 4: Dual Parser Harness — C ABI Layer Design**

**日付**: 2025-10-16
**ステータス**: 設計完了・レビュー待ち
**関連**: [TODO.md](./TODO.md) Phase 4, [STRATEGY.md](./STRATEGY.md)

---

## 目次

1. [責務定義](#1-責務定義)
2. [データ構造](#2-データ構造)
3. [関数インターフェース](#3-関数インターフェース)
4. [メモリ管理](#4-メモリ管理)
5. [エラーハンドリング](#5-エラーハンドリング)
6. [Cargo統合](#6-cargo統合)
7. [サンプルコード](#7-サンプルコード)
8. [想定される問題と対策](#8-想定される問題と対策)
9. [段階導入計画](#9-段階導入計画)

---

## 1. 責務定義

### するべきこと

**C ABI層（parser_harness.c）の責務**:
- 薄いABI境界の提供（100-200行）
- Rust Parser と Hakorune Parser の呼び出し切り替え
- 基本的なデータ変換（Rust型 ↔ C型 ↔ Hakorune型）
- メモリ所有権の明確化（誰が確保・解放するか）
- NULL安全性の保証

**役割の明確化**:
```
┌─────────────────────────────────────────────┐
│            Rust Runner (CLI)                │
│  - ファイル読み込み                         │
│  - Backend選択（VM/LLVM/nyvm）              │
│  - 環境変数読み取り                         │
└──────────────┬──────────────────────────────┘
               │
               v
┌─────────────────────────────────────────────┐
│      C ABI Layer (parser_harness.c)         │ ← 薄い層（100-200行）
│  - ParserMode選択（rust/hako/both）         │
│  - ParseResult変換                          │
│  - メモリ管理の境界                         │
└──┬───────────────────────────────────────┬───┘
   │                                       │
   v                                       v
┌──────────────────┐             ┌──────────────────┐
│  Rust Parser     │             │ Hakorune Parser  │
│  (src/front/)    │             │ (.hako impl)     │
└──────────────────┘             └──────────────────┘
```

### するべきでないこと

**C ABI層がやってはいけないこと**:
- ❌ 複雑なパース処理の実装
- ❌ AST変換ロジック
- ❌ 最適化処理
- ❌ エラー診断メッセージの生成（エラー伝達のみ）
- ❌ ファイルI/O（呼び出し元の責務）
- ❌ 環境変数の直接参照（呼び出し元が渡す）

**境界の明確化**:
- C ABI層は「橋渡し」に徹する
- ビジネスロジックはRust/Hakorune側に置く
- エラーハンドリングは「成功/失敗」のみ伝達し、詳細は呼び出し元が処理

---

## 2. データ構造

### 2.1 ParserMode

```c
/**
 * パーサーモード選択
 * 環境変数 SMOKES_PARSER_MODE で制御
 */
typedef enum {
    PARSER_MODE_RUST = 0,   // Rust実装パーサー（既定）
    PARSER_MODE_HAKO = 1,   // Hakorune自己ホストパーサー
    PARSER_MODE_BOTH = 2    // 両方実行して比較
} ParserMode;
```

### 2.2 ParseResult

```c
/**
 * パース結果構造体
 * 所有権: C側が確保、呼び出し元が解放義務
 */
typedef struct {
    // バージョン情報（将来の互換性確保）
    uint32_t version;           // ABI version（現在は1）

    // パース結果の種別
    char kind[32];              // "Program", "Module", "Expression" など

    // 統計情報（最小比較用）
    uint32_t stmt_count;        // ステートメント数
    uint32_t expr_count;        // 式の数（オプション・未使用なら0）

    // 成功/失敗フラグ
    uint8_t success;            // 0=失敗, 1=成功

    // エラーメッセージ（失敗時のみ）
    char* error_msg;            // NULL=エラーなし、非NULL=エラーメッセージ

    // AST JSON（オプション・デバッグ用）
    char* ast_json;             // NULL=なし、非NULL=JSON文字列
    size_t ast_json_len;        // ast_jsonの長さ（0=なし）

    // 内部使用（拡張用予約領域）
    void* _reserved[4];
} ParseResult;
```

**設計のポイント**:
- `version`: 将来のABI変更に対応（現在は1固定）
- `kind`: 最小比較対象（"Program"など）
- `stmt_count`: 最小同等性検証の主要指標
- `success`: 明示的な成功/失敗フラグ
- `error_msg`: 動的確保、NULL可（成功時はNULL）
- `ast_json`: デバッグ用、本番では不要（NULL可）
- `_reserved`: 将来の拡張用（現在は未使用）

### 2.3 ParseStats（補助構造体）

```c
/**
 * 詳細統計情報（オプション）
 * 必要に応じてParseResultに追加可能
 */
typedef struct {
    uint32_t functions;
    uint32_t boxes;
    uint32_t static_boxes;
    uint32_t global_vars;
} ParseStats;
```

---

## 3. 関数インターフェース

### 3.1 parse_source_dual()

```c
/**
 * メイン関数: ソースコードをパースし、結果を返す
 *
 * @param source       パース対象のソースコード（NULL終端必須）
 * @param mode         パーサーモード（RUST/HAKO/BOTH）
 * @param out_result   結果格納先（呼び出し元が用意、NULLチェック必須）
 * @return             0=成功, 負数=エラー
 *
 * エラーコード:
 *   0: 成功
 *  -1: 引数エラー（NULL引数など）
 *  -2: Rustパーサー失敗
 *  -3: Hakoruneパーサー失敗
 *  -4: 両方比較モードで不一致検出
 *  -5: メモリ不足
 */
int parse_source_dual(
    const char* source,
    ParserMode mode,
    ParseResult* out_result
);
```

**使用例**:
```c
ParseResult result = {0};  // ゼロ初期化
int ret = parse_source_dual(source_code, PARSER_MODE_RUST, &result);
if (ret != 0) {
    fprintf(stderr, "Parse failed: %s\n",
            result.error_msg ? result.error_msg : "unknown error");
    free_parse_result(&result);
    return ret;
}
// 成功時の処理
printf("Parsed successfully: %d statements\n", result.stmt_count);
free_parse_result(&result);
```

### 3.2 free_parse_result()

```c
/**
 * ParseResult のメモリ解放
 *
 * @param result  解放対象（NULLチェック内部で実施）
 *
 * 注意:
 * - result自体は呼び出し元が確保したものなので解放しない
 * - result内の動的確保メモリ（error_msg, ast_json）のみ解放
 * - 二重解放を防ぐため、解放後はポインタをNULLにセット
 */
void free_parse_result(ParseResult* result);
```

**実装の安全性**:
```c
void free_parse_result(ParseResult* result) {
    if (!result) return;

    if (result->error_msg) {
        free(result->error_msg);
        result->error_msg = NULL;
    }

    if (result->ast_json) {
        free(result->ast_json);
        result->ast_json = NULL;
    }

    // フラグのリセット（オプション）
    result->success = 0;
    result->stmt_count = 0;
}
```

### 3.3 parse_source_rust()（内部関数）

```c
/**
 * Rust実装パーサーを呼び出す（内部使用）
 *
 * @param source       ソースコード
 * @param out_result   結果格納先
 * @return             0=成功, 負数=エラー
 */
static int parse_source_rust(
    const char* source,
    ParseResult* out_result
);
```

### 3.4 parse_source_hako()（内部関数）

```c
/**
 * Hakorune実装パーサーを呼び出す（内部使用）
 *
 * @param source       ソースコード
 * @param out_result   結果格納先
 * @return             0=成功, 負数=エラー
 */
static int parse_source_hako(
    const char* source,
    ParseResult* out_result
);
```

### 3.5 compare_results()（内部関数）

```c
/**
 * 2つのParseResultを比較（BOTH モード用）
 *
 * @param rust_result  Rust実装の結果
 * @param hako_result  Hakorune実装の結果
 * @return             0=一致, -4=不一致
 *
 * 比較対象:
 * - version（必須一致）
 * - kind（文字列比較）
 * - stmt_count（数値比較）
 */
static int compare_results(
    const ParseResult* rust_result,
    const ParseResult* hako_result
);
```

---

## 4. メモリ管理

### 4.1 確保戦略

**原則: 誰が確保するか**

| データ | 確保者 | 理由 |
|--------|--------|------|
| `ParseResult` 本体 | 呼び出し元（Rust） | スタック確保で高速化 |
| `error_msg` | C ABI層 | 動的サイズ、失敗時のみ |
| `ast_json` | C ABI層 | 動的サイズ、デバッグ時のみ |
| `source` | 呼び出し元（Rust） | 入力データの所有者 |

**具体例**:
```c
// 呼び出し元（Rust側）
ParseResult result = {0};  // スタック確保
parse_source_dual(source, mode, &result);
// ... 使用 ...
free_parse_result(&result);  // 内部の動的メモリのみ解放
```

### 4.2 解放戦略

**解放責務マトリックス**:

| データ | 解放者 | タイミング |
|--------|--------|-----------|
| `ParseResult` 本体 | 呼び出し元 | スコープ終了時（自動） |
| `error_msg` | `free_parse_result()` | 明示的呼び出し |
| `ast_json` | `free_parse_result()` | 明示的呼び出し |
| `source` | 呼び出し元 | 元の所有者が管理 |

**安全性の確保**:
```c
void free_parse_result(ParseResult* result) {
    if (!result) return;  // NULLチェック

    // 二重解放防止: free後にNULLセット
    if (result->error_msg) {
        free(result->error_msg);
        result->error_msg = NULL;
    }

    if (result->ast_json) {
        free(result->ast_json);
        result->ast_json = NULL;
    }
}
```

### 4.3 所有権ルール

**文字列の所有権**:

```c
// ❌ 悪い例: const char* を返す（ダングリングポインタの危険）
const char* get_error_msg() {
    static char buf[256];
    sprintf(buf, "Error: %s", ...);
    return buf;  // 危険: 静的バッファの再利用
}

// ✅ 良い例: char* を返す（所有権を移譲）
char* allocate_error_msg(const char* msg) {
    return strdup(msg);  // 呼び出し元がfree責務を持つ
}
```

**ルール**:
1. `const char*` = 読み取り専用、所有権なし、解放禁止
2. `char*` = 書き込み可能、所有権あり、解放必須
3. 関数が `char*` を返す場合、必ずドキュメントで解放責務を明記

### 4.4 エラー時のメモリリーク防止

**失敗時の後始末**:
```c
int parse_source_dual(const char* source, ParserMode mode, ParseResult* out_result) {
    // 引数チェック
    if (!source || !out_result) {
        return -1;
    }

    // 初期化
    memset(out_result, 0, sizeof(ParseResult));
    out_result->version = 1;

    // パース実行
    int ret = 0;
    if (mode == PARSER_MODE_RUST) {
        ret = parse_source_rust(source, out_result);
    } else if (mode == PARSER_MODE_HAKO) {
        ret = parse_source_hako(source, out_result);
    } else if (mode == PARSER_MODE_BOTH) {
        ParseResult rust_res = {0}, hako_res = {0};

        ret = parse_source_rust(source, &rust_res);
        if (ret != 0) {
            free_parse_result(&rust_res);  // 失敗時も解放
            return ret;
        }

        ret = parse_source_hako(source, &hako_res);
        if (ret != 0) {
            free_parse_result(&rust_res);  // ここで解放
            free_parse_result(&hako_res);
            return ret;
        }

        ret = compare_results(&rust_res, &hako_res);

        // 結果をコピー（Rust版を優先）
        memcpy(out_result, &rust_res, sizeof(ParseResult));

        // hako_res のみ解放（rust_resは移譲済み）
        free_parse_result(&hako_res);

        if (ret != 0) {
            // 不一致時のエラーメッセージ設定
            out_result->error_msg = strdup("Parser mismatch: rust vs hako");
        }
    }

    return ret;
}
```

---

## 5. エラーハンドリング

### 5.1 成功/失敗の表現方法

**戻り値ベース**:
```c
// 関数の戻り値
// 0       = 成功
// 負数    = エラーコード
// 正数    = 使用しない（将来の拡張用）

// エラーコード定義
#define PARSE_SUCCESS           0
#define PARSE_ERR_NULL_ARG     -1
#define PARSE_ERR_RUST_FAILED  -2
#define PARSE_ERR_HAKO_FAILED  -3
#define PARSE_ERR_MISMATCH     -4
#define PARSE_ERR_NO_MEMORY    -5
```

**構造体フラグベース**:
```c
// ParseResult.success フィールド
result.success = 1;  // 成功
result.success = 0;  // 失敗
```

**両方を併用する理由**:
- 戻り値: 即座のエラーハンドリング（制御フロー）
- フラグ: 結果オブジェクト内の状態保持（ログ・デバッグ）

### 5.2 エラーメッセージの伝達方法

**動的確保 + NULL可**:
```c
typedef struct {
    uint8_t success;
    char* error_msg;  // NULL = エラーなし
} ParseResult;

// エラー設定ヘルパー
static void set_error(ParseResult* result, const char* msg) {
    result->success = 0;
    result->error_msg = strdup(msg);  // 動的確保
}

// 使用例
if (parse_failed) {
    set_error(out_result, "Syntax error at line 10");
    return PARSE_ERR_RUST_FAILED;
}
```

**エラーメッセージの生成**:
```c
// Rustパーサーからエラーメッセージを取得（外部関数）
extern const char* rust_parser_last_error();

static int parse_source_rust(const char* source, ParseResult* out_result) {
    // Rustパーサー呼び出し
    if (!rust_parse_call(source)) {
        const char* err = rust_parser_last_error();
        out_result->error_msg = strdup(err ? err : "Unknown error");
        out_result->success = 0;
        return PARSE_ERR_RUST_FAILED;
    }

    out_result->success = 1;
    return PARSE_SUCCESS;
}
```

### 5.3 NULL安全性の確保

**防御的プログラミング**:
```c
int parse_source_dual(const char* source, ParserMode mode, ParseResult* out_result) {
    // 必須引数チェック
    if (!source) {
        fprintf(stderr, "parse_source_dual: source is NULL\n");
        return PARSE_ERR_NULL_ARG;
    }

    if (!out_result) {
        fprintf(stderr, "parse_source_dual: out_result is NULL\n");
        return PARSE_ERR_NULL_ARG;
    }

    // ゼロ初期化（重要: ダングリングポインタ防止）
    memset(out_result, 0, sizeof(ParseResult));
    out_result->version = 1;

    // ... 処理 ...
}

void free_parse_result(ParseResult* result) {
    if (!result) {
        return;  // 早期リターン（エラー出力不要）
    }

    // NULL チェック後に解放
    if (result->error_msg) {
        free(result->error_msg);
        result->error_msg = NULL;
    }

    if (result->ast_json) {
        free(result->ast_json);
        result->ast_json = NULL;
    }
}
```

**チェックリスト**:
- ✅ すべてのポインタ引数にNULLチェック
- ✅ 構造体の初期化（memset or ={0}）
- ✅ free前のNULLチェック
- ✅ free後のNULL代入（二重解放防止）
- ✅ 外部関数呼び出しの戻り値チェック

---

## 6. Cargo統合

### 6.1 build.rs での cc crate 使用

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // C ABI層のビルド
    cc::Build::new()
        .file("src/parser_harness/parser_harness.c")
        .include("src/parser_harness")
        .warnings(true)
        .extra_warnings(true)
        .flag_if_supported("-std=c11")
        .flag_if_supported("-pedantic")
        .compile("parser_harness");

    // リンク設定
    println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.c");
    println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.h");

    // Rust側でのヘッダーパス設定
    let include_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("parser_harness");
    println!("cargo:include={}", include_path.display());
}
```

### 6.2 Cargo.toml 設定

```toml
[build-dependencies]
cc = "1.0"

[dependencies]
# C ABIとのバインディング用
libc = "0.2"

[features]
# パーサーハーネスの有効化（段階導入用）
parser-harness = []
```

### 6.3 ヘッダファイルの配置

**推奨ディレクトリ構造**:
```
src/
├── parser_harness/
│   ├── parser_harness.h      # ヘッダー
│   ├── parser_harness.c      # 実装
│   └── mod.rs                # Rust binding
├── front/
│   ├── parser.rs             # Rust parser
│   └── ...
└── lib.rs
```

**インクルードパス設定**:
```c
// parser_harness.c
#include "parser_harness.h"  // ローカル
```

```rust
// mod.rs
use std::os::raw::{c_char, c_int};

#[link(name = "parser_harness", kind = "static")]
extern "C" {
    pub fn parse_source_dual(
        source: *const c_char,
        mode: c_int,
        out_result: *mut ParseResult,
    ) -> c_int;

    pub fn free_parse_result(result: *mut ParseResult);
}
```

---

## 7. サンプルコード

### 7.1 ヘッダファイル (parser_harness.h)

```c
#ifndef PARSER_HARNESS_H
#define PARSER_HARNESS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// パーサーモード
typedef enum {
    PARSER_MODE_RUST = 0,
    PARSER_MODE_HAKO = 1,
    PARSER_MODE_BOTH = 2
} ParserMode;

// パース結果
typedef struct {
    uint32_t version;
    char kind[32];
    uint32_t stmt_count;
    uint32_t expr_count;
    uint8_t success;
    char* error_msg;
    char* ast_json;
    size_t ast_json_len;
    void* _reserved[4];
} ParseResult;

// エラーコード
#define PARSE_SUCCESS           0
#define PARSE_ERR_NULL_ARG     -1
#define PARSE_ERR_RUST_FAILED  -2
#define PARSE_ERR_HAKO_FAILED  -3
#define PARSE_ERR_MISMATCH     -4
#define PARSE_ERR_NO_MEMORY    -5

// 公開API
int parse_source_dual(
    const char* source,
    ParserMode mode,
    ParseResult* out_result
);

void free_parse_result(ParseResult* result);

#ifdef __cplusplus
}
#endif

#endif // PARSER_HARNESS_H
```

### 7.2 実装ファイル (parser_harness.c) 最小実装

```c
#include "parser_harness.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>

// 外部関数宣言（Rust側で実装）
extern int rust_parser_parse(const char* source, uint32_t* out_stmt_count, char** out_kind);
extern const char* rust_parser_last_error(void);
extern int hako_parser_parse(const char* source, uint32_t* out_stmt_count, char** out_kind);
extern const char* hako_parser_last_error(void);

// 内部関数: Rustパーサー呼び出し
static int parse_source_rust(const char* source, ParseResult* out_result) {
    uint32_t stmt_count = 0;
    char* kind = NULL;

    int ret = rust_parser_parse(source, &stmt_count, &kind);
    if (ret != 0) {
        const char* err = rust_parser_last_error();
        out_result->error_msg = strdup(err ? err : "Rust parser failed");
        out_result->success = 0;
        return PARSE_ERR_RUST_FAILED;
    }

    out_result->stmt_count = stmt_count;
    strncpy(out_result->kind, kind ? kind : "Program", sizeof(out_result->kind) - 1);
    out_result->success = 1;

    if (kind) free(kind);
    return PARSE_SUCCESS;
}

// 内部関数: Hakoruneパーサー呼び出し
static int parse_source_hako(const char* source, ParseResult* out_result) {
    uint32_t stmt_count = 0;
    char* kind = NULL;

    int ret = hako_parser_parse(source, &stmt_count, &kind);
    if (ret != 0) {
        const char* err = hako_parser_last_error();
        out_result->error_msg = strdup(err ? err : "Hakorune parser failed");
        out_result->success = 0;
        return PARSE_ERR_HAKO_FAILED;
    }

    out_result->stmt_count = stmt_count;
    strncpy(out_result->kind, kind ? kind : "Program", sizeof(out_result->kind) - 1);
    out_result->success = 1;

    if (kind) free(kind);
    return PARSE_SUCCESS;
}

// 内部関数: 結果比較
static int compare_results(const ParseResult* rust_result, const ParseResult* hako_result) {
    if (rust_result->version != hako_result->version) {
        return PARSE_ERR_MISMATCH;
    }

    if (strcmp(rust_result->kind, hako_result->kind) != 0) {
        return PARSE_ERR_MISMATCH;
    }

    if (rust_result->stmt_count != hako_result->stmt_count) {
        return PARSE_ERR_MISMATCH;
    }

    return PARSE_SUCCESS;
}

// 公開API: メイン関数
int parse_source_dual(const char* source, ParserMode mode, ParseResult* out_result) {
    // 引数チェック
    if (!source) {
        fprintf(stderr, "parse_source_dual: source is NULL\n");
        return PARSE_ERR_NULL_ARG;
    }

    if (!out_result) {
        fprintf(stderr, "parse_source_dual: out_result is NULL\n");
        return PARSE_ERR_NULL_ARG;
    }

    // 初期化
    memset(out_result, 0, sizeof(ParseResult));
    out_result->version = 1;

    // モード別処理
    int ret = PARSE_SUCCESS;

    if (mode == PARSER_MODE_RUST) {
        ret = parse_source_rust(source, out_result);
    }
    else if (mode == PARSER_MODE_HAKO) {
        ret = parse_source_hako(source, out_result);
    }
    else if (mode == PARSER_MODE_BOTH) {
        ParseResult rust_res = {0}, hako_res = {0};

        ret = parse_source_rust(source, &rust_res);
        if (ret != PARSE_SUCCESS) {
            free_parse_result(&rust_res);
            return ret;
        }

        ret = parse_source_hako(source, &hako_res);
        if (ret != PARSE_SUCCESS) {
            free_parse_result(&rust_res);
            free_parse_result(&hako_res);
            return ret;
        }

        ret = compare_results(&rust_res, &hako_res);

        // 結果コピー（Rust優先）
        memcpy(out_result, &rust_res, sizeof(ParseResult));

        // hako_resのみ解放
        free_parse_result(&hako_res);

        if (ret != PARSE_SUCCESS) {
            out_result->error_msg = strdup("Parser mismatch: rust vs hako");
        }
    }
    else {
        fprintf(stderr, "parse_source_dual: invalid mode %d\n", mode);
        return PARSE_ERR_NULL_ARG;
    }

    return ret;
}

// 公開API: メモリ解放
void free_parse_result(ParseResult* result) {
    if (!result) {
        return;
    }

    if (result->error_msg) {
        free(result->error_msg);
        result->error_msg = NULL;
    }

    if (result->ast_json) {
        free(result->ast_json);
        result->ast_json = NULL;
    }

    result->success = 0;
    result->stmt_count = 0;
}
```

### 7.3 Rust Binding (mod.rs)

```rust
// src/parser_harness/mod.rs
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

#[repr(C)]
pub struct ParseResult {
    pub version: u32,
    pub kind: [u8; 32],
    pub stmt_count: u32,
    pub expr_count: u32,
    pub success: u8,
    pub error_msg: *mut c_char,
    pub ast_json: *mut c_char,
    pub ast_json_len: usize,
    pub _reserved: [*mut c_void; 4],
}

#[repr(i32)]
pub enum ParserMode {
    Rust = 0,
    Hako = 1,
    Both = 2,
}

extern "C" {
    fn parse_source_dual(
        source: *const c_char,
        mode: c_int,
        out_result: *mut ParseResult,
    ) -> c_int;

    fn free_parse_result(result: *mut ParseResult);
}

pub fn parse_with_mode(source: &str, mode: ParserMode) -> Result<ParseResultOwned, String> {
    let c_source = CString::new(source).map_err(|_| "Invalid source string")?;

    let mut result = ParseResult {
        version: 0,
        kind: [0; 32],
        stmt_count: 0,
        expr_count: 0,
        success: 0,
        error_msg: std::ptr::null_mut(),
        ast_json: std::ptr::null_mut(),
        ast_json_len: 0,
        _reserved: [std::ptr::null_mut(); 4],
    };

    let ret = unsafe {
        parse_source_dual(c_source.as_ptr(), mode as c_int, &mut result)
    };

    if ret != 0 || result.success == 0 {
        let err_msg = if !result.error_msg.is_null() {
            unsafe { CStr::from_ptr(result.error_msg).to_string_lossy().into_owned() }
        } else {
            format!("Parse failed with code {}", ret)
        };

        unsafe { free_parse_result(&mut result) };
        return Err(err_msg);
    }

    // 結果をRust型に変換
    let owned = ParseResultOwned {
        version: result.version,
        kind: extract_kind(&result.kind),
        stmt_count: result.stmt_count,
        success: result.success != 0,
    };

    unsafe { free_parse_result(&mut result) };
    Ok(owned)
}

fn extract_kind(kind: &[u8; 32]) -> String {
    let len = kind.iter().position(|&b| b == 0).unwrap_or(32);
    String::from_utf8_lossy(&kind[..len]).into_owned()
}

pub struct ParseResultOwned {
    pub version: u32,
    pub kind: String,
    pub stmt_count: u32,
    pub success: bool,
}
```

---

## 8. 想定される問題と対策

### 8.1 問題1: Cコンパイラの互換性

**問題**:
- GCC/Clang/MSVCで挙動が異なる可能性
- C標準ライブラリの差異（strdup の有無）

**対策**:
```c
// strdup が標準でない場合の互換実装
#ifndef _GNU_SOURCE
static char* my_strdup(const char* s) {
    if (!s) return NULL;
    size_t len = strlen(s) + 1;
    char* p = malloc(len);
    if (p) memcpy(p, s, len);
    return p;
}
#define strdup my_strdup
#endif
```

### 8.2 問題2: メモリ境界のバグ

**問題**:
- バッファオーバーフロー
- Use-after-free
- 二重解放

**対策**:
```c
// 固定長バッファの安全なコピー
strncpy(out_result->kind, kind, sizeof(out_result->kind) - 1);
out_result->kind[sizeof(out_result->kind) - 1] = '\0';  // NULL終端保証

// 解放後のNULLセット（二重解放防止）
if (result->error_msg) {
    free(result->error_msg);
    result->error_msg = NULL;  // 重要
}

// Valgrind での検証
// valgrind --leak-check=full --show-leak-kinds=all ./test_parser_harness
```

### 8.3 問題3: スレッド安全性

**問題**:
- 静的変数の競合
- グローバル状態の共有

**対策**:
```c
// ❌ 悪い例: 静的バッファ
static char error_buffer[256];

// ✅ 良い例: 呼び出し毎に確保
char* allocate_error(const char* msg) {
    return strdup(msg);
}

// または thread_local（C11以降）
_Thread_local char error_buffer[256];
```

### 8.4 問題4: エラー診断の詳細不足

**問題**:
- "Parse failed" だけでは原因不明

**対策**:
```c
// エラーコンテキストを含める
static void set_error_with_context(
    ParseResult* result,
    const char* phase,
    const char* detail
) {
    char buf[512];
    snprintf(buf, sizeof(buf), "[%s] %s", phase, detail);
    result->error_msg = strdup(buf);
    result->success = 0;
}

// 使用例
if (parse_failed) {
    set_error_with_context(out_result, "Rust Parser", "Syntax error at line 10:5");
    return PARSE_ERR_RUST_FAILED;
}
```

### 8.5 問題5: ABIの将来の変更

**問題**:
- 構造体拡張時の互換性破壊

**対策**:
```c
// バージョンチェック
if (out_result->version != 1) {
    fprintf(stderr, "Unsupported ABI version: %u\n", out_result->version);
    return PARSE_ERR_NULL_ARG;
}

// 予約領域の活用
typedef struct {
    uint32_t version;
    // ... 既存フィールド ...
    void* _reserved[4];  // 将来の拡張用
} ParseResult;

// 将来の拡張例
// version 2: _reserved[0] = new_field1
// version 3: _reserved[1] = new_field2
```

---

## 9. 段階導入計画

### Phase 4-A: 基盤実装（1-2日）

**タスク**:
1. `parser_harness.h/.c` 作成（最小実装）
2. `build.rs` にcc crate統合
3. Rust binding (`mod.rs`) 作成
4. 単体テスト作成（Cレベル）

**受け入れ基準**:
- `cargo build` が成功
- Valgrindでメモリリークなし
- 単体テストが全てPASS

### Phase 4-B: Rust Parser統合（1日）

**タスク**:
1. `parse_source_rust()` 実装
2. Rust側のエラー伝達機能追加
3. スモークテスト1本（PARSER_MODE_RUST）

**受け入れ基準**:
- 既存Rustパーサーと同じ結果
- quick プロファイルが緑維持

### Phase 4-C: Hakorune Parser統合（1-2日）

**タスク**:
1. `parse_source_hako()` 実装
2. Hakorune側のエラー伝達機能追加
3. スモークテスト1本（PARSER_MODE_HAKO）

**受け入れ基準**:
- 最小プログラムがパース成功
- エラー時のメッセージ伝達確認

### Phase 4-D: 比較モード実装（1日）

**タスク**:
1. `compare_results()` 実装
2. PARSER_MODE_BOTH の動作確認
3. スモークテスト3本追加

**受け入れ基準**:
- セミコロン受理/if-else/ブロック終端のテストがPASS
- 不一致時のエラー報告が明確

### Phase 4-E: ドキュメント・CI統合（0.5日）

**タスク**:
1. README更新
2. CI設定追加（Valgrind実行）
3. 環境変数ドキュメント追加

**受け入れ基準**:
- docs/guides/ に使い方ガイド
- CIで毎回Valgrindチェック

---

## まとめ

### 設計の核心原則

1. **薄い層**: 100-200行のC ABI層、ビジネスロジックはRust/Hakorune側
2. **明確な責務**: データ変換とモード切り替えのみ
3. **安全第一**: NULL安全性、メモリリーク防止、二重解放防止
4. **段階導入**: feature gateで既定OFF、dev環境で検証後に既定ON
5. **可逆性**: 問題発生時は即座に旧実装に戻せる

### 次のステップ

1. このドキュメントのレビュー（チーム/AI協調）
2. Phase 4-A着手（基盤実装）
3. Rustチームとの連携確認
4. スモークテスト策定

---

**レビュアー**: @claude @chatgpt
**最終更新**: 2025-10-16
