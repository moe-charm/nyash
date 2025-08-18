# BID Code Generation from Copilot

このフォルダには、CopilotさんがPR #117で実装したBIDコード生成機能を保存しています。

## 📦 含まれるファイル

### コア機能
- **schema.rs**: BIDスキーマ定義（YAML/JSONパース）
- **codegen/generator.rs**: コード生成エンジン
- **codegen/mod.rs**: モジュール定義

### 各言語向け生成ターゲット
- **codegen/targets/vm.rs**: VM用バイトコード生成
- **codegen/targets/wasm.rs**: WebAssembly生成（最も詳細）
- **codegen/targets/llvm.rs**: LLVM IR生成（スタブ）
- **codegen/targets/python.rs**: Pythonバインディング（スタブ）
- **codegen/targets/typescript.rs**: TypeScript定義（スタブ）

## 🎯 用途

将来的に以下の用途で活用可能：

1. **プラグインの多言語対応**: 
   - C以外の言語でプラグイン作成
   - 各言語向けバインディング自動生成

2. **バックエンド統合**:
   - VM/WASM/LLVM向けの統一インターフェース
   - 外部関数定義の一元管理

3. **型安全性向上**:
   - スキーマベースの型チェック
   - コンパイル時の整合性検証

## 📝 メモ

- 現在は使用していない（既存のnyash.tomlベースが動作中）
- cli.rsとrunner.rsへの大幅変更は含まれていない（別フォルダ保存）
- 必要に応じて段階的に統合可能