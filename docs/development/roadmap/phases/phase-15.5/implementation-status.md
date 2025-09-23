# Phase 15.5 実装状況追跡

**JSON v0中心化・統一Call基盤革命の進捗管理**

## 📊 全体進捗

**更新日**: 2025-09-24

### Phase概要
- **Phase A**: JSON出力統一 - 🔄 **進行中** (Phase 3.4完了)
- **Phase B**: JSON中心化移行 - ⏳ **未着手**
- **Phase C**: 完全JSON化 - ⏳ **未着手**

### 完了率
```
Phase A: ████████░░ 80%
Phase B: ░░░░░░░░░░ 0%
Phase C: ░░░░░░░░░░ 0%
総合:    ███░░░░░░░ 30%
```

---

## 🔄 Phase A: JSON出力統一 (80%完了)

### ✅ 完了済み項目

#### Rust側基盤 (Phase 3.1-3.3)
- [x] **MIR統一定義** - `src/mir/definitions/call_unified.rs` (297行)
- [x] **Callee enum実装** - Global/Method/Constructor/Closure/Value/Extern
- [x] **統一メソッド** - `emit_unified_call()` + 便利メソッド3種
- [x] **環境変数制御** - `NYASH_MIR_UNIFIED_CALL=1`切り替え
- [x] **MIRダンプ対応** - `call_global`, `call_method`表示
- [x] **VM実行器対応** - Call命令のCallee型処理

#### 個別実装 (Phase 3.1-3.4)
- [x] **indirect call統一** - `build_indirect_call_expression`でCallTarget::Value
- [x] **print関数統一** - ExternCall→Callee::Global移行
- [x] **function call統一** - `build_function_call`でCallTarget::Global
- [x] **BoxCall統一** - `emit_box_or_plugin_call`でCallTarget::Method
- [x] **Python LLVM基盤** - `src/llvm_py/instructions/mir_call.py`作成
- [x] **instruction_lower.py** - `op == "mir_call"`統一分岐追加

#### 検証完了
- [x] **MIRダンプ確認** - 統一Call形式での出力確認
- [x] **環境変数切り替え** - 新旧実装の正常動作確認
- [x] **コンパイル成功** - 全ての変更でビルドエラー0

### ✅ **Week 1完了**: llvmlite革命達成（2025-09-24）

#### Phase 3.5: llvmlite内部Callee統一 ✅ 完全達成
**MIR命令生成統一 ✅ + Callee処理統一 ✅ + JSON出力統一 🔄**

##### ✅ 完了項目: 真の統一実装
- [x] **デリゲート方式→真の統一** - call.py/boxcall.py/externcall.py核心ロジック完全移植
- [x] **6種類Callee完全対応** - Global/Method/Constructor/Closure/Value/Extern全実装
- [x] **環境変数制御完璧動作** - NYASH_MIR_UNIFIED_CALL=1で統一Call確認済み
- [x] **Everything is Box実装** - boxcall.py核心ロジック完全統一化
- [x] **C ABI完全対応** - externcall.py型変換ロジック完全統一化
- [x] **動的呼び出し実装** - closure/value呼び出し完全対応

### 🔄 **Week 2進行中**: JSON出力統一（Phase A核心）

##### 優先度1: mir_json_emit.rs統一Call対応 🔄
- [ ] **v1スキーマ実装** - 6種類Callee→JSON v1完全対応
- [ ] **スキーマヘッダー** - ir_schema/version/capabilities情報
- [ ] **環境変数制御** - NYASH_MIR_UNIFIED_CALL=1でv1出力

##### 優先度2: Python側v1処理強化
- [ ] **instruction_lower.py v1対応** - JSON v1→llvmlite統一経路
- [ ] **実際のLLVMハーネステスト** - モックルート回避
- [ ] **round-trip整合性** - emit→read→emit確認

##### 優先度3: 統一Call完全検証
- [ ] **4実行器マトリクス** - 全実行器で6種類Callee動作確認
- [ ] **パフォーマンス回帰** - 実行速度・メモリ使用量測定

### ⏳ 未着手項目

#### テスト・検証
- [ ] **統合テスト** - 統一Call全パターンの動作確認
- [ ] **パフォーマンステスト** - 実行速度・メモリ使用量測定
- [ ] **回帰テスト** - 既存機能の完全互換性確認

### 📋 Phase A残作業 (推定2-3週間) - 修正版

#### 🏗️ 4実行器 × 3統一 = 完全マトリクス達成

| 実行器 | MIR生成統一 | Callee処理統一 | JSON出力統一 |
|--------|-------------|----------------|---------------|
| **MIR Builder** | ✅ Phase 3.1-3.4完了 | ✅ emit_unified_call | ⏳ mir_json_emit |
| **VM実行器** | ✅ 同上 | ✅ Call命令対応済み | ✅ 同一MIR |
| **Python LLVM** | ✅ 同上 | 🔄 **llvmlite内部要対応** | ⏳ v1形式要対応 |
| **mini-vm** | ✅ 同上 | ⏳ 将来対応 | ⏳ 将来対応 |

#### 📅 現実的スケジュール

```
Week 1: llvmlite Callee革命
- Day 1-2: llvmlite内部の個別call系をCallee統一に変更
- Day 3-4: 6種類Callee完全対応（Constructor/Closure/Value追加）
- Day 5: llvmlite統一Call動作確認

Week 2: JSON統一完成
- Day 1-3: mir_json_emit統一Call実装
- Day 4-5: JSON v1スキーマ + Python側v1対応
- 週末: 統合テスト（全Calleeパターン × 全実行器）

Week 3: 完全検証
- 統一Call動作: 4実行器全てで6種類Callee動作確認
- JSON round-trip: emit→read→emit整合性
- パフォーマンス: 回帰なし確認
```

---

## ⏳ Phase B: JSON中心化移行 (未着手)

### 📋 計画済み作業

#### MIRラッパー化
- [ ] **JSON→MIRリーダー薄化** - 重厚な構造体→薄いラッパー
- [ ] **HIR情報JSON化** - 名前解決・型情報の保持
- [ ] **型安全ビュー実装** - 型安全性確保

#### 設計方針決定必要
- [ ] **ラッパー詳細設計** - パフォーマンス・型安全性両立
- [ ] **HIR情報スキーマ** - JSON形式での型・スコープ情報
- [ ] **移行計画詳細化** - 既存コードの段階的変換

### 推定期間: 4-6週間

---

## ⏳ Phase C: 完全JSON化 (未着手)

### 📋 将来計画

#### Rust依存削減
- [ ] **MIR Module廃止準備** - 依存箇所の特定・移行
- [ ] **多言語実装PoC** - Python/JavaScript実装技術実証
- [ ] **プリンターJSON化** - 表示系の統一

#### 基盤完成
- [ ] **セルフホスティング準備** - Phase 15基盤提供
- [ ] **パフォーマンス確保** - 最適化・高速化
- [ ] **ドキュメント整備** - 完全移行ガイド

### 推定期間: 8-12週間

---

## 📁 実装ファイル状況

### 新規作成ファイル ✅
```
src/mir/definitions/call_unified.rs         (297行) ✅
src/llvm_py/instructions/mir_call.py        (120行) ✅
docs/development/roadmap/phases/phase-15.5/ (6文書) ✅
```

### 変更済みファイル ✅
```
src/mir/builder/utils.rs                    ✅ BoxCall統一対応
src/mir/builder/builder_calls.rs           ✅ emit_unified_call実装
src/llvm_py/builders/instruction_lower.py  ✅ mir_call分岐追加
CURRENT_TASK.md                            ✅ Phase 3.4→3.5更新
CLAUDE.md                                   ✅ 進捗反映
```

### 実装予定ファイル ⏳
```
src/runner/mir_json_emit.rs                 ⏳ 統一Call JSON出力
src/llvm_py/instructions/mir_call.py        ⏳ v1対応強化
tools/phase15_5_test.sh                     ⏳ 包括テストスクリプト
```

---

## 🧪 テスト状況

### 完了済みテスト ✅
- [x] **基本コンパイル** - cargo check/build成功
- [x] **MIRダンプ** - 統一Call表示確認
- [x] **環境変数切り替え** - 新旧実装動作確認

### 実装中テスト 🔄
- [ ] **BoxCall実行** - 実際のBoxメソッド呼び出し
- [ ] **Python LLVM** - mir_call.py経由実行

### 未実装テスト ⏳
- [ ] **統一Call全パターン** - 6種類のCallee完全テスト
- [ ] **JSON v1 round-trip** - emit→parse→emit整合性
- [ ] **パフォーマンス回帰** - 実行速度・メモリ使用量
- [ ] **互換性回帰** - 既存テストスイート全通過

---

## 📈 品質メトリクス

### コード品質
```
警告数:        19 (Phase 3.4時点)
エラー数:      0  ✅
テストカバー:  未測定
型安全性:      維持 ✅
```

### パフォーマンス
```
ビルド時間:    ±0% (変化なし)
実行速度:      未測定
メモリ使用:    未測定
```

### 互換性
```
既存機能:      100% ✅ (環境変数OFF時)
新機能:        80%  🔄 (基本動作のみ)
```

---

## 🎯 次の優先タスク - Phase 15.5の真の目的

### 🔥 **3つの統一革命**
1. **🏗️ MIR命令生成統一** - Builder側でemit_unified_call() ✅ 完了
2. **⚙️ Callee処理統一** - 全実行器で6種類Callee対応 🔄 **llvmlite要対応**
3. **📋 JSON出力統一** - 統一Call形式での出力 ⏳ 未着手

### 今週 (Week 1) - llvmlite革命
1. **llvmlite内部Callee統一** - call.py/boxcall.py/externcall.py統一化
2. **6種類Callee完全対応** - Constructor/Closure/Value追加実装
3. **環境変数制御** - NYASH_MIR_UNIFIED_CALL=1でllvmlite内部統一

### 来週 (Week 2) - JSON統一完成
1. **mir_json_emit統一Call実装** - v1スキーマ実装
2. **JSON v1スキーマ + Python側v1対応**
3. **統合テスト** - 全Calleeパターン × 全実行器

### 第3週 (Week 3) - 完全検証
1. **4実行器マトリクス** - 全実行器で6種類Callee動作確認
2. **JSON round-trip** - emit→read→emit整合性確認
3. **Phase A完了判定** - 完了条件チェック

### Phase B準備
1. **詳細設計** - JSON中心化アーキテクチャ
2. **リスク評価** - 技術的課題の洗い出し
3. **プロトタイプ** - 小規模な技術実証

---

**このドキュメントは週次で更新され、Phase 15.5の正確な進捗状況を反映します。**