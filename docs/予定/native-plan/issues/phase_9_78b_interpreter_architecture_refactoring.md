# Phase 9.78b: インタープリター・VM統合アーキテクチャ再設計

**作成日**: 2025-08-21  
**優先度**: 最高（Phase 9.78aの前提条件）  
**設計者**: Codex exec (天才的洞察)  
**実装者**: Claude + User

## 🎯 目標

InterpreterとVMの「実装詳細共有」から「モデル共有・実行時共有」への転換を実現し、依存関係を整理する。

## 📊 現状の問題（Phase 9.78aで発覚）

1. **依存関係の逆転**
   - `BoxDeclaration`が`interpreter::BoxDeclaration`として定義
   - VMがインタープリターに依存

2. **SharedState中心の設計**
   - インタープリター固有の実装詳細
   - VMから使いにくい

3. **BoxFactoryのtrait問題**
   - `Arc<BoxFactory>`でコンパイルエラー（`dyn`が必要）

4. **グローバル副作用**
   - テスト・並行実行で問題

## 🏗️ 新アーキテクチャ設計

```
AST/Model → Runtime → Interpreter/VM
                 ↑
              Plugins
```

### 核心的変更
- **モデル層**: BoxDeclaration等の純粋データ
- **ランタイム層**: BoxClass/Factory、インスタンス管理
- **バックエンド層**: 実行戦略のみ（AST実行 or MIR実行）

## 📋 実装ステップ（8段階）

### ✅ Step 1: BoxDeclarationの移動
**期限**: 1日  
**リスク**: 低  
**作業内容**:
1. `src/core/model.rs`を作成
2. `BoxDeclaration`を`interpreter::BoxDeclaration`から移動
3. 一時的な別名で互換性維持
```rust
use core::model::BoxDeclaration as InterpreterBoxDecl;
```

**成功基準**: 
- [ ] ビルド成功（警告OK）
- [ ] 既存テスト全パス

### ✅ Step 2: NyashRuntime骨組み作成
**期限**: 1日  
**リスク**: 低  
**作業内容**:
1. `src/runtime/mod.rs`作成
2. 最小限の`NyashRuntime`構造体
3. `NyashRuntimeBuilder`追加

**成功基準**:
- [ ] 新モジュールのビルド成功
- [ ] 既存コードへの影響なし

### ✅ Step 3: BoxFactoryのdyn化
**期限**: 2日  
**リスク**: 中  
**作業内容**:
1. すべての`Arc<BoxFactory>`を`Arc<dyn BoxFactory>`に変更
2. `BoxClass`トレイト導入
3. `BoxRegistry`実装

**成功基準**:
- [ ] trait object正しく使用
- [ ] VMでのコンパイルエラー解消

### ✅ Step 4: グローバル登録の排除
**期限**: 1日  
**リスク**: 中  
**作業内容**:
1. `register_user_defined_factory()`削除
2. `NyashRuntimeBuilder::with_factory()`追加
3. 既存の登録箇所を修正

**成功基準**:
- [ ] グローバル関数の完全削除
- [ ] 明示的な依存注入に移行

### ✅ Step 5: SharedState分解
**期限**: 3日  
**リスク**: 高  
**作業内容**:
1. `SharedStateShim`導入（互換層）
2. フィールドを段階的に移行
   - `box_declarations` → `type_space`
   - `global_box` → `ExecutionSession.root_box`
3. テストを随時実行

**成功基準**:
- [ ] SharedStateShim経由で動作
- [ ] 既存機能の維持

### ✅ Step 6: Interpreter/VM統一
**期限**: 2日  
**リスク**: 中  
**作業内容**:
1. 共通コンストラクタ実装
2. `ExecutionSession`共有
3. VM側のBox管理をRuntime経由に

**成功基準**:
- [ ] 両者が同じRuntimeを使用
- [ ] VMでのBox生成成功

### ✅ Step 7: ライフサイクル統一
**期限**: 2日  
**リスク**: 中  
**作業内容**:
1. birth/finiをBoxClass経由に
2. ScopeTrackerとの統合
3. 全Box型で動作確認

**成功基準**:
- [ ] birth/fini正しく呼ばれる
- [ ] メモリリークなし

### ✅ Step 8: クリーンアップ
**期限**: 1日  
**リスク**: 低  
**作業内容**:
1. SharedState完全削除
2. 不要な互換層削除
3. ドキュメント更新

**成功基準**:
- [ ] コードベースの簡潔性
- [ ] 全テストパス

## 🔗 関連リンク

- **設計書**: [architecture-redesign-proposal.md](../../../architecture-redesign-proposal.md)
- **VM実装状況**: [CURRENT_VM_CHANGES.md](../../../CURRENT_VM_CHANGES.md)
- **現在のタスク**: [CURRENT_TASK.md](../../../CURRENT_TASK.md)
- **Codex分析**: nyash_interpreter_refactoring_analysis.txt

## 📊 進捗トラッキング

| Step | 状態 | 開始日 | 完了日 | 担当 | 備考 |
|------|------|--------|--------|------|------|
| 1 | 未着手 | - | - | - | BoxDeclaration移動 |
| 2 | 未着手 | - | - | - | Runtime骨組み |
| 3 | 未着手 | - | - | - | dyn化 |
| 4 | 未着手 | - | - | - | グローバル排除 |
| 5 | 未着手 | - | - | - | SharedState分解 |
| 6 | 未着手 | - | - | - | 統一 |
| 7 | 未着手 | - | - | - | ライフサイクル |
| 8 | 未着手 | - | - | - | クリーンアップ |

## ⚠️ リスクと対策

### 高リスク項目
1. **SharedState分解（Step 5）**
   - 対策: SharedStateShimで段階的移行
   - ロールバック: git stashで保存

2. **ライフサイクル統一（Step 7）**
   - 対策: 十分なテストケース作成
   - ロールバック: 旧実装を一時保持

### 中リスク項目
1. **BoxFactoryのdyn化（Step 3）**
   - 対策: コンパイラエラーを1つずつ解決
   - ロールバック: trait定義を調整

## 🧪 テスト戦略

1. **各ステップでの確認**
   - `cargo test`全パス必須
   - `cargo check --lib`でビルド確認

2. **統合テスト**
   - インタープリター動作確認
   - VM動作確認（Step 6以降）

3. **パフォーマンステスト**
   - Step 8完了後に実施
   - 既存ベンチマークと比較

## 📝 作業時の注意事項

1. **trait objectは必ず`Arc<dyn Trait>`**
   - ❌ `Arc<BoxFactory>`
   - ✅ `Arc<dyn BoxFactory>`

2. **段階的移行の厳守**
   - 各ステップでビルド成功必須
   - テスト失敗したら即修正

3. **CURRENT_TASK.mdの更新**
   - 作業開始時に更新
   - 問題発生時に記録
   - 完了時に結果記載

---

**総工数見積もり**: 14日（各ステップにバッファ含む）  
**推奨アプローチ**: Step 1-2を先行実施して感触を掴む