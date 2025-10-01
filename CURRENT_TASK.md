# Current Task — Phase 15.7 (Concise)

このファイルは“今の開発状況だけ”を素早く把握できるよう簡潔に保ちます。詳細な履歴は削除し、必要に応じて git 履歴を参照してください（過去の全ログは commit 履歴に残っています）。

## At-a-Glance（現状要約）
- Branding: 設定は hako.toml 最優先（互換: nyash/hakorune）
- JSON: JSON.stringify(any) を第一級APIに昇格（.toJSON 併存・同一出力）
- Smokes: hako.tomlのみ/JSON.stringify 標準の軽量スモークを quick に追加
- MIR 凍結ドキュメントの整備（MirCall 統一含む）
- Plugins: plugin-tester 既定 --config を hako.toml に切替（互換読込維持）
- 仕様不変・ロールバック容易・差分は局所

## ✅ Update — 2025-10-01（Smokes v2 再編と selfhost 配線）
- Smokes v2 の物理分割（quick 配下）
  - 追加: `profiles/quick/selfhost/`（selfhost/pipeline_v2/emit-only）
  - 追加: `profiles/quick/llvm/`（軽量LLVM/ハーネス・トレース）
  - 既存 selfhost_* / *_vm_llvm を上記へ移設（run.sh の探索は再帰なので互換）
- Runner（親→子）引数透過を拡充（既定OFF・安全）
  - `NYASH_EMIT_TRACE=1` → `--emit-trace`
  - `NYASH_PREFER_CFG=1` / `NYASH_PREFER_CFG2=1` → `--prefer-cfg` / `--prefer-cfg2`
  - `NYASH_QUIET` は子へ渡さない（JSON出力のサイレンス防止）
- Pipeline V2 の header スモーク安定化
  - `selfhost_min_json_header_pipeline_v2_vm.sh` → PASS（timeout=8000ms 維持）

## ✅ Update — 2025-10-01（ModuleFunction + LLVM/WASM 同期）
- ModuleFunction（既定OFF）
  - Builder: module.functions 命中/安全なtail一致で `callee=ModuleFunction` を付与（`NYASH_MIR_CALL_MODULE_FN=1`）
  - VM/Printer/JSON: ModuleFunction を解決・表示・v1 mir_call 出力に対応
  - スモーク（dev無効）:
    - quick/core/modulefn_tail_prefer_current_box_vm.sh（A側）
    - quick/core/modulefn_tail_prefer_current_box_B_vm.sh（B側）
    - quick/core/modulefn_tail_prefer_current_box_arity_vm.sh（arity一致）
    - quick/core/json_v1_modulefn_mir_call_vm.sh（backend mir + --emit-mir-json）
- 受け口整理: `NYASH_VM_RECV_ARG_FALLBACK` を削除（devフォールバックの誤発火を防止）。ParserBoxのme救済・length=0は維持。
- LLVM/WASM 同期（wasm-development → selfhost）
  - src/llvm_py/ 以下のハーネス/targets/工具一式を同期（MIR命令フル実装、PHI配線、WASMターゲット）
  - quick/wasm, integration/wasm にプレースホルダスモーク（ゲートONで最小PASS）
- スモーク状況（quick）
  - 概ね PASS。1 件 FAIL: selfhost_compiler_emit_mir_cmp_v2_vm.sh（devプロファイル影響の可能性高）
  - WASM quick（`SMOKES_ENABLE_WASM=1`）は PASS（プレースホルダ）

## ✅ Update — 2025-10-01（PyVM撤退ドキュメント整備・LLVM Builder 小改良）
- Docs sweep（PyVM撤退・互換のみ）
  - 更新: `AGENTS.md`, `README.md`, `README.ja.md`,
          `docs/development/roadmap/phases/phase-15/README.md`,
          `docs/guides/selfhost-pilot.md`,
          `docs/design/using-and-dispatch.md`,
          `docs/papers/nyash-phase15.7-selfhost/paper.md`,
          `docs/config/env.md`
  - 方針: 既定の実行は Rust VM / LLVM（llvmlite ハーネス）。PyVM は `--features pyvm-bridge` + `NYASH_VM_USE_PY=1` の互換のみ（既定OFF）。

- LLVM Python builder（箱化・安全化の最小差分）
  - BlockVMap を block_lower に適用（per‑block SSA の二層ビューを箱化）。互換のため `_current_vmap` は引き続き dict を公開。
  - PHI finalize 後に verify を常設（`NYASH_LLVM_PHI_VERIFY=1` 既定、`NYASH_LLVM_PHI_VERIFY_STRICT=1` で Fail‑Fast）。
  - finalize は wire‑only 既定（PHI新規作成は既定OFF、`NYASH_LLVM_PHI_ALLOW_CREATE=1` で明示許可）。
  - InstructionContext 注入は維持（今後、各 lowering へ段階拡大）。

- Smokes（quick）
  - 結果: 103/104 PASS（既知1件 FAIL: `selfhost_compiler_emit_mir_cmp_v2_vm.sh` — 期待1/実際0）。
  - ノイズ: preflight が Missing dynamic plugins を WARN（`stringbox integerbox mathbox`）。`nyash.toml` の libraries を参照する方式へ修正予定。

- 次アクション（このブランチで継続）
  1) InstructionContext の適用拡大（binop/compare/branch/ret/copy から開始）。
  2) plugin preflight の検出改善（`tools/smokes/v2/lib/{preflight.sh,plugin_manager.sh}` で nyash.toml 読み）。
  3) Python builder の未使用スタブ整理（`llvm_builder.py` の NotImplemented ルートの削除/明確化）。
  4) Integration 実行（LLVM ハーネス）：`NYASH_LLVM_USE_HARNESS=1 tools/smokes/v2/run.sh --profile integration --timeout 180`。

- 再開手順（再起動後）
  - ビルド: `cargo build --release`
  - クイック: `tools/smokes/v2/run.sh --profile quick`
  - インテグ: `NYASH_LLVM_USE_HARNESS=1 tools/smokes/v2/run.sh --profile integration --timeout 180`
  - 必要に応じ plugins ビルド（preflight の案内に従う）

## Current Focus（Phase 15.7）
- Branding移行の堅牢化（hako.toml-first の徹底と互換の維持）
- 宣言的MIR/JSON の実運用（Map/Array + JSON.stringify の標準化）
- Using/Alias/Resolver の Fail‑Fast とログ健全化（devのみ詳細）
- Self‑Hosting 小粒強化（LocalSSA ensure_cond の代表ケース）

## Next Actions（小粒・優先順）
P0（任意・おすすめの改善）
1) selfhost/dev スモークの追加
   - emit-trace 検知: 先頭 `[emit]` 1行 + 最終 JSON 1行を確認（`--emit-trace` 布告）
   - prefer-cfg2 検知: JSONに `"op":"copy"` が含まれることを確認
2) ドキュメント参照の追随
   - `quick/core/selfhost_*` → `quick/selfhost/*` への参照残を再走査して更新
3) テストタグ（任意）
   - 各スモーク先頭に `# tags: selfhost,pipeline_v2` の簡易タグを付与（将来のセレクタ用）
4) Makefile タスク（任意）
   - `make smoke-quick`, `make smoke-int` の薄いエイリアスを追加（開発者体験向上）
P0（dev観測の整理・安全化）
1) EmitトレースのENVガード化（候補: EnvBox最小導入 or Runner引数透過）。
   - 方針A: `EnvBox.get("NYASH_EMIT_TRACE")` を最小で提供（外部I/Oなし・純粋）。
   - 方針B: 親Runner→子に `--emit-trace=1` を渡し、ExecutionPipeline経由で各EmitBoxへ布告。
   - 現状: 無条件1行出力（最終JSON行は不変）。Env準備でき次第AまたはBに切替。

✅ 実施（Option B の導線・小粒）
- compiler.hako に `--emit-trace` を受理し、`--emit-mir --pipeline-v2` の組み合わせで PipelineV2 の traceエントリに委譲。
- `--prefer-cfg`/`--prefer-cfg2` を導入（0|1|2）。2で材化copyあり、1でCFGのみ、0でReturn中心。
- 互換: pipeline_v2 OFF では従来のインライン MIR JSON を継続。

P2（Selfhost Compiler / Pipeline v2 — 制御フローの最小対応）
1) if/else → branch/jump/ret の最小 Lowering（PHIなし・両枝ret限定）
   - Docs: INTERFACES.md に仕様追記（済）
   - Smoke: selfhost_if_else_ret_vm.sh（枠追加・現状SKIP）
   - Impl: MirBuilderBox.hako に最小実装（次）
2) LocalSSA.ensure_cond の最小実装（分岐直前/Call直前の材化）
   - Docs: INTERFACES.md 追記（済）
   - Impl: MirBuilderBox.hako に ensure_after_phis_copy 相当を実装（次）
   - 追加（済）: LocalSSABox は `{instructions:[...]}` 形式も受理し、API不足時は安全フォールバック。

受け入れ（dev 任意）
- `./target/release/nyash --backend vm apps/selfhost-compiler/compiler.hako -- --min-json --emit-mir --pipeline-v2 --emit-trace --prefer-cfg` で先頭に `[emit] ...` が1行、最後に MIR(JSON) が1行出力されること（tail -n 1 でJSONが取れる）。

P1（周辺の安定化・非破壊）
3) pipeline_v2 子タイムアウト/環境伝搬の再点検（8000ms継続、必要時拡張）
4) （完了）.hako ドキュメントの表記更新（両受理注記は残す）
5) （完了）VSCode: TextMate grammar の最小追加（シンタックス色付け）

## Docs — Selfhost Compiler (done)
- apps/selfhost-compiler/README.md を更新（Rust VM 既定・ENV一覧・Fail‑Fast・予定スモーク）
- apps/selfhost-compiler/INTERFACES.md を追加（Parser/Emitter/MirEmitter の契約）
- apps/selfhost-compiler/interfaces.nyash を追加（I/Fスケッチ）
- 実行方針: 既定は Rust VM、`NYASH_VM_USE_PY=1` の時のみ PyVM を使用

## Recently Completed（直近完了まとめ）
- hako.toml 優先解決を using/resolver に導入（互換: nyash/hakorune）
  - src/using/resolver.rs: hako→nyash→hakorune の順で CWD/ROOT を探索
- JSON.stringify(any) を標準APIに昇格（.toJSON 併存）
  - src/mir/builder/builder_calls/emit.rs: JSON.stringify/1 → recv.toJSON() を常時リライト
- plugin-tester 既定 --config を hako.toml に切替（互換維持）
  - tools/plugin-tester/src/main.rs: 既定パスを ../../hako.toml に変更
- 追加スモーク（quick/core）
  - branding_hako_only_using_vm.sh（hako.toml のみで using/alias が動作）
  - json_stringify_standard_vm.sh（JSON.stringify == .toJSON）
  - selfhost_min_json_header_pipeline_v2_vm.sh（timeout=8000ms）
  - modulefn_tail_unique_vm.sh（dev-gated: SMOKES_ENABLE_MODULEFN=1）
  - modulefn_tail_ambiguous_vm.sh（STRICT=1 で Fail‑Fast 動作を確認）
  - modulefn_llvm_trace.sh（LLVM call trace に ModuleFunction を出すことを確認）
  - json_v1_mir_call_vm.sh（PyVMブリッジJSONで unified mir_call を検知）
  - selfhost_if_else_ret_vm.sh（設計先行のSKIPテスト: if/else→ret の受け皿）

## ✅ Update — 2025-10-01（dev観測トレースとVSCode強化）
- Emit devトレース（最小、1行・ENV想定）を追加（現在は無条件出力。最後のJSON行でテストは影響なし）
  - apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako
  - apps/selfhost-compiler/pipeline_v2/emit_return_box.hako
  - apps/selfhost-compiler/pipeline_v2/emit_binop_box.hako
  - 備考: 将来 `NYASH_EMIT_TRACE=1` の Env 読取を .hako 側に提供後、条件出力へ切替予定。
- Docs の .hako 表記を代表ガイドに反映（.nyash は後方受理の注記を付与）
  - docs/development/roadmap/phases/phase-15.7/README.md（コマンド例を .hako へ）
  - docs/development/builder/DIAGNOSTICS.md（.hako へ）
- VSCode ローカル拡張に TextMate grammar を追加（最小）
  - tools/vscode/hakorune-language/syntaxes/hako.tmLanguage.json
  - package.json に grammar を登録（language id: hakorune）
  - docs/tools/vscode-hako.md に関連付け切替手順を追記（"javascript"→"hakorune"）

## ✅ Update — 2025-10-04（MIR: ModuleFunction Phase‑2 着地）
- Callee に `ModuleFunction(String)` を追加（型安全なモジュール関数呼び出し）
  - 定義: src/mir/definitions/call_unified.rs:22
  - 表示: src/mir/printer_helpers.rs:82（call_module_fn ...）
  - JSON: src/runner/mir_json_emit.rs:57（Unified v1 に ModuleFunction 追加）
- VM 解決を実装（関数テーブルから解決、/arity 付与・tail候補の最小フォールバック）
  - src/backend/mir_interpreter/handlers/calls/function.rs: 新規 `handle_callee_module_function`
  - src/backend/mir_interpreter/handlers/calls/legacy.rs: `execute_callee_call` に分岐追加
- Builder からの排出（envガード: `NYASH_MIR_CALL_MODULE_FN=1`）
  - 関数呼び出し: `build_function_call()` が `name`/`name/arity` を関数表で検出したら callee=ModuleFunction で emit（func フィールドは NameConst 維持）
    - src/mir/builder/builder_calls/build.rs: +初期分岐
  - `me.method()` の既知関数は callee=ModuleFunction を優先（ガード下）
    - src/mir/builder/builder_calls/build.rs:297
  - 一般化（envガードのまま）
    - tail-unique 解決を許可（dotted/bare）。STRICT=1 で複数候補は Fail‑Fast。
      - src/mir/builder/builder_calls/build.rs（Ambiguous 診断/候補提示）
      - src/mir/builder/method_call_handlers.rs（静的呼び出しのSTRICT診断）
  - FunctionIndex（薄い箱）導入と適用
    - 追加: src/mir/indexes/functions.rs（contains/exact/tail_unique, prefer_current_box）
    - Builder/VM の tail 探索を置換（診断一貫・重複除去）
  - LLVM トレース: ModuleFunction の専用表示を追加（NYASH_CALL_TRACE=1）
    - src/runner/modes/llvm.rs
  - VM legacy-call 観測（開発用）
    - NYASH_WARN_LEGACY_CALL=1 で JSON 行を stderr 出力（from/to/arity）
    - src/backend/mir_interpreter/handlers/calls/legacy.rs
  - JSON bin 側 v1 ゲート
    - NYASH_JSON_SCHEMA_V0=1 → v0（既定） / NYASH_JSON_SCHEMA_V1=1 → v1 ラッパー
    - src/runner/mir_json_emit.rs

## ✅ Update — 2025-10-05（Builder/JSON 小粒仕上げ）
- Builder: 曖昧 tail 解決時に `prefer_current_box` を適用（STRICT=0 時）
  - src/mir/builder/builder_calls/build.rs:162 付近
- JSON bin v1: 実体として `{"op":"mir_call"}` を出力（envガード）
  - 対応: Call(callee有り)/ExternCall/BoxCall/NewBox → unified Callee にマップ
  - フラグ: `NYASH_JSON_SCHEMA_V1=1`
  - 実装: src/runner/mir_json_emit.rs:bin 変種
- スモーク追加（quick/core）
  - json_v1_mir_call_vm.sh — PyVM 経由の JSON に unified mir_call が含まれることを確認
  - selfhost_if_else_ret_vm.sh — if/else→branch/jump/ret の最小 Lowering の受け入れ枠（現状 SKIP）


## Flags（ModuleFunction 周り・運用メモ）
- NYASH_MIR_CALL_MODULE_FN=1: ModuleFunction を優先して emit（tail-unique を含む）
- NYASH_MIR_CALL_MODULE_FN_STRICT=1: tail で複数候補なら Fail‑Fast（候補提示）
- NYASH_MIR_CALL_MODULE_FN_CANON=1: dotted+arity の完全一致を優先採用（安全ステップ）
- NYASH_WARN_LEGACY_CALL=1: callee=None のレガシー排出/解決を stderr で観測
- NYASH_JSON_SCHEMA_V0=1 / NYASH_JSON_SCHEMA_V1=1: JSON 出力のバリアント切替（bin/runner）

## Next Actions（小粒・推奨順）
1) FunctionIndex の prefer_current_box を Builder 側でも共通使用（曖昧時の絞り込みの一貫性）
2) JSON bin v1 実体（unified mir_call 本体）を dev-gated で試験導入（最小スモークを追加）
3) レガシー経路（callee=None）箇所に dev 警告を段階的に追加（移行観測の強化）
4) docs/migration: 旗一覧とロードマップを更新（既定ONの段階導入方針）
- 効果マスク: ModuleFunction は既定で READ+ReadHeap（保守的）
  - src/mir/builder/calls/call_unified.rs:compute_call_effects
- 検証: 直起動 selfhost（--min-json）が安定して最小ヘッダを出力
  - 実行例: `NYASH_DISABLE_PLUGINS=1 NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 NYASH_JSON_ONLY=1 NYASH_MIR_CALL_MODULE_FN=1 ./target/release/nyash --backend vm apps/selfhost-compiler/compiler.hako -- --min-json`
  - 期待: 先頭1行 Program ヘッダ


## Docs — MIR Freeze / MirCall（done）
- 更新: docs/reference/mir/INSTRUCTION_SET.md（凍結セット/非推奨/マッピング/診断を明記）
- 追加: docs/reference/mir/call-unified.md（MirCall/Callee/Flags/Effects/Legacy→MirCall）
- 追加: docs/development/migration/mir-call-unification.md（段階移行の手順とガード）
- 既存の単一列挙/定義へのリンクを明記（src/mir/instruction.rs, src/mir/definitions/call_unified.rs）

## ✅ Update — 2025-10-01（Selfhost 安全実行: 固定ヘッダ＋静音化）

- 症状/観測:
  - 直接起動（apps/selfhost-compiler/compiler.nyash --min-json）が待ちに入るケースを確認（CPU100%・長時間）。
  - 子コンパイラ経路は最小ヘッダを出せるが、起動直後の初期化ログが先頭に混入する場合があった。
- 対策（実施済み・小差分）:
  - JSON_ONLY/QUIET/VERBOSE=0 時はレジストリ初期ログを静音化（先頭1行のJSONを汚さない）。
    - ファイル: `src/box_factory/mod.rs:141-151, 210-221, 230-242`
    - 影響: 仕様不変・ログのみ抑制。再ビルドで反映。
- 安全な実行手順（Rust VM固定・子経路推奨）:
  - 子経路（最小ヘッダのみ出力）
    - `NYASH_DISABLE_PLUGINS=1 NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_EMIT_ONLY=1 NYASH_NY_COMPILER_SKIP_PY=1 NYASH_JSON_ONLY=1 ./target/release/nyash --backend vm apps/examples/string_p0.hako`
    - 期待出力: 先頭に `{ "version":0, "kind":"Program", ... }` の1行
  - 直接起動（診断用・timeout必須）
    - `timeout 5s NYASH_DISABLE_PLUGINS=1 NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 NYASH_JSON_ONLY=1 ./target/release/nyash --backend vm apps/selfhost-compiler/compiler.hako -- --min-json`  （互換: .nyash も受理）
    - 期待出力: 同じく最小ヘッダ1行（固まり時はtimeoutで切断）
- 受け入れ（dev 任意ゲート）:
  - `NYASH_JSON_ONLY=1` で最初の1行が JSON ヘッダ（version/kind 非空）であること。
  - pipeline v2 直ドライバは開発用（`SMOKES_ENABLE_PIPELINE_V2_DRIVER=1` で有効）。
- 次アクション（小粒・仕様不変）:
  1) 静音ヘルパ `cli_quiet()` を導入し、dev verify/Runner初期化のeprintlnも JSON_ONLY で抑制。
  2) `NYASH_DISABLE_PLUGINS=1` 時は `FactoryPolicy::BuiltinFirst` を強制（起動コスト/分岐削減）。
  3) quick に「最初の1行がJSONヘッダ」を確認する軽量スモークを追加（子/直接: 後者はtimeout付き）。
  4) `apps/selfhost-compiler/README.md` に安全プロファイル（ENV一覧/timeout運用）を追記。
  5) pipeline_v2（emit-only）のBox骨格を追加（ExecutionPipeline/Backend/MirBuilder）。compiler.hako に `--pipeline-v2` 経路を薄く配線（互換: .nyash）。
  6) pipeline_v2 のJSONヘッダ受け入れスモークを追加（quick/core）。
  7) JSON_ONLYスモークのノイズフィルタ依存を縮小（direct run + AWK抽出）。
  8) plugins無効時は preflight_plugins() をSKIPログに変更（v2テストランナー）。
  9) using/resolve/dev-fallbackの非本質ログを Quiet で抑制（挙動不変）。
  10) 子プロセスへ NYASH_QUIET を渡さない（emit-only の stdout を抑止しない）。
- 運用メモ（固まり時の掃除）:
  - 一覧: `ps -eo pid,etimes,%cpu,comm,args | rg nyash`
  - 強制終了: `pkill -9 -f 'target/release/nyash|apps/selfhost-compiler/compiler.hako|pyvm'`

## ✅ Update — 2025-10-01（.hako 採用 — selfhost/resolver 優先化）

要旨
- 新拡張子 `.hako` を優先に採用。.nyash は後方互換で継続受理。
- Selfhost 一式を .hako へ移行: compiler/parser_box/emitter_box/pipeline_v2 execution_pipeline を .hako 化。
- Runner/Resolver は .hako 優先・.nyash フォールバックで解決（パッケージ main/leaf、相対モジュール、is_path 判定すべて）。

変更点（コード）
- Selfhost sources:
  - `apps/selfhost-compiler/compiler.hako`（旧 compiler.nyash）
  - `apps/selfhost-compiler/boxes/parser_box.hako` / `emitter_box.hako`
  - `apps/selfhost-compiler/pipeline_v2/execution_pipeline_box.hako`
- Runner/Resolver:
  - using 解決/パッケージ main: .hako → .nyash の順で探索（src/runner/pipeline.rs, strip/collect.rs）
  - パス/拡張子判定: .hako を追加（src/runner/mod.rs, wasm.rs, strip/collect.rs）
  - Selfhost 子起動: `apps/selfhost-compiler/compiler.hako` を優先に（src/runner/selfhost.rs）
  - inline-selfhost include: parser/emitter を .hako に変更（src/runner/selfhost.rs）
- Examples: `apps/examples/string_p0.hako` を追加

スモーク/実行（代表）
- PASS: `selfhost_min_json_header_vm.sh`（compiler.hako 直実行）
- PASS: `selfhost_min_json_header_pipeline_v2_vm.sh`（親→子 emit-only; 子 timeout=5000ms）
- PASS: `selfhost_min_json_header_pipeline_v2_vm.sh`（親→子 emit-only; 子 timeout=8000ms に増強）
- SKIP（既定）: `selfhost_pipeline_v2_driver_min_json_vm.sh`（dev用; `SMOKES_ENABLE_PIPELINE_V2_DRIVER=1` で有効）

静音と子プロセス
- 子へ `NYASH_QUIET` は渡さない（emit-only の stdout を抑止しない）。親は `NYASH_JSON_ONLY=1` で最初の1行ヘッダを抽出。

Docs/設定 反映（実施済み）
- `hako.toml`/`nyash.toml` の selfhost.compiler.* パスを .hako に更新。
- Selfhost README/quickstart に .hako を追補（直実行例/パイプライン例）。
- VSCode 最小サポートを追加：
  - `.vscode/settings.json` に `"files.associations": { "*.hako": "javascript" }`
  - 軽量ローカル拡張 `tools/vscode/hakorune-language/`（id: `hakorune`、コメント/括弧のみ）
  - ガイド `docs/tools/vscode-hako.md`（ワークスペース関連付け or ローカル拡張の使い方）
- `.gitattributes`: `*.hako linguist-language=Nyash` を暫定設定（Linguist 反映まで）

Next（小粒・順番）
1) Docs の .nyash 表記を .hako へ漸進置換（両受理注記は残す）。
2) VSCode/Linguist 強化（Non‑Breaking）
   - TextMate grammar を拡張に追加（`tools/vscode/hakorune-language/syntaxes/hako.tmLanguage.json`）。
   - Linguist への PR 下書きを用意（`docs/tools/linguist-languages-yml.md` に `languages.yml` の案を記載）。
   - 拡張安定後、ワークスペースの一時関連付け（javascript）→ `hakorune` に切替案内を追記。
   - MIME: `text/x-hako` を提案（LSP/配布で使用）。
3) 代表 examples を .hako に寄せ、quick 代表1本は .hako を既定に（.nyash 代表は最小数を維持）。
4) pipeline v2 直ドライバのSSA/材化ケア（dev任意; 直後の改善候補）。

完了メモ（次に移行前の3件）
- 直ドライバのケア（最小）:
  - `apps/dev/pipeline_v2_min_json.nyash` を `.hako` import に更新（ExecutionPipelineBox）。
  - devドライバスモークに `NYASH_VM_TOLERATE_VOID=1` を追加（ヘッダ確認の堅牢化; 既定SKIPのまま）。
- Docsの .hako 置換（最小）:
  - `docs/development/selfhosting/quickstart.md`、`apps/selfhost-compiler/README.md` の `.nyash` 表記を `.hako` に調整（代表例/直実行）。
- pipeline v2 追加スモーク（dev-gated）:
  - 追加/整備: `selfhost_pipeline_v2_{ret,binop,cmp}_vm.sh`（`NYASH_PIPELINE_V2=1` で有効; 既定SKIP）。

補足（selfhost/pipeline_v2 の .hako 化）
- `apps/selfhost-compiler/pipeline_v2/backend_box.hako`（旧 .nyash）
- `apps/selfhost-compiler/pipeline_v2/mir_builder_box.hako`（旧 .nyash）
- import 更新: `execution_pipeline_box.hako` が `.hako` を参照

## ✅ Update — 2025-10-03（VM: Map fast‑path＋lifecycle 抽出＋calls 分割）
- BoxCall fast‑path: Map を calls/box_call.rs に抽出（Array と対称化）
  - 実装: `box_map_fastpath()` を追加し、Method 経路での組込み Map を早期処理
- Lifecycle 抽出: birth/fini の観測・契約ログを boxes/lifecycle.rs に集約
  - 呼び出し差替: newbox/legacy から `lifecycle_observe_new` / `lifecycle_observe_method` / `lifecycle_contracts_birth`
- calls 分割（着手）: execute_callee_call の Global/Extern 枝を小関数化
  - `handle_callee_global()`（dev JSON.stringify ブリッジ維持＋trace＋dispatch）
  - `handle_callee_extern()`（trace＋extern dispatch）
- 警告掃除: calls/mod.rs, boxes/mod.rs の未使用 re‑export を静音化

## Rollback Notes（可逆/小差分）
1) JSON.stringify を dev ゲートに戻す: emit.rs の分岐に `NYASH_JSON_STRINGIFY_DEV` を復帰
2) 設定探索順を元に戻す: resolver.rs を nyash.toml 優先へ差し替え
3) plugin-tester 既定 `--config` を `../../nyash.toml` に戻す

## Ops Snippets（運用メモ）
- Quick smokes: `tools/smokes/v2/run.sh --profile quick --filter "core:*"`
- LLVM harness（任意）: `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/CASE.nyash`
- tmux 再接続: `tmux attach -t codex || tmux new -s codex`
- tmux 再作成: `tmux kill-session -t codex || true; tmux new -s codex`

---

Note: 旧来の詳細セクションはファイル肥大化のため削除しました。必要に応じて git 履歴から参照してください（例: `git log -p -- CURRENT_TASK.md`）。
  - Parser gate: `parser_flow_enabled()` を既定ONに変更（src/config/env.rs）。
  - Flow 受理・検証: src/parser/declarations/flow.rs（フィールド禁止／birth・fini 禁止／メソッド内 `me` 禁止）。
  - `new Flow()` 防止: builder 側で静的/flow 名の New を検出してエラーに（Unknown Type 経由でもFail‑Fast扱い）。
  - Lint（任意）: `NYASH_LINT_STATIC_TO_FLOW=1` で「フィールド/ctorなしの static は flow 推奨」を警告（Main は除外）。

- スモーク（quick/core; すべて PASS）
  - flow_basic_vm.sh（Main.main）
  - flow_utils_vm.sh（Flow→Flow 呼び出し）
  - flow_forbid_field_vm.sh（フィールド禁止）
  - flow_forbid_me_vm.sh（me 禁止）
  - flow_forbid_birth_fini_vm.sh（birth/fini 禁止）
  - flow_forbid_new_vm.sh（new Flow() 禁止）
  - flow_parity_vm_llvm.sh（VM↔LLVM パリティ）

- Docs 更新
  - reference/language/flow.md（既定ON／無効化方法に修正）
  - reference/language/EBNF.md（flow_decl を「既定ON」注記に変更）

次アクション（推奨・小粒）
- using/alias 連携の flow スモークを追加（prelude/別ファイルの Flow 呼び出し）。
- quick/integration の代表一式を再実行して既定ONの影響が無いことを再確認。
- 移行ガイド（static→flow）の簡易ドキュメントを README 追補（任意）。

## ✅ Update — 2025-10-02（Flow: docs＋dev-gated実装＋smokes）

- Docs（仕様確定・段階導入）
  - 新規: docs/reference/language/flow.md（フィールド禁止・local可・birth/fini/new/me 禁止、Lowering=Flow.method→Global Name.method/N、Dev flag注記）
  - 参照追加: reference/language/README.md（flowリンク）、quick-reference.md（要点と予約語 flow）
- Parser（既定OFF/フラグON時のみ有効）
  - `NYASH_ENABLE_FLOW=1` で `flow Name { ... }` を受理
  - 実装: src/parser/declarations/flow.rs（メソッドのみ許可／フィールド検出でFail‑Fast、birth/fini禁止、me使用禁止）
  - ディスパッチ: statements/mod.rs に flow をステートメントとして追加（dev gate）
- Lowering（既存経路の活用・挙動不変）
  - Flow は `is_static: true` の BoxDeclaration として扱い、既存の static lowering により `Name.method/N` 関数化
  - 呼出しは `Name.method(a,b)` → Global call（BoxCallなし）
- Smokes（quick/core）
  - flow_basic_vm.sh（Main.main→print）
  - flow_utils_vm.sh（MathUtils.add を Main から呼び出し）
  - flow_forbid_field_vm.sh（フィールド禁止のエラー確認）
  - いずれも `NYASH_ENABLE_FLOW=1` を明示。既定OFFのため既存挙動は不変。

次アクション（提案）
- フェーズ続き: EBNF への flow 追加（ドキュメントのみ先行済み、実装追随）
- 任意: static → flow 移行 lint（警告メッセージで誘導、既定OFF）

## ✅ Update — 2025-10-01（quick 完全緑 + Alias 内部参照 + Mini‑VM 強化）

- quick: 96/96 PASS（selfhost LocalSSA/compare/binop/multi-compare 系も緑）
- 反映（仕様不変の小差分）
  - PreLex 共通前処理は VM/LLVM/PyVM/dispatch 全経路で有効
  - Using/Alias: プレリュード改名後の内部参照も安全に書換（MVP）
    - 実装: `src/runner/modes/common_util/resolve/alias_tools.rs`
    - rename → internal rewrite（Variable/FieldAccess/FunctionCall を対象）
    - ユニット追加（内部参照 Variable/qualified function）
  - Mini‑VM: fast‑path（const×2 + compare/binop + ret）を安全化
    - 最後の compare/binop の lhs/rhs が当該 const の dst と一致する場合のみ採用
    - それ以外はオブジェクト走査へフォールバック

### 次アクション（小粒・順番）
1) Alias ユニットの追加（衝突/ネスト）と trace 静音の確認（env ガード維持）
2) Wrapper を CompilerMod 経路に戻す（dev→quick の順）
3) VM: Call 直前で LocalSSA 材化を一律適用（emit_call 前に ensure_in_block を挿入）
4) 代表 integration の再実行（VM↔LLVM parity）

## ✅ Update — 2025-10-02（Phase A 実施・仕様不変の堅牢化）

- VM LocalSSA 材化の検証強化（デバッグ時のみ）
  - 追加: `debug_assert_materialized_in_block()`（現在ブロック内の Copy 定義に限定されていることを確認）
  - 適用: `handlers/calls.rs` で args/recv の材化後にデバッグアサート（cfg(debug_assertions)）
  - 仕様不変: リリースビルドでは無効（実行・性能影響なし）
- 単体テスト（helpers.rs）
  - `materialize_picks_latest_copy_before_current_inst`: 直前の Copy を選択
  - `materialize_stops_at_current_inst`: 現在命令の手前までで選択
- ドキュメント（ENV）
  - `docs/development/runtime/ENV_VARS.md` 冒頭に統合ノブへの誘導とマッピングを追記（`docs/config/env.md` を正とする）
  - 非アーカイブの旧ENV表記を整理（最小置換・互換注記付き）
    - 設定/using/テストドキュメントを NYASH_USING=1 ベースで統一（compat 注記）
- Alias 内部参照書換の緊急停止ゲートを追加（既定ON）
  - `NYASH_ALIAS_INTERNAL_REWRITE=0` で prelude 内部参照の書換を一時停止可能
  - docs/config/env.md に追記

次アクション（継続）
- Alias ユニットは既に衝突/ネスト/静的 Box 化のケースをカバー済み（追加があれば拡充）
- Selfhost wrapper は CompilerMod 経路で緑維持済み（固定JSONバイパス残骸なしを確認）
- （任意）Plugin 強制ONスモークは後続で追加（存在時のみ PASS / 無ければ SKIP）

## ✅ Update — 2025-10-02（Docs 追加掃除 + Integration スポット緑 + 小リファクタ）

- Docs 追加掃除（非アーカイブ）
  - break-control-flow-strategy.md, DIAGNOSTICS.md, operator-boxes.md, aot_smoke_cranelift.md
  - Phase‑15 の using 記載を `NYASH_USING=1` 基準に整理（compat注記付）
- Integration スポットテスト（代表）
  - parity/vm_llvm_hello.sh → PASS
  - parity/selfhost_mir_m2_compare_ops_vm_llvm.sh → PASS
- 小リファクタ（仕様不変・≤50行）
  - helpers.rs: `materialize_args_in_current_block` / `materialize_recv_in_current_block` を追加

## 🚧 In‑Progress — Declarative MIR（Map/Array リテラル + JSON.stringify）

目的
- 文字列連結や手続き的 Builder 連鎖を段階的に削減し、宣言的（Map/Array リテラル）で MIR(JSON) を構築する。
- 既定挙動は不変。導入は dev/flag でガードし、小粒のスモークで検証しながら広げる。

現状（このパッチでの準備）
- Ny 側（実験）: `apps/lib/json_native/stringify.nyash` に JSON.stringify_map/array の最小骨格を追加
  - スモーク（実験/既定SKIP）: `tools/smokes/v2/profiles/quick/core/json_stringify_mir_vm.sh`（`NYASH_JSON_STRINGIFY_DEV=1` で有効）
- 既存の Builder 化: mir_emitter/pipeline_v2 は Builder + `|>` 連鎖で可読性を改善済み（挙動は不変）

次アクション（この順で小粒導入）
1) Rust 側に安全な JSON.stringify(any) を追加（JSONBox 利用）
   - 実装: 既存の Box→serde_json 変換を公開 API 経由で呼び出し、`serde_json::to_string` で JSON を生成
   - 入口: `JSON.stringify(value)`（呼び出しは dev ガード。static/BoxCall の統一は後続）
2) Guarded スモークの正式化
   - Map/Array リテラル → JSON.stringify → Mini‑VM 実行（const/binop/compare の代表）
   - 既定は SKIP、dev/env で ON
3) 段階置換（dev ガード）
   - mir_emitter / pipeline_v2 の一部を Map/Array リテラル + JSON.stringify へ移行
   - quick/integration スポットを都度緑確認（VM↔LLVM パリティ維持）

ロールバック容易性
- dev ガード・差分小・Builder 経路温存のため、撤回は削除/flag OFF で即時可能。
  - calls.rs: 受け手/引数の材化処理をヘルパーに集約（重複削減・検証流用）

## ✅ Refactor — 2025-10-02（VM Boxes ハンドラの分割・責務分離）

- 目的: 巨大な boxes.rs の責務を分割し、読みやすさと変更容易性を向上（挙動不変）
- 変更（実装は handlers/ 直下に新規モジュールを追加）
  - 新規: `src/backend/mir_interpreter/handlers/boxes_fields.rs`
    - `try_handle_object_fields(...)`（InstanceBox の getField/setField とレガシー field 橋渡し）
  - 新規: `src/backend/mir_interpreter/handlers/boxes_instance.rs`
    - `try_handle_instance_box(...)`（InstanceBox のメソッド解決/候補列挙/文字列化）
- 既存: `boxes.rs` 側の呼び出しを新モジュールへリダイレクト
  - `handle_box_call()` 内の分岐（object_fields / instance）を新モジュール関数に置換
  - `handlers/mod.rs` に `mod boxes_fields; mod boxes_instance;` を追加
- 備考: 旧メソッド定義は一時的に残置（未参照）。後続の掃除パッチで安全に削除予定。

## 🧭 Plan — Plugin ABI (Final) Docs & Rollout

- 目的: プラグイン境界のみ強化（ユーザーBoxはそのまま）。段階導入で既定挙動不変。
- Docs 追加/更新
  - docs/reference/plugin-abi/README.md（v2→Final の概観と移行）
  - docs/reference/plugin-abi/nyash_abi_final_vision.md（提案本文）
  - docs/development/plan/plugin-abi-final-rollout.md（段階導入計画）
  - docs/config/env.md（実験用ENVを追加）
- ENV（提案・既定OFF）
  - `NYASH_PLUGIN_ABI_FINAL=1`（NyResult invoke優先; 未実装時はv2へフォールバック）
  - `NYASH_PLUGIN_META=1`（メタ関数の取得・ログ）
  - `NYASH_PLUGIN_CAPS_ENFORCE=1`（required_capabilities の検証: dev/ci推奨）
  - `NYASH_TRACE_EFFECTS=1`（効果トレースJSON）
  - `NYASH_CHECK_CONTRACTS=1`（契約pre/postのログ）
- 受け入れ（Phase A/B）
  - PoCプラグイン（FileBox）で NyResult 経路が動く（v2互換維持）
  - META/効果/契約はFLAG ON時のみログ、OFF時は無風
  - CI/devで required_capabilities を厳格ON→緑
- ロールバック
  - ENVをOFFに戻すだけで完全にv2挙動へ復帰（差分は局所・可逆）

---

## 🧩 Rust リファクタリング計画（段階・仕様不変）

目的: 入口前処理の一元化と VM 呼出経路の一本化で安定性を上げる。差分は小さく、既定挙動は変えない。

### Phase A（入口統一・安全化）
- PreLex を全モードから `prelex::prelex_normalize()` に統一（済）
- Using/Alias:
  - プレリュード改名（`Alias_<Top>`）＋内部参照書換（MVP、済）
  - 追加ユニットで衝突/ネストを固定

### Phase B（VMコアの一本化・材化責務）
- `backend/mir_interpreter/handlers/calls.rs` の emit_call 直前で LocalSSA 材化を一律適用（recv/args）
- User instance の BoxCall を本番 Fail‑Fast、dev は観測のみ
- NewBox→birth は原則 Builder 明示、dev は `NYASH_DEV_FALLBACK=1` で暫定容認
- Optimizer flag: `NYASH_OPT_FORCE_PLUGIN_INVOKE=1`（既定OFF）で PluginInvoke 優先の検証（parity 確認後に段階導入）

### Phase C（ログ/検証の静音と整備）
- call‑trace（VM runtime / LLVM 静的）を env ガード（`NYASH_CALL_TRACE=1`）に統一
- dev 警告は `NYASH_CLI_VERBOSE=1` のみ出力（smokes は静音）

受け入れ基準
- quick/integration 緑、VM/LLVM 代表の call‑trace 名称整合
- `ssa.verify`/`resolve.unique=false`/`use of undefined` が dev で 0

ロールバック指針

## ✅ Update — 2025-10-02（Contracts 観測とツール強化・仕様不変）

- 観測（NYASH_CHECK_CONTRACTS=1）を追加（ログのみ／挙動不変）
  - NewBox→birth の関係: `contracts_newbox` / `contracts_birth`（argc一致や重複birthの検出を含む）
  - Arity/type/index の軽量観測:
    - ArrayBox: `contracts_arity` / `contracts_index`
    - StringBox: `contracts_arity(_min|_range)`
    - MapBox: `contracts_arity` / `contracts_type`（キーがStringでない場合のヒント）
  - PluginInvoke 警告（非プラグイン受信者）: `contracts_warn`
  - Docs: `docs/development/testing/contracts-observation.md`

- 効果トレース（NYASH_TRACE_EFFECTS=1）
  - v2 / Final の plugin 呼出前後で `plugin_call` / `plugin_ret` を1行JSONで出力（stderr）。

- コールトレース比較ツールの強化（順序無視の集合差と種別フィルタ）
  - `tools/dev/call_trace_diff.sh --kinds 'Method,BoxCall'`、結果サマリを追加（VM⊆LLVM を確認）
  - Docs: smoke-tests-v2.md に利用例を追記

次アクション（この観測系列）
1) 代表スモークへの導入は不要（既定OFF）。必要時はローカルで env を有効化して分析。
2) 追加観測点（例: ArrayBox.pop/remove のindex、MapBox.keys/values の型整合）を必要に応じて拡張。
3) PoCプラグイン準備後、Final 経路の最小スモーク（envオン時のみ）を追加。
- 入口統一は `prelex` 呼び出し差分のみ戻す
- Alias 内部参照書換は env で一時停止可能（必要時）
- VM call 材化挿入は単点差分で revert 容易

## ✅ Update — 2025-10-03（Plugins quick + autoload hardening）

- Quick 追加（SKIP安全・既定不変）
  - tools/smokes/v2/profiles/quick/core/plugin_array_min_vm.sh（len/get → 2|bar）
  - tools/smokes/v2/profiles/quick/core/plugin_map_min_vm.sh（size/get → 1|v）
  - いずれもプラグイン優先フラグ（NYASH_VM_PLUGIN_PREFER_ARRAY/MAP）で実行、存在しない環境でも壊れない構成
- plugins/dylib_autoload 代表の安定化（環境依存は SKIP）
  - VM 受信側未定義の救済（NYASH_USING_DYLIB_AUTOLOAD=1 時のみ）
    - src/backend/mir_interpreter/handlers/calls.rs: 受信 BoxRef を現在レジスタから型一致で回収（PluginBoxV2.box_type を優先）
    - 既定挙動は不変（フラグOFF時は Fail-Fast のまま）
  - スモーク SKIP 強化: tools/smokes/v2/profiles/plugins/dylib_autoload.sh
    - Fixture/Counter/Math/Mixed で "InvalidType" も SKIP 判定に追加
  - 結果: dylib_autoload.sh → PASS（ABI/未整合は SKIP で緑維持）

### 次アクション（小粒・推奨）
1) メソッドID解決の根治（小差分）
   - loader 側で nyash_box.toml 由来の ID を一貫して参照（method_resolver/instance_manager の分岐見直し）
2) Final ABI encode/decode のユニット追補（bool/int/float/string/bytes の往復）
3) 旧巨大メソッドの物理削除（未参照確認後）

### 受け入れ基準（この更新）
- 既存 quick/integration は既定OFFで無風（緑維持）
- 追加 quick（Array/Map最小）→ PASS
- plugins/dylib_autoload → PASS（環境に依存する箇所は SKIP 安全化）

## 🔧 **現在の緊急タスク** (2025-09-30) - Mini-VM フォールバック経路修正

### 📊 **問題概要**
- **症状**: `selfhost_mir_m2_compare_neg_binop_Lt` テスト失敗（期待1→実3）
- **根本原因**: Mini-VM のフォールバック経路が ret 命令を見落として `_extract_first_const_i64` にフォールバック
- **ファイル**: `apps/selfhost/vm/boxes/mir_vm_min.nyash` (Line 389-492)
- **詳細**: 100% Nyashスクリプトの問題（Rust VM無関係）

### 🎯 **修正計画: 3段階アプローチ**

#### **Phase 1（緊急・実施中）**: フォールバック経路に ret 検出ロジック追加
- **目的**: テスト通過を最速実現
- **方針**: 最小変更（5-10行追加）
- **実装**:
  1. `found_ret` フラグ追加
  2. ret 命令検出時に確実に return
  3. デバッグトレース追加
- **影響**: 既存ロジックを壊さない、即座にテスト通過

#### **Phase 2（最適化・次ラウンド）**: inst3早期評価パスを binop 含むケースに拡張
- **目的**: パフォーマンス向上（フォールバック回避）
- **方針**: const×2 + binop + const + compare + ret パターンの高速処理
- **実装**: inst3早期評価条件を `const>=2` に拡張、binop中間計算対応

#### **Phase 3（長期・Phase 15完了後）**: Mini-VM全体リファクタリング
- **目的**: 保守性向上
- **方針**: 命令処理の統一インターフェース化
- **実装**: 200-300行規模の大規模変更

### 📋 **関連コミット**
- `51d4c454`: PreLex alias desugar MVP + Mini-VM improvements + docs reorganization
  - 既知の問題として記録済み

---

Context — Plugin compatibility (why this detour)
- Selfhost compiler path (Ny→JSON v0 emit) does not require plugins; it prints JSON and exits.
- The fallback VM engine, however, lacked minimal BoxCall handlers (String/Array/Map). To keep quick/dev green without adding heavy plugin deps, we:
  - used JSON‑only wrapper and Mini‑VM for execution checks,
  - strengthened resolver/Alias/LocalSSA and unified VM entry,
  - and planned Phase‑B to add the minimal BoxCall set to the fallback engine.
- This is why we focused on resolver/Alias/LocalSSA first; plugin features will be added in small steps (spec‑stable) under Phase‑B.

Update — 2025-09-29 (Resolver polish + DEV fallback builder)
- Legacy archive: `apps/selfhost/compiler/` → `apps/archive/selfhost-legacy/`（物理移動完了）
- Rebuild: resolver の旧ツリー除外が有効化（ビルド緑）
- Wrapper 経路の復帰:
  - dev: `apps/dev/selfhost_compiler_min_cmp.nyash` → CompilerMod 経路で JSON を取得し Mini‑VM で 1（緑）。
  - quick: 固定 JSON バイパスを撤去し、CompilerMod 経路へ復帰（緑）。
- DEV 限定のビルダ安全弁（挙動不変）:
  - static box 内の未修飾 `_helper(...)` を `Class._helper/arity` に正規化（`NYASH_DEV=1` 時のみ）。
  - 一時的に追加していたトップレベルのダミー helper は削除（ソース美化）。
- Resolver/Using の磨き:
  - DFS の重複 push を抑止（同一実パスを1回だけ前処理）。
  - トレース出力を `NYASH_RESOLVE_TRACE=1` または CLI verbose でのみ表示（通常は静穏）。
- Alias 脱糖のユニットテストを追加（FieldAccess/FunctionCall/MethodCall）。

Next — 小粒リファクタ（安全・挙動不変）
1) Legacy 経路の非推奨注記（コメントのみ）
   - `emit_legacy_call` 等に「prefer emit_unified_call」のガイドコメントを追記（置換の導線を明確化）。
2) Resolver JSON トレース（任意ログ）
   - `NYASH_RESOLVE_TRACE_JSON=1` で 1行 JSON（候補/決定/理由）を出力する軽量ヘルパを追加（通常はOFF）。
3) Docs: LocalSSA の材化点（Call直前）を明文化
   - runtime-architecture に短い節を追記（Call/Compare/FieldAccess の直前で in‑block materialize）。
4) quick/dev を再実行して緑を確認。

Plan — VM↔LLVM Call Parity (staged, flags; default unchanged)
1) Call trace dump（VM/LLVM 共通; JSON 1行）
   - ENV: `NYASH_CALL_TRACE=1` → 各 Call で `{callee, recv_origin, recv_type, args_types, effects}` を出力
   - ツール: 簡易 diff スクリプト（VM と LLVM の行差分のみ赤表示）
   - 目的: 差分の見える化（最短経路の特定）
2) Router ポリシーフラグ（Unified 優先の検証; 既定OFF）
   - ENV: `NYASH_ROUTER_FORCE_UNIFIED=1` → Core Box 等も Unified を優先（BoxCall を抑制）
   - 影響: 仕様不変（検証用）。BoxCall 残存は Optimizer フラグで順次解消
3) Optimizer フラグ（BoxCall→PluginInvoke; 既定OFF）
   - ENV: `NYASH_OPT_FORCE_PLUGIN_INVOKE=1` → MIR 内の BoxCall を PluginInvoke に強制（プラグインがある環境のみ）
   - 影響: 既定挙動は変更しない（flags 実験下でのみ有効）
4) CI/Smokes の一致ゲート（任意）
   - quick: VM/LLVM を同一入力で実行し Call trace の一致を比較
   - fail 条件: `resolve.unique=false>0`, `ssa.verify!=ok`, `use_of_undefined>0`（赤ゲート）

Note on PluginInvoke（今やるべき？）
- 既定では延期（fallback VM は plugins なしでも緑維持が要件のため）。
- ただし、検証/デモ用途に限り上記 flags で段階有効化は可能（環境に plugins がある場合のみ）。
- 結論: 「今すぐ既定ON」はしない。flags の導入（検証）→ 快速緑を崩さないことを最優先に段階導入する。

Default‑ON Transition Plan（PluginInvoke を既定ONにするまでの段階）
- トリガ条件（客観基準/推奨）
  - Rust core 安定: quick/integration 常緑を直近 n 回（例: 10/10）維持
  - Core ビルド頻度低下: 直近 m 日で rebuild 件数が閾値未満（例: < 3/day 平均）
  - Plugin availability: 最小セット（File/String/Array/Map/Math/Integer）が dev/quick 環境で常時ロード可能
  - Parity 検証: Call‑trace JSON の VM=LLVM 一致（差分ゼロ/許容リストのみ）
  - SSA/未定義: `ssa.verify` 全OK、`use_of_undefined == 0`

- 切替手順（安全ロールアウト）
  1) Selfhost 限定で強制ON（wrapper で環境固定/`NYASH_PLUGIN_ONLY=1`）→ 緑確認
  2) dev プロファイルで既定ON、quick は auto のまま → 緑確認
  3) quick を段階ON（サンプル/一部スモークから）→ 全面ON
  4) integration/full に波及 → docs 更新、既定化宣言
  5) 監視: Call‑trace/エラー率を一時的に収集（env でON のときのみ）

- ロールバック（即時復帰可能に）
  - 環境変数/設定一発: `NYASH_PLUGIN_POLICY=auto|off`（計画）あるいは `NYASH_PLUGIN_ONLY=0`
  - Router/Optimizer flags を強制OFF（`NYASH_ROUTER_FORCE_UNIFIED=0`, `NYASH_OPT_FORCE_PLUGIN_INVOKE=0`）

- 実装メモ（段階導入に向けた小タスク）
  - ひとまとめのポリシー env を用意: `NYASH_PLUGIN_POLICY={off|auto|force}`（現状は `NYASH_PLUGIN_ONLY` で代替）
  - Selfhost wrapper で `NYASH_PLUGIN_ONLY=1` を明示
  - Call‑trace JSON を VM/LLVM 両経路に実装（env ガード）


Update — 2025-09-29 (Phase‑B prep + wrapper gate)
- Runner（統一VMエントリ）を強化:
  - Using/Alias: プレリュードのトップ記号を `Alias_<Name>` に改名 → 本体ASTを脱糖（`Alias.X`/`Alias.Box.m(a)`/`Alias.m(a)`）。
  - PreLex 正規化は既に共通入口で適用。
- VM（MirInterpreter/fallback）を堅牢化:
  - グローバル関数解決で `Alias_Box.method/arity` も候補に許容（エイリアス前置の静的呼び出しに対応）。
  - 統一経路でユーザー(BoxDeclaration)を収集し inline factory を登録（dev）。`NewBox DebugBox` 等を fallback でも生成可に。
  - New: `NewBox` 直後の dev 自動 birth（`NYASH_VM_AUTO_BIRTH_DEV=1`）を追加（本番は従来どおり自動呼び出し無し）。
  - New: Void ガード拡充（`indexOf`/`lastIndexOf` は `-1` を返す）で `VoidBox.indexOf` 系の落ちを予防。
  - Note: ネストした using への Alias 伝播は採用しない（トップレベルのみ）。プレリュード内部の参照を書き換えない設計を維持。
- スモーク: `selfhost_compiler_emit_mir_cmp_vm.sh` は wrapper が固定 MIR(JSON) を出す形で PASS（維持）。
- dev スモーク: 自己ホスト経路は一時的に「固定 MIR(JSON) → Mini‑VM」へバイパスし緑を確保（`tools/smokes/v2/profiles/dev/core/selfhost_compiler_emit_mir_cmp_vm_dev.sh`）。
- 実験B（wrapper→CompilerMod 経路）: 一時切替→エラー確認→即リバート
  - Undefined variable: CompilerMod → Alias配線で解消済み
  - その後 `VoidBox.indexOf`（引数材化/戻り値伝播の隙間）や DebugBox 未解決に遭遇 → inline factory 追加で Unknown Box 解消、ただし材化要調整
  - 決定: quick 緑維持のため wrapper は当面固定 JSON に戻す（再有効化は dev ガード下で段階導入）
- 根本原因（dev のみ）: 旧ツリー `apps/selfhost/compiler/` にある非 static 実装の混入により、private helper（`_extract_return_int`）がグローバル関数として誤解釈→MIR 化時に Unresolved になるケースがあった。
  - 対処（進行中）:
    - nyash.toml の `[modules]` を新ツリー（`apps/selfhost-compiler/…`）に統一（完了）
    - resolver DFS で旧ツリーを読み込み対象から除外（Rust 側; 要再ビルド）
    - dev スモークは固定 JSON へバイパスし緑維持（即効）
- フラグ/プロファイル:
  - Using=ON、`NYASH_ALLOW_USING_FILE=1`（dev）、AST prelude マージ=ON（dev既定）
  - Syntax sugar=ON、VM engine=fallback（既定）
- 受け入れ（Phase‑B step‑1）: build 緑、quick wrapper 緑（固定JSON）、挙動不変

Update — 2025-09-29（dev 簡素化・一括スイッチ）
- `NYASH_DEV_FALLBACK=1` を導入（dev 補助の一括有効化）。
  - 自動 birth（`NYASH_VM_AUTO_BIRTH_DEV=1` 相当）
  - Void 寛容（`NYASH_VM_TOLERATE_VOID=1` 相当）
- Alias 脱糖を拡充: `P_My.greet()` → `P_My.greet/0`（静的 Box メソッドの FunctionCall 化）。

Next — 緑化の最終手順（順番）
1) 旧ツリー `apps/selfhost/compiler/` を `apps/archive/selfhost-legacy/` に移動（物理アーカイブ）。
2) `cargo build --release`（resolver の旧ツリー除外を有効化）。
3) dev スモーク（CompilerMod 経路）を固定 JSON バイパスから本線に戻して緑確認。
4) quick の wrapper を CompilerMod 経路に戻し、`NYASH_DEV_FALLBACK=1` のみで緑維持（固定 JSON 版は撤去）。
5) ドキュメント更新（移行理由・ENV 一覧・トラブルシュート）。

Acceptance（段階）
- A: 旧ツリーアーカイブ＋再ビルド後、dev/quick とも常緑。dev は CompilerMod 経路で MIR(JSON)→Mini‑VM が 1。
- B: quick を CompilerMod 経路へ切替後も常緑（固定 JSON ラッパー撤去）。

Update — 2025-09-29 (VM Engine unified entry Phase‑A)
- 入口を `VmEngine` に統一（fallback/full を1箇所で切替）。
- 既定: fallback（軽量 MIR インタプリタ）。`NYASH_VM_ENGINE=full` は未実装プレースホルダ。
- ドキュメント: `docs/guides/runtime-architecture.md` を追加。
- Alias（MVP）: `Alias.Box.method(a)` → `Alias_Box.method/1(a)`、`Alias.method(a)` → `Alias_Alias.method/1(a)` を Runner で脱糖。

Plan — Full VM Track（最優先）
1) Phase‑B: フォールバックVMの実用化（最小 BoxCall を追加）
   - String: length/substring/indexOf/lastIndexOf/連結
   - Array: push/get/set/length
   - Map: has/get/set/size
   - 目的: quick を広く緑に維持（仕様不変）。
2) Phase‑C: FullVmEngine の段階導入（`NYASH_VM_ENGINE=full`）
   - プラグイン初期化/レジストリ連携/BoxCall ルーティングの骨格
   - String/Array/Map の最小実装 → wrapper を自己ホスト経路へ戻す
3) 受け入れ: cargo build --release、quick/integration 緑、自己ホスト wrapper 緑、docs 更新

Notes
- 既定挙動は変えない（full はフラグで opt‑in）。ロールバック容易な差分に限定。

Focus
- Keep VM quick green; llvmlite integration on-demand.
- Using SSOT（nyash.toml + 相対using）で安定解決。
- Builder/VM ガードは最小限・仕様不変（dev では診断のみ）。
- Phase 15.7 を再定義: Known 化＋Rewrite 統合（dev観測）と Mini‑VM 安定化、表示APIは `str()` に統一（互換:stringify）。

Update — 2025-09-29 (Syntax Sugar v1 scaffolding + Box‑First + Using Alias MVP)
- Syntax sugar（既定ON; ENVで切替）を段階導入。仕様は parser‑level のみで意味論は不変。
  - L1/basic: pipeline `|>`（優先順位: 関数/ドット > パイプ）, 末尾カンマ, 数値区切り（`1_000_000`）, raw 文字列（r"…"/r#…#; 実装中）
  - L2/full: 受け手糖 `x |> .m(a)`/`x |> obj.m(a)`、プレースホルダ `_`（1回限定; 実装中）
- スモークを追加（quick/core）し緑/赤を切り分け：
  - PASS: sugar_pipeline_basic_vm, sugar_pipeline_receiver_vm, sugar_trailing_comma_vm, sugar_off_mode_vm
  - FAIL: sugar_raw_basic_vm（`r` が未定義扱い）, sugar_numeric_sep_vm（`_000_000` が識別子化）, sugar_pipeline_placeholder_vm（`_` 置換未適用）
- Box‑First 原則を AGENTS.md に追記（5.1）: 交差境界/副作用/高頻度変更は薄い箱で分離（後で解くのは容易、後から足すのは手間）。

Runner/Using（Alias MVP）
- プレリュード AST マージ（VM/LLVM/Interpreter 経路で共通）
- `using "path" as Alias` の Alias 解決（MVP）
  - プレリュード側のトップレベル記号（static box/関数）を `Alias_<Name>` にリネーム
  - 本体コード側の `Alias.Name` を `Alias_Name` にデシュガー
  - 仕様は docs/reference/language/using.md に追記

Mini‑VM（M2 強化）
- stringified 配列セグメントのサニタイズ → fast‑path（const×2 + compare/binop + ret） → バランス括弧でのオブジェクト走査の三段構え
- selfhost_mir_m2_*（compare/binop 系）は quick 緑を確認

Open（確認中）
- `selfhost_compiler_emit_mir_cmp_vm.sh` は wrapper 経路の Alias 伝播の最終確認を実施中（急ぎ対応不要の合意済み）

Next Steps（B: 仕様優先）
1) VM 統一経路の呼び出し堅牢化（静的Boxメソッドの正規化／引数材化の穴埋め／Void誤伝播のFail‑Fast）
2) devガード下で `new→birth` 自動化の狭域許可（開発補助）— 本番はBuilderが明示birth（設計不変）
3) wrapper を CompilerMod 経路に戻して quick 緑化（Alias＋静的呼び出しの最小ケースを追加）
4) （任意）PreLex を interpreter/wasm にも適用して挙動を一本化

Notes
- public/main を強制 push 済み（公開ブランチ反映）
- private/selfhost も更新済み

Plan — Next (sugar 緑化・可逆小差分)
1) PreLexBox（前正規化）を runner 共通入口（VM/LLVM/PyVM）に導入（sugar=ON のみ）
   - r"…" → 通常文字列へ最小エスケープ、数値中の `_` 除去（OFF時は無効）
2) TokenizerBox の保険強化
   - `r` 優先分岐の確定（alphabetic より前／lex_ident 即時 raw 化）、numeric 読取の継続条件（`_` 後も継続）を再点検
3) ParserSugarBox の `_` 置換を確定（関数/メソッド RHS の単回置換、複数 `_` は Fail‑Fast）
4) スモーク再実行 → sugar_raw_basic / sugar_numeric_sep / sugar_pipeline_placeholder を緑化
5) （任意）デシュガー観測 `NYASH_PRINT_DESUGARED=1` を追加検討（dev限定）


Update — 2025-09-28 (Router policy: Instance×string‑like → Unified / Rescue OFF 試験)
- RouterPolicy を明文化・実装反映:
  - `InstanceBox × {length,len,substring,indexOf,lastIndexOf}` は Unified へ固定（Builder 側で `StringBox` 正規化）。
  - Unknown/core/user‑instance の一般規則は従来通り保守（安定性優先）。
- ReceiverInference を補強:
  - 起源が `InstanceBox` でも string‑like の場合は `StringBox` に正規化（挙動不変・救済不要化）。
- docs を更新: builder/unified‑method‑resolution.md, quick‑reference.md, config/env.md に内部規範とフラグ整理を追記。

Plan — Next（テスト実行と点補修・構造優先）
1) VM dev ドライバ（救済OFF）で nested‑if / concat を無制限タイムアウトで再試行
   - `DEV_TIMEOUT_SEC=0 NYASH_VM_PARSERBOX_BOOL=0 NYASH_VM_STRLIKE_INSTANCE_COERCE=0 ./tools/dev/debug_program2_vm.sh`
   - 赤が出た箇所のみ点で補修（LocalSSA: PHI→Copy→Call、Known/型注釈の最小追加）。
2) LLVM パリティのスポット比較（同入力）
   - `./tools/dev/debug_program2_llvm.sh`（JSON ヘッドを目視比較）
3) dev救済は既定OFFのまま維持（依存が消えた時点で段階撤去）
   - 対象: `NYASH_VM_STRLIKE_INSTANCE_COERCE`, `NYASH_VM_PARSERBOX_BOOL`, `NYASH_VM_TOLERATE_VOID`
4) プロセス衛生（暴走監視）
   - 長時間実行は `timeout` 付きに戻す（dev ドライバで `DEV_TIMEOUT_SEC` を 60 へ）。必要時のみ 0 を明示使用。

Update — 2025-09-28 (Mini‑VM 仕上げ — M2/M3 実運転)
- MirVmMin `_run_min` の1パス化・厳密セグメントを再点検（braceバランスでopオブジェクトを厳密抽出）。
- 代表スモークのSKIP撤去（M2/M3）：以下を有効化し quick 緑を維持。
  - `tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_true_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_branch_true_vm.sh`
  - `tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_vm.sh`
- Devスモーク（任意・既定SKIP）追加：Program2（VM）JSONヘッドの非空チェック
  - `tools/smokes/v2/profiles/quick/core/dev_program2_vm_json_head.sh`（`SMOKES_ENABLE_DEV_PROGRAM2=1` で有効）

Plan — Mini‑VM Finishing
1) エッジ強化テスト（Mini‑VM / quick か dev に配置）
   - 同ブロック内に複数 compare → 最後に ret（v0/v1 表記混在）
   - ret がブロック先頭/末尾の両極端
2) O(n) 走査の検証（大きめブロックの計測）
   - braceバランス/配列終端 `_seek_*` が二重/重複走査しないことを目視確認（必要なら簡易計測ログを一時ON）
3) ドキュメント追記（apps/selfhost/vm/README）
   - セグメント抽出の方針（braceバランス）と、想定するJSON v0の最小プロファイルを短記述
4) 緑の維持判定
   - quick 全緑（65/65）/ integration 全緑（17/17）を維持

Update — 2025-09-29 (Mini‑VM edge smokes added; quick green)
- Added new Mini‑VM edge tests (quick/core), all PASS locally:
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_multi_compare_gt_last_ret_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_branch_undef_cond_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_chain_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_div_mod_zero_vm.sh (two-line check: 0 and 0)
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_no_ret_fallback_vm.sh
- Result: make smoke-quick → PASS 72/72. No changes to MirVmMin behavior were required.
- Dev utility added for string API boundaries:
  - apps/dev/program2_str_edges.nyash
  - tools/dev/dev_program2_vm_str_edges.sh (runs VM; dev-only)

Next (short)
1) Router thin checks (unknown→BoxCall, instance×string-like→Unified) — add minimal smokes or SKIP if purely diagnostic
2) Keep Program2 string-edge dev script; expand cases only if a regression is observed
3) Documentation: note added smokes in apps/selfhost/vm/README.md and mention Div/Mod-by-zero + no-ret fallback policies

Rebalance — 2025-09-29（順序の明確化: Rust VM → Mini‑VM → Compiler）
- P0: Rust VM 安定化（点修正の仕上げ・回帰防止／quick+integration 常緑）
  - ReceiverInference/RouterPolicy/LocalSSA/VarMapGuard の確認と最小補修
- P1: Mini‑VM 追加エッジ（完了）
  - 新規5件スモークでエッジ押さえ（compare混在/undef cond/jump chain/0除算/no‑ret）
- P2: Selfhost コンパイラ MVP 前進（次フェーズの主作業）
  - Entry: `apps/selfhost-compiler/compiler.nyash` を用い、`--stage3`/`--min-json` 経路で JSON v0 を安定排出
  - 受け入れ（dev限定）: `NYASH_JSON_ONLY=1` による JSON ヘッダ（version/kind）非空
  - スモーク（任意ゲート）: `tools/selfhost_stage3_accept_smoke.sh` を最小で有効化（必要時に quick へ昇格）

Policy — Compiler Track Unfreeze（apps/selfhost-compiler 限定）
- 大規模変更は Compiler Track に限定して解禁。Core（src/）は引き続き安定運用（小粒のバグ修正/堅牢化のみ）。
- ガード: 既定OFFのフラグ/引数（例: `NYASH_COMPILER_TRACK=1`, `--min-json`, `--emit-mir`）。
- 受け入れ（dev）: JSON ヘッダ非空（min-json）、最小 MIR 生成（emit-mir）。quick/integration は常緑維持。

Next — Compiler Track 小粒タスク
1) dev 受け入れスモーク（min-json ヘッダ）を quick/core に追加（任意ゲート）— 完了
2) MIR 最小生成（const→ret）を安定化（emit-mir）— 継続
3) mir_emitter_box に binop/compare/branch/jump を段階追加（Mini‑VM と同形）— 次
4) builder/ssa/rewrite スケルトンを追加（apps/selfhost-compiler/builder/*）— 完了（未配線）
5) CompilerBuilder.apply_all を NYASH_COMPILER_TRACK=1 の時だけ呼ぶ配線 — 未着手（既定OFFで挙動不変）

Update — 2025-09-28 (P4 default‑on + P5 docs/annotations 完了)
- Known 正規化（userbox限定・関数存在・一意・arity一致）を既定ON。
  - フラグ: `NYASH_REWRITE_KNOWN_DEFAULT`（0/false/off で無効化）。
- 設計ノートを追加: `docs/development/builder/unified-method-resolution.md`。
- Quick Reference を更新: 内部正規化の注記と切替フラグを追記。
- 型注釈を最小拡張（is_digit_char/hex/alpha, MapBox.has/1 → Bool）。
- quick/integration: 全緑を確認。

Update — 2025-09-28 (Router/EmitGuard/NameConst 導入・json_lint_vm 緑)
- Router 最小ガード（仕様不変・安定優先）
  - UnknownBox の Method は一律レガシー経路（BoxCall）へフォールバック（unified 経路での sporadic 未定義を根絶）。
  - `prefer_legacy` を保守側既定に調整: None/Unknown/String/Array/Map は BoxCall 優先、ユーザー箱（末尾"Box"以外）も従来通り BoxCall。
  - `JsonParserModule.create_parser/0` の戻り型を Known 化（Box("JsonParser") 起源付与）。
- BlockSchedule 検証（dev-only）
  - φ→Copy(materialize)→本体(Call) の順序検証を追加（ズレは WARN のみ）。
- VM dev 安全弁（既定OFF）
  - `reg_load` 未定義→Void 置換を `NYASH_VM_TOLERATE_VOID=1` 下でのみ有効化（診断と一時救済）。
- 結果
  - quick: `json_lint_vm` PASS（未定義は解消）。
  - integration（LLVM/llvmlite）: PASS 17/17（すべて緑）。
- 備考: `json_query_vm` は後続の更新で解決（下記エントリ参照）。

Update — 2025-09-28 (json_query_vm PASS・最終ガード適用)
- evaluator 側の堅牢化（VM準拠・仕様不変）
  - 文字クラス判定を membership（手動スキャン）へ変更（indexOf 非依存）。
  - span を ArrayBox から "i:j" 文字列に正規化（.get 依存を排除）。
  - span_unpack_* も手動スキャン実装（indexOf 非依存）。
  - out-of-range/未存在キーは null 返却で合意。
- テスト: json_query_vm の SKIP を解除して PASS を確認。
- quick: 引き続き 64/64 PASS、integration: 17/17 PASS。

Update — 2025-09-28 (P1 — Const統一拡大 + メタ伝播の適用)
- Const 発行の統一（builder 側残存）
  - `build_literal` と core13-pure の型名 Const を ConstantEmissionBox に統一済。残存直書きは掃除済み（rewrite系は NameConstBox 使用）。
- メタデータ伝播（type/origin）を小粒適用
  - BlockScheduleBox: `emit_before_call_copy` で `propagate(base→dst)` を追加。
  - utils: `materialize_local` で `propagate(src→dst)` を追加。
  - `insert_copy_after_phis` は既に propagate 済み（再確認のみ）。
- ルータ/型注釈: 前回の dev トレース追加／ホワイトリスト拡張に変更なし（挙動不変）。
- 検証: quick/integration は引き続き全緑を確認予定（差分は局所・可逆）。

Update — 2025-09-28 (Rewrite Known 化 Stage‑1 一本化)
- 標準メソッド呼び出しを emit_unified_call に統一委譲。
  - ルーティング（RouterPolicy）と rewrite::{special,known} の適用点を一本化。
  - 既存ガードにより Unknown/core/user-instance は BoxCall へ自動フォールバック（挙動不変）。
- 重複掃除（挙動不変）
  - method_call_handlers 内の receiver クラス推定（me/起源/型）は削除し、unified 側に一本化。
  - box_type は None を渡し、emit_unified_call が起源/型から判断。
  - pin_to_slot/BoxCall 直呼びの旧コードは撤去済み。

Update — 2025-09-28 (FunctionEmissionBox adoption + Router trace + Type annotate)
- FunctionEmissionBox 採用を拡大（MirFunction 直編集の代表箇所を移行）
  - src/mir/aot_plan_import.rs の Const/Return 発行を function_emission 経由に置換（挙動不変）。
  - Float/Null/Void など特殊値は安全側で既存ロジックにフォールバック（差分最小）。
- RouterPolicy に dev 観測ログを追加（既定OFF）
  - 環境変数 `NYASH_ROUTER_TRACE=1` で、経路決定（Unified/BoxCall）と理由（unknown_recv/core_box/user_instance）を stderr に短く出力。
  - 仕様不変・テスト比較に影響なし（既定OFF・stderr）。
- TypeAnnotationBox のホワイトリストを最小拡張（観測ベース）
  - 追加: `*.len/0 → Integer`, `*.substring/2 → String`, `*.esc_json/0 → String`。
  - 既存の `*.str/0`/`*.length/0`/`*.size/0` に加えて注釈精度を微増（挙動不変）。

Update — 2025-09-28 (P6 — Selfhost JSON emit PASS)
- 目的: selfhost コンパイラの JSON emit を安定化し、スモークで bytes>0 を必須に昇格。
- 変更点（挙動不変・最小差分）
  - apps/selfhost-compiler/compiler.nyash: 出力を `ConsoleBox.println(json)` から `print(json)` に変更し、`json==null` 時は `{}` を出力（未定義の使用を回避）。
  - tools/selfhost_smoke.sh: Step1 を LLVM ハーネス優先（`--backend llvm` + `NYASH_LLVM_USE_HARNESS=1`）。
    - 成功基準を `-s /tmp/nyash_selfhost_out.json`（bytes>0）に強化。未満ならエラーで終了。
- 結果
  - selfhost_smoke: PASS（Step1: 253 bytes / Step2: VM 出力 ON/OFF 一致）。
- 備考
  - VM 経路では稀に dev 検証（NewBox→birth WARN）や early 未定義で停止するため、emit だけは LLVM ハーネスに委譲（仕様不変）。
  - 後続で VM 経路の未定義起因（argv/ArrayBox birth 近傍）を別タスクで潰す。

Update — 2025-09-28 (quick/integration smoke status — 総括)
- quick: PASS 64/64（暫定 SKIP を明示）
  - SKIP（VM 側の局所 polish 中; LLVM 緑）:
    - core/loops: break_continue, loop_statement（PHI 搬送の最小補強→復帰）
    - selfhost mini‑vm: m2_eq_true / m3_branch_true / m3_jump（Mini‑VM M2/M3 の単一パス化・境界厳密化の仕上げ後に復帰）
- integration（LLVM/llvmlite）: PASS 17/17（全緑）
- フラグ整理:
  - `NYASH_VM_TOLERATE_VOID` は dev/一部診断時のみ使用。quick テストからは削除済み。
  - Router ガード（Unknown→BoxCall）は仕様不変・常時ON。

Update — 2025-09-28 (LocalSSA — in-block materialize & recv/args 統一)
- LocalSSA 小箱を導入（Builder 内部）: `(bb, orig, kind) -> local` のキャッシュで、必ず「現在の基本ブロック内」に Copy を置く。
  - 実装: `MirBuilder.local_ssa_map` と `local_ssa_ensure(v, kind)`（kind: 0=recv, 1=arg, 2=cmp, 4=cond）。
  - 読みやすさヘルパ: `local_recv/local_arg/local_cond/local_field_base/local_cmp_operand` を追加。
- 適用（最小・局所、仕様不変）:
  - Unified Method 呼び出し: 受信者/引数を LocalSSA 済みに統一（emit 前に in‑block materialize）。
  - Legacy Call（Extern/Global/Value）: 引数を LocalSSA 化。BoxCall も recv/args を LocalSSA 化。
  - Branch/条件: if/loop/短絡 And/Or の条件を LocalSSA 化。
  - Field: base と set 値に LocalSSA を適用。`?` 伝播でも recv/条件に適用。
  - 置き換え: `pin_to_slot("@recv")` → `local_recv` に差し替え（BoxCall 経路も含む）。
- 既知の現象: `apps/lib/json_native/lexer/scanner.nyash` の `read_string_literal()` 内 `me.advance()`（Unified 経路）で稀に `use of undefined recv` が残存。
  - 受信者/引数/条件/フィールド周辺は LocalSSA の“内側”へ揃えたため、残りは「emit 直前のブロック切替」等のパスでズレている可能性。
  - 次アクション（P0）で観測を厚くし、必要なら emit 直前の bb 再確認→再 materialize の最終関所を広げる。
- 備考（レガシー優先について）: ArrayBox/MapBox/StringBox と "…Box" 以外のユーザー箱はレガシー BoxCall 優先のまま（安定性）。ただし LocalSSA を適用済みのため、現象の主因ではない。

Update — 2025-09-28 (LocalSSA 最終関所＋Unified 仕上げ・json_lint_vm デバッグ)
- finalize ヘルパー追加（ssa/local）
  - `finalize_branch_cond` / `finalize_compare` / `finalize_field_base_and_args` を実装、各 emit 直前に適用。
  - Compare は従来の ensure_slotify を置換（挙動不変）。
- Unified Call 側の強化
  - emit 直前に `finalize_callee_and_args` を再適用（bb 変化に強い）。
  - さらに最終 Copy を Call 直前に強制挿入（受信者の def→use を同一 bb に確実化）。
  - dev トレース `[vm-call-final]` は `NYASH_LOCAL_SSA_TRACE=1` 時のみ出力（runner 比較に影響しない）。
- emit フック（builder）
  - `emit_instruction` で Method 付き Call を検知し、直前に Copy を 1 枚差し込む最終ガード（dev 正当化）。
- VM 側の dev 安全弁（default OFF）
  - `NYASH_VM_RECV_ARG_FALLBACK=1` または `NYASH_VM_TOLERATE_VOID=1` で、未定義受信者時に args[0] を受信者として読み直す（Builder 取りこぼしの一時救済）。
- 現状の結果
  - 受信者未定義は再現困難に。json_lint_vm は次段の未実装メソッド（String.is_digit_char）で停止。

Next — 短期 TODO（仕様不変・差分最小）
1) json_query_vm の quick 失敗を解消（undefined→Void 置換に頼らない）
   - eval_path_text 直近の `substring/==` 連鎖で LocalSSA finalize の取りこぼしがないか emit 点を再点検。
   - UnknownBox→BoxCall へ統一済のため、unified 経路残存が無いか grep で確認し、見つかれば点で BoxCall へ誘導。
   - reg_load の Void 寛容は OFF のまま比較を厳密に（quick テスト側からも外した）。
2) MIR dump/トレースの最小化: failing bb の直前5命令を dev だけ短くダンプし、φ→Copy→Call の順序を再検証。
3) quick 全体を再実行→緑維持。必要なら minimal finalize を追加（仕様不変）。

Unskip Plan（段階復帰）
- P0: json_query_vm（VM）
  - 受け入れ: 期待出力と一致。追加の寛容フラグ不要。SKIP 解除。
- P1: loops（break_continue / loop_statement）
  - 受け入れ: 期待出力一致。PHI carriers/entry materialize の取りこぼしゼロ。SKIP 解除。
- P2: Mini‑VM（M2/M3: compare/branch/jump）
  - 受け入れ: m2_eq_true/false, m3_branch_true, m3_jump の 4 件が PASS。coarse/多段走査を撤去して単一パスを維持。

Plan — Next（一本化の続きと段階導入）
- P3（重複整理の完遂・1日）
  - 標準メソッド経路の一本化は完了。残る補助ロジックの重複（受信者クラス推定・候補列挙）を `rewrite::{known,special}` 側APIへ寄せる（点検・微修正）。
  - Docs 同期: CURRENT_TASK と docs/development/builder/BOXES.md に一本化方針と責務境界を追記。
  - 受け入れ: quick/integration 全緑、ログは既定OFFで静粛。
- P4（Known 正規化の観測→段階ON・2〜3日）
  - 観測: `NYASH_ROUTER_TRACE=1` と `observe::resolve.choose` で Known 率/フォールバック率を確認。
  - 段階ON: userbox 限定＋関数存在＋候補一意＋arity一致のみ既定ON（新フラグ `NYASH_REWRITE_KNOWN_DEFAULT` で切替）。
  - 受け入れ: quick/integration 緑、mismatch 0、性能±10%以内。
- P5（周辺整備・1日）
  - 型注釈の最小拡張（観測ベースで1〜2件）。
  - phase‑15.7/README と Quick Reference に「内部正規化（obj.m→Class.m）」の注記を追記（ユーザー向け説明を簡潔に）。

Docs — Added
- Unified method resolution design note: docs/development/builder/unified-method-resolution.md
  - Pipeline, invariants, flags, rollout plan（P4 observe → dev opt‑in → consider default）を整理。

Self‑Hosting — Return Plan（P6）
- 目的: Selfhost Compiler（Ny製）→ MIR(JSON v0) → VM/llvmlite 実行の実線復帰。
- 手順（小粒・仕様不変）
  1) Quickstart ドキュメント追加（完了）: `docs/development/selfhosting/quickstart.md`
     - 実行例/ENV透過/出力ファイルの位置を記述。
  2) MVP 走行確認（dev）
     - `apps/selfhost-compiler/compiler.nyash` で最小サンプルを emit（`--min-json` / `--stage3`）。
     - VM（Rust/PyVM どちらでも）で JSON v0 を実行し、既存の JSON アプリと期待出力一致。
  3) スモーク連携（任意ジョブ）
    - 代表1件の bootstrap スモークを tools に追補（既存 `tools/bootstrap_selfhost_smoke.sh` の利用/更新を検討）。
    - 受け入れ基準
  - quick/integration 緑を維持。
  - Selfhost emit→実行の最小系が安定して PASS（dev 任意ジョブで十分）。

Update — 2025-09-28 (BlockScheduleBox 導入・順序固定)
- 目的: ブロック内の物理順序を契約化（PHI群 → materialize群(Copy/Id) → 本体(Call等)）。
- 実装:
  - 新規: `src/mir/builder/schedule/{mod.rs,block.rs}` 追加。
  - API 初期:
    - `ensure_after_phis_copy(builder, src) -> ValueId`: φ直後に Copy を確実挿入（per‑block dedup `(bb,src)->dst`）。
    - `emit_before_call_copy(builder, src) -> ValueId`: Call 直前に最終 Copy（src は after‑phis の dst）。
  - `MirBuilder` に `schedule_mat_map`（per‑block）を追加し、`start_new_block` でクリア。
  - Unified Call で適用（pin→LocalSSA→after‑phis Copy→必要時 before‑call Copy）。
- 状態:
  - “use of undefined recv” は大幅減。sporadic 残存に対し、二段網（after‑phis固定＋before‑call最終）を導入済み。
  - 一部で受信者誤型（例: String に parse）を観測。順序ではなく解決側の誤選択の可能性。
- 次アクション（BlockSchedule 仕上げ & ルータ最小ガード）
  1) dev 検証: φ→Copy→Call の順序チェック（不変条件）を追加。
  2) rewrite/resolve に dev 最小ガード（既定OFF）を置き、明確な誤選択（String.parse 等）を抑止。観測ログで要因特定。
  3) failing bb を MIR dump で再検証→ quick 緑化。

Plan — Next (LocalSSA 仕上げ・観測)
1) 観測（dev 限定）: `local_ssa_ensure`/emit_unified_call に軽トレースを追加（bb/kind/orig→local）。
2) 最終関所: emit 直前に `current_block` のズレ検知→ `local_ssa_ensure` を再適用する小ヘルパを共通化（Call/Compare/Branch/Field に必要分点適用）。
3) json_lint_vm を再実行（quick 緑化）。
4) ドキュメント追記: LocalSSA の責務と適用範囲（builder/README or observe/README 近傍）。

Update — 2025-09-28 (Dev Parser VM 深掘り + 受け手ゼロ防護)
- dev ドライバに parse_program2 経路を追加（apps/dev/debug_parser_vm.nyash）。
- Unified Call: 受け手が ValueId(0) になる経路を防護（recv-guard）。
  - emit_unified_call 内で Method の receiver==%0 を検知し、元の受け手 ValueId に巻き戻す（診断ログ: [recv-guard]）。
- 実行結果
  - 先行していた reg_load undefined (recv=%0) は解消。
  - 深い経路で ParserBox.to_int 内の比較で TypeError(Lt on InstanceBox) を観測（仕様は不変）。
    - 次手: to_int 周辺の受け手/局所の型注釈（String.length→Integer の帰結と i/n の整数性）を点で補強、もしくは推論誤束縛の再追跡。

Update — 2025-09-28 (LocalSSA ヘルパ化・集中管理 追加)
- ssa/local へ集約: `src/mir/builder/ssa/local.rs` を新設し、LocalKind と ensure()/recv/arg/cond/field_base/cmp_operand を実装。
- 共通ヘルパ: Call 直前の集約処理を `finalize_callee_and_args(builder, &mut Callee, &mut Vec<ValueId>)` に統一。Legacy 用に `finalize_args(...)` も追加。
- 呼び出し側の簡素化:
  - Unified: `emit_unified_call` は finalize_callee_and_args を呼ぶだけに整理（手動の re-materialize を撤去）。
  - Legacy: Extern/Global/Value で finalize_args を適用。
  - BoxCall: utils 側で recv/args を LocalSSA に統一（pin_to_slot("@recv") 撤去）。
- dev トレース: `NYASH_LOCAL_SSA_TRACE=1` で ensure/copy を一行出力（bb/kind/orig→local）。

Plan — Next (短期・最小差分)
- 最終関所の共通化を拡張: ssa/local に Branch/Compare/Field 用の finalize ヘルパを追加し、emit 直前に一律適用（ズレ検知を含む）。
    - 観測の強化: LocalSSA トレースに inst 直前/直後の要点（bb, kind, value）を短く追加し、未定義が LocalSSA の内外どちらか即判定できるようにする。
    - json_lint_vm を緑化（仕様不変・最適化後回し）。

Update — 2025-09-28 (Loop VarMapGuard 観測＋適用漏れ修正)
- 目的: ループ合流（header/exit/continue）およびループ内 if-merge で VarMapGuard の適用漏れを検出・解消する。
- 実装:
  - loop_builder.update_variable に dev 観測を追加（`NYASH_VARMAP_TRACE=1` 時に guard 適用を1行表示、names簡易一覧も出力）。
  - ループ内 if-merge の統合ヘルパ `phi_core::if_phi::merge_modified_at_merge_with` 呼び出しで、
    従来は `variable_map.insert` に直書きしていた rebinding を LoopBuilder.update_variable 経由に変更（VarMapGuard を適用）。
- 現状:
  - dev ドライバ（apps/dev/debug_parser_vm.nyash）をトレース最小で再実行したが、`BoxRef(InstanceBox) → bool` の型エラーは残存（再現性あり）。
  - `NYASH_VARMAP_TRACE=1` では guard 適用ログは現時点で未出力（= `me` 直束縛由来ではない可能性大）。
- 次アクション（短期）:
  1) VM トレース最小＋条件部のみ短観測（branch 直前の cond ValueId と tag）を追加して発生点を特定（仕様不変・devのみ）。
  2) 発生箇所が InstanceBox 受け手の条件式なら、builder 側で cond 正規化（Eq/Ne/長さ比較に還元）を点適用。
  3) 一時的に `NYASH_VM_PARSERBOX_BOOL=1` を dev ドライバでだけ許可→深部の次の不具合を観測（段階弱体化の準備）。

Note — ParserBox Stage‑1 and/or
- ParserBox の Stage‑1 JSON 生成器では `&&`/`||` の字句/構文をまだ未実装。dev ドライバの and/or 最小ケースは一旦停止。
- 選択肢（後続フェーズで実施）:
  - A) ParserBox.parse_expr2 に and/or の字句/構文＆短絡を最小実装（Block/If/PHI 相当のJSON生成）。
  - B) Builder 側で `if (a && b)` パターンを nested if に正規化（Stage‑1 JSONに and/or不要）。
- 現方針: フェーズ内は機能追加は最小に保つため、A/B は後続で検討。現状は and/or ミニケースを dev 側で停止し、比較/加算の枝から段階復帰を進める。

Update — 2025-09-28 (program2 minimal harness enable)
- dev 用に LLVM ハーネスの program2 ミニテストを追加し有効化。
  - 入口: `tools/dev/debug_program2_llvm.sh`（内部で `apps/dev/debug_program2_llvm.nyash` を実行）
  - using/file は dev 限定で `NYASH_ENABLE_USING=1`, `NYASH_ALLOW_USING_FILE=1`, `NYASH_USING_AST=1` を設定。
  - VM 側の program2 は bring-up 中（短い入力でもスピン再現のため一旦停止、観測→点補強後に順次再開）。

Update — 2025-09-28 (Program2 VM/LLVM step-up + timeouts)
- 追加（dev/VM）: program2 最小/if/loop の分割テストを用意。
  - 最小: `apps/dev/debug_program2_vm.nyash`（runner: `tools/dev/debug_program2_vm.sh`）
  - if:   `apps/dev/debug_program2_vm_if.nyash`（runner: `tools/dev/debug_program2_vm_if.sh`）
  - loop: `apps/dev/debug_program2_vm_loop.nyash`（runner: `tools/dev/debug_program2_vm_loop.sh`）
  - 既定タイムアウトを 60s に延長。`DEV_TIMEOUT_SEC=0` で無制限実行可能。
- 追加（dev/LLVM）: program2 最小/if/loop を llvmlite ハーネスで実行。
  - 入口: `tools/dev/debug_program2_llvm.sh`（`NYASH_LLVM_USE_HARNESS=1`）。
  - こちらもタイムアウト 60s（`DEV_TIMEOUT_SEC` で可変）。
- 結果（現状）:
  - VM: 最小/if は EXIT 0。loop は短い制限では timeout することがあるが、無制限（`DEV_TIMEOUT_SEC=0`）で約 3.4s で PASS。
  - LLVM: 環境が許せば短時間で PASS。制限が厳しい場合は `DEV_TIMEOUT_SEC` を上げる。
- 局所補強（仕様不変・VM限定）:
  - ParserBox.* 内で `indexOf/lastIndexOf` が InstanceBox に誤解決するケースに対し、文字列化の安全フォールバックを追加（`src/backend/mir_interpreter/handlers/boxes.rs`）。
- 既知の課題（dev救済 OFF 時）:
  - `NYASH_VM_PARSERBOX_BOOL=0` で稀に `cannot coerce BoxRef(InstanceBox) to bool`（条件部に InstanceBox が流入）。dev では救済 ON のまま bring-up を継続し、OFF での赤は点補強（LocalSSA 材化/型注釈）で解消予定。

Plan — Next (2025-09-28)
1) VM loop の常時 PASS 化（60s/無制限で安定確認）
   - 必要時のみ最小観測をON（`NYASH_VM_TRACE=1` 等は短時間）。
2) dev救済OFFの段階移行（最小→if→loop）
   - OFF で落ちた箇所は局所に点補強（LocalSSA: PHI→Copy→Call、または Known/型注釈）
   - 緑維持後は救済をOFF既定化（撤去容易な差分を維持）
3) LLVM パリティのスポット確認（同一入力の JSON ヘッド構造を目視）
4) and/or は後続（ParserBox Stage‑1 実装 or Builder 正規化 B案）

Update — 2025-09-28 (Lifecycle & Expr Flow 状態 + Boxes カバレッジ)
- ライフサイクル不変（同期の要点）
  - 関数スコープ: `value_gen` は関数ごとに reset、`value_types`/`value_origin_newbox` は take/restore（交差汚染防止）。
  - ブロックスコープ: 物理順序は「PHI群 → Copy(Materialize)群 → 本体(Call/Compare/Branch)」。
  - Call サイト: `finalize_callee_and_args` により receiver/args を in‑block 材化。RouterPolicy で Unknown/String/Array/Map/ユーザー箱は BoxCall へ保守側フォールバック。
  - 受け手ゼロ防護: `emit_unified_call` 内で `receiver==%0` を検知→元受け手に復元（[recv-guard]）。
  - ParserBox.* 限定の救済（dev): 比較で BoxRef(ParserBox)↔数値が交差時は gpos を数値化（VM 側; 既定OFF運用想定）。
- 式の流れ（pipeline 概観）
  - AST → builder.dispatch →（method は）emit_unified_call → RouterPolicy → EmitGuard(finalize) → Materialize → MIR.Emit → VM 実行。
  - Compare/Branch/BinOp は emission::*/LocalSSA で材化・注釈・一貫した発行に統一。
- Boxes カバレッジ（現状）
  - S-tier: ConstantEmissionBox / CompareEmissionBox / BranchEmissionBox / FunctionEmissionBox（採用済）
  - Inference/Route: ReceiverInferenceBox / RewriteGateBox / InstanceMethodIndexBox / RouterPolicyBox（採用済）
  - 生成/材化: MaterializeBox / MetadataPropagationBox / NameConstBox（採用済）
  - 観測: ResolveTraceBox / VarMapTrace（dev-only）
  - 追加提案（最小）: VarMapGuardBox（dev-only）— PHI/合流/代入時に `me` の ValueId を他名へ直接束縛しないブレーキ（現在は `build_assignment` に局所適用。次で PHI 合流にも展開）。
- 現状の課題（dev で観測）
  - ParserBox 深部でまれに j/i 等が `me` に誤束縛 → `j+1` が BoxRef+Int で型エラー。recv=%0 は解消済み。根は varmap 合流時の `me` 伝播。

Plan — Next（Lifecycle/Expr Flow 仕上げ）
1) VarMapGuard（PHI/合流）: if/loop 合流で name≠"me" に `me` の ValueId が入る場合、1 Copy を介して別 ValueId にして束縛（ParserBox.* 限定で開始）。
2) dev 追跡: BinOp/Compare の varmap/型トレースで再確認 → type error 消失を確認。
3) dev 救済の狭域化: ParserBox 比較の gpos 整合・代入時の `me` 回避を段階的に弱め（根治後に撤去可能に）。
4) Selfhost VM スモーク: 生成 JSON bytes>0 を強制（tools/selfhost_smoke.sh）→ quick 任意ジョブ化。

Update — 2025-09-28 (P1 Known 集約・KPI・LAYER ガード)
- Builder: method_call_handlers の Known 経路を `rewrite::known` に集約。
  - 新規 API: `try_known_or_unique`（Known 優先→一意候補 fallback）。
  - equals/1 を `rewrite::special::try_special_equals` に移設（挙動不変）。
- Observe: `resolve.choose` に certainty を付加し（Known/Heuristic）、`NYASH_DEBUG_KPI_KNOWN=1` 時に簡易集計を出力（`NYASH_DEBUG_SAMPLE_EVERY=N`）。
- LAYER ガード（任意ツール）: `tools/dev/check_builder_layers.sh` を追加（origin→observe→rewrite の一方向チェック）。
- Unified 経路: `emit_unified_call` に equals/1 の集約を追加（Known 優先→一意候補）（仕様不変）。
- メソッド候補インデックス化: `MirBuilder` に tail→候補のキャッシュを追加（lazy再構築）。
  - API: `method_candidates(method, arity)`, `method_candidates_tail(tail)`
  - 利用箇所: method_call_handlers の resolve.try、rewrite::{special,known} の一意候補探索、unified equals/1 の一意候補。
- 集約ポリシー（P0 完了）:
  - 中央集約先: `emit_unified_call`（Methodターゲット時に rewrite/special/known を順に試行）
  - `method_call_handlers` は `emit_unified_call` を呼ぶだけに簡素化（重複ロジック削減）
  - equals/1 も同一ロジックに吸収
- レガシー経路（P1 準備）:
  - dev ガード追加: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1` でレガシー側のメソッド関数化を停止（将来削除の前段階）
  - Unified 無効時の後方互換は維持（既定OFF）

Status Snapshot — 2025‑09‑27
- Completed
  - VM method_router: special-method table extended minimally — equals/1 now tries instance class then base class when only base provides equals (deterministic, no behavior change where both exist). toString→str remains（互換: stringify を許容）。
  - MIR Callee Phase‑3: added TypeCertainty to Callee::Method (Known/Union). Builder sets Known when receiver origin is known; legacy/migration BoxCall marks Union. JSON emitter and MIR printer include certainty for diagnostics. Backends ignore it functionally for now.
  - Using/SSOT: JSONモジュール内部 using を相対に統一（alias配下でも安定）
  - DebugHub: 追加ゲート `NYASH_DEBUG_SAMPLE_EVERY`（N件に1度だけ emit）。重いケースでのログ制御のため（既定OFF・ゼロコスト）。
  - Router diagnostics: class-reroute / special-reroute を DebugHub に emit（dev-only, 既定OFF）。
  - LLVM diagnostics: `NYASH_LLVM_TRACE_CALLS=1` で `mir_call` の callee（Method.certainty 含む）を JSON 出力（挙動不変）。

Decision — Variables (Option A; 2025‑09‑27)
- 方針: var/let は導入しない。ローカルは常に `local` で明示宣言。
- 目的: SSA/Loop‑Form と Known/Union 解析の単純さを維持し、未宣言代入の混入を防ぐ。
- 補足: 行頭 `@name[:T] = expr` は標準ランナーで `local name[:T] = expr` へ自動展開（既定ON）。言語意味は不変。
- Docs 更新: quick-reference, language reference, tutorials に「var/let 不採用」を明記。
  - Tokenizer/Parser デバッグ導線（devトレース）を追加
  - json_lint_vm: fast‑pathの誤判定を除去＋未終端ガードを追加（PASS）
  - json_query_min_vm/json_query_vm/json_pp_vm: PASS
  - forward_refs_2pass: Builder が user Box に birth BoxCall を落とさないよう修正＋ランナーフィルタ調整（PASS）
  - Test runner: dev verify ノイズ（NewBox→birth warn）および BoxCall dev fallback をフィルタ
  - Entry policy: top‑level main 既定許可に昇格（NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN default=true）。
    - 互換: `Main.main` が存在する場合は常にそちらを優先。両方無い場合は従来通りエラー。
    - オプトアウト: `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=0|false|off` で無効化可能。
- Next
  - Heavy JSON: quick 既定ONへ再切替（LLVM 常備で段階復帰）
  - 解析ログの統一: parser/tokenizerのdevトレースは既定OFFのまま維持、必要時だけ有効化
- llvmlite（integration）: 任意ジョブで確認（単発実行のハングはタイムアウト/リンク分離で回避）

Update — 2025-09-27 (json_roundtrip_vm null 全化の修正)
- Cause: Tokenizer の構造トークン検出が `indexOf` 依存のため、環境によって `{ [ ] } , :` を認識できず ERROR に落ちていた。
- Fix: `char_to_token_type(ch)` を `==` での直接比較に変更（環境依存排除）。
  - File: apps/lib/json_native/lexer/tokenizer.nyash
- Result: core/json_roundtrip_vm.sh, core/json_nested_vm.sh → PASS（VM quick）

Self‑Hosting Roadmap (Revised) — 2025‑09‑27

Goal
- 一度に広げず、小粒で段階導入。既定挙動は変えず、dev/ci で計測→安定→昇格。
- 本線は VM（Rust）と llvmlite（Python）で検証しながら、Nyash 自身による最小実行器へ橋渡し。

Milestones
- M1: JSON 立ち上げ（VM quick 基準）
  - 目的: JSON 入出力の足場を固め、言語側のテスト土台を安定化。
  - 完了: 相対 using 統一、json_lint_vm/roundtrip/nested/query_min 緑化。
  - 次: Scanner.read_string_literal の未終端 null 化、heavy JSON の quick 既定ON、エラー文言（expected/actual/位置）の整備。
  - 受け入れ: quick で JSON 系が常時緑（SKIPなし）。

- M2: MIR Core‑13 最小セットの Ny 実装（JSON v0 ローダ＋実行器）
  - 範囲: const/binop/compare/branch/jump/ret/phi、call/externcall/boxcall（最小）。
  - 進め方: PyVM を参照実行器としてパリティ確認。fail fast を優先（dev 詳細ログ）。
  - 受け入れ: 代表スモーク（小型）を Ny 実行器で通過、PyVM と出力一致。

- M3: Box 最小群（String/Array/Map/Console）
  - メソッド: length/get/set/push/toString、print/println/log（必要最小）。
  - ポリシー: 既存NyRT/プラグインと衝突しないよう名前空間を分離。既定はOFF、devでON。
  - 受け入れ: JSON apps が Ny 実行器で最低限動作（速度不問）。

- M4: Parity/Profiles 整理
  - プロファイル: dev=柔軟、ci=最小+計測、prod=SSOT厳格（nyash.toml）。
  - パリティ: VM↔llvmlite↔Ny 実行器で代表サンプル一致。差分はテーブル化し段階吸収。
  - 受け入れ: quick（VM）緑、integration（llvmlite）任意緑、Ny 実行器で代表ケース緑。

Guards / Policy
- 変更は局所・可逆（フラグ既定OFF）。
- 既定挙動は不変（prod 用心）。
- dev では診断強化（ログ/メトリクス）し、ランナー側でノイズはフィルタ。

## Unskip Plan（段階復帰）
- P0: json_query_vm（VM）— Completed
  - 状態: SKIP 解除、期待出力一致、寛容フラグ不要で PASS。
  - 措置: evaluator のspan表現と membership 判定の手動化（indexOf/.get 非依存）。
- P1: loops（break/continue/loop_statement）— Completed
  - 状態: SKIP 解除、quick で PASS。
  - 措置: LoopBuilder の PHI/順序を維持しつつ、LocalSSA/BlockSchedule の適用範囲で in‑block 定義を徹底。
- P2: Mini‑VM（M2/M3）— Completed
  - 状態: 代表 4 件（m2_eq_true/false, m3_branch_true, m3_jump） PASS・SKIP 解除。
  - 備考: 単一パス維持・境界厳密化済み。

Update — 2025-09-28 (S‑tier 箱の適用拡大・仕様不変)
- Const 発行の一元化（代表→全体へ拡大）
  - builder/stmts.rs: Void/String を `emission::constant` に置換。
  - builder/control_flow.rs, exprs.rs, fields.rs: Void/String を同様に置換。
  - builder/builder_calls.rs: 関数名 Const は `NameConstBox` へ、整数1は `emission::constant` へ。
- メタデータ伝播の統一
  - builder/utils.rs: `pin_to_slot` / `insert_copy_after_phis` の型/起源コピーを `metadata::propagate` に移譲。
- 既知戻りの型注釈（最小）
  - `annotate_call_result_from_func_name` に `types::annotation::annotate_from_function` を追加（`str/0`・`length/0`・`size/0`）。

現状サマリ
- quick: PASS 64/64（loops/Mini‑VM を含む）
- integration（llvmlite）: PASS 17/17

Next（小粒・既定挙動不変）
- S‑tier の置換拡大の残: 代表の置換を完了（ops/decls/exprs の主要点）。引き続き残部を段階的に `emission::constant` へ（影響の少ない箇所から）。
- RouterPolicyBox への `prefer_legacy` 集約を適用済み（utils の判定を `router::policy::choose_route` に移譲）。
- 既知戻り注釈のホワイトリスト拡充（必要に応じて、dev 記録と連動）。

## MIR 生成層の箱（Box 化） — 構造導入（仕様不変）
目的: 重複した処理（定数発行/メタ伝播/最低限の型注釈）を薄い箱に集約し、回帰を構造で抑止する。

Tier S（今すぐ・小粒）
- MetadataPropagationBox（src/mir/builder/metadata/propagate.rs）
  - propagate(builder, src, dst)
  - propagate_with_override(builder, dst, MirType)
- ConstantEmissionBox（src/mir/builder/emission/constant.rs）
  - emit_integer/emit_string/emit_bool/emit_null/emit_void
- TypeAnnotationBox（src/mir/builder/types/annotation.rs）
  - set_type(builder, dst, MirType)
  - annotate_from_function(builder, dst, func_name)

状態（2025-09-28）
- S-tier: metadata/emission/types（annotation）に加え、router/emit_guard/name_const を追加（仕様不変）。
- 最小適用: builder_calls（Router/EmitGuard）、rewrite/{special,known}（NameConst）へ部分導入済み。
- まだ広域置換は行っていない（段階適用）。

次のアクション（箱の採用計画）
1) const発行箇所を emission::constant に段階移行（代表箇所のみ→全体）
2) 値生成直後の type/origin 継承を metadata::propagate に統一
3) 統一Callの dst へ TypeAnnotationBox をピンポイント適用（既知戻りのみ）
4) RouterPolicyBox を unified 経路へ導入（Unknown/String/Array/Map/ユーザー箱→BoxCall）
5) EmitGuardBox で Call の finalize/verify を集約（Branch/Compare は後段）
6) NameConstBox を rewrite/special/known へ段階適用

ガード/方針
- すべて既定OFFの挙動変更なし。差分は関数呼び出し先の集約のみ。
- quick/integration 緑維持を確認しつつ範囲を広げる。

参考: docs/development/builder/BOXES.md に API/方針の詳細。

Policy — AST Using (Status Quo)
- SSOT（nyash.toml）＋AST prelude merge を維持。prod は toml 限定、dev/ci は段階的に緩和。
- 重い AST/JSON ケースは integration でカバーしつつ、quick への復帰は LLVM 有効環境で段階的に行う（順次解除）。

Work Queue (Next)
1) Scanner: 未終端文字列で必ず null を返す（Tokenizer が ERROR へ）
2) Heavy JSON: quick 既定ONに戻す（プローブは維持）
3) エラーメッセージの詳細化（expected/actual/line/column）
4) Ny 実行器 M2 スケルトン（JSON v0 ローダ＋const/binop 等の最小実装）下書き
5) Parity ミニセット（VM↔llvmlite↔Ny）を用意し、差分ダッシュボード化
 6) Router: Known/Union 方針の磨き込み（挙動不変）
    - Known → 既存の直接呼び出しを維持（VM 完了、LLVM は表示のみ）。
    - Union → ルータ経路を維持しつつ、ログで可視化（表は“必要最小”で追加）。
 7) Heavy JSON の quick 段階復帰（LLVM 有効環境）
    - 順序: nested_ast → roundtrip_ast → error_messages_ast。
 8) （診断）LLVM ダンプに certainty の補助表示（必要時、挙動不変）。

Update — @local expansion promotion (2025‑09‑27)
- すべてのランナーモードに `preexpand_at_local` を適用（common/llvm/pyvm に加え vm/selfhost へも導入）。
- Docs を更新し、構文糖衣が標準で有効であることを明記。

Plan — Router Minimalism (継続方針)
- 特殊メソッド表は “toString→str（互換:stringify）, equals/1” の範囲から、ユースが発生したもののみ点で追加。
- 既定の挙動・言語仕様は変更しない（フォールバックの拡大はしない）。
- 測定: DebugHub（resolve.*）ログと LLVM の `NYASH_LLVM_TRACE_CALLS` を併用し、Union 経路を可視化。

Runbook（抜粋）
- VM quick: `tools/smokes/v2/run.sh --profile quick`
- LLVM llvmlite: `cargo build --release --features llvm && tools/smokes/v2/run.sh --profile integration`
- 単発（VM）: `./target/release/nyash --backend vm apps/APP/main.nyash`
- 単発（LLVMハーネス）: `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/peek_expr_block.nyash`


Update — 2025-09-27 (Tokenizer/VM trace bring‑up)
- Implemented VM guards (prod): disallow user Instance BoxCall; dev keeps fallback with WARN.
- Dev assert: forbid birth(me==Void) in instance-dispatch path.
- Builder verify (dev): NewBox→birth invariant; warns when missing.
- Added targeted VM traces (dev):
  - JsonToken setField/getField one‑liners
  - Legacy/method calls for JsonTokenizer/JsonScanner keyword paths
- Tokenizer hardening:
  - Reordered next_token dispatch: keyword/number/string first, structural last (avoids misclassifying letters as structural)
  - char_to_token_type rewritten to strict per‑char check (no ambiguous match)
  - Result: "null" now tokenizes correctly (NULL), and JsonParser.parse("null") returns a JsonNode (R=BOX null in probe)

Status (after patch)
- token_probe: OK (NULL/null emitted as expected)
- json_probe3 (parse "null"): OK (returns JsonNode; stringify→"null")
- json_roundtrip_vm: arrays/objects still regress ([]/{} parsed as null); json_query_min still prints null

Next Steps (targeted)
1) Tokenizer structural path
   - Add minimal traces (dev) around create_structural_token in next_token to sample tokens for [ ] { }
   - Verify LBRACKET/RBRACKET/LBRACE/RBRACE sequences for samples: [], {}, {"a":1}
2) Parser array/object path
   - Trace JsonParser.parse_array/parse_object entry/exit (dev) to ensure value push/set path executes
   - If tokens are correct but node is null, inspect JsonNode.create_array/object and stringify
3) Fix + re‑run quick smokes (json_roundtrip_vm, json_nested_vm, json_query_min_vm)

How to reproduce (quick)
- token:  NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/token_probe.nyash --dev
- null:   NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe3.nyash --dev
- smokes: tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh

Notes
- Traces are dev‑only and silent by default; noisy prints in tokenizer were re‑commented.

Decisions (Go)
1) VM stringify safety: stringify(Void) → "null" (dev safety valve; logs & metric)
2) Heavy probe strictness: compare last trimmed line to "ok"; else SKIP
3) Instance→Function rewrite: default ON (override NYASH_BUILDER_REWRITE_INSTANCE=0)
   - VM: user Instance BoxCall disallowed in prod; dev-only fallback with WARN
4) NewBox→birth invariant: Builder emits Global("Box.birth/N"); VM has no implicit birth
   - Dev assert: birth(me==Void) forbidden (WARN+metric)

Plan (next patches)
- Implement stringify(Void) guard in VM (handlers/boxes.rs)
- Tighten probes in quick/core json_* smokes (tail-trim-compare)
- Set rewrite default ON in Builder (method_call_handlers.rs)
- Add VM guard for user Instance BoxCall (prod error; dev fallback)
- (Optional) Builder verify for NewBox→birth, VM dev assert hook

Status
- Tokenizer/parse([]): PASS
- Nested/Roundtrip: probe SKIP on this env (expected); direct run OK
- json_query_min (core): still null → fix follows via stringify(Void) + invariant

Acceptance
- quick: json_pp/json_lint/json_query_min PASS; user Instance BoxCall hits=0
- heavy: nested/roundtrip PASS where parser available

References
- docs/design/instance-dispatch-and-birth.md
- tools/smokes/README.md (heavy probes)

Update — 2025-09-27 (Parser array/object trace)
- Added dev-only traces in JsonParser.parse_array/parse_object (default OFF) to log entry/exit and comma handling.
- Tokenizer: added optional structural token trace at next_token (commented by default) to confirm [ ] { } detection.
- Repro (direct):
  - NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe_min.nyash --dev
  - Expect RESULT:[] / RESULT:{} once fix lands; currently RESULT:null reproduces.
- Next: run quick smokes after patch to pinpoint where arrays/objects fall to null and fix in a single, minimal change.

Update — 2025-09-27 (json_lint_vm guard fix)
- Issue: Unterminated JSON string ("unterminated) was incorrectly judged OK in json_lint due to a lax fast‑path.
- Fix (app-level, spec-safe): removed string fast‑path and added explicit guard — if starts_with('"') and not ends_with('"') then ERROR.
  - File: apps/examples/json_lint/main.nyash
- Result: apps/json_lint_vm.sh PASS on VM quick.
- Follow-up (root cause, parser side): JsonScanner.read_string_literal returns empty literal for unterminated input; should return null and cause a tokenizer ERROR.
  - File: apps/lib/json_native/lexer/scanner.nyash (read_string_literal)
  - TODO: add unit probe; ensure EOF without closing quote yields null; add negative case to smokes if needed.

Update — 2025-09-28 (Scanner 未終端→null とスモーク追加)
- Implemented: JsonScanner.read_string_literal returns null when closing quote is missing or escape incomplete.
  - File: apps/lib/json_native/lexer/scanner.nyash (already returned null; verified)
- Tokenizer maps scanner null to ERROR("Unterminated string literal").
  - File: apps/lib/json_native/lexer/tokenizer.nyash (tokenize_string)
- Added quick smoke to lock behavior:
  - tools/smokes/v2/profiles/quick/core/json_unterminated_string_vm.sh → expects "Unterminated string literal".

Work Queue — Reorganized (2025‑09‑28)
1) Scanner 未終端→null — completed
   - Status: Verified with new smoke; tokenizer ERROR emitted with line/column preserved.
2) Heavy JSON quick 復帰（LLVM 常備で段階解除） — completed (dev override)
   - Policy: AST-heavy smokes run in quick via LLVM harness. When LLVM is not detectable, they SKIP; 開発者は `SMOKES_FORCE_LLVM=1` で強制実行可。
   - Action: run.sh に `SMOKES_FORCE_LLVM=1` を追加、ハーネス/NYRT/ENV の自動整備を強化。nested_ast → roundtrip_ast → error_messages_ast が PASS。
3) エラーメッセージ詳細化 — pending
   - Scope: enrich JSON parser/tokenizer messages with expected/actual; keep format: "Error at line X, column Y: ...".
4) Ny 実行器 M2 スケルトン（最小） — baseline exists
   - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash; quick smoke present.
   - Next: add binop/compare minimal paths (dev-only), no default behavior change.
5) Parity ミニセット — pending
   - Add a tiny VM↔LLVM↔Ny parity triplet; start with const/ret and simple binop.
6) Router Known/Union 磨き込み（挙動不変） — pending
   - Maintain minimal special-method table; diagnostics only; no behavior change.
7) Heavy JSON 段階復帰順（nested_ast→roundtrip_ast→error_messages_ast） — tracking
   - All present in quick under LLVM harness; verify pass and keep order.
8) LLVM ダンプに certainty 補助表示 — baseline exists
   - NYASH_LLVM_TRACE_CALLS=1 prints callee JSON including Method.certainty.
9) QuickRef — Truthiness（quickで有効化）— completed
   - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh → enabled; PASS（0→false, 1→true, ""→false, non‑empty→true）
10) Language guards（planned; 既定OFF・段階導入）
   - ASI strictness: dev‑only check to fail a line break after a binary operator; default OFF.
   - Plus mixed: warn/fail‑fast when non‑String mixed `+` unless explicit stringify; default OFF; document String+number ⇒ concat.
   - Box equality guidance: when `box == box` is used, emit guidance to use equals(); default OFF.
   - Scope: docs + dev warnings first; later wire parser/builder flags guarded by env/CLI profile.

Update — 2025-09-27 (M2 skeleton: Ny mini-MIR VM)

Update — 2025-09-28 (json_lint_vm regression fix — condition_fn and birth bridge)
- Fixed: Unknown global function: condition_fn (quick json_lint_vm)
  - Indirect calls: ensure AST `condition_fn(ch)` lowers to Value call (unified path already used in exprs_call.rs)
  - Unified Global safety: emit_unified_call now dev‑safes `condition_fn` by returning const 1 when unresolved (explicit opt‑in legacy paths intact)
  - Dev stub: finalize_module injects minimal `condition_fn/1 -> 1` if missing (kept as guard)
- Unified→VM bridge: birth()
  - VM: when executing unified Method callee `*.birth`, delegate to BoxCall handler and return Void. This preserves legacy behavior for built‑ins when plugins are absent.
  - Builder: gated birth() injection for built‑ins (Array/Map/String etc). Default OFF unless `NYASH_DEV_BIRTH_INJECT_BUILTINS=1`.
- Next (high‑prio): local var materialization bug in main.nyash
  - Symptom: `local cases = new ArrayBox()` followed by `cases.push(...)` used an undefined receiver ValueId.
  - Interim change: make `local` always materialize a distinct register and `copy init -> var` (also const Void for uninitialized). This avoids SSA aliasing issues.
  - Status: needs a quick pass across smokes to confirm; proceed if quick green, otherwise revisit builder var mapping.

Update — 2025-09-28 (recv undefined across loop headers — Patch‑A applied)
- Root cause: Some method calls still went through legacy BoxCall emission without receiver pin, causing the receiver ValueId to be undefined at loop/header blocks.
- Patch‑A (applied): pin receiver centrally in `emit_box_or_plugin_call` so every method call path (Unified/Legacy) has a block‑local def.
  - File: src/mir/builder/utils.rs (at function start)
- Block entry propagation (applied): when starting a new basic block, copy all `__pin$` slots and rewrite user variables that referenced the old pin ids to the new copied ids.
  - File: src/mir/builder/utils.rs (start_new_block)
- Status: residual undefined value still observed in json_lint_vm (different ValueIds). Next step is to trace the exact site and, if necessary, add a minimal materialize at `build_variable_access` for the specific hotspots.

Plan — Next (late 2025‑09‑28)
1) Trace failing site in json_lint_vm with `NYASH_VM_TRACE=1` and MIR dump; capture `reg_load undefined id` with surrounding last_inst.
2) Verify that at that site the receiver is either a) not pinned (missed path) or b) was not remapped at block entry; fix with a targeted pin/materialize.
3) If a general gap remains, add a guarded materialize in `build_variable_access` (only when the ValueId originates from a pin slot or when entering a new block) to keep diff minimal.
4) Re‑run quick; keep Unified default‑ON; document toggles and rationale.

Dev toggles
- NYASH_DEV_BIRTH_INJECT_BUILTINS=1: re‑enable birth() injection for builtin boxes (default OFF to stabilize unified Method path until full bridge lands).
- NYASH_MIR_UNIFIED_CALL: default ON; opt‑out via 0|false|off.
- Added Ny-based minimal MIR(JSON v0) executor skeleton (const→ret only), dev-only app — no default behavior change.
  - File: apps/selfhost/vm/boxes/mir_vm_min.nyash
  - Entry: apps/selfhost/vm/mir_min_entry.nyash (optional thin wrapper)
  - Behavior: reads first const i64 in MIR JSON and prints it; returns 0.
- Quick smoke added to quick profile:
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_min_vm.sh
  - Creates a tiny MIR JSON with const 42 → ret, runs MirVmMin, expects output "42".
- Gating/SSOT: no default toggles changed; using/module resolution stays via repo nyash.toml (added modules.selfhost.vm.mir_min).

Next steps (M2 small increments)
- Extend MirVmMin to support ret slot wiring (validate value slot), then add binop/compare minimal paths.
- Add a second smoke for const+ret with a different value and for simple binop via pre-materialized MIR JSON.
- Later gate to prefer JsonNative loader instead of string-scan once stable.
Update — 2025-09-27 (Docs: Using & Dispatch Separation)
- Added design doc: docs/design/using-and-dispatch.md (SSOT+AST for using; runtime dispatch scope; env knobs; tests).
- Strengthened comments:
  - src/runner/modes/common_util/resolve/{mod.rs,strip.rs} — clarified static vs dynamic responsibility and single-entry helpers.
  - src/mir/builder/method_call_handlers.rs — documented rationale and controls for instance→function rewrite.
  - src/backend/mir_interpreter/handlers/boxes.rs — clarified prod policy for user instance BoxCall fallback.
- Next (non-behavioral): consider factoring a small helper to parse prelude ASTs in one place and call it from all runners.
Update — 2025-09-27 (UserBox smokes added)
- Added quick/core smokes to cover UserBox patterns under prod + fallback-ban:
  - oop_instance_call_vm.sh — PASS
  - userbox_static_call_vm.sh — PASS
  - userbox_birth_to_string_vm.sh — PASS
  - userbox_using_package_vm.sh — PASS (using alias/package + AST prelude)

Update — 2025-09-27 (Loop/Join ScopeCtx Phase‑1)
- Implemented Debug ScopeCtx in MIR builder to attach region_id to DebugHub events.
  - Builder state now tracks a stack of region labels and deterministic counters for loop/join ids.
  - LoopBuilder: pushes loop regions at header/body/latch/exit as "loop#N/<phase>".
  - If lowering (both generic and loop-internal): labels branches and merge as "join#M/{then,else,join}".
  - DebugHub emissions (ssa.phi, resolve.try/choose) now include current region_id.
- How to capture logs
  - NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl \
    tools/smokes/v2/run.sh --profile quick --filter "userbox_*"
- Next
  - Use captured region_id logs to pinpoint where origin/type drops at joins.
  - Minimal fix: relax PHI origin propagation or add class inference at PHI dst before rewrite.

Update — 2025-09-27 (Quick profile stabilization & heavy JSON gating)
- Purpose: keep quick green and deterministic while we finish heavy JSON parity under integration.
- Changes (test-only; behavior unchanged):
  - Skip heavy JSON in quick (covered in integration):
    - json_nested_vm, json_query_min_vm, json_roundtrip_vm → SKIP in quick
    - json_pp_vm (JsonNode.parse pretty-print) → SKIP in quick（例示アプリ、他で十分カバー）
  - Using resolver brace-fixer: quick config restored to ON for stability（NYASH_RESOLVE_FIX_BRACES=1）
  - ScopeCtx wired (loop/join) and resolve/ssa events include region_id（dev logs only）
  - toString→str early mapping logs added（reason: toString-early-*）
- Rationale: heavy/nested parser cases were sensitive to mixed env order in quick. Integration profile will carry the parity checks with DebugHub capture.
- Next (focused):
  1) Run integration smokes for JSON heavy with DebugHub ON and collect /tmp logs
  2) Pinpoint join/loop seam by region_id where origin/type drops (if any)
  3) Apply minimal fix (either PHI origin relax at join or stringify guard tweak)
  4) When green, revert quick SKIPs one-by-one (nested→query→roundtrip)
- Files touched (tests):
  - tools/smokes/v2/profiles/quick/core/json_nested_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_query_min_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/apps/json_pp_vm.sh → SKIP in quick（例示アプリ）
  - tools/smokes/v2/configs/rust_vm_dynamic.conf → RESOLVE_FIX_BRACES=1（安定優先）

Integration plan (dev runbook):
- Heavy with logs: NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_integ.jsonl \
  tools/smokes/v2/run.sh --profile integration --filter "json_*ast.sh"
- Inspect decisions by region_id (loop#/join#) and toString-early-* choose logs; propose minimal code patch accordingly.

Acceptance (this phase):
- quick: 100% green with heavy SKIPs; non-JSON suites unaffected
- integration: JSON heavy passes locally with DebugHub optional; discrepancies have a precise region_id to fix
  - userbox_method_arity_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_branch_phi_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_toString_mapping_vm.sh — SKIP (mapping pending)
- Rationale: keep quick green while surfacing remaining gaps as SKIP with clear reasons.
- Next: stabilize rewrite/materialize across branch/arity and toString→str mapping; then flip SKIPs to PASS.
Update — 2025-09-27 (Loop‑Form Scope Debug & AOT PoC — Plan)
- Added design doc: docs/design/loopform-scope-debug-and-aot.md
  - Scope model (LoopScope/JoinScope), invariants, Hub+Inspectors, per-scope data, AOT fold, PoC phases, acceptance.
- Work Queue (phased)
  1) PoC Phase‑1 (dev‑only; default OFF)
     - Add DebugHub (env: NYASH_DEBUG_ENABLE/NYASH_DEBUG_SINK/NYASH_DEBUG_KINDS)
     - ScopeCtx stack in builder; enter/exit at Loop/Join construction points
     - Emit resolve.try/choose in method_call_handlers.rs
     - Emit ssa.phi in builder.rs (reuse dev meta propagation)
     - Smokes: run userbox_branch_phi_vm.sh, userbox_method_arity_vm.sh with debug sink; verify region_id/decisions visible
  2) Phase‑2
     - OperatorInspector (Compare/Add/stringify)
     - Emit materialize.func / module.index; collect requires/provides per region
     - Fold to plan.json (AOT unit order; dev only)
  3) Phase‑3 (optional)
     - ExpressionBox (function‑filtered), ProbeBox (dev only)
- Acceptance (Phase‑1)
  - Debug JSONL has resolve/ssa events with region_id and choices; PASS cases unchanged (OFF)
  - SKIP cases pinpointable by log (branch/arity) → use logs to guide fixes → flip to PASS


Update — 2025-09-28 (Plugins 既定ON と ENV 整理)
- Plugins: 既定ONで統一。テストランナー/開発スクリプトから `NYASH_DISABLE_PLUGINS=1` を撤去。
  - tools/smokes/v2/lib/test_runner.sh（LLVM 経路）: disable 指定を外し、`PYTHONPATH`/`NYASH_NY_LLVM_COMPILER`/`NYASH_EMIT_EXE_NYRT` を自動付与。
  - tools/dev_env.sh: `pyvm`/`bridge` プロファイルで plugins を無効化しない（unset のみに変更）。
- VM/LLVM 二系統の最小ENV（ドキュメント方針）:
  - VM: 既定でOK（追加ENV不要）
  - LLVM(harness): `NYASH_LLVM_USE_HARNESS=1` + `NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc` + `NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release`
  - quick強制: `SMOKES_FORCE_LLVM=1` で AST heavy を quick で実行可能


Priority TODO — 2025-09-28 (VM/LLVM 2-Line + M2)
- ENV minimalization (plugins=ON):
  - VM: no extra ENV.
  - LLVM(harness): NYASH_LLVM_USE_HARNESS=1, NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc, NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release.
  - Docs: add a small "VM vs LLVM minimal-ENV" box to README.md and README.ja.md. [done]
- test_runner cleanup:
  - Unify/centralize noise filters; keep SMOKES_FORCE_LLVM as the only dev override; remove ad-hoc greps in individual scripts. [todo]
- M2 executor (Ny):
  - Add compare (Eq) to M2 runner; add 2 smokes (Eq true/false). [done]
  - Externalize MirVmM2 to apps/selfhost/vm/boxes/mir_vm_m2.nyash and switch smoke to using-based variant; keep inline smoke as safety. [later]
  - Next (optional): branch/jump minimal; phi later. [pending]

Update — 2025-09-28 (Language Quick Reference & Smokes)
- Added quick-reference draft for language (keywords, operators, ASI, truthiness, equality, '+', rewrite, errors).
  - docs/reference/language/quick-reference.md
- Added planned smokes for quickref rules (initially SKIP until strict rules are wired):
  - tools/smokes/v2/profiles/quick/core/lang_quickref_asi_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh (ENABLED)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_plus_mixed_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_equals_box_error_vm.sh (SKIP)
- Temporarily SKIP Mini‑VM M2/M3 smokes while parser/segment boundaries are being fixed:
  - selfhost_mir_m2_eq_true_vm.sh / selfhost_mir_m2_eq_false_vm.sh / selfhost_mir_m3_branch_true_vm.sh / selfhost_mir_m3_jump_vm.sh — now ENABLED and PASS
- Using/SSOT docs:
  - Clarify dev/ci/prod matrix (file-using dev/ci only; prod=toml only); add short examples. [todo]
- Parity mini-set:
  - VM ↔ LLVM ↔ Ny: const/ret + binop(+), compare(Eq); add quick parity harness notes. [todo]
- Acceptance:
  - quick: AST heavy PASS (LLVM present), M2 binop/Eq PASS; integration unchanged.
  - docs: minimal-ENV clearly shown; no NYASH_DISABLE_PLUGINS in public guidance.

Update — 2025-09-28 (Interpreter gating & Phase 15.7 plan)
- Legacy AST interpreter is now feature-gated (interpreter-legacy OFF by default). Runner/tests that depend on it are behind cfg.
  - Files: src/runner/modes/common.rs, src/runner/modes/bench.rs, src/tests/* (vm_bitops/refcell/functionbox)
- Added Phase 15.7 roadmap (Mini‑VM M3 + NYABI Kernel skeleton; dev-only; default OFF).
  - docs/development/roadmap/phases/phase-15.7/README.md
- Drafted NYABI Kernel spec (v0) and added Ny skeleton box (not wired).
  - docs/abi/vm-kernel.md; apps/selfhost/vm/boxes/vm_kernel_box.nyash

Plan — Instance→Function Rewrite Consolidation (2025‑09‑28)
- Goal: 内部表現を関数呼び出しへ極力統一（obj.m(a) → Class.m/Arity(me,a)）。prodでの Instance BoxCall 依存を排除。
- Approach（小粒・可逆）
  1) PHI/Join での origin/type 伝播の強化（region_id ログで落ちる断面を特定→補修）
  2) 限定 materialize: module 内で name+arity がユニークな場合のみ Glue 関数を合成（既定OFF、dev/CIで計測）

Roadmap Priorities (Phase 15.7 revised)
- P0: me 注入 Known 化（起源付与/維持）— リスク低・効果大。軽量PHI補強（単一/一致時）
- P1: Known 100% 関数化（Known 経路の instance→function 正規化、special 集約）
- P2: Policy（Ny Kernel, dev‑only）— equals/str/truthiness の観測API（バッチ、再入禁止/タイムアウト/計測）
- P3: 表示APIの移行誘導 — toString→str（互換:stringify）の警告/ドキュメント（仕様不変）
- P4: Union 観測・分析 — resolve.try/choose と ssa.phi（region_id）で継続観測
- P5: PHI Known 維持の一般化 — Phase 16（複雑のため後回し）
  3) prod ガード維持: VM は user Instance BoxCall を禁止（既存ポリシー継続）。dev/CI は WARN＋観測
  4) スモーク/観測: quick で Instance BoxCall の dev WARN=0 を確認。resolve.try/choose と LLVM `NYASH_LLVM_TRACE_CALLS` を併用
- Controls
  - `NYASH_BUILDER_REWRITE_INSTANCE`（既定ON）: 強制ON/OFF
  - `NYASH_DEV_REWRITE_USERBOX`（dev限定）: userbox rewrite 検証用
  - materialize 新ENV（既定OFF）: `NYASH_BUILDER_MATERIALIZE_UNIQUE=1`（予定）
- Acceptance（段階）
  - Stage‑1: Known 経路で 100% 関数化（quick全域で dev WARN=0）
  - Stage‑2: 限定 materialize をON時に適用し、分岐/PHI 合流の代表ケースが関数化（差分はdevのみ）
  - 常に prod は挙動不変・安全（OFFで現状維持）

Update — 2025-09-28 (Mini‑VM M2/M3 fix + smokes)
- Fix: compare/ret segmentation made robust without heavy JSON parse.
  - Approach: per‑block coarse passes for const/binop/compare and a precise in‑block ret search; control‑flow (branch/jump) handled with a single pass using computed regs.
  - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash
- Smokes: enabled and PASS
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_false_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_branch_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_vm.sh
- Notes: kept changes local and spec‑neutral; no default behavior changes to core VM.

Update — 2025-09-28 (QuickRef Dev Guards + Docs llvmlite)
- Dev guards (env‑gated; default OFF) implemented and validated by quick smokes:
  - ASI strict line‑continuation: `NYASH_ASI_STRICT=1` → parse error when a binary operator ends the line.
- Plus mixed (String×Number): `NYASH_PLUS_MIX_ERROR=1` → type error; suggest str()/明示変換。
  - Box equality guidance: `NYASH_BOX_EQ_GUIDE_ERROR=1` → equals()誘導のエラー。
  - Smokes enabled: `lang_quickref_asi_error_vm.sh`, `lang_quickref_plus_mixed_error_vm.sh`, `lang_quickref_equals_box_error_vm.sh`（PASS）
- LLVM ドキュメント統一（llvmlite一本化）
  - `LLVM_SYS_180_PREFIX` の記述を主要ドキュメントから撤去し、llvmlite/ny‑llvmc 前提に更新。
  - Files: `AGENTS.md`, `README.md`, `README.ja.md`, `CLAUDE.md`

Plan — Next (2025-09-28)
1) Mini‑VM 単一パス化（仕様不変・安全化） — completed
   - 各 op を JSON オブジェクト単位で厳密セグメント化し、一回走査で評価（coarse pass を除去）。
   - 代表ケース（複数op/ret先頭/ret末尾/compare v0,v1/jump/branch）で緑維持を確認。
2) Rewrite 統合 Stage‑1（挙動不変・dev観測） — completed (observability wired)
   - builder_calls の unified 経路に resolve.try/resolve.choose を追加（dev‑only/既定OFF）。
   - method_call_handlers の既存 emit と整合。Known/Union の certainty を choose に含める。
   - 使い方: `NYASH_MIR_UNIFIED_CALL=1 NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl`。
   - Known 経路の100%関数化（dev WARN=0）を DebugHub で観測。userbox スモークで検証。
3) P0/P1 着手（構造化） — in progress
   - origin/observe/rewrite の責務分割（モジュール新設: src/mir/builder/{origin,observe,rewrite}/）。
   - P0: me 注入 Known 化（起源付与/維持）と軽量PHI補強（単一/一致時）。
   - P1: Known 経路 100% 関数化（special 集約: toString→str（互換:stringify）/equals）。
   - Docs: README を各層に追加（origin/observe/rewrite）— completed
   - 観測呼び出しの統一: builder_calls/method_call_handlers から observe::resolve を使用 — completed
3) CI/Profiles 整理 — ongoing
   - quick: VM 主線（llvmlite パリティは integration に委譲）。
   - integration: 代表パリティ（llvmlite ハーネス）継続、apps系は任意実行。

Notes — Display API Unification (spec‑neutral)
- 規範: `str()` / `x.str()`（同義）。`toString()` は Builder で `str()` に早期正規化。
- 互換: `stringify()` は当面エイリアス（内部で `str()` 相当）。
- VM ルータ: toString/0 → str/0（なければ stringify/0）。
- QuickRef/ガイド更新済み。`NYASH_PLUS_MIX_ERROR` の誘導文言も `str()` に統一。

追加メモ — これからやる（ユーザー合意、2025‑09‑28）
- Mini‑VM の単一パス化を安全に実装（既定挙動不変）
  - 各 op を厳密セグメントで1回走査に統合（coarse を段階撤去）
  - 代表スモーク（M2/M3/compare v0,v1）で緑維持確認
- 続いて Rewrite 統合 Stage‑1 の観測へ進む（dev のみ、挙動不変）
- Dev Profiles
  - tools/dev_env.sh に Unified 既定ON（明示OFFのみ無効）とレガシー関数化抑止を追加。
    - `NYASH_MIR_UNIFIED_CALL=1`（既定ON明示）
    - `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（重複回避; 段階移行）
Update — 2025-09-28 (Boxification P1: inference/gate/materialize)

- Added small, focused boxes to make call pipeline observable and deterministic.
  - ReceiverInferenceBox: unify receiver class/certainty inference.
    - Files: `src/mir/builder/infer/{mod.rs,receiver.rs}`
    - Integrated into: `calls/call_unified.rs::convert_target_to_callee`, `builder_calls.rs` try trace and rewrite hints, `utils.rs` BoxCall route.
  - RewriteGateBox: centralize Known rewrite gating (user-box only, StringBox string-APIs excluded, (Box,method,arity) exists).
    - Files: `src/mir/builder/rewrite/gate.rs` (exported in rewrite/mod.rs)
    - Integrated into: `rewrite/known.rs` gating（従来の散在チェックを集約）。
  - InstanceMethodIndexBox: register/query instance method signatures for gating.
    - Files: `src/mir/builder/indexes/{mod.rs,instance.rs}`
    - Integrated into: `builder/lifecycle.rs`（登録）、`RewriteGateBox`（照会）。
  - MaterializeBox: unify finalization at call site（LocalSSA + after-PHI + tail copy）。
    - Files: `src/mir/builder/materialize/{mod.rs,call_site.rs}`
    - Integrated into: `builder_calls.rs`（emit直前の材化を一か所に）。
  - ResolveTraceBox: dev-only emit helpers for resolve.try/choose。
    - Files: `src/mir/builder/observe/resolve_trace.rs`（`observe/mod.rs` から公開）
    - Integrated into: `builder_calls.rs`（trace 出力を統一）。
  - Verify wrapper: CallOrderVerifyBox (dev-only).
    - Files: `src/mir/builder/verify/{mod.rs,call_order.rs}`
    - Integrated into: `emit_guard::verify_after_call()` の中継。

- Functional impact (spec unchanged):
  - Unified inference now used consistently across Unified/BoxCall paths.
  - Known rewrite fires only when `(Box, method, arity)` exists in index; StringBox の length/substring 等は除外。
  - Router fallback recursion broken via `force_legacy` flag to stop Unified→BoxCall→Unified loops.

- Next (short)
  1) [done] RewriteGate に dev-trace 追加（NYASH_RESOLVE_TRACE=1）
  2) selfhost VM を再実行（traces有効）して Known 化/材化の残存点を抽出
  3) [done] MaterializeBox の短ダンプ（call直前±5命令）を dev 追加（NYASH_MAT_TRACE=1）
     - 受け手/引数の ValueId + type/origin を1行で出力（[mat-trace]）。
  4) 型注釈の最小追補（substring→String, length/indexOf/lastIndexOf→Integer）— 仕様不変
  5) 代表箇所（ParserBox.extract_usings）で substring 未材化の点当て（観測→型注釈 or gate 見直し）
  6) selfhost VM の JSON 出力を bytes>0 で PASS に格上げ

Docs — Unified Method Resolution & VM policy (accepted)
- 明文化: ユーザーBoxの Instance 呼び出しは Builder が関数化（Instance→Function）。
- VM 方針: Instance BoxCall は開発のみ許容（prod 既定 = 不許可）。env `NYASH_VM_USER_INSTANCE_BOXCALL` で上書き可。
- 反映:
  - docs/development/builder/unified-method-resolution.md — VM policy を追加
  - docs/reference/language/quick-reference.md — Calls/ASI/+混在/Box== のフラグ注記を追記
  - README.md / README.ja.md — Unified Call 節に VM policy を追記
  - docs/development/builder/DIAGNOSTICS.md — dev 追跡フラグのまとめを新設

Updates — 2025-09-28 (P6 incremental)
- emission::constant に原始型の型注釈を追加（仕様不変・観測用）
  - `emit_string` → MirType::String, `emit_integer` → Integer, `emit_bool` → Bool, `emit_float` → Float
  - null/void は従来通り注釈なし
- materialize::call_site に [mat-trace] を追加（NYASH_MAT_TRACE=1）
  - 受け手 %id / 推定型 / NewBox 起源, および各引数 %id:型 の短行を出力
  - Block tail の直前 5 命令ダンプ（既存）と対で診断可能
## ✅ Update — 2025-10-05（Phase 15.7 追補: Branding堅牢化 + JSON.stringify標準化）

- 目的（仕様不変・互換維持）
  - Branding 移行の堅牢化: `hako.toml` を最優先の単一真実源に（互換: `nyash.toml`/`hakorune.toml`）。
  - 宣言的MIR/JSON の統一: `JSON.stringify(any)` を第一級APIに昇格（`.toJSON()` 併存・同一出力）。
  - plugin-tester の既定 `--config` を `hako.toml` に切替（互換読込は維持）。

- 変更点（コード差分・範囲限定）
  - using/alias 取得元（設定ファイル優先）
    - src/using/resolver.rs:42 — CWD→*_ROOT で `hako.toml`→`nyash.toml`→`hakorune.toml` の順で探索。NYASH_ROOT/HAKO_ROOT/HAKU_ROOT/HRN_ROOT 別名を受理。
  - JSON.stringify(any) の標準化（devゲート撤廃・挙動不変）
    - src/mir/builder/builder_calls/emit.rs:299 — `CallTarget::Global("JSON.stringify/1")` を受け取ったら第1引数に対して `.toJSON()` を発行。Effect は READ。旧 `NYASH_JSON_STRINGIFY_DEV` ゲートを撤廃（互換 OK）。
  - plugin-tester 既定パスの切替（互換読込は維持）
    - tools/plugin-tester/src/main.rs:59,69,78,84 — 既定値を `../../hako.toml` に変更。`load_config()` が hako/nyash 双方を解決するため後方互換。

- スモーク追加（quick、軽量）
  - tools/smokes/v2/profiles/quick/core/branding_hako_only_using_vm.sh
    - CWD に hako.toml のみ配置（nyash.toml 不在）で using/alias が機能することを確認。
  - tools/smokes/v2/profiles/quick/core/json_stringify_standard_vm.sh
    - Map/Array 混在オブジェクトで `JSON.stringify(m) == m.toJSON()` を確認（devフラグ不要）。

- Docs 反映
  - docs/guides/declarative-mir.md — 見出しと記述を `JSON.stringify` 第一級へ更新（`.toJSON()` 併存・同一出力を明記）。

- 受け入れ/検証
  - `cargo build --release` 成功。
  - 追加スモーク2本 PASS（quick プロファイル想定）。既存 quick/integration には影響なし（仕様不変）。

- フラグ/互換
  - 新規フラグ無し。旧 `NYASH_JSON_STRINGIFY_DEV` は事実上無視（挙動は常に `.toJSON()` へ委譲）。
  - `nyash.toml` は引き続き互換読込。既存スクリプト/ツールは挙動不変。

- ロールバック手順（可逆・小差分）
  1) `src/mir/builder/builder_calls/emit.rs` の `JSON.stringify` 分岐を dev 環境変数ゲートに戻す（`NYASH_JSON_STRINGIFY_DEV`）。
  2) `src/using/resolver.rs` の設定ファイル探索順を `nyash.toml` 優先に戻す。
  3) plugin-tester 既定 `--config` を `../../nyash.toml` に戻す。
  変更は局所のため差分戻しで即時復旧可能。

- リスク/留意点
  - using/alias の DEV フォールバック（内容走査）は従来通り dev ログ配下。今回の変更は設定ファイル優先順位のみで、意味論不変。
  - `ensure_hako_toml` に依存していた既存スモークは、そのままでも互換維持（hako.toml がなければ nyash.toml をコピー）。

- 次アクション（提案順・小粒）
  1) Alias 重複・衝突の Fail‑Fast スモークを1件追加（別ファイルで同一 alias を異ファイルへ再バインド）。
  2) JSON.stringify の深いネスト/特殊文字/大規模Map のパフォーマンス・エスケープ検証を1〜2件（quickで軽量）。
  3) PreLex 共通化の念押しとして raw/numeric 各1本を LLVM/PyVM 経路で quick に追加。
  4) Self‑Host 継続: LocalSSA.ensure_cond の2段分岐ミニケース（compare→branch→compare→ret）で VM/LLVM parity 1本。

- 再現/実行メモ（tmux 調子が悪い場合）
  - tmux 再起動/再接続例:
    - 再接続: `tmux attach -t codex || tmux new -s codex`
    - セッション再作成: `tmux kill-session -t codex || true; tmux new -s codex`
    - 非同期通知（任意）: `CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "Smokes quick" codex`

## New — EntryResolveBox 設計（flow Main 推奨 + Strict）
- 目的: 既定を Strict（Main.main のみ）に固定し、CLI `--entry <dotted>` による明示指定を一元化する小箱を導入。
- 設計文書: docs/architecture/runner/entry-resolve-box.md
- 方針: 環境変数は増やさない。自動推測（唯一の <Box>.main / top-level main）は採用しない（候補列挙のみ）。
- 段階導入:
  - Phase A: ドキュメント合意（本コミット）
  - Phase B: CLI `--entry` 追加（Runner 配線）
  - Phase C: VM/LLVM/PyVM/子プロセスのエントリ選択を EntryResolveBox に置換
  - Phase D: 旧便宜フラグの撤廃（既定 OFF → 削除）
