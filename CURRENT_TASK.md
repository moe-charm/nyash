# Current Task — Phase 15 Self‑Hosting (2025‑09‑16)

TL;DR
- 目標は「自己ホスティング達成」＝ Nyash製パーサで Ny → JSON v0 → Bridge → MIR 実行を安定化すること。
- PyVM は意味論の参照実行器（開発補助）。llvmlite は AOT/検証。配布やバンドル化は後回し（基礎固めが先）。

What Changed (today)
- Selfhost 経路の安定化（Python MVP 優先→PyVM 実行）。Selfhost Stage‑2（直/Bridge）スモークは緑化。
- Using/Resolver を Runner 前処理に集約し、BoxIndex（グローバル）＋解決キャッシュを導入。
  - nyash.toml の `[aliases]`/env `NYASH_ALIASES` 対応、候補提示、`NYASH_RESOLVE_TRACE=1` でトレース。
  - strict プレフィクス: `NYASH_PLUGIN_REQUIRE_PREFIX=1` または `[plugins] require_prefix=true`。
  - per‑plugin meta（`prefix/require_prefix/expose_short_names`）の読取導線を実装（挙動は現状据え置き）。
- CLI `--using` を追加（`--using "ns as Alias"` / `--using '"apps/foo.nyash" as Foo'`）。
- フィールドは box 先頭のみルールのリンタを Runner に追加（`NYASH_FIELDS_TOP_STRICT=1` でエラー）。
- Syntax Torture スイートの実行正規化（末行比較）。一部テスト本文を Nyash 仕様に合わせて修正。
- JSON v0 仕様に Stage‑3 ノード（Break/Continue/Throw/Try）を追記。Parser Stage‑3 設計メモの現状/残課題を更新。
- LLVM curated smokes を新設（`tools/smokes/curated_llvm.sh`）。core/async/loop/peek を10s/ケースで実行、PHI‑off も検証可（`--phi-off`）。
- LLVM Stage‑3 受理スモークを追加（`tools/smokes/curated_llvm_stage3.sh`）。try/finally・dead throw を10s/ケースで実行。PHI‑off（`--phi-off`）/trap抑止（`NYASH_LLVM_TRAP_ON_THROW=0`）も確認。
- 旧スモークは `tools/smokes/archive/` へ整理（JIT/Cranelift 系は当面対象外）。
- Bridge/Builder に PHI 非生成モードを導入（`NYASH_MIR_NO_PHI=1`）。LLVM Resolver 合成と統一し、Verifier は緩和ゲート（`verify_allow_no_phi()`）を追加。
- LLVM 側に PHI 合成トレースを追加（`NYASH_LLVM_TRACE_PHI=1`）。jump/finalize/resolve の観測を統一タグで出力。
- LLVM Throw を最小降ろし（`llvm.trap`→`unreachable`、`NYASH_LLVM_TRAP_ON_THROW=0` で trap 抑止）。
- 環境変数アクセスを `config::env` に集約（`mir_no_phi()`/`verify_allow_no_phi()`/`llvm_use_harness()`）。
- dev プロファイル `tools/dev_env.sh phi_off` を追加。ルート清掃ユーティリティ `tools/clean_root_artifacts.sh` を追加。
- CI（GH Actions）を curated LLVM（PHI‑on/off）実行に刷新。旧JITジョブは停止。
- Verifier: verification.rs 内の compute_* ラッパーを撤去し、全て `verification::utils::*` を直参照に置換。
- Parser: `bit_or/xor/and` と `equality/comparison/range/term/shift/factor` に加え、`call/primary` も `parser/expr/` へ分割。`expressions.rs` は委譲ラッパーのみに縮退（互換維持）。
- Optimizer(BoxField): 同一ブロック内の set 直後の get（同一 box+index）を Copy に置換する軽量 peephole を追加（load-after-store 短絡）。
- LLVM(select/terminators): `function.rs` から `instructions::term_emit_*` を利用しつつ、`normalize_branch_condition()` をブランチ直前で適用する流れを固定化（truthy 正規化の前段フック）。
- Runner/env 集約: `src/config/env.rs` に CLI/自ホスト/VM まわりの getter を追加（`cli_verbose()/enable_using()/vm_use_py()/ny_compiler_*()` など）。`runner/selfhost.rs`/`runner/pipe_io.rs`/`runner/modes/common.rs` のホットパスを getter 参照に更新（段階導入）。
- VM dispatch: 実装は既に dispatch 中央化済み（`backend::dispatch` 経由）。`NYASH_VM_USE_DISPATCH` フラグの getter を追加（将来の選択切替用）。
- Runner/log: `runner/trace.rs` を追加。`cli_verbose()` と `cli_v!` マクロを導入し、`modes/common.rs` の一部情報ログを置換（verbose ガードの明確化）。
- Selfhost helpers: PyVM ハーネス実行を `NyashRunner::run_pyvm_harness(..)` として共通化。EXE パーサ経路の JSON 抽出をまとめる `exe_try_parse_json_v0(..)` を追加（段階適用の足場）。

Refactor Progress (2025‑09‑16, end of day)
- Runner: ヘッダ指令スキャンとトレース出力を分離（`runner/cli_directives.rs`, `runner/trace.rs`）。using 解決ログを集約。
- LLVM: terminators(select) の足場を追加し、呼び出しを alias 経由に切替（挙動不変）。
- Optimizer: パス別に分割し、オーケストレータから委譲（挙動不変の足場）。
  - `optimizer_passes/{normalize,diagnostics,reorder,boxfield,intrinsics}.rs`
  - 統計を `optimizer_stats.rs` へ分離。
- Verifier: 主要チェックをモジュール化し、`verification.rs` を薄いオーケストレータ化。
  - `verification/{ssa,dom,cfg,barrier,legacy,awaits,utils}.rs`
- AST: `Span` を `ast/span.rs` へ分離し、`ast.rs` は re‑export。
- Parser: expressions を段階分割（ternary/coalesce/logic を `parser/expr/*` へ）。

Remaining Refactors (Phase‑15 mainline)
- Verifier（仕上げ）
  - `verification.rs` 内の `compute_*` ラッパーを完全撤去し、全呼び出しを `verification::utils` に集約。
  - テスト追加: reachability/phi/await チェックの簡易ケース（任意）。
- Parser（段階分割の続き）
  - `bit_or/bit_xor/bit_and`、`comparison/range/term/shift/factor` を `parser/expr/` へ移動し、`parse_expression` チェインを維持。
  - `call/primary` は最後に移動（依存が多いため）。
- AST（構造の分離）
  - `ast/nodes/{structure,expression,statement}.rs` へノード定義を分離し、`ast.rs` は `pub use` 集約のみへ縮退。
- Optimizer（足場→実装へ）
  - `reorder/boxfield/intrinsics` の実装を段階導入（まず small win: CSE のキーフィルタ改善、boxfield の load-after-store）。
  - `normalize` の terminator 側の補完（未移行箇所があれば寄せる）。
- Runner/env 集約
  - ホットパスの環境参照を `config::env` getter へ置換（残件: VM trace/diagnostics の一部）。
- Runner/log 整理（第2〜第3弾）
  - 情報ログ（verbose 連動）を `cli_v!` に一本化。エラー系は従来通り eprintln のまま。
- Selfhost EXE 経路の関数抽出（仕上げ）
  - 既存の EXE‑first ブロックを `exe_try_parse_json_v0(..)` を用いた薄い分岐に縮退（挙動不変）。
- LLVM select/terminators（実装化）
  - `select` に truthy 規約の軽い正規化を追加（等価変換のみ）。
  - `terminators` へ実体移動（`flow` からの段階的差し替え）。
- VM dispatch（段階導入）
  - `NYASH_VM_USE_DISPATCH=1` フラグを導入し、無副作用命令から `backend/dispatch.rs` 経由に切替。

Notes
- すべて挙動等価の範囲で段階的に進める。足場化したモジュールは後続で実装を徐々に移す。

Self‑Hosting plumbing (2025‑09‑16, later in day)
- Runner: 自己ホスト経路で子プログラム（`apps/selfhost-compiler/compiler.nyash`）を優先実行し、`--read-tmp` 常時付与で安定運用に変更。
- PyVM 優先の統一（`NYASH_VM_USE_PY=1`）は EXE/inline/child の全分岐で尊重。
- CLI: `--stage3` を追加（`NYASH_NY_COMPILER_STAGE3=1` を設定）。
- Script args パススルー（B案）実装: `nyash ... FILE -- arg1 arg2` → `NYASH_SCRIPT_ARGS_JSON=["arg1","arg2"]` として `Main.main(args)` に ArrayBox で注入。
  - Interpreter: `ArrayBox.of(...)` を追加して初期配列を簡潔に構築。
  - Runner: Python MVP は `NYASH_NY_COMPILER_SKIP_PY=1` でスキップ可能（自己ホスト子を優先）。


Decision (Phase‑15 wrap‑up)
- MIR13 移行（PHI 非生成）: Phase‑15 の締めとして、MIR 生成層（Bridge/Builder）は PHI を生成しない方針に切替。PHI 合成は LLVM 層（llvmlite/Resolver）に集約。
- LoopForm は次フェーズ（MIR18）で導入: まずは MIR14 を維持し、次フェーズで `LoopHeader/Enter/Latch` 等の占位命令を追加。現行 Phase‑15 は CFG パターン検知でループ搬送値を合成。
- 例外は段階導入: Throw/Catch は現行維持（Bridge ゲートで出力可）。Try/Finally の構造化は将来の TryRegion で検討。

Next Focus (Throw/Try — LLVM first)
- ブリッジ設計: `emit_degraded_throw` の差し替え方針を策定し、JSON v0 `Try` ノード → MIR 変換の仕様を決める（Stage-3 例外モデル）。
- MIR Builder/Runtime 調査: Rust VM/PyVM の `ControlFlow::Throw` 経路と既存 TryCatch 降格の挙動を整理。必要に応じて docs と CURRENT_TASK に反映。
- PyVM 設計: 例外モデルをどこまで Python 側に実装するか決め、最小テスト計画を用意。
- LLVM 実装方針: Throw/Try の MIR 命令を LLVM 側がどう扱うか（panic扱い or fallback）を設計し、smoke 更新案を作る（現状 Throw は trap/unreachable 最小降ろし完了）。
- テスト計画: JSON フィクスチャと `tools/llvm_smoke.sh` を中心に Stage-3 例外用のスモーク/単体テストを整備。

Open Items (handoff)
- Selfhost Stage‑3 E2E smoke（Runner→子→JSON→Bridge）の最終緑化
  - 現状: 子へ `--read-tmp` は付与済み。`--stage3` は env→子へ透過済み。
  - 直近 TODO: tools/selfhost_stage3_accept_smoke.sh を Runner 優先経路に再調整し、`NYASH_NY_COMPILER_SKIP_PY=1` 併用で安定化。
  - 期待値: try/finally/break/continue/throw(accept) で exit=0（degrade 経路）
- CLI argv 前処理の挙動監視
  - `--` なしの通常実行での Positional FILE 受理を再検証（Clapの警告再現時は build_command の宣言順/Arg要件を再点検）。
- LLVM: Throw 実投げの終了コード方針を決定し、trap on/off の期待値を固定化。

Next Steps (suggested order)
1) Selfhost Stage‑3 E2E 緑化（smoke 修正 → 確認）
2) CLI FILE positional の回帰があれば即修正（Clap設定の微調整）
3) LLVM Throw 実投げの設計（終了コード/シグナル）＋スモーク
4) Runner 実装の集約（modes/common → selfhost.rs）と微ノイズ削減
5) CI に Selfhost Stage‑2/E2E（軽量）をオプションで追加

※ Cranelift/JIT 系は当面対象外。ビルド時も LLVM のみを有効化（JIT 関連 feature/CI は無視）。

Runner updates (2025‑09‑16)
- Selfhost pipeline: PyVM 優先（`NYASH_VM_USE_PY=1`）を全分岐で適用（EXE/inline/child 経路の一貫性）。
- 重複関数の整理: `modes/common.rs::try_run_ny_compiler_pipeline` は `selfhost.rs::try_run_selfhost_pipeline` に委譲（ドリフト防止）。
- Stage‑3 受理導線: `NYASH_NY_COMPILER_STAGE3=1` で子プロセスに `--stage3` を付与。inline フォールバックは `stage3_enable(1)` を既定有効化。

- llvmlite/AOT（本戦）強化 — コアコレクション配線とエントリ統一
  - Array/Map の BoxCall を NyRT ハンドルAPIに直結：
    - Array: `push`→`nyash.array.push_h`、`length/len`→`nyash.any.length_h`
    - Map: `set`→`nyash.map.set_hh`、`get`→`nyash.map.get_hh`、`has`→`nyash.map.has_hh`、`size`→`nyash.any.length_h`
  - `ny_main` を i64 戻りに統一し、`Main.main/1` を優先（既定 args は `new ArrayBox()`）。
  - Core Box 生成の安定化：`nyash.array.birth_h` / `nyash.map.birth_h` を追加し、llvmlite `new` は birth_h を優先。
  - AOT 実行確認：
    - `[1,2,3].length()` → `Result: 3`
    - `{"name":"Alice","age":25}.size()` → `Result: 2`
    - `m.has("name") ? m.get("name").length() : 0` → `Result: 5`


Quick Next (today)
- いよいよ「Nyash で書く」段階へ（Self‑Hosting 実装の着手）：
  1) ParserBox 拡張（Stage‑2 の堅牢化・回帰修正）✅ Done 2025‑09‑16
     - bool/null リテラルと空 RHS（代入/return/local）を Int(0) フォールバックで正規化。
     - simple assignment → Local 正常化を `==` 判定と共に調整。
     - 三項演算子 `cond ? a : b` を `Ternary` ノードに正規化し、自走スモーク追加。
  2) EmitterBox 拡張（JSON v0 の安定化）✅ Done 2025‑09‑16
     - `meta.usings` を常時出力（空は `[]`）。
  3) Resolver/BoxIndex の prefix メタ反映 ✅ Done 2025‑09‑16
     - `plugin_meta_by_box` を構築し、`require_prefix` / `expose_short_names` を `resolve_using_target` へ適用。
     - `NYASH_PLUGIN_REQUIRE_PREFIX` が無効でも per-plugin meta で短名禁止を検知。
  4) Parser Stage‑3 下地 ✅ Done 2025‑09‑16
     - `ParserBox.stage3_enable()` を追加し、Break/Continue/Throw/Try を JSON v0 に出力できるゲートを実装。
     - `--stage3` CLI フラグから ParserBox へ渡す導線を追加。
     - `docs/reference/architecture/parser_mvp_stage3.md` に Stage‑3 設計を記録。
  5) 自己ホスト経路で Ny 実装切替のゲート準備（現状は Python MVP 優先を維持）。
 6) テスト:
     - `source tools/dev_env.sh pyvm`
     - `NYASH_VM_USE_PY=1 ./tools/selfhost_stage2_smoke.sh`
     - `NYASH_VM_USE_PY=1 ./tools/selfhost_stage2_bridge_smoke.sh`
     - Torture（VM中心）: `(cd tests/nyash_syntax_torture_20250916 && BACKENDS="vm" NYASH_BIN=../../target/release/nyash bash run_spec_smoke.sh)`
     - LLVM Stage‑3 smoke (手動): `NYASH_LLVM_STAGE3_SMOKE=1 ./tools/llvm_smoke.sh release`
- Runner/Bridge 実行系
  - `--ny-parser-pipe` は `NYASH_PIPE_USE_PYVM=1` で PyVM に委譲（exit code 判定に統一）。
  - 自己ホスト JSON 生成は Python MVP を優先、LLVM EXE/インラインVMを段階フォールバック。
  - Runner 分割（第1弾〜第2弾）: `dispatch.rs` へ backend 分岐を集約、`tasks.rs`/`build.rs`/`demos.rs` へ職責分離。`mod.rs` を薄型化。
- LLVM Codegen リファクタ（第1弾〜第2弾）
  - `codegen/utils.rs` を新設し `sanitize_symbol`/`build_const_str_map` を抽出。
  - `codegen/function.rs` を追加し `lower_one_function` を完全移管（呼び出しは `function::lower_one_function`）。
  - 旧レガシー断片コメントを除去して軽量化。機能・出力は不変。
- MIR Builder 整理（小分割）
  - `builder/vars.rs` を追加し、Lambda の自由変数収集ロジックを外出し。
  - 既存の `LoopBuilder`/`phi` 分割方針は維持（今後 small utils を `loops.rs` に抽出予定）。

Current Status
- Stage‑2: 自己ホスト → JSON v0 → PyVM の代表スモークは緑（配列/文字列/論理/if/loop）。
- Stage‑3: 構文受理のみ完了（break/continue/throw/try/catch/finally）。現時点では JSON 降格（no‑op/Expr）で安全受理。
- Runner: Using/Resolver を前処理に統合（BoxIndex/キャッシュ/strict）。`--ny-parser-pipe` は PyVM 委譲（exit code 判定）。
- llvmlite/AOT: Array/Map の基本操作（push/get/set/has/size, length）が NyRT ハンドルAPIで動作。`ny_main` は i64 戻り・`Main.main/1` 優先で起動。
- ログ: verbose 出力の一部を `cli_v!` で共通化（引き続き適用範囲を拡大中）。

Open
- Bridge/PHI の正規化: 短絡（入れ子）における merge/PHI incoming を固定化（rhs_end/fall_bb の順序）。
- JSON v0 の拡張方針: break/continue/try/catch/finally の表現（受け皿設計 or 受理時の事前降下）。➡ `docs/reference/architecture/parser_mvp_stage3.md`
- per‑plugin meta の反映: `require_prefix/expose_short_names/prefix` を Resolver 挙動へ段階適用（導線は実装済み）。✅ 2025‑09‑16 prefix enforcement とテスト追加済み。
- `me` の扱い: MVP は `NYASH_BRIDGE_ME_DUMMY=1` の仮注入を継続（将来撤去）。
- LLVM 直結（任意）: JSON v0 → LLVM の導線追加は後回し。

- NyRT 整頓:
  - FFI ヘルパー化（handles/boxing 正規化）／birth_h→new_i64x 統合／Core Box のプラグイン事前登録／FFI エクスポートのマクロ化。
- llvmlite 整頓:
  - boxcall のテーブル駆動化、追加 API（delete/keys/values など）の段階配線。

Plan (to Self‑Hosting)
1) Phase‑1: Stage‑2 完了＋堅牢化（今ここ）
   - 正常系スモークを自己ホスト直/Bridge（PyVM）で常緑化（追加分を反映済み）。
   - 進捗ガードの継続検証（不完全入力セット）。
2) Phase‑2: Bridge 短絡/PHI 固定＋パリティ収束
   - 入れ子短絡の merge/PHI incoming を固定し、stdout 判定でスモークを緑化。
   - PyVM/llvmlite パリティを常時緑（代表ケースを exit code 判定へ統一）。
3) Phase‑3: 構文受理の拡張（完了）→ Bootstrap c0→c1→c1’
   - 受理のみ: break/continue/throw/try-catch-finally（実行意味論は降格）。
   - emit‑only で c1 を生成→既存経路にフォールバック実行、正規化 JSON 差分で等価を確認。

How to Run (dev)
- 推奨環境: `source tools/dev_env.sh pyvm`（PyVM を既定。Bridge→PyVM 直送）
- 自己ホスト（子経路 ON）: `NYASH_USE_NY_COMPILER=1`
- 安全弁: `NYASH_NY_COMPILER_TIMEOUT_MS=2000`、emit‑only 既定: `NYASH_NY_COMPILER_EMIT_ONLY=1`

Smokes
- 無限ループ防止: `./tools/selfhost_progress_guard_smoke.sh`
- 自己ホスト → Interpreter（BoxCallなし集合）: `./tools/selfhost_stage2_smoke.sh`
- 自己ホスト → JSON → PyVM（Array/String/Console 含む）: `./tools/selfhost_stage2_bridge_smoke.sh`
- 動作確認（ローカル）
  - VM: `./target/debug/nyash --backend vm apps/tmp_hello.nyash`（Result: 0）
  - LLVM モック: `./target/debug/nyash --backend llvm apps/tmp_hello.nyash`
  - PyVM/Stage‑2: `tools/pyvm_stage2_smoke.sh`（All PASS）
  - Bridge/Stage‑2: `tools/ny_stage2_bridge_smoke.sh`（All PASS）

Notes / Policies
- PyVM は意味論の参照実行器として運用（exit code 判定を基本）。
- Bridge は JSON v0 → MIR 降下で PHI を生成（Phase‑15 中は現行方式を維持）。
- 配布/バンドル/EXE 化は任意の実験導線として維持（Phase‑15 の主目的外）。

Smoke Snapshot (2025‑09‑15)
- 修正: `runner/dispatch.rs` に `vm` 分岐が欠落しており `--backend vm` が interpreter にフォールバックしていたため、PyVM スモークが作動せず。分岐を追加して復旧済み。
- PyVM Stage‑2 部分結果:
  - PASS: string ops basic, me method call
  - FAIL: loop/if/phi → 出力 `sum=4`（期待 `sum=9`）
    - 原因分析: ループ内 if の merge で `sum` の Phi 正規化が入らず、latch 側スナップショットが else 系の一時値を優先（`16`）しうる構造。`LoopBuilder::build_statement(If)` が `normalize_if_else_phi` 相当を呼ばず、変数マップが φ 統合されていない。
    - 対応方針（最小修正）:
      - LoopBuilder の If 降下で merge 到達時に「両枝が同一変数に代入」の場合は `phi(dst=[then,else])` を emit→その φ を対象変数に bind。
      - latch スナップショットはこの φ 後の変数マップで採取する。
      - 代替（短期）: Builder 側の `normalize_if_else_phi` を呼ぶ薄いフックを設けて流用。

Fixes Applied (2025‑09‑15)
- LoopBuilder If 降下に φ 正規化を追加（両枝代入の変数を merge 時に φ で束ねて再束縛）。
- PyVM φ 解決ロジックを安定化（incoming を [value, pred] 形に限定し、[pred, value] の曖昧推測を削除）。偶然一致による誤選択を排除。
- これにより `tools/pyvm_stage2_smoke.sh` は全 PASS を確認済み。

Refactor Candidates (early plan)
- runner/mod.rs（~70K chars）: “runner pipeline” を用途別に分割（TODO #15）
  - runner/pipeline.rs（入力正規化/using解決/環境注入）
  - runner/pipe_io.rs（stdin/file の JSON v0 受理・整形）
  - runner/selfhost.rs（自己ホスト EXE/VM/Python フォールバック、timeout/ログ含む）
  - runner/dispatch.rs（backend 選択と実行、PyVM 委譲）
  - 既存 json_v0_bridge/mir_json_emit は流用、mod.rs から薄いファサードに縮退。
- backend/llvm/compiler/codegen（責務分割の継続）
  - 済: utils 抽出、`lower_one_function` を `function.rs` へ移管。
  - 次: 終端系・選択系の薄層切り出し。
    - `instructions/terminators.rs`: return/jump/branch の分岐ドライバ（emit_* 呼び出し集約）。
    - `instructions/select.rs`: 条件・短絡・PHI 前処理（sealed-SSA 前提の前段正規化）。
  - 目標: `function.rs` の見通し改善（1関数=制御フロー骨格）、テスト容易化。
- mir/builder.rs（ヘッダ~80行、全体~1K行）
  - 既に多くを modules に分割済み。残る “variable/phi 合流”“loop ヘッダ/出口管理” を builder/loops.rs / builder/phi.rs に抽出。
  - 目標: 依存関係（utils/exprs/stmts）を維持したまま、1ファイル1責務を徹底。

Recommended Next (short list)
- LLVM Codegen（B 継続）
  - `instructions/terminators.rs` を新設し、`function.rs` から終端分岐（return/jump/branch）を移譲。
  - `instructions/select.rs` を新設し、条件/短絡/PHI 前処理（sealed-SSA 前提の軽い正規化）を集約。
  - `function.rs` は「BB 周回＋各 lowering 呼び出し」の骨格のみへ縮退。
- MIR Builder（C 継続）
  - `builder/loops.rs` を新設し、ループのヘッダ/出口の小物ユーティリティを抽出（`LoopBuilder` の補助レイヤ）。

Refactor Plan (MIR Core / Parser / Runner)

- MIR Core（中〜高）
  - 問題点
    - `optimizer.rs` と `verification.rs` に処理が集中し、追加パスや検証増に弱い構造（巨大化: 994/980 行規模）。
  - 提案
    - パス駆動に分割: `mir/passes/{dce.rs, ssa.rs, const_fold.rs, simplify.rs}` と `MirPass` トレイト（`run(&mut MirModule)`）。
      - 進捗: dce/cse 抽出済み、`MirPass` 骨格導入済み。次は normalize 系の抽出を段階実施。
    - `optimizer.rs` はパイプライン組立（順序・ゲート・Stats 集約）に縮小。
    - `verification.rs` もカテゴリ分割（ブロック整合性、SSA/PHI、型整合）。失敗メッセージ表現の一貫化（hintを統一）。

- Parser/Tokenizer（中）
  - 問題点
    - `parser/expressions.rs`（~986行）、`parser/statements.rs`（~562行）、`tokenizer.rs`（~863行）が肥大。
  - 提案
    - Pratt/precedence テーブル化で演算子別分岐の重複削減。
    - 共通エラー生成ユーティリティで `expected(...)` メッセージを統一。
    - `tokenizer.rs` を `tokens.rs`（定義）と `lexer.rs`（実装）に分離。テストは `tests/lexer_*.rs` へ退避。

- Runner（中）
  - 問題点
    - `runner/modes/common.rs`（~734行）、`runner/mod.rs`（~597行）に CLI/環境フラグ/実行分岐が混載。
  - 提案
    - `runner/modes/common/` ディレクトリ化し、CLI引数処理・環境フラグ解決・モードディスパッチを分離。
    - 重複ログ/検証を共通ヘルパへ集約。

  - `builder/vars.rs` に SSA 変数正規化の小物を段階追加（変数名再束縛/スコープ終端の型ヒント伝搬など）。
- Runner（仕上げ）
  - `mod.rs` の残置ヘルパ（usingの候補提示・環境注入ログ）を `pipeline/dispatch` へ集約し、`mod.rs` を最小のオーケストレーションに。
  - Namespaces Phase‑1（実装着手）: BoxIndex 構築・3段階解決・toml aliases・曖昧エラー改善・トレース

Smoke Policy (Phase‑15)
- PyVM: 一部チェックのみ（async/nowait/await/GC/sync は対象外）
- LLVM: フル対応（llvmlite harness）。`tools/smokes/curated_llvm.sh [--phi-off]` を利用
- JIT: 未整備（JIT向けスモークは `tools/smokes/archive/` に移管）
 - Stage‑3 acceptance（Bridge 経路）: `tools/ny_stage3_bridge_accept_smoke.sh`（Try/Break/Continue/Throw を JSON v0 で受理できることを確認）

Next Phase — Selfhost Parser/Compiler in Nyash（着手準備完了）
- 目的: Nyash スクリプトで Parser/Emitter を実装し、Ny → JSON v0 → Bridge → MIR 実行の自己ホスト路線に移行。
- ステップ（最小 MVP）:
  1) `apps/selfhost-compiler/` に ParserBox/EmitterBox を Nyash で実装（Stage‑2 構文、JSON v0 出力）。
  2) ランナーに `NYASH_USE_NY_COMPILER=1` ゲートを追加し、子プロセス/pipe で JSON v0 を受け取って Bridge→MIR 実行。
  3) curated LLVM スモークの一部を自己ホスト経路で通す（PyVMは非対象、LLVMで検証）。
  4) CI に自己ホスト最小ジョブを追加（timeout/静音運用、PHI‑on 既定）。
- ガード/ポリシー:
  - 既存 Rust Parser/Emitter はフォールバックとして保持（`NYASH_SKIP_TOML_ENV=1` で隔離可能）。
  - 仕様差が出た場合は LLVM 側の意味論に合わせて Nyash 実装を調整。

MIR13 Plan（Phase‑15 終盤）
- Bridge/Builder: PHI を生成しない（受理は維持）。If/Loop の合流は LLVM Resolver に任せる。
- llvmlite: Resolver を使い、BB 先頭で PHI 合成。ループは preheader/cond/body の CFG から搬送値を復元（break は exit 側でマージ）。
- Smoke: LLVM はまず loop‑only（break/continue）を常時緑化。例外系（throw/try）は IR 降ろし込み整備後に復帰。
- 詳細設計: `docs/private/papers/paper-e-loop-signal-ir/mir-evolution-plan.md` に MIR14→MIR13→MIR17 の段階的移行計画を記載。

Array/Map Literals Plan（Syntax Sugar）
- Stage‑1: Array literal `[e1, e2, ...]` を実装（ゲート: `NYASH_SYNTAX_SUGAR_LEVEL=basic|full` または `NYASH_ENABLE_ARRAY_LITERAL=1`）。
  - Lowering: `new ArrayBox()` → 各要素を評価 → `.push(elem)` を左から右に順に発行 → 最後に配列値を返す。
  - 末尾カンマ許可。
  - スモーク: `apps/tests/array_literal_basic.nyash`（size/順序/副作用1回性）。
- Stage‑2: Map literal `{ "k": v, ... }`（文字列キー限定）を実装（ゲート: `NYASH_SYNTAX_SUGAR_LEVEL=basic|full` or `NYASH_ENABLE_MAP_LITERAL=1`）。
  - Lowering: `new MapBox()` → 各ペアを評価 → `.set("k", v)` を左から右に順に発行 → 最後に map 値を返す。
  - 末尾カンマ許可。識別子キー糖 `{name: v}` は次フェーズ。
  - スモーク: `apps/tests/map_literal_basic.nyash`（size/get/順序検証）。
- Stage‑3: 識別子キー糖 `{name: v}` と末尾カンマを強化（任意）。

Gates / Semantics
- 左から右で評価（一度だけ）。push/set 失敗は即時伝播（既存 BoxCall 規約に追従）。
- IR 変更なし（BoxCall/MethodCall のみ）。将来 `with_capacity(n)` 最適化は任意で追加。

Decision Log (2025‑09‑15)
- Q: 警告削減（`ops_ext.rs` / `selfhost.rs`）を先にやる？それとも挙動スモークを先に回す？
- A: スモークを先に実施。理由は以下。
  - リファクタ直後は回帰検出を最優先（PyVM/自己ホスト/Bridge の3レーンで即座に検証）。
  - 警告削減は挙動非変化を原則とするが、微妙なスコープや保存スロットの触りが混入し得るため、先に“緑”を固める。
Namespaces / Using（現状）
- 解決順（決定性）: 1) ローカル/コア → 2) エイリアス（nyash.toml/env）→ 3) 相対/using.paths → 4) プラグイン（短名/qualified）
- 曖昧時はエラー＋候補提示（qualified または alias を要求）。
- モード切替: Relaxed（既定）/Strict（`NYASH_PLUGIN_REQUIRE_PREFIX=1` または toml `[plugins] require_prefix=true`）
- needs 糖衣は using の同義（Runner で alias 登録）。
- Plugins は統合名前空間。qualified `network.HttpClient` 常時許可。
- nyash.toml（MVP）: `[aliases]`/`[plugins]`（グローバル `require_prefix` のみ反映。per‑plugin は導線のみ）
- Index とキャッシュ（Runner）:
  - BoxIndex（グローバル）: `plugin_boxes`, `aliases` を保持。plugins init 後に構築。
  - Resolve Cache（グローバル）: `tgt|base|strict|paths` キーで再解決回避。
  - `NYASH_RESOLVE_TRACE=1`: 解決手順/キャッシュヒット/未解決候補をログ出力。

  - スモークが緑＝基礎健全性確認後に、静的ノイズの除去を安全に一気通貫で行う。

**AOT Quick**
- Array literal: `NYASH_SYNTAX_SUGAR_LEVEL=basic ./tools/build_llvm.sh tmp/aot_array_literal_main.nyash -o app && ./app`
- Map literal: `NYASH_SYNTAX_SUGAR_LEVEL=basic NYASH_ENABLE_MAP_LITERAL=1 ./tools/build_llvm.sh tmp/aot_map_literal_main.nyash -o app && ./app`

Refactoring Plan (Phase‑15 follow‑up)
- 背景: 巨大ファイル（AST/MIR/LLVM）が保守コスト増の主因。機能非変化の小刻みリファクタで視認性と変更容易性を上げる。

- 目的と範囲（非機能変更・段階適用）
  1) AST（src/ast.rs:1）: 定義とヘルパの分離、将来のサブenum活用導線の整備（現行API互換）。
  2) MIR Builder（src/mir/builder.rs:1）: build_module の骨格化・責務分離、既存分割（builder/*）の徹底。
  3) Python LLVM（src/llvm_py/llvm_builder.py:1）: lower_function の前処理/本体分離、lower_block の責務拡張。
  4) MIR 命令（src/mir/instruction.rs:1）: 構造体+トレイト導線の導入（enum への委譲を維持した非破壊移行）。

- 実施順（小PR単位、CI緑維持）
  PR‑1: AST utils 抽出（非破壊）
  - 追加: `src/ast/utils.rs` に classify/info/span/to_string などのヘルパを移設。
  - `src/ast.rs` は ASTNode/StructureNode/ExpressionNode/StatementNode の定義中心に縮退。
  - 互換維持: `pub use ast::utils::*;` で既存呼び出しを壊さない。
  - 受入: 全ビルド/スモーク緑、差分はファイル移動のみ。

  PR‑2: MIR Builder build_module 分割（非破壊）
  - `build_module` を `prepare_module`/`lower_root`/`finalize_module` に3分割。
  - 型推定（value_types→返り値）は finalize 側へ集約（現行ロジック移設）。
  - 既存の exprs/stmts などの委譲を明示し、build_module 本体を「骨格のみ」に縮退。
  - 受入: LLVM/PyVM/Bridge スモーク緑（挙動非変化）。

  PR‑3: Python LLVM lower_function の前処理抽出
  - 新設: `setup_phi_placeholders()` を導入し、PHI 宣言/CFG 依存前処理をここへ移設。
  - `lower_block()` に snapshot（block_end_values 収集）までの責務を移動。メインループは薄い周回に。
  - 受入: `tools/smokes/curated_llvm.sh` / `curated_llvm_stage3.sh` 緑。

  PR‑4: AST ラッパー導入（非破壊導線）
  - 追加: `src/ast/nodes.rs` に小さな構造体群（Assign/Return/...）。
  - enum `StatementNode/ExpressionNode` を構造体で保持。`From<T> for ASTNode` / `TryFrom<ASTNode> for T` を提供。
  - Builder 入口（`builder/stmts.rs`, `builder/exprs.rs`）で ASTNode → TryFrom 変換を 1 行追加し以降はサブ型で match。
  - 受入: ビルド/スモーク緑（機能非変化）。

  PR‑5: MIR 命令トレイト POC（Const/BinOp）
  - 追加: `src/mir/instruction_kinds/` に `const_.rs`, `binop.rs`（各命令を struct 化）。
  - 共通トレイト `InstructionMeta { effects(), dst(), used() }` を定義。
  - `MirInstruction::{effects,dst_value,used_values}` から一部を構造体 impl に委譲（match 縮退の礎）。
  - 受入: スモーク緑、`instruction_introspection.rs` の挙動非変化。

- リスクとガード
  - 機能非変化を原則（挙動差分は不可）。
  - CI で LLVM/Bridge を優先確認。Selfhost/E2E は任意ジョブで回す。
  - PR は 400 行未満/ファイル移動中心を目安に分割。

- 参考ファイル
  - AST: `src/ast.rs`, （新規）`src/ast/utils.rs`, （将来）`src/ast/nodes.rs`
  - Builder: `src/mir/builder.rs`, `src/mir/builder/*`
  - Python LLVM: `src/llvm_py/llvm_builder.py`
  - MIR 命令: `src/mir/instruction.rs`, （新規）`src/mir/instruction_kinds/*`

Acceptance Criteria
- すべての変更は機能非変化（スモーク/CI 緑）。
- 大型関数・巨大match の見通しが改善し、追従点が局所化。
- 新規追加の導線（AST サブ型/命令トレイト）は既存 API と共存し、段階移行を可能にする。
