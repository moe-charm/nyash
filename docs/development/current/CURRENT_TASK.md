# 🎯 CURRENT TASK - 2025-08-27（Phase 10_b → 10_c → 10_4/10_6横串）

フェーズ10はJIT実用化へ！Core-1 Lowerの雛形を固めつつ、呼出/フォールバック導線を整えるよ。

## 🆕 Quick Hot Update（2025-08-27-2）
- JitStatsBox: perFunction() 追加（関数単位の統計をJSON配列で取得）
- b1 PHI格子 強化（Compare/Const/Cast(TypeOp)/Branch/Copy/PHI/Store/Loadの固定点伝播）
- 独立JIT: `--jit-direct` に引数対応（`NYASH_JIT_ARGS` をMIR型にcoerce）、結果の型名も表示
- b1 ABI切替の下地: `NYASH_JIT_ABI_B1_SUPPORT=1` でcapを強制ONできるように（将来のtoolchain更新に備えたスイッチ）
- リンク: JSONスキーマ(v1) → `docs/reference/jit/jit_stats_json_v1.md`

## ⏱️ 今日のサマリ（10_c実行経路の堅牢化＋10_7分岐/PHI/独立ABI＋GC/スケジューラ導線）
- 目的: JIT実行を安全に通す足場を仕上げつつ、GC/スケジューラ導線を整備し回帰検出力を上げる。
  - 10_c: panic→VMフォールバック（`catch_unwind`）/ JIT経路のroot区域化 / Core-1 i64 param minimal pass（`emit_param_i64` + LowerCoreで供給）✅ 完了
  - 10_4a/10_4b: GC導線 + Write-Barrier挿入（ArraySet/RefSet/BoxCall）、root API（enter/pin/leave）
  - 10_4c: CountingGcカウンタ出力＋roots/BoxRef内訳、depth2リーチャビリティ観測（`NYASH_GC_TRACE=1/2/3`）✅ 完了
  - 10_4d: STRICTバリア検証（CountingGc前後比較で漏れ即検出）✅ 完了（`NYASH_GC_BARRIER_STRICT=1`）
  - 10_6b: シングルスレ・スケジューラ（spawn/spawn_after/poll）、Safepointで`poll()`連携、`NYASH_SCHED_POLL_BUDGET`対応 ✅ 完了
  - 10_7: JIT分岐配線（Cranelift）— MIR Branch/Jump→CLIFブロック配線、条件b1保持、分岐b1/`i64!=0`両対応（feature: `cranelift-jit`）
  - 10_7: 最小PHI（単純ダイアモンド）— `NYASH_JIT_PHI_MIN=1` で有効、ブロック引数で合流値を受け渡し ✅ 初期対応
  - 10_7: JIT独立ABI導入 — `JitValue`（i64/f64/bool/handle） + VMValue↔JitValueアダプタ、TLS分離（legacy VMArgs / JIT Args）✅ 完了
  - ベンチ: CLIベンチへJIT比較追加（ウォームアップあり）、`branch_return`ケース追加、スクリプト版`examples/ny_bench.nyash`追加（TimerBoxでops/sec）

### 🆕 追加アップデート（10_7進捗）
- ハンドルレジストリ + スコープ管理（JIT呼び出し単位で自動クリーンアップ）✅
- HostCallブリッジ（ハンドル経由・read-only中心）✅
  - Array/Map/String: `length`/`isEmpty`、Map: `has`、String: `charCodeAt`
  - Array/Map get/set/push/size: 既存PoCをハンドル経由に段階移行
- 多値PHIの配線（then/else/jumpで順序つき引数渡し）✅
  - DOT可視化（`NYASH_JIT_DOT=path.dot` でCFG出力）✅ 初期対応
  - 引数ネイティブ型（10_7hの前段）: MIRシグネチャから `Float→F64`, `Bool→I64(0/1)` を反映（`NYASH_JIT_NATIVE_F64/BOOL`）✅ 初期対応
- ダンプの簡潔化（CFG/PHI要点表示、f64/boolネイティブ有効フラグ表示）✅
- CFG/PHIダンプの詳細化（`NYASH_JIT_DUMP=1`）✅ 初期強化
  - 例: `phi: bb=3 slots=2 preds=1|2` に続けて `dst vX <- pred:val, ...` を列挙（合流値の源が一目で分かる）
- JIT設定の集約（Rust側）✅ 初期導入
  - `src/jit/config.rs` に `JitConfig` を追加し、RunnerからCLIと環境変数を一元反映
- JitConfigBox 追加（10_7f）✅ 初期対応
  - `new JitConfigBox()` で `exec/stats/dump/phi_min/hostcall/.../threshold` を操作し `apply()` でenv反映
  - `toJson()/fromJson()` で再現性確保、`summary()` で状態確認
- f64ネイティブ最小経路（`NYASH_JIT_NATIVE_F64=1`）✅
  - f64定数、F64同士の四則、F64戻りシグネチャ・トランポリン
  - トランポリンは`desired_ret_is_f64`を捕捉（env依存のズレを排除）✅
- Boolネイティブ最小経路（`NYASH_JIT_NATIVE_BOOL=1`）↗ 準備済み
  - 比較はb1で保持し分岐に直接供給（既定）／戻りは当面i64(0/1)で正規化（将来b1戻りに拡張）
- 細かな修正: P2PBoxでtransportコールバック登録時の借用期間エラーを修正（lock解放順を明示）
- Bugfix: Cranelift `push_block_param_i64_at` で `switch_to_block` 未呼出によるpanicを修正（CFG/PHI経路の安定化）✅
 - LowerCore小型足場追加 ✅
   - ret_boolヒント: 返り値がBoolの関数で `builder.hint_ret_bool(true)` を呼ぶ切替点を用意（現状no-op、将来1行で切替）
   - Castの最小下ろし: `I::Cast` を値供給＋既知整数の継承で安全通過（副作用なし領域）
 - 観測強化 ✅
   - b1正規化カウンタ: `b1_norm_count`（分岐条件・b1ブロック引数の正規化で加算）
   - JitStatsBox: `{abi_mode, abi_b1_enabled, abi_b1_supported, b1_norm_count}` を `toJson()` で取得可能（BoxFactory登録済み）
   - 統合JIT統計（テキスト/JSON）に `abi_mode/…/b1_norm_count` を追加
 - 便利CLI ✅
   - `--emit-cfg <DOT_FILE>` 追加（`NYASH_JIT_DOT` のCLI版。`NYASH_JIT_DUMP`も自動オン）
- 重要: Cranelift未有効時の挙動を是正 ✅
  - 以前: `NYASH_JIT_EXEC=1` でJITスタブが0を返すことがあり、結果が変わり得た
  - 変更: Cranelift未有効では「未コンパイル扱い」にしてVMへ完全フォールバック（JIT有効でも結果は不変）

- 独立JITモード ✅ 追加
  - `--jit-direct`: VM実行ループを介さず、JITエンジンで `main` を直接コンパイル実行（Cranelift有効時）
  - `--jit-only`: フォールバック禁止（JIT未コンパイル/失敗は即エラー）— 独立性検証に最適

- b1 PHIの型/格子解析 強化 ✅
  - Compare/Const Bool起点＋Copy/PHIに加え、Load/Storeヒューリスティックを導入（固定点伝播）
  - PHI統計は常時計測（`phi_total_slots`/`phi_b1_slots`）→ JSON/ダンプ一致

- 関数単位JIT統計 ✅
  - LowerCore→JitEngine→JitManagerで `phi_total/phi_b1/ret_bool_hint/hits/compiled/handle` を集約
  - `JitStatsBox.summary()` に `perFunction` 配列を追加（関数ごとの明細）

- f64 E2Eと簡易ベンチ（Cranelift向け）✅ 追加
  - 例: `examples/jit_f64_e2e_add_compare.nyash` / ベンチ: `examples/ny_bench_f64.nyash`
  - 注意: VM単体ではf64演算/比較は未対応。`--features cranelift-jit` + `NYASH_JIT_EXEC=1 NYASH_JIT_NATIVE_F64=1` で実行

- JSONスキーマの文書化 ✅
  - 参照: `docs/reference/jit/jit_stats_json_v1.md`（version=1）
  - 統一JSON/Box JSONともに `version` フィールドを付与

- f64 E2Eと簡易ベンチ（Cranelift向け）✅ 追加
  - 例: `examples/jit_f64_e2e_add_compare.nyash`
  - ベンチ: `examples/ny_bench_f64.nyash`
  - 注意: VM単体ではf64演算/比較は未対応。`--features cranelift-jit` + `NYASH_JIT_EXEC=1 NYASH_JIT_NATIVE_F64=1` で実行

- JSONスキーマの文書化 ✅ 追加
  - 参照: `docs/reference/jit/jit_stats_json_v1.md`（version=1）
  - 統一JSON/Box JSONともに `version` フィールドを付与

- PHI統計（常時計測）✅
  - `phi_total_slots`/`phi_b1_slots` をLowerCoreで常時計測し、JSONに出力

- Cranelift互換性修正 ✅
  - b1シグネチャを使用せず、戻りはi64に正規化（`select(b1,1,0)`）。`types::B1`/`bint` 非依存でビルド安定化

### 直近タスク（小さく早く）
1) 10_b: Lower/Core-1 最小化（進行中 → ほぼ完了）
   - IRBuilder抽象 + `NoopBuilder`（emit数カウント）✅ 完了
   - `CraneliftBuilder` 雛形（feature `cranelift-jit`）✅ 完了
   - LowerCore（Const/Copy/BinOp/Cmp/Branch/Ret）✅ 完了（emit→Builder）
   - Engine.compile: builder選択（feature連動）＋Lower実行＋JIT handle発行✅ 完了
   - JIT関数テーブル（stub: handle→ダミー関数）✅ 完了
   - 残: 最小emit（const/binop/ret）をCLIFで生成し、関数ポインタをテーブル登録（feature有効時）
     → 実装: CraneliftBuilderでi64用の`const/binop/ret`を生成し、JIT関数テーブルへクロージャとして登録完了（i64引数0〜6個対応／bool,f64はi64正規化）✅ 更新
2) 10_c: 呼出/フォールバック（最小経路）✅ 完了
   - VM側の疑似ディスパッチログ（compiled時/実行時ログ）✅
   - JIT実行→`VMValue`返却、panic時VMフォールバック ✅（`engine.execute_handle`で`catch_unwind`）
   - Core-1最小: i64 param/return、Const(i64/bool→0/1)、BinOp/Compare/Return ✅
   - HostCall最小（Array/Map: len/get/set/push/size）ゲート`NYASH_JIT_HOSTCALL=1` ✅
   - Branch/JumpはCranelift配線導入済み（feature `cranelift-jit`）。副作用命令は未lowerのためVMへフォールバック
3) 10_7: 分岐/PHI/独立ABI（Cranelift）— 進捗中
   - LowerCore: BB整列・マッピング→builderの`prepare_blocks/switch/seal/br_if/jump`呼出 ✅
   - CraneliftBuilder: ブロック配列管理、`brif/jump`実装、条件b1/`i64!=0`両対応 ✅
   - 最小PHI（単純ダイアモンド）導入（`NYASH_JIT_PHI_MIN=1`ガード）✅ 初期対応
   - JIT独立化: `JitValue` ABI + 変換アダプタ + TLS分離でVM内部へ非依存化 ✅ 完了
   - 残: 副作用命令の扱い方針（当面VMへ）、CFG可視化の追加改善（ブロック引数の整形など）
    - 検証: `--features cranelift-jit` 有効ビルドで `examples/jit_branch_demo.nyash` / `jit_phi_demo.nyash` がJIT経路で正常動作を確認（`NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1`）✅

### 🆕 追加（2025-08-28）
- JitStatsBox: `perFunction()` をVMディスパッチへ実装（JSON配列で name/hits/phi_total/phi_b1/ret_bool_hint/compiled/handle を返却）✅
- PHI(b1)読出しの安定化（最小）✅
  - `NYASH_JIT_PHI_MIN=1` 有効時、ブールと推定されたPHIは b1 としてブロック引数を読み出し（分岐に直結）
  - 分岐条件をブール格子のシードに追加（pre-scanで `condition` をbooleanとして扱う）
- デモ更新: `examples/jit_stats_summary_demo.nyash` に `perFunction()` 出力を追加 ✅
- jit-direct（独立JIT）足場の強化 ✅
  - read-onlyガード（WriteHeap検出で明示エラー）/ 統一エラーJSON（`NYASH_JIT_ERROR_JSON=1`）
  - 最小E2E: `examples/jit_direct_local_store_load.nyash` / `jit_direct_bool_ret.nyash` / `jit_direct_f64_ret.nyash`
  - 参考: Box-First運用キット `docs/engineering/box_first_enforcement.md`（PR/CIはアドバイザリ導入）

## 🔜 次：Phase 10.9 Builtin-Box JIT（Box-Firstで最小から）

目的: VMとJITでビルトインBoxの読み取り系を同等動作にし、後続（生成/書き込み）へ進む足場を固める。

### 必要な箱（最小セット）
- JitPolicyBox（ポリシー箱）
  - 役割: read-onlyゲート/書き込み検出/HostCall許可リストの一本化（1箇所で切替）
  - 経路: runner/jit-direct/engine/lower が参照（env直読みはしない）
- JitEventsBox（観測箱）
  - 役割: compile/execute/fallback/trapを JSONL へAppend（{fn, abi_mode, reason, ms}）
  - 経路: 既存statsと共存。回帰の目視を容易に。
- HostcallRegistryBox（read-only）
  - 役割: 許可HostCallと引数/戻りの種別を宣言。型検査の単一点にする。
  - 初期: String.length/isEmpty/charCodeAt, Array.length/isEmpty/get, Map.size/has
- FrameSlotsBox（スロット箱）
  - 役割: ptr→slot の管理と型注釈（今はi64のみ）。LowerCoreから委譲。
- CallBoundaryBox（呼出し境界箱／薄い）
  - 役割: JIT↔JIT/JIT↔VM呼出しの単一点（将来の多関数対応の足場）。今は型変換の箱だけ用意。

最小原則: まず箱を置く（no-op/ログだけでもOK）→ 切替点が1箇所になったら機能を足す。

### 実装計画（小さい積み木で順に）
1) Week 10.9-α（足場）
   - JitPolicyBox v0: read-only/HostCall whitelist を箱へ移動（runnerの散在チェックを統合）
   - JitEventsBox v0: compile/executeのJSONLイベント（オプトイン）
   - CURRENT_TASK/CLAUDEへポリシーと使い方を簡潔追記
2) Week 10.9-β（読み取り系カバレッジ）
   - HostcallRegistryBox v0: String/Array/Mapの読み取りAPIを登録・型検査
   - LowerCore: BoxCallのread-only経路をRegistry参照に切替（散在ロジックの削減）
   - E2E: length/isEmpty/charCodeAt/get/size/has（jit-direct + VMで一致）
3) Week 10.9-γ（生成の足場）
   - CallBoundaryBox v0: JIT→VM（new演算子など）を一旦VMへ委譲する薄い箱
   - new StringBox/IntegerBox/ArrayBox の最小経路（jit-directは方針次第で拒否でも可）
4) Week 10.9-δ（書き込みの導線のみ）
   - JitPolicyBox: write許可のスイッチを箱に実装（既定OFF）
   - LowerCore: 書き込み命令はPolicy参照で拒否/委譲/許可（1箇所で判断）

### 成功判定（DoD）
- 機能: 読み取り系ビルトインBoxがJIT経路でVMと一致（上記API）
- 箱: Policy/Events/Registryが1箇所で参照される（散在ロジック解消）
- 観測: JSONLイベントが最低1件以上出力（明示opt-in）
- 回帰: `jit-direct` read-only方針が維持され、拒否理由がイベント/JSONで判読可

### リスクと箱での緩和
- HostCallの型崩れ: HostcallRegistryBoxで型検査→不一致はPolicy経由で明示拒否
- 方針変更の揺れ: JitPolicyBoxのフラグだけで切替可能（コード散在を回避）
- 観測不足: JitEventsBoxのイベント粒度を先に用意（必要最小限でOK）


備考（制限と次の着手点）
- 返り値はi64（VMValue::Integer）に限定。f64/boolは設計着手（ダンプで有効フラグ観測のみ）
- 引数はi64最小パス。複数引数はparamマッピングで通過、非i64はハンドル/正規化で暫定対応
- Branch/JumpのCLIF配線は導入済み（feature `cranelift-jit`）。条件はb1で保持し、必要に応じて`i64!=0`で正規化
- 副作用命令（print等）はJIT未対応のためVMへ委譲（安全性優先）
- JIT/VM統合統計（フォールバック率/時間の一括出力）✅ 実装済み（最小）
  - `maybe_print_jit_unified_stats()` により、テキスト/JSON（`NYASH_JIT_STATS_JSON=1`）でsites/compiled/hits/exec_ok/trap/fallback_rate/handlesを出力

## 📦 箱作戦（JIT独立化の方針）
- ABIの箱: `JitValue`（I64/F64/Bool/Handle）でJIT入出力を統一、VM型に非依存
- 変換アダプタ: VMValue↔JitValueはアダプタに集約、境界は必ずアダプタ経由
- TLS分離: legacy VMArgs（既存HostCall用）とJIT Args（新ABI）を分離し段階移行
- HostCall独立化（計画）: Handle(u64) + ハンドルレジストリ導入でJIT→ホストを完全独立（VM型非参照）
- 設定の箱（進捗）: Rust側で`JitConfig`を導入しenv/CLI設定を集約（Nyash側Box化は後続）
- 可視化: `NYASH_JIT_DUMP=1`でCFG/ブロック引数を可視化し回帰検出力を強化

### すぐ試せるコマンド
```bash
cargo build --release -j32
NYASH_JIT_STATS=1 NYASH_JIT_DUMP=1 ./target/release/nyash examples/p2p_ping_pong.nyash

# 疑似実行パスを確認（まだVMフォールバック）
NYASH_JIT_STATS=1 NYASH_JIT_DUMP=1 NYASH_JIT_EXEC=1 \
  ./target/release/nyash examples/p2p_ping_pong.nyash

# （任意）Craneliftを含めてビルド（今は最小初期化のみ）
cargo build --release -j32 --features cranelift-jit

# JIT分岐デモ（feature有効時）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
  ./target/release/nyash --backend vm examples/jit_branch_demo.nyash
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 \
  ./target/release/nyash --backend vm examples/jit_loop_early_return.nyash

# スクリプトベンチ（TimerBox版）
./target/release/nyash examples/ny_bench.nyash
./target/release/nyash --backend vm examples/ny_bench.nyash
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 ./target/release/nyash --backend vm examples/ny_bench.nyash

# CLIベンチ（ウォームアップ＋3バックエンド比較）
./target/release/nyash --benchmark --iterations 200
```

#### HostCall（ハンドルPoC）
```bash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_array_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_map_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_map_int_keys_param_call.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_map_has_int_keys.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_string_param_length.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_string_is_empty.nyash
./target/release/nyash --backend vm --jit-exec --jit-hostcall examples/jit_string_charcode_at.nyash
```

## 現在の地図（Done / Next）

### ✅ 完了（Phase 10.10）
- **GC Switchable Runtime**: GcConfigBox実装（counting/trace/barrier_strictフラグ制御）
- **Unified Debug System**: DebugConfigBox実装（JIT events/stats/dump/dot設定の一元管理）
- **Box Method Dispatch**: GcConfigBox/DebugConfigBoxのメソッドディスパッチ実装
- **JitPolicyBox**: ホワイトリストとプリセット機能実装（mutating_minimal/common）
- **ANCP設計**: AI-Nyash Compact Notation Protocol設計完了（50-90%トークン削減）

### ✅ 完了（Phase 10.9）
- **HostCall統合**: Registry v0実装、Policy/Events連携
- **HH経路**: Map.get_hh実装（Handle,Handle引数でJIT直実行）
- **RO/WO分離**: read-only既定、mutatingはopt-in（policy_denied_mutatingイベント）
- **Math関数**: sin/cos/abs/min/max等のF64署名一致時のallow実装

### ✅ 完了（Phase 9.79b）
- TypeMeta/Thunk正式化・Poly-PIC（2〜4）・Plugin TLV拡張（bool/i64/f64/bytes）
- VM fast-path整備（Instance/Plugin/Builtin）と統計サマリ強化

### ⏭️ 次（Phase 10）
- 10_a: JITブートストラップ ✅ 完了
- 10_b: Lower(Core-1) – Const/Move/BinOp/Cmp/Branch/Ret（最小emit仕上げ中）
- 10_c: ABI/呼出し – JIT→JIT/JIT→VM、例外バイアウト（実行経路を実体化）
- 10_d: コレクション基礎 – Array/Mapブリッジ ✅ 完了（param経路）
- 10_e: BoxCall高速化 – Thunk/PIC直結
- 10_f: TypeOp/Ref/Weak/Barrier（最小）
- 10_g: 診断/ベンチ/回帰
- 10_h: 硬化・最適化調整

## 参考リンク
- フェーズ10ロードマップ: `docs/development/roadmap/phases/phase-10/phase_10_cranelift_jit_backend.md`
- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`
- VM/Thunk/PIC: `docs/development/roadmap/phases/phase-9/phase_9_79b_3_vm_vtable_thunks_and_pic.md`

## Parking Lot（後でやる）
- Lower emitのテスト雛形
- CLIFダンプ/CFG表示（`NYASH_JIT_DUMP=1`）
- VM `--vm-stats` とJIT統計の統合
  → 実装済み（VM終了時にJIT統合サマリ出力。JSON出力も可）

### 残タスク（箇条書き）
- 10_c:
  - CLIF: Branch/Jumpの実ブロック配線、Compare結果の適用確認
  - 返り値/引数の型拡張（bool/f64）、複数引数の網羅
  - JIT/VM統合統計（フォールバック率/時間の一括出力）✅ 済（最小）
- 10_4c:
  - リーチャビリティ観測の深さ拡張（depth=2→N）と軽量ダンプ
  - （将来）実Mark/TraverseのPoC（解放はしない）
- 10_4d:
  - STRICTモードのCI導入（CountingGc前提）/ goldenベンチ導入
- 10_6b:
  - スケジューラ: poll予算の設定ファイル化、将来のscript API検討（継続）
- 10_7:
  - Hostハンドルレジストリ導入（JIT側はHandleのみを見る）
  - IRBuilder APIの整理（block param/分岐引数の正式化）とCranelift実装の安定化（暫定APIの整形）
  - 副作用命令のJIT扱い（方針: 当面VMへ、将来はHostCall化）
  - CFG検証と`NYASH_JIT_DUMP=1`でのCFG可視化（ブロックと引数のダンプ）✅ 初期強化済み（更なる整形・網羅は継続）
  - JitConfigBoxで設定の箱集約（env/CLI整合、テスト容易化）✅ Rust側の`JitConfig`導入済（Box化は後続）
  - Bool戻り最小化: `hint_ret_bool`をCranelift側に配線し、b1→i64とb1ネイティブを切替可能に
  - Compare/BinOpの副作用なし領域の拡大（Copy/Const/Castは開始済み、And/Orは当面VMフォールバック）
  - PHI(b1)の扱い整理: ブロック引数のb1対応を正式化（i64 0/1正規化とb1の両立）
  - 統計/ダンプの整備: `b1_norm_count`のJSON/テキスト両方での検証手順をドキュメント化
  - b1 PHI格子の精度UP（簡易エイリアス/型注釈の取り込み）
  - ネイティブb1 ABI: ツールチェーンcap連動で返り/引数b1署名への切替（切替点は一本化済）
  - VMのf64演算/比較パリティ（10.8へ移送）
  - JitStatsBox: `perFunction()` の箱API追加（summaryを介さず単体取得）
  - b1 PHIタグの頑健化（Load/Store越しの由来推定を型/格子解析で補強）
  - VMのf64演算/比較パリティ（10.8へ移送予定）
  - JSONスキーマのversion管理とサンプル更新（`v1`整理済み、今後の拡張計画をドキュメント化）
- ベンチ:
  - `examples/ny_bench.nyash`のケース追加（関数呼出/Map set-get）とループ回数のenv化

### 🛠️ メンテ・ビルドノート（環境依存）
- WSL配下（`/mnt/c/...`）で `cargo build` 時に `Invalid cross-device link (os error 18)` が発生する場合あり
  - 対策: リポジトリをLinux FS側（例: `~/work/nyash`）に配置／`CARGO_TARGET_DIR` と `TMPDIR` を同一FS配下へ設定
  - 備考: リンク動作とFS差分が原因のため、コード修正ではなく環境調整で解決
- Codex CLIの実行権限・サンドボックス設定忘れで「ビルドできない」誤検知になる場合あり
  - 対策: `codex --ask-for-approval never --sandbox danger-full-access` を付与して起動する（この指定忘れが原因だったケースあり）

---

10_d まとめ（完了）
- HostCall導線（Import+Call）と引数配線（i64×N→i64?）を実装
- C-ABI PoC（Rust内）: `nyash.array.{len,push,get,set}` / `nyash.map.size`
- Param経路のE2E確認（配列/Mapを関数引数に渡す形でlen/sizeが正しく返る）
- セーフティ: `NYASH_JIT_HOSTCALL=1`ゲート運用、問題時はVMフォールバック

移管（10_e/10_fへ）
- ハンドル表PoC（u64→Arc<Box>）でローカルnew値もHostCall対象に
- 型拡張（整数以外、文字列キーなど）
- BoxCallカバレッジ拡張とデオプ/フォールバック強化
- 10.9-β 進捗チェックポイント（2025-08-28-夜）
  - 完了: Policy/Events α（既存）/ Registry v0（最小）/ HostcallRegistryBox 追加
  - 接続: ハンドル系HostCallに `registry + policy + events` を暫定接続（mutatingはfallback、ROはallowログ）
  - Lower: `NYASH_JIT_HOSTCALL` → `jit::config::current().hostcall` に置換（env直読の排除）
  - E2E（追加サンプル）:
    - 成功: `examples/jit_hostcall_len_string.nyash`（String.length → allow）
    - 失敗: `examples/jit_hostcall_array_append.nyash`（Array.push → fallback）
    - 境界: `examples/jit_hostcall_math_sin_mismatch.nyash`（math.sinにi64 → sig_mismatch相当のfallbackイベント）
  - 次: Registryに署名（args/ret）を持たせ、唯一の切替点で `sig_mismatch` を厳密化／math.* のROブリッジ薄接続

### ⚠️ リカバリ/再起動チェックリスト（短縮）
- ビルド: `cargo build --release --features cranelift-jit`
- 主要フラグ: 
  - `NYASH_JIT_EXEC=1`（JIT実行有効）
  - `NYASH_JIT_THRESHOLD=1`（即JIT）
  - `NYASH_JIT_EVENTS=1`（JSONLイベント標準出力）
  - （任意）`NYASH_JIT_EVENTS_PATH=target/nyash/jit-events.jsonl`
- 代表サンプル:
  - 成功: `./target/release/nyash --backend vm examples/jit_hostcall_len_string.nyash`
  - 失敗: `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_array_append.nyash`
  - 境界: `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_math_sin_mismatch.nyash`
  - 署名一致(allow観測): `NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_hostcall_math_sin_allow_float.nyash`
  - 関数スタイル(math.*): `NYASH_JIT_NATIVE_F64=1 NYASH_JIT_EVENTS=1 ./target/release/nyash --backend vm examples/jit_math_function_style_sin_float.nyash`
    - 追加: `cos/abs/min/max` それぞれ `examples/jit_math_function_style_*.nyash`
- うまく動かない時:
  - `--features cranelift-jit` が付いているか確認
  - `NYASH_JIT_EVENTS=1` でイベントJSONを確認（fallback/trap理由が出る）
  - `cargo clean -p nyash-rust` → 再ビルド
  - 数値緩和: `NYASH_JIT_HOSTCALL_RELAX_NUMERIC=1` で i64→f64 のコアーションを許容（既定は `native_f64=1` 時に有効）

### ✅ 10.9-β HostCall統合（完了）
- math.*（関数スタイル）: 署名（F64）一致時に allow/sig_ok をイベント出力。戻りは Float 表示で安定。
- Map.get:
  - 受け手=param, key=I64 → `id: nyash.map.get_h`（Handle,I64）で allow かつ JIT実行
  - 受け手=param, key=Handle → `id: nyash.map.get_hh`（Handle,Handle）で allow かつ JIT実行（HH経路）
  - 受け手がparamでない場合 → fallback（`reason: receiver_not_param`）をイベントに記録し VM 実行
- RO系（length/isEmpty/charCodeAt/size）: 受け手=param は allow/sig_ok。受け手≠param は fallback/receiver_not_param を必ず記録。
- Quick flags: `NYASH_JIT_EVENTS=1` のとき Runner が未指定の `NYASH_JIT_THRESHOLD` を自動で `1` に設定（Lowerが1回目から走ってイベントが出る）。
- 代表コマンド:
  ```bash
  # math.min（関数スタイル）
  NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_NATIVE_F64=1 NYASH_JIT_EVENTS=1 \
    ./target/release/nyash --backend vm examples/jit_math_function_style_min_float.nyash

  # Map.get（パラメータ受け＋Handleキー → HH直実行）
  NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
    ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash

  # Map.get（非パラメータ受け → fallback記録）
  NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
    ./target/release/nyash --backend vm examples/jit_hostcall_map_get_handle.nyash
  ```

### ⏭️ 10.9-δ（書き込みの導線のみ：次）
- 方針: 既定は read-only（`policy.read_only=true`）。書き込みは fallback（`reason: policy_denied_mutating`）をイベントで可視化。（実装済）
- 対象: Array.push/set, Map.set/delete など。イベントIDは `*_h`（Handle受け）に統一。
- 仕上げ: `JitPolicyBox` に opt-in 追加（完了）
  - `set("read_only", true|false)` / `setWhitelistCsv("id1,id2")` / `addWhitelist("id")` / `clearWhitelist()`
  - プリセット: `enablePreset("mutating_minimal")`（Array.push_h） / `enablePreset("mutating_common")`（Array.push_h, Array.set_h, Map.set_h）

---

## 🚀 Phase 10.10 – Python→Nyash→MIR→VM/Native 実用化整備（着手）

目標（DoD 抜粋）
- DebugConfig（Box+Rust）で dump/events/stats/dot/phi_min 等の切替を一本化（CLI/env/Box同期）
- GC切替（Null/Counting）をCLI/Boxから操作可能、root/bARRIERサイト観測が動く
- HostCall（RO/一部WO）が param でallow、非paramでfallback（イベントで確認可）
- 代表サンプル最小集合でE2E安定

着手タスク（In Progress）
1) DebugConfigBox（新規）
   - setFlag(name,bool)/setPath(name,string)/apply()/summary()（env反映）
   - 対応: NYASH_JIT_EVENTS/…/DUMP/…/DOT（最小）
2) GC切替のCLI/Box連携（Pending）
3) 代表サンプル/README整備（Pending）

---

### ⏸️ Checkpoint（2025-08-28 再起動用メモ）
- 10.9-β/δ 完了（RO allow+fallback可視化、WOはpolicy/whitelistでopt-in）
- HH経路（Map.get_hh）実装済み。戻りはCallBoundaryBoxでBoxRef復元。
- JitPolicyBox: addWhitelist/clearWhitelist/enablePreset（mutating_minimal/common 等）
- DebugConfigBox: JIT観測系（events/stats/dump/dot）をBoxで一元制御→apply()でenv反映
- GcConfigBox: counting/trace/barrier_strict をBoxで操作→apply()でenv反映
- 例:
  - jit_map_get_param_hh.nyash（HH直実行）
  - jit_policy_optin_mutating.nyash（mutatingのopt-in）
  - gc_counting_demo.nyash（CountingGcの可視化、VMパス）

### ✅ 完了（2025-08-28 late 整理）
- Runner統合（DebugConfig/CLI/Envの整合）
  - DOT指定時にdump暗黙ON、観測系ON時のしきい値auto=1（未指定時）
- GCドキュメントに GcConfigBox 使用例を追記（短文 + コマンド）
- examples/README.md 最小手順を整備（HH直実行・mutating opt-in・CountingGc）
- DebugConfigBox拡張（最小）: `jit_events_compile/runtime`, `jit_events_path` を追加（`apply()`でenv反映）
- JIT Eventsのphase分離（lower/execute）とAPI導入（compileは明示opt-in）
- サンプル追加: `examples/jit_policy_whitelist_demo.nyash`（read_only→whitelist／events分離）
- runtime trapイベント出力（JIT実行失敗時に1行JSONL）
- CIスモーク導入: runtime系（3本）＋ compile-events（phase:"lower"検証）

⏭️ 次（Phase 10.10 続行・箱は増やさない）
- ANYヘルパ運用で十分（length_h / is_empty_h）→ 追加なし、Box-Firstの最小原則を維持
- whitelist回帰の軽量確認は `jit_policy_whitelist_demo.nyash` で実施（CI移行は後段）
- 将来候補: `nyash.any.type_h` は Phase 11 で型特化最適化検討時に導入可

### 🔍 Smoke（Phase 10.10 最小確認）
- スクリプト: `tools/smoke_phase_10_10.sh`
  - 実行: `bash tools/smoke_phase_10_10.sh`
  - 内容: Build → HH直実行 → mutating opt-in → GCデモ

代表コマンド（HostCallはNYASH_JIT_HOSTCALL=1を明示）
```
# Build（Cranelift込み推奨）
cargo build --release -j32 --features cranelift-jit

# HH直実行（Map.get_hh）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash

# mutating opt-in（policy whitelist）
NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 \
  ./target/release/nyash --backend vm examples/jit_policy_whitelist_demo.nyash

# GC counting（VMパス）
./target/release/nyash --backend vm examples/gc_counting_demo.nyash

# compileのみ（必要時）
NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS_PATH=events.jsonl \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash
```

## ✅ Phase 10.10 完了（DoD確認）
- DebugConfig/Box/CLIの一貫挙動（apply後の即時反映）: OK
- GC切替とバリアサイト観測が可能: OK（CountingGc/TRACE/STRICT）
- HostCall（RO/一部WO）: paramでallow／非paramでfallback（イベントで確認）: OK
- 代表サンプルが examples/README の手順で成功（CIスモークもgreen）: OK
- イベントスキーマ（v0.1）文書化・trap出力: OK

⏭️ 次（Phase 10.1 着手）
- Python統合（chatgpt5_integrated_plan.md）に沿って Week1 を開始
- 10.10の観測・回帰はCIスモークで継続監視（問題検知時は即fix）

## ♻️ リファクタリング（10.10締めの整備）
- 目的: 大型ファイルを分割し、他AIでも扱いやすい（~1000行以内目安）構成に整理。
- 進捗（完了）:
  - jit/lower/builder.rs → extern_thunks 分離（`src/jit/lower/extern_thunks.rs`）
  - jit/lower/core.rs → cfg_dot 分離（`src/jit/lower/cfg_dot.rs`）
  - builder.rs 1220→980行、core.rs 1072→1012行（機能差分なしの移動）
- 次（小刻みで継続）:
  - core.rs の HostCall 降ろしを `core_hostcall.rs` へ分割（イベントlower＋emit_host_call周り）
  - 算術/比較/分岐を `core_ops.rs` に段階分離
  - 各ステップごとにビルド＋スモーク（CI）確認、ロジック変更なしで進める
