# 🎯 現在のタスク (2025-08-13 Native Nyash Phase 7 & 弱参照テスト追加)

## 🎯 2025-08-13 現在の進行状況

### 🚀 Native Nyash実装
- **Phase 6.1**: ✅ 完了（PR #43 - RefNew/RefGet/RefSet実装）
- **Phase 7**: 📝 Issue #44作成済み - Async model (nowait/await)実装待ち

### 🧪 ChatGPT5さんからの弱参照テスト追加タスク

#### 実装予定のテスト（tests/integration_tests.rs）
- [ ] `test_weak_field_cannot_finalize`: weakフィールドに対して `me.field.fini()` を呼ぶとエラーになることを確認
  - コード: weak parent を持つ Child に対して `me.parent.fini()` を呼ぶ
  - 期待: 実行エラー（メッセージに "Cannot finalize weak field" を含む）
  
- [ ] `test_usage_prohibited_after_fini`: インスタンスを `fini()` 後にメソッド呼び出しすると使用禁止エラーになることを確認
  
- [ ] weak自動null化テスト
  - 親 `p.fini()` 後に `c.isParentNull()` が `true` になる

#### テスト実行のワークアラウンド
- 推奨: `cargo test --tests -j32`（examples除外）
- Windows向けexampleをビルド対象から一時外すか、CIのmatrixでexamplesをスキップ

#### 追加検討事項
- weakの複数段/循環
- fini内でのカスケード順序検証

---

## 📋 以前の実装計画（参考）

### 📋 **段階的実装計画（優先度順）**

#### **Phase 1: 基本型Box実装** (最優先)

##### 1. **FloatBox** - 浮動小数点数Box 📊
- **依存**: なし (f64の基本実装)
- **実装内容**:
  - 基本値の保持・表示
  - 四則演算メソッド
  - 文字列変換
  - 比較演算子
- **テスト**: 
  ```nyash
  f = new FloatBox(3.14)
  print(f.add(2.86))  // 6.0
  print(f.toString()) // "3.14"
  ```

##### 2. **ArrayBoxの改良** - 配列機能の強化 📚
- **依存**: 既存ArrayBox実装
- **追加機能**:
  - sort()メソッド - 配列ソート
  - reverse()メソッド - 配列反転
  - indexOf()メソッド - 要素検索
  - slice()メソッド - 部分配列
- **テスト**:
  ```nyash
  arr = new ArrayBox()
  arr.push(3); arr.push(1); arr.push(2)
  arr.sort()  // [1, 2, 3]
  ```

#### **Phase 2: 演算子システム** (高優先)

##### 3. **基本演算子の改良** ➕➖✖️➗
- **依存**: 既存の演算子実装
- **改良内容**:
  - 型間演算の対応 (IntegerBox + FloatBox)
  - 文字列 + 数値の連結
  - より良いエラーメッセージ
- **テスト**:
  ```nyash
  print(42 + 3.14)     // 45.14 (型変換)
  print("Value: " + 42) // "Value: 42"
  ```

##### 4. **比較演算子の完全実装** 🔍
- **実装内容**:
  - ==, !=, <, >, <=, >= の完全対応
  - 型間比較のサポート
  - null比較の正しい動作
- **テスト**: 全ての型の組み合わせテスト

#### **Phase 3: ユーティリティBox** (中優先)

##### 5. **DateTimeBox** - 日時操作 📅
- **依存**: chrono crate (既存)
- **機能**:
  - 現在時刻の取得
  - 日時の計算・比較
  - フォーマット変換
- **テスト**: 日時計算、文字列変換

##### 6. **FileBox** - ファイル操作 📁
- **依存**: std::fs
- **機能**:
  - ファイル読み書き
  - 存在確認
  - ディレクトリ操作
- **テスト**: 基本的なファイル操作

### 🎯 **今週の実装目標**

#### **今日 (2025-08-11)**: FloatBox実装
1. FloatBox構造体作成
2. 基本メソッド実装 (add, sub, mul, div)
3. Nyashからの使用テスト
4. インタープリター統合

#### **明日**: ArrayBox改良
1. sort()メソッド実装
2. reverse()メソッド実装
3. テストスクリプト作成・動作確認

#### **明後日**: 演算子改良
1. 型間演算の実装
2. エラーハンドリング改善
3. 包括的テスト

### 📊 **実装ステータス**

#### ✅ 実装済み (Arc<Mutex>統一完了)
- StringBox, IntegerBox, BoolBox, NullBox
- ConsoleBox, MathBox, TimeBox, MapBox
- DebugBox, RandomBox, ArrayBox (基本)
- BufferBox, RegexBox, JSONBox, StreamBox

#### 🚧 今回追加予定
- FloatBox (今日)
- ArrayBox改良 (明日)
- 演算子改良 (明後日)

#### 📋 将来実装予定
- DateTimeBox, FileBox
- より複雑なBox (P2PBox等)

### 💭 **重要な原則**

1. **一つずつ確実に**: 1つのBoxを完全に実装してから次へ
2. **テストファースト**: 必ずNyashスクリプトで動作確認
3. **段階的複雑化**: シンプルから複雑へ
4. **ビルド確認**: 毎回`cargo build`で確認
5. **依存関係注意**: 複雑な依存は後回し

この方針で、確実で安定した実装を進めていきます！

---
最終更新: 2025-08-11 - シンプルBox段階実装方針決定！