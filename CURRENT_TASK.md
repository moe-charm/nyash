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
  1) ParserBox 拡張（Stage‑2 の堅牢化・回帰修正）
     - 算術/比較/論理/呼出/メソッド/if/else/loop/local/return/new の受理を再確認。
     - 代入文の正規化（`identifier = expr` → Local/Store）。
  2) EmitterBox 拡張（JSON v0 の安定化）
     - `meta.usings` の付与一貫化、配列/Map リテラルの後方対応（将来拡張の下地）。
  3) 自己ホスト経路で Ny 実装切替のゲート準備（現状は Python MVP 優先を維持）。
  4) テスト:
     - `source tools/dev_env.sh pyvm`
     - `NYASH_VM_USE_PY=1 ./tools/selfhost_stage2_smoke.sh`
     - `NYASH_VM_USE_PY=1 ./tools/selfhost_stage2_bridge_smoke.sh`
     - Torture（VM中心）: `(cd tests/nyash_syntax_torture_20250916 && BACKENDS="vm" NYASH_BIN=../../target/release/nyash bash run_spec_smoke.sh)`
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

Open
- Bridge/PHI の正規化: 短絡（入れ子）における merge/PHI incoming を固定化（rhs_end/fall_bb の順序）。
- JSON v0 の拡張方針: break/continue/try/catch/finally の表現（受け皿設計 or 受理時の事前降下）。
- per‑plugin meta の反映: `require_prefix/expose_short_names/prefix` を Resolver 挙動へ段階適用（導線は実装済み）。
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
  - `builder/vars.rs` に SSA 変数正規化の小物を段階追加（変数名再束縛/スコープ終端の型ヒント伝搬など）。
- Runner（仕上げ）
  - `mod.rs` の残置ヘルパ（usingの候補提示・環境注入ログ）を `pipeline/dispatch` へ集約し、`mod.rs` を最小のオーケストレーションに。
  - Namespaces Phase‑1（実装着手）: BoxIndex 構築・3段階解決・toml aliases・曖昧エラー改善・トレース

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
