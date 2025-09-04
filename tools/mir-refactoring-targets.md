# MIRリファクタリング対象ファイル

## 🚨 緊急度高：大きなファイル（900行以上）

### 1. mir/verification.rs (965行)
**分割案**：
- `mir/verification/basic.rs` - 基本検証
- `mir/verification/types.rs` - 型検証
- `mir/verification/control_flow.rs` - 制御フロー検証
- `mir/verification/ownership.rs` - 所有権検証

### 2. mir/builder.rs (930行) 
**状態**: ChatGPT5が作業中
**分割案**：
- `mir/builder/exprs.rs` - 式のビルド（一部完了）
- `mir/builder/stmts.rs` - 文のビルド（一部完了）
- `mir/builder/decls.rs` - 宣言のビルド（一部完了）
- `mir/builder/control_flow.rs` - 制御構造

### 3. mir/instruction.rs (896行)
**状態**: MIR13固定化で大幅変更予定
**現在**: 20命令（ChatGPT5設計）→ 目標: 13命令
**作業内容**:
- 不要な命令の削除
- BoxCall統一（ArrayGet/Set, RefNew/Get/Set等）
- TypeOp統一（TypeCheck, Cast）

### 4. mir/optimizer.rs (875行)
**分割案**：
- `mir/optimizer/constant_folding.rs`
- `mir/optimizer/dead_code.rs`
- `mir/optimizer/inline.rs`
- `mir/optimizer/type_inference.rs`

## 📊 MIR命令削減マッピング（20→13）

### 削除予定の命令
```
ArrayGet, ArraySet → BoxCall
RefNew, RefGet, RefSet → BoxCall  
WeakNew, WeakGet → BoxCall
MapGetProperty, MapSetProperty → BoxCall
TypeCheck, Cast → TypeOp
PluginInvoke → BoxCall（プラグイン統合）
Copy → Load + Store
Debug, Print → ExternCall
Nop → 削除
Throw, Catch → ExternCall
Safepoint → 削除（VMレベルで処理）
```

### 最終的な13-14命令
1. Const - 定数
2. Load - 読み込み
3. Store - 書き込み
4. BinOp - 二項演算
5. UnaryOp - 単項演算
6. Compare - 比較
7. Branch - 条件分岐
8. Jump - 無条件ジャンプ
9. Return - 戻り値
10. Call - 関数呼び出し
11. BoxCall - Box操作統一
12. TypeOp - 型操作統一
13. Phi - SSA合流
14. ExternCall - 外部呼び出し（オプション）

## 🚀 実行コマンド例

```bash
# 非同期でverification.rsのリファクタリング
./tools/codex-async-notify.sh "Refactor src/mir/verification.rs into smaller modules (basic, types, control_flow, ownership)"

# optimizer.rsの分割
./tools/codex-async-notify.sh "Split src/mir/optimizer.rs into separate optimization pass modules"

# MIR命令削減の実装
./tools/codex-async-notify.sh "Reduce MIR instructions from 57 to 13-14 by unifying with BoxCall and TypeOp"
```