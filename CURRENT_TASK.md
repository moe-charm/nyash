# CURRENT TASK (Phase 11.7 kick-off: JIT Complete / Semantics Layer)

Phase 11.7 へ仕切り直し（会議合意）

- 単一意味論層: MIR→Semantics→{VM/Cranelift/LLVM/WASM} の設計に切替。VMは参照実装、実行/生成はCodegen側で一本化。
- フォールバック廃止: VM→JITの実行時フォールバックは行わない。JITは“コンパイル専用/AOT補助”に限定。
- 共通ABI: handle/i64/ptr 変換、to_bool/compare、タグ分類、invoke（固定/可変）、NyRTシム呼び出しを共通化。
- Cranelift: 軽量JIT/AOTパスを追加し、最小実装からMIR-15を段階的に緑化。LLVM AOT は併存。

Docs: docs/development/roadmap/phases/phase-11.7_jit_complete/{README.md, PLAN.md, CURRENT_TASK.md, MEETING_NOTES.md}

以降は下記の旧計画（LLVM準備）をアーカイブ参照。スモークやツールは必要箇所を段階で引継ぎ。

Update (2025-09-01 AM / JIT handoff follow-up)

- Cranelift 最小JITの下地は進捗良好（LowerCore→CraneliftBuilder 経路）
  - Compare/Branch/Jump、最小Phi（block params）、StackSlotベースの Load/Store は実装済み（builder.rs/core.rs）。
  - ExternCall(env.console.log/println) の最小橋渡し（ConsoleBox.birth_h→by-id invoke）を追加済み。
- 追加スモーク（jit-direct 用／I/Oなし）
  - apps/tests/mir-store-load: x=1; y=2; x=x+y; return x → 3
  - apps/tests/mir-phi-min: if(1<2){x=10}else{x=20}; return x → 10
  - apps/tests/mir-branch-multi: 入れ子条件分岐 → 1
  - 実行例: `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct apps/tests/mir-store-load/main.nyash`
- jit-direct 分岐/PHI 根治（進行中）
  - 現象: select/compare の実行時観測では cond=1/then=1/else=0 と正しいが、最終結果が 0 に落ちるケースあり。
  - 統合JIT（`--backend cranelift`）は期待どおり→LowerCore の意味論は正しく、jit-direct のCFG/合流が疑わしい。
  - 主因（仮説→確度高）: 関数共有の value_stack をブロック間で使い回し→分岐/合流で返値取り違え。

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
  - Fast-path（読みやすさ＆単純化）: then/else が定数 return の場合、`select(cond, K_then, K_else)`→`emit_return` に縮約（`NYASH_JIT_FASTPATH_SELECT=1` で強制）。

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
     - 調査方針: execute_statement(Return)→execute_function_call の伝播経路、Variable 解決/共有の箇所を追跡し、Box 値ではなく内部ID/インデックスを返している箇所を特定・修正。
     - 対応: Return 直前と関数エピローグでの実値/型ログ（限定ログ）を差し込み、最小修正。
  2) PluginBox 同士の演算の包括対応
     - 暫定は toString→数値/文字列へ正規化で回避。恒久対応は Semantics/VM と同じ規約（handle-first + 文字列like/数値like）に寄せる。
  3) 文字列連結の広範囲対応
     - toString()/toUtf8/Result.Ok の内包を最優先で文字列正規化（現状の強化で多くは通る。追加ケースが出れば順次取り込み）。

- 次アクション（Interpreter fix plan）
  - [ ] semantics_test_branch 再現→Return 値の実体化ルート修正
  - [ ] simple_test の演算パスを toString 正規化で網羅（必要なら算術 toNumber も）
  - [ ] test_string_concat の失敗パターン収集→ try_box_to_string の対象拡張
  - [ ] SemanticsVM/ClifAdapter パリティ小スモーク追加（分岐/配列/extern/await）

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
