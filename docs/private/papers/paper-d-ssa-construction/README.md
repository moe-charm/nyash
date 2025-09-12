# 論文D: Box指向言語におけるSSA形式の実践的構築

- タイトル（案）: Practical SSA Construction for a Box-Oriented Language
- 略称: Nyash SSA Paper
- ステータス: 執筆中（実装経験を基に）

## 要旨

Box指向言語NyashのLLVMバックエンドにおけるSSA（Static Single Assignment）形式構築の実践的課題と解決策を提示する。特に、動的型付けBox言語特有のPHI配置問題、BuilderCursorによる位置管理、sealed SSAアプローチの適用について、実装の困難と工夫を詳述する。

## 位置づけ

- **論文A（MIR14）**: 箱理論とMIR設計（哲学→実装）
- **論文B（Nyash）**: 言語設計と実行モデル
- **論文D（本稿）**: SSA構築の実践的アルゴリズム ← ここ
- **論文E（LoopForm）**: 制御フロー正規化の革新

## 主要貢献

1. **PHI配置の実践的課題**
   - 複雑な制御フローでのdominance違反
   - Box（i64 handle）とptr型の混在問題
   - 未定義値とゼロ合成の設計判断

2. **BuilderCursor設計**
   - 終端後挿入の構造的防止
   - with_blockによる位置管理の分離
   - closed状態の明示的追跡

3. **Sealed SSAの段階的導入**
   - block_end_valuesによるスナップショット
   - pred/succ関係の正規化
   - 段階的なPHI配線戦略

## 実装からの教訓

### 1. 実際に遭遇した問題
- `dep_tree_min_string.nyash`での複雑なループ構造
- 文字列処理での頻繁な型変換（handle↔ptr）
- esc_json関数での深いネスト制御フロー

### 2. 解決アプローチ
- 環境変数によるデバッグ制御（NYASH_LLVM_TRACE_PHI）
- 段階的なsealed SSA導入（NYASH_LLVM_PHI_SEALED）
- ゼロ値合成による安全なフォールバック

### 3. 未解決課題
- 完全なdominance保証の実現
- 最適なPHI最小化アルゴリズム
- LoopFormとの統合戦略

## 章構成（案）

1. **Introduction**: Box言語でのSSA構築の特殊性
2. **Background**: SSA形式とLLVM IRの基礎
3. **Challenges**: Nyash特有の問題（Box型、動的性）
4. **BuilderCursor**: 位置管理の新手法
5. **Sealed SSA**: 段階的導入と実装
6. **Evaluation**: 実プログラムでの評価
7. **Related Work**: 他言語のSSA構築との比較
8. **Conclusion**: 教訓と将来展望

## 実験データ

- 対象プログラム: `dep_tree_min_string.nyash`
- メトリクス: PHI数、ゼロ合成回数、verifyエラー率
- 比較: sealed ON/OFF、LoopForm有無

## 関連ファイル

- 実装: `src/backend/llvm/compiler/codegen/`
- テスト: `apps/selfhost/tools/dep_tree_min_string.nyash`
- ログ: PHI配線トレース、dominance違反箇所

---

*Note: この論文は現在進行中のLLVM実装の苦闘から生まれた実践的研究である。*