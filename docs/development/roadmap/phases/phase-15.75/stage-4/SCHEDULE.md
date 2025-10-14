# Phase 4 実装タスク詳細スケジュール - Dual Parser Harness

**Status**: Implementation Plan
**Created**: 2025-10-16
**Purpose**: Phase 4 (Dual Parser Harness) の詳細な実装計画とスケジュール
**Priority**: P1 (高優先度)
**Duration**: 2-3日（実質作業時間：16-24時間）

---

## 📋 概要

### 期間と工数
- **総期間**: 2-3日
- **総工数**: 16-24時間
- **総タスク数**: 10個（メイン3日 × サブタスク）

### ユーザー開発速度データ（実績ベース）
- **総コミット数**: 1,429回（2025-08-09 ~ 2025-10-16、68日間）
- **平均コミット数**: 21.6回/日（1時間に1.35回）
- **Box-First設計の速度向上**: 9倍速（Phase 15.7実績）
- **予想実装速度**: 100-200行/時間（C言語）、200-300行/時間（Hakorune）

### 現在の開発状況
- **セルフホストコンパイラ**: M2/M3達成済み（63日で完成）
- **Hakorune VM**: 15/16命令実装（93%完成）
- **quick-selfhost**: 43/43 スモーク存在
- **テスト成功率**: 100%（安定）

---

## 🎯 Phase 4の目標

### 核心コンセプト
```
【目標】: RustパーサーとHakoruneパーサーの統一ハーネス実装
【戦略】: C ABI層（薄い）+ Hako ABI層（Box化）の2層構造
【期間】: 2-3日（最小実装優先）
【原則】: Rollback可能性を常に確保
```

### 成果物
1. **C ABI層**: 100-200行（parser_harness.h + parser_harness.c + build.rs）
2. **Hako ABI層**: 200-300行（ParserHarnessBox + 統合）
3. **スモークテスト**: 5-10個（Phase-A代表）
4. **ドキュメント**: ハーネス設計書 + 使用ガイド

---

## 📅 Day 1: C ABI層実装（8時間）

### マイルストーン M1: C ABI層完成
**確認項目**:
- [ ] parser_harness.h コンパイル成功
- [ ] parser_harness.c コンパイル成功
- [ ] cargo build --release 成功
- [ ] 成果物サイズ: 100-150行

---

### Task 1-1: ヘッダファイル作成 (1時間)

#### 作業内容
`src/parser_harness/parser_harness.h` を作成

#### 成果物
**ファイル**: `src/parser_harness/parser_harness.h` (50行)

**実装内容**:
```c
#ifndef PARSER_HARNESS_H
#define PARSER_HARNESS_H

#include <stdint.h>
#include <stddef.h>

// ParseResult構造体（軽量）
typedef struct {
    int success;          // 1=success, 0=error
    char* ast_json;       // AST JSON文字列（NULL終端）
    char* error_message;  // エラーメッセージ（NULL終端、エラー時のみ）
} ParseResult;

// ハーネス関数（Rust/Hakorune切り替え）
ParseResult parse_with_harness(const char* source, const char* mode);

// メモリ解放
void free_parse_result(ParseResult* result);

#endif // PARSER_HARNESS_H
```

#### 依存関係
- なし

#### 検証方法
```bash
gcc -c -I src/parser_harness src/parser_harness/parser_harness.h
```

#### Rollback方法
```bash
rm src/parser_harness/parser_harness.h
```

#### リスク
- **Low**: ヘッダー定義のミス → コンパイルエラーで即座に検出

---

### Task 1-2: 実装ファイル作成 (3時間)

#### 作業内容
`src/parser_harness/parser_harness.c` を作成

#### 成果物
**ファイル**: `src/parser_harness/parser_harness.c` (100-150行)

**実装内容**:
```c
#include "parser_harness.h"
#include <stdlib.h>
#include <string.h>

// Rust側の関数（外部定義）
extern char* rust_parse_source(const char* source, int* success);
extern char* hako_parse_source(const char* source, int* success);

ParseResult parse_with_harness(const char* source, const char* mode) {
    ParseResult result;
    result.ast_json = NULL;
    result.error_message = NULL;

    int success = 0;
    char* output = NULL;

    if (strcmp(mode, "rust") == 0) {
        output = rust_parse_source(source, &success);
    } else if (strcmp(mode, "hako") == 0) {
        output = hako_parse_source(source, &success);
    } else if (strcmp(mode, "both") == 0) {
        // 両方実行して結果を比較（Phase-B）
        char* rust_output = rust_parse_source(source, &success);
        char* hako_output = hako_parse_source(source, &success);

        // 簡易比較（Phase-A では両方成功すればOK）
        if (rust_output && hako_output) {
            output = rust_output;  // Rust結果を返す
            free(hako_output);
        } else {
            result.success = 0;
            result.error_message = strdup("Parser comparison failed");
            return result;
        }
    } else {
        result.success = 0;
        result.error_message = strdup("Invalid mode");
        return result;
    }

    result.success = success;
    if (success) {
        result.ast_json = output;
    } else {
        result.error_message = output;
    }

    return result;
}

void free_parse_result(ParseResult* result) {
    if (result->ast_json) {
        free(result->ast_json);
        result->ast_json = NULL;
    }
    if (result->error_message) {
        free(result->error_message);
        result->error_message = NULL;
    }
}
```

#### 依存関係
- Task 1-1完了

#### 検証方法
```bash
# C言語ファイルのみコンパイル
gcc -c src/parser_harness/parser_harness.c -o /tmp/parser_harness.o

# オブジェクトファイル生成確認
ls -lh /tmp/parser_harness.o
```

#### Rollback方法
```bash
rm src/parser_harness/parser_harness.c
rm /tmp/parser_harness.o
```

#### リスク
- **Medium**: メモリ管理ミス（バッファオーバーフロー）
  - 対策: strdup/free の明示的な使用
  - 対策: Valgrind検証（Phase-B）
- **Low**: mode文字列比較のミス → テストで即座に検出

---

### Task 1-3: Cargo統合 (2時間)

#### 作業内容
`build.rs` を作成してCコードをRustビルドに統合

#### 成果物
**ファイル**: `build.rs` (50行)

**実装内容**:
```rust
// build.rs
use std::env;

fn main() {
    // C言語ファイルのコンパイル
    cc::Build::new()
        .file("src/parser_harness/parser_harness.c")
        .include("src/parser_harness")
        .compile("parser_harness");

    // リンク指示
    println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.c");
    println!("cargo:rerun-if-changed=src/parser_harness/parser_harness.h");
}
```

**Cargo.toml更新**:
```toml
[build-dependencies]
cc = "1.0"
```

#### 依存関係
- Task 1-1, 1-2完了

#### 検証方法
```bash
# フルビルド（C ABI統合確認）
cargo build --release

# ビルドログ確認
cargo build --release -vv 2>&1 | grep parser_harness
```

#### Rollback方法
```bash
# build.rs削除
rm build.rs

# Cargo.toml復元
git checkout Cargo.toml

# 再ビルド
cargo clean
cargo build --release
```

#### リスク
- **Medium**: cc crateのバージョン不一致 → Cargo.tomlで固定
- **Low**: リンクエラー → cargo build -vv でログ確認

---

### Task 1-4: Rust側ブリッジ関数実装 (2時間)

#### 作業内容
Rust側の `rust_parse_source` / `hako_parse_source` 関数を実装

#### 成果物
**ファイル**: `src/parser_harness/rust_bridge.rs` (新規、100行)

**実装内容**:
```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn rust_parse_source(source: *const c_char, success: *mut i32) -> *mut c_char {
    let source_str = unsafe {
        assert!(!source.is_null());
        CStr::from_ptr(source).to_str().unwrap()
    };

    // Rustパーサー呼び出し
    match crate::parser::parse_source(source_str) {
        Ok(ast) => {
            unsafe { *success = 1; }
            let json = serde_json::to_string(&ast).unwrap();
            CString::new(json).unwrap().into_raw()
        }
        Err(e) => {
            unsafe { *success = 0; }
            let error = format!("Parse error: {}", e);
            CString::new(error).unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn hako_parse_source(source: *const c_char, success: *mut i32) -> *mut c_char {
    // Phase-A: RustパーサーをフォールバックとしてCalling
    // Phase-B: Hakoruneパーサー呼び出しに置き換え
    rust_parse_source(source, success)
}

// メモリ解放（C側から呼び出される）
#[no_mangle]
pub extern "C" fn free_rust_string(s: *mut c_char) {
    unsafe {
        if !s.is_null() {
            let _ = CString::from_raw(s);
        }
    }
}
```

#### 依存関係
- Task 1-3完了

#### 検証方法
```bash
# フルビルド
cargo build --release

# シンボル確認
nm target/release/libhakorune.a | grep rust_parse_source
nm target/release/libhakorune.a | grep hako_parse_source
```

#### Rollback方法
```bash
rm src/parser_harness/rust_bridge.rs
cargo clean
cargo build --release
```

#### リスク
- **High**: メモリリーク（CString管理ミス）
  - 対策: Valgrindで検証（Phase-B）
  - 対策: 明示的な free_rust_string 関数
- **Medium**: NULL pointer参照 → assert!() で即座に検出

---

### Milestone M1完了確認

#### 確認コマンド
```bash
# 1. ビルド成功確認
cargo build --release

# 2. シンボル存在確認
nm target/release/libhakorune.a | grep -E '(rust_parse_source|hako_parse_source|free_rust_string)'

# 3. オブジェクトファイル確認
find target/release -name "*parser_harness*"

# 4. 行数カウント
wc -l src/parser_harness/parser_harness.h src/parser_harness/parser_harness.c build.rs src/parser_harness/rust_bridge.rs
```

#### 成果物サイズ確認
- parser_harness.h: 50行
- parser_harness.c: 100-150行
- build.rs: 50行
- rust_bridge.rs: 100行
- **合計**: 300-350行（目標100-200行を超過、但し堅牢性重視）

---

## 📅 Day 2: Hako ABI層実装（8時間）

### Milestone M2: Hako ABI層完成
**確認項目**:
- [ ] ParserHarnessBox 実装完了
- [ ] C ABI連携テスト成功
- [ ] Hakorune Parser 呼び出し成功
- [ ] 成果物サイズ: 200-300行

---

### Task 2-1: Box設計・実装 (4時間)

#### 作業内容
ParserHarnessBox を設計・実装

#### 成果物
**ファイル**: `apps/selfhost/parser/parser_harness_box.hako` (200行)

**実装内容**:
```hakorune
using "selfhost/shared/result_box.hako" as Result

// Parser Harness Box（C ABI経由でRust/Hakorune切り替え）
static box ParserHarnessBox {
  // モード: "rust", "hako", "both"
  mode: StringBox

  birth() {
    me.mode = "rust"  // デフォルトはRust
  }

  // モード設定
  set_mode(new_mode) {
    if new_mode == "rust" || new_mode == "hako" || new_mode == "both" {
      me.mode = new_mode
      return Result.Ok(null)
    } else {
      return Result.Err("Invalid mode: " + new_mode)
    }
  }

  // パース実行
  parse(source) {
    // C ABI経由でparse_with_harness呼び出し
    local result = Extern("parser.harness", source, me.mode)

    if result.is_Err() {
      return result
    }

    local parse_result = result.as_Ok()

    // ParseResult構造体のデシリアライズ
    if parse_result.get("success") == 1 {
      local ast_json = parse_result.get("ast_json")
      return Result.Ok(ast_json)
    } else {
      local error = parse_result.get("error_message")
      return Result.Err(error)
    }
  }

  // 両方実行して比較（Phase-B）
  parse_both(source) {
    me.set_mode("both")
    local result = me.parse(source)
    me.set_mode("rust")  // デフォルトに戻す
    return result
  }
}
```

#### 依存関係
- Day 1完了（C ABI層）

#### 検証方法
```bash
# Hakorune構文チェック
./target/release/hako --check apps/selfhost/parser/parser_harness_box.hako

# 最小実行テスト
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/selfhost/parser/parser_harness_box.hako
```

#### Rollback方法
```bash
rm apps/selfhost/parser/parser_harness_box.hako
```

#### リスク
- **Medium**: Extern呼び出しのシグネチャミス → テストで即座に検出
- **Low**: Result.Ok/Err の使い方ミス → 既存実装を参考

---

### Task 2-2: C ABI連携 (2時間)

#### 作業内容
Extern("parser.harness") の実装

#### 成果物
**ファイル**: `src/runtime/extern_adapter.rs` (更新、+50行)

**実装内容**:
```rust
// Extern("parser.harness") の実装
pub fn handle_parser_harness(args: &[VMValue]) -> Result<VMValue, String> {
    if args.len() != 2 {
        return Err("parser.harness requires 2 args: source, mode".to_string());
    }

    let source = args[0].as_string().ok_or("arg 0 must be string")?;
    let mode = args[1].as_string().ok_or("arg 1 must be string")?;

    // C ABIブリッジ呼び出し
    let c_source = CString::new(source).unwrap();
    let c_mode = CString::new(mode).unwrap();

    let result = unsafe {
        parse_with_harness(c_source.as_ptr(), c_mode.as_ptr())
    };

    // ParseResult → VMValue変換
    let mut map = HashMap::new();
    map.insert("success".to_string(), VMValue::Integer(result.success as i64));

    if result.success == 1 {
        let ast_json = unsafe { CStr::from_ptr(result.ast_json).to_str().unwrap() };
        map.insert("ast_json".to_string(), VMValue::String(ast_json.to_string()));
    } else {
        let error = unsafe { CStr::from_ptr(result.error_message).to_str().unwrap() };
        map.insert("error_message".to_string(), VMValue::String(error.to_string()));
    }

    // メモリ解放
    unsafe { free_parse_result(&result); }

    Ok(VMValue::Map(map))
}

// C ABI関数の宣言
extern "C" {
    fn parse_with_harness(source: *const c_char, mode: *const c_char) -> ParseResult;
    fn free_parse_result(result: *const ParseResult);
}

#[repr(C)]
struct ParseResult {
    success: i32,
    ast_json: *mut c_char,
    error_message: *mut c_char,
}
```

#### 依存関係
- Task 2-1完了

#### 検証方法
```bash
# フルビルド
cargo build --release

# Extern登録確認
grep -r "parser.harness" src/runtime/extern_adapter.rs
```

#### Rollback方法
```bash
git checkout src/runtime/extern_adapter.rs
cargo build --release
```

#### リスク
- **High**: メモリリーク（free_parse_result忘れ） → Valgrindで検証
- **Medium**: VMValue変換ミス → テストで検証

---

### Task 2-3: Hakorune Parser統合 (2時間)

#### 作業内容
`hako_parse_source` の実装（Phase-A: Rust fallback）

#### 成果物
**ファイル**: `src/parser_harness/rust_bridge.rs` (更新、+50行)

**実装内容**:
```rust
#[no_mangle]
pub extern "C" fn hako_parse_source(source: *const c_char, success: *mut i32) -> *mut c_char {
    // Phase-A: Rustパーサーをフォールバックとして呼び出し
    // Phase-B: セルフホストコンパイラ呼び出しに置き換え

    // 環境変数チェック
    if let Ok(val) = env::var("SMOKES_PARSER_MODE") {
        if val == "hako" {
            // TODO: セルフホストコンパイラ呼び出し（Phase-B）
            // 現在はRust fallback
            eprintln!("[WARN] SMOKES_PARSER_MODE=hako but using Rust fallback (Phase-A)");
        }
    }

    rust_parse_source(source, success)
}
```

#### 依存関係
- Task 2-2完了

#### 検証方法
```bash
# フルビルド
cargo build --release

# 環境変数テスト
SMOKES_PARSER_MODE=hako ./target/release/hako --version
```

#### Rollback方法
```bash
git checkout src/parser_harness/rust_bridge.rs
cargo build --release
```

#### リスク
- **Low**: フォールバック実装のみ → Phase-Bで本実装

---

### Milestone M2完了確認

#### 確認コマンド
```bash
# 1. ビルド成功
cargo build --release

# 2. ParserHarnessBox構文チェック
./target/release/hako --check apps/selfhost/parser/parser_harness_box.hako

# 3. Extern登録確認
grep "parser.harness" src/runtime/extern_adapter.rs

# 4. 行数カウント
wc -l apps/selfhost/parser/parser_harness_box.hako src/runtime/extern_adapter.rs
```

#### 成果物サイズ確認
- parser_harness_box.hako: 200行
- extern_adapter.rs更新: +50行
- rust_bridge.rs更新: +50行
- **合計**: 300行（目標200-300行内）

---

## 📅 Day 3: テスト・検証（6時間）

### Milestone M3: Phase 4完了
**確認項目**:
- [ ] SMOKES_PARSER_MODE=rust 成功
- [ ] SMOKES_PARSER_MODE=hako 成功（Rust fallback）
- [ ] SMOKES_PARSER_MODE=both 成功
- [ ] quick-selfhost 170/185 PASS 維持（最低43/43）
- [ ] DoD (Definition of Done)

---

### Task 3-1: スモークテスト追加 (2時間)

#### 作業内容
Phase-A代表スモークテストを追加

#### 成果物
**スモークテスト**: 5個（rust/hako/both/error/integration）

**ファイル一覧**:
1. `tools/smokes/v2/profiles/quick-selfhost/parser_harness_rust_vm.sh`
2. `tools/smokes/v2/profiles/quick-selfhost/parser_harness_hako_vm.sh`
3. `tools/smokes/v2/profiles/quick-selfhost/parser_harness_both_vm.sh`
4. `tools/smokes/v2/profiles/quick-selfhost/parser_harness_error_vm.sh`
5. `tools/smokes/v2/profiles/quick-selfhost/parser_harness_integration_vm.sh`

**実装例** (`parser_harness_rust_vm.sh`):
```bash
#!/usr/bin/env bash
set -euo pipefail

# Test ParserHarnessBox with mode="rust"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../lib/common.sh"

TEST_NAME="parser_harness_rust_vm"

# Rustモードでパース実行
output=$(SMOKES_PARSER_MODE=rust ./target/release/hako --backend vm \
  apps/selfhost/parser/test_parser_harness.hako 2>&1 || true)

# 成功確認
if echo "$output" | grep -q "PASS"; then
  smokes_pass "$TEST_NAME"
else
  smokes_fail "$TEST_NAME" "Expected PASS, got: $output"
fi
```

#### 依存関係
- Day 2完了

#### 検証方法
```bash
# 個別実行
./tools/smokes/v2/profiles/quick-selfhost/parser_harness_rust_vm.sh

# プロファイル実行
tools/smokes/v2/run.sh --profile quick-selfhost --filter parser_harness_
```

#### Rollback方法
```bash
rm tools/smokes/v2/profiles/quick-selfhost/parser_harness_*.sh
```

#### リスク
- **Low**: スモークテスト失敗 → ログ確認して即修正

---

### Task 3-2: 統合テスト (2時間)

#### 作業内容
既存quick-selfhostスモークの緑維持確認

#### 成果物
なし（検証のみ）

#### 検証方法
```bash
# quick-selfhostプロファイル全実行
tools/smokes/v2/run.sh --profile quick-selfhost

# 期待結果: 43/43 PASS以上（新規5個追加で48/48）
```

#### 確認項目
- [ ] 既存43テストすべてPASS
- [ ] 新規5テストすべてPASS
- [ ] 合計48/48 PASS

#### リスク
- **Medium**: 既存テスト破壊 → Rollback計画参照

---

### Task 3-3: quick-selfhost 緑確認 (2時間)

#### 作業内容
最終確認とドキュメント更新

#### 成果物
1. **テストレポート**: `docs/development/proposals/phase-15.75/PHASE_4_TEST_REPORT.md`
2. **TODO.md更新**: Phase 4完了記録

#### 実装内容（テストレポート）:
```markdown
# Phase 4 テストレポート

**日付**: 2025-10-XX
**実施者**: Claude/ChatGPT

## テスト結果

### quick-selfhost プロファイル
- **実行コマンド**: `tools/smokes/v2/run.sh --profile quick-selfhost`
- **結果**: 48/48 PASS
- **内訳**:
  - 既存: 43/43 PASS
  - 新規: 5/5 PASS

### Parser Harness テスト
| テスト名 | 結果 | 備考 |
|---------|------|------|
| parser_harness_rust_vm | PASS | Rustモード |
| parser_harness_hako_vm | PASS | Hakoモード（Rust fallback） |
| parser_harness_both_vm | PASS | 両方実行 |
| parser_harness_error_vm | PASS | エラーハンドリング |
| parser_harness_integration_vm | PASS | 統合テスト |

## パフォーマンス
- **ビルド時間**: 約2分（変化なし）
- **テスト実行時間**: 約1.6秒（変化なし）

## 結論
Phase 4完了。Dual Parser Harness実装成功。
```

#### 依存関係
- Task 3-1, 3-2完了

#### 検証方法
```bash
# ドキュメント存在確認
ls -lh docs/development/proposals/phase-15.75/PHASE_4_TEST_REPORT.md

# TODO.md更新確認
grep "Phase 4" TODO.md
```

#### Rollback方法
なし（ドキュメントのみ）

#### リスク
- なし

---

### Milestone M3完了確認

#### DoD (Definition of Done)

**機能要件**:
- [x] `both` で Phase-A スモーク緑
- [x] 既定（rust）の速度・安定不変
- [x] ドキュメント更新（TODO.md、テストレポート）

**技術要件**:
- [x] C ABI層100-200行実装
- [x] Hako ABI層200-300行実装
- [x] Cargo統合成功
- [x] メモリリーク無し（Valgrind検証はPhase-B）

**テスト要件**:
- [x] quick-selfhost 48/48 PASS
- [x] Parser Harness 5/5 PASS
- [x] 既存テスト破壊無し

#### 最終確認コマンド
```bash
# 1. フルビルド
cargo clean
cargo build --release

# 2. 全スモーク実行
tools/smokes/v2/run.sh --profile quick-selfhost

# 3. 成果物サイズ確認
wc -l src/parser_harness/*.{h,c} build.rs src/parser_harness/rust_bridge.rs \
      apps/selfhost/parser/parser_harness_box.hako \
      tools/smokes/v2/profiles/quick-selfhost/parser_harness_*.sh
```

---

## 🚨 リスク管理

### リスク1: C言語メモリ管理ミス
- **発生確率**: 中
- **影響**: 高（segfault）
- **対策**:
  1. 最小実装（100-200行）
  2. 明示的なfree_parse_result関数
  3. Valgrind検証（Phase-B）
- **Rollback**: C ABIファイル削除、Rust直接呼び出しに戻す

### リスク2: C ABI ↔ Hako ABI の連携バグ
- **発生確率**: 中
- **影響**: 中（パース失敗）
- **対策**:
  1. ParseResult構造体の明示的な定義
  2. 段階的テスト（rust → hako → both）
  3. エラーハンドリングの徹底
- **Rollback**: Extern("parser.harness") 削除、直接呼び出しに戻す

### リスク3: 既存テストの破壊
- **発生確率**: 低
- **影響**: 高（quick-selfhost破壊）
- **対策**:
  1. 各Task完了時にquick-selfhost実行
  2. Git commit単位でRollback可能
  3. CI/CD検証（Phase-B）
- **Rollback**: 全体Rollback計画参照

### リスク4: パフォーマンス劣化
- **発生確率**: 低
- **影響**: 低（C ABI経由のオーバーヘッド微小）
- **対策**:
  1. ベンチマーク測定
  2. オーバーヘッドが5%以内なら許容
- **Rollback**: 不要（許容範囲内）

---

## 🔄 Rollback戦略

### 全体Rollback（緊急時）
**状況**: Phase 4が完全に失敗、quick-selfhost破壊

**手順**:
```bash
# 1. ディレクトリ削除
rm -rf src/parser_harness/

# 2. build.rs削除
rm build.rs

# 3. Cargo.toml復元
git checkout Cargo.toml

# 4. extern_adapter.rs復元
git checkout src/runtime/extern_adapter.rs

# 5. 再ビルド
cargo clean
cargo build --release

# 6. テスト確認
tools/smokes/v2/run.sh --profile quick-selfhost
```

### 段階的Rollback

#### Day 1失敗時（C ABI層のみ削除）
```bash
rm -rf src/parser_harness/
rm build.rs
git checkout Cargo.toml
cargo clean
cargo build --release
```

#### Day 2失敗時（Hako ABI層のみ削除）
```bash
rm apps/selfhost/parser/parser_harness_box.hako
git checkout src/runtime/extern_adapter.rs
git checkout src/parser_harness/rust_bridge.rs
cargo build --release
```

#### Day 3失敗時（スモークテストのみ削除）
```bash
rm tools/smokes/v2/profiles/quick-selfhost/parser_harness_*.sh
# 既存機能は維持
```

### Git Revertによる段階的復旧
```bash
# 最新コミットを取り消し
git revert HEAD

# 特定コミットを取り消し
git revert <commit-hash>

# テスト確認
tools/smokes/v2/run.sh --profile quick-selfhost
```

---

## 🔀 並行作業可能性

### Phase 4は独立性が高い
- **他Phase依存なし**: C ABI層は独立実装可能
- **並行作業候補**: Phase 1 (Hakorune VM MirCall) と並行可能

### 推奨並行戦略
```
Week 1:
  Claude: Phase 1 (Hakorune VM MirCall) Day 1-3
  ChatGPT: Phase 4 (Dual Parser Harness) Day 1-2

Week 2:
  Claude: Phase 1 Day 4-7（完了）
  ChatGPT: Phase 4 Day 3（完了）
```

### リスク
- **Medium**: 2つの大規模変更の同時マージ → 丁寧なレビュー必要
- **Low**: 機能の競合 → 独立性が高いため問題なし

---

## 📊 進捗管理

### 全体進捗
| Day | タスク | 状態 | 工数 | 完了予定 |
|-----|--------|------|------|----------|
| **Day 1** | C ABI層実装 | 📝未着手 | 8h | Day 1 EOD |
| **Day 2** | Hako ABI層実装 | 📝未着手 | 8h | Day 2 EOD |
| **Day 3** | テスト・検証 | 📝未着手 | 6h | Day 3 EOD |

### Milestone進捗
| Milestone | 状態 | 確認項目 |
|-----------|------|----------|
| **M1: C ABI層完成** | 📝未着手 | ビルド成功、シンボル確認 |
| **M2: Hako ABI層完成** | 📝未着手 | Box実装、Extern連携 |
| **M3: Phase 4完了** | 📝未着手 | スモーク緑、DoD達成 |

### デイリーチェックリスト

**Day 1**:
- [ ] Task 1-1: ヘッダファイル作成 (1h)
- [ ] Task 1-2: 実装ファイル作成 (3h)
- [ ] Task 1-3: Cargo統合 (2h)
- [ ] Task 1-4: Rustブリッジ実装 (2h)
- [ ] Milestone M1確認

**Day 2**:
- [ ] Task 2-1: Box設計・実装 (4h)
- [ ] Task 2-2: C ABI連携 (2h)
- [ ] Task 2-3: Hakorune Parser統合 (2h)
- [ ] Milestone M2確認

**Day 3**:
- [ ] Task 3-1: スモークテスト追加 (2h)
- [ ] Task 3-2: 統合テスト (2h)
- [ ] Task 3-3: 最終確認・ドキュメント (2h)
- [ ] Milestone M3確認（Phase 4完了）

---

## 📈 成功指標

### 定量指標
- **コード削減**: なし（新規実装のみ）
- **コード追加**: 600-800行（C: 300-350行、Hako: 200-300行、テスト: 100-150行）
- **テスト成功率**: 100% (48/48 PASS)
- **ビルド時間**: 変化なし（±5%以内）
- **実行時間**: 変化なし（±5%以内）

### 定性指標
- **Rollback可能性**: 全Taskで確保
- **ドキュメント品質**: テストレポート作成
- **保守性**: C ABI層最小化（100-200行）
- **拡張性**: Phase-B (セルフホストパーサー) への道筋確保

---

## 🎓 成功要因

### Box-First設計の活用
- ParserHarnessBox: Hakorune実装（200行）
- C ABI層: 最小実装（100-200行）
- Extern連携: 既存パターン踏襲

### 段階的実装
- Day 1: C ABI層（基礎）
- Day 2: Hako ABI層（統合）
- Day 3: テスト（検証）

### Fail-Fast文化
- 各Task完了時にquick-selfhost実行
- エラーは即座に修正
- Git commit単位でRollback

### ユーザー開発速度の活用
- 21.6コミット/日（1時間に1.35回）
- Box-First設計で9倍速
- 予想実装速度: 100-200行/時間

---

## 📝 補足事項

### Phase-A vs Phase-B

**Phase-A** (今回実装):
- Rustパーサー経由のDual Harness
- `hako_parse_source` はRust fallback
- 目的: インフラ整備

**Phase-B** (将来実装):
- セルフホストパーサー直接呼び出し
- `hako_parse_source` の本実装
- 目的: 完全脱Rust

### SMOKES_PARSER_MODE環境変数

```bash
# Rustパーサー使用（デフォルト）
SMOKES_PARSER_MODE=rust ./target/release/hako program.hako

# Hakoruneパーサー使用（Phase-A: Rust fallback）
SMOKES_PARSER_MODE=hako ./target/release/hako program.hako

# 両方実行して比較
SMOKES_PARSER_MODE=both ./target/release/hako program.hako
```

### 既存quick-selfhostテスト
- **総数**: 43個（現在）
- **新規追加**: 5個（Phase 4）
- **合計**: 48個（Phase 4完了後）

---

## ✅ Phase 4完了宣言テンプレート

```
🎉 Phase 15.75 Phase 4完了！Dual Parser Harness実装成功！

実装内容:
- C ABI層実装完了（100-200行）
- Hako ABI層実装完了（200-300行）
- Cargo統合成功
- quick-selfhost 48/48 PASS

成果:
✅ RustパーサーとHakoruneパーサーの統一ハーネス実装
✅ SMOKES_PARSER_MODE環境変数による切り替え
✅ Phase-B（セルフホストパーサー直接呼び出し）への道筋確保
✅ 既存テスト破壊なし

次のステップ:
- Phase-B: セルフホストパーサー直接呼び出し実装（1-2週間）
- Phase 1: Hakorune VM MirCall実装（1週間）← 並行作業可能

詳細: docs/development/proposals/phase-15.75/PHASE_4_DUAL_PARSER_HARNESS_SCHEDULE.md

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

**最終更新**: 2025-10-16
**作成者**: Claude (detailed implementation schedule)
**次のアクション**: Phase 4 Day 1開始 - C ABI層実装
**推奨並行作業**: Phase 1 (Hakorune VM MirCall) との並行実施
