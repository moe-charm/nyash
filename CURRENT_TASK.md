# CURRENT TASK (Phase 12 — TypeBox ABI / VTable 統合)

## New: Phase 12.7-B 基本糖衣構文（basic）着手メモ（2025‑09‑04）
- 目的: セルフホス前に `|>`, `?.`, `??`, `+=` 系, `..` を“正規AST”へ正規化する最小導入（可逆・段階導入）。
- スコープ（Week 1）:
  - tokenizer: `??`, `?.`, `|>`, `+=`, `-=`, `*=`, `/=`, `..` を2文字優先で追加
  - parser/sugar.rs: `apply_sugar(ast, &SugarConfig)` 実装（上記の正規化）
  - config: `nyash.toml [syntax] sugar_level=none|basic|full` を読込み、basicのみON
  - tests: `sugar_basic_test.rs` 追加、`tools/smoke_vm_jit.sh` に `NYASH_SYNTAX_SUGAR_LEVEL=basic`
  - docs: phase-12.7/README に basic 実装済みの注記
- 注意:
  - 高階演算子（`/:`, `\:`, `//`）は衝突のため初期見送り（関数名糖衣等で代替検討）。
  - `//` はコメントと衝突。採用しない。
  - パイプラインの規約（関数/メソッド解決）はREADMEに明記。


このファイルは Phase 12 の実装要点を短く保つために再編しました。詳細ログは docs 配下に移管します。

- ドキュメント: `docs/development/roadmap/phases/phase-12/{README.md, PLAN.md, TASKS.md}`
- ABI 最小コア: `docs/reference/abi/NYASH_ABI_MIN_CORE.md`

## 概要（Executive Summary / 圧縮版）
- 目的: ユーザー/プラグイン/内蔵を TypeBox+VTable で統一し、VM/JIT/WASM の同一実行を実現。
- 現状: Phase 12 完了（JIT/VM FunctionBox 呼び出し統一、Lambda→FunctionBox 値化、最小Builtin方針）。WASM v2 最小ディスパッチ導入。

次フェーズ: MIR統一 + リファクタリング（Phase 11.8/13）
- 目標: 1ファイル1000行以内を目安に分割・整理。他AI/将来タスクが読みやすい構造へ。
- 制約: 挙動不変・公開API維持・段階的分割＋逐次ビルド/テスト。

Compact Snapshot（2025‑09‑03/Finalize）
- Builtin（最小/条件付き）: 既定は plugins-only。wasm32 と cargo test では自動でBuiltin登録。任意で `--features builtin-core` で明示ON。
- P2P 安定化: `sys.pong` 返信3ms遅延・`ping()`既定300ms。`on_once` 自己送信は deliver直後のflag無効化＋flag配列クリア＋`debug_active_handler_count` の最短確定で安定化。
- FunctionBox 呼び出しの MIR/VM 統一:
  - C1: `ASTNode::Call` 正規化（Lambda以外は `MirInstruction::Call(func,args)`）。
  - C2: VM `execute_call` が 文字列（関数名）/ `FunctionBox`（BoxRef）両対応。`FunctionBox` 実行は interpreter ヘルパで return を正しく伝播。
  - テスト: `src/tests/functionbox_call_tests.rs`（インタープリタ）、`src/tests/vm_functionbox_call.rs`（MIR→VM）。

- Lambda→FunctionBox 値化（最小）: `MirInstruction::FunctionNew` を導入。簡易キャプチャ/`me`を MIR で値として保持。
- JIT: `Call`→`nyash_fn_call0..8` シムで FunctionBox 実行をブリッジ（最大8引数）。
- LLVM: 本実装は低優先のため保留中（モック実行経路で FunctionNew/Call をサポート）。

次タスク（新フェーズ: リファクタリング / 圧縮）
- A1) `backend/vm_instructions.rs` を機能別へ分割（目安: <1000行）
- A2) `runtime/plugin_loader_v2.rs` を役割別へ分割（ffi_tlv/registry/host_bridge/loader）
  - 完了: `src/runtime/plugin_loader_v2/` へ分割（enabled/{types,loader,globals}.rs + stub.rs）。公開APIは据え置き（`PluginLoaderV2`/`PluginBoxV2`/`make_plugin_box_v2`/`get_global_loader_v2` 等）
  - A3) `backend/vm.rs` の状態/実行/GC/診断を分離
    - 進捗: `ControlFlow` を `src/backend/vm_control_flow.rs` に抽出し、`vm.rs` から再エクスポートで互換維持（次は実行ループの段階切り出し）
- B1) `mir/builder.rs` の `build_expression` 群を `builder/exprs.rs` へ抽出
- B2) `interpreter/plugin_loader.rs` 分割（scan/link/abi/util）
- C) 命名/コメント整備・公開APIのre-export最適化・軽微な util 抽出

実行メモ（抜粋）
- ユニット: `cargo test`（test時はBuiltin自動ON）
- E2E: `cargo test --features e2e -- --nocapture`（普段は無効）
- FunctionBoxテスト: 
  - `cargo test --lib functionbox_call_tests -- --nocapture`
  - `cargo test --lib vm_functionbox_call -- --nocapture`

運用フラグ
- 既定: `plugins-only`（Builtin未登録）
- 自動ON: `wasm32` / `test` では Builtin 有効
- 手動ON: `--features builtin-core`（Builtin最小コアを有効化）


## 次タスク（優先順）
- フェーズM（MIR Core‑13 統一・挙動不変）
  - M1) Core‑13 を既定ON（nyash.toml [env] 推奨: NYASH_MIR_CORE13=1, NYASH_OPT_DIAG_FORBID_LEGACY=1）
  - M2) BuilderをCore‑13準拠に調整（ArrayGet/Set・RefGet/Set・PluginInvokeをemitしない。BoxCallへ正規化）
  - M3) OptimizerでUnary→BinOpを常時変換、Load/StoreのSSA置換（最終MIRから旧命令を排除）
  - M4) Core‑13検証を追加（最終MIRに旧命令が存在したらエラー）
  - M5) VM/JIT/AOTのBoxCall fast‑path/vtable維持（setはBarrier必須）
- フェーズA（安全分割・挙動不変）
  - A1) vm_instructions を 10前後のモジュールへ分割（consts/arith/compare/flow/call/boxcall/array/refs/future/newbox/print_debug）
    - 現状: `src/backend/vm_instructions/{core,call,newbox,function_new,extern_call,boxcall,plugin_invoke}.rs` に分割済み
  - A2) plugin_loader_v2 を 4〜5ファイルへ分割（ffi_tlv/registry/host_bridge/loader/errors）
  - A3) vm を 3〜4ファイルへ分割（state/exec/gc/format）
- フェーズB（読みやすさ整形）
  - B1) mir/builder の expr系切り出し
 - B2) interpreter/plugin_loader の役割分離
- フェーズC（軽整理）
  - 命名/コメント整備、公開API re-export、1000行未満へ微調整

## 次のゴール（Phase W — Windows JIT(EXE) × Egui 起動）
目的: Windows 上で Cranelift JIT（必要に応じ EXE ラッパ）で Egui ウィンドウを起動する最小スモークを確立。成功の再現性を高め、論文化に向けて手順を固定。

推奨タスク（開いたら一発で着手できる指示）
- W1) 準備（Windows）
  - `cargo build --release --features cranelift-jit`
  - Egui プラグイン DLL をビルド（`plugins/nyash-egui-plugin`）し、`nyash.toml` の `[plugins]/[libraries]` と検索パスが DLL 配置と一致しているか確認。
  - 必要なら PATH 追加または `nyash.toml [plugin_paths]` に DLL ディレクトリを追記。
- W2) 最小アプリ（例: apps/ny-egui-hello/main.nyash）
  - `open(width,height,title)` → `uiLabel("Hello, Nyash Egui")` → `run()` の極小シナリオを用意。
  - 実行: `.
    target\release\nyash --backend jit apps\ny-egui-hello\main.nyash`
- W3) HostBridge 推奨トグル（任意）
  - `NYASH_JIT_HOST_BRIDGE=1`（JIT から HostBridge を優先利用。BoxCall→HostCall/Bridge を強制）
  - `NYASH_USE_PLUGIN_BUILTINS=0`（HostCall 優先の確認時に推奨）
- W4) スモークスクリプト化（PowerShell）
  - `tools/egui_win_smoke.ps1` を追加: ビルド→PATH 調整→実行までを一括化（再現性向上）。
- W5) 追加JITスモーク（Core‑13 準拠）
  - Instance: `getField/setField` の往復（HostBridge 経由）
  - Extern: `console.log` の起動（ログ確認）

判定条件（Done）
- Windows で Egui ウィンドウが起動（タイトル/ラベル表示確認）
- PowerShell スクリプトでワンコマンド起動が再現（DLL 探索含め手戻りゼロ）
- Core‑13 JIT スモーク（Array/Map/Instance/Extern）の最小セットが緑

参考メモ（現状の統一状況）
- Core‑13 は Builder→Optimizer→Compiler の三段ガードで旧命令ゼロを強制（最終MIR厳格チェックも導入済み）。
- JIT の BoxCall fast‑path は HostCall 優先に整理（PluginInvoke は保険/フォールバック）。
- vtable スタブは Map/String/Array/Console をヘルパ化済み（挙動不変・Barrier維持）。

## 完了（Done）
- TypeBox ABI 雛形: `src/runtime/type_box_abi.rs`
- TypeRegistry 雛形: `src/runtime/type_registry.rs`
  - Array: get(100)/set(101)/len,length(102)
  - Map: size(200)/len(201)/has(202)/get(203)/set(204)
  - String: len(300)
  - Console: log(400)/warn(401)/error(402)/clear(403)
- VM vtable 優先スタブ: `execute_boxcall` → `try_boxcall_vtable_stub`（`NYASH_ABI_VTABLE=1`）
  - Instance: getField/setField/has/size
  - Array/Map/String: 代表メソッドを直接/host経由で処理
  - PluginBoxV2 受信時でも Array/Map/String を vtable→host.invoke で実行（set は GC バリア）
- MapBox 文字列キー互換: get/has の第1引数が String なら getS/hasS を常時使用（plugin_loader_v2/VM）
- Console.readLine フォールバック（VM/Plugin 両経路）: stdin 読み込み/EOF=Null 返却で無限ループ防止
- WASM v2 統一ディスパッチ（最小）: console/array/map のスロット対応

進捗アップデート（Phase 13 / 2025-09-03）
- A1 完了: `src/backend/vm_instructions.rs` をモジュール分割
  - 新構成: `src/backend/vm_instructions/{mod.rs, core.rs, call.rs, newbox.rs, function_new.rs, extern_call.rs, boxcall.rs, plugin_invoke.rs}`
  - 役割:
    - `core.rs`: const/binop/unary/compare/print/ctrl/type/phi/mem/array/refs/weak/barriers/exn/await
    - `call.rs`: 関数呼び出し（FunctionBox対応）
    - `newbox.rs`: NewBox
    - `function_new.rs`: Lambda→FunctionBox 値化
    - `extern_call.rs`: ExternCall（env./registry経路）
    - `boxcall.rs`: BoxCall + VTableスタブ + 汎用フォールバック
    - `plugin_invoke.rs`: PluginInvoke（強制プラグイン経路）
  - 可視性: `pub(crate)`/`pub(super)` 調整、`dispatch.rs` 経由の呼び出し互換維持
  - ビルド: `cargo build` 緑（警告のみ／挙動不変）

- A2 完了: `runtime/plugin_loader_v2.rs` をサブモジュール化
  - 新構成: `src/runtime/plugin_loader_v2/`
    - `mod.rs`（cfg切替）
    - `enabled/{types.rs, loader.rs, globals.rs}`
      - `types.rs`: `PluginBoxV2`/`PluginHandleInner`/`NyashTypeBoxFfi`、`make_plugin_box_v2`/`construct_plugin_box`
      - `loader.rs`: `PluginLoaderV2`（extern_call/invoke_instance_method/create_box 等のAPIを維持）
      - `globals.rs`: `get_global_loader_v2`/`init_global_loader_v2`/`shutdown_plugins_v2`
    - `stub.rs`: plugins無効/wasm 用スタブ（同名API維持）
  - 公開API/パス互換: 既存の `crate::runtime::plugin_loader_v2::*` 参照はそのまま
  - ビルド: `cargo build` 緑（警告のみ）

- A3 進行中: `backend/vm.rs` の段階分割（最新: 2025-09-04）
  - 第1段: `ControlFlow` を `src/backend/vm_control_flow.rs` に抽出し、`vm.rs` から再エクスポート（完了）
  - 第2段: 実行ループを `src/backend/vm_exec.rs` へ抽出（完了）
    - 移動対象: `VM::execute_module`、`VM::execute_function`、`VM::call_function_by_name`、`VM::execute_instruction`、`VM::print_cache_stats_summary`
    - 可視性調整: `VM` の内部状態（`values/current_function/frame/previous_block/loop_executor/module/instr_counter/exec_start/scope_tracker` 等）を `pub(super)` に変更し、`vm_exec.rs` から安全に参照できるようにした。GCダイアグ用メソッド（`gc_print_roots_breakdown`/`gc_print_reachability_depth2`）も `pub(super)` 化。
    - `ControlFlow` を `pub(crate)` に変更し、`vm_instructions` サブモジュールの `pub(crate)` API と可視性を整合。
    - ビルド: 成功（警告あり）。`private_interfaces`/`unused_*`/`unexpected_cfg` などの警告は機能的影響なし。
  - 第3段: GC ルート管理と診断を `src/backend/vm_gc.rs` へ抽出（完了）
    - 移動対象: `enter_root_region`、`pin_roots`、`leave_root_region`、`gc_site_info`、`gc_print_roots_breakdown`、`gc_print_reachability_depth2`
    - 既存呼び出しは変更なし（同名メソッドとして移設）。旧定義は一時的に `*_old` 名へ退避し、後続で削除予定。
    - 注記: `vm.rs` 内の旧メソッドは一時的に `*_old` 名へリネームし残置（安全移行用）。後続ステップで完全削除予定。
  - 第4段: VM 基本状態を `src/backend/vm_state.rs` へ抽出（完了）
    - 生成・状態・値アクセス・統計・phi 補助
  - 第5段: Box メソッドディスパッチのラッパを `src/backend/vm_methods.rs` へ抽出（完了）
    - `VM::call_box_method` / `call_unified_method` は委譲に変更（実装は `vm_boxcall.rs`）
  - 旧プレースホルダ削除（完了）
    - `execute_module_old_moved` / `*_old` 系を撤去
  - 現在の行数スナップショット: `src/backend/vm.rs` ≈ 973 行（< 1000 行達成）

## 次タスク（2025-09-04 更新）
- フェーズA/B 優先順（影響大→小）
  1) interpreter/plugin_loader.rs を役割別へ分割（B2）
     - ステップ1: ディレクトリ化（plugin_loader/mod.rs へ移行）済
     - 構成案: `interpreter/plugin_loader/{scan.rs,link.rs,abi.rs,registry.rs,errors.rs,util.rs}`
     - 互換: `interpreter/plugin_loader/mod.rs` で再エクスポート、既存API維持
  2) backend/llvm/compiler.rs の機能別分割
     - `llvm/{init.rs,types.rs, instr_*.rs}`（算術/比較/制御/メモリ/呼出/Box/配列/Map/文字列/Ref/Weak/Future/Phi/Cast/Type）
     - `mod.rs` で `Compiler` の `impl` を分散読み込み
  3) mir/builder.rs の `builder/{exprs.rs,stmts.rs,decls.rs,utils.rs}` への抽出

注記: 公開APIは維持。各段階ごとに `cargo build` と限定ユニットで確認して進める。

## 残タスク（To‑Do）
1) MIR Core‑13 統一（M1〜M5）＋ スモーク修正
2) リファクタフェーズA/B/C 実施（段階コミット＋スモーク）
3) ドキュメント更新（Phase 11.8 の README/PLAN/TECHNICAL_SPEC と CIポリシー）
4) LLVM（本実装）は低優先：BoxCall シムのinlining設計だけ先行

## 実行コマンド（サマリ）
- ビルド: `cargo build --release --features cranelift-jit`
- ny-echo（Script/VM/JIT）
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash apps/ny-echo/main.nyash`
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash --backend vm apps/ny-echo/main.nyash`
  - `printf "Hello\n" | NYASH_CLI_VERBOSE=0 ./target/release/nyash --backend vm --jit-exec --jit-hostcall apps/ny-echo/main.nyash`
- ベンチ（参考）
  - `NYASH_CLI_VERBOSE=0 ./target/release/nyash [--backend vm|--jit-exec --jit-hostcall] apps/ny-array-bench/main.nyash`
  - `NYASH_CLI_VERBOSE=0 ./target/release/nyash [--backend vm|--jit-exec --jit-hostcall] apps/ny-mem-bench/main.nyash`

## トレース/環境変数（抜粋）
- ABI: `NYASH_ABI_VTABLE=1`, `NYASH_ABI_STRICT=1`
- VM: `NYASH_VM_PIC_STATS`, `NYASH_VM_PIC_TRACE`, `NYASH_VM_VT_TRACE`
- JIT: `NYASH_JIT_DUMP`, `NYASH_JIT_TRACE_BLOCKS`, `NYASH_JIT_TRACE_BR`, `NYASH_JIT_TRACE_SEL`, `NYASH_JIT_TRACE_RET`

---
詳細な履歴や議事録は docs 配下の Phase 12 セクションを参照してください。

## フェーズ13（リファクタ）方針・成功条件
- 方針: 公開APIを維持しつつ内部構造を機能別に分割。1ファイル1000行以内を目安に段階導入。
- 手順: 1モジュールずつ分割→ビルド→限定ユニット/スモーク→次へ。
- 成功条件:
  - 大規模ファイル（>1000行）が解消（vm_instructions / plugin_loader_v2 / vm / builder）
  - ビルド/主要ユニットが従来通り通る（挙動不変）
  - 他AI/将来タスクが読みやすいレイアウト（役割ごとに参照しやすい）

Docs（Phase 12 直近）
- [x] Minimal Core ABI方針の文書化（NYASH_ABI_MIN_CORE.md）
- [ ] TECHNICAL_DECISIONSの最小ABI/API交渉・互換・安全の章を精緻化（進行中）
- [ ] PLAN/READMEへのリンク整備と“同一実行テスト”の詳細化

Phase 12 ゴール（検証観点）
- Cross-backend 同値性: 同一プログラム（Nyashコード）が VM と JIT で同一の最終結果・ログ・副作用（Box状態）を生む。
- ゴールデン/スモーク: 共通テストハーネスで VM/JIT を同条件で走らせ比較（差分があれば落とす）。

## 残件・課題と対応方針（2025-09-03）

- VMユニット赤の原因と対応（plugins-onlyでBuiltin未登録）
  - 症状: Array/Map などの生成で Unknown Box type（プラグインのみのレジストリ）。
  - 対応: 既定を Builtin 登録に戻し、plugins-only は feature 化。
    - 実装: BuiltinBoxFactory を追加し、NyashRuntime/UnifiedRegistry 初期化時に登録（plugins-only 時はスキップ）。

## 引継ぎメモ（再起動用 / 2025-09-04）

- 進捗サマリ
  - A3(vm) 分割 S1/S2 完了: `vm_exec.rs`/`vm_gc.rs`/`vm_state.rs`/`vm_methods.rs` 抽出、`*_old` 削除。
    - 現在: `src/backend/vm.rs` ≈ 973 行（<1000行）。ビルド成功（警告のみ）。
  - B2(interpreter/plugin_loader) 着手: ディレクトリ化＋下位ファイル用意。
    - 変更: `src/interpreter/plugin_loader.rs` → `src/interpreter/plugin_loader/mod.rs` へ移動（API互換維持）。
    - 追加: 下位モジュール（現時点では未読み込みのため重複は未発生）
      - `src/interpreter/plugin_loader/types.rs`（PLUGIN_CACHE／LoadedPlugin／PluginInfo／各Handle）
      - `src/interpreter/plugin_loader/proxies.rs`（File/Math/Random/Time/DateTime 各 Proxy）
      - `src/interpreter/plugin_loader/loader.rs`（PluginLoader: load_*/create_* エントリ）
  - B1(builder) 続行: `build_static_main_box` / `build_box_declaration` を `src/mir/builder/decls.rs` に抽出。
    - `src/mir/builder.rs` は `mod decls;` を追加し、委譲。ビルド（`cargo test --no-run`）成功。
  - A2(plugin_loader_v2) 進捗: `enabled/{errors.rs, host_bridge.rs}` を新設し、`loader.rs` からエラー変換と invoke ブリッジを抽出。
    - `errors.rs`: `from_fs`/`from_toml`/`or_plugin_err` ヘルパ
    - `host_bridge.rs`: `invoke_alloc`（TLV 呼び出しの小ラッパ）
    - `enabled/mod.rs` に `mod errors; mod host_bridge;` を追加し、ビルド確認済み。

- 再開手順（最短ルート）
  1) `src/interpreter/plugin_loader/mod.rs` を分割モードに切替
     - 先頭に `mod types; mod proxies; mod loader;` を追加
     - 末尾付近で `pub use` を追加（互換維持）:
       - `pub use types::{PLUGIN_CACHE, LoadedPlugin, PluginInfo, FileBoxHandle, MathBoxHandle, RandomBoxHandle, TimeBoxHandle, DateTimeBoxHandle};`
       - `pub use loader::PluginLoader;`
       - `pub use proxies::{FileBoxProxy, MathBoxProxy, RandomBoxProxy, TimeBoxProxy, DateTimeBoxProxy};`
  2) `mod.rs` 内の重複定義を削除
     - `lazy_static!` PLUGIN_CACHE／構造体（LoadedPlugin/PluginInfo/各Handle）／各Proxy実装／PluginLoader 実装を、types/proxies/loader に移った分だけ除去。
     - 目標: `mod.rs` にはドキュメント＋`mod`/`pub use` のみ残す。
  3) ビルド確認（feature注意）
     - `cargo build`（通常）
     - 動的ロード経路は `--features dynamic-file` が必要な箇所あり。
  4) 呼び出し側の互換確認
     - 例: `src/interpreter/methods/math_methods.rs` は `crate::interpreter::plugin_loader::{MathBoxProxy, RandomBoxProxy, TimeBoxProxy, DateTimeBoxProxy}` を参照 → `pub use` により互換のはず。

- 注意点
  - 分割ファイル（types/proxies/loader）は現在 `mod.rs` から未参照（意図的）。上記(1)の `mod` 追加で参照されコンパイル対象になります。
  - `#[cfg(feature = "dynamic-file")]` の条件分岐により libloading/FFI シンボルが有効化されます。ビルド時の feature セットに留意。
  - 公開API互換を維持すること（`crate::interpreter::plugin_loader::*` の既存呼び出しが動作するよう `pub use` を整備）。

- 次の大物（plugin_loader 完了後）
  - llvm/compiler.rs: `llvm/{init.rs,types.rs,instr_*.rs}` へ段階分割
  - mir/builder.rs: `builder/{exprs.rs,stmts.rs,decls.rs,utils.rs}` へ抽出

（この引継ぎに沿って再開すれば、直ちに plugin_loader の分割完了→ビルド確認まで進められます）
    - 追加: Cargo.toml に `plugins-only` feature を定義。

- P2PBox の once/ping 安定化方針
  - once: deliver 後にフラグ無効化（現行仕様維持、分岐独立を再確認）。
  - ping: `sys.pong` 返信スレッドの遅延を 3ms に調整、`ping()` 既定タイムアウトは 300ms に。
  - 目的: 記録タイミング（last_from/last_intent）競合の低減とCI安定化。

- FunctionBox 呼び出しの MIR/VM 統一（段階計画）
  - C1) MIR 正規化: `ASTNode::Call` は Lambda 以外を `MirInstruction::Call(func, args)` に正規化（完了）。
  - C2) VM 実行: `func` が 文字列なら名前呼び出し、`FunctionBox` ならインタープリタヘルパで本体実行（完了）。
  - C3) LLVM/JIT: C2 のシムを後続で移植（VMで安定化→JITへ）。Lambda→FunctionBox 値化は小PRで導入予定（MIRのみで関数値を生成できるように）。

- テスト整理
  - E2E は feature で切替済み（`--features e2e`）。
  - ユニットは初期化ヘルパ（Builtin登録）で安定化。plugins-only 依存は `#[cfg(feature="plugins-only")]` で保護。

- ドキュメント/デモ更新
  - FunctionBox ハンドラのデモを追加（`apps/p2p-function-handler-demo`）。
  - function values / captures / `this→me` / `Parent::` / `?` / `peek` のガイドを `docs/development/current/function_values_and_captures.md` に追記済み。


> Quick Resume (Phase 12 bridge)

- Where to look next:
  - Phase 12 overview: docs/development/roadmap/phases/phase-12/README.md
  - Task board: docs/development/roadmap/phases/phase-12/TASKS.md
  - ABI digest: docs/development/roadmap/phases/phase-12/NYASH-ABI-DESIGN.md
  - Refactoring plan: docs/development/roadmap/phases/phase-12/REFACTORING_PLAN.md
- Build/run (VM/JIT):
  - VM: `cargo build --release --features cranelift-jit && ./target/release/nyash --backend vm apps/tests/ny-echo-lite/main.nyash`
  - JIT: `./target/release/nyash --backend jit apps/tests/ny-echo-lite/main.nyash`
- MapBox extensions (VM/JIT): remove/clear/getOr/keys/values/JSON are available; keys/values currently return newline-joined String (shim).
- First refactor candidate: split `src/runner.rs` into `runner/mod.rs` + `runner/modes/*` (see REFACTORING_PLAN.md).


Phase 11.7 へ仕切り直し（会議合意）

- 単一意味論層: MIR→Semantics→{VM/Cranelift/LLVM/WASM} の設計に切替。VMは参照実装、実行/生成はCodegen側で一本化。
- フォールバック廃止: VM→JITの実行時フォールバックは行わない。JITは“コンパイル専用/AOT補助”に限定。
- 共通ABI: handle/i64/ptr 変換、to_bool/compare、タグ分類、invoke（固定/可変）、NyRTシム呼び出しを共通化。
- Cranelift: 軽量JIT/AOTパスを追加し、最小実装からMIR-15を段階的に緑化。LLVM AOT は併存。

Docs: docs/development/roadmap/phases/phase-11.7_jit_complete/{README.md, PLAN.md, CURRENT_TASK.md, MEETING_NOTES.md}

以降は下記の旧計画（LLVM準備）をアーカイブ参照。スモークやツールは必要箇所を段階で引継ぎ。

開発哲学（Box-First）
- 「箱を作って下に積む」原則で進める。境界を先に置き、no-op足場→小さく通す→観測→厳密化の順で段階導入。
- 詳細: docs/development/philosophy/box-first-manifesto.md

次の候補（再開時）
- spawn を本当の非同期化（TLV化＋Scheduler投入）
- JIT/EXE用 nyash.future.spawn_method_h C-ABI 追加

Async Task System (Structured Concurrency)
- Spec/Plan: docs/development/roadmap/phases/phase-11.7_jit_complete/async_task_system/{SPEC.md, PLAN.md}
- 目的: Nyash→MIR→VM→JIT/EXE まで一貫した構造化並行を実現（TaskGroup/Future を Box化）
- 進捗（P1）: FutureBox を Mutex+Condvar 化、await は safepoint+timeout（NYASH_AWAIT_MAX_MS）で無限待ち防止済み。

Update (2025-09-01 late / Async P2 skeleton + P3 await.Result unify)
- P2（雛形）
  - CancellationToken 型（cancel/is_cancelled）を追加: src/runtime/scheduler.rs
  - Scheduler::spawn_with_token / global_hooks::spawn_task_with_token（現状はspawnへ委譲）
  - TaskGroupBox 雛形: src/boxes/task_group_box.rs（API: new/cancel_all/is_cancelled）
- P3（awaitのResult化・第一弾）
  - VM Await: Futureの値を NyashResultBox::Ok で返す（src/backend/vm_instructions.rs）
  - env.future.await（VM/Unified）: Ok(value)／Err("Timeout") を返す（timeoutは NYASH_AWAIT_MAX_MS 既定5秒）
  - JIT Await: await_h は常にハンドル返却→ nyash.result.ok_h で Ok(handle) にラップ
    - 追加: src/jit/extern/result.rs（SYM_RESULT_OK_H）／lowerで await 後に ok_h を差し込み
- Smokes/安全ガード
  - tools/smoke_async_spawn.sh（timeout 10s + NYASH_AWAIT_MAX_MS=5000）。apps/tests/async-spawn-instance は nowait/awaitの安全版
  - ハングした場合もOS timeoutで残存プロセスを避ける

次の実装順（合意があれば即着手）
1) Phase 2: VM 経路への最小配線（暗黙 TaskGroup）
   - nowait の着地点を TaskGroup.current().spawn(...) に切替（内部は現行 spawn_instance を踏む）
   - scope 終了時の cancelAll→joinAll（LIFO）は雛形から段階導入（まずは no-op フック）
2) Phase 3: JIT 側の Err 統一
   - nyash.result.err_h(handle_or_msg) をNyRT/JITに追加し、timeout/キャンセル時に Err 化（await ラッパで分岐）
   - 0/None フォールバックを撤去（VM/Externは実装済み／JITを揃える）
3) Verifier: await 前後の checkpoint 検証
   - Lowerer/パスで ExternCall(env.runtime.checkpoint) の前後挿入・検証ルールを追加
4) CI/Smokes 最終化
   - async-await-min, async-spawn-instance（VM/JIT）、timeout系（await_timeout）の3本を最小温存
   - {vm,jit} × {default,strict} で timeout(10s) ガード

Update (2025-09-01 PM / Semantics unify + Async Phase-2 prep)

- Semantics layer in place and adopted by Interpreter/VM/JIT
  - New: `src/runtime/semantics.rs` with `coerce_to_string` / `coerce_to_i64` and unified `+` ordering
  - Interpreter/VM now delegate to semantics (string concat matches VM path; plugin String toUtf8→toString→fallback)
  - JIT: added hostcall `nyash.semantics.add_hh` (handle,handle) for dynamic add; wired in builder/core
- Cross-backend smokes (Script/VM/JIT via VM+jit-exec)
  - `apps/tests/mir-branch-ret` → 1 (all three)
  - `apps/tests/semantics-unified` → 0 (concat/numeric semantics aligned across backends)
  - Tool: `tools/apps_tri_backend_smoke.sh` (summarizes Result lines for 3 modes)
- Async Phase‑2 (front): minimal spawn API and await path
  - Exposed `env.future.spawn_instance(recv, method_name, args...) -> FutureBox`
    - For now resolves synchronously (safe baseline). Hooked to `PluginHost::invoke_instance_method`
  - `env.runtime.checkpoint` now calls `global_hooks::safepoint_and_poll()` for future scheduler integration
  - `env.future.await(value)` pass‑through unless value is FutureBox (then wait_and_get)
  - `apps/tests/async-await-min`: uses `nowait fut = 42; await fut;` → VM/JIT return 42; Interpreter runs (CLI does not print standardized Result line)

Delta to code
- Added: `src/runtime/semantics.rs`
- Updated: `src/interpreter/expressions/operators.rs`, `src/backend/vm_values.rs` → use semantics layer
- JIT: `src/jit/lower/{builder.rs, core_ops.rs, extern_thunks.rs}`, `src/jit/extern/collections.rs`
- NyRT C‑ABI: `crates/nyrt/src/lib.rs` exported `nyash.semantics.add_hh`
- Runtime externs: `src/runtime/plugin_loader_v2.rs` added `env.future.spawn_instance`, improved `env.runtime.checkpoint`
- Hooks/Scheduler: `src/runtime/global_hooks.rs` (safepoint+poll, spawn_task stub), `src/runtime/scheduler.rs` (single‑thread scheduler API scaffold present)
- Smokes/tools: `apps/tests/semantics-unified`, `apps/tests/gc-sync-stress`, `tools/apps_tri_backend_smoke.sh`

Open items / Next steps (proposed)
1) True async spawn (Phase‑2 complete)
   - Change `spawn_instance` to queue a task (Scheduler) instead of inline: capture `recv(type_id/instance_id)`, `method_name`, and TLV‑encoded args
   - On run: decode TLV and `invoke_instance_method`, set Future result; ensure safepoint via `checkpoint`
   - Add NyRT C‑ABI `nyash.future.spawn_method_h` for JIT/EXE and wire lowering
2) Interpreter CLI result normalization (optional)
   - Align “Result:” line for static box Main return to make Script mode consistent with VM/JIT in smokes
3) Keep smokes minimal (avoid bloat); current trio is sufficient for gating

—
Update (2025-09-01 AM / JIT handoff follow-up)

- Cranelift 最小JITの下地は進捗良好（LowerCore→CraneliftBuilder 経路）
  - Compare/Branch/Jump、最小Phi（block params）、StackSlotベースの Load/Store は実装済み（builder.rs/core.rs）。
  - ExternCall(env.console.log/println) の最小橋渡し（ConsoleBox.birth_h→by-id invoke）を追加済み。
- 追加スモーク（jit-direct 用／I/Oなし）
  - apps/tests/mir-store-load: x=1; y=2; x=x+y; return x → 3
  - apps/tests/mir-phi-min: if(1<2){x=10}else{x=20}; return x → 10
  - apps/tests/mir-branch-multi: 入れ子条件分岐 → 1
  - 実行例: `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-store-load/main.nyash`
  - jit-direct 分岐/PHI 根治: 単一出口＋BlockParam 合流で安定（fast-path は常時有効）。

変更点（犯人切り分けと根治のための構造改革＋ログ）
- CraneliftBuilder（jit-direct 経路）
  - 単一出口＋明示合流（SSA）へ切替（途中段階→完了段階へ移行中）
    - ret 合流ブロックを導入し、BlockParam(i64) で戻り値を受け渡し。
    - すべての `emit_return` を「i64 正規化 → `jump(ret, [val])`」に統一（return 命令は合流側だけ）。
    - end_function 側で ret 合流ブロックに切替し、BlockParam を f64 変換（必要時）のうえ return。
  - Compare/Branch/Select まわり
    - Compare 結果をローカルスロットへ保存→Branch/Select 時に確実にロード（スタック取りこぼし排除）。
    - `br_if` 直前に cond を b1 正規化（i64→b1）。
  - デバッグ用 extern を登録（必要時のみ）
    - `nyash.jit.dbg_i64(tag: i64, val: i64) -> i64`（値観測用）
    - `nyash.jit.block_enter(idx: i64) -> void`（ブロック入場ログ）
- LowerCore（return 値の堅牢化）
  - Return 値が known/param/slot 経路に乗らない場合、同一ブロックの Const 定義をスキャンして materialize。
  - Fast-path（読みやすさ＆単純化）: then/else が定数 return の場合、`select(cond, K_then, K_else)`→`emit_return` に縮約（常時有効）。

診断ログ（必要時のみ ON）
- `NYASH_JIT_TRACE_BLOCKS=1` … ブロック入場ログ（`[JIT-BLOCK] enter=<idx>`）
- `NYASH_JIT_TRACE_BR=1` …… br_if の cond 有無（`[JIT-CLIF] br_if cond_present=...`）
- `NYASH_JIT_TRACE_SEL=1` … select の cond/then/else 値（tag=100/101/102）
- `NYASH_JIT_TRACE_RET=1` … return の値: emit_return 直前（tag=201）、ret 合流（tag=200）

再現・実行コマンド（jit-direct）
- 分岐の最小: `apps/tests/mir-branch-ret/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 NYASH_JIT_DUMP=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - 診断ON: `NYASH_JIT_THRESHOLD=1 NYASH_JIT_DUMP=1 NYASH_JIT_TRACE_RET=1 NYASH_JIT_TRACE_BLOCKS=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
- PHI最小: `apps/tests/mir-phi-min/main.nyash`
- 複合分岐: `apps/tests/mir-branch-multi/main.nyash`

期待と現状（2025-09-01 PM 時点）
- 統合JIT（`--backend cranelift`）: OK（`mir-branch-ret`→1）。
- jit-direct: 単一出口＋BlockParam 合流の配線で安定化を確認（cond/then/else も正常）。tag=201/200 は一致。

検証結果（jit-direct / 2025-09-01 実行）
- `apps/tests/mir-branch-ret`: Result=1, [JIT-DBG] 201=1 / 200=1
- `apps/tests/mir-phi-min`: Result=10, [JIT-DBG] 201=10 / 200=10
- `apps/tests/mir-branch-multi`: Result=1, [JIT-DBG] 201=1 / 200=1
- `apps/tests/mir-nested-branch`: Result=1, [JIT-DBG] 201=1 / 200=1
- `apps/tests/mir-phi-two`: Result=50

次のタスク（仕上げ）
1) LowerCore fast‑path/select のガード整理（`NYASH_JIT_FASTPATH_SELECT` の簡素化）。
2) b1 返り値 ABI を有効化する場合の経路確認（feature `jit-b1-abi`）。
3) ドキュメント整備（CraneliftBuilder 単一出口方針と TRACE 変数の最終化）。

Update (2025-09-02 / JIT seal・PHI安定化 + builder分割 進捗)

- 完了（JIT / jit-direct 最小3本グリーン）
  - seal 管理の一本化（途中seal撤廃、end_functionで最終seal）。
  - PHI(min) 合流: 事前スキャン→ensure_block_params、br/jump時のargs/append秩序確立。
  - Return の安定化（known/param/slot優先、フォールバック const materialize）。
  - 診断ログ整備（NYASH_JIT_DUMP/TRACE_*）。
- スモーク（debug/release）: mir-branch-ret=1, mir-phi-min=10, mir-branch-multi=1。

- 追加（2025-09-02 PM / fast-path 簡素化）
  - Branch fast-path を常時有効化（両後続が i64 定数 return の場合に限り `select+return` で縮約）。
  - 環境変数 `NYASH_JIT_FASTPATH_SELECT` は不要になりました（存在しても無視）。
  - jit-direct の3本スモーク（debug/release）で回帰なしを確認。

- リファクタリング（builder 1,000行目安に向けて段階実施）
  - 分離済み:
    - `src/jit/lower/builder/noop.rs`（NoopBuilder）
    - `src/jit/lower/builder/object.rs`（AOT .o 用 ObjectBuilder、finish対応）
    - `src/jit/lower/builder/rt_shims.rs`（nyash_jit_dbg_i64 等の小シム群）
    - `src/jit/lower/builder/tls.rs`（clif_tls と TLS 呼び出しヘルパ）
  - 動作維持: pub use で既存パス互換。jit-direct スモーク通過。

  - 追加（2025-09-02 PM / MIR builder 分割の第一段）
    - Calls 抽出: `src/mir/builder/builder_calls.rs` 新設。
      - 移動: `build_function_call` / `build_method_call` / `build_from_expression` / `lower_method_as_function` / `lower_static_method_as_function` + 補助 `parse_type_name_to_mir` / `extract_string_literal`。
      - 依存を明示（`use crate::mir::{TypeOpKind, slot_registry};`）。
    - Stmts ラッパ導入: `src/mir/builder/stmts.rs` 新設。
      - 現状は `build_*` 系をラッパで委譲（`*_legacy`）し、動作回帰チェックを優先。
    - ビルド/スモーク: release + jit-direct 3本（branch-ret/phi-min/branch-multi）緑維持。

  - 次のステップ（builder 分割 続き）
    - [x] 1) Stmts 本体移設: `builder/stmts.rs` に移動し、`builder.rs` から削除。
    - [x] 2) Ops 抽出: `builder/ops.rs` に移動。
    - [x] 3) Utils 抽出: `builder/utils.rs` に移動。
    - [x] 4) 残存 `*_legacy` の削除と最終ビルド＋jit-direct 3本スモークで回帰確認。
    - [x] 5) 目標: `src/mir/builder.rs` を < 1,000 行に縮小（現状: 967 行）。
    - Docs: 新モジュール構成のメモを `docs/development/mir/MIR_BUILDER_MODULES.md` に追加（参照）。

- 残タスク（次手）
  - [ ] CraneliftBuilder 本体を `builder/cranelift.rs` に分離（大枠）。
  - [ ] `builder.rs` を薄い “ハブ” 化（trait/enum/API公開 + pub use）。
  - [ ] 分離ごとに jit-direct 3本（debug/release）スモーク再確認。
  - [ ] LowerCore の段階分割（`analysis.rs` / `cfg.rs` / `ops.rs`）検討（別PRでも可）。

- 実行メモ（JIT）
  - Build: `cargo build --release --features cranelift-jit`
  - jit-direct: `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - 診断: `NYASH_JIT_DUMP=1 NYASH_JIT_TRACE_BLOCKS=1 NYASH_JIT_TRACE_RET=1` 併用可。
  - 単一出口方針: jit-direct は関数終端で単一の ret ブロックに合流し、PHI(min) は BlockParam で表現。
  - TRACE 変数一覧（JIT）: `NYASH_JIT_DUMP`, `NYASH_JIT_TRACE_BLOCKS`, `NYASH_JIT_TRACE_BR`, `NYASH_JIT_TRACE_SEL`, `NYASH_JIT_TRACE_RET`

Update (2025-09-01 PM2 / Interpreter parity blockers)

- 目的: Semantics 層での VM/JIT/Interpreter パリティ検証に向け、Interpreter 側の既知不具合を記録・引き継ぎ。

- 再現ファイル（ユーザー提供）
  - examples/semantics_test_branch.nyash
    - 期待: 100
    - 実際(Interpreter): IntegerBox(4)（変数IDが返る挙動）
    - 実際(VM): 100
  - examples/simple_test.nyash
    - 事象: PluginBoxV2 同士の加算でエラー（IntegerBoxダウンキャストのみを試行して失敗）
  - /tmp/test_string_concat.nyash
    - 事象: 文字列連結でエラー（PluginBox と String の連結が不可）

- 現状の暫定対応（実装済み）
  - Await(JIT): I::Await を nyash.future.await_h に降下（同期get）
  - Safepoint(JIT): I::Safepoint を nyash.rt.checkpoint に降下（global_hooks 経由で GC/scheduler 連携）
  - Barrier(JIT): Array.set/Map.set で nyash.gc.barrier_write emit
  - Interpreter: FutureBox を登録（new FutureBox(42) 可）
  - Interpreter: 加算まわりのフォールバック強化
    - 文字列likeの優先連結（toUtf8/Result.Ok含む）
    - 数値文字列→整数にパースできる場合は加算

- 未解決（最優先）
  1) 返り値が変数IDになる（examples/semantics_test_branch.nyash）
     - 現状: 再現せず（CLI実行で Result: 100 を確認）。
     - 対応: Return 直前と関数エピローグに限定トレース追加（`NYASH_INT_RET_TRACE=1`）。再発時に型/値を即観測可。
  2) PluginBox 同士の演算の包括対応
     - 暫定は toString→数値/文字列へ正規化で回避。恒久対応は Semantics/VM と同じ規約（handle-first + 文字列like/数値like）に寄せる。
  3) 文字列連結の広範囲対応
     - toString()/toUtf8/Result.Ok の内包を最優先で文字列正規化（現状の強化で多くは通る。追加ケースが出れば順次取り込み）。

- 次アクション（Interpreter fix plan）
  - [ ] semantics_test_branch 再現→Return 値の実体化ルート修正
  - [ ] simple_test の演算パスを toString 正規化で網羅（必要なら算術 toNumber も）
  - [ ] test_string_concat の失敗パターン収集→ try_box_to_string の対象拡張
  - [ ] SemanticsVM/ClifAdapter パリティ小スモーク追加（分岐/配列/extern/await）


Update (2025-09-02 late / jit-direct TLS単一FBリファクタ step-2 部分反映)

- Runner 小リファクタ（計画どおりの分割）
  - `src/runner.rs` → `src/runner/mod.rs` に移動。
  - `NyashRunner::run()` の `run_task` 重複分岐を削除（無害な重複除去）。

- jit-direct（CraneliftBuilder）: 単一FunctionBuilder（TLS）化の前進
  - ローカル（store/load）をTLS単一FB経由に統一。
  - `switch_to_block`: 未終端のまま別ブロックへ切替時に `jump` 注入→`cur_needs_term=false` に更新。
  - `br_if_with_args`: TLS単一FBで発行し、分岐を明示的に終端扱い（`cur_needs_term=false`）。
  - エントリを `seal_block(entry)` 済みに変更（入口でのPHI完了を前提）。
  - ブロックパラメータ（PHI）: その場のappendはやめて「必要数を記録」→`switch_to_block` 前に不足分をappendする方式に変更（命令追加後のparam禁止アサート回避）。
  - `end_function`: 未sealedブロックを最終sealするフェーズを追加（内部追跡セットで二重seal回避）。

- 現状の挙動
  - `cargo build --features cranelift-jit` は通過（警告あり）。
  - `NYASH_JIT_THRESHOLD=1 ./target/debug/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
    - 以前の「未終端での切替」アサートは抑止済み。
    - なお finalize 時に「未sealed blockN」が残るケースを観測（例: block4）。seal の最終整合に取りこぼしがある。

- 追加で必要な作業（次の小ステップ）
  1) LowerCore 側の `seal_block` 呼び出し順序の確認・補強（分岐/合流直後に seal されるよう統一）。
  2) 生成ブロックが `self.blocks` に全て含まれることの確認（ret_block を含む）。不足時は `begin_function` で `pending_blocks` を尊重して作成。
  3) 最小スモークの緑化: `mir-branch-ret` → 1, 続いて `mir-phi-min` → 10, `mir-branch-multi` → 1。
  4) 上記が通ったら、終端注入・最終sealの実装を整理（不要な箇所の削減、ログスイッチ固定）。

- 実行/検証メモ（再掲）
  - ビルド: `cargo build --features cranelift-jit`
  - 実行: `NYASH_JIT_THRESHOLD=1 ./target/debug/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - 追跡: `NYASH_JIT_TRACE_BLOCKS=1`（ブロック切替）/`NYASH_JIT_TRACE_RET=1`（返り値）


開発メモ / 注意点
- 分岐の取り違えは「ブロックまたぎの共有スタック」が原因になりがち。根治策として BlockParam 経由の合流・単一出口 return を徹底。
- デバッグログは “必要時のみ ON” の方針で仕込む。収束後に不要箇所は落とす（本番は静かに）。

チェックリスト（収束条件）
- [ ] jit-direct: `mir-branch-ret` → 1（tag=201/200 とも 1）
- [ ] jit-direct: `mir-phi-min` → 10（合流経路で BlockParam 値が正しい）
- [ ] jit-direct: `mir-branch-multi` → 1（ネスト分岐でも合流が正しい）
- [ ] 統合JIT（--backend cranelift）と一致
- [ ] ログ・一時コードの整理（必要なものだけ残す）
- 既知の注意（jit-direct 経路のみ）
  - 条件分岐系（Branch/Jump）で戻り値が 0 になる事象を確認。`--backend cranelift`（統合経路）では期待値どおり（例: mir-branch-ret → 1）。
  - 影響範囲: jit-direct 実験フラグのみ。LowerCore/CraneliftBuilder の IR 自体は生成されており、統合経路では正しく実行される。

---

最終確認（apps フォルダ tri‑backend 実行計画 / 実行ログ用）

対象: C:\git\nyash-project\nyash\apps 下の各アプリ（インタープリター／VM／JIT(exe)）

前提
- ビルド: `cargo build --release --features cranelift-jit`
- 実行パス（Linux/Mac）: `./target/release/nyash`
- 実行パス（Windows PowerShell）: `target\release\nyash.exe`
- 3モードの呼び分け:
  - Script(インタープリター): `nyash apps/APP/main.nyash`
  - VM: `nyash --backend vm apps/APP/main.nyash`
  - JIT(exe): `nyash --backend vm --jit-exec --jit-hostcall apps/APP/main.nyash`

補足/既知
- 一部アプリは CLI 入力/標準入力/環境定数に依存（例: ny-echo の標準入力、NYASH_VERSION の参照）。必要に応じて簡易入力をパイプで与えるか、定数をスクリプト先頭に仮定義して実行確認する。
- PluginOnly 強制は apps では無効化する（toString 経路が PluginInvoke 固定になると出力整形に影響）。

進め方（手順テンプレート）
1) 共通ビルド
   - [ ] `cargo build --release --features cranelift-jit`
2) アプリごとに 3 モード実行（下記テンプレートをコピーして使用）

テンプレート（各アプリ用）
- アプリ名: <app-name>
  - Script: `nyash apps/<app-name>/main.nyash`
    - [ ] 実行OK / 出力: <貼付>
  - VM: `nyash --backend vm apps/<app-name>/main.nyash`
    - [ ] 実行OK / 出力: <貼付>
  - JIT(exe): `nyash --backend vm --jit-exec --jit-hostcall apps/<app-name>/main.nyash`
    - [ ] 実行OK / 出力: <貼付>
  - 備考: 例）標準入力が必要 → `echo "Hello" | ...`、定数 `NYASH_VERSION` を仮定義 等

対象アプリ（初期リスト）
- ny-echo
  - 入力例（Script）: `echo "Hello" | nyash apps/ny-echo/main.nyash`
  - 入力例（VM）:     `echo "Hello" | nyash --backend vm apps/ny-echo/main.nyash`
  - 入力例（JIT）:    `echo "Hello" | nyash --backend vm --jit-exec --jit-hostcall apps/ny-echo/main.nyash`
  - 既知: `NYASH_VERSION` を参照するため、未定義時はエラー。必要ならスクリプト先頭で仮定義（例: `version = "dev"`）して確認。

- ny-array-bench
  - Script/VM/JIT で 3 モード実行し、処理完了を確認（所要時間・出力サマリを記録）。

- ny-mem-bench
  - Script/VM/JIT で 3 モード実行し、処理完了を確認（所要時間・出力サマリを記録）。

クロスチェック（簡易スクリプト）
- 補助: `tools/apps_tri_backend_smoke.sh apps/ny-echo/main.nyash apps/ny-array-bench/main.nyash apps/ny-mem-bench/main.nyash`
  - 3 モードの Result ライン要約を出力（インタラクティブ入出力が必要なものは手動実行を推奨）。

ゴール/合格基準
- 各アプリで Script/VM/JIT(exe) の 3 モードがクラッシュ無しで完走し、期待する出力/挙動が観測できること。
- 不一致/未定義エラーが出た場合は「備考」に記録し、必要に応じて最小限の仮定義（標準入力や定数）での再実行結果も併記する。
  - 次回対応: brif 直後のブロック制御/シール順の見直し（entry/sealing）、条件値スタック消費タイミングの再点検。


Update (2025-09-01 night / JIT-direct branch/PHI fix)

- Summary
  - Fixed jit-direct returning 0 for branches by unifying returns to a single-exit ret block and improving MIR return type inference (including PHI-based inference).
  - Stabilized PHI joins by materializing PHI results into locals and preferring local loads for Return.
  - Corrected stack order for `br_if_with_args` (pop else args → then args → cond), matching LowerCore push order; added minimal fallback when predecessor mapping is unavailable.
  - Added debug symbols and logs (gated by env) for ret path and CFG wiring.

- Code touched
  - MIR: `src/mir/builder.rs` (return type inference incl. PHI inputs)
  - Lowering core: `src/jit/lower/core.rs` (PHI materialize to local; arg wiring fallback; small traces)
  - Builder (Cranelift): `src/jit/lower/builder.rs` (single-exit return; br_if/jump args order; debug symbol registrations)

- New smokes
  - `apps/tests/mir-phi-two` … merge two locals (x,y) then add
  - `apps/tests/mir-nested-branch` … nested branches returning constants

- Status (jit-direct)
  - mir-branch-ret → 1
  - mir-phi-min → 10
  - mir-phi-two → OK（合流＋加算）
  - mir-nested-branch → 1
  - LLVM AOT snips (prebuilt binaries) still OK for VInvoke samples.

- How to run
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-phi-min/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-phi-two/main.nyash`
  - `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-nested-branch/main.nyash`
  - Optional logs: `NYASH_JIT_DUMP=1` and/or `NYASH_JIT_TRACE_RET=1`

- Next (small boxes; avoid saturation)
  1) Fast-path(select) 1-hop extension only:
     - If then/else each jump to a block that immediately returns a Const Integer, lower to `select(cond, K_then, K_else) → return`.
     - Keep scope narrow to avoid regressions.
  2) Multi-PHI (limited):
     - Add one smoke with two PHI slots; confirm arg counts/ordering under `phi_min`.
  3) Logging remains env-gated; no default noise. No broad refactors until the above are green.


Update (2025-09-02 / jit-direct FB lifecycle refactor)

いま動くもの
- Interpreter/VM/MIR の基本スモーク: OK
- await の Result.Ok/Err 統一: Interpreter/VM/JIT で整合
- Cranelift 実行（`--backend cranelift`）: OK（例: `mir-branch-ret` → 1）

いま詰まっている点（要修正）

Update (2025-09-03 / Phase 11.8 MIR cleanup 準備・判断固め)

- 方針（箱言語原則 × 軽快最適化）
  - ArrayGet/Set と RefGet/Set を BoxCall に集約（Core‑13 化）。
  - 算術/比較（BinOp/Compare）は現状維持（MIR に残す）。定数畳み込みや分岐簡約の主戦場として維持し、型不明ケースは Lower/セマンティクス側でBoxCall/Hostcallにフォールバック。
  - EffectMask 正確化（READ/WRITE/MAY_GC/IO）と WriteBarrier の確実化。
  - 最適化は VM の execute_boxcall / JIT の lower_boxcall に集約（脱仮想化・境界消去・Barrier）。

- 準備タスク（Phase 11.8 Kickoff）
  1) Docs: 仕様と着手順を `docs/development/roadmap/phases/phase-11.8_mir_cleanup/PLAN.md` に確定（このコミットで追加）。
  2) Env 設計: 段階導入トグルを定義（NYASH_MIR_ARRAY_BOXCALL / NYASH_MIR_REF_BOXCALL / NYASH_MIR_CORE13 など）。管理棟（config::env）での一括適用方針。
  3) Optimizer: Array/Field→BoxCall 変換パスのスケルトン追加（デフォルトOFF）。
  4) VM: execute_boxcall に予約IDの fast‑path フック（Array get/set・Field get/set）雛形。
  5) JIT: lower_boxcall の fast‑path 雛形（Bounds/Barrier含む、失敗時 plugin_invoke）。
  6) Smokes/Bench: array/field/arithmetic_loop の最小3種を用意・回帰基準±5%/±10%を導入。
  7) Cleanup sweep: 残存のレガシー/未使用コード・コメントの一括整理（claude code指摘の残骸候補を含む）。

- 参照: docs/development/roadmap/phases/phase-11.8_mir_cleanup/TECHNICAL_SPEC.md / PLAN.md
- jit-direct で Cranelift FunctionBuilder が「block0 not sealed」でパニック
  - begin/end のたびに短命の FunctionBuilder を作って finalize している設計が、最新の Cranelift の前提（全ブロック seal 済みで finalize）と合っていない
  - 単一出口（ret_block）方針は Cranelift 側に途中まで入っているが、ObjectBuilder と二重実装があり、Cranelift 側の finalize 前にブロックを seal しきれていない箇所が残っている

直近の変更（対策の第一歩）
- CraneliftBuilder
  - return は ret_block へ jump（エピローグで最終 return）に変更（単一出口に合わせて安全化）
  - entry block の seal を begin_function で実施
  - end_function 最後に blocks/entry/ret の seal を実施
- ObjectBuilder
  - emit_return は従来通りダイレクト return（ret_block を持たないため）

現状の評価
- 上記を入れても FunctionBuilder finalize のアサーションは残存。
- jit-direct の builder ライフサイクル（複数回 finalize する設計）そのものを見直す必要あり。

次の実装（推奨順）
1) CraneliftBuilder のビルドモデルを単一 FunctionBuilder 方式へ
   - 関数スコープで1つの FunctionBuilder を保持し、lower 中は finalize しない
   - switch/jump/phi/hostcall も同一 FB で emit（現状の都度 new/finalize を撤廃）
   - seal は then/else/target を LowerCore 側からタイミング良く呼ぶ＋end_function で最終チェック
2) jit-direct での AOT emit パス（ObjectBuilder）は現状通りだが、strict 判定を整理
   - `mir-branch-ret` のような最小ケースは unsupported=0 を確実に維持
   - まずはこの1本で .o 生成→リンク→EXE 実行を通す
3) ツールチェイン側（`tools/build_aot.sh`）の strict モードヒントを活かしつつ、上記の最小成功ケースを CI スモークに追加

全側で続けてこのリファクタに着手。まずは FunctionBuilder のライフサイクル一本化から進め、`mir-branch-ret` の AOT（EXE）生成・実行まで通し切る。


# （以下、旧タスク: Phase 10.8 記録）

Contributor note: 開発手順・レイアウト・PR要件はリポジトリルートの `AGENTS.md`（Repository Guidelines）参照。ビルド/テストやCIスモークの環境変数も簡潔にまとまっています。

## Handoff (Phase 11 next): 実行確認プラン（tests/ は対象外）

目的: apps 配下の「簡単なアプリ」から順に、VM → AOT(LLVM) の順で確実に動作させる。tests/ フォルダは除外。

前提/共通
- LLVM: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix)` を付与してビルド
- AOTリンク: `tools/build_llvm.sh <app.nyash> -o app` を使用（`target/aot_objects/*.o` 固定）
- Verbose: トラブル時は `NYASH_CLI_VERBOSE=1`
- プラグインテスター: `tools/plugin-tester` を利用可（`cargo run --release -- check --config ../../nyash.toml --library <lib>`）

推奨テスト順（簡単→段階的）
1) apps/ny-array-llvm-ret/main.nyash（Array push/get 戻り値）
   - 期待: `Result: 3`
   - VM: `./target/release/nyash --backend vm apps/ny-array-llvm-ret/main.nyash`
   - AOT: `tools/build_llvm.sh apps/ny-array-llvm-ret/main.nyash -o app && ./app`

2) apps/ny-vinvoke-llvm-ret-size/main.nyash（by-id size）
   - 期待: `Result: 1`
   - VM/AOT 同上

3) apps/ny-vinvoke-llvm-ret/main.nyash（by-id get 可変長経路）
   - 期待: `Result: 42`
   - VM/AOT 同上

4) apps/ny-echo-lite/main.nyash（readLine → print 最小エコー）
   - 期待: 出力に `Result:` 行が含まれる（llvm_smoke の基準）
   - 実行例: `echo hello | ./target/release/nyash --backend vm apps/ny-echo-lite/main.nyash`
   - AOT: `tools/build_llvm.sh apps/ny-echo-lite/main.nyash -o app_echo && echo hello | ./app_echo`

5) apps/ny-llvm-smoke/main.nyash（Array get/set/print）
   - 期待: `Result: 3`
   - 備考: 文字列連結は NyRT concat シムへフォールバック済み。emit 検知が不安定な場合は再試行。問題が残る場合はこの項を後回し可。

6) apps/ny-echo/main.nyash（オプション付きエコー）
   - 期待: VM/JIT/AOT の出力一致（upper/lower/そのまま）

7) apps/ny-map-llvm-smoke/main.nyash（Map by-id 経路）
   - 期待: 行に `Map: v=42` と `size=1`
   - 備考: 連結シム適用済み。必要なら `NYASH_LLVM_ALLOW_BY_NAME=1` で一時回避。

トラブルシュート要点
- AOT emit: `NYASH_LLVM_OBJ_OUT=$PWD/target/aot_objects/<name>.o ./target/release/nyash --backend llvm ...`
- リンク: `NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT=... tools/build_llvm.sh ...`
- プラグイン: `nyash.toml` のパス解決（拡張子はOSで異なる）。tester は拡張子補完に対応。

Update (2025-08-31 AM / Phase 11.1 quick pass)

Update (2025-08-31 PM / Phase 11.2 partial)

- 方式A（LLVM専用 NyRT 静的ライブラリ）で前進。by-id を本線、by-name はデバッグ用ラッパ方針。
- Lowering 更新
  - NewBox（引数あり 1～2個）→ `nyash.box.birth_i64(type_id, argc, a1, a2)`（int/handle ptr の最小対応）。0引数は `birth_h`
  - BoxCall（by-id, method_idあり）→ `nyash_plugin_invoke3_i64(type_id, method_id, argc, a0, a1, a2)` 接続（a0=receiver handle）
    - 戻り: dstが整数/真偽ならi64のまま、Box/String/Array等は i64(handle)→i8*(ptr)
    - ArrayBox.get/set は既存の `nyash_array_get_h/set_h` 安全パスを存続
  - 生成関数名: `ny_main` に変更（NyRTの起動ルーチンから呼び出し）
- NyRT(libnyrt.a) 追加シンボル
  - `nyash_string_new(i8*, i32)->i8*`（Const String用）
  - `nyash_array_get_h(i64,i64)->i64`, `nyash_array_set_h(i64,i64,i64)->i64`
  - 既存の `nyash.box.birth_h/i64`, `nyash.rt.checkpoint`, `nyash.gc.barrier_write` などは維持
- ツール
  - `tools/build_llvm.sh` 追加（.o → libnyrt.a リンク → EXE）
  - `tools/llvm_smoke.sh`（.o生成のスモーク）
- スモーク
  - `examples/llvm11_core_smoke.nyash` で EXE 実行し `Result: 3` を確認

Update (2025-08-31 PM2 / Phase 11.2 lightweight LLVM)

- 方針（拡張性優先 / コア最小化: Tier‑0）
  - ExternCall: 環境/I/Oのみ（env.console/debug/runtime など）。print は ExternCall(env.console.log) 本線。
  - BoxCall: データ構造/Box 操作（Array/Map/String/Instance 等）。AOT/LLVM は最小の安全シムのみ直結。
  - コアに残す安全シム（NyRT）: Array(get/set/push/length), Instance(getField/setField)。Map はコアに足さない（後述）。
  - Map/JSON/Math 等は当面コア外（必要時はプラグイン by‑id + 汎用シムでAOTを通す）。

- 実装・反映
  - MIR パス: `passes/method_id_inject` 追加（NewBox/Copy 由来の型から BoxCall に method_id 注入。PluginInvoke は可能なら BoxCall(by‑id)へ書換）。
  - LLVM Lowering:
    - ExternCall: `env.console.log` → `nyash.console.log`（NyRT）, `env.debug.trace` → `nyash.debug.trace`。
    - ExternCall: `env.console.readLine` 追加 → `nyash.console.readline`（stdin 1行, CR/LF 除去, C文字列返却）。
    - ArrayBox: `get/set/push/length` を NyRT 安全シム（`nyash_array_get_h/set_h/push_h/length_h`）に直結。
    - Instance: `getField/setField` を NyRT 安全シム（`nyash.instance.get_field_h/set_field_h`）に直結（既存）。
    - プラグイン by‑id: f64 戻りの選択（`nyash_plugin_invoke3_f64`）/ i64 戻り（`..._i64`）。先頭2引数はタグ付け（int/float/handle）対応（`..._tagged_i64`）。
    - by‑name 薄フォールバック（デバッグ用）: `NYASH_LLVM_ALLOW_BY_NAME=1` 下で `nyash.plugin.invoke_by_name_i64` を使用。
  - NyRT(libnyrt.a): 上記 API を追加（console.log/debug.trace/readline, array push/length, instance get/set_field, by‑name, tagged_i64）。
  - オブジェクト出力: `NYASH_LLVM_OBJ_OUT=<path>` で .o を明示出力（`runner/modes/llvm.rs` 経由）。
  - ツール更新: `tools/build_llvm.sh` / `tools/llvm_smoke.sh` が `NYASH_LLVM_OBJ_OUT` を使用して .o を取得後、NyRT とリンク。
  - サンプル: `apps/ny-llvm-smoke/main.nyash`（Array get/set/print）、`apps/ny-echo-lite/main.nyash`（readLine→print）。

- しないこと / 後回し
  - Map のコア安全シム追加（`nyash_map_*_h`）は見送り。必要ならプラグイン by‑id + 汎用シムでAOT実行。
  - ConsoleBox の高度機能は ExternCall 側で段階導入（出力は既存 log、入力は readline のみ）。
  - 汎用可変長引数（>2）は後段（タグ付けの拡張で対応予定）。

- 次にやること（短期）
  - ny-echo を縮小AOT対応（console.readLine + print のみで OK 版）。
  - Map（プラグイン最小版）で string-key の get/set/size を by‑id 汎用シム経由でAOT実行（コアは増やさない）。
  - CI/スモーク: `.o→EXE→実行` を `apps/ny-llvm-smoke` / `apps/ny-echo-lite` で追加。
  - ガードレール: コア安全シムを Tier‑0 以外に増やさない簡易チェック（grep ベース）導入検討。

- How to Build / Run（AOT/LLVM）
  - ビルド: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
  - .o→EXE: `tools/build_llvm.sh <file.nyash> -o app_llvm` → `./app_llvm`
  - .o のみ: `NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o ./target/release/nyash --backend llvm <file.nyash>`
  - 推奨 @env（main.nyash 冒頭コメント）:
    - `// @env NYASH_CLI_VERBOSE=1`（詳細ログ）
    - デバッグ時のみ `// @env NYASH_LLVM_ALLOW_BY_NAME=1`

Update (2025-08-31 PM3 / LLVM VInvoke triage)

- 直近のテスト結果（要約）
  - VM backend（正常）
    - MapBox.size(): Result: 1 ✅
    - MapBox.get(1): Result: 42 ✅
    - 可変長（VInvoke: get(1,9,8,7,6)）: 期待どおりにTLV I64(tag=3)でエンコードされ、出力確認済 ✅
  - LLVM backend（要修正）
    - MapBox birth は成功（instance_id=1）
    - set()/get() 呼び出し経路で戻り値が Result: 0（期待は 42）❌
    - LLVM 実行時に VM 相当の PluginLoaderV2 Invoke 詳細ログが出ず、by-id/可変長の値流しに不整合がある可能性

- 実装・修正状況
  - ランタイム（NyRT）
    - by-id 固定長/可変長の argc を「レシーバ除外の実引数個数」に統一済み
    - invoke 時の type_id を「レシーバ実体（PluginBoxV2）から取得した実 type_id」に変更（呼び出し先揺れを排除）
  - LLVM Lowering
    - <=4 引数: nyash_plugin_invoke3_tagged_i64（f64→i64ビット化＋タグ付与）
    - >=5 引数: nyash.plugin.invoke_tagged_v_i64（vals/tags の vector 経路）
  - スモーク
    - apps/ny-vinvoke-llvm-ret: 戻り値で 42 を検証する LLVM 用スモークを追加（print/concatに依存しない）
    - tools/llvm_smoke.sh に VInvoke（戻り値）スモークをオプション追加（NYASH_LLVM_VINVOKE_RET_SMOKE=1）

- いま見えている課題（LLVM）
  - by-id vector 経路（vals/tags/argc）のどこかで齟齬 → get が 0 を返す（キーが TLV 化されていない／タグずれなど）
  - 固定長（<=4）経路は未切り分け → まず size()/get(1) を LLVM 戻り値で確定（size→1, get(1)→42）
  - .o 出力の一部環境不安定は Runner/Lowering にフォールバックを追加済（write_to_memory_buffer）

- 次アクション
  1) LLVM 戻り値テストを段階確認
     - MapBox.size() → Result: 1
     - MapBox.get(1) → Result: 42
     - これで固定長 by-id の健全性を確定後、可変長（>=5）vector 経路へ絞り込み
  2) NyRT デバッグ（NYASH_CLI_VERBOSE=1 時のみ）を最小追加
     - nyash.plugin.invoke_tagged_v_i64: argc/method_id、vals[0..2], tags[0..2] を stderr に出力
     - 実際のTLV化の前に観測し、ズレ箇所を確定
  3) LLVM Lowering（vector 経路）の配列構築を点検
     - alloca([N x i64]) → inbounds GEP → store → i64* へ pointer_cast → 呼び出し
     - GEP index（[0,i]）/型一致/メモリ幅を再確認
  4) 必要なら一時的に「<=4 でも vector 経路を選択」する実験分岐を作り、経路差異を切り分け
  5) tools/llvm_smoke.sh: target/aot_objects 固定・事前 emit → link のフローをデフォルト強化（CI安定化）

- ゴール（本フェーズ収束条件）
  - LLVM backend で MapBox: size()/get(1)/get(1,9,8,7,6) が戻り値ベースで一致
  - VInvoke（可変長）経路が by-id で安定
  - print/concat のLoweringは後続（必要最小）に回す

- タスク実行（MVP）
  - `nyash.toml` に `[env]` / `[tasks]` を記述し、CLIから `--run-task <name>` で実行可能。
  - 例:
    - `[tasks] build_llvm = "LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm"`
    - 実行: `./target/release/nyash --run-task build_llvm`
  - 備考: `{root}` 変数展開対応。OS別/依存/並列は未対応（将来拡張）。


残作業（合意順）

1) method_id 埋め込みと by-id 本線化
   - ロード時に 名前→id を確定・キャッシュ（PluginLoaderV2）し、MIR へ `method_id` 注入（実行時は常に by-id）
2) BoxCall 汎用拡張
   - 引数3個以上/戻り型の拡張（i64/handle/f64 等）。Field 系（getField/setField）を BoxCall として安全パス接続
3) by-name ラッパ（デバッグ/テスト用）
   - Lowering フォールバックとして薄く導入（env/flag 下でのみ使用）、本番は by-id 固定
4) ExternCall 網羅
   - `env.console/debug/runtime/future` 等を puts 暫定から RT関数に置換、署名整備
5) スモーク/CI 拡張
   - by-id の代表例（CounterBox等）、console/array/field/extern を .o→EXE→起動まで

- LLVM Lowering: Phi/Load/Store の最小実装を追加（inkwell 0.5 / LLVM 18）
  - Phi: 事前に各BB先頭でPhiノード生成→Branch/Jump時にincomingを配線
  - Load/Store: entryでのalloca管理（型は注釈/値から推定）。i1/i64の簡易変換、ポインタはpointer_cast対応
  - 型なしポインタ（opaque）対応のため、`alloca`の要素型を別マップで追跡
  - ビルド検証: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --features llvm` 成功
  - 既存のConst/Unary/BinOp/Compare/Branch/Jump/Return と併せ、Phase 11.1の目標範囲は到達

---

一時メモ（MIRインタプリタ 80/20 方針）

- ConsoleBox.readLine ローカルフォールバック（標準入力1行読み）を一時実装（`--backend mir` の echo-lite 用）。
  - 後で必ず削除し、プラグイン側メソッド/Extern 経由に置換すること。
  - 追跡タスク: 「ConsoleBox メソッド群の正式実装（PluginInvoke/ExternCall）とローカルフォールバックの撤去」。
  - 影響箇所: `src/backend/mir_interpreter.rs`（BoxCall: ConsoleBox.readLine 分岐）、`apps/tests/ny-echo-lite` スモーク。

次アクション（この項に紐づく）
- ClifSem 最小Lowering（Const/Return/Add）を実装し、JITスケルトンの0終了スモークを追加。
- ConsoleBox フォールバックは温存のまま（echo-liteのみのため）。Console 正式化のタイミングで除去。

---

# Handoff (Phase 11.7) — MIR Interpreter + Cranelift Minimal JIT

目的: LLVMが重くなったため仕切り直し。新しいMIR解釈層と軽量Cranelift JITの最小機能を整備し、段階拡張しやすい骨格を確立。

実装済み（要点）
- 共通ABI/ユーティリティ:
  - `src/backend/abi_util.rs`（to_bool/eq/tag/handle）
- MIRインタプリタ（--backend mir）:
  - 対応: Const/Unary/BinOp(＋String結合)/Compare/Load/Store/Copy/Branch/Jump/Return/Print/Debug/Barrier/Safepoint(no-op)
  - NewBox/PluginInvoke/BoxCall/ExternCallの最小対応（PluginBoxV2はプラグインホスト経由）
  - ConsoleBox.readLine: 一時フォールバックで標準入力1行読み（CURRENT_TASKに削除タスク記載済）
- Cranelift最小JIT（--backend cranelift）:
  - 実JIT（jit.rs）: Const(i64/f64/bool->0/void->0)/Add/Sub/Mul/Div/Mod、Compare(Eq/Ne/Lt/Le/Gt/Ge)、Load/Store（StackSlot）、Copy、Return/Jump/Branch
  - 箱化: `src/backend/cranelift/context.rs` に ClifContext/BlockMap/ValueEnv を用意（JIT構築をカプセル化）
  - LowerCore→ClifBuilder（IRBuilder実体）: 録画→実IR生成（Const/Add/Return）を暫定実装
    - 起動切替: `NYASH_JIT_LOWERCORE=1` で LowerCore→ClifBuilder 実IR経路を実行
- スモーク:
  - `apps/tests/mir-const-add/main.nyash`（0終了）
  - `apps/tests/mir-branch-ret/main.nyash`（条件分岐で1）
  - どちらも --backend cranelift / --backend mir で確認済

使い方（コマンド）
- Cranelift有効ビルド: `cargo build --features cranelift-jit`
- MIRインタプリタ: `./target/debug/nyash --backend mir apps/tests/mir-const-add/main.nyash`
- Cranelift最小JIT: `./target/debug/nyash --backend cranelift apps/tests/mir-branch-ret/main.nyash`
- LowerCore→ClifBuilder 実IR: `NYASH_JIT_LOWERCORE=1 ./target/debug/nyash --backend cranelift apps/tests/mir-const-add/main.nyash`

次のタスク（推奨順）
1) ClifBuilder 実IR生成の拡張（LowerCore連携）
   - Compare/Branch/Jump の実IR
   - 最小Phi（Block Params）
   - StackSlotベースのLoad/Store（ValueEnvから完全移行）
2) ExternCall（env.console.log）最小対応（JIT経路でもprintln相当へ）
3) スモーク追加: Load/Store、Phi最小ケース、Compare/Branch複合ケース
4) Console/Extern 正式化完了後に ConsoleBox.readLine フォールバック削除（本ファイルにタスク済）
5) 警告/CFG整理: 使っていないfeature cfgやunusedを段階的に整理

既知の注意/制限（80/20の割り切り）
- BoolはI64の0/1として扱っており、B1専用ABIは未導入（将来拡張）。

---

Update (2025-09-02 night / jit-direct TLS単一FBリファクタ 進捗・引き継ぎ)

- 目的: jit-direct の Cranelift FunctionBuilder ライフサイクルを「関数ごとに1つ」に統一し、finalizeは end_function の一度のみとする（Craneliftの前提に整合）。

- 実装済み（最小スコープ）
  - TLSに Context/FBC/FunctionBuilder を保持（begin_functionで生成→end_functionでfinalize）。
  - per-op finalize の撤去。主要経路（const/binop/compare/select/branch/jump/return/hostcall 等）を TLS 単一FB に切替中。
  - 単一出口（ret_block + i64 block param）維持。emit_return は ret_block へ jump、end_function で epilogue return を生成。
  - prepare_blocks は begin_function 前はTLSに触れず pending_blocks に貯め、begin_function で create_block。
  - host/import 呼び出しは tls_call_import_ret/tls_call_import_with_iconsts ヘルパへ分離（module.declare_func_in_func + call を安全化）。
  - 未終端ブロック切替の安全弁: IRBuilder::switch_to_block に「未終端なら jump 注入」（cur_needs_term）を導入。

- 現状ステータス
  - cargo build --features cranelift-jit: OK
  - jit-direct 実行: まだ1箇所「you have to fill your block before switching」（未終端での block 切替）アサートが残存。
    - 再現: `NYASH_JIT_THRESHOLD=1 ./target/debug/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
    - 多くの switch_to_block は closure から排除済みだが、特定条件下で未終端のまま切替が残っている模様。

- 次の小ステップ（箱を下に積む順）
  1) IRBuilder::switch_to_block の重複抑止（同一 index への再切替は no-op）。
  2) cur_needs_term の更新確認（emit_return/br_if/jump 後は必ず false）。主要箇所は反映済みだが再点検。
  3) emit_* 内の残存 switch_to_block を整理（挿入点は LowerCore 側の switch_to_block に一本化）。
  4) トレースで最終合流（ret_block）直前の切替を観測：
     - 環境: `NYASH_JIT_TRACE_BLOCKS=1 NYASH_JIT_TRACE_BR=1`
  5) スモーク（jit-direct）を順に通す:
     - `mir-branch-ret` → 1
     - `mir-phi-min` → 10
     - `mir-branch-multi` → 1
  6) hostcall_typed / plugin_by_name の TLS 呼び出し統一（未対応部分があれば最小限で補完）。

- 実行/検証メモ
  - ビルド: `cargo build --features cranelift-jit`
  - 実行: `NYASH_JIT_THRESHOLD=1 ./target/debug/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
  - 追跡ログ: `NYASH_JIT_TRACE_BR=1`（brif出力）、`NYASH_JIT_TRACE_BLOCKS=1`（block切替通知）

- 影響範囲
  - jit-direct（CraneliftBuilder）限定。ObjectBuilder（AOT .o生成）は従来通り。
  - docs/development/roadmap/phases/phase-11.7_jit_complete 配下のフェーズ文書・計画は維持（削除・変更なし）。

備考
- まずは TLS 方式で単一FBモデルを安定化（動かすことを最優先）。その後、余力があれば IRBuilder/LowerCore に FB を明示渡しするクリーン版へ段階移行を検討。
- String/Null/Void のConstは暫定的に0へ丸め（必要箇所から段階的に正規化）。
- `jit-b1-abi` 等のunexpected cfg警告は今後整理対象。

関連ファイル
- 追加: `src/backend/abi_util.rs`, `src/backend/mir_interpreter.rs`, `src/runner/modes/mir_interpreter.rs`
- 追加: `apps/tests/mir-const-add/main.nyash`, `apps/tests/mir-branch-ret/main.nyash`
- Cranelift: `src/backend/cranelift/jit.rs`, `src/backend/cranelift/context.rs`, `src/backend/cranelift/builder.rs`
- Semantics: `src/jit/semantics/{mod.rs,clif.rs}`（スケルトン）

メモ/トグル
- LowerCore→ClifBuilder 実IR: `NYASH_JIT_LOWERCORE=1`
- カバレッジログ: `NYASH_JIT_DUMP=1`



Handoff Snapshot (2025-08-31 / Phase 11 kick-off)

- Core-15 凍結（第三案 / Box-SSA）
  - セット: { Const, UnaryOp, BinOp, Compare, TypeOp, Load, Store, Jump, Branch, Return, Phi, Call, NewBox, BoxCall, ExternCall }
  - Optimizer: ArrayGet/ArraySet/RefGet/RefSet/PluginInvoke → BoxCall に正規化（get/set/getField/setField）
  - Verifier: 上記レガシー命令を UnsupportedLegacyInstruction としてエラー化（環境で一時解除可: NYASH_VERIFY_ALLOW_LEGACY=1）
- VM: BoxCall("getField"/"setField") を InstanceBox に配線（fieldsへ委譲）。Arrayの get/set は既存BoxCall経路で動作
- 命令数固定テスト: Core‑15（第三案）へ切替済（tests/mir_instruction_set_sync.rs）
- LLVM 導入（Phase 11 開始）
  - 依存: LLVM 18 + inkwell 0.5.0（features=["llvm18-0"]）。feature `llvm` で有効化
  - ビルド要件: LLVM_SYS_180_PREFIX（例: /usr/lib/llvm-18）, 追加依存: polly, zstd（libzstd-dev 等）
  - 現状のLowering（11.1の最小スケルトン → 11.2 反映）:
    - 対応: Const(Integer/Float/Bool/String/Null), Unary(Neg/Not/BitNot), BinOp（整数/浮動の主要演算）, Compare, Branch/Jump, Return
    - 追加: Phi/Load/Store（最小実装）
    - 追加: NewBox（引数なし→nyash.box.birth_hへ; nyash.tomlの[box_types]からtype_id解決）
    - 追加: BoxCall（ArrayBox.get/set→nyash_array_get_h/set_h 経由の安全パス）
    - 追加: ExternCall（env.console.log/env.debug.trace→libc putsで暫定出力）
    - 未対応（次タスク）: NewBox（引数あり）, 一般BoxCall（by-name/slot 汎用化）, その他ExternCall
  - エントリ: Main.main のみ対象に .o 出力（backend::llvm::compile_to_object）
- ドキュメント更新（phase‑11）
  - README.md: 進行中に更新 / 4週スプリント計画（11.1→11.4）
  - MIR_TO_LLVM_CONVERSION_PLAN.md: PluginInvoke→BoxCall統一、配列はBoxCallとして安全パス→型特化の二段階Lowering
  - MIR_ANNOTATION_SYSTEM.md: setField/getField（BoxCall）前提に更新
  - INSTRUCTION_SET.md: PluginInvokeはDeprecated（BoxCallに統一）

How to Build/Run (recap)

- 通常/JIT: `cargo build --release --features cranelift-jit`
- LLVM（AOTスケルトン）: 
  - 事前: LLVM 18 / inkwell 0.5.0, polly, zstd を導入
  - 例: `LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm`
- スモーク: `tools/mir15_smoke.sh release`

Next Steps (Phase 11)

1) 11.1 仕上げ（本タスク）
   - Phi のLowering（BB事前作成→incoming追加）
   - Load/Store（alloca/ローカル表現の最小規約、整数/浮動/ポインタ）
2) 11.2 安全パス（Box/Extern）
   - [実装] NewBox(引数なし)→ `nyash.box.birth_h(type_id:i64)->i64` を呼び、i8*にinttoptr（type_idはnyash.tomlから解決）
   - [実装] Arrayの BoxCall("get"/"set") → `nyash_array_get_h/set_h`（ハンドルi64渡し）
   - [実装] ExternCall: `env.console.log`/`env.debug.trace` は暫定で `puts` に接続（AOTデバッグ用）
   - [残] BoxCall 汎用（by-name/slot）, Field系（getField/setField）, ExternCallの網羅
3) 11.3 最適化導線
   - 注釈（inline/purity/gc/alias）→ LLVM属性/メタデータ
   - 型特化: Array/Field の inline GEP + write barrier
4) 11.4 高度化
   - 脱箱化、TBAA、PGO/ThinLTO

メモ

- Verifier の緩和スイッチ: `NYASH_VERIFY_ALLOW_LEGACY=1`（移行用）。通常はOFFで運用。
- Optimizer のRewrite はLLVM前提のBoxCall統一規約と整合済み。

最優先: MIR命令セットをCore-15に統一し、VM/JIT/AOTを整えてからLLVM(inkwell)へ移行する。

目的: MIR→VM→JIT→AOT の汎用化・単純化を一気に進める。命令の重複・メタ・実装露出を撤去/統合し、Builderが実際に発行するコア命令を最小化する。

現状観測（2025-08-31）

- 実装定義: 37命令（src/mir/instruction.rs）
- Docs: Canonical 26（移行注記・Core-15ターゲット追記済）
- Builder使用: 24命令（自動集計）
  - 上位頻度: Const(19), TypeOp(13), Jump(6), ExternCall(5), Return(3), Call(3)
  - 中頻度: NewBox(2), WeakRef(2), Barrier(2), BoxCall(1), Branch(1), Phi(1), PluginInvoke(1), Await(1)
- JITサポート: 約20命令（ホストコール分は外部委譲で簡素）

統合方針（Core-15）

- 重複統合
  - TypeCheck, Cast → TypeOp に完全統合（Builder 既に主に TypeOp を発行）
  - WeakNew, WeakLoad → WeakRef に統合
  - BarrierRead, BarrierWrite → Barrier に統合
- Box哲学へ移譲
  - Print → env.console.log (ExternCall)（Builder更新済）
  - Debug → DebugBox.trace()/env.debug.trace（ExternCall/BoxCall）
  - Throw, Catch → ExceptionBox（throw/catch相当のBox APIへ移譲; 移行期はRewrite）
  - Safepoint → RuntimeBox.checkpoint（ExternCall）
- 未使用/メタ
  - FutureSet → 一旦廃止（Builder未使用）
  - Copy, Nop → メタ命令（Optim/降格専用; Coreからは除外）

最終ターゲット: Core-15 命令

- 基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
- メモリ(2): Load, Store
- 制御(4): Branch, Jump, Return, Phi
- Box(3): NewBox, BoxCall, PluginInvoke
- 配列(2): ArrayGet, ArraySet
- 外部(1): ExternCall（暫定; 将来はBox化でも可）

進め方（Core-15確定 → LLVM）

1) Builder発行の一元化（非破壊）
   - 既に Print→ExternCall 置換済み
   - Builderが TypeCheck/Cast/WeakNew/WeakLoad/BarrierRead/BarrierWrite を発行しないよう整理（既存箇所の差し替え）
   - 既存テスト（builder/optimizer）を ExternCall/TypeOp/WeakRef/Barrier に合わせて更新
2) 互換Rewriteパスの追加（MIR最適化フェーズ）
   - 古いMIR（手書き/スナップショット/ツール）が生成した命令をコア命令に機械的変換
     - TypeCheck/Cast → TypeOp
     - WeakNew/WeakLoad → WeakRef
     - BarrierRead/Write → Barrier
     - Print → ExternCall(env.console.log)
     - Debug → ExternCall(env.debug.trace)
     - Throw/Catch → ExternCall(env.exception.*) もしくは BoxCall(ExceptionBox)
     - Safepoint → ExternCall(env.runtime.checkpoint)
3) VM/JITの段階撤去と整理
   - VM/JIT: コア命令に集中（ホストコール/Box経由のI/Oや例外）
4) Docs/CI
   - INSTRUCTION_SET をCore-15へ更新（26→15マッピング表）
   - 命令数固定テストを「15」に切替

タスク分解（本フェーズ）

- [x] Builderからのレガシー発行のデフォルト停止（WeakNew/WeakLoad/BarrierRead/Writeを統一命令に切替、トグルで復活可）
- [x] MIR Rewriteパス追加（一部完了: Print/Type/Weak/Barrier、Debug/Safepointはトグル）
- [ ] Optimizer/Verifier/Printerの非互換の見直し（Verifierでレガシー命令をエラー化）
- [ ] VM: レガシー命令のコードパスに警告ログ（将来削除フラグ）
- [ ] JIT: コア命令に合わせたテーブル整理（未使用ホストコールの棚卸し）
- [ ] Docs更新（命令セット、移行ガイド、Box哲学との整合）
- [ ] 回帰テスト更新（builder_modularizedを含む一式）

追加の即応ステップ（10.8a）

- [x] Rewrite: Print → ExternCall(env.console.log)
- [x] Rewrite: TypeCheck/Cast → TypeOp、WeakNew/WeakLoad → WeakRef、BarrierRead/Write → Barrier
- [x] Rewrite(トグル): Debug → ExternCall(env.debug.trace)（NYASH_REWRITE_DEBUG=1）
- [x] Rewrite(トグル): Safepoint → ExternCall(env.runtime.checkpoint)（NYASH_REWRITE_SAFEPOINT=1）
- [x] Builder: Try/Catch/Throw/Safepoint の直接発行を抑止（トグル導入・モジュール化系にも適用）
- [x] Runtime: extern_call スタブ追加（env.runtime.checkpoint, env.debug.trace）
- [x] Rewrite(トグル・スキャフォールド): FutureNew/FutureSet/Await → ExternCall(env.future.*) 変換（NYASH_REWRITE_FUTURE=1）

引き継ぎ（2025-08-31 深夜）

サマリ

- JIT予約シンボル: `nyash.rt.checkpoint`, `nyash.gc.barrier_write` を確保（AOT/JITの双方で登録）
- Future/Await Rewrite: `NYASH_REWRITE_FUTURE=1` で ExternCall(env.future.*) に段階導入（runtime最小実装あり）
- Builderレガシー停止: Weak/Barrier 系の直接発行を既定で無効化（統一命令に集約、必要時トグル）
- JIT-direct安定化: entry seal/戻り値制御/シム登録/コード寿命を調整し落ち着かせ済み
- 次の着手順: MIR15のVM/JITカバレッジ拡張 → LLVM(inkwell) 移行

- 完了/修正
  - JIT/AOT 予約シンボルの登録完了（`nyash.rt.checkpoint`, `nyash.gc.barrier_write`）
  - JITスタブ実装（no-op＋トレース）とAOT(nyrt)エクスポートを追加
  - Future/Await Rewriteのスキャフォールド（`NYASH_REWRITE_FUTURE=1`）＋ runtime側`env.future.*`最小実装
  - Builderレガシー停止を既定化（WeakNew/WeakLoad/BarrierRead/Write→統一命令）。必要時はトグルで復活
    - `NYASH_BUILDER_LEGACY_WEAK=1`, `NYASH_BUILDER_LEGACY_BARRIER=1`
  - JIT directの安定化（segfault修正）
    - エントリblockのseal遅延（PHI/引数受け用）
    - MIRシグネチャに基づく戻り値有無（Void関数はret無し/void呼び出し）
    - `nyash.console.birth_h` のJIT内シム追加＋JITBuilder登録
    - finalize済みコードの寿命延長（JITModuleをリークして新モジュールに差し替え）

- カバレッジ確認（MIR15→VM/JIT）
  - 追加ドキュメント: docs/reference/mir/MIR15_COVERAGE_CHECKLIST.md
  - スモークスクリプト: tools/mir15_smoke.sh（実行例: `cargo build --release --features cranelift-jit` → `tools/mir15_smoke.sh release`）
  - 現状OK（JIT-direct）: BinOp, Compare(真偽戻り), Load/Store(ローカル), Branch/Jump/PHI最小, ExternCall(console.log)
  - まだoptional（フォールバックで許容）: 配列/Mapのhostcall系（len/isEmpty/get/push 等）

実行メモ

- ビルド: `cargo build --release --features cranelift-jit`
- スモーク: `tools/mir15_smoke.sh release`
- AOT(.o)簡易出力: `NYASH_AOT_OBJECT_OUT=target/aot_objects ./target/release/nyash --jit-direct examples/aot_min_return_42.nyash`
  - 生成: `target/aot_objects/main.o`
  - 備考: `tools/build_aot.sh` は jit-direct で .o を生成するよう更新済（要検証）

LLVM足場（VM側 先行）

- Escape Analysis（VMのみ）: `NYASH_VM_ESCAPE_ANALYSIS=1`
  - 非エスケープなBoxの `Barrier(Read/Write)` を保守的に `Nop` 化
  - 実装: `src/mir/passes/escape.rs`（NewBox起点のローカル追跡＋Return/Call/Store使用でescape検出）
  - 適用: VM実行前にMIRへ適用（`src/runner/modes/vm.rs`）
  - 目的: LLVM(inkwell)への最適化ヒント連携を見据えた足固め（まずVMで効果検証）

- 次の着手（この順で）
  1) MIR15のVM/JITカバレッジをもう一段拡張（配列/Map hostcallのJIT対応 or optionalのまま明確化）
  2) スモークに代表サンプルを追加し、CI/ローカルでワンコマンド確認
  3) LLVMフェーズ（inkwell）へ移行（Const/Return→BinOp/Compare→CF/PHIの順）

次フェーズ提案

- まずMIR15のVM/JITを固める（hostcallはoptional許容 or 段階実装）
- その後、LLVM（inkwell）へ移行開始

MIRセット（移行メモ）

- 現行の参照ドキュメントは「26命令（Canonical）」を維持（`docs/reference/mir/INSTRUCTION_SET.md`）。
- 実装はCore-15へ段階移行中（TypeOp/WeakRef/Barrier 統合、Print Deprecated）で、MIR15のカバレッジは `MIR15_COVERAGE_CHECKLIST.md` で運用。
- Core-15が安定した時点で参照ドキュメント側を「15命令」に更新し、命令数固定テストも切替える。

- [x] JIT/AOT: 将来のGCバリア/セーフポイント用のシンボル予約（nyash.gc.barrier_write, nyash.rt.checkpoint）

環境変数（段階移行トグル）

- NYASH_BUILDER_SAFEPOINT_ENTRY=1: 関数エントリにSafepointを発行
- NYASH_BUILDER_SAFEPOINT_LOOP=1: ループ各回でSafepointを発行
- NYASH_BUILDER_LEGACY_WEAK=1: 旧WeakNew/WeakLoad発行を有効化（既定: 無効、WeakRefに統一）
- NYASH_BUILDER_LEGACY_BARRIER=1: 旧BarrierRead/Write発行を有効化（既定: 無効、Barrierに統一）
- NYASH_BUILDER_DISABLE_TRYCATCH=1: try/catch/finallyを無効化（try本体のみ）
- NYASH_BUILDER_DISABLE_THROW=1: throwをenv.debug.traceへフォールバック
- NYASH_REWRITE_DEBUG=1: Debug命令をExternCall(env.debug.trace)に書き換え
- NYASH_REWRITE_SAFEPOINT=1: Safepoint命令をExternCall(env.runtime.checkpoint)に書き換え
- NYASH_REWRITE_FUTURE=1: FutureNew/Set/Await を ExternCall(env.future.*) に書き換え（スキャフォールド）
- NYASH_DEBUG_TRACE=1: env.debug.traceのログをstderrに出力
- NYASH_RUNTIME_CHECKPOINT_TRACE=1: env.runtime.checkpointのログをstderrに出力

直近実装（完了）

- AOT/JIT: string-like hostcalls 実装（concat_hh/eq_hh/lt_hh）とLowerer経路、シンボル登録
- Print命令の非推奨化: BuilderでExternCall(console.log)へ統一、Rewriteでも変換
- Builder（legacy抑止のトグル）: Safepoint/Try-Catch/Throwをトグル化、loop safepointも任意化
- Runtime extern_call: env.debug.trace / env.runtime.checkpoint を追加

次の着手（順序）

1. JIT/AOT: GCバリア/セーフポイントのシンボル予約と下準備（nyash.gc.barrier_write, nyash.rt.checkpoint）
2. Docs: 上記トグル/Extern API/命令マッピングの追記（INSTRUCTION_SET, runtime extern, migration）
3. Future/AwaitのRewriteスキャフォールド（NYASH_REWRITE_FUTURE=1）と最小実装方針の明文化（完了）
4. Builderのlegacy API（emit_weak_new/load, barrier_read/write）の非推奨化と使用箇所の削減
5. JIT directのBlock-Sealパニック修正（block seal順序・entry sealの見直し）

期待効果

- 命令 37→15（目安）で読みやすさ/実装コスト/検証コストを大幅削減
- JIT/AOT の対応面積が小さくなり、今回の string-like hostcall のような追加の導入が容易に
- 「Everything is Box」に合致（I/O, 例外, ランタイム機能をBox/Externに集約）

優先度/スケジュール

- 優先度: 最優先（10.5c/10.7に割り込み）
- 目安: 1〜2日でBuilder/Rewrite/Docs、続いてVM/JITの掃除を段階投入


直近スナップショット（2025-08-30 更新）

Current State

- Plugin-First/Handle-First/TLVはAOT/VMで安定（10.5e完了状態を継続）
- 10.6計画（Thread-Safety/Scheduler）と10.7計画（トランスパイルAll-or-Nothing）を確定
- Nyash-onlyパイプライン（tools/pyc）を開始（Parser/CompilerはNyashで実装方針）
- include式の最小実装を追加（式でBoxを返す／1ファイル=1static box）
  - インタプリタ: include式は実行時評価
  - VM/AOT: MIRビルダーが取り込み先を同一MIRに連結（MIR命令は増やさない）
  - nyash.tomlの[include.roots]でルート解決（拡張子省略、index.nyash対応）
- tools/pycをモジュール分割
  - tools/pyc/pyc.nyash（エントリ: includeでPyIR/PythonParserNy/PyCompilerを取り込み）
  - tools/pyc/PyIR.nyash, PythonParserNy.nyash, PyCompiler.nyash（Nyash-only実装）

How To Run（Nyash-only）

- VM: `NYASH_PY_CODE=$'def main():\n  return 42' ./target/release/nyash --backend vm tools/pyc/pyc.nyash`
  - 出力: Parser JSON → IR（return 42）→ 生成Nyashソース（現状は骨組み）
- include動作サンプル: `./target/release/nyash --backend vm examples/include_main.nyash`（Math.add(1,2)=3）

進捗（2025-08-30 夜）

- include: 循環検出を追加（インタプリタ/VM収集器ともにロード中スタックで経路出力）。examples/cycle_a/b で検証
- tools/pyc: 最小IR（return定数）→Nyash生成を通し、出力をprintまで接続
- 文字列基盤: VMにString統一ブリッジを着手（内部StringBoxとプラグインStringBoxの比較互換、内部Stringメソッドのフォールバック）
- 追加プラグイン（小粒・基底）
  - RegexBox（compile/isMatch/find/replaceAll/split）: examples/regex_min.nyash
  - EncodingBox（utf8/base64/hex）: examples/encoding_min.nyash
  - TOMLBox（parse/get/toJson）: examples/toml_min.nyash
  - PathBox（join/dirname/basename/extname/isAbs/normalize）: examples/path_min.nyash

Next Steps（優先順・更新）

1. String統一ブリッジ（実装済・一次完了）
   - VM: 比較/加算/代表メソッドのフォールバック（length/isEmpty/charCodeAt/concat/+）をstring-like正規化で実装
   - Interpreter: 比較/加算はstring-like正規化を適用（メソッドは後続で最小追補があれば対応）
   - 例: encoding_min/regex_min/toml_min/path_min で回帰確認
2. AOT/JITへのブリッジ降ろし（MIR→VM→JIT→exeの汎用性維持・ハードコーディング禁止）
   - 文字列演算のhostcall化（read-only）: nyash.string.concat_hh / eq_hh / lt_hh
   - Lowerer: BinOp(Add) / Compare(Eq/Lt) を「string-like」判定時にhostcallへフォールバック
   - 代表メソッド: length/isEmpty/charCodeAtは既存hostcall経由で維持、concat(メソッド)も追加検討
   - Registry: 署名/権限（ReadOnly）登録、シンボル解決とJITビルダー登録
   - 目標: examples/string_bridge_min.nyash をAOTでも成功
3. tools/pyc: IR→Nyashの反映強化（return/If/Assignを安定化、Strictスイッチ連動）
4. Strictスイッチ: tools/pyc（unsupported_nodes非空でErr、envでON/OFF）
5. CLI隠しフラグ `--pyc`/`--pyc-native`（Parser→Compiler→AOTの一本化導線）
6. 最小回帰（VM/AOTの差分記録）とdocs追補（include/exportとpyc、Regex/Encoding/TOML/PathのAPI概要）

Env Keys（pyc）

- NYASH_PY_CODE: Pythonソース文字列（Nyash-onlyパイプライン/Parser用）
- NYASH_PY_IR: IR(JSON)直接注入（Rust雛形Compilerの確認用・オプション）

目的: Handle-First + by-name を軸に、Python統合（PyRuntimeBox/PyObjectBox）を汎用・安全に実装する。最適化は後段。さらに10.7のNyash-onlyトランスパイルC2（pyc）を最小構成で立ち上げる。

ステータス（2025-08-30 更新）

- フェーズ: 10.5c 汎用Handle/TLV実装の拡張（Python統合開始）
- 方針: 「綺麗に作って動かす」= ハードコーディング排除・Handle/TLV統一・最適化は後回し

10.5b 完了項目（橋渡し済み）

- by-name シム（getattr/call）を実装（JIT/AOT）し、Lowerer から a0 を `nyash.handle.of` で確実にハンドル化して呼び出し
- 引数 a1/a2 はハンドル優先／なければレガシー参照から TLV 構築（String/Integer はプリミティブ化）
- 汎用 birth シムを追加
  - `nyash.box.birth_h(type_id:i64)->i64`（JIT/AOT）
  - `nyash.box.birth_i64(type_id:i64, argc:i64, a1:i64, a2:i64)->i64`（JIT/AOT）
  - Lowerer: NewBox（引数無し）は birth_h に統一。引数ありは安全なケース（Integer const／引数が既にハンドル）だけ birth_i64 に段階導入
- AOT: examples/aot_py_math_sqrt_min.nyash で Strict でも .o 生成を確認（target/aot_objects/main.o）
- ログ
  - AOT: NYASH_CLI_VERBOSE=1 で birth_h の可視化
  - JIT: events で by-name/birth の観測（必要十分の最小限）

10.5c 着手項目（進行中）
- Lowerer: PluginInvoke（type_id/method_id & by-name）の Handle-First 配線を統一（a0を常にnyash.handle.of）
- JIT/AOT: birth（_h/_i64）と by-name シムでTLV生成を汎用化（String/Integerはプリミティブ化、他はHandle）
- Strict時のJIT実行停止（コンパイル専用）でVM=仕様の原則を徹底

非対応（後回し・最適化）

- StringBox 専用の known_string/再利用最適化
- 汎用的な定数プール／birth の可変長 TLV 一括最適化

次の作業（10.5c 続き）

1) FFI仕様の短文化（a0/a1/a2=Handle優先→TLV、レガシー抑止フラグ、戻りTLVのdecodeポリシー）
2) birth引数の一般化メモ（可変長TLV、例外時ハンドリング）
3) Python統合の最小チェーン（import→getattr→call）のAOT/VM双方での実装確認サンプル追加
4) ドキュメント更新（10.5c README/INDEX、FFIガイド）

合意済みルール

- まず汎用・安全に動かす（最適化は内部に隠し、後段）
- StringBox 等の個別特化は入れない。Handle/TLV で統一し、Box 追加を阻害しない
- Strict/Fail‑Fast を維持（fallback で隠さない）
Update (2025-09-02 AM / Async unify + VM await fix + JIT AOT builder plan)

- What’s implemented (since last update)
  - Interpreter/VM/JIT await semantics unified to Result.Ok/Err.
    - Interpreter: await now returns Ok(value) or Err("Timeout") with cooperative polling and NYASH_AWAIT_MAX_MS guard.
    - VM: execute_await() changed to safepoint + scheduler.poll loop with timeout → Err("Timeout") on expiry, Ok(value) on success.
    - JIT: await_h produces handle (0 on timeout), then ok_h/err_h wrap into Result.Ok/Err (already wired).
  - TaskGroup scaffolding connected to Interpreter
    - nowait registers Future into implicit TaskGroup; function/static/parent calls push/pop task scopes to enable join on scope exit.
  - TokenBox added as a first-class Box
    - New Box: TokenBox (wraps CancellationToken). Externs: env.task.currentToken() → TokenBox, env.task.cancelCurrent() → cancel current scope token.
  - Delay future (scheduler-backed)
    - Extern: env.future.delay(ms) → FutureBox that resolves to void after ms (uses SingleThreadScheduler.spawn_after or thread fallback).
  - CLI result normalization (interpreter path)
    - When printing results, prefer semantics::coerce_to_i64/coerce_to_string and special-case plugin IntegerBox.get() so “IntegerBox(id)” prints as numeric value.

- New samples (smoke-friendly)
  - apps/tests/mir-safe-min: minimal MIR (plugins disabled)
    - Run: `NYASH_DISABLE_PLUGINS=1 ./target/debug/nyash --backend mir apps/tests/mir-safe-min/main.nyash` → Result: 3
  - apps/tests/async-nowait-basic: Interpreter nowait/await using threads
    - Run: `NYASH_DISABLE_PLUGINS=1 ./target/debug/nyash apps/tests/async-nowait-basic/main.nyash` → Result: 33
  - apps/tests/async-scope-token: VM token + delay demo (plugins on)
    - Run: `./target/debug/nyash --backend vm apps/tests/async-scope-token/main.nyash`
    - Output: token: …; after delay; token after cancel: …; Result: 0
  - apps/tests/async-await-timeout: VM await timeout demo
    - Run: `./target/debug/nyash --backend vm apps/tests/async-await-timeout/main.nyash`
    - Output: `Err(Timeout)` then Result: 0

- JIT (execute) status
  - `--backend cranelift` (skeleton) runs: `apps/tests/mir-branch-ret` → Result: 1
  - JIT-direct path compiles/executes for simple cases (single-exit return strategy in place, PHI materialization to locals, etc.).

- JIT (AOT/EXE) current blocker and plan
  - Symptom: jit-direct path panics in Cranelift FunctionBuilder finalize: “FunctionBuilder finalized, but block block0 is not sealed”.
  - Root cause: Current CraneliftBuilder repeatedly creates short‑lived FunctionBuilder instances and finalizes them per emission step; sealing discipline diverges from expected pattern (single FunctionBuilder per function, seal blocks after predecessors known). Entry sealing/ret-epilogue sealing were added, but per‑step finalize still violates constraints.
  - Plan (box-first, clean layering)
    1) Refactor CraneliftBuilder to hold a single FunctionBuilder per function lifetime.
       - Maintain current block, value stack, and IR emission without re‑creating/finalizing FB on every op.
       - Emit jump/branch/hostcall/phi consistently in the same FB.
       - Seal blocks when predecessors are determined (via LowerCore callbacks), and perform a final seal sweep before define_function.
    2) Keep ObjectBuilder (AOT .o path) returning directly (no ret_block), unchanged aside from any minimal alignment with single‑FB pattern (it already returns directly and finishes module per function).
    3) Target sample: apps/tests/mir-branch-ret for first green AOT object emission.
       - Success criteria: tools/build_aot.sh invoked via `--compile-native -o app` produces an executable that prints Result: 1.
    4) After branch/ret green, extend to minimal PHI case (mir-phi-min) ensuring paramized block args are declared prior to seals.

- Interim guidance
  - For JIT testing use `--backend cranelift` (skeleton exec) or VM path with `NYASH_JIT_EXEC=0` unless running jit-direct read‑only smokes.
  - For AOT/EXE, wait for the single‑FB refactor merge; current tools/build_aot.sh in strict mode forbids fallback and will fail on the sealing assertion.

- Env toggles / helpers
  - Await timeout: `NYASH_AWAIT_MAX_MS` (default 5000)
  - Scheduler trace/budget: `NYASH_SCHED_TRACE=1`, `NYASH_SCHED_POLL_BUDGET=N`
  - JIT lower dump/trace: `NYASH_JIT_DUMP=1`, `NYASH_JIT_TRACE_RET=1`, `NYASH_JIT_TRACE_BLOCKS=1`
  - JIT policy (read-only in jit-direct): `NYASH_JIT_STRICT=1` and policy.read_only enforced

- Next actions (execution order)
  1) CraneliftBuilder: single FunctionBuilder per function（finalize at end_functionのみ）。
     - Remove per‑op new/finalize; switch emit_* to use the persistent FB.
     - Seal entry immediately; seal successors when wiring is complete; final global seal sweep before define_function.
  2) Verify jit-direct with `apps/tests/mir-branch-ret` (NYASH_JIT_THRESHOLD=1).
  3) Enable AOT object emission for the sample and link via tools/build_aot.sh; run resulting EXE (expect Result: 1).
  4) Extend to `mir-phi-min` (ensure ensure_block_params + sealing order correct).
  5) Wire tri-backend/async/timeout smokes in tools/ (minimal, concise outputs) and add to CI.
## Phase 12 — Handoff (VM/JIT 統一経路・Nyash ABI vtable/by-slot)

Updated: 2025-09-03 — Quick Handoff Summary（長文は下に残し、ここを最新ソースに）

目的
- VM/JIT を vtable/by-slot で統一し、Extern と BoxCall の経路分離（BoxCall→vtable/PIC/汎用、Extern→name/slot）を確立。
- 逆呼び（plugins→host）をTLV＋スロットで安定化。JIT/VM の安全な境界（TLS/GCバリア）を担保。

今回の到達点（実装済み）
- JIT host-bridge 完配線（Cranelift thunk＋シンボル登録）
  - Instance.getField/setField: 統一3引数シンボル（field3: (recv,name,val/-1)）で by-slot 呼び出し
  - String.len: 受け手 StringBox は host-bridge 経路へ
  - 文字列リテラル→StringBox ハンドル化: `nyash.string.from_u64x2`＋builder API
  - NewBox(Instance, 引数なし): `nyash.instance.birth_name_u64x2` でグローバルUnifiedRegistry経由生誕
- Lowering 強化
  - Instance.getField/setField と String.len を最優先ルートに（simple_reads より前）
  - Const(String) の伝搬（known_str）を追加 → name/val を確実にハンドル化
- 一致テスト（ローカル）
  - OK: `identical_exec_string`（"hello".len → 5）
  - OK: `identical_exec_instance`（Person.setField/getField → "Alice"）

How to Run（再現手順）
- 前提: `--features cranelift-jit`
- 文字列/インスタンス一致
  - `NYASH_ABI_VTABLE=1 NYASH_JIT_HOST_BRIDGE=1 cargo test --features cranelift-jit --lib tests::identical_exec_string::identical_vm_and_jit_string_len -- --nocapture`
  - `NYASH_ABI_VTABLE=1 NYASH_JIT_HOST_BRIDGE=1 cargo test --features cranelift-jit --lib tests::identical_exec_instance::identical_vm_and_jit_person_get_set_slots -- --nocapture`

主要フラグ
- `NYASH_ABI_VTABLE=1`（VM vtable）
- `NYASH_JIT_HOST_BRIDGE=1`（JIT host-bridge 経路）
- 任意: `NYASH_JIT_TRACE_BRIDGE=1`（ブリッジ経路の最小ログ）

主な変更点（ファイル）
- Host-bridge/Thunk: `src/jit/extern/host_bridge.rs`, `src/jit/lower/extern_thunks.rs`
- Lowering: `src/jit/lower/core/ops_ext.rs`, `src/jit/lower/core.rs`, `src/jit/lower/builder/{cranelift.rs,builder.rs,noop.rs}`
- JITエンジン経路: `src/backend/cranelift/mod.rs`, `src/jit/engine.rs`
- Runtime/Registry: `src/runtime/{nyash_runtime.rs,unified_registry.rs}`（既存）
- テスト: `src/tests/{identical_exec_string.rs,identical_exec_instance.rs}`（Factory注入を追加）

未了/次の一手（小さく）
- Collections/Reverse-call サブセットをVM/JITで再確認（Map/Array by-slot の最小一致）
- vtable_* ユニットの NewBox 失敗時はテスト内で Factory 注入（必要箇所のみ）
- CI: 一致系サブセット（string/instance/host_reverse）を first-wave に追加

メモ/注意
- host-bridge シンボルは固定アリティ・固定戻り（i64）で宣言し、call-site側で戻り値の使用有無を制御
- NewBox(Instance) はJIT側で UnifiedRegistry を直接叩くため、グローバルに必要Factoryを登録しておく（テスト内で注入済み）

（以下、旧詳細ログは履歴のため残置）
  - 第4段: VM 基本状態を `src/backend/vm_state.rs` へ抽出（完了）
    - 移動: `new/with_runtime`、`get_value/set_value`、`record_instruction`、`jit_threshold_from_env`、`loop_execute_phi`（各 `impl VM`）
    - `mod.rs` に `mod vm_state;` を追加。各呼び出し元のシンボルは従来どおり `VM::...` で参照可。
    - ビルド: 成功。

現状のレイアウト（A3 途中）
- `backend/vm.rs`: VM 本体（構造体・値型・最小グルー）。現在 ~1295 行（旧メソッド退避を除き圧縮済み）
- `backend/vm_exec.rs`: 実行エントリ/ループ/1命令実行
- `backend/vm_gc.rs`: ルート領域 API と GC 診断出力
- `backend/vm_state.rs`: 生成・状態・値アクセス・統計・phi 補助
- `backend/vm_values.rs`: 算術/論理/比較の内部演算
- `backend/vm_instructions/`: 命令ハンドラ群
- `backend/vm_boxcall.rs`: VTable/PIC スタブと BoxCall 補助
- `backend/dispatch.rs`: MIR 命令 → 実行関数 振り分け

次の分割（提案 / おすすめ）
- S1) `vm_methods.rs` 抽出（Box メソッドディスパッチ）
  - 対象: `VM::call_box_method`（大ブロック）＋`call_unified_method` ラッパ
  - 期待効果: `vm.rs` を < 1000 行へ。呼び出しは現行どおり `VM::call_box_method`。
- S2) `vm.rs` 旧プレースホルダ（`*_old`, `execute_module_old_moved` など）を段階削除
  - 互換検証後に削除してノイズ低減。
- S3) `vm_types.rs`（任意）
  - `VMError`/`VMValue` 定義を分離し参照しやすく。
  - ただし変更範囲が大きいため最後に予定。
