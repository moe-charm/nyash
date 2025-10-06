# Self Current Task — Now (main)

2025‑09‑08：現状と直近タスク（selfhosting）

## 🔴 重要：ループビルダーのバグ発見（詳細調査完了）

### 問題詳細
- **症状**: dep_tree_min_string.hakoがVM実行でエラー `Invalid value: Value %57 not set`
- **発生箇所**: `loop(i + m <= n) { if ... { return 1 } i = i + 1 }`のような構造

### 根本原因（深掘り調査結果）
- **ループビルダーの致命的欠陥**: `build_statement`が親ビルダーに単純委譲するだけ
  ```rust
  // src/mir/loop_builder.rs:397-399
  fn build_statement(&mut self, stmt: ASTNode) -> Result<ValueId, String> {
      self.parent_builder.build_expression(stmt)  // スコープ管理が失われる！
  }
  ```
- **block_var_mapsの不完全性**: preheaderとlatchの2箇所しか保存しない
  - ループ内if文で生成される中間ブロックの変数状態が保存されない
  - 結果として変数の最終値が追跡できない

### 🎯 「箱を下に積む」解決策
Nyashの開発哲学「Everything is Box」に従い、スコープも箱化する：

1. **ScopeBox構造の導入**
   ```rust
   struct ScopeBox {
       block_id: BasicBlockId,
       variables: HashMap<String, ValueId>,
       parent: Option<Box<ScopeBox>>,  // 親スコープへの参照
   }
   ```

2. **全BasicBlockで変数スナップショットを保存**
   - ブロック遷移時に自動的に変数状態を箱として保存
   - 階層的な変数解決（現在の箱→親の箱→...）

3. **最小限の修正案**
   - `build_statement`でブロック遷移を検出し、変数マップを保存
   - phi node作成時に適切な箱から値を取得

## 進捗
- MIR ビルダー（self_main）へ Loop CFG/continue/break を移植（単一 exit + 単一 backedge、post‑terminated 禁止）。
- dep_tree（string最小版）の実装完了も、上記バグにより実行不可。

## 直近タスク（優先度順）
1. **🔥 ループビルダーのバグ修正**（ブロッカー）
   - **推奨案**: 「箱を下に積む」アプローチで最小限の修正
     - `build_statement`を改修してブロック遷移検出
     - 各ブロックの変数状態を自動的に箱として保存
     - 実装工数: 中程度（既存構造を活用）
   - 代替案A: ループビルダーの根本的な再設計（工数大）
   - 代替案B: 一時的なワークアラウンド（dep_tree_min_string.hakoの書き換え）
2. dep-tree 深さ1（直下 include）を children に反映（行ベースの素朴抽出、`//`/`#` 行コメントスキップ）。
3. `make dep-tree` 結果の JSON 形を確認（先頭 `{`、必須キー、children のリーフが path のみ）。
4. その後、深さ2→任意深さ（max-depth=64、visited）を段階的に解禁。

代表コマンド
- ビルド: `cargo build --release`
- 最小 dep-tree: `./target/release/nyash --backend vm apps/selfhost/tools/dep_tree_min_string.hako`
- 生成: `make dep-tree`（`tmp/deps.json`）
