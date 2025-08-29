# 🎯 CURRENT TASK - 2025-08-29（Phase 10.1 革新的転換）

Phase 10.10 は完了（DoD確認済）。**重大な発見**：プラグインシステムを活用したJIT→EXE実現の道を発見！

## 🚀 革新的発見：プラグインBox統一化

### 核心的洞察
- 既存のプラグインシステム（BID-FFI）がすでに**完全なC ABI**を持っている
- すべてのBoxをプラグイン化すれば、JIT→EXEが自然に実現可能
- "Everything is Box" → "Everything is Plugin" への進化

## ⏱️ 今日のサマリ（Array/Map プラグイン経路の安定化→10.2へ）
- 実装: Array/Map のプラグイン（BID-FFI v1）を作成し、nyash.toml に統合
- Lower: `NYASH_USE_PLUGIN_BUILTINS=1` で Array(len/get/push/set), Map(size/get/has/set) を `emit_plugin_invoke(..)` に配線
- サンプル: array/map デモを追加し VM 実行で正常動作確認
- 次: 10.2（Craneliftの実呼び出し）に着手

## 現在地（Done / Doing / Next）
- ✅ Done（Phase 10.10）
  - GC Switchable Runtime（GcConfigBox）/ Unified Debug（DebugConfigBox）
  - JitPolicyBox（allowlist/presets）/ HostCallのRO運用（events連携）
  - CIスモーク導入（runtime/compile-events）/ 代表サンプル整備
- 🔧 Doing（Phase 10.1 新計画）
  - NewBox→birthのJIT lowering（String/Integer、handleベース）
  - AOT最小EXE: libnyrt.aシム + ny_main ドライバ + build_aot.sh 整備
  - リファクタリング作業は継続（core_hostcall.rs完了）
- ⏭️ Next（Phase 10.1 実装）
  - Week1: 主要ビルトインBoxの移行（RO中心）
  - Week2: 静的同梱基盤の設計（type_id→nyplug_*_invoke ディスパッチ）
  - Week3: ベンチ/観測性整備（JIT fallback理由の粒度）
  - Week4: AOT配布体験の改善（nyash.toml/soの探索・ガイド）

## リファクタリング計画（機能差分なし）
1) core_hostcall 分割（イベントlower＋emit_host_call周辺）
   - 追加: `src/jit/lower/core_hostcall.rs`
   - `mod.rs`/`core.rs` のモジュール参照を更新
   - 確認: `cargo check` → `bash tools/smoke_phase_10_10.sh`
2) core_ops 分割（算術/比較/分岐）
   - 追加: `src/jit/lower/core_ops.rs`
   - CLIF配線やb1正規化カウンタは移動のみ
   - 確認: `cargo check` → 代表JITデモ2本を手動確認
3) 仕上げ
   - 1ファイル ~1000行以内目安を満たすこと
   - ドキュメント差分は最小（本CURRENT_TASKのみ更新）

### DoD（Refactor）
- `cargo check` が成功し、`tools/smoke_phase_10_10.sh` がGreen
- ログ/イベント出力がリファクタ前と一致（体感差分なし）
- `core.rs`/`builder.rs` の行数削減（目安 < 1000）

## Phase 10.1 新計画：プラグインBox統一化
- 参照: `docs/development/roadmap/phases/phase-10.1/` （新計画）
- 詳細: `docs/ideas/new-features/2025-08-28-jit-exe-via-plugin-unification.md`
- Week1（概要）
  - ArrayBoxプラグイン実装とテスト
  - JIT→Plugin呼び出しパス確立
  - パフォーマンス測定と最適化
  
## Phase 10.5（旧10.1）：Python統合
- 参照: `docs/development/roadmap/phases/phase-10.5/` （移動済み）
- ChatGPT5の当初計画を後段フェーズへ

### 進捗（10.5a→10.5b 最小実装）
- 新規: `plugins/nyash-python-plugin/` 追加（ABI v1、動的 `libpython3.x` ローダ）
- Box: `PyRuntimeBox(type_id=40)`, `PyObjectBox(type_id=41)` を `nyash.toml` に登録
- 実装: `birth/fini`（Runtime）, `eval/import`（Handle返却）, `getattr`（Handle返却）, `call`（int/string/bool/handle対応） `callKw`（kwargs対応・key:stringと値のペア） , `str`（String返却）
- 設計ドキュメント: `docs/development/roadmap/phases/phase-10.5/10.5a-ABI-DESIGN.md`

### ビルド/テスト結果（2025-08-29）
- ✅ Pythonプラグインビルド成功（警告ありだが動作問題なし）
- ✅ 本体ビルド成功（Cranelift JIT有効）
- ✅ プラグインロード成功（`libnyash_python_plugin.so` 初期化OK）
- ✅ `PyRuntimeBox.birth()` 正常実行（instance_id=1）
- ✅ VM側委譲: `PluginBoxV2` メソッドを `PluginHost.invoke_instance_method` に委譲（BoxCallでも実体plugin_invoke実行）
- ✅ E2Eデモ: `py.import("math").getattr("sqrt").call(9).str()` がVM経路で実行（`examples/py_math_sqrt_demo.nyash`）
- ✅ R系API: `evalR/importR/getattrR/callR/callKwR` がResult（Ok/Err）で安定（エラーメッセージの保持確認済）
- ✅ 自動デコード（オプトイン）: `NYASH_PY_AUTODECODE=1` で eval/getattr/call/callKw の数値/文字列/bytesがTLVで直接返る
- ✅ kwargs対応: `callKw` 実装（TLVで key:string と value のペア）、`examples/py_kw_round_demo.nyash` を追加（builtins.intで検証）

### JIT強化（10.2 連携の下ごしらえ）
- 追加: i64シムの戻りdecode拡張（I32/I64/Bool/F64[暫定]）
- 追加: f64専用シム `nyash_plugin_invoke3_f64` と `emit_plugin_invoke` 切替（ENV=`NYASH_JIT_PLUGIN_F64`）
- 目的: Python含むプラグインのROで「数値/Bool/f64（選択）」戻りをJIT/AOT経路で受ける足場を整備
 - 追加: シム・トレース（ENV=`NYASH_JIT_SHIM_TRACE=1`）とカナリー検査（出力バッファのオーバーラン検出）
 - 追加: レシーバ自動解決フォールバック（a0<0時はVM引数を走査してPluginBoxV2を特定）

### 方針決定（Built-inとの関係）
- いまはビルトインBoxを削除しない。余計な変更を避け、プラグイン優先の運用で干渉を止める。
- 例・テスト・CIをプラグイン経路に寄せ、十分に安定してから段階的に外す。

### 次アクション（小さく通す）
1) DebugBox（Phase 1）: JITシムイベント取得（getJitEvents）/プラグイン呼び出し履歴トレース（tracePluginCalls）
2) JIT typedシムの拡張: f64の自動選択（Lowerer型ヒント）→ Handle/String返却シムの導入（ハンドルID/文字列返却）
3) AOTスモーク: Python数値戻りの .o 生成をもう1本追加（f64/Bool/i64いずれか）
4) ガイド: R系APIとNYASH_PY_AUTODECODEの使い分け、JITシムトレースの使い方を追記

## すぐ試せるコマンド（現状維持の確認）
```bash
# Build（Cranelift込み推奨）
cargo build --release -j32 --features cranelift-jit

# Smoke（10.10の代表確認）
bash tools/smoke_phase_10_10.sh

# HostCall（HH直実行・read-only方針）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash
NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 \
  ./target/release/nyash --backend vm examples/jit_policy_whitelist_demo.nyash

# GC counting（VMパス）
./target/release/nyash --backend vm examples/gc_counting_demo.nyash

# compileイベントのみ（必要時）
NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS_PATH=events.jsonl \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash

# Plugin demos（Array/Map）
(cd plugins/nyash-array-plugin && cargo build --release)
(cd plugins/nyash-map-plugin && cargo build --release)
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/array_plugin_demo.nyash
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/array_plugin_set_demo.nyash
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/map_plugin_ro_demo.nyash

# Python plugin demo（Phase 10.5）
(cd plugins/nyash-python-plugin && cargo build --release)
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_eval_demo.nyash
# 追加デバッグ（TLVダンプ）
NYASH_DEBUG_PLUGIN=1 NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_eval_demo.nyash

# math.sqrtデモ（import→getattr→call→str）
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_math_sqrt_demo.nyash

# kwargsデモ（builtins.int）
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_kw_round_demo.nyash

# 自動デコード（evalの数値/文字列が直接返る）
NYASH_PY_AUTODECODE=1 NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_eval_autodecode_demo.nyash

# JIT（f64戻りのシム選択: type_id:method_id を指定）
# 例: PyObjectBox.callR(=12) を f64 扱いにする（実験用）
NYASH_JIT_PLUGIN_F64="41:12" NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_math_sqrt_demo.nyash

# kwargsデモ（builtins.int）
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/py_kw_round_demo.nyash

## AOT最小EXE（New!）
```bash
# 1) .o生成（Cranelift必須）
NYASH_AOT_OBJECT_OUT=target/aot_objects \
NYASH_USE_PLUGIN_BUILTINS=1 NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
  ./target/release/nyash --backend vm examples/aot_min_string_len.nyash

# 2) libnyrtビルド + リンク + 実行（Linux例）
(cd crates/nyrt && cargo build --release)
cc target/aot_objects/main.o -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o app
./app

# 3) ワンコマンド
bash tools/build_aot.sh examples/aot_min_string_len.nyash -o app
```

## ⏭️ Next（Phase 10.2: JIT実呼び出しの実体化）
- 目的: Craneliftの `emit_plugin_invoke` を実装し、JITでも実体のプラグインAPIを呼ぶ
- 方針:
  - シム関数 `extern "C" nyash_plugin_invoke3_i64(type_id, method_id, argc, a0, a1, a2) -> i64` を実装
    - a0: 受け手（param index／負なら未解決）
    - args: i64 を TLV にエンコードして plugin invoke_fn へ橋渡し
    - 戻り: TLV（i64/Bool）の最初の値を i64 に正規化
  - CraneliftBuilder: `emit_plugin_invoke` で上記シムを import→call（常に6引数）
  - 対象: Array(len/get/push/set), Map(size/get/has/set) の i64 1〜2引数経路
```

### 10.2 追加: AOT接続（.o出力）
- 新規: `NYASH_AOT_OBJECT_OUT=/path/to/dir-or-file` を指定すると、JITでコンパイル成功した関数ごとに Cranelift ObjectModule で `.o` を生成します。
  - ディレクトリを指定した場合: `/<dir>/<func>.o` に書き出し
  - ファイルを指定した場合: そのパスに上書き
- 現状の到達性: JITロワラーで未対応命令が含まれる関数はスキップされるため、完全カバレッジ化が進むにつれて `.o` 出力関数が増えます。
- 未解決シンボル: `nyash_plugin_invoke3_i64`（暫定シム）。次フェーズで `libnyrt.a` に実装を移し、`nyrt_*`/`nyplug_*` 記号と共に解決します。

### 10.2b: JITカバレッジの最小拡張（ブロッカー解消）
- 課題: 関数内に未対応命令が1つでもあると関数全体をJITスキップ（現在の保守的ポリシー）。`new StringBox()` 等が主因で、plugin_invoke のテストや `.o` 出力まで到達しづらい。
- 対応方針（優先度順）
  1) NewBox→birth の lowering 追加（プラグイン birth を `emit_plugin_invoke(type_id, 0, argc=1レシーバ扱い)` に変換）
  2) Print/Debug の no-op/hostcall化（スキップ回避）
  3) 既定スキップポリシーは維持しつつ、`NYASH_AOT_ALLOW_UNSUPPORTED=1` で .o 出力だけは許容（検証用途）
- DoD:
  - `examples/aot_min_string_len.nyash` がJITコンパイルされ `.o` が出力される（Cranelift有効ビルド時）
  - String/Integer の RO メソッドで plugin_invoke がイベントに現れる

### 現状の診断（共有事項）
- JITは「未対応 > 0」で関数全体をスキップする保守的設計（決め打ち）。plugin_invoke 自体は実装済みだが、関数がJIT対象にならないと動かせない。
- プラグインはVM経路で完全動作しており、JIT側の命令サポート不足がAOT検証のボトルネック。

## 参考リンク
- Phase 10.1（新）: `docs/development/roadmap/phases/phase-10.1/README.md` - プラグインBox統一化
- Phase 10.5（旧10.1）: `docs/development/roadmap/phases/phase-10.5/README.md` - Python統合
- Phase 10.10: `docs/development/roadmap/phases/phase-10/phase_10_10/README.md`
- プラグインAPI: `src/bid/plugin_api.rs`
- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`

## Checkpoint（再起動用メモ）
- 状態確認: `git status` / `git log --oneline -3` / `cargo check`
- スモーク: `bash tools/smoke_phase_10_10.sh`
- 次の一手: core_hostcall → core_ops の順に分割、毎回ビルド/スモークで確認
