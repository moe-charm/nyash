# Phase 15.79 - Escape from Rust (Bootstrap Compiler)

**期間**: 2025-12-21 - 2026-02-28 (10週間)
**状態**: Planning
**前提**: Phase 15.77完了 (凍結EXE確定)

---

## 🎯 このフェーズで実現すること

**凍結EXEを使ってHakorune製コンパイラを起動 - 真の自己ホスト達成**

1. **Bootstrap Compiler完成**: 凍結EXE上で動くHakorune製コンパイラ
2. **3段階Bootstrap Chain確立**: Rust → Hakorune(v1) → Hakorune(v2)
3. **C Code Generator実装**: MIR → C コード生成（最小限）
4. **v1 == v2検証**: 2つのHakoruneコンパイラが同一出力を生成

---

## 💡 このフェーズの位置づけ

### Phase 15.77で確立した凍結EXEを活用

```
Phase 15.77（凍結EXE確定）✅
├── hako-frozen-v1.exe (724KB MSVC)
├── NyRT関数呼び出し可能
└── MIR JSON → .o → EXE 導線確立

Phase 15.79（Bootstrap実現）← 今ここ
├── Stage 1: Rust製コンパイラ（凍結）
├── Stage 2: Hakorune製コンパイラ v1
│   └── 凍結EXE上で実行
└── Stage 3: Hakorune製コンパイラ v2
    └── v1でコンパイル、v1と同一出力

Phase 15.80〜（完全自己ホスト）
└── Rust層完全削除
```

---

## 🏆 成功基準（DoD）

### 1️⃣ Bootstrap Chain動作確認

```bash
# Stage 1: Rust製コンパイラ（凍結EXE）
./hako-frozen-v1 program.hako --emit-mir program.mir.json

# Stage 2: Hakorune製コンパイラ v1
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input program.hako \
  --output program_v1.c

# Stage 3: v1でv2をビルド
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  --input apps/bootstrap-compiler/main.hako \
  --output bootstrap_v2.c

# 検証: v1 == v2
diff program_v1.c program_v2.c
# Expected: 同一
```

### 2️⃣ C Code Generator実装

- [ ] MIR JSON → C コード変換
- [ ] 16命令すべてサポート
- [ ] NyRT関数呼び出し対応
- [ ] スモークテスト: 10個 PASS

### 3️⃣ コンパイラパリティ検証

- [ ] 10個のテストプログラムで v1 == v2
- [ ] MIR出力の完全一致
- [ ] 実行結果の完全一致
- [ ] パフォーマンス: v2はv1の80%以上

### 4️⃣ ドキュメント整備

- [ ] Bootstrap手順書
- [ ] C Code Generator設計書
- [ ] トラブルシューティングガイド
- [ ] 完了報告書

---

## 📊 週次計画（Week 1-10）

### Week 1-2（2025-12-21 - 2026-01-03）設計・調査フェーズ

**目標**: 実装戦略確定、既存コード調査

#### タスク
- [ ] apps/selfhost-compiler/ 完全調査
- [ ] 凍結EXE制約分析（使用可能Box確認）
- [ ] C Code Generator設計書作成
- [ ] Bootstrap Chain詳細設計

#### 成果物
```
docs/development/roadmap/phases/phase-15.79/
├── DESIGN.md                  # 全体設計
├── C_CODE_GENERATOR.md        # C出力設計
├── BOOTSTRAP_CHAIN.md         # Bootstrap戦略
└── REUSABILITY_ANALYSIS.md    # コード再利用分析
```

### Week 3-4（2026-01-04 - 01-17）Parser適応

**目標**: 既存ParserBoxを凍結EXE環境で動作させる

#### タスク
- [ ] ParserBox依存関係整理
- [ ] 凍結EXE制約対応（File/JSON Box使用）
- [ ] テストケース作成（10個）
- [ ] 動作確認（AST JSON出力）

#### 成果物
```
apps/bootstrap-compiler/
├── parser/
│   ├── parser_box.hako       # 既存から移植
│   ├── lexer_box.hako
│   └── tests/                 # 10テストケース
└── README.md
```

### Week 5-6（2026-01-18 - 01-31）MIR Builder移植

**目標**: AST → MIR 変換実装

#### タスク
- [ ] MIR Builder基本構造移植
- [ ] 16命令すべてサポート
- [ ] CFG構築（Branch/Jump/Phi）
- [ ] スモークテスト: MIR JSON出力

#### 成果物
```
apps/bootstrap-compiler/
├── mir_builder/
│   ├── builder_box.hako       # MIR構築
│   ├── cfg_box.hako           # CFG管理
│   └── tests/                 # MIRテスト
└── INTERFACES.md              # Box間契約
```

### Week 7-8（2026-02-01 - 02-14）C Code Generator実装

**目標**: MIR → C コード変換

#### タスク
- [ ] C Code Emitter Box実装
- [ ] 16命令 → C変換
- [ ] NyRT関数呼び出し生成
- [ ] テスト: C → EXE → 実行確認

#### C出力例
```c
// MIR: const %0, 42
// C:
int64_t v0 = 42;

// MIR: binop %2 = add %0, %1
// C:
int64_t v2 = nyrt_int_add(v0, v1);

// MIR: boxcall %3 = %str.concat(%arg)
// C:
int64_t v3 = nyrt_boxcall(vstr, "concat", &varg, 1);

// MIR: ret %3
// C:
return v3;
```

#### 成果物
```
apps/bootstrap-compiler/
├── codegen/
│   ├── c_emitter_box.hako     # C出力
│   ├── c_runtime_box.hako     # NyRT呼び出し
│   └── tests/                 # C生成テスト
└── examples/
    ├── hello.c                # 生成例
    └── arithmetic.c
```

### Week 9（2026-02-15 - 02-21）Bootstrap Chain統合

**目標**: 3段階Bootstrap動作確認

#### タスク
- [ ] Stage 1 → Stage 2確認
- [ ] Stage 2 → Stage 3確認
- [ ] v1 == v2パリティ検証（10ケース）
- [ ] パフォーマンス計測

#### 検証スクリプト
```bash
# tools/bootstrap_verify.sh

#!/bin/bash
set -e

echo "Stage 1: Rust → v1"
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  test_cases/case1.hako -o case1_v1.c

echo "Stage 2: v1 → v2"
./hako-frozen-v1 apps/bootstrap-compiler/main.hako \
  apps/bootstrap-compiler/main.hako -o bootstrap_v2.c

echo "Stage 3: v2でcase1をコンパイル"
# (v2バイナリでcase1をコンパイル)

echo "Parity Check"
diff case1_v1.c case1_v2.c
echo "✅ PASS: v1 == v2"
```

### Week 10（2026-02-22 - 02-28）ドキュメント・レビュー

**目標**: ドキュメント整備、統合テスト、レビュー

#### タスク
- [ ] Bootstrap手順書作成
- [ ] トラブルシューティングFAQ
- [ ] 完了報告書作成
- [ ] ChatGPT/Claudeレビュー
- [ ] スモークテスト全緑確認

#### 成果物
```
docs/development/roadmap/phases/phase-15.79/
├── README.md                  # Phase概要（本文書）
├── COMPLETION_REPORT.md       # 完了報告
├── TROUBLESHOOTING.md         # FAQ
└── LESSONS_LEARNED.md         # 学び

docs/guides/
└── bootstrap-compiler-guide.md  # ユーザー向けガイド
```

---

## 🎯 実装戦略の決定

### オプションA: ミニマルParser新規実装

**アプローチ**:
- 最小限の構文のみサポート（return/if/loop/local/call）
- 1500行程度の新規実装
- 凍結EXE制約を最初から考慮

**メリット**:
- 軽量・シンプル
- 凍結EXE専用最適化可能
- デバッグが容易

**デメリット**:
- 機能が限定的
- apps/selfhost-compiler/の資産を活用できない
- 段階的拡張が必要

### オプションB: 既存selfhost-compiler再利用（推奨）⭐

**アプローチ**:
- apps/selfhost-compiler/ を基盤として活用
- 凍結EXE制約に合わせて調整
- 段階的に移植・適応

**メリット**:
- 2500行の既存実装を活用
- Parser/Emitter/MIR Builder既に存在
- 170テストケース継承可能
- 既に動作実績あり（quick-selfhost）

**デメリット**:
- using依存の整理が必要
- 一部Boxの凍結EXE制約対応が必要
- 初期の調査コストがやや高い

### 推奨: オプションB（既存コード再利用）

**理由**:
1. **実績**: apps/selfhost-compiler/ は既に170テストPASS
2. **資産活用**: 2500行の実装を活用、開発期間短縮
3. **安定性**: 既存テストケースが移行の安全網
4. **拡張性**: 段階的に機能追加可能

**制約対応**:
```
既存コード: apps/selfhost-compiler/
├── ParserBox           ✅ そのまま使用可能
├── EmitterBox          ✅ そのまま使用可能
├── MirEmitterBox       ⚠️ 軽微な調整必要
└── JsonProgramBox      ✅ そのまま使用可能

新規実装:
└── CCodeEmitterBox     ❌ 新規（Week 7-8）
```

---

## 📦 コード再利用分析

### apps/selfhost-compiler/ 構造

```
apps/selfhost-compiler/
├── boxes/                     # 36ファイル、2504行
│   ├── parser/
│   │   ├── parser_box.hako    237行 ✅ 再利用可能
│   │   ├── expr/              570行 ✅ 再利用可能
│   │   └── stmt/              521行 ✅ 再利用可能
│   ├── emitter_box.hako       10行  ✅ 再利用可能
│   ├── mir_emitter_box.hako   179行 ⚠️ 軽微調整
│   └── json_program_box.hako  264行 ✅ 再利用可能
├── builder/                   # SSA/最適化
│   └── ssa/                   200行 ⚠️ 必要に応じて
├── common/                    # ヘルパー
│   └── *_emit_box.hako        各50行 ✅ 再利用可能
└── tests/                     # テストケース
    └── stage1/                ✅ 継承

合計: ~2500行（再利用率: 90%以上）
```

### 凍結EXE利用可能Box

```
凍結EXE (frozen v1) 同梱Box:
├── String             ✅ 使用
├── Array              ✅ 使用
├── Map                ✅ 使用
├── Console (print)    ✅ 使用
├── Time (now_ms)      ✅ 使用
├── JSON (stringify)   ✅ 使用
└── File[min]          ✅ 使用

追加必要Box:
└── なし（既存Boxで十分）
```

### using依存の整理

```hakorune
// 既存（apps/selfhost-compiler/）
using "selfhost/shared/common/string_helpers.hako"
using "selfhost/shared/json/json_utils.hako"

// 移行後（apps/bootstrap-compiler/）
using "apps/bootstrap-compiler/common/string_helpers.hako"
using "apps/bootstrap-compiler/common/json_utils.hako"

// または凍結EXE同梱Boxで置き換え
// JSON操作 → 凍結EXE内のJSONBox使用
// String操作 → StringBox標準メソッド使用
```

---

## 🔄 Bootstrap Chain詳細設計

### Stage 1: Rust製コンパイラ（凍結）

**役割**: Hakorune v1コンパイラをビルド

```
入力: apps/bootstrap-compiler/**/*.hako
処理: Rust VM + Parser + MIR Builder
出力: bootstrap_v1.mir.json → bootstrap_v1.exe

特性:
- 凍結EXE (hako-frozen-v1.exe)
- 変更不可（Phase 15.77で確定）
- 信頼できる基準実装
```

### Stage 2: Hakorune製コンパイラ v1

**役割**: 任意のHakoruneプログラムをCコードへ変換

```
入力: program.hako
処理:
  1. Parser → AST JSON
  2. MIR Builder → MIR JSON
  3. C Code Generator → program.c
出力: program.c

実装:
- 言語: Hakorune
- 実行: 凍結EXE上
- コード: apps/bootstrap-compiler/
```

### Stage 3: Hakorune製コンパイラ v2

**役割**: v1と同一のコンパイラ（検証用）

```
入力: apps/bootstrap-compiler/**/*.hako
処理: v1でv2をコンパイル
出力: bootstrap_v2.c → bootstrap_v2.exe

検証:
- v1とv2が同一のCコードを生成
- v2もプログラムをコンパイル可能
- v2でv3をビルド → v2 == v3
```

### 検証フロー

```
      ┌─────────────┐
      │ program.hako│
      └──────┬──────┘
             │
    ┌────────┴────────┐
    │                 │
┌───▼────┐      ┌────▼────┐
│Stage 1 │      │Stage 2  │
│(Rust)  │      │(Hako v1)│
└───┬────┘      └────┬────┘
    │                │
    v                v
┌────────┐      ┌────────┐
│mir.json│      │prog.c  │
└────────┘      └────────┘

        ┌────────────┐
        │bootstrap   │
        │compiler    │
        │(v1 source) │
        └──────┬─────┘
               │
          ┌────▼────┐
          │Stage 2  │
          │(Hako v1)│
          └────┬────┘
               │
               v
          ┌────────┐
          │boot_v2.c│
          └────┬───┘
               │
          ┌────▼────┐
          │Stage 3  │
          │(Hako v2)│
          └────┬────┘
               │
          Verify: v1 == v2
```

---

## 🛠️ C Code Generator設計

### 基本方針

1. **最小限のC出力**: 可読性より正確性優先
2. **NyRT依存**: すべてのBox操作はNyRT経由
3. **16命令完全サポート**: MIR凍結セット準拠
4. **テスト駆動**: 各命令ごとにテストケース

### MIR → C 変換例

```c
// Header
#include <stdint.h>
extern int64_t nyash_box_from_i8_string(const char*);
extern int64_t nyash_string_concat_hh(int64_t, int64_t);
extern int64_t nyash_string_len_h(int64_t);

// Function
int64_t ny_main(void) {
  int64_t v0, v1, v2, v3;

  // const %0 = "Hello"
  v0 = nyash_box_from_i8_string("Hello");

  // const %1 = " World"
  v1 = nyash_box_from_i8_string(" World");

  // boxcall %2 = %0.concat(%1)
  v2 = nyash_string_concat_hh(v0, v1);

  // boxcall %3 = %2.len()
  v3 = nyash_string_len_h(v2);

  // ret %3
  return v3;
}
```

### 16命令 → C 変換表

| MIR命令 | C出力例 | 備考 |
|---------|---------|------|
| const | `v0 = 42;` | リテラル |
| binop | `v2 = v0 + v1;` | 演算子 |
| compare | `v3 = (v0 > v1);` | 比較 |
| jump | `goto bb1;` | 無条件分岐 |
| branch | `if (v0) goto bb_then; else goto bb_else;` | 条件分岐 |
| phi | `v5 = phi_v5;` | PHI変数 |
| ret | `return v3;` | 戻り値 |
| call | `v4 = ny_func();` | 関数呼び出し |
| boxcall | `v5 = nyrt_boxcall(v0, "method");` | メソッド |
| externcall | `v6 = nyrt_extern("iface.method");` | 外部関数 |
| load | `v7 = *ptr7;` | ロード |
| store | `*ptr8 = v8;` | ストア |
| copy | `v9 = v8;` | コピー |
| typeop | `v10 = nyrt_typecheck(v9);` | 型操作 |
| barrier | `nyrt_barrier();` | GCバリア |
| safepoint | `nyrt_safepoint();` | GCセーフポイント |

---

## ⚠️ リスク & 対策

### リスク1: 凍結EXEの制約

**問題**: 凍結EXEで使用可能なBoxが限定的

**対策**:
- 事前調査: 必要Boxリストアップ
- 代替実装: 不足機能はHakoruneで実装
- テスト: 各Box機能の動作確認

### リスク2: C Code Generatorの複雑さ

**問題**: 16命令すべてのC変換は非自明

**対策**:
- 段階的実装: 基本命令から順次
- テスト駆動: 各命令ごとにテスト
- 参考実装: 既存LLVM Backendを参考

### リスク3: Bootstrap Chain検証

**問題**: v1 == v2の検証が難しい

**対策**:
- 差分ツール: C出力の差分を詳細比較
- Golden Test: 既知のテストケースで検証
- 段階的検証: 小→大のプログラムで確認

### リスク4: パフォーマンス

**問題**: Hakorune製コンパイラが遅い可能性

**対策**:
- 測定: 各段階の実行時間計測
- 最適化: ボトルネック特定・改善
- 許容基準: v2はv1の80%以上なら合格

---

## 📚 関連リソース

### 前フェーズ
- [Phase 15.77 - 凍結EXE確定](../phase-15.77/)
- [Phase 15.76 - extern_c & Frozen Toolchain](../phase-15.76/)

### 参考実装
- [apps/selfhost-compiler/](../../../../apps/selfhost-compiler/) - 既存Hakoruneコンパイラ
- [src/llvm_py/](../../../../src/llvm_py/) - LLVM Backend参考

### 論文資料
- [Rapid Self-Hosting Paper](../../../../private/papers-active/rapid-selfhost-ai-collaboration/)

### 業界標準パターン
- **Rust**: stage0（凍結）→ stage1（ブートストラップ）→ stage2（検証）
- **Go**: Go 1.4 frozen → Go 1.5 self-hosted
- **OCaml**: ocamlc frozen → ocamlopt self-hosted

---

## 💬 開発体制

### 実装担当
- **ChatGPT**: C Code Generator実装主導
- **Claude**: Parser移植・レビュー
- **tomoaki**: 戦略判断・方向決定

### レビュー方針
- 各Week終了時にレビュー
- 凍結EXE動作を常に維持
- 問題発生時は即座にロールバック

---

## 🎉 成功後の世界

Phase 15.79完了後:

1. **完全自己ホスト達成**: HakoruneでHakoruneをビルド
2. **Rust依存最小化**: Rustは実行エンジンのみ（~200行）
3. **開発速度向上**: Hakorune単一コードベース
4. **拡張容易**: 言語機能追加がシンプルに

---

**作成日**: 2025-10-14
**Phase開始予定**: 2025-12-21（Phase 15.77完了後）
**想定期間**: 10週間
**戦略**: 既存selfhost-compiler再利用（オプションB）
