# MIR14論文 Phase 15更新計画

最終更新: 2025-09-27

## 🎯 更新の目的

Phase 15の最新仕様（2本柱体制、PHI-on、演算子Box等）を反映し、論文を完成させる。

---

## 📋 更新項目リスト

### 優先度1（必須更新）

#### 1. **2本柱体制への更新**
- ❌ 古い: 5つの実行形態（Interpreter/VM/JIT/AOT/WASM）
- ✅ 新: 2本柱 + 特殊用途
  - Rust VM: 開発・デバッグ・検証（712行の高品質実装）
  - LLVM: 本番・最適化・配布（Python/llvmlite）
  - PyVM: JSON v0ブリッジ専用

**更新箇所**:
- README.md: 実行形態の説明
- chapters/05-evaluation.md: ベンチマーク対象
- main-paper-jp.md: 実行モデル図

#### 2. **PHI-on標準化**
- ❌ 古い: PHI-off（エッジコピー）前提
- ✅ 新: PHI-on標準、LoopForm実装

**更新箇所**:
- MIR13_CORE13_SPEC.md → MIR14_SPEC.md
- chapters/02-box-theory.md: SSA形式説明追加
- 制御構造の説明でLoopForm追加

#### 3. **LoopForm: 制御構造のBox化**
- ❌ 古い: ループは特殊構文
- ✅ 新: LoopFormで制御もBox化

**追加内容**:
- LoopFormの設計と実装
- PHI生成の自動化
- break/continue処理

#### 4. **Callee型: 型安全な関数呼び出し**
- ❌ 古い: 文字列ベースの関数解決
- ✅ 新: Callee enum（Global/Method/Value/Extern）

**追加内容**:
- シャドウイング問題の解決
- コンパイル時型解決
- VM/LLVM両対応

### 優先度2（強く推奨）

#### 5. **演算子Box統一**
- 新機能: AddOperator, CompareOperator等
- observe/adopt段階的移行
- デバッグ可視化の威力

**追加箇所**:
- chapters/03-boxcall-unification.md: 演算子Box追加
- 実装例とデバッグ事例

#### 6. **実装実証の更新**
- JSON Native: 完全な構文解析器
- スモークテスト: quick/integration/full
- VM/LLVMパリティ検証

**更新箇所**:
- chapters/05-evaluation.md: 最新ベンチマーク
- 実アプリケーション例

### 優先度3（あれば尚良）

#### 7. **MIR Unified Call計画**
- 6種類のCall → 1つのMirCallに統一予定
- 7,372行 → 5,468行（26%削減見込み）

**記載内容**:
- Future Workとして言及
- Phase 15.5以降の計画

---

## 📊 章構成（更新後）

### Introduction
- Nyash言語とMIR14の概要
- 2本柱体制の説明
- Everything is Box哲学

### Chapter 2: MIR14設計
- 14命令の詳細
- PHI-on標準化
- LoopFormによる制御Box化
- Callee型による型安全化

### Chapter 3: BoxCall統一
- データBox
- 演算子Box（新規追加）
- 制御Box（LoopForm）

### Chapter 4: 2本柱実装
- Rust VM: 開発・デバッグ
- LLVM: 本番・最適化
- VM/LLVMパリティ戦略

### Chapter 5: 実装実証
- JSON Native
- スモークテスト結果
- ベンチマーク

### Conclusion
- Everything is Boxの完全実現
- 世界初: 14命令で完全実装
- Future Work: MIR Unified Call

---

## ✅ 完成チェックリスト

- [ ] README.md更新
- [ ] MIR14_SPEC.md作成
- [ ] Chapter 2: MIR14設計（PHI-on, LoopForm追加）
- [ ] Chapter 3: BoxCall統一（演算子Box追加）
- [ ] Chapter 4: 2本柱実装（新規作成）
- [ ] Chapter 5: 実装実証（最新データ）
- [ ] main-paper-jp.md統合
- [ ] Abstract更新
- [ ] AI査読（ChatGPT/Claude）

---

## 🗓️ スケジュール

- **Day 1-2**: 構造更新、MIR14_SPEC作成
- **Day 3-5**: Chapter 2-3更新（PHI-on, LoopForm, 演算子Box）
- **Day 6-7**: Chapter 4-5更新（2本柱、実証）
- **Day 8**: 統合、AI査読
- **Day 9-10**: 修正、完成

**目標**: 10日以内に完成 ✨
