# Nyash 設計図（アーキテクチャ概要）

最終更新: 2025-08-21（Phase 9.78b〜3 反映）

本書はNyashの実装設計を、バックエンド共通で理解できる単一ドキュメントとしてまとめたもの。言語コア、MIR、インタープリター/VM統合、ランタイム/プラグイン、ビルドと配布の観点を俯瞰する。

## レイヤー構成

- 構文/AST: `tokenizer`, `parser`, `ast`
- モデル層: `core::model`（BoxDeclaration等の純粋データ）
- ランタイム層: `runtime`（UnifiedBoxRegistry, PluginLoader, NyashRuntime）
- 実行戦略層: `interpreter`（AST実行）/ `mir`+`backend::vm`（MIR実行）/ 将来 `wasm`/`llvm`
- 付帯基盤: `box_factory`, `instance_v2`, `scope_tracker`, `boxes/*`, `stdlib`

## コア概念

- Everything is Box: すべての値はBox（ビルトイン、ユーザー定義、プラグイン）
- 統一コンストラクタ: `birth(args)`（packはビルトイン継承内部用に透過化）
- 明示デリゲーション: `box Child from Parent` と `from Parent.method()`
- 厳密変数宣言/スコープ安全: `local`, `outbox`、スコープ退出時の`fini`一元化

## モデル層（core::model）

- `BoxDeclaration` を `interpreter` から分離し `core::model` に移動
  - name, fields, methods, constructors(birth/N), extends, implements, type_parameters
  - 実行戦略非依存の純粋データ

## ランタイム層（runtime）

- `NyashRuntime`
  - `box_registry: UnifiedBoxRegistry`（ビルトイン/ユーザー定義/プラグインを順序付き検索）
  - `box_declarations: RwLock<HashMap<String, BoxDeclaration>>`
  - BuilderでDI（`with_factory`）可能。Interpreter/VMから共有・注入できる
- `UnifiedBoxRegistry`
  - `Arc<dyn BoxFactory>` の列で優先解決（builtin > user > plugin）
  - `create_box(name, args)` の統一エントリ
- `BoxFactory`
  - builtin: 全ビルトインBoxの生成
  - user_defined: `BoxDeclaration`に基づき`InstanceBox`生成（birthは実行戦略側で）
  - plugin: BID-FFI準拠のプラグインBox（将来のExternCall/MIR接続）

## 実行戦略（Interpreter / VM）

- Interpreter（AST実行）
  - `SharedState` は段階的に分解し、宣言等を `NyashRuntime` に寄せ替え
  - 生成は統一レジストリへ委譲、コンストラクタ実行は`birth/N`のASTを実行

- VM (MIR実行)
  - `VM::with_runtime(runtime)` でDI、`NewBox`は`runtime.box_registry.create_box`へ
  - `ScopeTracker`でスコープ退出時に`fini`（InstanceBox/PluginBox）
  - birth/メソッドのMIR関数化（Phase 2/3）：
    - Builderが `new` を `NewBox` + `BoxCall("birth")` に展開
    - Box宣言の `birth/N` と通常メソッド(`method/N`)を `"{Box}.{name}/{N}"` のMIR関数へ関数化
    - VMの`BoxCall`は `InstanceBox` なら該当MIR関数へディスパッチ（me + 引数）

## MIR（中間表現）

- 目的: バックエンド共通の最適化/実行基盤（VM/LLVM/WASM/JIT）
- Builder
  - AST→MIR lowering。`ASTNode::New`→`NewBox`(+ `BoxCall("birth")`)
  - `ASTNode::BoxDeclaration` の `constructors` / `methods` をMIR関数化
  - if/loop/try-catch/phi等の基本構造を提供
- VM
  - Stackベースの簡易実装→順次強化中
  - `call_function_by_name` による関数呼び出しフレームの最小実装

## インスタンス表現（InstanceBox）

- 統一フィールド`fields_ng: HashMap<String, NyashValue>`
- メソッドASTを保持（ユーザー定義時）
- `fini()`による簡易解放（将来、リソースBoxは明示やRAII連携）

## ライフサイクル統一（fini）

- Interpreter: スコープ復帰時に`InstanceBox.fini()`等を呼ぶ
- VM: `ScopeTracker`で関数入退出時に登録Boxを`fini`

## プラグイン（BID-FFI）

- v2ローダ（`runtime::plugin_loader_v2`）とテスター完備
- 目標: MIRの`ExternCall`→ローダに接続し、VM/LLVM/WASMで共通パス

## Runner/ビルド

- VMモード:
  1) ASTパース
  2) ランタイムにBox宣言収集 + UserDefinedBoxFactory登録
  3) MIRコンパイル
  4) VMを`with_runtime`で起動し実行

## 進行中フェーズと方針

- Phase 9.78b: Interpreter/VMのモデル・ランタイム共有（完了）
- Phase 2/3（実質）: birth/メソッドのMIR関数化とVMディスパッチ（実装済・基本動作）
- 次: 
  - BoxCall→Callへの段階的置換（型決定済みのとき）
  - ExternCallの実装（VM→プラグイン）
  - WASM/LLVMバックエンドへMIR関数の共有

## 参考ファイル

- `src/core/model.rs`（BoxDeclaration）
- `src/runtime/nyash_runtime.rs`（NyashRuntime）
- `src/box_factory/*`（builtin/user_defined/plugin）
- `src/mir/*`（builder/instruction/function/etc.）
- `src/backend/vm.rs`（VM実行）
- `src/interpreter/*`（AST実行）

