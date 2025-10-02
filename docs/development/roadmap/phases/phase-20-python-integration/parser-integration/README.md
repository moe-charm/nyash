# Parser Integration - Pythonパーサー統合

## 📋 概要

PythonのAST（抽象構文木）をHakoruneで扱うためのパーサー統合設計です。

## 📁 ファイル一覧

### 実装計画
- **[implementation-plan.md](implementation-plan.md)** - PythonパーサーBox実装計画
- **[builtin-box-flow.md](builtin-box-flow.md)** - ビルトインBox実装フロー

## 🎯 目的

### PythonパーサーBoxの役割

1. **Python AST解析**
   - Pythonコードをパース
   - AST（抽象構文木）を生成
   - Hakorune内で操作可能な形式に変換

2. **型推論・スコープ解決**
   - Python動的型の静的解析
   - スコープ情報の抽出
   - 型アノテーション活用

3. **Hakorune MIRへの変換**
   - Python AST → Hakorune AST
   - Hakorune AST → MIR
   - 最適化情報の付与

## 🏗️ アーキテクチャ

### PythonParserBox（計画）

```hakorune
box PythonParserBox {
    // パース
    parse(code: StringBox) -> ASTBox

    // AST操作
    visit_nodes(ast: ASTBox, visitor: FunctionBox)
    get_node_type(node: ASTBox) -> StringBox

    // 変換
    to_hakorune_ast(ast: ASTBox) -> HakoruneASTBox
    to_mir(ast: ASTBox) -> MIRBox
}
```

### ビルトインBox統合フロー

1. **パーサー初期化**
   - CPython APIの初期化
   - パーサーモジュールのロード

2. **パース実行**
   - コード文字列を受け取り
   - Python ASTを生成
   - エラーハンドリング

3. **AST変換**
   - Python AST → Hakorune AST
   - 型情報の付与
   - スコープ情報の追加

4. **MIR生成**
   - Hakorune AST → MIR
   - 最適化パスの適用

## 🔧 技術的課題

### 1. Python AST API統合
- CPython C API使用
- `ast` モジュールとの連携
- メモリ管理（参照カウント）

### 2. 型推論
- Python動的型の静的解析
- 型アノテーション活用
- 型推論アルゴリズム

### 3. スコープ解決
- Pythonのスコープルール（LEGB）
- Hakoruneスコープへのマッピング
- クロージャ・ネストスコープ対応

### 4. 構文マッピング
- Python構文 → Hakorune構文
- イディオム変換
- サポートする機能範囲の決定

## 📊 実装ステータス

| コンポーネント | ステータス | 備考 |
|--------------|----------|------|
| パーサーBox設計 | ✅ 完了 | 設計書完成 |
| CPython API統合 | 📅 未実装 | - |
| AST変換 | 📅 未実装 | - |
| 型推論 | 📅 未実装 | - |
| MIR生成 | 📅 未実装 | - |

## ⚠️ リスク要因

1. **CPython依存**
   - CPythonバージョン互換性
   - プラットフォーム固有の問題

2. **型推論の精度**
   - Python動的型の限界
   - 型アノテーションの不完全性

3. **パフォーマンス**
   - パース・変換のオーバーヘッド
   - 大規模コードベースでの性能

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Planning](../planning/)
- [Core Implementation](../core-implementation/)
