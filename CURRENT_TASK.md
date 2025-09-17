# Current Task — Phase 15 Self‑Hosting (2025‑09‑17)

Summary
- Default execution is MIR13 (PHI‑off). Bridge/Builder do not emit PHIs; llvmlite synthesizes PHIs when needed. MIR14 (PHI‑on) remains experimental for targeted tests.
- PyVM is the semantic reference engine; llvmlite is used for AOT and parity checks.
 - GC: user modes defined; controller実装（rc+cycle skeleton + metrics/diagnostics）に移行。LLVM safepoint輸出/NyRT配線と自動挿入（envゲートON）を完了。

What Changed (recent)
- MIR13 default enabled
  - `mir_no_phi()` default set to true (can disable via `NYASH_MIR_NO_PHI=0`).
  - Curated LLVM runner defaults to PHI‑off; `--phi-on` enables MIR14 lane.
  - Added doc: `docs/development/mir/MIR13_MODE.md`; README references it.
- JSON v0 Bridge lowering refactor + features
  - Split helpers: `src/runner/json_v0_bridge/lowering/{if_else.rs, loop_.rs, try_catch.rs, merge.rs}`（既存）に加え、式系を `lowering/expr.rs` に分離（振る舞い不変）。
  - 新規サポート: Ternary/Peek の Lowering を実装し、`expr.rs` から `ternary.rs`/`peek.rs` へ委譲（MIR13 PHI‑off=Copy合流／PHI‑on=Phi 合流）。
  - Self‑host 生成器（Stage‑1 JSON v0）に Peek emit を追加: `apps/selfhost-compiler/boxes/parser_box.nyash`。
  - Selfhost/PyVM スモークを通して E2E 確認（peek/ternary）。
- llvmlite stability for MIR13（bring‑up進行中）
  - Control‑flow 分離: `instructions/controlflow/{branch,jump,while_.py}` を導入し、`llvm_builder.py` の責務を縮小。
  - プリパス（環境変数で有効化）: `NYASH_LLVM_PREPASS_LOOP=1`, `NYASH_LLVM_PREPASS_IFMERGE=1`
    - ループ検出（単純 while 形）→ 構造化 lower（LoopForm失敗時は regular while）
    - if‑merge（ret‑merge）前処理: ret 値 PHI の前宣言と finalize 配線の一意化
    - CFG ユーティリティ: `src/llvm_py/cfg/utils.py`（preds/succs）
  - PHI 配線の分離: `src/llvm_py/phi_wiring.py` に placeholder/finalize を移管（builder 薄化）
  - 値解決ポリシー共通化: `src/llvm_py/utils/values.py`（prefer same‑block SSA → resolver）
  - vmap の per‑block 化: `vmap_cur` を用意し、ブロック末に `block_end_values` へスナップショット。cross‑block 汚染を抑制。
  - Resolver 強化: end‑of‑block 解決で他ブロック PHI を安易に採用しない（自己参照/非支配を回避）。
  - BuildCtx 導入: `src/llvm_py/build_ctx.py` で lowering 引数を集約（compare/ret/call/boxcall/externcall/typeop/newbox/safepoint が ctx 対応）
  - トレース統一: `src/llvm_py/trace.py` を追加し、`NYASH_CLI_VERBOSE`/`NYASH_LLVM_TRACE_PHI`/`NYASH_LLVM_TRACE_VALUES` を一元管理
  - curated スモーク拡張: `tools/smokes/curated_llvm.sh --with-if-merge` を追加（if‑merge ケース含む）
- Parity runner pragmatics
  - `tools/pyvm_vs_llvmlite.sh` compares exit code by default; use `CMP_STRICT=1` for stdout+exit.
  - Stage‑2 smokes更新: `tools/selfhost_stage2_smoke.sh` に "Peek basic" を追加。
- GC controller/metrics（Phase‑15）
  - `GcController`（統合フック）導入＋CLI `--gc`（`NYASH_GC_MODE`）。CountingGcは互換ラッパに縮退。
  - NyRT exports: `ny_safepoint` / `ny_check_safepoint` / `nyash.gc.barrier_write` → runtime hooks 連携。
  - LLVM：自動 safepoint 挿入（loop header / call / externcall / boxcall）。`NYASH_LLVM_AUTO_SAFEPOINT` で制御（既定=1）。
  - メトリクス：text/JSON（`NYASH_GC_METRICS{,_JSON}=1`）。JSONに alloc_count/alloc_bytes/trial_nodes/edges/collections/last_ms/reason_bits/thresholds を含む。
  - 診断：`NYASH_GC_LEAK_DIAG=1` でハンドルTop‑K（残存）出力。Array/Map 到達集合の試走（gc_trace）。

- CI/DevOps（Self‑Hosting パイロット強化）
  - 追加: `.github/workflows/selfhost-bootstrap.yml`（常時） — `tools/bootstrap_selfhost_smoke.sh` を40s timeoutで実行。
  - 追加: `.github/workflows/selfhost-exe-first.yml`（任意/cron） — LLVM18 + llvmlite をセットアップし `tools/exe_first_smoke.sh` を実行。
  - スモーク堅牢化: `tools/bootstrap_selfhost_smoke.sh`/`tools/exe_first_smoke.sh` に timeout を付与。
  - JSON v0 スキーマ追加: `docs/reference/mir/json_v0.schema.json` と検証ツール `tools/validate_mir_json.py`。EXE‑first スモークに組み込み。

- LLVM crate 分離の足場（Phase‑15.6 向け）
  - 新規クレート（スキャフォールド）: `crates/nyash-llvm-compiler`（CLI名: `ny-llvmc`）。
    - `--dummy --out <o>` でダミー `.o` を生成。
    - `--in <mir.json|-> --out <o>` で MIR(JSON)→`.o` を llvmlite ハーネス経由で生成（`tools/llvmlite_harness.py`）。
  - ドキュメント追記: `docs/LLVM_HARNESS.md` に `ny-llvmc` とスキーマ検証の項を追加。

- Nyash ABI v2（TypeBox）検出の足場
  - ローダに `nyash_typebox_<Box>` シンボル検出を追加（`abi_tag='TYBX'`/`version`/`invoke_id`）し、Boxスペックへ保持（まだ実行には未使用）。

Current Status
- Self‑hosting Bridge → PyVM smokes: PASS（Stage‑2 代表: array/string/logic/if/loop/ternary/peek/dot-chain）
- PyVM core fixes applied: compare(None,x) の安全化、Copy 命令サポート、最大ステップ上限（NYASH_PYVM_MAX_STEPS）
- MIR13（PHI‑off）: if/ternary/loop の合流で Copy が正しく JSON に出るよう修正（emit_mir_json + builder no‑phi 合流）
- Curated LLVM（PHI‑off 既定）: 継続（個別ケースの IR 生成不備は未着手）
LLVM ハーネス（llvmlite/AOT）:
  - `loop_if_phi`: プリパスON＋構造化whileで EXE 退出コード 0（緑）。
  - `ternary_nested`: if‑merge プリパス＋phi_wiring で ret‑merge を構造化し、退出コード一致（緑）。

Next (short plan)
0) Refactor/Structure（継続）
   - BuildCtx 展開を完了（barrier/atomic/loopform も ctx 主経路に）
   - trace 化の残り掃除（環境直読み print を削減）
   - phi_wiring を関数分割（解析/配線/タグ付け）→ ユニットテスト追加
1) Legacy Interpreter/VM offboarding (phase‑A):
   - ✅ Introduced `vm-legacy` feature (default OFF) to gate old VM execution層。
   - ✅ 抽出: JIT が参照する最小型（例: `VMValue`）を薄い共通モジュールへ切替（`vm_types`）。
   - ✅ `interpreter-legacy`/`vm-legacy` を既定ビルドから外し、既定は PyVM 経路に（`--backend vm` は PyVM へフォールバック）。
   - ✅ Runner: vm-legacy OFF のとき `vm`/`interpreter` は PyVM モードで実行。
   - ✅ HostAPI: VM 依存の GC バリアは vm-legacy ON 時のみ有効。
   - ✅ PyVM/Bridge Stage‑2 スモークを緑に再整備（短絡/三項/合流 反映）
2) Legacy Interpreter/VM offboarding (phase‑B):
   - 物理移動: `src/archive/{interpreter_legacy,vm_legacy}/` へ移設（ドキュメント更新）。
3) LLVM/llvmlite 整備（優先中）:
   - MIR13 の Copy 合流を LLVM IR に等価反映（pred‑localize or PHI 合成）: per‑block vmap 完了、resolver/phi_wiring 強化済。
   - 代表ケース:
     - `apps/tests/loop_if_phi.nyash`: プリパスONで緑（退出コード一致）。
     - `apps/tests/ternary_nested.nyash`: if‑merge + phi_wiring で退出コード一致を継続。
   - `tools/pyvm_vs_llvmlite.sh` で PyVM と EXE の退出コード一致（必要に応じて CMP_STRICT=1）。
4) PHI‑on lane（任意）: `loop_if_phi` 支配関係を finalize/resolve の順序強化で観察（低優先）。
5) Runner refactor（小PR）:
   - `selfhost/{child.rs,json.rs}` 分離; `modes/common/{io,resolve,exec}.rs` 分割; `runner/mod.rs`の表面削減。
6) Optimizer/Verifier thin‑hub cleanup（非機能）: orchestrator最小化とパス境界の明確化。
7) GC（controller）観測の磨き込み
   - JSON: running averages / roots要約（任意） / 理由タグ拡張
   - 収集頻度のサンプリング支援
   - plugin/FFI は非移動のまま、ハンドル間接を継続
8) LLVM crate split（EXE‑first）
   - LLVM harness/builder を `nyash-llvm-compiler` crate と CLI（`ny-llvmc`）に分離（入力: MIR JSON v0 / 出力: .o/.exe）
   - `tools/build_llvm.sh` 内部を新crate APIに寄せ、Runnerからも呼べるよう段階移行
   - CI: selfhost smokes と LLVM EXE smokes を分離しアーティファクト配布線を評価

9) Nyash ABI v2 統一（後方互換なし）
   - 方針: 既存 Type‑C ABI（library‑level `nyash_plugin_invoke`）を撤退し、Box単位の TypeBox へ一本化。
   - ローダ: `nyash_typebox_<Box>` の `invoke_id(instance_id, method_id, ...)` を実行ポインタとして保持し、birth/fini も含めて統一。
   - プラグイン: 公式プラグイン（String/File/Array/Map/Console/Integer）を順次 v2 へ移行。`resolve(name)->method_id` 実装。
   - 仕様: エラー規約（OK/E_SHORT/E_ARGS/E_TYPE/E_METHOD/E_HANDLE/E_PLUGIN）・TLVタグ一覧を docs に凍結、Cヘッダ雛形（`nyash_abi.h`）を配布。
   - CI: v2専用スモークを常時化（Linux）。Windows/macOS は任意ジョブで追随。

How to Run
- PyVM reference smokes: `tools/pyvm_stage2_smoke.sh`
- Bridge → PyVM smokes: `tools/selfhost_stage2_bridge_smoke.sh`
- LLVM curated (PHI‑off default): `tools/smokes/curated_llvm.sh`
- LLVM PHI‑on (experimental): `tools/smokes/curated_llvm.sh --phi-on`
- LLVM curated with if‑merge prepass: `tools/smokes/curated_llvm.sh --with-if-merge`
- Parity (AOT vs PyVM): `tools/pyvm_vs_llvmlite.sh <file.nyash>` (`CMP_STRICT=1` to enable stdout check)
  - 開発時の補助: `NYASH_LLVM_PREPASS_LOOP=1` を併用（loop/if‑merge のプリパス有効化）。
 - GC modes/metrics: see `docs/reference/runtime/gc.md`（`--gc` / 自動 safepoint / 収集トリガ / JSONメトリクス）

Self‑Hosting CI
- Bootstrap（常時）: `.github/workflows/selfhost-bootstrap.yml`
- EXE‑first（任意）: `.github/workflows/selfhost-exe-first.yml`

LLVM Crate（試用）
- ダミー: `cargo build -p nyash-llvm-compiler --release && ./target/release/ny-llvmc --dummy --out /tmp/dummy.o`
- JSON→.o: `./target/release/ny-llvmc --in mir.json --out out.o`

Operational Notes
- 環境変数
  - `NYASH_PYVM_MAX_STEPS`: PyVM の最大命令ステップ（既定 200000）。ループ暴走時に安全終了。
  - `NYASH_VM_USE_PY=1`: `--backend vm` を PyVM ハーネスへ切替。
  - `NYASH_PIPE_USE_PYVM=1`: `--ny-parser-pipe` / JSON v0 ブリッジも PyVM 実行に切替。
  - `NYASH_CLI_VERBOSE=1`: ブリッジ/エミットの詳細出力。
- スモークの実行例
  - `timeout -s KILL 20s bash tools/pyvm_stage2_smoke.sh`
  - `timeout -s KILL 30s bash tools/selfhost_stage2_bridge_smoke.sh`

Backend selection (Phase‑A after vm‑legacy off)
- Default: `vm-legacy` = OFF, `interpreter-legacy` = OFF
- `--backend vm` → PyVM 実行（python3 と `tools/pyvm_runner.py` が必要）
- `--backend interpreter` → legacy 警告の上で PyVM 実行
- `--benchmark` → vm‑legacy が必要（`cargo build --features vm-legacy`）

Enable legacy VM/Interpreter (opt‑in)
- `cargo build --features vm-legacy,interpreter-legacy`
- その後 `--backend vm`/`--backend interpreter` が有効

Key Flags
- `NYASH_MIR_NO_PHI` (default 1): PHI‑off when 1 (MIR13). Set `0` for PHI‑on.
- `NYASH_VERIFY_ALLOW_NO_PHI` (default 1): relax verifier for PHI‑less MIR.
- `NYASH_LLVM_USE_HARNESS=1`: route AOT through llvmlite harness.
- `NYASH_LLVM_TRACE_PHI=1`: trace PHI resolution/wiring.
- `NYASH_LLVM_PREPASS_LOOP=1`: enable loop prepass (while detection/structure)
- `NYASH_LLVM_PREPASS_IFMERGE=1`: enable if‑merge (ret‑merge) prepass
- `NYASH_LLVM_TRACE_VALUES=1`: trace value resolution path

Notes / Policies
- Focus is self‑hosting stability. JIT/Cranelift is out of scope (safety fixes only).
- PHI generation remains centralized in llvmlite; Bridge/Builder keep PHI‑off by default.
- No full tracing/moving GC yet; handles/Arc lifetimes govern object retention. Safepoint/barrier/roots are staging utilities.
 - GC mode UX: keep user‑facing modes minimal (rc+cycle, minorgen); advanced modes are opt‑in for language dev.
 - Legacy Interpreter/VM は段階的にアーカイブへ。日常の意味論確認は PyVM を基準として継続。

Plugin ABI v2 updates (2025‑09‑17)
- v2 migration (TypeBox) 完了/進捗
  - 完了: FileBox / PathBox / RegexBox / MathBox / TimeBox
  - Net 完了: ClientBox / ResponseBox / RequestBox / ServerBox / SockServerBox / SockClientBox / SockConnBox
  - 既存: ConsoleBox / StringBox / IntegerBox / MapBox は v2 実装あり
- ローダ診断強化
  - `NYASH_DEBUG_PLUGIN=1` で TypeBox 未検出/ABI不一致/invoke_id未定義の詳細をログ出力
- Docs 追補
  - `docs/reference/plugin-abi/nyash_abi_v2.md` に命名規約・例（Regex/Net）を追記
  - `include/nyash_abi.h`（Cヘッダ）追加済み
- 設定/スモーク/CI
  - `nyash.toml` に各 v2 Box を登録（type_id/method_id 定義）
  - スモーク: `tools/plugin_v2_smoke.sh`（Linux常時）。全 v2 プラグインのロード確認＋簡易機能スモーク（`apps/tests/plugin_v2_functional.nyash`）
- LLVM 共通化の足場
  - `tools/build_llvm.sh` に `NYASH_LLVM_COMPILER=crate|harness` を追加（`crate` は `ny-llvmc`。JSON は `NYASH_LLVM_MIR_JSON` 指定）
  - JSON スキーマ検証を可能なら実行（`tools/validate_mir_json.py`）

Plugin ABI v2 updates (2025‑09‑17 — Python family + Net smoke + JSON emit)
- v2 migration（Python 系 完了）
  - `plugins/nyash-python-plugin`: `nyash_typebox_PyRuntimeBox` / `nyash_typebox_PyObjectBox` を追加（resolve/invoke_id 実装。既存 v1 は残存）
  - `plugins/nyash-python-parser-plugin`: `nyash_typebox_PythonParserBox` を追加（birth/parse/fini）
  - `plugins/nyash-python-compiler-plugin`: `nyash_typebox_PythonCompilerBox` を追加（birth/compile/fini）
  - `nyash.toml` に Python 系 3 ライブラリを登録（type_id: 40/41/60/61）
- Net 往復スモーク（最小）
  - 追加: `apps/tests/net_roundtrip.nyash`（Server.start→Client.get→Server.accept/respond→Client.readBody）
  - 追加: `tools/plugin_v2_smoke.sh` に Net 機能スモークを条件付きで実行（CI常時ジョブに内包）
- nyash → MIR JSON emit フラグ
  - CLI `--emit-mir-json <path>` を追加（`src/cli.rs`）。`runner.execute_mir_module` でファイル出力→即終了を実装。
  - これにより `ny-llvmc` へ JSON を直結しやすくなった（次の CI 経路で使用予定）

Plan after restart（次の計画）
- Python 系プラグインの v2 化（parser/compiler/python-plugin）
- Docs 追記（Net/Regex のメソッド表、型/戻りTLVの簡易表）
- スモーク強化
  - Net: `ServerBox.start -> Client.get -> Request.respond -> Response.readBody` の往復最小ケースを追加
  - 主要 v2 Box の軽機能（String/Array/Map/Regex/Path/Math/Time）を 1 ジョブで走らせる
- LLVM 共通化
  - `nyash` からの JSON emit コマンド/フラグ導入（`--emit-mir-json <path>` など）→ `ny-llvmc` 直結
  - CI に `ny-llvmc` 実 JSON 経路を追加（Linux 常時）
- NyRT 整理（軽）
  - TLV/エラー定数を `include/nyash_abi.h` と整合させる（ヘッダ経由参照）
  - （必要時）`nyrt_last_error()` の追加検討
