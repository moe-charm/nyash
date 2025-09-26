# Nyash言語論文 Phase 15更新計画

最終更新: 2025-09-27

## 🎯 更新の目的

Phase 15の最新仕様（Everything is Box完全版、birth統一、try撤廃等）を反映し、論文を完成させる。

---

## 📋 更新項目リスト

### 優先度1（必須更新）

#### 1. **Everything is Box完全版**
- ❌ 古い: データだけBox
- ✅ 新: すべてがBox

**完全なBox化**:
- データBox: StringBox, IntegerBox, ArrayBox... ✅
- 演算子Box: AddOperator, CompareOperator... ✅（世界初！）
- 制御Box: LoopForm ✅（世界初！）

**更新箇所**:
- README.md: Everything is Boxの説明
- chapters/02-language-design.md: Box哲学完全版
- main-paper-jp.md: 中心的主張

#### 2. **birth統一**
- ❌ 古い: init/birth/pack混在
- ✅ 新: birthに完全統一

**統一内容**:
- ビルトインBox: birth
- ユーザー定義Box: birth
- プラグインBox: birth
- デリゲーション: `from ParentBox.birth()`

**更新箇所**:
- chapters/03-memory-model.md: コンストラクタ統一
- コード例すべて

#### 3. **try文撤廃革命**
- ❌ 古い: 従来のtry-catch-finally
- ✅ 新: postfix catch/cleanup

**新構文**:
```nyash
method() catch(Error e) { }
method() cleanup { }
method() 
  catch(e) { }
  cleanup { }
```

**追加箇所**:
- 新章: Exception Handling Revolution
- try撤廃の哲学的意義
- ネスト削減の実例

#### 4. **Property System革命**
- 新機能: stored/computed/once/birth_once

**4種類のProperty**:
- `stored`: 通常フィールド
- `computed`: 計算プロパティ
- `once`: 遅延評価キャッシュ
- `birth_once`: 即時評価

**追加内容**:
- Python @property/@cached_property完全マッピング
- 10-50x高速化の実証
- コード例

### 優先度2（強く推奨）

#### 5. **using system**
- 新機能: ドット記法、namespace解決

**特徴**:
- `plugin.StringBox` ドット記法
- 修飾名・namespace解決
- AST using統一（SSOT）
- 重複検出自動化

**追加箇所**:
- chapters/02-language-design.md: モジュールシステム

#### 6. **2本柱実行体制**
- ❌ 古い: 5つの実行形態
- ✅ 新: 2本柱 + 特殊用途

**実行モデル**:
- Rust VM: 開発・デバッグ・検証
- LLVM: 本番・最適化・配布
- PyVM: JSON v0ブリッジ専用

**更新箇所**:
- chapters/05-execution-backends.md: 実行モデル刷新

#### 7. **演算子Box: デバッグ革命**
- observe/adopt段階的移行
- Void混入即座特定
- ChatGPT「最強クラス」評価

**追加内容**:
- 演算子Boxの威力実証
- デバッグ事例
- AI協働開発での活用

### 優先度3（あれば尚良）

#### 8. **P2P Intentモデル**
- 既存内容の更新
- Box間通信の実例

#### 9. **match式**
- パターンマッチング
- ブロック式・値式の統一

---

## 📊 章構成（更新後）

### Introduction
- Nyash言語の概要
- Everything is Box完全版の主張
- 世界初の完全Box言語

### Chapter 2: Language Design
- Everything is Box哲学
  - データBox
  - 演算子Box
  - 制御Box（LoopForm）
- birth統一
- using system

### Chapter 3: Memory Model
- birth/fini対称性
- GCオン/オフ切替
- WeakBox設計

### Chapter 4: Exception Handling Revolution
- try文撤廃の哲学
- postfix catch/cleanup
- 段階的決定モデル
- ネスト削減の実例

### Chapter 5: Property System
- 4種類のProperty
- Python統合戦略
- 10-50x高速化実証

### Chapter 6: Execution Backends
- 2本柱体制
- VM/LLVMパリティ
- Phase 15戦略

### Chapter 7: Case Studies
- JSON Native
- 実アプリケーション
- プラグインエコシステム

### Conclusion
- Everything is Boxの完全実現
- 世界初の成果
- Future Work

---

## ✅ 完成チェックリスト

- [ ] README.md更新
- [ ] Chapter 2: Language Design（Everything is Box完全版）
- [ ] Chapter 3: Memory Model（birth統一）
- [ ] Chapter 4: Exception Handling Revolution（新規作成）
- [ ] Chapter 5: Property System（新規作成）
- [ ] Chapter 6: Execution Backends（2本柱更新）
- [ ] Chapter 7: Case Studies（最新実例）
- [ ] main-paper-jp.md統合
- [ ] Abstract更新
- [ ] AI査読（ChatGPT/Claude）

---

## 🗓️ スケジュール

- **Day 1-2**: 構造更新、Everything is Box完全版
- **Day 3-4**: Chapter 2-3（Language Design, Memory Model）
- **Day 5-6**: Chapter 4-5（Exception, Property）
- **Day 7-8**: Chapter 6-7（Execution, Case Studies）
- **Day 9**: 統合、AI査読
- **Day 10**: 修正、完成

**目標**: 10日以内に完成 ✨
