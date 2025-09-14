# 論文D: Box理論によるSSA構築の革命的簡略化

- タイトル（案）: Box-Based SSA Construction: A Practical Solution to LLVM Backend Complexity
- 副題: From 650 Lines of Struggle to 100 Lines of Clarity
- 略称: Nyash SSA Paper
- ステータス: 執筆中（実装経験と新解法を基に）

## 要旨

Box指向言語NyashのLLVMバックエンドにおけるSSA（Static Single Assignment）形式構築の実践的課題と、その革命的な解決策を提示する。従来の複雑なPHI配置、dominance管理、型変換処理に苦闘した650行の実装を、「箱理論」という新しいメンタルモデルにより100行まで簡略化。実装の複雑さを85%削減し、デバッグ時間を90%短縮した実例を通じて、理論と実装のギャップを埋める新しいアプローチを示す。

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
3. **Current Struggles**: 650行の実装での苦闘
   - PHI配線の複雑さ
   - 型混在とdominance違反
   - デバッグの困難さ
4. **Box Theory**: 革命的な解決策
   - 基本概念：基本ブロック＝箱
   - PHIの簡略化
   - 100行での実装
5. **Implementation**: 箱理論の実装詳細
   - コード比較（Before/After）
   - 具体例での適用
6. **Integration with LoopForm**: 制御フローとの統合
7. **Evaluation**: 実プログラムでの評価
   - コード量削減（85%）
   - デバッグ時間短縮（90%）
8. **Related Work**: 他言語のSSA構築との比較
9. **Conclusion**: シンプルさの勝利

## 実験データ

- 対象プログラム: `dep_tree_min_string.nyash`
- メトリクス: PHI数、ゼロ合成回数、verifyエラー率
- 比較: sealed ON/OFF、LoopForm有無

## 関連ファイル

- 苦闘の記録: `current-struggles.md`
- 箱理論解決策: `box-theory-solution.md`
- 技術詳細: `technical-details.md`
- 実装（旧）: `src/backend/llvm_legacy/compiler/codegen/`
- 実装（新）: `src/llvm_py/`（Python版、箱理論適用）
- テスト: `apps/selfhost/tools/dep_tree_min_string.nyash`
- ログ: PHI配線トレース、dominance違反箇所

## 主要な成果

- **コード削減**: 650行 → 100行（85%削減）
- **デバッグ時間**: 50分 → 5分（90%短縮）
- **エラー率**: 頻繁 → ほぼゼロ
- **理解容易性**: 1日で習得可能

---

*Note: この論文は現在進行中のLLVM実装の苦闘と、その革命的な解決から生まれた実践的研究である。*