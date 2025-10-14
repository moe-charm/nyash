# Phase 15.75 Bootstrap戦略統合ドキュメント

**📚 [← ../INDEX.md に戻る](../INDEX.md)** | **🔒 安全性保証・Rollback戦略 + 🔧 ツールチェーン要件**

**最終更新**: 2025-10-14
**ステータス**: Phase 15.75計画段階
**目的**: MIRの疎結合を活用したゼロリスクBootstrap戦略とツールチェーン要件の完全定義

---

## 📋 目次

1. [Executive Summary](#-executive-summary)
2. [MIR疎結合の技術的仕組み](#-1-mir疎結合の技術的仕組み)
3. [Stage 0/1/2 並行運用戦略](#-2-stage-012-並行運用戦略)
4. [Rollback手順の詳細](#-3-rollback手順の詳細)
5. [パフォーマンス境界の詳細](#-4-パフォーマンス境界の詳細)
6. ["Chicken and Egg"問題の解消](#-5-chicken-and-egg問題の解消)
7. [Bootstrap戦略の実装手順](#-6-bootstrap戦略の実装手順)
8. [リスク評価の修正](#-7-リスク評価の修正)
9. [ツールチェーン要件](#-8-ツールチェーン要件)
10. [将来の開発環境](#-9-将来の開発環境)
11. [まとめ](#-10-まとめゼロリスクbootstrap戦略)

---

## 📋 Executive Summary

Phase 15.75のRust脱却は**不可逆的な書き換えではなく、実行バックエンドの追加**です。

### 🎯 **核心原則**
```
MIR = 疎結合ポイント
→ Stage 0/1/2 は並行運用可能
→ いつでもRollback可能
→ パフォーマンス境界はMIRレベル

hakorune.exe が自分自身をビルド・テスト・デバッグできる環境
= 外部依存最小化 + 段階的自己実装
```

### ⚠️ **従来の誤解 vs 正しい理解**

#### **❌ 誤解（従来の"コンパイラ書き換え"モデル）**
```
Stage 0 ──→ Stage 1 ──→ Stage 2 （一方通行）
(Rust)    (Hybrid)    (Hakorune)
           ↑
        戻れない危険地帯
```

#### **✅ 正しい理解（Hakorune MIRモデル）**
```
           MIR (疎結合)
              ↓
    ┌─────────┼─────────┐
    ↓         ↓         ↓
Stage 0   Stage 1   Stage 2
(Rust VM) (LLVM AOT)(Hako VM AOT)

← いつでも切替可能 →
```

**重要**: すべてのStageが**同じMIRを実行**するため、バックエンドはいつでも交換可能

### 🚀 **優先度定義**
- **P0 (Critical)**: なければ開発不可能（1週間以内に必要）
- **P1 (High)**: 生産性に大きく影響（1ヶ月以内）
- **P2 (Medium)**: あると便利（将来実装）
- **P3 (Low)**: Nice-to-have（需要次第）

---

## 🔧 1. MIR疎結合の技術的仕組み

### 1.1 MIRとは何か

**MIR (Mid-level Intermediate Representation)**: Hakoruneの中間表現

```json
{
  "functions": [{
    "name": "main",
    "blocks": [{
      "id": 0,
      "instructions": [
        {"op": "const", "dst": 1, "value": {"Int": 42}},
        {"op": "const", "dst": 2, "value": {"Int": 10}},
        {"op": "binop", "dst": 3, "kind": "Add", "lhs": 1, "rhs": 2}
      ],
      "terminator": {"op": "ret", "value": {"Register": 3}}
    }]
  }]
}
```

### 1.2 MIRが提供する疎結合

```
Source Code (.hako)
    ↓
[Parser]  ← Stage 0: Rust / Stage 1-2: Selfhost
    ↓
★ MIR (JSON) ★  ← 疎結合ポイント（安定インターフェース）
    ↓
[Execution Backend]
    ├─ Rust VM (Stage 0)
    ├─ LLVM AOT (Stage 1)
    └─ Hakorune VM AOT (Stage 2)
```

**重要な特性**:
1. **安定性**: MIRフォーマットは凍結済み（16命令で完結）
2. **独立性**: Parser と Execution は完全に分離
3. **交換可能性**: 同じMIRなら、どのバックエンドでも同じ結果

### 1.3 実行例：同じMIRを3つのバックエンドで実行

```bash
# MIR生成（どのStageでもOK）
./hakorune-stage0.exe --emit-mir-json test.mir.json test.hako

# Stage 0: Rust VM実行
./hakorune-stage0.exe test.mir.json
→ 出力: 52

# Stage 1: LLVM AOT実行
./hakorune-stage1.exe test.mir.json
→ 出力: 52

# Stage 2: Hakorune VM AOT実行
./hakorune-stage2.exe test.mir.json
→ 出力: 52
```

**結論**: MIRが正しければ、すべてのバックエンドで同じ結果

---

## 🚀 2. Stage 0/1/2 並行運用戦略

### 2.1 3つのStageの役割分担

| Stage | 実装 | 用途 | 開発速度 | 最適化 | デバッグ |
|-------|------|------|---------|--------|---------|
| **Stage 0** | Rust VM + Rust Parser | デバッグ・最終Rollback | 遅 | 低 | ★★★★★ |
| **Stage 1** | LLVM AOT + Selfhost Parser | 本番推奨・開発 | 中 | ★★★★★ | ★★★★ |
| **Stage 2** | Hakorune VM AOT + Selfhost Parser | 実験的・将来 | 高 | ★★★ | ★★ |

### 2.2 並行運用の具体例

#### **ケース1: 通常開発（Stage 1メイン）**
```bash
# コード編集
vim selfhost/compiler/parser.hako

# ビルド＆テスト（Stage 1）
./hakorune-stage1.exe build --target selfhost/
./hakorune-stage1.exe test --profile quick

# 問題なければcommit
git commit -m "feat: improve parser error messages"
```

#### **ケース2: デバッグ必要（Stage 0にRollback）**
```bash
# Stage 1でバグ発見
./hakorune-stage1.exe buggy_code.hako
→ Segmentation fault（原因不明）

# Stage 0で詳細トレース
HAKO_VM_TRACE="op=*;regs=1" ./hakorune-stage0.exe buggy_code.hako
→ [vm] bb=3 inst=7 boxcall recv=v%5(null) ← NULL参照発見！

# 修正後、Stage 1でテスト
./hakorune-stage1.exe buggy_code.hako
→ 正常動作
```

#### **ケース3: パフォーマンス検証（Stage 2実験）**
```bash
# Stage 1で基準測定
time ./hakorune-stage1.exe benchmark.hako
→ 1.2秒

# Stage 2で比較
time ./hakorune-stage2.exe benchmark.hako
→ 2.5秒（遅い）

# 結論: Stage 1を継続使用（問題なし）
```

### 2.3 hakorune.exe の維持戦略

#### **方針: 3つの hakorune 実行ラインを並行維持**

```
hakorune-selfhost/
├── bin/
│   ├── hakorune           ← `target/release/nyash` ラッパー（現行実体）
│   ├── hakorune-stage0    ← Rust VM（現行は `nyash` に委譲）
│   └── hakorune-stage1    ← LLVM ライン（現行は `nyash --backend llvm`）
├── src/                   ← Stage 0用 Rust コード（ブランチ保存）
├── selfhost/              ← Stage 1-2用 Hakoruneコード
└── hako.toml
```

#### **ビルド手順**

**Stage 0: Rust Bootstrap（初回のみ）**
```bash
# 初回ビルド
cargo build --release
cp target/release/hako bin/hakorune-stage0

# 以降は保存のみ（更新不要）
git checkout -b stage0-preserved
git commit -m "chore: preserve Stage 0 for debugging"
```

備考: 現時点の実体は `nyash`。`bin/` のラッパー経由で名称を `hakorune` に統一しつつ、徐々に本体も置換していく（段階移行）。

**Stage 1: LLVM ライン（メイン開発）**
```bash
# Selfhost Parserでビルド
./bin/hakorune-stage0 build --target selfhost/ --backend llvm
→ bin/hakorune-stage1 生成

# 以降は Stage 1で自己ビルド可能
./bin/hakorune-stage1 build --target selfhost/ --backend llvm
→ bin/hakorune-stage1 更新
```

**Stage 2: Hakorune VM AOT（将来）**
```bash
# Hakorune VM実装後
./bin/hakorune-stage1 build --target selfhost/ --backend hakorune-vm
→ bin/hakorune-stage2 生成

# Stage 2で自己ビルド可能
./bin/hakorune-stage2 build --target selfhost/ --backend hakorune-vm
→ bin/hakorune-stage2 更新
```

---

## 🔄 3. Rollback手順の詳細

### 3.1 Rollback Level 1: バックエンド切替（即座）

**状況**: Stage 1実行時に問題発生

```bash
# 問題発生
./hakorune-stage1.exe problem.hako
→ エラー（原因不明）

# 即座にStage 0へ切替
./hakorune-stage0.exe problem.hako
→ 正常動作 or 詳細エラーメッセージ
```

**所要時間**: 0秒（バイナリ切替のみ）

### 3.2 Rollback Level 2: Parser Rollback（MIR経由）

**状況**: Selfhost Parserにバグ（Stage 1-2共通）

```bash
# Selfhost Parserでパースエラー
./hakorune-stage1.exe new_syntax.hako
→ Parse error（Selfhost Parserのバグ）

# Stage 0（Rust Parser）で回避
./hakorune-stage0.exe new_syntax.hako
→ 正常動作

# Selfhost Parserのバグ修正
vim selfhost/compiler/parser.hako

# Stage 0でテスト（Rust VM）
./hakorune-stage0.exe test tests/test_parser.hako
→ PASS

# Stage 1で再ビルド
./hakorune-stage0.exe build --target selfhost/ --backend llvm
→ hakorune-stage1.exe 更新

# 修正確認
./hakorune-stage1.exe new_syntax.hako
→ 正常動作
```

**所要時間**: 5-10分（修正 + テスト）

### 3.3 Rollback Level 3: Full Rollback（Git branch切替）

**状況**: Phase 15.75全体を巻き戻し

```bash
# 現在: Phase 15.75完了後（Stage 1運用中）
git branch
* phase-15.75-complete
  stage0-preserved
  main

# 問題発生（致命的）
./hakorune-stage1.exe critical_app.hako
→ Runtime panic（修正不可能）

# Stage 0ブランチへ完全Rollback
git checkout stage0-preserved

# Rustビルド
cargo build --release

# Stage 0で動作確認
./target/release/hako critical_app.hako
→ 正常動作

# 問題修正後、再度Phase 15.75へ
git checkout phase-15.75-complete
```

**所要時間**: 10-30分（ブランチ切替 + ビルド）

---

## 📊 4. パフォーマンス境界の詳細

### 4.1 パフォーマンス劣化の境界

ユーザーの指摘：「パフォーマンス劣化のラインはそこ（MIRレベル）でキレます」

```
┌─────────────────────────┐
│ Source Code (.hako)     │
│                         │ ← 正しさの領域
│ ↓ Parser                │   （パフォーマンス影響なし）
├─────────────────────────┤
│ ★ MIR (JSON) ★          │ ← ★パフォーマンス境界★
├─────────────────────────┤
│ ↓ Execution Backend     │
│   - Rust VM (遅)        │ ← 最適化の領域
│   - LLVM AOT (高速)     │   （バックエンド選択で変動）
│   - Hakorune VM (中速)  │
└─────────────────────────┘
```

### 4.2 MIR品質 = パフォーマンス上限

**重要な意味**:
1. **Selfhost Parserが生成するMIR = Rust Parserと同等**
   → パフォーマンス上限は保証される

2. **実行バックエンドの違い = 最適化レベルの差**
   → Stage 0（Rust VM）: 最適化なし
   → Stage 1（LLVM AOT）: 最適化あり（推奨）
   → Stage 2（Hakorune VM）: 将来的に最適化予定

3. **MIRを改善すれば、すべてのStageで性能向上**
   ```bash
   # MIR最適化パス追加
   ./hakorune-stage1.exe --mir-opt-level 2 program.hako
   → すべてのバックエンドで高速化
   ```

### 4.3 実測パフォーマンス比較（予測）

| バックエンド | ビルド時間 | 実行速度 | デバッグ性 | 推奨用途 |
|-------------|-----------|---------|-----------|---------|
| Stage 0 (Rust VM) | 2秒 | 1.0x（基準） | ★★★★★ | デバッグ・検証 |
| Stage 1 (LLVM AOT) | 5秒 | 10-50x | ★★★★ | 本番・開発 |
| Stage 2 (Hako VM) | 3秒 | 2-5x（予測） | ★★ | 実験的 |

**結論**: Stage 1（LLVM AOT）が最速 → メイン開発推奨

---

## 🎯 5. "Chicken and Egg"問題の解消

### 5.1 従来の誤解

**誤解**: 「Stage 2に移行したら、Stage 1でビルドできなくなる（循環依存）」

```
❌ 誤ったモデル:
hakorune-stage1.exe（LLVM AOT）
    ↓ 削除
hakorune-stage2.exe（Hakorune VM）のみ
    ↓
Stage 2で問題発生 → Stage 1がない！（詰み）
```

### 5.2 正しい理解

**正しいモデル**: 「Stage 1-2は並行維持、削除しない」

```
✅ 正しいモデル:
hakorune-stage1.exe（LLVM AOT） ← 削除しない！
    ↓ 並行
hakorune-stage2.exe（Hakorune VM） ← 実験的に追加
    ↓
Stage 2で問題発生 → Stage 1に戻る（問題なし）
```

### 5.3 実際のビルドフロー

#### **Phase 5完了時（Stage 1確立）**
```bash
# Stage 0でStage 1ビルド（初回のみ）
./hakorune-stage0.exe build --backend llvm
→ hakorune-stage1.exe 生成

# Stage 1で自己ビルド可能
./hakorune-stage1.exe build --backend llvm
→ hakorune-stage1.exe 更新（自己再生産）
```

#### **Phase 8完了時（Stage 2追加）**
```bash
# Stage 1でStage 2ビルド
./hakorune-stage1.exe build --backend hakorune-vm
→ hakorune-stage2.exe 生成

# Stage 2で自己ビルド可能
./hakorune-stage2.exe build --backend hakorune-vm
→ hakorune-stage2.exe 更新

# Stage 1も維持
./hakorune-stage1.exe build --backend llvm
→ hakorune-stage1.exe 更新（並行維持）
```

**重要**: Stage 0/1/2すべてが自己再生産可能 → 循環依存なし

---

## 📋 6. Bootstrap戦略の実装手順

### 6.1 Phase 1-2: Stage 1確立

**目標**: hakorune-stage1.exe（LLVM AOT）を自己ビルド可能に

```bash
Week 1-2: Selfhost Parser実装
  ├─ Stage 0でテスト（Rust VM）
  ├─ MIR出力確認（Rust Parser と一致）
  └─ Stage 0でStage 1ビルド

Week 2終了時:
  ✅ hakorune-stage0.exe（保存）
  ✅ hakorune-stage1.exe（自己再生産可能）
```

**Rollback可能性**: いつでもStage 0に戻れる

### 6.2 Phase 3-5: Rust削減

**目標**: Rust 99,406行 → 10,400行

```bash
Week 3-4: Box移行 + Rust VM削除 + Runtime薄層化
  ├─ Stage 1で開発（メイン）
  ├─ Stage 0で検証（並行テスト）
  └─ スモークテスト全pass確認

Week 4終了時:
  ✅ hakorune-stage0.exe（ブランチ保存）
  ✅ hakorune-stage1.exe（10,400行 Rust kernel）
```

**Rollback可能性**: いつでもStage 0ブランチに戻れる

### 6.3 Phase 6-8: Stage 2実装（オプション）

**目標**: hakorune-stage2.exe（Hakorune VM AOT）を実験的に追加

```bash
Week 5-8: Hakorune VM実装（オプション）
  ├─ Stage 1で開発（メイン継続）
  ├─ Hakorune VM実装（新規）
  └─ Stage 2ビルド（実験的）

Week 8終了時:
  ✅ hakorune-stage0.exe（ブランチ保存）
  ✅ hakorune-stage1.exe（メイン開発）
  ✅ hakorune-stage2.exe（実験的）
```

**Rollback可能性**: Stage 1-2両方維持、いつでも切替可能

---

## 🚨 7. リスク評価の修正

### 7.1 従来のリスク評価（誤解ベース）

| リスク | レベル | 理由 |
|--------|--------|------|
| Rollback不可 | HIGH | Stage 2移行後は戻れない |
| 循環依存 | HIGH | Chicken and Egg問題 |
| デバッグ劣化 | MEDIUM | Rust VMがなくなる |

### 7.2 修正後のリスク評価（MIR疎結合ベース）

| リスク | レベル | 理由 | 対策 |
|--------|--------|------|------|
| Rollback不可 | **NONE** | MIR疎結合により常にRollback可能 | Stage 0保存 |
| 循環依存 | **NONE** | Stage 1-2並行維持 | 削除しない |
| デバッグ劣化 | **NONE** | Stage 0永久保存 | ブランチ保存 |
| MIR品質劣化 | **LOW** | Selfhost Parserのバグ | Stage 0で検証 |
| パフォーマンス劣化 | **LOW** | Stage 2が遅い可能性 | Stage 1継続使用 |

**結論**: MIR疎結合により、すべての致命的リスクが解消

---

## 🔧 8. ツールチェーン要件

Phase 15.75でRustを削減（99,406行 → 10,400行 or 3,500行）する際、**開発継続性**を保証するために必要なツールチェーンを定義します。

用語と実体（現時点の対応）
- 実行バイナリ実体: `target/release/nyash`
- 呼称/ラッパー: `bin/hakorune`, `bin/hakorune-stage0`, `bin/hakorune-stage1`
- Cargo alias（開発導線）: `build-rust`/`run-rust`（legacy）, `build-hako`/`run-hako`（plugin-only）
  - `.cargo/config.toml` に定義済み（Phase 15.75 名称分離）。
  - 将来は実体名も `hakorune` に移行（ラッパー→本体置換の段階移行）。

### 8.1 ビルドシステム（P0 - Critical）

#### HakoBuildBox - Build Orchestrator

**目的**: `cargo build` の代替として、Hakoruneプロジェクトをビルド

**必要機能**:
```hakorune
static box HakoBuildBox {
  // プロジェクト構造解析
  parse_project(root_path) -> ProjectBox

  // 依存関係解決
  resolve_dependencies(project) -> DependencyGraphBox

  // ビルド実行
  build(project, config) -> BuildResultBox

  // インクリメンタルビルド
  incremental_build(project, changed_files) -> BuildResultBox

  // クリーン
  clean(project) -> Result
}
```

**実装戦略**:

**Phase 5.1: 最小実装（1週間）**
```bash
# 目標: hakorune.exe 自己ビルド可能に
hako build.hako --target selfhost/compiler/pipeline_v2/

# 機能:
- ファイル列挙（*.hako）
- using依存解決（トポロジカルソート）
- MIR生成（hakorune.exe呼び出し）
- LLVM AOT実行（python llvm_builder.py）
```

**実装サイズ見積もり**: 500-800行（Hakorune）

**依存ライブラリ**:
- FileBox (plugin) - ファイルIO
- ArrayBox, MapBox (builtin) - データ構造
- StringBox (builtin) - パス操作

**代替案**:
```bash
# 暫定: Makefileで代用（Phase 1-4期間）
make -f hakorune.mk build
```

#### hako.toml パーサー（P0）

**目的**: プロジェクト設定読み込み

**必要機能**:
```hakorune
static box HakoTomlBox {
  parse(content) -> ConfigBox

  get_dependencies(config) -> ArrayBox
  get_build_settings(config) -> MapBox
  get_module_paths(config) -> ArrayBox
}
```

**実装戦略**:
```bash
# Phase 5.1: 最小TOMLサブセット
[package]
name = "hakorune-selfhost"
version = "0.1.0"

[dependencies]
selfhost = { path = "./selfhost" }

[build]
target = "llvm"
```

**実装サイズ見積もり**: 300-500行

**代替案**:
- JSON設定ファイル（hako.json） - TOMLより簡単
- Hakorune内埋め込み設定

### 8.2 コードエディタサポート（P1 - High）

#### HakoLspBox - Language Server Protocol

**目的**: VSCode/Vim等でコード補完・エラー表示

**必要機能**:
```hakorune
static box HakoLspBox {
  // 初期化
  initialize(root_uri) -> Result

  // 補完
  textDocument_completion(params) -> CompletionListBox

  // 定義ジャンプ
  textDocument_definition(params) -> LocationBox

  // エラー診断
  textDocument_publishDiagnostics(uri) -> DiagnosticsBox

  // ホバー情報
  textDocument_hover(params) -> HoverBox
}
```

**実装戦略**:

**Phase 6.1: 最小LSP（2週間）**
```
機能:
✅ 構文エラー表示（赤波線）
✅ using 解決（定義ジャンプ）
✅ 基本補完（static box メソッド名）

実装方法:
- JSON-RPC over stdio
- Parser再利用（--check-syntax mode）
- シンボルテーブル構築（using追跡）
```

**実装サイズ見積もり**: 1,200-2,000行

**優先度判断**:
- **P1** - エディタなしは生産性激減
- ただし Phase 1-5 は既存エディタで可能
- Phase 6 で実装開始でOK

**代替案**:
```bash
# 暫定: 構文チェックスクリプト
hako --check-syntax file.hako
```

#### HakoFmtBox - Code Formatter（P2）

**目的**: コード整形統一

**実装サイズ見積もり**: 400-800行

**優先度判断**:
- **P2** - 現状手動整形で対応可能
- 複数人開発開始後に必要

### 8.3 テストフレームワーク（P0 - Critical）

#### HakoTestBox - Unit Test Framework

**目的**: `cargo test` の代替

**必要機能**:
```hakorune
static box HakoTestBox {
  // テスト実行
  run_tests(test_dir) -> TestResultBox

  // アサーション
  assert_eq(actual, expected) -> Result
  assert_ne(actual, expected) -> Result
  assert(condition, message) -> Result

  // テストレポート
  report(results) -> StringBox
}
```

**実装戦略**:

**Phase 5.2: 最小実装（3日）**
```hakorune
// テストファイル: tests/test_parser.hako
using "hakorune/test" as Test

static box TestParser {
  test_basic_expression() {
    local result = Parser.parse("1 + 2")
    Test.assert_eq(result.is_Ok(), 1)
  }

  test_invalid_syntax() {
    local result = Parser.parse("1 +")
    Test.assert(result.is_Err())
  }
}
```

```bash
# 実行
hako test tests/

# 出力
Running test_basic_expression... PASS
Running test_invalid_syntax... PASS

Test Results: 2 passed, 0 failed
```

**実装サイズ見積もり**: 200-400行

**代替案**:
```bash
# 暫定: スモークテストスクリプト流用
bash tools/smokes/v2/run.sh --profile quick
```

### 8.4 デバッグツール（P0 - Critical）

#### MIRダンプツール（P0 - 実装済み✅）

**現状**: `--dump-mir` で実装済み

```bash
# 既存機能
hako --dump-mir program.hako
hako --emit-mir-json debug.json program.hako
```

**追加で必要な機能**:
```bash
# MIR最適化レベル確認
hako --dump-mir --mir-opt-level 2 program.hako

# MIR diff（最適化前後）
hako --dump-mir-diff program.hako
```

**実装サイズ見積もり**: 100-200行（追加分のみ）

#### VM Trace Debugger（P0 - 実装済み✅）

**現状**: 既に実装済み

```bash
# Rust VM トレース
HAKO_VM_TRACE="op=compare,binop;regs=1" hako program.hako

# Rust VM ステッパ
HAKO_VM_STEP=1 hako program.hako

# Selfhost Mini-VM トレース
# （MiniVmEntryBox.run_trace() 使用）
```

**追加で必要な機能**:
```bash
# ブレークポイント
HAKO_VM_BREAK="block=5,inst=3" hako program.hako

# レジスタウォッチ
HAKO_VM_WATCH="v%3,v%5" hako program.hako
```

**実装サイズ見積もり**: 200-300行（追加分のみ）

#### Performance Profiler（P2）

**目的**: ボトルネック特定

**実装サイズ見積もり**: 800-1,200行

**優先度判断**:
- **P2** - 最適化フェーズで必要（Phase 8以降）
- 暫定: `time` コマンドで代用

### 8.5 パッケージマネージャー（P2 - Medium）

#### HakoPkgBox - Package Manager

**目的**: 外部ライブラリ管理（`cargo install` 代替）

**実装サイズ見積もり**: 1,000-1,500行

**優先度判断**:
- **P2** - Phase 1-6 は `using` で十分
- 外部ライブラリエコシステム成長後に必要

**代替案**:
```bash
# Git submodule
git submodule add https://github.com/user/hako-json-parser libs/json_parser
```

### 8.6 コード解析ツール（P3 - Low）

#### HakoLintBox - Static Analyzer（P3）

**実装サイズ見積もり**: 600-1,000行

**優先度判断**:
- **P3** - Nice-to-have
- 現状: コードレビューで対応

#### HakoDocBox - Documentation Generator（P3）

**実装サイズ見積もり**: 800-1,200行

**優先度判断**:
- **P3** - 手動ドキュメント管理で十分（当面）

---

## 🚀 9. 将来の開発環境

### 9.1 ツールチェーン実装優先度マトリックス

| ツール | 優先度 | 実装時期 | 実装サイズ | 依存 | 代替案 |
|--------|--------|---------|-----------|------|--------|
| **HakoBuildBox** | P0 | Phase 5.1 (Week 3) | 500-800行 | FileBox | Makefile |
| **HakoTomlBox** | P0 | Phase 5.1 (Week 3) | 300-500行 | - | JSON設定 |
| **HakoTestBox** | P0 | Phase 5.2 (Week 3) | 200-400行 | - | Bashスクリプト |
| **MIRダンプ** | P0 | ✅実装済み | +100-200行 | - | - |
| **VM Trace** | P0 | ✅実装済み | +200-300行 | - | - |
| **HakoLspBox** | P1 | Phase 6.1 (Week 5-6) | 1,200-2,000行 | - | --check-syntax |
| **統合テスト** | P1 | Phase 7 (Week 7-8) | 600-1,000行 | - | 既存Bash |
| **HakoFmtBox** | P2 | Phase 7 (Week 8) | 400-800行 | - | 手動整形 |
| **HakoPkgBox** | P2 | Phase 7-8 (Week 9+) | 1,000-1,500行 | - | Git submodule |
| **Profiler** | P2 | Phase 8+ | 800-1,200行 | - | time コマンド |
| **HakoLintBox** | P3 | 将来 | 600-1,000行 | - | レビュー |
| **HakoDocBox** | P3 | 将来 | 800-1,200行 | - | 手動 |

### 9.2 実装工数見積もり

#### **P0ツール（必須 - 1ヶ月以内）**
```
HakoBuildBox:  5日（Week 3）
HakoTomlBox:   3日（Week 3）
HakoTestBox:   3日（Week 3）
MIRダンプ強化: 2日（Week 4）
VM Trace強化:  2日（Week 4）

合計: 15日 ≈ 3週間（Week 3-4）
```

#### **P1ツール（重要 - 2ヶ月以内）**
```
HakoLspBox:    10日（Week 5-6）
統合テスト:    5日（Week 7）

合計: 15日 ≈ 3週間（Week 5-7）
```

#### **P2ツール（便利 - 3ヶ月以内）**
```
HakoFmtBox:    5日（Week 8）
HakoPkgBox:    10日（Week 9-10）
Profiler:      8日（Week 11）

合計: 23日 ≈ 5週間（Week 8-12）
```

### 9.3 最小実装セット（Phase 15.75完了に必要）

```
✅ P0ツール（Week 3-4）
  - HakoBuildBox（500行）
  - HakoTomlBox（300行）
  - HakoTestBox（200行）
  - MIRダンプ（+100行）
  - VM Trace（+200行）

合計: 約1,300行 + 既存デバッグツール
期間: 2週間（Phase 5期間中に並行実装）
```

**結論**: Phase 15.75（1ヶ月計画）完了に必要なツールチェーンは**すべて実現可能**

### 9.4 段階的実装アプローチ

```
Week 1-2: Phase 1-2 (Selfhost Parser)
  → 既存 hakorune.exe 使用（ツール不要）

Week 3: Phase 5.1 (P0ツール実装)
  → HakoBuildBox（5日）
  → HakoTomlBox（3日）
  → HakoTestBox（3日）
  ※並行作業: Parser開発 + ツール開発

Week 4: Phase 3-4 (Box移行 + Rust VM削除)
  → 既存ツール使用
  → MIRダンプ強化（2日）

Week 5-6: Phase 6 (P1ツール実装)
  → HakoLspBox（10日）
  ※オプション: Phase 5完了後でもOK

Week 7+: P2ツール（将来実装）
  → 需要に応じて追加
```

### 9.5 ツール間依存関係

```
┌─────────────────┐
│ hakorune.exe    │ ← 最優先（常に動作保証）
└────────┬────────┘
         │
    ┌────┴────┐
    │ MIR     │ ← 疎結合ポイント
    └────┬────┘
         │
┌────────┴────────┐
│ HakoBuildBox    │ ← ビルド統括
│  ├─ HakoTomlBox │   設定読込
│  ├─ FileBox     │   ファイルIO
│  └─ hakorune.exe│   MIR生成呼出
└─────────────────┘
         │
    ┌────┴────┐
    │ HakoTestBox │ ← テスト実行
    │  └─ hakorune.exe
    └─────────┘
         │
    ┌────┴────┐
    │ HakoLspBox  │ ← エディタ支援
    │  └─ Parser  │   構文解析再利用
    └─────────┘
```

**重要**: すべてのツールが `hakorune.exe` と `MIR` に依存
→ MIRの安定性がツールチェーン全体の安定性を保証

### 9.6 ツールチェーン品質保証戦略

```hakorune
// tests/test_hako_build.hako
using "hakorune/test" as Test
using "hakorune/build" as Build

static box TestHakoBuild {
  test_simple_project() {
    local project = Build.parse_project("./test_fixtures/simple")
    Test.assert_eq(project.name(), "simple")

    local result = Build.build(project, null)
    Test.assert(result.is_Ok())
  }

  test_dependency_resolution() {
    local project = Build.parse_project("./test_fixtures/with_deps")
    local deps = Build.resolve_dependencies(project)
    Test.assert_eq(deps.size(), 3)
  }
}
```

---

## 🎯 10. まとめ：ゼロリスクBootstrap戦略

### 10.1 核心原則（再確認）

```
1. MIR = 疎結合ポイント
   → Parser と Execution Backend は完全分離

2. Stage 0/1/2 = 並行運用
   → いつでも切替可能、削除しない

3. hakorune.exe = 常に動作保証
   → 開発継続性100%維持

4. パフォーマンス境界 = MIRレベル
   → MIRが正しければパフォーマンス保証

5. Rollback = 即座に可能
   → バイナリ切替（0秒）〜 ブランチ切替（30分）

6. ツールチェーン = 段階的自己実装
   → P0（1ヶ月）→ P1（2ヶ月）→ P2（3ヶ月）
```

### 10.2 Phase 15.75の成功条件

```
✅ hakorune-stage1.exe が自己ビルド可能
✅ Rust 99,406行 → 10,400行削減
✅ スモークテスト全pass
✅ Stage 0/1並行維持
✅ いつでもRollback可能
✅ P0ツールチェーン完備（Build/Test/Debug）
```

### 10.3 最終確認

**Q: Phase 15.75完了後、問題が起きたらどうする？**
```bash
# A: 即座にStage 0へRollback（0秒）
./hakorune-stage0.exe problem.hako
```

**Q: Selfhost Parserにバグがあったらどうする？**
```bash
# A: Stage 0（Rust Parser）で開発継続（問題なし）
./hakorune-stage0.exe build --target selfhost/
```

**Q: Stage 2（Hakorune VM）が遅かったらどうする？**
```bash
# A: Stage 1（LLVM AOT）を継続使用（問題なし）
./hakorune-stage1.exe program.hako
```

**Q: ツールチェーンが間に合わなかったらどうする？**
```bash
# A: 代替案で継続（Makefile/Bash/既存ツール）
make -f hakorune.mk build
bash tools/smokes/v2/run.sh --profile quick
```

**結論**: すべてのケースでRollback可能 + ツールチェーン段階実装 → **ゼロリスクBootstrap戦略**

---

## 🔗 関連ドキュメント

- [Phase 15.75 INDEX](./INDEX.md)
- [Phase 15.75 Master Roadmap](./PHASE_15_75_RUST_FREE_ROADMAP.md)
- [Implementation Phases](./implementation_phases.md)
- [TODO Roadmap](./TODO_ROADMAP.md)
- [Timeline Analysis](./TIMELINE_ANALYSIS.md)
- [Velocity Analysis](./VELOCITY_ANALYSIS.md)

---

## 📝 変更履歴

- 2025-10-14: BOOTSTRAP_STRATEGY_MIR_ROLLBACK.md と TOOLCHAIN_REQUIREMENTS.md を統合
  - MIR疎結合によるRollback能力の詳細説明
  - Stage 0/1/2並行運用戦略の定義
  - "Chicken and Egg"問題の解消
  - パフォーマンス境界の明確化
  - ツールチェーン要件の完全定義
  - 段階的実装アプローチの統合
- 2025-10-13: 元の2ファイル作成（Claude Code + ユーザー指摘反映）
