# 🎯 CURRENT TASK - 2025-08-29（Phase 10.5 転回：JIT分離=EXE専用）

Phase 10.10 は完了（DoD確認済）。アーキテクチャ転回：JITは「EXE/AOT生成専用コンパイラ」、実行はVM一本に統一。

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

### 10.2 追加アップデート（2025-08-29 PM）
- ✅ static box 内メソッドのMIR関数化に成功
  - 例: `Main.helper/1`, `Main.add/2` が独立関数として生成（MIR dumpで確認）
  - VM実行でも JIT マネージャの sites に現れる（`sites=2`）
- ✅ JITコンパイル成功
  - `Main.helper/1` が JIT コンパイルされ handle 付与（handle=1）
  - 単純算術（`Main.add/2` 等）は JIT 実行 `exec_ok=1` を確認
- ✅ ArrayBox.length() の値は正しく返る（JIT無効時に Result: 3）

- ❌ 残課題（ブロッカー）
  1) プラグイン呼び出しの JIT 実行時に Segfault 発生
     - 事象: `arr.length()` のようなプラグインメソッドで JIT 実行時にクラッシュ
     - 状態: JITコンパイル自体は成功するが実行で落ちるため、DebugBox の i64 シムイベント取得に未到達
  2) i64 シムトレース未取得
     - Segfault解消後に `DebugBox.tracePluginCalls(true)` → `getJitEvents()` で i64.start/end, tlv 等を観測予定

- ▶ 次の具体ステップ（提案）
  - [ ] Lowerer: `ArrayBox.length()` を hostcall 経路（ANY_LEN_H）から plugin_invoke 経路へ切替
        - 目的: i64 シム（`nyash_plugin_invoke3_i64`）を真正面から踏ませ、シムの前後でのcanary・TLVを観測
  - [ ] Segfault 再現最小ケースの確立と原因究明
        - 候補: 受け手 a0（param index）/ argc / TLV ヘッダの組み立て、戻りTLVのdecode、ハンドル走査の境界
  - [ ] DebugBox での i64 シムイベントログ取得（start/end, tlv, rc/out_len/canary）
  - [ ] 必要に応じて f64 シム (`NYASH_JIT_PLUGIN_F64="type:method"`) の点検

---

## 2025-08-29 PM3 再起動スナップショット（Strict/分離確定版）

### 現在の着地（Strict準備済み）
- InvokePolicy/Observe を導入し、Lowerer の分岐をスリム化
  - ArrayBox: length/get/push/set → policy+observe 経由（plugin/hostcallの一元化）
  - MapBox: size/get/has/set → 同上
  - StringBox: length/is_empty/charCodeAt → 同上
- VM→Plugin 引数整合の安定化
  - 整数は I64 (tag=3) に統一／Plugin IntegerBox は自動プリミティブ化（get）
- 予約型の安全な上書き制御
  - `NYASH_PLUGIN_OVERRIDE_TYPES=ArrayBox,MapBox`（デフォルト同値）で型別に制御
- StringBoxのpost-birth初期化
  - `new StringBox()` 直後の `length()` でsegfaultしないよう、空文字で初期化
- 特殊コメント（最小）
  - `// @env KEY=VALUE`, `// @jit-debug`, `// @plugin-builtins`, `// @jit-strict`

### Strict/分離（Fail-Fast / ノーフォールバック）
- 目的: 「VM=仕様 / JIT=コンパイル」。JITで未対応/フォールバックがあれば即コンパイル失敗
- 有効化: 実行はVM固定、JITは `--compile-native`（AOT）でのみ使用
- 仕様（現状）
  - Lowerer/Engine: unsupported>0 または compile-phase fallback>0 でコンパイル中止
  - 実行: JITディスパッチ既定OFF（VMのみ）。StrictはJITを常時JIT-only/handle-only相当で動かす
  - シム: 受け手解決は HandleRegistry 優先（`NYASH_JIT_ARGS_HANDLE_ONLY=1`）

### 再起動チェックリスト
- Build（Cranelift有効）: `cargo build --release -j32 --features cranelift-jit`
- Array（param受け）: `examples/jit_plugin_invoke_param_array.nyash` → Result: 3
- Map（E2E）: `examples/jit_map_policy_demo.nyash` → Result: 2
- String（RO）: `examples/jit_string_length_policy_demo.nyash` → Result: 0（空文字）
- Strict 観測（fail-fast動作確認）:
  - ファイル先頭: `// @jit-strict` `// @jit-debug` `// @plugin-builtins`
  - 実行: `NYASH_JIT_ONLY=1 ./target/release/nyash --backend vm <file>`
  - 期待: 未対応lowerがあれば compile失敗→JIT-onlyでエラー（フォールバックなし）

### 観測の標準手順（compile/runtime/シム）
```bash
cargo build --release --features cranelift-jit

# Array（param受け、JIT観測一式）
NYASH_USE_PLUGIN_BUILTINS=1 \
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_EVENTS_RUNTIME=1 \
NYASH_JIT_EVENTS_PATH=jit_events.jsonl NYASH_JIT_SHIM_TRACE=1 \
  ./target/release/nyash --backend vm examples/jit_plugin_invoke_param_array.nyash

# Map（policy/observe経由の確認）
NYASH_USE_PLUGIN_BUILTINS=1 NYASH_PLUGIN_OVERRIDE_TYPES=ArrayBox,MapBox \
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_EVENTS_RUNTIME=1 \
NYASH_JIT_EVENTS_PATH=jit_events.jsonl \
  ./target/release/nyash --backend vm examples/jit_map_policy_demo.nyash

# String（length RO）
NYASH_USE_PLUGIN_BUILTINS=1 NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_EVENTS_RUNTIME=1 \
NYASH_JIT_EVENTS_PATH=jit_events.jsonl \
  ./target/release/nyash --backend vm examples/jit_string_length_policy_demo.nyash

# Strictモード（フォールバック禁止）最小観測
NYASH_USE_PLUGIN_BUILTINS=1 NYASH_JIT_EXEC=1 NYASH_JIT_ONLY=1 NYASH_JIT_STRICT=1 \
NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_RUNTIME=1 NYASH_JIT_EVENTS_COMPILE=1 \
NYASH_JIT_EVENTS_PATH=jit_events.jsonl \
  ./target/release/nyash --backend vm examples/jit_plugin_invoke_param_array.nyash
```

### これからの実装（優先順）
1) 算術/比較 emit の穴埋め（Strictで落ちる箇所を優先）
2) String RO の必要最小を policy に追加（過剰に増やさない）
3) 追加サンプルは最小限（回帰用の小粒のみ）
4) 必要に応じて Strict 診断のJSONイベントを最小追加（compile-fail時）

### 現在の達成状況（✅）
- ✅ static box メソッドのMIR関数化に成功
  - 例: `Main.helper/1`, `Main.add/2` が独立関数として生成され、JITの sites に出現
- ✅ JITコンパイル成功／実行成功
  - `Main.helper/1` に handle が付与（handle=1）、`compiled=1`、`exec_ok=1`
- ✅ compile-phase イベント出力
  - `plugin:ArrayBox:push` / `plugin:ArrayBox:length`（型ヒントにより StringBox へ寄るケースも増加見込み）
- ✅ length() の正値化
  - `arr.length()` が 3 を返す（受け手解決の安全化・フォールバック整備済み）

### 既知の課題（❌）
- ❌ runtime-phase イベントが出ない環境がある
  - 対処: `NYASH_JIT_EVENTS=1` を併用（ベース出力ON）、必要に応じて `NYASH_JIT_EVENTS_PATH=jit_events.jsonl`
  - 純JIT固定: `NYASH_JIT_ONLY=1` を付与してフォールバック経路を抑止
- ❌ シムトレース（[JIT-SHIM i64]）が出ない環境がある
  - 対処: 上記と同時に `NYASH_JIT_SHIM_TRACE=1` を指定。plugin_invoke シム経路を確実に踏むため length は plugin 優先

### 直近で入れた変更（要点）
- 「型ヒント伝搬」パスを追加（箱化）
  - 追加: `src/mir/passes/type_hints.rs`、呼び出し元→callee の param 型反映（String/Integer/Bool/Float）
  - 反映: `optimizer.rs` から呼び出し、責務を分割
- length() の plugin_invoke 優先
  - BoxCall簡易hostcall（simple_reads）から length を除外、Lowerer の plugin_invoke 経路に誘導
- シムの受け手解釈を「ハンドル優先」に変更
  - `nyash_plugin_invoke3_{i64,f64}` で a0 を HandleRegistry から解決→PluginBoxV2/ネイティブ(Array/String)
  - レガシー互換のparam indexも残し、安全フォールバック
- runtime観測の強化
  - シム入り口で runtime JSON を出力（kind:"plugin", id:"plugin_invoke.i64/f64"、type_id/method_id/inst/argc）
  - ANY長さ（`nyash_any_length_h`）にparam indexフォールバックを追加

### 観測の標準手順（必ずこれで確認）
```bash
cargo build --release --features cranelift-jit

# 標準出力に compile/runtime/シムトレースを出す（純JIT固定）
NYASH_USE_PLUGIN_BUILTINS=1 \
NYASH_JIT_EXEC=1 NYASH_JIT_ONLY=1 NYASH_JIT_THRESHOLD=1 \
NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_EVENTS_RUNTIME=1 \
NYASH_JIT_SHIM_TRACE=1 \
  ./target/release/nyash --backend vm examples/jit_plugin_invoke_param_array.nyash

# もしくはruntime JSONをファイルに
NYASH_JIT_EVENTS_RUNTIME=1 NYASH_JIT_EVENTS_PATH=jit_events.jsonl \
  ./target/release/nyash --backend vm examples/jit_plugin_invoke_param_array.nyash
cat jit_events.jsonl
```

### 設計ルールの曖昧さと「箱」整理（次の箱）
- TypeHintPass（完了）: `src/mir/passes/type_hints.rs` — 型伝搬をここに集約（最小実装済）
- InvokePolicyPass（新規）: `src/jit/policy/invoke.rs` — plugin/hostcall/ANY の経路選択を一元化（Lowerer から分離）
- Observe（新規）: `src/jit/observe.rs` — compile/runtime/trace 出力の統一（ガード/出力先/JSONスキーマ）

### 今後のToDo（優先度順：分離/AOT）
1) 実行モード分離（CLI/Runner）
   - 目的: `nyash file.nyash` は常にVM実行。`--compile-native -o app` でEXE生成。
   - DoD: VM内のJITディスパッチは既定OFF。StrictはJIT=AOTで常時Fail-Fast。
2) AOTパイプライン確立（obj→exe）
   - 目的: Lower→CLIF→OBJ→`ny_main`+`libnyrt.a`リンクの一発通し
   - DoD: `tools/build_aot.sh` の内製依存をCLIサブコマンド化。Windows/macOSは後段。
3) AOT箱の追加
   - AotConfigBox: 出力先/ターゲット/リンクフラグ/プラグイン探索を管理し、apply()でenv同期
   - AotCompilerBox: `compile(file, out)` でOBJ/EXEを生成、events/結果文字列を返す
4) 観測の統一
   - 目的: `NYASH_JIT_EVENTS=1` で compile/runtime が必ず出力。PATH指定はJSONL追記
   - DoD: `jit::observe` 経由へ集約

### 受け入れ条件（DoD）
- compile-phase: `plugin:*` のイベントが関数ごとに安定
- runtime-phase: `plugin_invoke.*` が必ず出力（stdout または JSONL）
- シムトレース: `NYASH_JIT_SHIM_TRACE=1` で [JIT-SHIM …] が可視
- length(): `arr=ArrayBox([…])`→3、`s=StringBox("Hello")`→5（どちらもJIT実行時に正値）

### 備考（TIPS）
- ConsoleBox.log はVMでは標準出力に流れません。観測は `print(...)` か runtime JSON を利用してください。
- runtime JSON が見えない場合は `NYASH_JIT_EVENTS=1` を必ず併用（ベース出力ON）。


## 現在地（Done / Doing / Next）
- ✅ Done（Phase 10.10）
  - GC Switchable Runtime（GcConfigBox）/ Unified Debug（DebugConfigBox）
  - JitPolicyBox（allowlist/presets）/ HostCallのRO運用（events連携）
  - CIスモーク導入（runtime/compile-events）/ 代表サンプル整備
- 🔧 Doing（Phase 10.5 分離/AOT）
  - VM実行の既定固定（JITディスパッチは既定OFF）
  - AOT最小EXE: libnyrt.aシム + ny_main ドライバ + build_aot.sh → CLI化
  - リファクタリング継続（core_hostcall.rs→observe/policy統合）
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
  
## Phase 10.5（旧10.1）：Python統合 / JIT Strict 前倒し
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
1) JIT Strict モード（最優先）
   - // @jit-strict（ENV: NYASH_JIT_STRICT=1）で有効化
   - Lowerer: unsupported>0 でコンパイル中止（診断を返す）
   - 実行: JIT_ONLYと併用でフォールバック禁止（fail-fast）
   - シム: 受け手解決は HandleRegistry 優先（param-index 経路は無効化）
2) Array/Map のパリティ検証（strict）
   - examples/jit_plugin_invoke_param_array.nyash / examples/jit_map_policy_demo.nyash で compile/runtime/シム整合を確認
3) Python統合（RO中心）の継続
   - eval/import/getattr/call の strict 観測と整合、数値/Bool/文字列の戻りデコード

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

# Strictモード（フォールバック禁止）最小観測
NYASH_USE_PLUGIN_BUILTINS=1 NYASH_JIT_EXEC=1 NYASH_JIT_ONLY=1 NYASH_JIT_STRICT=1 \
  NYASH_JIT_EVENTS=1 NYASH_JIT_EVENTS_RUNTIME=1 NYASH_JIT_EVENTS_COMPILE=1 \
  NYASH_JIT_EVENTS_PATH=jit_events.jsonl \
  ./target/release/nyash --backend vm examples/jit_plugin_invoke_param_array.nyash

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
