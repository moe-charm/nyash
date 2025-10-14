# Integration Strategy — Stage‑4 → Stage‑N (Claude詳細版)

## 🎯 **核心戦略: MIR JSON中間形式によるブートストラップ**

Phase 15.75の最終目標は、Hakorune(.hako)で書かれたコンパイラが自分自身をコンパイルする完全セルフホストです。

### **ブートストラップの課題**

```
課題: VmもすでにHakoruneで作っている (.hako)
   ↓
問題: .hakoを実行するにはVMが必要
   ↓
矛盾: VM自体が.hakoで書かれている（ブートストラップパラドックス）
```

### **解決策: Method C (MIR JSON中間形式)**

```bash
# フェーズ0: 完全Rust（現状）
rustc → hako.exe (Rust VM内蔵)

# フェーズ1: C ABI橋渡し（Stage 4-5）
hako.exe (Rust VM) + thin C ABI (~150行)
  ↓
parser.hako を実行
  ↓
parser が C ABI経由で呼ばれる

# フェーズ2: MIR JSON中間（Stage 6-N）
Step 1: .hako → MIR JSON (Rust VMで実行)
  hako-rust.exe parser.hako --emit-mir-json -o build/parser.mir.json

Step 2: MIR JSON → LLVM → オブジェクト
  python3 tools/llvmlite_harness.py build/parser.mir.json -o build/parser.o

Step 3: clangでリンク（ldではなく）
  clang build/parser.o build/mir_builder.o build/vm.o src/main_bootstrap.o \
    -L target/release -lnyrt \
    -o hako-selfhost.exe

# フェーズ3: 完全セルフホスト
hako-selfhost.exe が自分自身をコンパイル
  hako-selfhost.exe parser.hako → parser.mir.json → parser.o
  (Rust VMなし！)
```

---

## 📋 **ChatGPTの4つの改善提案（統合版）**

### **1. リンクには `clang` を使う（`ld` 直接ではなく）**

**理由**:
- プラットフォーム依存の問題を回避（Linux/macOS/Windowsで異なるldフラグ）
- clangが適切なCRTライブラリを自動リンク
- 標準ライブラリパスの自動解決

**実装例**:
```bash
# ❌ 避けるべき（プラットフォーム依存）
ld -o hako-selfhost.exe build/*.o -lnyrt -lc -lm

# ✅ 推奨（プラットフォーム非依存）
clang -o hako-selfhost.exe build/*.o -L target/release -lnyrt
```

### **2. 個別EXE → 統合の段階的アプローチ**

**理由**:
- 各コンポーネントの動作を個別にテスト可能
- デバッグが容易（問題の切り分け）
- 段階的な統合リスク管理

**実装例**:
```bash
# Stage 6: Parser単体EXE
tools/hako-build apps/selfhost/parser.hako -o bin/hako-parser.exe
echo "1+2;" | bin/hako-parser.exe --emit-ast

# Stage 7: MIR Builder単体EXE
tools/hako-build apps/selfhost/mir_builder.hako -o bin/hako-mir.exe
bin/hako-parser.exe program.hako | bin/hako-mir.exe --emit-mir-json

# Stage 8: VM単体EXE
tools/hako-build apps/selfhost/vm.hako -o bin/hako-vm.exe
bin/hako-mir.exe program.hako | bin/hako-vm.exe

# Stage 9: 統合（全部リンク）
clang build/parser.o build/mir.o build/vm.o build/main.o \
  -o hako-selfhost.exe
```

### **3. `tools/hako-build` ワンコマンドラッパー**

**理由**:
- 3ステップ（.hako → mir.json → .o → exe）を1コマンドに
- 一貫したビルドフロー
- CI/CD統合が容易

**実装例**:
```bash
#!/bin/bash
# tools/hako-build

SOURCE=$1
OUTPUT=$2
TEMP_MIR=$(mktemp --suffix=.mir.json)

# Step 1: .hako → MIR JSON
./target/release/hako "$SOURCE" --emit-mir-json -o "$TEMP_MIR"

# Step 2: MIR JSON → .o
python3 tools/llvmlite_harness.py "$TEMP_MIR" -o "$OUTPUT.o"

# Step 3: Link
clang "$OUTPUT.o" -L target/release -lnyrt -o "$OUTPUT"

rm "$TEMP_MIR"
echo "✅ Built: $OUTPUT"
```

**使用例**:
```bash
tools/hako-build apps/selfhost/parser.hako -o bin/hako-parser
```

### **4. MIR JSONにバージョン情報を付与**

**理由**:
- 再現可能ビルド（reproducible builds）
- デバッグ時のツールチェーン特定
- 互換性チェック

**実装例**:
```json
{
  "__metadata__": {
    "toolchain_version": "hakorune-0.1.0",
    "build_profile": "release",
    "timestamp": "2025-10-14T12:34:56Z",
    "source_file": "apps/selfhost/parser.hako",
    "mir_version": "1.0"
  },
  "functions": [
    {
      "name": "parse_source",
      "blocks": [...]
    }
  ]
}
```

**Rustコード追加箇所**:
```rust
// src/mir/emit.rs または src/runner/vm_pipeline.rs
pub fn emit_mir_json_with_metadata(mir: &MirModule, source_path: &str) -> String {
    let metadata = json!({
        "__metadata__": {
            "toolchain_version": env!("CARGO_PKG_VERSION"),
            "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "source_file": source_path,
            "mir_version": "1.0"
        }
    });
    // merge with actual MIR JSON...
}
```

---

## 🎯 **Stage完了基準（DoD）**

### **Stage 4 (Thin C ABI)**
- ✅ `Cargo.toml` に `parser-c-abi` feature追加（デフォルトOFF）
- ✅ `src/parser_harness/parser_harness.c` 実装（100-150行）
- ✅ `parse_source_dual()`, `free_parse_result()` 2関数のみ
- ✅ `HakoParseResult` 構造体（6フィールド: abi_version, struct_size, success, stmt_count, kind, error_msg）
- ✅ Rust externs: `parse_source_rust()`, `parse_source_hako()` 実装
- ✅ 1個のスモークテスト（`tools/smokes/v2/profiles/quick-selfhost/parser_facade_both_min_header_vm.sh`）
- ✅ `SMOKES_PARSER_MODE=both` でヘッダー比較（stmt_count, kind）
- ✅ Rollback可能（feature OFF で元に戻る）
- ✅ quick-selfhost: 170/185 PASS維持

### **Stage 5 (Parser完全移行)**
- ✅ `parse_source_hako()` が実際にHakorune Parser呼び出し（stubではなく）
- ✅ `apps/selfhost/parser.hako` 実装完了
- ✅ `SMOKES_PARSER_MODE=hako` 単独テスト成功
- ✅ `SMOKES_PARSER_MODE=both` 全テスト成功（ヘッダー一致）
- ✅ Rust Parser削除の準備完了（まだ削除しない、Stage 6で）
- ✅ quick-selfhost: 170/185 PASS維持

### **Stage N (完全セルフホスト)**
- ✅ `apps/selfhost/parser.hako` → `build/parser.mir.json` 生成成功
- ✅ `build/parser.mir.json` → `build/parser.o` コンパイル成功
- ✅ `clang` で全コンポーネントリンク成功
- ✅ `hako-selfhost.exe` が自分自身をビルド可能
- ✅ Rust VM削除（最終段階）
- ✅ 再現可能ビルド（MIR JSONバージョン管理）
- ✅ CI/CD統合完了

---

## 🔧 **実装手順（詳細版）**

### **Day 1: C ABI実装（Stage 4前半）**

#### **タスク1.1: プロジェクト設定**
```bash
# Cargo.toml
[features]
parser-c-abi = []

[build-dependencies]
cc = "1.0"

# build.rs
#[cfg(feature = "parser-c-abi")]
fn main() {
    cc::Build::new()
        .file("src/parser_harness/parser_harness.c")
        .include("src/parser_harness")
        .compile("parser_harness");
}

#[cfg(not(feature = "parser-c-abi"))]
fn main() {}
```

#### **タスク1.2: C ABIヘッダー**
```c
// src/parser_harness/parser_harness.h
#ifndef PARSER_HARNESS_H
#define PARSER_HARNESS_H

#include <stdint.h>

typedef enum {
    HAKO_PARSER_MODE_RUST = 0,
    HAKO_PARSER_MODE_HAKO = 1,
    HAKO_PARSER_MODE_BOTH = 2
} HakoParseMode;

typedef struct HakoParseResult {
    uint32_t abi_version;   /* must be 1 */
    uint32_t struct_size;   /* sizeof(HakoParseResult) */
    uint32_t success;       /* 1=ok, 0=error */
    uint32_t stmt_count;    /* minimal stat */
    const char* kind;       /* e.g., "Program" (owned) */
    const char* error_msg;  /* nullable (owned) */
} HakoParseResult;

/* Returns heap-allocated result */
HakoParseResult* parse_source_dual(const char* source_utf8, HakoParseMode mode);

/* Frees result + owned strings */
void free_parse_result(HakoParseResult* result);

#endif
```

#### **タスク1.3: C ABI実装**
```c
// src/parser_harness/parser_harness.c
#include "parser_harness.h"
#include <stdlib.h>
#include <string.h>

// Rust externs (declared in Rust code)
extern HakoParseResult* parse_source_rust(const char* src);
extern HakoParseResult* parse_source_hako(const char* src);

HakoParseResult* parse_source_dual(const char* source, HakoParseMode mode) {
    if (mode == HAKO_PARSER_MODE_RUST) {
        return parse_source_rust(source);
    } else if (mode == HAKO_PARSER_MODE_HAKO) {
        return parse_source_hako(source);
    } else if (mode == HAKO_PARSER_MODE_BOTH) {
        HakoParseResult* rust_res = parse_source_rust(source);
        HakoParseResult* hako_res = parse_source_hako(source);

        // Both must succeed
        if (rust_res->success == 0) {
            free_parse_result(hako_res);
            return rust_res;
        }
        if (hako_res->success == 0) {
            free_parse_result(rust_res);
            return hako_res;
        }

        // Compare headers
        if (rust_res->stmt_count != hako_res->stmt_count ||
            strcmp(rust_res->kind, hako_res->kind) != 0) {
            // Mismatch
            HakoParseResult* err = malloc(sizeof(HakoParseResult));
            err->abi_version = 1;
            err->struct_size = sizeof(HakoParseResult);
            err->success = 0;
            err->stmt_count = 0;
            err->kind = NULL;

            char* msg = malloc(256);
            snprintf(msg, 256, "mismatch: stmt rust=%u hako=%u",
                     rust_res->stmt_count, hako_res->stmt_count);
            err->error_msg = msg;

            free_parse_result(rust_res);
            free_parse_result(hako_res);
            return err;
        }

        // Success - return rust result, free hako
        free_parse_result(hako_res);
        return rust_res;
    }

    return NULL;
}

void free_parse_result(HakoParseResult* result) {
    if (!result) return;
    if (result->kind) free((void*)result->kind);
    if (result->error_msg) free((void*)result->error_msg);
    free(result);
}
```

### **Day 2: Rust Externs実装（Stage 4後半）**

#### **タスク2.1: Rust externs**
```rust
// src/front/parser_layer/c_abi.rs（新規作成）
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct HakoParseResult {
    pub abi_version: u32,
    pub struct_size: u32,
    pub success: u32,
    pub stmt_count: u32,
    pub kind: *const c_char,
    pub error_msg: *const c_char,
}

#[no_mangle]
pub extern "C" fn parse_source_rust(src: *const c_char) -> *mut HakoParseResult {
    let source = unsafe {
        if src.is_null() { return std::ptr::null_mut(); }
        CStr::from_ptr(src).to_str().unwrap_or("")
    };

    // Call existing Rust parser
    match crate::front::parser::NyashParser::parse_from_string(source) {
        Ok(ast) => {
            let stmt_count = count_statements(&ast);
            let kind = CString::new("Program").unwrap();

            let result = Box::new(HakoParseResult {
                abi_version: 1,
                struct_size: std::mem::size_of::<HakoParseResult>() as u32,
                success: 1,
                stmt_count,
                kind: kind.into_raw(),
                error_msg: std::ptr::null(),
            });
            Box::into_raw(result)
        }
        Err(e) => {
            let error_msg = CString::new(format!("{}", e)).unwrap();
            let result = Box::new(HakoParseResult {
                abi_version: 1,
                struct_size: std::mem::size_of::<HakoParseResult>() as u32,
                success: 0,
                stmt_count: 0,
                kind: std::ptr::null(),
                error_msg: error_msg.into_raw(),
            });
            Box::into_raw(result)
        }
    }
}

#[no_mangle]
pub extern "C" fn parse_source_hako(src: *const c_char) -> *mut HakoParseResult {
    // Stage 4: Stub (not-implemented)
    let error_msg = CString::new("not-implemented").unwrap();
    let result = Box::new(HakoParseResult {
        abi_version: 1,
        struct_size: std::mem::size_of::<HakoParseResult>() as u32,
        success: 0,
        stmt_count: 0,
        kind: std::ptr::null(),
        error_msg: error_msg.into_raw(),
    });
    Box::into_raw(result)
}

fn count_statements(ast: &crate::front::ast::ASTNode) -> u32 {
    // Simple count for MVP
    match ast {
        crate::front::ast::ASTNode::Block(stmts) => stmts.len() as u32,
        _ => 1,
    }
}
```

#### **タスク2.2: Runner統合**
```rust
// src/runner/vm_pipeline.rs の parse_and_merge_ast() を修正

pub fn parse_and_merge_ast(code: &str, prelude_asts: Vec<ASTNode>) -> Result<ASTNode> {
    #[cfg(feature = "parser-c-abi")]
    {
        let mode = std::env::var("SMOKES_PARSER_MODE").ok();
        if mode.as_deref() == Some("both") || mode.as_deref() == Some("hako") {
            return parse_via_c_abi(code, mode.as_deref().unwrap());
        }
    }

    // Existing path
    let use_facade = std::env::var("HAKO_FRONT_USE_FACADE")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(false);
    let main_ast = if use_facade {
        crate::front::parser_layer::facade::parse_source_to_ast(code)?
    } else {
        NyashParser::parse_from_string(code)?
    };

    // merge preludes...
    Ok(main_ast)
}

#[cfg(feature = "parser-c-abi")]
fn parse_via_c_abi(code: &str, mode: &str) -> Result<ASTNode> {
    use std::ffi::CString;
    use crate::front::parser_layer::c_abi::{HakoParseResult, parse_source_dual};

    let c_mode = match mode {
        "rust" => 0,
        "hako" => 1,
        "both" => 2,
        _ => return Err("Invalid SMOKES_PARSER_MODE".into()),
    };

    let c_source = CString::new(code).unwrap();
    let result = unsafe { parse_source_dual(c_source.as_ptr(), c_mode) };

    if result.is_null() {
        return Err("C ABI returned null".into());
    }

    let result_ref = unsafe { &*result };
    if result_ref.success == 0 {
        let error = unsafe {
            if result_ref.error_msg.is_null() {
                "Unknown error".to_string()
            } else {
                CStr::from_ptr(result_ref.error_msg).to_string_lossy().into_owned()
            }
        };
        unsafe { free_parse_result(result); }
        return Err(error.into());
    }

    // Success - but we need actual AST, not just header
    // For Stage 4, fallback to Rust parser for actual AST
    unsafe { free_parse_result(result); }
    NyashParser::parse_from_string(code)
}
```

### **Day 3: スモークテスト（Stage 4完了）**

```bash
#!/bin/bash
# tools/smokes/v2/profiles/quick-selfhost/parser_facade_both_min_header_vm.sh

export SMOKES_PARSER_MODE=both

./target/release/hako --features parser-c-abi <<'EOF'
static box Main {
    main() {
        return 42
    }
}
EOF

# Check exit code
if [ $? -eq 0 ]; then
    echo "✅ PASS: parser_facade_both_min_header_vm"
else
    echo "❌ FAIL: parser_facade_both_min_header_vm"
    exit 1
fi
```

---

## 🚨 **Rollback戦略**

### **Level 1: Feature無効化（2分）**
```bash
# C ABI無効
cargo build --release  # parser-c-abi feature無し

# 有効化（テスト時）
cargo build --release --features parser-c-abi
```

### **Level 2: ファイル削除（5分）**
```bash
rm -rf src/parser_harness/
rm src/front/parser_layer/c_abi.rs
git checkout build.rs Cargo.toml src/runner/vm_pipeline.rs
cargo build --release
```

### **Level 3: Full Rollback（30分）**
```bash
git revert --no-commit HEAD~5..HEAD
git commit -m "Rollback: Stage 4全削除"
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 170/185 PASS 復帰確認
```

---

## 📊 **期待される成果**

### **短期（Stage 4完了後）**
- ✅ 薄いC ABI層（100-150行）確立
- ✅ Feature-gated build安全性
- ✅ 1つのboth-modeスモークテストPASS
- ✅ Rollback可能性保証

### **中期（Stage 5完了後）**
- ✅ Hakorune Parser実装完了
- ✅ SMOKES_PARSER_MODE=hako 成功
- ✅ Rust Parser削除準備完了

### **長期（Stage N完了後）**
- ✅ 完全セルフホスト達成
- ✅ Rust 99,406行 → 10,400行（89.5%削減）
- ✅ 再現可能ビルド
- ✅ MIR JSON標準化

---

**作成者**: Claude (Sonnet 4.5)
**作成日**: 2025-10-14
**バージョン**: Claude詳細版（ChatGPTフィードバック統合）
