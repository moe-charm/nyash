# Hakorune Mini-VM Migration Plan

**作成日**: 2025-10-08
**最終更新**: 2025-10-08（Strategy C採用）
**目的**: HakoruneでセルフホストMini-VMを実装し、MIR16凍結セット完全対応
**戦略**: **Strategy C（段階的統合）** - enum MVP → Mini-VM実装 → 完全enum化

---

## 🎯 0. 戦略的意思決定（重要！）

### Strategy C（段階的統合）採用の背景

**分析結果**: 10年間の技術的負債を考慮した結果、**Strategy C（段階的統合）**を採用

#### 採用理由
1. **長期コード品質が最優先**
   - セルフホストコードは10年以上メンテナンス対象
   - Rust VM → Hakorune Selfhost Compiler → MIR JSON → VM実行（Bootstrap Chain）
   - 技術的負債の複利的増加を回避（100 → 800-1000 debt points）

2. **現状の技術的負債**
   - 既存Mini-VM: 66箇所のnullチェック
   - 既存Mini-VM: 34箇所のエラーコード（-1/-2/0）
   - 品質スコア: 5/10（中〜高レベルの技術的負債）

3. **3つの戦略比較**（10年スパン）
   - **Strategy A（enum-first）**: 28-42人日、品質★★★★★、10年後最優
   - **Strategy B（Mini-VM-first）**: 13-20人日、品質★☆☆☆☆、10年後最悪
   - **Strategy C（段階統合）**: 25-35人日、品質★★★★☆、**バランス最良**

### Strategy C 実行計画

```
Step 1: enum MVP実装（3-5人日）
  ├─ Option<T> 基本実装
  ├─ Result<T,E> 基本実装
  └─ 基本パターンマッチング（@enum/@matchマクロなし）

Step 2: Mini-VM実装 with enum MVP（10-15人日）← 本ドキュメントのPhase 1-5
  ├─ **新規コードのみ** Option<T>/Result<T,E> 使用
  ├─ 既存コードは最小限の修正（リファクタリングしない）
  └─ 技術的負債の新規追加を防ぐ

Step 3: セルフホスト達成（6-7週間合計）
  └─ Phase 15.7完了

Step 4: enum完全実装（Phase 20、10-15人日）
  ├─ @enum/@matchマクロ実装
  ├─ 既存コード段階的リファクタリング
  │   ├─ 66箇所のnullチェック → Option<T>
  │   └─ 34箇所のエラーコード → Result<T,E>
  └─ 技術的負債の段階的解消
```

### ユーザーの決定的発言

> **「hakoruneセルフホスティング　コードは　綺麗にするのとても大切とおもいますにゃ　一番大本のrust vmからの立上げで　何かあったとき　ここからビルドする事も想定しますにゃ　全ての開発にかかわってきますにゃ」**

この発言により、短期的スピード（Strategy B）より長期的品質（Strategy C）を優先する戦略に転換。

---

## 🎯 1. プロジェクト概要（Step 2: Mini-VM実装部分）

### 目的
HakoruneでセルフホストMini-VMを実装し、MIR16凍結セット（基本演算5 + メモリ2 + 制御4 + 呼び出し1 + GC2 + 構造2）に完全対応する。

### スコープ
- **対象**: MIR16凍結命令セット（INSTRUCTION_SET.md準拠）
- **実装言語**: Hakorune（.hako）
- **参考実装**:
  - Rust VM（/src/backend/mir_interpreter/）- 本番品質アーキテクチャ
  - LLVM Python（/src/llvm_py/）- 8,370行、MIR16 100%実装済み
  - 既存Mini-VM（/apps/selfhost/vm/boxes/）- 1,831行、部分実装

### 成功基準
1. **機能**: MIR16命令すべて実行可能
2. **パリティ**: VM/LLVM/Mini-VMで同一出力
3. **自己実行**: Mini-VM自身をMini-VMで実行可能（セルフホスト）
4. **スモークテスト**: quick profileすべてPASS

---

## 🏗️ 2. 技術選択と設計方針

### 参考実装の選択
- **第1参考**: LLVM Python（8,370行）
  - **理由**: MIR16 100%実装済み、箱理論適用（650行→100行圧縮実績）
  - **採用部分**: 命令ハンドラ構造、PHI処理ロジック、制御フロー設計

- **第2参考**: Rust VM（/src/backend/mir_interpreter/）
  - **理由**: 本番品質、エラーハンドリング完備
  - **採用部分**: エラー処理戦略、契約（contracts）設計

- **第3参考**: 既存Mini-VM（1,831行）
  - **理由**: Hakorune実装実績あり
  - **採用部分**: JSON解析パターン、文字列操作ノウハウ

### アーキテクチャ設計

#### Core構造（Box-First原則）
```
MiniVmCore (メインエンジン)
  ├─ InstructionDispatcher (命令振り分け)
  ├─ ValueManager (値・レジスタ管理)
  └─ ControlFlowManager (制御フロー・PHI)

Handlers（命令ハンドラ群）
  ├─ ArithmeticHandler (Const, BinOp, UnaryOp, Compare, TypeOp)
  ├─ MemoryHandler (Load, Store)
  ├─ ControlFlowHandler (Branch, Jump, Return, Phi)
  ├─ CallHandler (MirCall統一)
  └─ GCHandler (Barrier, Safepoint)
```

#### データフロー
```
JSON MIR入力
  ↓ (JsonParserBox)
MIR構造体
  ↓ (InstructionDispatcher)
命令ハンドラ
  ↓ (ValueManager)
レジスタ更新
  ↓ (ControlFlowManager)
次ブロック決定
```

### 段階導入方針（80/20ルール適用）
- **Phase 1-3**: 基盤構築（Const/BinOp/Branch/Jump/Ret）→ 80%動作優先
- **Phase 4**: 呼び出し実装（MirCall）→ 失敗記録必須
- **Phase 5**: 残り命令（TypeOp/Load/Store等）→ 段階検証

---

## 📅 3. 実装フェーズ（80/20ルール適用）

### Phase 1: 基盤構築（3-5人日）
**目標**: 最小動作VM（Const + Ret + 基本演算）

#### タスク
- [ ] **JSON MIRパーサー基盤**（1日）
  - JsonCursorBox活用（既存実装流用）
  - block/instructions構造解析
  - 参考: `/apps/selfhost/common/json/json_cursor.hako`

- [ ] **命令ディスパッチャ基盤**（1日）
  - InstructionDispatcherBox実装
  - op文字列→ハンドラマッピング
  - 参考: LLVM Python `instruction_lower.py`

- [ ] **値管理（VMValue相当）**（1日）
  - ValueManagerBox実装
  - レジスタMap（`v%1` → 値）
  - 型判定（i64/string/box/void）

- [ ] **基本命令3つ**（1日）
  - Const（定数代入）
  - BinOp（加算のみ）
  - Ret（戻り値）

#### 成功基準
```hakorune
// test_phase1.hako
return 42  // → MIR: const v%1=42; ret v%1
// 期待: Mini-VM実行 → 42
```

#### 想定問題
1. **JSON解析のパフォーマンス**
   - 影響: 大規模MIRで遅延
   - 対策: Phase 1は無視、Phase 5で最適化検討

2. **Hakoruneの言語制約（enum未サポート）**
   - 影響: 高（命令種別の表現方法）
   - 対策: Box継承で代替（例: `InstructionBase` → `ConstInstruction`）

3. **レジスタ型の混在（i64/string/box）**
   - 影響: 中（型変換エラー頻発）
   - 対策: ValueManagerBoxで型タグ付き値管理

---

### Phase 2: 演算・比較（2-3人日）
**目標**: 算術・比較・型変換

#### タスク
- [ ] **BinOp完全対応**（1日）
  - 算術: Add, Sub, Mul, Div, Mod
  - ビット: And, Or, Xor, Shl, Shr
  - 参考: LLVM Python `binop.py`

- [ ] **UnaryOp**（0.5日）
  - Neg（負数）
  - Not（論理否定）

- [ ] **Compare**（1日）
  - 比較: Eq, Ne, Lt, Le, Gt, Ge
  - 参考: `/apps/selfhost/common/mini_vm_compare.hako`

- [ ] **TypeOp**（0.5日）
  - Cast（型変換）
  - TypeCheck（型判定）

#### 成功基準
```hakorune
// test_phase2.hako
return 1 + 2 * 3  // → 7
return 10 > 5     // → 1 (true)
```

#### 想定問題
1. **オーバーフロー処理**
   - Hakoruneの数値はi64、オーバーフロー動作未定義
   - 対策: Phase 2は無視、Phase 5でチェック追加

2. **比較演算の型混在（i64 vs string）**
   - 影響: 高（既存Mini-VMでバグ発生中、CURRENT_TASK.md L102）
   - 対策: CompareOpsBoxの既存実装を改善

---

### Phase 3: 制御フロー（3-4人日）
**目標**: 分岐・ループ・PHI

#### タスク
- [ ] **Branch（条件分岐）**（1日）
  - 条件評価（cond register）
  - then/else分岐
  - 参考: LLVM Python `controlflow/branch.py`

- [ ] **Jump（無条件ジャンプ）**（0.5日）
  - target block遷移

- [ ] **Phi（SSA値解決）**（2日）
  - **最重要・最難関タスク**
  - predecessor判定
  - incoming値選択
  - 参考: LLVM Python `phi_handler.py`（197行）、Rust VM `exec.rs:76`
  - 既知バグ: 到達不能predecessor混入（Phase 15.8で修正済み）

#### 成功基準
```hakorune
// test_phase3_if.hako
if (x > 0) return 1 else return 0
// 期待: x=5 → 1, x=-3 → 0

// test_phase3_loop.hako
local sum = 0
local i = 0
loop(i < 10) { sum = sum + i; i = i + 1 }
return sum  // → 45
```

#### 想定問題
1. **PHI処理の複雑さ**（最高リスク）
   - 影響: 極大（実装失敗でPhase 3全滅）
   - 対策:
     - 先にif-PHI実装（シンプル）
     - 次にloop-PHI（forward reference対応）
     - ChatGPT/LLVM Python実装を精読

2. **ループback-edge処理**
   - 影響: 高（forward reference未解決でクラッシュ）
   - 対策: incomplete_phis管理（LLVM Python方式）

3. **到達不能ブロック混入**
   - 影響: 中（既知バグ、Phase 15.8で修正済み）
   - 対策: is_block_reachable()チェック追加

---

### Phase 4: 呼び出し（3-5人日）
**目標**: 関数・メソッド・外部呼び出し

#### タスク
- [ ] **MirCall基盤**（1日）
  - Calleeパース（Global/ModuleFunction/Method/Extern）
  - 引数準備
  - 参考: Rust VM `handlers/calls/`

- [ ] **Global Call**（1日）
  - ビルトイン関数呼び出し
  - print(), _int_to_str()等

- [ ] **ModuleFunction Call**（1日）
  - 別関数呼び出し
  - スタック管理（再帰対応）

- [ ] **Method Call（BoxCall相当）**（1.5日）
  - レシーバ解決
  - メソッドディスパッチ
  - 参考: Rust VM `method_router.rs`

- [ ] **ExternCall**（0.5日）
  - 外部関数マッピング
  - WASI互換（fd_write等）

#### 成功基準
```hakorune
// test_phase4.hako
print(42)              // → "42\n"
return factorial(5)    // → 120 (再帰呼び出し)
local arr = new ArrayBox()
arr.push(10)           // → BoxCall
return arr.length()    // → 1
```

#### 想定問題（失敗記録最重要！）
1. **using経由のstatic box呼び出しで引数null問題**（既知バグ）
   - 影響: 極大（CURRENT_TASK.md L107-109で報告済み）
   - 症状: `using "..." as Box; Box.method(param)` → param消失
   - 対策:
     - Rust VM calls/function.rs（ModuleFunction経路）精読
     - 引数転送の最小ログで調査
     - **Phase 4前に根本修正必須**

2. **再帰呼び出しでスタックオーバーフロー**
   - 影響: 高（深い再帰でクラッシュ）
   - 対策: 再帰深さカウンタ（max 1000回）

3. **メソッド解決の曖昧性**
   - 影響: 中（同名メソッド混在でエラー）
   - 対策: certainty判定（Rust VM方式）

---

### Phase 5: 残り命令 + 最適化（2-3人日）
**目標**: MIR16完全対応 + 安定化

#### タスク
- [ ] **Load/Store**（1日）
  - メモリアクセス
  - ローカル変数

- [ ] **Copy**（0.5日）
  - レジスタコピー

- [ ] **Barrier/Safepoint**（0.5日）
  - stub実装（GC統合はPhase 16以降）

- [ ] **パフォーマンス改善**（1日）
  - JSON解析キャッシュ
  - 頻出パターン高速化

#### 成功基準
```bash
# MIR16全命令のスモークテスト通過
tools/smokes/v2/run.sh --profile quick
# 期待: 全PASS（Mini-VMバックエンド使用）
```

#### 想定問題
1. **JSON解析のボトルネック**
   - 影響: 中（大規模MIRで10倍遅延）
   - 対策: プロファイリング→ホットパス特定→最適化

2. **メモリリーク（GC未実装）**
   - 影響: 低（短時間実行のみ）
   - 対策: Phase 5はスコープ外、ドキュメントに記載

---

## ⚠️ 4. リスク管理

### 🔴 重大リスク

| リスク | 確率 | 影響 | 対策 |
|-------|------|------|------|
| **PHI処理の実装失敗** | 高 | 極大 | ① LLVM Python実装精読<br>② if-PHIから段階実装<br>③ ChatGPT支援依頼 |
| **using経由引数null問題** | 高 | 極大 | ① Phase 4前にRust VM修正<br>② 最小再現コード作成<br>③ 根本原因特定 |
| **Hakorune言語制約（enum未サポート）** | 確定 | 高 | ① Box継承で代替<br>② InstructionBaseクラス設計<br>③ 動的ディスパッチ |

### 🟡 中程度リスク

| リスク | 確率 | 影響 | 対策 |
|-------|------|------|------|
| **JSON解析パフォーマンス** | 中 | 中 | ① Phase 1-4は無視<br>② Phase 5でプロファイリング<br>③ 必要なら部分的にRust実装 |
| **再帰呼び出しスタックオーバーフロー** | 中 | 中 | ① 深さカウンタ（max 1000）<br>② エラーメッセージ改善 |
| **比較演算の型混在バグ** | 中 | 中 | ① CompareOpsBox改善<br>② 型タグ厳格化 |

### 🟢 軽微リスク

| リスク | 確率 | 影響 | 対策 |
|-------|------|------|------|
| **メモリリーク（GC未実装）** | 確定 | 低 | ① ドキュメントに記載<br>② Phase 16以降対応 |
| **オーバーフロー未対応** | 低 | 低 | ① Phase 5でチェック追加<br>② テストケース整備 |

---

## 📦 5. 成果物

### コード（/apps/selfhost/vm/mini_vm_v2/）

#### コア実装
- `mini_vm_core.hako` - メインエンジン（300-400行見込み）
- `instruction_dispatcher.hako` - 命令振り分け（100-150行）
- `value_manager.hako` - 値・レジスタ管理（150-200行）
- `control_flow_manager.hako` - 制御フロー・PHI（200-250行）

#### ハンドラ群（/handlers/）
- `arithmetic_handler.hako` - Const/BinOp/UnaryOp/Compare/TypeOp（150行）
- `memory_handler.hako` - Load/Store（80行）
- `control_flow_handler.hako` - Branch/Jump/Return/Phi（150行）
- `call_handler.hako` - MirCall統一（200行）
- `gc_handler.hako` - Barrier/Safepoint stub（50行）

#### テスト・ツール
- `tests/phase1_basic.hako` - 基本演算テスト
- `tests/phase2_arithmetic.hako` - 算術・比較テスト
- `tests/phase3_control.hako` - 制御フローテスト
- `tests/phase4_call.hako` - 呼び出しテスト
- `tests/phase5_full.hako` - MIR16全命令テスト
- `tools/mini_vm_runner.hako` - 実行ラッパー

### ドキュメント

#### 実行計画・進捗
- `docs/development/current/main/mini_vm_migration_plan.md` - **本ドキュメント**
- `docs/development/current/main/mini_vm_progress.md` - 進捗記録（日次更新）
- `docs/development/current/main/mini_vm_lessons.md` - **失敗・学び記録**（最重要）

#### 設計書
- `docs/architecture/mini_vm_v2_design.md` - アーキテクチャ詳細
- `docs/guides/mini_vm_debugging.md` - デバッグガイド
- `docs/guides/mini_vm_porting.md` - 移植ノウハウ（Rust VM/LLVM Python比較）

---

## 🎯 6. マイルストーン

### Strategy C 全体スケジュール

| Step | Phase | 期間 | 成果物 | リスク |
|------|-------|------|--------|-------|
| **Step 1** | enum MVP | 3-5人日 | Option<T>/Result<T,E> 基本実装 | 🟡 中 |
| **Step 2** | Phase 1-5 | 10-15人日 | Mini-VM（MIR16完全対応） | 🔴 高 |
| **Step 3** | 統合・検証 | 3-5人日 | セルフホスト達成 | 🟡 中 |
| **Step 4** | Phase 20 | 10-15人日 | enum完全実装・リファクタ | 🟢 低 |

**合計**: 25-35人日（5-7週間、1人体制）

### Step 2（Mini-VM実装）詳細マイルストーン

| Phase | 期間 | 成果物 | 成功基準 | リスク |
|-------|------|--------|---------|-------|
| **Phase 1** | 2-3日 | 基盤VM | `return 42` 実行成功 | 🟡 中（enum MVP活用） |
| **Phase 2** | 2-3日 | 演算VM | `1+2*3` `10>5` 実行成功 | 🟢 低 |
| **Phase 3** | 3-4日 | 制御VM | if/loop-PHI実行成功 | 🔴 高（PHI実装） |
| **Phase 4** | 3-5日 | 呼び出しVM | `print(x)` `factorial(5)` 実行成功 | 🔴 高（引数null問題） |
| **Phase 5** | 2-3日 | 完全VM | MIR16全対応、スモークPASS | 🟡 中（パフォーマンス） |

**Step 2合計**: 10-15人日（2-3週間、1人体制）

**注**: Phase 1-5は**enum MVP実装後**に開始。新規コードはOption<T>/Result<T,E>を積極活用し、技術的負債の新規追加を防ぐ。

---

## 📊 7. 進捗追跡

### Phase完了チェックリスト

#### Phase 1: 基盤構築 ⏸️
- [ ] JsonCursorBox統合完了
- [ ] InstructionDispatcherBox実装
- [ ] ValueManagerBox実装
- [ ] Const/BinOp(Add)/Ret実装
- [ ] test_phase1.hako PASS
- [ ] **失敗記録**: mini_vm_lessons.md更新

#### Phase 2: 演算・比較 ⏸️
- [ ] BinOp全演算実装（9種）
- [ ] UnaryOp実装（Neg/Not）
- [ ] Compare実装（6種）
- [ ] TypeOp実装（Cast/TypeCheck）
- [ ] test_phase2.hako PASS
- [ ] **失敗記録**: 型混在バグ対応記録

#### Phase 3: 制御フロー ⏸️
- [ ] Branch実装
- [ ] Jump実装
- [ ] if-PHI実装（シンプル）
- [ ] loop-PHI実装（forward reference）
- [ ] test_phase3_if.hako PASS
- [ ] test_phase3_loop.hako PASS
- [ ] **失敗記録**: PHI実装試行錯誤記録

#### Phase 4: 呼び出し ⏸️
- [ ] **事前**: using引数null問題修正完了
- [ ] MirCall基盤実装
- [ ] Global Call実装
- [ ] ModuleFunction Call実装
- [ ] Method Call実装
- [ ] ExternCall実装
- [ ] test_phase4.hako PASS
- [ ] **失敗記録**: 呼び出し実装の失敗記録

#### Phase 5: 完全対応 ⏸️
- [ ] Load/Store実装
- [ ] Copy実装
- [ ] Barrier/Safepoint stub実装
- [ ] パフォーマンス改善
- [ ] tools/smokes/v2/run.sh --profile quick PASS
- [ ] **失敗記録**: 最適化試行錯誤記録

### 日次進捗フォーマット（mini_vm_progress.md）

```markdown
## 2025-10-XX (Phase X, Day Y)

### ✅ 完了
- タスク1
- タスク2

### ❌ 失敗・問題
- 問題1: [詳細]
  - 原因: [根本原因]
  - 影響: [どれくらい深刻か]
  - 対策: [次どうするか]

### ⏸️ ブロッカー
- ブロッカー1: [何が止まっているか]

### 📊 統計
- 実装行数: XXX行
- テスト通過: X/Y
- 所要時間: X時間
```

---

## 🚨 8. 失敗記録の重要性（最優先）

### プログラム開発では失敗報告が一番大事

**成功報告より失敗報告が重要な理由**:
- ✅ 失敗は**次の改善の種**（成功は既に終わったこと）
- ✅ 失敗は**学習の最大の機会**（同じミスを繰り返さない）
- ✅ 失敗は**システムの脆弱性を教えてくれる**（本番障害を未然に防ぐ）
- ✅ 失敗は**見積もり精度を上げる**（楽観的予測を修正）

### 報告すべき失敗の種類

#### 1️⃣ 実行失敗・テスト失敗
```
❌ Phase 3実装完了したがテスト失敗（loop-PHI）
❌ if-PHI実装4回試行、すべて失敗
❌ 動作確認できていない状態で次Phase進行提案
```

#### 2️⃣ 見積もりの失敗
```
当初見積もり: Phase 3は2日
実際の結果:   4日（見積もりの200%）

原因: PHI forward reference処理を考慮していなかった
```

#### 3️⃣ 設計判断の失敗
```
判断: InstructionをMapで管理
結果: 順序保証されずバグ → Array変更（+2日）

原因: Hakoruneのコレクション特性を調査不足
```

#### 4️⃣ 理解不足・調査不足
```
問題: MirCall統一の仕様が不明
対応: 仮実装で進める → 後で全書き直し（+3日）
根本原因: **INSTRUCTION_SET.md精読していなかった**
```

### 客観的な失敗報告フォーマット（mini_vm_lessons.md）

```markdown
## ❌ Phase X.X の問題点・失敗

### 1️⃣ **[失敗の種類]**
**問題**: [何が起きたか]
**期待**: [何を期待していたか]
**実際**: [実際にどうなったか]
**原因**: [なぜ失敗したか]
**影響**: [どのくらい深刻か（遅延日数等）]
**学び**: [次回どう避けるか]

### 2️⃣ **[次の失敗]**
...
```

---

## 🔄 9. 実装戦略（80/20ルール詳細）

### 80/20ルールの実践方法

#### Phase 1の例
**80%（動くもの）**:
- JSON解析: 最低限のblock/instructions読み取り
- ディスパッチ: Const/BinOp/Retのみ対応
- エラー処理: panic（詳細メッセージなし）

**20%（改善候補、Phase 5以降）**:
- JSON解析: キャッシュ・最適化
- ディスパッチ: 動的最適化
- エラー処理: 詳細スタックトレース

#### Phase 3の例
**80%（動くもの）**:
- if-PHI: 2 predecessorのみ対応
- loop-PHI: シンプルwhile（単一変数）のみ
- エラー: 到達不能predecessor検出（簡易）

**20%（改善候補）**:
- 複雑PHI: 3+ predecessors対応
- ネストループ: 2重PHI対応
- エラー: 完全な到達性解析

### 失敗記録は100%必須
**80%で完了とするのは「機能」だけです。失敗・問題点の記録は100%必須です。**

---

## 📚 10. 参考資料

### 内部ドキュメント
- [MIR命令セット](../../../reference/mir/INSTRUCTION_SET.md) - **最重要**
- [Phase 15.8 WASM実装](../../roadmap/phases/phase-15.8/README.md) - PHI実装参考
- [LLVM Python実装](../../../../src/llvm_py/README.md) - 8,370行、MIR16 100%実装
- [Rust VM実装](../../../../src/backend/mir_interpreter/README.md) - 本番品質参考
- [既存Mini-VM](../../../../apps/selfhost/vm/boxes/README.md) - Hakorune実装参考

### 外部リソース
- [Box-First原則](../../../guides/box-first-principle.md)
- [80/20開発ルール](../../../../CLAUDE.md#L85-L95)
- [失敗報告の重要性](../../../../CLAUDE.md#L265-L347)

---

## 🚀 11. 次のアクション（Strategy C実行）

### ⚠️ 重要: 実行順序

**Strategy C により、実行順序は以下の通り**:
1. **Step 1: enum MVP実装**（3-5人日）← 最優先
2. Step 2: Mini-VM実装（10-15人日）← 本ドキュメントのPhase 1-5
3. Step 3: セルフホスト達成
4. Step 4: enum完全実装（Phase 20）

### Step 1: enum MVP実装（最優先）

#### 準備（0.5日）
1. **環境確認**
   - [ ] Hakoruneビルド確認（`cargo build --release`）
   - [ ] スモークテスト実行（`tools/smokes/v2/run.sh --profile quick`）

2. **ドキュメント精読**
   - [ ] Phase 20 VariantBox設計書（`docs/development/roadmap/phases/phase-20-variant-box/DESIGN.md`）
   - [ ] 既存ResultBox実装（`apps/selfhost/vm/boxes/result_box.hako`、34行）
   - [ ] 言語仕様確認（Box継承、birth lifecycle）

3. **失敗記録準備**
   - [ ] enum_mvp_progress.md作成（日次更新用）
   - [ ] enum_mvp_lessons.md作成（失敗記録用）

#### 実装（3-5人日）
1. **Day 1-2: Option<T> 基本実装**
   ```hakorune
   box OptionBox {
       is_some: BoolBox
       value: Box  // null または実値

       birth() {
           me.is_some = new BoolBox(0)  // None
           me.value = null
       }

       some(v) {
           me.is_some = new BoolBox(1)
           me.value = v
       }

       is_some() { return me.is_some }
       is_none() { return !me.is_some }
       unwrap() {
           if !me.is_some {
               panic("Called unwrap on None")
           }
           return me.value
       }
   }
   ```

2. **Day 2-3: Result<T,E> 基本実装**
   ```hakorune
   box ResultBox {
       is_ok: BoolBox
       value: Box
       error: Box

       ok(v) {
           me.is_ok = new BoolBox(1)
           me.value = v
           me.error = null
       }

       err(e) {
           me.is_ok = new BoolBox(0)
           me.value = null
           me.error = e
       }

       is_ok() { return me.is_ok }
       is_err() { return !me.is_ok }
       unwrap() {
           if !me.is_ok {
               panic("Called unwrap on Err: " + me.error)
           }
           return me.value
       }
   }
   ```

3. **Day 4-5: テスト・統合**
   - [ ] test_option_basic.hako（10パターン）
   - [ ] test_result_basic.hako（10パターン）
   - [ ] スモークテスト追加
   - [ ] ドキュメント作成（使用ガイド）

#### 成功基準（Step 1完了）
- [ ] Option<T> 基本操作すべて動作
- [ ] Result<T,E> 基本操作すべて動作
- [ ] スモークテスト PASS
- [ ] Mini-VMコードで使用可能な状態

---

### Step 2: Mini-VM実装（enum MVP完了後）

#### Phase 1開始前（準備、0.5日）
1. **ドキュメント精読**
   - [ ] INSTRUCTION_SET.md完全理解
   - [ ] LLVM Python phi_handler.py精読（197行）
   - [ ] Rust VM exec.rs PHI処理精読（L76-90）

2. **失敗記録準備**
   - [ ] mini_vm_progress.md作成（日次更新用）
   - [ ] mini_vm_lessons.md作成（失敗記録用）
   - [ ] テンプレート準備（上記フォーマット）

#### Phase 1開始（Day 1）
1. **JsonCursorBox統合** → 4時間（**Result<T,E>活用**）
2. **InstructionDispatcherBox実装** → 3時間（**Option<T>活用**）
3. **test_phase1.hako作成・実行** → 1時間
4. **失敗記録更新** → 必須（所要時間問わず）

**重要**: 新規コードは**必ず**Option<T>/Result<T,E>を使用。nullチェック・エラーコード（-1/-2/0）の新規追加を禁止。

---

## 📝 12. 補足事項

### Hakoruneの言語制約対応

#### enum未サポート → Box継承で代替
```hakorune
// ❌ Hakoruneで不可能
enum Instruction { Const, BinOp, ... }

// ✅ Box継承で実現
box InstructionBase { op: StringBox }
box ConstInstruction from InstructionBase { value: IntegerBox }
box BinOpInstruction from InstructionBase { kind: StringBox, lhs: IntegerBox, rhs: IntegerBox }
```

#### 動的ディスパッチ
```hakorune
// InstructionDispatcherBox
dispatch(inst_json) {
  local op = JsonFragBox.get_string(inst_json, "op")
  if op == "const" { return me.handle_const(inst_json) }
  if op == "binop" { return me.handle_binop(inst_json) }
  // ...
}
```

### JSON解析パターン（既存実装活用）
```hakorune
// 高速パターン（JsonCursorBox）
using "apps/selfhost/common/json/json_cursor.hako" as JsonCursorBox

parse_block(mir_json, block_id) {
  local key = "\"id\":" + StringHelpers.int_to_str(block_id)
  local pos = mir_json.indexOf(key)
  local inst_start = JsonCursorBox.seek_array_start(mir_json, pos, "instructions")
  local inst_end = JsonCursorBox.seek_array_end(mir_json, inst_start)
  return mir_json.substring(inst_start, inst_end)
}
```

### デバッグ戦略
```bash
# 1. MIR出力確認
./target/release/hako --dump-mir test.hako

# 2. Mini-VM実行（トレースON）
HAKO_MINI_VM_TRACE=1 ./target/release/hako --backend mini-vm test.hako

# 3. Rust VM比較
./target/release/hako --backend vm test.hako

# 4. 差分確認
diff <(HAKO_MINI_VM_TRACE=1 ./hako --backend mini-vm test.hako 2>&1) \
     <(HAKO_VM_TRACE=1 ./hako --backend vm test.hako 2>&1)
```

---

## ✅ 13. 計画書レビューチェックリスト

### Strategy C 採用版（2025-10-08更新）

- [x] **戦略的意思決定** - Strategy C（段階的統合）採用、10年視点での品質優先
- [x] **長期コード品質** - 技術的負債分析（66 null, 34 error codes）、10年累積モデル
- [x] **実行順序明確** - Step 1（enum MVP）→ Step 2（Mini-VM）→ Step 3（統合）→ Step 4（完全enum化）
- [x] **目的・スコープ明確** - MIR16凍結セット完全対応
- [x] **参考実装選択根拠** - LLVM Python（100%実装）+ Rust VM（品質）+ 既存Mini-VM（実績）
- [x] **アーキテクチャ設計** - Box-First原則、5層構造
- [x] **Phase分割合理的** - 5 Phase、各3-5日、段階検証
- [x] **リスク特定・対策** - 重大3件（PHI/引数null/enum制約）、対策明記
- [x] **成功基準具体的** - 各Phase実行可能コード例あり
- [x] **失敗記録重視** - セクション8で詳細、フォーマット提供
- [x] **80/20ルール適用** - 各Phase「動くもの」80%優先、20%は後回し
- [x] **マイルストーン現実的** - Step 1: 3-5人日、Step 2: 10-15人日、合計25-35人日
- [x] **次アクション明確** - Step 1（enum MVP）実装詳細、Step 2準備リスト

---

**作成完了**: 2025-10-08
**戦略更新**: 2025-10-08（Strategy C採用）
**実行可能性**: 高（LLVM Python実装100%完了済み、Rust VM参考可能）
**リスク管理**: 重大リスク3件特定、対策明記
**失敗記録体制**: フォーマット・ファイル準備完了
**長期品質**: 10年視点での技術的負債管理体制確立

**🎯 次のアクション: Step 1（enum MVP実装）開始準備！**

---

## 📊 14. Strategy C 技術的負債管理

### 現状の技術的負債（Phase 15.7時点）

**既存Mini-VM（2,379行、38ファイル）**:
- null チェック: 66箇所
- error コード（-1/-2/0）: 34箇所
- TODO/FIXME: 0箇所（マーカーなし）
- **品質スコア**: 5/10（中〜高レベルの技術的負債）

### 10年累積モデル

```
Phase 15.7（現在）: 100 debt points
  ↓
Phase 20（enum未導入）: 200 points（1.5倍成長 + 新規複雑性）
  ↓
Phase 25（型パス導入）: 400 points
  ↓
10年後: 800-1000 points

リファクタリングコスト: 50-100人日（遅延するほど増大）
```

### Strategy C による負債管理

**Step 1（enum MVP）**: 新規ツール提供
- Option<T>/Result<T,E> 基本実装
- 技術的負債: +10 points（新規実装のみ）

**Step 2（Mini-VM）**: 負債増加の抑制
- 新規コードのみ Option<T>/Result<T,E> 使用
- 既存コードは最小限修正
- 技術的負債: +20 points（新規null/error禁止により50%削減）

**Step 3（セルフホスト）**: 完了優先
- 技術的負債: +10 points（統合作業のみ）

**Step 4（完全enum化）**: 段階的解消
- 66箇所のnull → Option<T>
- 34箇所のerror → Result<T,E>
- 技術的負債: -60 points（段階的リファクタ）

**10年後の予測**:
- Strategy B（Mini-VM先行）: 800-1000 points
- **Strategy C（段階統合）**: 200-300 points（**70%削減**）

### ユーザーの判断

> **「全ての開発にかかわってきますにゃ」**

この発言により、短期的スピード（13-20人日）より長期的品質（25-35人日、但し10年で50-100人日節約）を優先する戦略決定。

**Bootstrap Chain の信頼性 = プロジェクトの10年生存率**
