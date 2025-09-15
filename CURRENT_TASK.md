# Current Task — Phase 15 Self‑Hosting (2025‑09‑15)

TL;DR
- 目標は「自己ホスティング達成」＝ Nyash製パーサで Ny → JSON v0 → Bridge → MIR 実行を安定化すること。
- PyVM は意味論の参照実行器（開発補助）。llvmlite は AOT/検証。配布やバンドル化は後回し（基礎固めが先）。

What Changed (today)
- ParserBox 強化（apps/selfhost-compiler/boxes/parser_box.nyash）
  - 進捗ガードを追加（parse_program2/parse_block2/parse_stmt2）: 位置非前進なら 1 文字強制前進して gpos 更新（無限ループ防止）。
  - Stage‑2 ヘルパ実装: starts_with_kw/i2s/read_ident2/read_string_lit/add_using。
  - 単項マイナス（unary）を 0−expr で構文化済み。論理（&&/||）/比較/呼出/メソッド/引数/if/else/loop/using/local/return を受理。
- Smokes 追加（自己ホスト集中）
  - `tools/selfhost_progress_guard_smoke.sh`（不完全入力でもハングしないことを検証）。
  - `tools/selfhost_stage2_smoke.sh`（自己ホスト → Interpreter で基本文法 E2E）。
  - `tools/selfhost_stage2_bridge_smoke.sh`（自己ホスト → JSON → PyVM で Array/String/Console を含めた E2E）。

Current Status
- 自己ホスト Stage‑2 サブセットは Ny → JSON v0 まで通る。Interpreter 経路で BoxCall を使わない集合は E2E 緑。
- Array/String/Console などの BoxCall を含む集合は Bridge→PyVM 経路で実行・検証。
- Runner: `NYASH_USE_NY_COMPILER=1` で自己ホスト経路 ON（子プロセス JSON v0→Bridge→MIR 実行）。

Open
- 短絡（&&/||）の入れ子: Bridge の merge/PHI incoming をログ基準で固定化（rhs_end→merge の incoming を `(rhs_end,rval)/(fall_bb,cdst)` に正規化）。
- `me` の扱い: MVP は `NYASH_BRIDGE_ME_DUMMY=1` で仮注入（将来撤去）。
- Stage‑2 正常系の網羅: nested call/method/new/var/compare/logical/if/else/loop の代表強化。

Plan (to Self‑Hosting)
1) Phase‑1: Stage‑2 完了＋堅牢化（今ここ）
   - 正常系スモークを自己ホスト直/Bridge（PyVM）で常緑化。
   - 進捗ガードの継続検証（不完全入力セット）。
2) Phase‑2: Bridge 短絡/PHI 固定＋パリティ収束
   - 入れ子短絡の merge/PHI incoming を固定し、stdout 判定でスモークを緑化。
   - PyVM/llvmlite パリティを常時緑（代表ケースを exit code 判定へ統一）。
3) Phase‑3: Bootstrap c0→c1→c1’
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

