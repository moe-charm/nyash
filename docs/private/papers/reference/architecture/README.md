# Nyash VM 実行基盤ガイド

最終更新: 2025-08-21（Phase 9.78 系対応）

本書は Nyash の VM バックエンド（MIR 実行機構）と、その周辺の実装・拡張ポイントをまとめた開発者向けドキュメントです。

## 全体像
- 入力: `MirModule`（`mir::MirCompiler` が AST から生成）
- 実行: `backend::vm::VM`
  - `execute_module` → `execute_function` → 命令列を逐次 `execute_instruction`
- ランタイム DI: `NyashRuntime`
  - `box_registry`（統一 BoxFactory 経由の生成）
  - `box_declarations`（ユーザー定義 Box の宣言）
- ライフサイクル: `ScopeTracker` により関数入退出で `fini()` を呼ぶ

## 主要ファイル
- `src/backend/vm.rs` … VM 本体（命令ディスパッチ、Call/BoxCall、NewBox ほか）
- `src/mir/*` … MIR 命令定義・Builder・Function/Module 管理
- `src/runtime/nyash_runtime.rs` … ランタイムコンテナ（DI 受け皿）
- `src/box_factory/*` … Builtin/User/Plugin の各 Factory 実装
- `src/runtime/plugin_loader_v2.rs` … BID-FFI v2 ローダ（ExternCall/Plugin 呼び出し）

関連ドキュメント
- 動的プラグインの流れ: [dynamic-plugin-flow.md](./dynamic-plugin-flow.md)
- 命令セットダイエット: [mir-26-instruction-diet.md](./mir-26-instruction-diet.md)
- MIR→VMマッピング: [mir-to-vm-mapping.md](./mir-to-vm-mapping.md)

## 実行フロー（概略）
1) Nyash コード → Parser → AST → `MirCompiler` で `MirModule` を生成
2) `VM::with_runtime(runtime)` で実行（`execute_module`）
3) 命令ごとに処理:
   - `Const/Load/Store/BinOp/...` など基本命令
   - `NewBox`: 統一レジストリ経由で Box 生成
   - `Call`: `"{Box}.{method}/{N}"` の関数名で MIR 関数呼び出し
   - `BoxCall`: Box の種類で分岐（ユーザー定義/ビルトイン/プラグイン）
   - `ExternCall`: `env.console`/`env.canvas` 等をローダへ委譲

## Box 生成と種別
- 生成パス（`NewBox`）は `NyashRuntime::box_registry` が担当
  - Builtin: `BuiltinBoxFactory` が直接生成
  - User-defined: `UserDefinedBoxFactory` → `InstanceBox`
  - Plugin: プラグイン設定（`nyash.toml`）に従い BID-FFI で `PluginBoxV2`
- **動的解決の詳細**: [dynamic-plugin-flow.md](./dynamic-plugin-flow.md) を参照

## birth/メソッドの関数化（MIR）
- Lowering ポリシー: AST の `new` は `NewBox` に続けて `BoxCall("birth")` を自動挿入
- Box 宣言の `birth/N` と通常メソッド `method/N` は `"{Box}.{name}/{N}"` の MIR 関数に関数化
  - 命名例: `Person.birth/1`, `Person.greet/0`
  - 引数: `me` が `%0`、ユーザー引数が `%1..N`（`me` は `MirType::Box(BoxName)`）
  - 戻り値型: 明示の `return <expr>` があれば `Unknown`、なければ `Void`（軽量推定）
- `VM` の呼び出し
  - `Call` 命令: 関数名（`Const(String)`）を解決 → `call_function_by_name`
  - `BoxCall` 命令: 下記の種類分岐に委譲

## BoxCall の種類分岐
- ユーザー定義 Box（`InstanceBox`）
  - `BoxCall("birth")`: `"{Box}.birth/{argc}"` を `Call` 等価で実行
  - 通常メソッド: `"{Box}.{method}/{argc}"` を `Call` 等価で実行
- プラグイン Box（`PluginBoxV2`）
  - `PluginLoaderV2::invoke_instance_method(box_type, method, instance_id, args)` を呼び出し
  - 引数/戻り値は最小 TLV でやり取り（タグ: 1=Int64, 2=String, 3=Bool）
  - 戻り値なしは `void` 扱い
- ビルトイン Box
  - 現状は VM 内の簡易ディスパッチ（`String/Integer/Array/Math` を中心にサポート）
  - 将来はビルトインも MIR 関数へ寄せる計画

## ExternCall（ホスト機能）
- `env.console.log`, `env.canvas.*` などを `PluginLoaderV2::extern_call` に委譲
- いまは最小実装（ログ出力・スタブ）。将来は BID-FFI 由来の外部機能にも接続予定

## ライフサイクル管理（ScopeTracker）
- `VM` は関数入退出で `push_scope()/pop_scope()` を実行
- 退出時に登録 Box を `fini()`（`InstanceBox`/`PluginBoxV2`）
- Interpreter でも同思想で `restore_local_vars()` にて `fini()` 呼び出し

## ランタイム DI（依存注入）
- `NyashRuntime`
  - `box_declarations`: AST から収集（Box 宣言）
  - `box_registry`: Builtin/User/Plugin の順で探索・生成
- Runner（CLI 実行）
  - AST パース後、Box 宣言を `runtime.box_declarations` へ登録
  - `UserDefinedBoxFactory` をレジストリに注入 → VM を `with_runtime(runtime)` で起動

## 最適化
- 置換: `new X(...).m(...)` → 直接 `Call("X.m/N", me+args)` に最適化
- 拡張余地: 変数へ束縛してからの `.m()` も静的に決まる範囲で `Call` 化可能
- 戻り値型: 軽量推定。将来は `MirType` 推論/注釈の強化

## 制約と今後
- ビルトインのメソッドはまだ簡易ディスパッチ（MIR 関数化は未）
- プラグインの TLV は最小型（Int/String/Bool）のみ。拡張予定
- 例外（throw/catch）は簡易扱い（将来の unwind/handler 連携は別設計）

## テスト
- 単体/E2E（抜粋）: `src/backend/vm.rs` の `#[cfg(test)]`
  - `test_vm_user_box_birth_and_method` … `new Person("Alice").greet()` → "Hello, Alice"
  - `test_vm_user_box_var_then_method` … 変数に束縛→メソッド→戻り値（11）
  - `test_vm_extern_console_log` … ExternCall の void 確認
- 実行例
  - `cargo test -j32`（plugins 機能や環境依存に注意）

## 実行（参考）
- VMルート実行（Runner 経由）
  - `nyash --backend vm your_file.nyash`
- WASM（ブラウザ）
  - plugins は無効。ExternCall はスタブ。MIR 関数はそのまま再利用される設計

---
開発ポリシー: 小さく安全に最適化 → MIR/VM の共有ロジックを増やす → Extern/Plugin を段階統合 → WASM/LLVM/JIT へ横展開。
