# 🎯 CURRENT TASK - 2025-09-01 Snapshot（Async Task System / Phase 11.7 + Plugin-First）

このスナップショットは Phase 11.7 の Async Task System 進捗を反映しました。詳細仕様/計画は下記を参照。
- SPEC: docs/development/roadmap/phases/phase-11.7_jit_complete/async_task_system/SPEC.md
- PLAN: docs/development/roadmap/phases/phase-11.7_jit_complete/async_task_system/PLAN.md

## ✅ Async Task System（進捗サマリ）
- P1 完了: FutureBox を Mutex+Condvar化。await は safepoint + timeout でハング抑止。
  - VM/Unified: `env.future.await` は `Result.Ok(value)` / `Result.Err("Timeout")` を返却。
  - JIT: `await_h` → `result.ok_h` ラップ済。さらに `result.err_h` 追加と `Ok/Err` 分岐を Lowerer に実装。
  - 修正: FutureBox クローンを共有化（Arc<Inner>）し、spawn側のsetterと呼び出し側のFutureが同一待機点を共有。
- P2（着手・足場）: CancellationToken / TaskGroup（雛形）
  - VM 経路: `env.future.spawn_instance` を `spawn_task_with_token(current_group_token(), ..)` に配線（no-opトークン）。
  - 付随: 暗黙グループの子Futureをグローバル登録（best-effort）し、簡易joinAllの足場（global_hooks.join_all_registered_futures）。
  - TaskGroupBox: `cancelAll()`/`joinAll(ms?)` をVM BoxCallで受付（plugins-only環境では new は不可）。
  - Runner終端: `NYASH_JOIN_ALL_MS`(既定2000ms)で暗黙グループの子Futureをbest-effort join。
  - グループ/トークンはスカフォールド（Phase 2で実体実装: 伝播/キャンセル/join順序）。
- P3（第一弾）: Await の Result 化（JIT側）
  - 新規シム: `nyash.result.err_h(handle)` 追加（<=0時は `Err("Timeout")` を生成）。
  - Lowerer: `await` を `await_h → (ok_h, err_h) → handle==0 で select` に更新。

### MIR 層の設計（合意メモ）
- 原則: 「すべては箱」+「汎用呼び出し」で表現し、専用命令は最小限。
- 箱の面（MIRから見えるもの）
  - TaskGroupBox: `spawn(recv, method, args…) -> Future`, `cancelAll()`, `joinAll(timeout_ms?)`, `fini(将来)`
  - FutureBox: `await(timeout_ms?) -> Result<T, Err>`, `cancel()`, `isReady()`
  - ResultBox: 既存（Ok/Err）
- MIR表現
  - nowait: 当面は `ExternCall("env.future","spawn_instance", [recv, mname, ...])`。TaskGroup実体が固まり次第 `BoxCall TaskGroup.spawn(...)` に移行。
  - await: 既存 `MirInstruction::Await` を使用（Lowererが await 前後に `env.runtime.checkpoint` を自動挿入）。
  - checkpoint: `ExternCall("env.runtime","checkpoint")` 相当。Verifierで Await の前後必須（実装済）。
- Loweringと実装対応
  - VM: `spawn_instance`→scheduler enqueue、`Future.get()`+timeout→`Result.Ok/Err("Timeout")`、checkpointでGC+scheduler.poll
  - JIT/AOT: `await_h`→i64 handle(0=timeout)→`result.ok_h/err_h`でResult化。checkpointは `nyash.rt.checkpoint` シムに集約。
- 効果: VM/JIT/AOTが同形のMIRを見て、JIT/EXEは既存のシムで統一挙動を実現。Verifierでawait安全性を機械チェック。

### 引き継ぎ（2025-09-01, late）
- これまでに着地したもの（コード）
  - Await 正規化（JIT）: `nyash.result.err_h` 追加、`await_h → ok_h/err_h → select` で Result.Ok/Err に統一。
  - Await 安全性: Builder が await 前後に `Safepoint` 自動挿入、Verifier が前後 checkpoint 必須を検証（`--verify`）。
  - Future 共有/弱化: `FutureBox` を Arc 共有に、`FutureWeak` を追加（`downgrade()/is_ready()`）。
  - 暗黙/明示 TaskGroup 足場:
    - `global_hooks`: 強/弱レジストリ、関数スコープ `push_task_scope()/pop_task_scope()`（外側でbest‑effort join）。
    - VM: 関数入口/出口にスコープ push/pop を配線（JIT早期return含む）。
    - TaskGroupBox: `inner.strong` で子Futureを強参照所有、`add_future()` / `joinAll(ms)` / `cancelAll()`（scaffold）。
    - `env.future.spawn_instance` 生成Futureは「現スコープのTaskGroup」または暗黙グループへ登録。

- 次の実装順（小粒から順に）
  1) TaskGroupBox.spawn(recv, method, args…)->Future を実装（所有は TaskGroupBox）。
     - Builder の nowait を `BoxCall TaskGroup.spawn` に段階移行（fallback: `ExternCall env.future.spawn_instance`）。
  2) LIFO join/cancel: スコープTaskGroupをネスト順で `cancelAll→joinAll`（まずは join、次段で token 伝播）。
  3) Err 統一（P3後半）: Cancelled/Panic を Result.Err に統一（JIT/AOT：必要なら NyRT シム追加）。
  4) テスト/CI: 
     - 単体（feature-gated）: Futureの強/弱参照・join・スコープjoinの確認。
     - E2E: nowait→await（Ok/Timeout）、終端join を {vm, jit, aot}×{default, strict} でスモーク化。
     - CI に async 3本（timeoutガード付き）を最小マトリクスで追加。

- 実行・フラグ
  - `NYASH_AWAIT_MAX_MS`（既定5000）: await のタイムアウト。
  - `NYASH_JOIN_ALL_MS`（既定2000）: Runner 終端 join のタイムアウト。
  - `NYASH_TASK_SCOPE_JOIN_MS`（既定1000）: 関数スコープ pop 時の join タイムアウト。

- 参考（動かし方）
  - ビルド: `cargo build --release --features cranelift-jit`
  - スモーク: `tools/smoke_async_spawn.sh`（VM/JIT, timeout 10s + `NYASH_AWAIT_MAX_MS=5000`）
  - デモ: `apps/tests/taskgroup-join-demo/main.nyash`（スコープ/終端 join の挙動確認）

- 既知の制約/注意
  - plugins‑only 環境では `new TaskGroupBox()` は未実装（箱自体はVM側で動くが、プラグイン同梱は未）。
  - cancel はフラグのみ（次段で CancellationToken 伝播と await 時の Cancelled をErrで返す）。
  - いくつかの既存テストが赤（別領域の初期化不備）。asyncテストはfeatureゲートで段階導入予定。


### 参考コード（主要差分）
- Runtime/スケジューラ+フック
  - `src/runtime/scheduler.rs`: `spawn_with_token` を含む Scheduler スケルトン。
  - `src/runtime/global_hooks.rs`: `spawn_task_with_token`・`current_group_token()` を追加。
- TaskGroup（雛形）
  - `src/boxes/task_group_box.rs`: 取消状態のみ保持（将来の伝播に備える）。
- Await の Result 化
  - VM: `src/backend/vm_instructions.rs`（Result.Okへ包む）。
  - Unified/V2: `src/runtime/plugin_loader_{unified,v2}.rs`（`env.future.await` を Ok/Err(Timeout) で返却）。
  - JIT: `src/jit/extern/{async.rs,result.rs}`（`await_h` と `ok_h/err_h`）、`src/jit/lower/core.rs`（await分岐）、`src/jit/lower/builder.rs`（シンボル登録）。
- スモーク
  - `tools/smoke_async_spawn.sh`（timeout 10s + `NYASH_AWAIT_MAX_MS=5000`）。
  - 参考デモ: `apps/tests/taskgroup-join-demo/main.nyash`（Runner終端joinの動作確認）。

### 次の実装順（合意済み）
1) Phase 2: VMの暗黙TaskGroup配線（現状no-opトークンで着地→次にグループ実体／join/cancel実装）
2) Phase 3: JIT側のErr統一（Timeout以外: Cancelled/Panicの表出整理、0/None撤去の完了）
3) Verifier: await前後のcheckpoint検証ルール追加（実装済・--verifyで有効）
4) CI/Smokes: async系3本を最小マトリクスでtimeoutガード

---

# 🎯 CURRENT TASK - 2025-08-30 Restart Snapshot（Plugin-First / Core最小化）

このスナップショットは最新の到達点へ更新済み。再起動時はここから辿る。

## ✅ 現在の着地（実装済み）
- プラグイン仕様・ローダー（二層）
  - 各プラグインに `plugins/<name>/nyash_box.toml`（type_id/methods/lifecycle/artifacts）。
  - 中央 `nyash.toml`: `[plugins]` と `[box_types]` を利用、`[libraries]` は最小互換で維持。
  - Loader: `nyash_box.toml` 優先で type_id/メソッド解決、従来ネストへフォールバック。
- 追加プラグイン（最低限）
  - ConsoleBox: stdout 出力（log/println）。
  - Math/Time: MathBox(sqrt/sin/cos/round: f64返り)/TimeBox(now: i64)。
  - 既存: filebox/string/map/array/python/integer/counter/net。
- MIR/VM 統一
  - 新 MIR 命令 `PluginInvoke` を導入。Builder は常に `PluginInvoke` を生成（BoxCall廃止方向）。
  - VM: `execute_plugin_invoke` 実装（TLV encode/戻り decode、f64/handle含む）。Handle(tag=8)→PluginBoxV2 復元対応。
- ビルトイン制御
  - レガシーのビルトインBox工場を削除（plugins専用化）。
  - `NYASH_PLUGIN_ONLY=1` で完全プラグイン運用（既定運用）。
  - `env.console` をプラグインConsoleBoxに自動バインド（VMのref_get特例）。
  - VM側で static birth 緩和（プリミティブ受け手→プラグインBox生成）。
  - 文字列/整数の自動変換: Plugin StringBox→toUtf8()でTLV string、Plugin IntegerBox→get()でTLV i64。
- Pythonプラグイン（Phase 10.5c 足場）
  - eval/import/getattr/call/str のRO経路をVMで動作（autodecode: NYASH_PY_AUTODECODE=1）。
  - returns_result（…R系）をVMが尊重し、Ok/ErrをResultに包んで返却。
  - 追加サンプル: `py_eval_env_demo.nyash`, `py_math_sqrt_demo.nyash`, `py_result_ok_demo.nyash`, `py_result_error_demo.nyash`, `py_getattrR_ok_demo.nyash`, `py_callR_ok_demo.nyash`。
- スモーク
  - `tools/smoke_plugins.sh`: python/integer/console/math_time をVMで実行（STRICT/デフォルト）。

### ✅ AOT/ネイティブ（最小経路の到達点）
- `tools/build_aot.sh` + `crates/nyrt`: JIT→.o 生成→ libnyrt.a とリンクしEXE化に成功。
- 最小AOT例: `examples/aot_py_eval_env_min.nyash`（`NYASH_PY_EVAL_CODE` で式注入）でバイナリ生成・実行OK。
- nyrtシム強化: `nyash_plugin_invoke3_{i64,f64}` が StringBox/IntegerBox ハンドルを TLV(string/i64) に自動変換（import/getattr/call で使用可能）。
- Lowerer緩和: strict時の `new/birth/eval`（PyRuntime/Integer）を no-op 許容→未サポカウントを抑制。
- 現状のAOT結果: 未サポート命令は大幅削減（27→5→今後0を目標）。

## 🎯 次のやること（短期）
1) Python3メソッドの AOT 実Emit（最小）:
   - Lowerer: `emit_plugin_invoke` で `PyRuntimeBox.import/getattr` と `PyObjectBox.call` を直接生成（has_ret/argc 正規化）。
   - nyrtシム: 追加の型（Bool/Float/Bytes）の引数TLVパスを確認し、必要なら拡張。
2) AOTの結果出力の最小対応:
   - ConsoleBox.println の strict 経路（extern寄せ or 直接 PluginInvoke）の緩和で簡易表示を可能に。
3) returns_result サンプルの拡充:
   - importR/getattrR/callR/callKwR の OK/Err を網羅、表示体裁（Ok(...)/Err(...)）の最終化。
4) CI/Golden 更新:
   - AOT最小ルート（eval/env）と VM Python スモークを追加。将来 {vm,jit,aot} × {gc on,off} に拡張。
5) ドキュメント整備:
   - Plugin-First 運用（`NYASH_PLUGIN_ONLY=1`）、`@env` ディレクティブ、最小AOT手順と制約の明記。

### 🆕 2025-08-30 PM — Python/AOT 進捗と残タスク（引き継ぎ）
#### ✅ 到達
- eval方式（`NYASH_PY_EVAL_CODE` または `py.eval(<code>)`）で AOT unsupported=0 達成。`.o` 生成OK、Console出力OK。
- NYASH_PY_AUTODECODE=1 でプリミティブ返り（FloatBox→f64）を確認（例: 4.0）。
- Console 橋渡し（`env.console.log/println` → ConsoleBox）を strict 経路で実行可能に。
- nyrtシムで String/Integer 引数を TLV(tag=6/3) に自動変換（import/getattr/call の基盤整備）。
- 戻りバッファの動的拡張で AOT 実行時の短バッファ起因の不安定さを軽減。
 - VM: per-runtime globals 実装により `py.import("math"); py.eval("math.sqrt(16)")` が Green（autodecode=1 で 4）。
   - 例: `examples/test_py_context_sharing.nyash`（戻り値で最終結果を確認）

#### ❗ 現状の制約 / 不具合
- VM: `py.import("math")` の後に `py.eval("math.sqrt(16)")` が "name 'math' is not defined"（文脈共有が未確立）。
  - 2025-08-30 解消: PyRuntimeInstance に per-runtime globals(dict) を実装（birthで `__main__` dict 確保、import成功時にglobalsへ挿入、evalは同globalsで評価）。
- getattr/call（PyObjectBox）: AOT 実Emitはまだ限定（Lowerer が import 返りの Box 型を把握できない）。
  - 対策方針（更新）: Python特化の型伝搬を撤廃し、Handle-First で汎用化。戻りが `box` のメソッドは「Handle（TLV tag=8）」として扱い、Lowerer は `emit_plugin_invoke` のみ（箱名固定を行わない）。必要に応じて by-name シムで実行時解決。

#### 🎯 次タスク（実装順・更新済）
1) 設計ドキュメント反映（最優先）
   - `phase-10.5/10.5c-handle-first-plugininvoke-plan.md` を追加（完了）。
   - MASTER_ROADMAP からの導線追記（別PRで可）。
2) Lowerer 汎用化（Python特化排除）
   - Python固有の型伝搬（dst=PyObjectBox 記録）を撤去し、戻りが `box` の場合は Handle として扱う（型名固定なし）。
   - `emit_plugin_invoke` は従来どおり使用（has_ret/argc 正規化）。
3) メタデータ解決
   - `PluginHost.resolve_method` に `returns.type` を露出。Lowerer が `box`/primitive のみを参照。
4) by-name シムの導入（必要時）
   - `nyrt`/builder に `nyash_plugin_invoke_by_name_{i64,f64}` を追加し、受け手箱名未確定時の実行時解決に使用。
5) AOT 実行の安定化
   - nyrt シム: Bytes/Bool/Float/複数引数 TLV のカバレッジ拡大。
   - 連鎖（import→getattr→call）の最小AOT例を Green（unsupported=0）。
6) ドキュメント/サンプル更新
   - Handle-First のガイドと最小AOT手順の追記。

### 10.5c ドキュメント/サンプル 追加（本スナップショット）
- FFI最小仕様（a0/a1/a2, 戻りTLV）を短文化: `docs/reference/abi/ffi_calling_convention_min.md`
- birth引数の一般化メモ（可変長TLV/例外伝搬）: `docs/ideas/new-features/2025-08-30-birth-args-tlv-generalization.md`
- Python最小チェーンの追加:
  - VM: `examples/py_min_chain_vm.nyash`
  - AOT: `examples/aot_py_min_chain.nyash`

### 10.5d AOT統合 仕上げ（今回）
- ガイド追加: `docs/guides/build/aot_quickstart.md`（CLI/スクリプト/内部フロー/FAQ）
- by-nameシム整理: nyrtの `nyash_plugin_invoke_name_{getattr,call}_i64` をFFI要約へ反映
- スモーク更新: `tools/smoke_aot_vs_vm.sh` に Python最小チェーンを追加（VM側は `NYASH_PY_AUTODECODE=1`）
- 今後: nyrtシムのTLV拡充（bytes/N引数）、Windowsのプラグイン探索微調整

### 10.5e 小仕上げ（今回）
- nyrtシム TLV拡充: BufferBox→bytes(tag=7) 自動エンコード、3引数目以降はレガシー引数からTLVに詰める暫定N引数対応
- Windows探索調整: EXE起動時に PATHへexe/`plugins/`を追加、`PYTHONHOME` 未設定時は `exe\python` を自動採用（存在時）。相対PYTHONHOMEはexe基準に正規化

### 次フェーズ: 10.6（Thread-Safety / Scheduler）
- 計画: docs/development/roadmap/phases/phase-10.6/PLAN.txt（新規）
- 10.6a 監査: Array/Map/Buffer/NyashRuntime/Scheduler の共有戦略（Arc+RwLock/Mutex）を確認し、未整備箇所をTODO化
- 10.6b スケジューラ: SingleThreadScheduler を Safepoint で `poll()` 連携（観測: `NYASH_SCHED_DEMO/TRACE/POLL_BUDGET`）
- 10.6c 並列GC設計: per-thread roots / safepoint協調 / カードマーキングの段階導入メモ確定

### 橋渡し: 10.7 Python Native（トランスパイル / All-or-Nothing）
- 方針と計画: docs/development/roadmap/phases/phase-10.7/PLAN.txt（新規）
- 二本立て明確化: 実行系（現行PyRuntimeBox）と トランスパイル系（Python→Nyash→MIR→AOT）を併走。自動フォールバック無し
- サブフェーズ: C1 Parser（1週）→ C2 Compiler Core（2週）→ C3 CLI配線（3日）→ C4 テスト（並行）
- 既存導線の活用: 生成Nyashは既存 `--compile-native` でAOT化（Strict）

### Nyash-only パイプライン（作業場 / 最小導線）
- 目的: すべてNyashで書き、即実行・即修正できる足場を先に用意
- 追加ファイル: tools/pyc/
  - PythonParserNy.nyash（PyRuntimeBox経由で ast.parse/dump。NYASH_PY_CODE を参照）
  - PyIR.nyash（IR最小ヘルパ）/ PyCompiler.nyash（Nyash側コンパイラ骨組み）/ pyc.nyash（エントリ）
- Parser/Compiler Rustプラグインは雛形として併存（将来削減）。当面はNyash実装を優先

### 次の順番（小粒で進める）
1) Parser JSON→IR 変換の最小実装（def/return）。tools/pyc/PyCompiler.nyash に追加（env NYASH_PY_CODE を Pythonで解析→IR生成）
2) IR→Nyash 生成の最小拡張（Return定数→Return文字列/数値に対応、If/Assignは後続）
3) All-or-NothingのStrictスイッチ（unsupported_nodes 非空ならErr）。開閉はenvで制御
4) CLI隠しフラグ `--pyc/--pyc-native` を追加し、Parser→Compiler→AOT を一本化（内部で現行AOTを使用）
5) サンプル/回帰: tools/pyc の最小ケースをVM/AOTで回し、差分を記録

### Python AOTサンプルの追加（最小）
- `examples/aot_py_min_chain.nyash`（import→getattr→call）
- `examples/aot_py_result_ok.nyash`（returns_result: Ok）
- `examples/aot_py_result_err.nyash`（returns_result: Err）
- kwargs暫定ブリッジ（env eval + **dict）: `examples/aot_py_eval_kwargs_env.nyash`

## 🔧 実行方法（再起動手順）
```bash
cargo build --release --features cranelift-jit
# プラグインをビルドし、VMスモーク
bash tools/smoke_plugins.sh
# 厳格（ビルトイン無効）2ndパス
NYASH_SMOKE_STRICT_PLUGINS=1 bash tools/smoke_plugins.sh
```

## 📌 方針（ChatGPT5助言に基づく抜粋）
- Coreは Box/意図/MIR だけ（演算・コレクション等は全部プラグイン）。
- 呼び出しは常に `plugin_invoke(type_id, method_id, argv[])`（VM/JIT共通）。
- フォールバックなし: 未実装は即エラー（場所とVM参照関数名を出す）。
- MIR 生成の一本化: 既存の built-in 特例を削除し、必ず `MIR::PluginInvoke` に落とす。
- VM/JIT の特例削除: if (is_string_length) 等の分岐は撤去。
- 静的同梱: profile=minimal/std/full で nyplug_*.a/.lib をバンドル（動的読込は将来の nyplug.toml）。
- バージョン整合: 起動時に Core v0 ⇔ 各 `nyash_plugin_abi()` 照合（ミスマッチ即終了）。
- テスト/CI: 各プラグインに Golden（VM→JIT→AOTの trace_hash 一致）、CIマトリクス {vm,jit,aot} × {gc on,off}。

### 箱を固める（Box-First 原則の再確認）
- 問題は必ず「箱」に包む: 設定/状態/橋渡し/シムは Box/Host/Registry 経由に集約し、境界を越える処理（TLV変換・型正規化）は1箇所に固定。
- 目的優先で足場を積む: 先に no-op/strict緩和で「落ちない足場」を作り、次に値の実Emit・型/戻りの厳密化を段階導入。
- いつでも戻せる: `@env`/env変数/featureフラグで切替点を1行に集約。実験→可視化→固定化のサイクルを高速化。


---

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

## 2025-08-29 PM3 再起動スナップショット（Strict/分離・ネイティブ基盤固め・Python準備）

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
1) ネイティブ基盤の仕上げ（10.5b）
   - `tools/build_aot.{sh,ps1}` の導線統一、Windows clang/cl内蔵化の検討
   - プラグイン解決の安定（拡張子変換/lib剥がし/検索パス/警告整備）
2) プラグイン仕様分離（中央=nyash.toml / 各プラグイン=nyash_box.toml）
   - Loaderが `plugins/<name>/nyash_box.toml` を読み、type_id/メソッドIDを反映
   - 旧[libraries]も後方互換で維持（当面）
3) Python統合（10.5c）
   - PyRuntimeBox/PyObjectBox のRO経路（eval/import/getattr/call/str）をVM/EXEで安定
   - autodecode/エラー伝搬の強化、WindowsでのDLL探索（PYTHONHOME/PATH）
4) 観測・サンプル
   - EXEの `Result:` 統一、VM/EXEスモークのGreen化
   - 追加サンプルは最小限（回帰用の小粒のみ）

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

---

### 新規フェーズ（提案）: Phase 10.11 Builtins → Plugins 移行
- 目的: 内蔵Box経路を段階的に廃止し、プラグイン/ユーザーBoxに一本化する（不具合の温床を解消）
- 現在の足場（済）:
  - ConsoleBox コンストラクタをレジストリ委譲（プラグイン優先）に変更
  - `NYASH_DISABLE_BUILTINS=1` でビルトインFactory登録を抑止可能
  - 設計ドキュメント: docs/development/roadmap/phases/phase-10.11-builtins-to-plugins.md
- 次ステップ:
  - 非基本コンストラクタの委譲徹底（Math/Random/Sound/Debugなど）
  - 主要ビルトインの plugin 化（nyash_box.toml 整備）
  - CIに `NYASH_USE_PLUGIN_BUILTINS=1` / `NYASH_PLUGIN_OVERRIDE_TYPES` のスモークを追加

---

## 引き継ぎ（Phase 11.9 / 統一文法アーキテクチャ + JIT分割）

現状サマリ（実装済み）
- 統一文法スキャフォールド
  - build時コード生成: `build.rs` → `src/grammar/generated.rs`
    - `KEYWORDS`（最小）と `OPERATORS_ADD_COERCION`, `OPERATORS_ADD_RULES` を生成
    - TOML未整備でも add 既定規則を生成側で補完
  - エンジン: `src/grammar/engine.rs`（`is_keyword_str`/`add_coercion_strategy`/`add_rules`/`decide_add_result`）
  - Tokenizerに非侵襲差分ログ（`NYASH_GRAMMAR_DIFF=1`）
- Add 規則の非侵襲導入
  - JIT: `lower_binop(Add)` で grammar ヒントをイベント出力
  - VM/Interpreter: 期待と実際の型を差分ログ（`NYASH_GRAMMAR_DIFF=1`）
  - オプトイン強制適用（挙動変更は未既定）: `NYASH_GRAMMAR_ENFORCE_ADD=1`
- スナップショットテスト
  - `tests/grammar_add_rules.rs`（grammar 期待 と 現行セマンティクスの一致検証）→ 単体実行で緑

JIT分割 進捗（継続観点）
- 完了: builder分割（`builder/cranelift.rs`）、core 第一段階分割（`core_ops.rs`、`core/analysis.rs`、`core/cfg.rs`）
- jit-direct スモーク緑（debug）: mir-branch-ret=1 / mir-phi-min=10 / mir-branch-multi=1

使い方（開発時）
- 差分ログ: `NYASH_GRAMMAR_DIFF=1`（Tokenizer/VM/Interp/JIT各所）
- 規則強制: `NYASH_GRAMMAR_ENFORCE_ADD=1`（Add のみ、他は非侵襲）
- JITスモーク例: `NYASH_JIT_THRESHOLD=1 ./target/debug/nyash --jit-direct apps/tests/mir-branch-ret/main.nyash`
- テスト（本件のみ）: `cargo test -q --test grammar_add_rules`

次のTODO（優先順）
1) JITロワラー分割の続き
   - 大きい分岐（Extern/PluginInvoke/BoxCall）を `src/jit/lower/core/ops_ext.rs` へ抽出
   - 各ステップごとに jit-direct スモーク確認
2) 統一文法の拡張
   - operators: Sub/Mul/Div の `type_rules` を TOML → 生成 → VM/Interp/JIT に非侵襲ログ（必要なら `*_ENFORCE_*`を用意）
   - keywords/alias/context の雛形を TOML 化（差分ログ継続）
3) スナップショット整備
   - add 以外の演算子でも「grammar期待 vs 実際」の表テストを追加
   - 将来、Tokenizer/Parser でも「grammar期待 vs 実際構文」のスナップショットを追加

注意
- 既存の他テストには未整備部分があり全体 `cargo test` は赤が出るため、当面は個別テスト/スモークを推奨
- Release の jit-direct 実行は `--features cranelift-jit` が必要

## Update: Phase 11.9 – 統一文法アーキテクチャ（MVP導入計画）

目的: Tokenizer/Parser/Interpreter/MIR/VM/JIT の解釈差異を解消するため、単一の文法・意味・実行定義を導入（詳細は `docs/development/roadmap/phases/phase-11.9/unified-grammar-architecture.md` と `docs/development/roadmap/phases/phase-11.9/PLAN.md`）。

直近TODO（M1/M2のMVP範囲）
- [ ] scaffolding: `build.rs` + `src/grammar/{mod.rs,engine.rs}` + `src/grammar/generated.rs`（codegen方式）
- [ ] `grammar/unified-grammar.toml` 初期化（keywords: `me`,`from`,`loop`; operators: `add`）
- [ ] Tokenizer に `engine.is_keyword()` を差し込み（`NYASH_GRAMMAR_DIFF=1` で差分ログ）
- [ ] `ExecutionSemantics` に `operators.add` を実装し、Interpreter/VM/JIT へ薄く統合（既存実装はフォールバック）
- [ ] 予約語マッピングの一貫性テストと、加算セマンティクスの VM/JIT/Interpreter 一致テスト

備考
- ランタイム I/O は避け、TOML→生成コードに変換して起動/ホットパスへの影響を最小化
- プラグイン拡張は将来の統合対象（優先度・名前空間・競合検知を設計）

## Progress: JIT Lowering リファクタ状況（11.8/12系）

完了
- [x] builder 分割（`src/jit/lower/builder.rs` を薄いハブ化、`builder/cranelift.rs` へ移動）
- [x] jit-direct の最小スモーク安定（debug）：
  - apps/tests/mir-branch-ret → 1
  - apps/tests/mir-phi-min → 10
  - apps/tests/mir-branch-multi → 1
- [x] core.rs の第一段階分割：
  - `src/jit/lower/core_ops.rs` にヘルパー移設（push_value_if_known_or_param, cover_if_supported, BinOp/Compareなど）
-  - `src/jit/lower/core/analysis.rs` 追加（Bool/PHI推論＋統計）
-  - `src/jit/lower/core/cfg.rs` 追加（PHI受け口順序とCFGダンプ）

次の分割候補
- [ ] Extern/PluginInvoke/BoxCall 周辺の肥大化した分岐を `core/ops_ext.rs` に整理
- [ ] `analysis`/`cfg` の補助関数（succ_phi_inputs など）の関数化
- [ ] 分割ごとに jit-direct スモークの緑維持（debug / release+feature）
