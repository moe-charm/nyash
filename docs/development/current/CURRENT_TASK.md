# 🎯 CURRENT TASK - 2025-08-27（Phase 10_b → 10_c → 10_4/10_6横串）

フェーズ10はJIT実用化へ！Core-1 Lowerの雛形を固めつつ、呼出/フォールバック導線を整えるよ。

## ⏱️ 今日のサマリ（10_c実行経路の堅牢化＋10_7分岐配線＋GC/スケジューラ導線）
- 目的: JIT実行を安全に通す足場を仕上げつつ、GC/スケジューラ導線を整備し回帰検出力を上げる。
  - 10_c: panic→VMフォールバック（`catch_unwind`）/ JIT経路のroot区域化 / Core-1 i64 param minimal pass（`emit_param_i64` + LowerCoreで供給）✅ 完了
  - 10_4a/10_4b: GC導線 + Write-Barrier挿入（ArraySet/RefSet/BoxCall）、root API（enter/pin/leave）
  - 10_4c: CountingGcカウンタ出力＋roots/BoxRef内訳、depth2リーチャビリティ観測（`NYASH_GC_TRACE=1/2/3`）✅ 完了
  - 10_4d: STRICTバリア検証（CountingGc前後比較で漏れ即検出）✅ 完了（`NYASH_GC_BARRIER_STRICT=1`）
  - 10_6b: シングルスレ・スケジューラ（spawn/spawn_after/poll）、Safepointで`poll()`連携、`NYASH_SCHED_POLL_BUDGET`対応 ✅ 完了
  - 10_7: JIT分岐配線（Cranelift）— MIR Branch/Jump→CLIFブロック配線、条件b1保持、分岐b1/`i64!=0`両対応（feature: `cranelift-jit`）
  - ベンチ: CLIベンチへJIT比較追加（ウォームアップあり）、`branch_return`ケース追加、スクリプト版`examples/ny_bench.nyash`追加（TimerBoxでops/sec）

### 直近タスク（小さく早く）
1) 10_b: Lower/Core-1 最小化（進行中 → ほぼ完了）
   - IRBuilder抽象 + `NoopBuilder`（emit数カウント）✅ 完了
   - `CraneliftBuilder` 雛形（feature `cranelift-jit`）✅ 完了
   - LowerCore（Const/Copy/BinOp/Cmp/Branch/Ret）✅ 完了（emit→Builder）
   - Engine.compile: builder選択（feature連動）＋Lower実行＋JIT handle発行✅ 完了
   - JIT関数テーブル（stub: handle→ダミー関数）✅ 完了
   - 残: 最小emit（const/binop/ret）をCLIFで生成し、関数ポインタをテーブル登録（feature有効時）
     → 実装: CraneliftBuilderでi64用の`const/binop/ret`を生成し、JIT関数テーブルへクロージャとして登録完了（args未対応・i64専用）
2) 10_c: 呼出/フォールバック（最小経路）✅ 完了
   - VM側の疑似ディスパッチログ（compiled時/実行時ログ）✅
   - JIT実行→`VMValue`返却、panic時VMフォールバック ✅（`engine.execute_handle`で`catch_unwind`）
   - Core-1最小: i64 param/return、Const(i64/bool→0/1)、BinOp/Compare/Return ✅
   - HostCall最小（Array/Map: len/get/set/push/size）ゲート`NYASH_JIT_HOSTCALL=1` ✅
   - Branch/JumpはCranelift配線導入済み（feature `cranelift-jit`）。副作用命令は未lowerのためVMへフォールバック
3) 10_7: 分岐配線（Cranelift）— 進捗中
   - LowerCore: BB整列・マッピング→builderの`prepare_blocks/switch/seal/br_if/jump`呼出 ✅
   - CraneliftBuilder: ブロック配列管理、`brif/jump`実装、条件b1/`i64!=0`両対応 ✅
   - 最小PHI（単純ダイアモンド）導入（`NYASH_JIT_PHI_MIN=1`ガード）✅ 初期対応
   - 残: 副作用命令の扱い方針（当面VMへ）、CFG可視化の拡張（`NYASH_JIT_DUMP=1`）

備考（制限と次の着手点）
- 返り値はi64（VMValue::Integer）に限定。f64はconst最小emit、boolはi64 0/1へ正規化（分岐条件入力に対応）
- 引数はi64のみ最小パス。複数引数はparamマッピングで通過、非i64は未対応 → 次対応
- Branch/JumpのCLIF配線は導入済み（feature `cranelift-jit`）。条件はb1で保持し、必要に応じて`i64!=0`で正規化
- 副作用命令（print等）はJIT未対応のためVMへ委譲（安全性優先）
- JIT/VM統合統計（フォールバック率/時間の一括出力）未統合 → 次対応

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

## 現在の地図（Done / Next）

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

### 残タスク（箇条書き）
- 10_c:
  - CLIF: Branch/Jumpの実ブロック配線、Compare結果の適用確認
  - 返り値/引数の型拡張（bool/f64）、複数引数の網羅
  - JIT/VM統合統計（フォールバック率/時間の一括出力）
- 10_4c:
  - リーチャビリティ観測の深さ拡張（depth=2→N）と軽量ダンプ
  - （将来）実Mark/TraverseのPoC（解放はしない）
- 10_4d:
  - STRICTモードのCI導入（CountingGc前提）/ goldenベンチ導入
- 10_6b:
  - スケジューラ: poll予算の設定ファイル化、将来のscript API検討（継続）
- 10_7:
  - 最小PHI（単純ダイアモンド）の導入（`NYASH_JIT_PHI_MIN=1`ガード）
  - IRBuilder APIの整理（block param/分岐引数の正式化）とCranelift実装の安定化
  - 副作用命令のJIT扱い（方針: 当面VMへ、将来はHostCall化）
  - CFG検証と`NYASH_JIT_DUMP=1`でのCFG可視化
- ベンチ:
  - `examples/ny_bench.nyash`のケース追加（関数呼出/Map set-get）とループ回数のenv化

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
