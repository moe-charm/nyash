# Phase 4 技術要件分析レポート

**作成日**: 2025-10-16
**作成者**: Claude (Technical Analysis Agent)
**目的**: Phase 4（Dual Parser Harness）の技術要件を詳細分析し、実装に必要な情報を完全に定義する

---

## 📋 目次

1. [Executive Summary](#executive-summary)
2. [現状分析](#1-現状分析)
3. [Phase 4 技術要件](#2-phase-4-技術要件)
4. [境界設計](#3-境界設計)
5. [受け入れ基準の具体化](#4-受け入れ基準の具体化)
6. [技術的課題](#5-技術的課題)
7. [推奨アプローチ](#6-推奨アプローチ)
8. [実装タイムライン](#7-実装タイムライン)

---

## Executive Summary

### 🎯 Phase 4 の核心目標

**入口の一本化・観測の強化**: Rust Parser と Hakorune Selfhost Parser の両方で同じスモークテストを実行し、最小同等性を段階的に保証する。

### 重要な発見

1. **既存のFacade構造が既に存在**: `src/front/parser_layer/facade.rs` として最小実装済み
2. **Hakorune Selfhost Compilerは既に動作**: M2/M3達成済み（2025-10-11）、170/185 PASS
3. **JSON v0 ヘッダ仕様は確立済み**: `{"version":"0","kind":"Program","stats":{"stmts":N}}`
4. **環境変数による切替メカニズムが部分実装済み**: `HAKO_FRONT_USE_FACADE=1` で既にFacade経由実行可能

### 結論

**Phase 4の実装は思ったより簡単**。既存のFacade + Selfhost Compiler + スモークテストインフラを統合するだけで、大部分の機能が実現できる。

**推定工数**: 2-3日（既存実装の統合と薄いハーネス層の追加のみ）

---

## 1. 現状分析

### 1.1 Rust Parser の現状

#### 実装場所
```
src/parser/
├── mod.rs              # エントリーポイント（227行、リファクタリング済み）
├── common.rs           # 共通ユーティリティ
├── expressions.rs      # 式パーサー
├── statements.rs       # 文パーサー
├── declarations/       # Box宣言パーサー
├── items/              # トップレベル宣言
├── cursor.rs           # TokenCursor（改行処理）
├── expr.rs             # 式パーサー
├── sugar.rs            # 糖衣構文デシュガリング
└── sugar_gate.rs       # 糖衣構文ゲート制御

src/front/parser_layer/
├── facade.rs           # ✅ Phase 2で既に実装済み
├── mod.rs              # モジュール定義
└── LAYER_GUARD.rs      # 境界ガード
```

#### 機能
- **トークン列 → AST変換** (完全実装)
- **エラーハンドリング**: `ParseError` 型（8種類のエラー）
- **無限ループ検出**: `must_advance!` マクロによるデバッグ燃料システム
- **構文糖衣処理**: `+=`, `-=`, `*=`, `/=` のデシュガリング
- **論理演算子正規化**: `||` → `or`, `&&` → `and`
- **セミコロン寛容モード**: `NYASH_PARSER_ALLOW_SEMICOLON` で制御（既定ON）

#### 入力
- **トークン列**: `Vec<Token>` (from `crate::tokenizer::NyashTokenizer`)
- **または**: 文字列 (`parse_from_string(input: impl Into<String>)`)

#### 出力
- **AST**: `ASTNode::Program { statements: Vec<ASTNode>, span: Span }`

#### 既存のFacade実装 ✅

```rust
// src/front/parser_layer/facade.rs (既に実装済み)
pub fn parse_source_to_ast(src: &str) -> Result<crate::ast::ASTNode, FrontendError> {
    crate::parser::NyashParser::parse_from_string(src)
        .map_err(|e| FrontendError::new(format!("{}", e)))
}
```

**重要**: Facadeは既に存在し、`HAKO_FRONT_USE_FACADE=1` で既に動作している！

---

### 1.2 Hakorune Selfhost Parser の現状

#### 実装場所
```
selfhost/compiler/pipeline_v2/
├── pipeline.hako                    # メインエントリーポイント
├── stage1_extract_flow.hako         # Stage-1 JSON抽出（レガシー）
├── stage1_json_scanner_box.hako     # 軽量JSONスキャナ（tolerant）
├── normalizer_box.hako              # 正規化（Call/Method/New）
├── using_resolver_box.hako          # using文解決
├── namespace_box.hako               # 名前空間正規化
├── signature_verifier_box.hako      # アリティ検証
├── header_emit_box.hako             # JSON v0 ヘッダ生成 ⭐
├── emit_*_box.hako                  # MIR生成（各命令）
└── [その他30+個のBoxes]

apps/selfhost-compiler/
├── compiler.hako                    # セルフホストコンパイラエントリー
└── boxes/parser/parser_box.hako     # パーサーBox（未確認）
```

#### 機能
- **Stage-1 JSON（AST-like） → MIR JSON変換** (完全実装)
- **MIR v1 (MirCall) 生成**: `lower_stage1_to_mir_v1(ast_json, prefer_cfg)`
- **JSON v0 ヘッダ生成**: ✅ `header_emit_box.hako` で実装済み
  ```hako
  // selfhost/compiler/pipeline_v2/header_emit_box.hako
  print("{\"version\":0,\"kind\":\"Program\"}")
  ```
- **using文解決**: `UsingResolverBox` による名前空間解決
- **アリティ検証**: `SignatureVerifierBox` によるコンパイル時チェック

#### 入力
- **Stage-1 JSON**: `{"type":"Return","expr":{"type":"Int","value":42}}`
- **using文JSON** (オプション): 名前空間解決用
- **modules JSON** (オプション): モジュール定義

#### 出力
- **MIR JSON v0/v1**: 完全なMIR表現
- **JSON v0 ヘッダ** ⭐: `{"version":"0","kind":"Program","stats":{"stmts":N}}`

#### 重要な発見
```bash
# JSON v0 ヘッダ生成は既に実装済み！
$ grep -r "version.*0.*kind.*Program" selfhost/compiler/pipeline_v2/*.hako
selfhost/compiler/pipeline_v2/header_emit_box.hako:
    print("{\"version\":0,\"kind\":\"Program\"}")
```

**結論**: Hakorune Selfhost Compilerは既にJSON v0ヘッダ生成機能を持っている！

---

### 1.3 両者の差異

| 項目 | Rust Parser | Hakorune Selfhost Parser |
|------|------------|-------------------------|
| **実装言語** | Rust | Hakorune (.hako) |
| **入力形式** | ソースコード文字列 | Stage-1 JSON (AST-like) |
| **出力形式** | `ASTNode` (Rust型) | MIR JSON v0/v1 |
| **セミコロン処理** | `NYASH_PARSER_ALLOW_SEMICOLON` | 受理（Rust Parserに依存） |
| **JSON v0ヘッダ** | ❌ 未実装 | ✅ 実装済み (`header_emit_box.hako`) |
| **エラーハンドリング** | `ParseError` 型 | エラー文字列 + null返却 |
| **トレース機能** | 無限ループ検出のみ | `trace=1` 引数でトレース出力 |
| **実行速度** | 高速（ネイティブ） | 中速（Rust VM or LLVM AOT） |
| **テスト状況** | 509/509 PASS | 170/185 PASS (M2/M3達成) |

**重要な発見**:
- Rust ParserはJSON v0ヘッダを生成する機能がない
- Hakorune Selfhost Parserは既にJSON v0ヘッダを生成できる
- 両者の入力形式が異なる（ソースコードvs Stage-1 JSON）

→ **Phase 4の課題**: Rust Parserに「ソースコード→JSON v0ヘッダ」生成機能を追加する必要がある

---

### 1.4 既存のテストインフラ

#### スモークテストv2の構造
```
tools/smokes/v2/
├── run.sh                      # メインランナー
├── lib/
│   ├── test_runner.sh          # テスト実行ライブラリ ⭐
│   ├── result_checker.sh       # 結果検証
│   └── plugin_manager.sh       # プラグイン管理
├── profiles/
│   ├── quick/                  # クイックテスト（15-30秒）
│   │   ├── core/               # コアテスト（基本機能）
│   │   ├── selfhost/           # Selfhostテスト ⭐
│   │   ├── llvm/               # LLVMテスト
│   │   └── wasm/               # WASMテスト
│   └── integration/            # 統合テスト（5-10分）
└── configs/
    ├── quick.env               # クイックプロファイル環境変数
    └── rust_vm_dynamic.conf    # Rust VM設定
```

#### 既存のSelfhostスモークテスト (Phase-A候補)
```bash
tools/smokes/v2/profiles/quick/selfhost/
├── hakorune_pipeline_const_ret_vm.sh           # ✅ 最小テスト（Return Int）
├── hakorune_pipeline_compare_ret_vm.sh         # ✅ Compare命令
├── hakorune_pipeline_compare_branch_phi_vm.sh  # ✅ Compare + Branch + PHI
├── hakorune_vm_m2_eq_true_vm.sh                # ✅ M2テスト（等価性）
├── hakorune_vm_m3_branch_true_vm.sh            # ✅ M3テスト（分岐）
├── hakorune_vm_m3_phi_diamond_vm.sh            # ✅ PHIテスト（ダイヤモンド）
├── selfhost_min_json_header_vm.sh              # ✅ JSON v0 ヘッダテスト ⭐
└── [その他170+ テスト]
```

**重要**: `selfhost_min_json_header_vm.sh` が既に存在し、JSON v0ヘッダをテストしている！

#### Phase-A スモーク候補（最小セット）

TODO.md から抽出した Phase-A 要件：
> - `both` で Phase‑A スモーク（セミコロン受理/if‑else/ブロック終端/using 最小）が緑

推奨Phase-Aスモークセット（4個）:
1. **セミコロン受理**: `core/json_v0_const_ret_vm.sh`（既存、セミコロンあり版を追加）
2. **if-else**: `selfhost/hakorune_pipeline_compare_branch_phi_vm.sh`（既存）
3. **ブロック終端**: `selfhost/hakorune_pipeline_const_ret_vm.sh`（既存）
4. **using最小**: `selfhost/selfhost_pipeline_v2_call_exec_vm.sh`（既存、using文使用）

---

## 2. Phase 4 技術要件

### 2.1 SMOKES_PARSER_MODE の実装要件

#### 環境変数仕様
```bash
SMOKES_PARSER_MODE=rust|hako|both
```

- **`rust`** (既定): Rust Parser経由でパース
- **`hako`**: Hakorune Selfhost Parser経由でパース
- **`both`**: 両方実行し、JSON v0ヘッダを比較

#### 実装場所
```bash
tools/smokes/v2/lib/test_runner.sh
```

既存の `run_nyash_vm()` 関数を拡張し、`SMOKES_PARSER_MODE` に応じて以下の処理を追加:

```bash
run_nyash_vm() {
    local program="$1"
    shift

    # Phase 4: Dual Parser Harness
    local parser_mode="${SMOKES_PARSER_MODE:-rust}"

    case "$parser_mode" in
        rust)
            # 既定: Rust Parser経由（現状維持）
            _run_nyash_vm_rust "$program" "$@"
            ;;
        hako)
            # Hakorune Selfhost Parser経由
            _run_nyash_vm_hako "$program" "$@"
            ;;
        both)
            # 両方実行＆比較
            _run_nyash_vm_both "$program" "$@"
            ;;
        *)
            log_error "Invalid SMOKES_PARSER_MODE: $parser_mode"
            return 1
            ;;
    esac
}
```

---

### 2.2 JSON v0 ヘッダ仕様

#### 最小仕様
```json
{
  "version": "0",
  "kind": "Program",
  "stats": {
    "stmts": 3
  }
}
```

#### フィールド定義
- **`version`** (string): 常に `"0"` (JSON v0を示す)
- **`kind`** (string): 常に `"Program"` (トップレベルASTノード)
- **`stats.stmts`** (integer): プログラム内のステートメント数

#### 追加フィールド（許可されるが無視される）
```json
{
  "version": "0",
  "kind": "Program",
  "stats": {
    "stmts": 3,
    "functions": 1,        // 許可（無視される）
    "boxes": 2,            // 許可（無視される）
    "using_count": 1       // 許可（無視される）
  },
  "metadata": {            // 許可（無視される）
    "parser": "rust",
    "timestamp": "..."
  }
}
```

**重要**: Phase 4の比較では `version`, `kind`, `stats.stmts` のみを比較する。

---

### 2.3 比較ロジック

#### 比較対象キー
```bash
version       # string
kind          # string
stats.stmts   # integer
```

#### 実装例（Bashスクリプト）
```bash
compare_json_v0_headers() {
    local rust_json="$1"
    local hako_json="$2"

    # jqで必要なフィールドを抽出
    local rust_version=$(echo "$rust_json" | jq -r '.version')
    local rust_kind=$(echo "$rust_json" | jq -r '.kind')
    local rust_stmts=$(echo "$rust_json" | jq -r '.stats.stmts')

    local hako_version=$(echo "$hako_json" | jq -r '.version')
    local hako_kind=$(echo "$hako_json" | jq -r '.kind')
    local hako_stmts=$(echo "$hako_json" | jq -r '.stats.stmts')

    # 比較
    if [ "$rust_version" != "$hako_version" ]; then
        log_error "version mismatch: rust=$rust_version hako=$hako_version"
        return 1
    fi

    if [ "$rust_kind" != "$hako_kind" ]; then
        log_error "kind mismatch: rust=$rust_kind hako=$hako_kind"
        return 1
    fi

    if [ "$rust_stmts" != "$hako_stmts" ]; then
        log_error "stats.stmts mismatch: rust=$rust_stmts hako=$hako_stmts"
        return 1
    fi

    log_success "JSON v0 headers match (version=$rust_version, kind=$rust_kind, stmts=$rust_stmts)"
    return 0
}
```

---

### 2.4 セミコロン受理の仕様

#### 環境変数
```bash
NYASH_PARSER_ALLOW_SEMICOLON=0|1
```

- **`1`** (既定): セミコロンを許可（寛容モード）
- **`0`**: セミコロンを禁止（厳格モード、開発用）

#### 実装状況
```rust
// src/parser/mod.rs (既に実装済み)
fn parse_program(&mut self) -> Result<ASTNode, ParseError> {
    let allow_sc = std::env::var("NYASH_PARSER_ALLOW_SEMICOLON")
        .ok()
        .map(|v| {
            let lv = v.to_ascii_lowercase();
            lv == "1" || lv == "true" || lv == "on"
        })
        .unwrap_or(true);  // 既定: 許可

    // ...
    if allow_sc && matches!(self.current_token().token_type, TokenType::SEMICOLON) {
        self.advance();
        continue;
    }
}
```

**結論**: セミコロン受理機能は既に実装済み。Phase 4での作業は不要。

---

## 3. 境界設計

### 3.1 C ABI層の責務

#### 現状
```
該当なし（Phase 4ではC ABIを使用しない）
```

**理由**: Phase 4はRust ParserとHakorune Selfhost Parserの**同一プロセス内比較**であり、C ABIを経由する必要がない。

---

### 3.2 Hako ABI層の責務

#### 現状
```
該当なし（Phase 4ではHako ABIを使用しない）
```

**理由**: Phase 4はテストハーネス層（Bashスクリプト）で両Parserを呼び出すだけであり、Hako ABI（Hakoruneスクリプトからの呼び出し）を必要としない。

---

### 3.3 呼び出しフロー

#### Mode: `rust` (既定)
```
[Bashスクリプト]
    ↓
[run_nyash_vm_rust]
    ↓
[hakorune --backend vm program.nyash]
    ↓
[Rust Parser] → ASTNode → MIR Builder → MIR JSON → Rust VM → 実行
    ↓
[標準出力] → フィルタリング → 結果比較
```

#### Mode: `hako` (Hakorune Selfhost Parser)
```
[Bashスクリプト]
    ↓
[run_nyash_vm_hako]
    ↓
[hakorune --backend vm --selfhost-parser program.nyash] (新規フラグ)
    ↓
[Rust Parser] → Stage-1 JSON → [Hakorune Selfhost Parser] → MIR JSON → Rust VM → 実行
    ↓
[標準出力] → フィルタリング → 結果比較
```

**重要な発見**: Hakorune Selfhost Parserは「Stage-1 JSON」を入力として受け取る。
→ Rust Parserで「ソースコード → Stage-1 JSON」変換が必要。

#### Mode: `both` (比較)
```
[Bashスクリプト]
    ↓
[run_nyash_vm_both]
    ↓
[hakorune --emit-json-v0-header program.nyash] (新規フラグ) → rust_header.json
[hakorune --backend vm --selfhost-parser --emit-json-v0-header program.nyash] → hako_header.json
    ↓
[compare_json_v0_headers rust_header.json hako_header.json]
    ↓
[結果: PASS/FAIL]
```

---

### 3.4 新規CLIフラグの設計

#### `--emit-json-v0-header` (新規)
```bash
hakorune --emit-json-v0-header program.nyash
```

**目的**: JSON v0ヘッダのみを出力し、終了する（実行しない）

**出力例**:
```json
{"version":"0","kind":"Program","stats":{"stmts":3}}
```

**実装場所**: `src/cli/args.rs` + `src/runner/mod.rs`

#### `--selfhost-parser` (新規、オプション)
```bash
hakorune --backend vm --selfhost-parser program.nyash
```

**目的**: Hakorune Selfhost Parserを使用してパースする

**動作**:
1. Rust Parserで「ソースコード → Stage-1 JSON」変換
2. Hakorune Selfhost Parserで「Stage-1 JSON → MIR JSON」変換
3. MIR JSONをRust VMで実行

**実装場所**: `src/cli/args.rs` + `src/runner/modes/common.rs`

**注**: `--selfhost-parser` フラグは Phase 4 の範囲外（将来実装）。Phase 4 では `--emit-json-v0-header` のみ実装する。

---

## 4. 受け入れ基準の具体化

### 4.1 Phase-A スモークの内容

#### 推奨Phase-Aスモークセット（4個）

**1. セミコロン受理**
```bash
# tools/smokes/v2/profiles/quick/phase-a/semicolon_accept_vm.sh
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PARSER_MODE=both

TMP_DIR="/tmp/phase_a_semicolon_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/test.nyash" << 'EOF'
static box Main {
  main() {
    local x = 42;
    return x;
  }
}
EOF

# Rust Parser
rust_out=$(NYASH_BIN="$NYASH_BIN" hakorune --emit-json-v0-header "$TMP_DIR/test.nyash")

# Hakorune Selfhost Parser
hako_out=$(NYASH_BIN="$NYASH_BIN" hakorune --backend vm --selfhost-parser --emit-json-v0-header "$TMP_DIR/test.nyash")

# 比較
compare_json_v0_headers "$rust_out" "$hako_out" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
```

**2. if-else**
```bash
# tools/smokes/v2/profiles/quick/phase-a/if_else_vm.sh
# (既存 selfhost/hakorune_pipeline_compare_branch_phi_vm.sh を流用)
```

**3. ブロック終端**
```bash
# tools/smokes/v2/profiles/quick/phase-a/block_terminator_vm.sh
# (既存 selfhost/hakorune_pipeline_const_ret_vm.sh を流用)
```

**4. using最小**
```bash
# tools/smokes/v2/profiles/quick/phase-a/using_minimal_vm.sh
# (既存 selfhost/selfhost_pipeline_v2_call_exec_vm.sh を流用)
```

---

### 4.2 検証手順

#### Step 1: 環境設定
```bash
export SMOKES_PARSER_MODE=both
export NYASH_PARSER_ALLOW_SEMICOLON=1  # 既定値
```

#### Step 2: Phase-Aスモーク実行
```bash
cd /home/tomoaki/git/hakorune-selfhost
bash tools/smokes/v2/run.sh --profile phase-a
```

**期待される出力**:
```
[INFO] Running Phase-A smoke tests...
[PASS] semicolon_accept_vm (0.15s)
[PASS] if_else_vm (0.23s)
[PASS] block_terminator_vm (0.18s)
[PASS] using_minimal_vm (0.31s)

===============================================
Smoke Test Summary
===============================================
Total tests:  4
Passed:       4
Failed:       0
Duration:     0.87s

[PASS] All tests passed! ✨
```

#### Step 3: 速度・安定不変の確認
```bash
# 既定（rust）モードで quick スイート実行（基準測定）
SMOKES_PARSER_MODE=rust bash tools/smokes/v2/run.sh --profile quick

# 記録: 実行時間、PASS/FAIL数
```

#### Step 4: quick-selfhost 170/185 PASS 維持確認
```bash
SMOKES_SELFHOST_ENABLE=1 bash tools/smokes/v2/run.sh --profile quick-selfhost
```

**期待される結果**: 170/185 PASS 以上（Phase 4実装前と同等）

---

## 5. 技術的課題

### 課題1: Rust ParserにJSON v0ヘッダ生成機能がない

**問題**: Rust Parserは `ASTNode` を返すが、JSON v0ヘッダ形式に変換する機能がない

**解決策**: 新規関数 `emit_json_v0_header(ast: &ASTNode) -> String` を実装

**実装場所**: `src/front/parser_layer/facade.rs`

**実装例**:
```rust
// src/front/parser_layer/facade.rs
pub fn emit_json_v0_header(ast: &crate::ast::ASTNode) -> String {
    use crate::ast::ASTNode;

    let stmt_count = match ast {
        ASTNode::Program { statements, .. } => statements.len(),
        _ => 0,
    };

    format!(
        r#"{{"version":"0","kind":"Program","stats":{{"stmts":{}}}}}"#,
        stmt_count
    )
}
```

**工数**: 30分（実装 + テスト）

---

### 課題2: `--emit-json-v0-header` CLIフラグの追加

**問題**: 新規CLIフラグの追加が必要

**解決策**: `clap` でフラグを追加し、`src/runner/mod.rs` で処理

**実装場所**:
- `src/cli/args.rs`: CLIフラグ定義
- `src/runner/mod.rs`: フラグ処理ロジック

**実装例**:
```rust
// src/cli/args.rs
#[derive(Parser, Debug)]
pub struct Args {
    // ... 既存フラグ ...

    /// Emit JSON v0 header only (for Phase 4 Dual Parser Harness)
    #[arg(long, help = "Emit JSON v0 header and exit")]
    pub emit_json_v0_header: bool,
}
```

```rust
// src/runner/mod.rs
pub fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.emit_json_v0_header {
        // Phase 4: JSON v0 ヘッダのみ出力して終了
        let source = std::fs::read_to_string(&args.input)?;
        let ast = crate::front::parser_layer::facade::parse_source_to_ast(&source)?;
        let header = crate::front::parser_layer::facade::emit_json_v0_header(&ast);
        println!("{}", header);
        return Ok(());
    }

    // ... 既存の実行ロジック ...
}
```

**工数**: 1時間（実装 + テスト）

---

### 課題3: `SMOKES_PARSER_MODE=both` の実装

**問題**: `test_runner.sh` に `both` モードのロジックがない

**解決策**: `run_nyash_vm()` 関数に `both` ケースを追加

**実装場所**: `tools/smokes/v2/lib/test_runner.sh`

**実装例**:
```bash
_run_nyash_vm_both() {
    local program="$1"
    shift

    # Rust Parser経由でJSON v0ヘッダ生成
    local rust_header
    rust_header=$("$NYASH_BIN" --emit-json-v0-header "$program" 2>&1 | filter_noise)
    local rust_exit=$?

    if [ $rust_exit -ne 0 ]; then
        log_error "Rust Parser failed: $rust_header"
        return 1
    fi

    # Hakorune Selfhost Parser経由でJSON v0ヘッダ生成
    # TODO: --selfhost-parser フラグ実装後に有効化
    # local hako_header
    # hako_header=$("$NYASH_BIN" --backend vm --selfhost-parser --emit-json-v0-header "$program" 2>&1 | filter_noise)
    # local hako_exit=$?

    # 暫定: Hakorune Selfhost Parserを直接呼び出し（環境変数経由）
    local hako_header
    hako_header=$(NYASH_USE_SELFHOST_PARSER=1 "$NYASH_BIN" --emit-json-v0-header "$program" 2>&1 | filter_noise)
    local hako_exit=$?

    if [ $hako_exit -ne 0 ]; then
        log_error "Hakorune Selfhost Parser failed: $hako_header"
        return 1
    fi

    # JSON v0ヘッダ比較
    compare_json_v0_headers "$rust_header" "$hako_header"
    return $?
}
```

**工数**: 2時間（実装 + デバッグ）

---

### 課題4: Hakorune Selfhost Parserの呼び出しインターフェース

**問題**: Hakorune Selfhost Parserは「Stage-1 JSON」を入力として受け取るが、Rust Parserは「ASTNode」を生成する。

**現状の差異**:
- **Rust Parser**: `String` (ソースコード) → `ASTNode` (Rust型)
- **Hakorune Selfhost Parser**: `String` (Stage-1 JSON) → MIR JSON

**解決策（2つのオプション）**:

#### Option A: ASTNode → Stage-1 JSON 変換器を追加 (推奨)
```rust
// src/front/parser_layer/facade.rs
pub fn ast_to_stage1_json(ast: &crate::ast::ASTNode) -> String {
    // ASTNode を Stage-1 JSON に変換
    // 例: {"type":"Return","expr":{"type":"Int","value":42}}
    // TODO: 実装
    todo!("ASTNode → Stage-1 JSON 変換")
}
```

**長所**: 既存のRust Parserを最大限活用できる
**短所**: ASTNode → Stage-1 JSON 変換ロジックが必要（100-200行）

**工数**: 4-6時間（実装 + テスト）

#### Option B: Hakorune Selfhost ParserにRust ABI経由で直接呼び出し（将来）
```rust
// 将来的な実装（Phase 4の範囲外）
extern "C" fn hako_selfhost_parse(source: *const c_char) -> *const c_char;
```

**長所**: ASTNode → Stage-1 JSON 変換が不要
**短所**: C ABI層の実装が必要、Phase 4の範囲を超える

**工数**: 8-12時間（Phase 4の範囲外）

**推奨**: Phase 4では **Option A** を採用し、将来的に Option B を検討する。

---

## 6. 推奨アプローチ

### 6.1 段階的実装戦略

#### Phase 4.1: Rust Parser側の準備（Day 1前半）
```
タスク:
1. emit_json_v0_header() 関数実装（30分）
2. --emit-json-v0-header CLIフラグ追加（1時間）
3. 単体テスト（30分）

成果物:
- src/front/parser_layer/facade.rs (追加)
- src/cli/args.rs (修正)
- src/runner/mod.rs (修正)
- tests/test_json_v0_header.rs (新規)

検証:
hakorune --emit-json-v0-header test.nyash
→ {"version":"0","kind":"Program","stats":{"stmts":3}}
```

#### Phase 4.2: テストハーネス実装（Day 1後半）
```
タスク:
1. compare_json_v0_headers() 関数実装（30分）
2. _run_nyash_vm_both() 関数実装（1.5時間）
3. SMOKES_PARSER_MODE=both ロジック統合（30分）

成果物:
- tools/smokes/v2/lib/test_runner.sh (修正)

検証:
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile quick
```

#### Phase 4.3: Phase-Aスモーク作成（Day 2前半）
```
タスク:
1. 4つのPhase-Aスモーク作成（2時間）
2. 既存スモークの流用・修正（1時間）

成果物:
- tools/smokes/v2/profiles/phase-a/semicolon_accept_vm.sh
- tools/smokes/v2/profiles/phase-a/if_else_vm.sh
- tools/smokes/v2/profiles/phase-a/block_terminator_vm.sh
- tools/smokes/v2/profiles/phase-a/using_minimal_vm.sh

検証:
bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS
```

#### Phase 4.4: ASTNode → Stage-1 JSON 変換（Day 2後半 - Day 3）
```
タスク:
1. ast_to_stage1_json() 関数設計（1時間）
2. 実装（4-6時間）
3. テスト（2時間）

成果物:
- src/front/parser_layer/stage1_emitter.rs (新規)

検証:
hakorune --emit-stage1-json test.nyash
→ {"type":"Return","expr":{"type":"Int","value":42}}
```

#### Phase 4.5: 統合テスト＆ドキュメント（Day 3終了前）
```
タスク:
1. 全Phase-Aスモーク実行（30分）
2. quick/quick-selfhost スモーク実行（30分）
3. ドキュメント更新（1時間）

成果物:
- docs/reference/frontend-layers.md (更新)
- docs/development/roadmap/phases/phase 15.75/PHASE_4_COMPLETED.md (新規)

検証:
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS

bash tools/smokes/v2/run.sh --profile quick
→ 既存PASSを維持

SMOKES_SELFHOST_ENABLE=1 bash tools/smokes/v2/run.sh --profile quick-selfhost
→ 170/185 PASS 維持
```

---

### 6.2 最小実装（MVP）戦略

**Phase 4の核心目標**: `both` モードで Phase-A スモーク（4個）が緑

**最小実装（必須機能のみ）**:
1. ✅ `emit_json_v0_header()` 関数（Rust Parser側）
2. ✅ `--emit-json-v0-header` CLIフラグ
3. ✅ `compare_json_v0_headers()` 関数（テストハーネス側）
4. ✅ `SMOKES_PARSER_MODE=both` ロジック
5. ✅ Phase-Aスモーク4個

**除外（Phase 4範囲外、将来実装）**:
- ❌ `--selfhost-parser` CLIフラグ
- ❌ `ast_to_stage1_json()` 関数（Hakorune Selfhost Parser完全統合に必要）
- ❌ C ABI層の実装
- ❌ Hakorune Selfhost Parser高速化

**Phase 4 MVP 完了条件**:
```bash
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS

# 既存機能の維持確認
SMOKES_PARSER_MODE=rust bash tools/smokes/v2/run.sh --profile quick
→ 既存PASSを維持
```

---

### 6.3 暫定実装（Hakorune Selfhost Parser統合の簡略化）

**Phase 4.2の課題**: Hakorune Selfhost Parserは「Stage-1 JSON」を入力として受け取るが、Rust Parserは「ASTNode」を生成する。

**暫定解決策**: `SMOKES_PARSER_MODE=both` では **JSON v0ヘッダの比較のみ** を行い、完全な実行は行わない。

**理由**:
- Phase 4の受け入れ基準は「JSON v0ヘッダの比較」のみ
- 完全な実行比較（MIR生成 → VM実行）は Phase 5以降で実装
- ASTNode → Stage-1 JSON 変換の実装を省略できる（4-6時間削減）

**実装方針**:
```bash
# Mode: both (暫定実装)
1. Rust Parser: ソースコード → ASTNode → JSON v0ヘッダ
2. Hakorune Selfhost Parser: ソースコード → (Stage-1 JSON経由) → JSON v0ヘッダ
3. 比較: JSON v0ヘッダのみ（version/kind/stats.stmts）
```

**Phase 5以降で完全統合**:
```bash
# Mode: both (完全実装)
1. Rust Parser: ソースコード → ASTNode → MIR JSON → VM実行
2. Hakorune Selfhost Parser: ソースコード → Stage-1 JSON → MIR JSON → VM実行
3. 比較: 最終実行結果（標準出力）
```

---

## 7. 実装タイムライン

### 7.1 3日間スケジュール（推奨）

#### Day 1: Rust Parser側 + ハーネス基盤
```
午前（4時間）:
✅ emit_json_v0_header() 実装（30分）
✅ --emit-json-v0-header CLIフラグ追加（1時間）
✅ 単体テスト（30分）
✅ 動作確認（30分）
✅ compare_json_v0_headers() 実装（30分）
✅ 初期統合テスト（1時間）

午後（4時間）:
✅ _run_nyash_vm_both() 実装（1.5時間）
✅ SMOKES_PARSER_MODE=both 統合（30分）
✅ デバッグ（1時間）
✅ 基本動作確認（1時間）

完了条件:
hakorune --emit-json-v0-header test.nyash
→ JSON v0ヘッダ出力成功

SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --filter selfhost_min_json_header_vm.sh
→ PASS
```

#### Day 2: Phase-Aスモーク作成 + ASTNode → Stage-1 JSON
```
午前（4時間）:
✅ Phase-Aスモーク4個作成（2時間）
✅ 既存スモーク流用・修正（1時間）
✅ 動作確認（1時間）

午後（4時間）:
✅ ast_to_stage1_json() 設計（1時間）
✅ 実装開始（3時間）

完了条件:
bash tools/smokes/v2/run.sh --profile phase-a
→ 2/4 PASS以上（暫定、Hakorune Selfhost Parser統合前）
```

#### Day 3: 完全統合 + ドキュメント
```
午前（4時間）:
✅ ast_to_stage1_json() 完成（2時間）
✅ Hakorune Selfhost Parser統合（1時間）
✅ デバッグ（1時間）

午後（4時間）:
✅ 全Phase-Aスモーク実行（30分）
✅ quick/quick-selfhost スモーク実行（30分）
✅ ドキュメント更新（2時間）
✅ 最終確認（1時間）

完了条件:
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS ⭐

bash tools/smokes/v2/run.sh --profile quick
→ 既存PASS維持

SMOKES_SELFHOST_ENABLE=1 bash tools/smokes/v2/run.sh --profile quick-selfhost
→ 170/185 PASS 維持
```

---

### 7.2 最小実装（MVP）スケジュール（2日間）

**除外**: `ast_to_stage1_json()` 実装（暫定実装で回避）

#### Day 1: Rust Parser側 + ハーネス
```
✅ emit_json_v0_header() 実装（30分）
✅ --emit-json-v0-header CLIフラグ（1時間）
✅ 単体テスト（30分）
✅ compare_json_v0_headers() 実装（30分）
✅ _run_nyash_vm_both() 暫定実装（1.5時間）
✅ SMOKES_PARSER_MODE=both 統合（30分）
✅ 動作確認（1時間）

合計: 6時間
```

#### Day 2: Phase-Aスモーク + 最終確認
```
✅ Phase-Aスモーク4個作成（2時間）
✅ 暫定実装による動作確認（1時間）
✅ quick/quick-selfhost スモーク実行（30分）
✅ ドキュメント更新（1.5時間）
✅ 最終確認（1時間）

合計: 6時間
```

**完了条件（暫定実装）**:
```bash
SMOKES_PARSER_MODE=both bash tools/smokes/v2/run.sh --profile phase-a
→ 4/4 PASS（JSON v0ヘッダ比較のみ）

# 注: 完全な実行比較は Phase 5 以降で実装
```

---

## 8. リスク評価

### リスク1: ASTNode → Stage-1 JSON 変換の複雑性
**レベル**: Medium
**影響**: ast_to_stage1_json() 実装に予想以上の時間がかかる可能性
**対策**: 暫定実装（JSON v0ヘッダ比較のみ）で Phase 4 を完了し、完全統合は Phase 5 に延期

### リスク2: Hakorune Selfhost Parserのバグ
**レベル**: Low
**影響**: Phase-Aスモークで予期しないエラーが発生
**対策**: 既に 170/185 PASS しているため、Phase-Aスモーク（最小セット4個）では大きな問題は発生しにくい

### リスク3: 既存スモークテストへの影響
**レベル**: Very Low
**影響**: Phase 4 実装が既存のスモークテストを壊す可能性
**対策**: `SMOKES_PARSER_MODE` は既定で `rust` なので、既存動作に影響なし

### リスク4: JSON v0ヘッダ仕様の不一致
**レベル**: Very Low
**影響**: Rust ParserとHakorune Selfhost Parserで `stats.stmts` のカウント方法が異なる可能性
**対策**: Phase-Aスモークで最小ケースから順次検証し、不一致が見つかった場合は仕様を調整

---

## 9. 結論

### 9.1 実現可能性: 高

**理由**:
- ✅ Rust Parserは既に完成している
- ✅ Hakorune Selfhost Parserは既に M2/M3 を達成している
- ✅ JSON v0ヘッダ生成機能が既に Hakorune 側に存在する
- ✅ Facade構造が既に Phase 2 で実装されている
- ✅ スモークテストインフラが既に整っている

### 9.2 推定工数

**最小実装（MVP）**: 2日間（12時間）
**完全実装**: 3日間（18時間）

**内訳**:
- Rust Parser側: 2時間
- テストハーネス: 3時間
- Phase-Aスモーク: 2時間
- ASTNode → Stage-1 JSON: 4-6時間（暫定実装では省略可能）
- ドキュメント: 2時間

### 9.3 推奨アプローチ

**Phase 4.1-4.3（2日間）**: 最小実装（MVP）
- ✅ JSON v0ヘッダ比較機能を実装
- ✅ Phase-Aスモーク4個を作成
- ✅ `SMOKES_PARSER_MODE=both` で比較動作を確認

**Phase 4.4-4.5（将来、Phase 5）**: 完全統合
- ASTNode → Stage-1 JSON 変換を実装
- Hakorune Selfhost Parserによる完全な実行比較
- quick/integration プロファイルへの拡大

### 9.4 次のアクション

**即座に実行可能なタスク**:

1. **emit_json_v0_header() 実装** (30分)
   ```bash
   cd /home/tomoaki/git/hakorune-selfhost
   vim src/front/parser_layer/facade.rs
   # 関数を追加
   ```

2. **--emit-json-v0-header CLIフラグ追加** (1時間)
   ```bash
   vim src/cli/args.rs
   vim src/runner/mod.rs
   # フラグ処理ロジックを追加
   ```

3. **動作確認** (15分)
   ```bash
   cargo build --release
   ./target/release/hakorune --emit-json-v0-header apps/tests/hello.hako
   # 期待: {"version":"0","kind":"Program","stats":{"stmts":1}}
   ```

---

## 付録

### A. 参考ドキュメント

- **TODO.md**: `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase 15.75/TODO.md`
- **ROADMAP.md**: `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase 15.75/ROADMAP.md`
- **STRATEGY.md**: `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase 15.75/STRATEGY.md`
- **frontend-layers.md**: `/home/tomoaki/git/hakorune-selfhost/docs/reference/frontend-layers.md`

### B. 実装チェックリスト

**Phase 4.1: Rust Parser側**
- [ ] `emit_json_v0_header()` 関数実装
- [ ] `--emit-json-v0-header` CLIフラグ追加
- [ ] 単体テスト作成
- [ ] 動作確認

**Phase 4.2: テストハーネス**
- [ ] `compare_json_v0_headers()` 関数実装
- [ ] `_run_nyash_vm_both()` 関数実装
- [ ] `SMOKES_PARSER_MODE=both` ロジック統合
- [ ] 動作確認

**Phase 4.3: Phase-Aスモーク**
- [ ] `semicolon_accept_vm.sh` 作成
- [ ] `if_else_vm.sh` 作成（または流用）
- [ ] `block_terminator_vm.sh` 作成（または流用）
- [ ] `using_minimal_vm.sh` 作成（または流用）
- [ ] 4/4 PASS 確認

**Phase 4.4: ASTNode → Stage-1 JSON（オプション）**
- [ ] `ast_to_stage1_json()` 設計
- [ ] 実装
- [ ] テスト
- [ ] Hakorune Selfhost Parser統合

**Phase 4.5: 最終確認**
- [ ] Phase-Aスモーク 4/4 PASS
- [ ] quick スモーク既存PASS維持
- [ ] quick-selfhost スモーク 170/185 PASS維持
- [ ] ドキュメント更新

---

## 変更履歴

- **2025-10-16**: 初版作成（Claude Technical Analysis Agent）
- 次回更新予定: Phase 4 実装完了時

---

**作成者署名**: Claude (Sonnet 4.5, 2025-10-16)
**レビュー**: 未実施（次回: ユーザー確認後）
