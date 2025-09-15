# Current Task — Phase 15 Self‑Hosting (2025‑09‑15)

TL;DR
- 目標は「自己ホスティング達成」＝ Nyash製パーサで Ny → JSON v0 → Bridge → MIR 実行を安定化すること。
- PyVM は意味論の参照実行器（開発補助）。llvmlite は AOT/検証。配布やバンドル化は後回し（基礎固めが先）。

What Changed (today)
- ParserBox 強化（Stage‑2 完了 + Stage‑3 受理を追加）
  - 進捗ガード（parse_program2/parse_block2/parse_stmt2）。
  - Stage‑2 受理一式: 単項/二項/比較/論理/呼出/メソッド/引数/if/else/loop/using/local/return/new。
  - 代入文（identifier = expr）を受理（Stage‑2 は Local に正規化）。
  - Stage‑3 受理のみ（意味論降下は後続）: break/continue/throw/try-catch-finally を no-op/Expr に降格して受理。
- Smokes 追加/拡充
  - Stage‑2: `tools/selfhost_stage2_smoke.sh`（自己ホスト直）・`tools/selfhost_stage2_bridge_smoke.sh`（自己ホスト→JSON→PyVM）。
  - JSON 固定ベクトル: `tests/json_v0/*.json` と `tools/pyvm_json_vectors_smoke.sh`（arith/if/while/logical/strings 代表）。
  - 新規 apps/tests: try/finally 合成、短絡＋PHI、二重ループ独立性、ループ片側PHI、メソッドチェーン。
  - Stage‑3 受理確認: `tools/selfhost_stage3_accept_smoke.sh`（受理のみを確認、実行意味論は降格）。
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
- Stage‑3: 構文受理のみ完了（break/continue/throw/try/catch/finally）。現時点では JSON 降格（no-op/Expr）で安全受理。
- Runner: `--ny-parser-pipe` で PyVM 委譲（exit code 判定）。自己ホスト JSON は Python MVP/EXE/VM の3段フォールバックで生成可能。

Open
- Bridge/PHI の正規化: 短絡（入れ子）における merge/PHI incoming を固定化（rhs_end/fall_bb の順序）。
- JSON v0 の拡張方針: break/continue/try/catch/finally の表現（受け皿設計 or 受理時の事前降下）。
- `me` の扱い: MVP は `NYASH_BRIDGE_ME_DUMMY=1` の仮注入を継続（将来撤去）。
- LLVM 直結（任意）: JSON v0 → LLVM の導線追加は後回し。

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
