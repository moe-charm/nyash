# 🎯 CURRENT TASK - 2025-08-27（Phase 10_b → 10_c）

フェーズ10はJIT実用化へ！Core-1 Lowerの雛形を固めつつ、呼出/フォールバック導線を整えるよ。

## ⏱️ 今日のフォーカス（10_b: Lower(Core-1) 最小化 + 10_c準備）
- 目的: IRBuilder抽象/Lowerを整備し、JIT関数テーブルとVM分岐の足場を実装。次の10_cで本実行に繋げる。

### 直近タスク（小さく早く）
1) 10_b: Lower/Core-1 最小化（進行中 → ほぼ完了）
   - IRBuilder抽象 + `NoopBuilder`（emit数カウント）✅ 完了
   - `CraneliftBuilder` 雛形（feature `cranelift-jit`）✅ 完了
   - LowerCore（Const/Copy/BinOp/Cmp/Branch/Ret）✅ 完了（emit→Builder）
   - Engine.compile: builder選択（feature連動）＋Lower実行＋JIT handle発行✅ 完了
   - JIT関数テーブル（stub: handle→ダミー関数）✅ 完了
   - 残: 最小emit（const/binop/ret）をCLIFで生成し、関数ポインタをテーブル登録（feature有効時）
     → 実装: CraneliftBuilderでi64用の`const/binop/ret`を生成し、JIT関数テーブルへクロージャとして登録完了（args未対応・i64専用）
2) 10_c: 呼出/フォールバック（準備 → 部分実装）
   - VM側の疑似ディスパッチログ（compiled時/実行時ログ）✅ 完了
   - 残: is_compiled + `NYASH_JIT_EXEC=1` でJIT実行→`VMValue`返却、trap時VMフォールバック
     → 実装: `VM.execute_function`で`NYASH_JIT_EXEC=1`かつ対象関数がcompiledならJIT実行し、その`VMValue`を即return（現状はargs未使用・trap未実装）

備考（制限と次の着手点）
- 返り値はi64（VMValue::Integer）に限定。f64・bool等は未emit
- 引数は未対応（Closureは無視）。MIRのLoad/Param配線が必要
- Compare/Branchはカウンタのみ（emit未着手）
- trap→VMフォールバックは未実装（Craneliftトラップハンドリング追加が必要）

### すぐ試せるコマンド
```bash
cargo build --release -j32
NYASH_JIT_STATS=1 NYASH_JIT_DUMP=1 ./target/release/nyash examples/p2p_ping_pong.nyash

# 疑似実行パスを確認（まだVMフォールバック）
NYASH_JIT_STATS=1 NYASH_JIT_DUMP=1 NYASH_JIT_EXEC=1 \
  ./target/release/nyash examples/p2p_ping_pong.nyash

# （任意）Craneliftを含めてビルド（今は最小初期化のみ）
cargo build --release -j32 --features cranelift-jit
```

## 現在の地図（Done / Next）

### ✅ 完了（Phase 9.79b）
- TypeMeta/Thunk正式化・Poly-PIC（2〜4）・Plugin TLV拡張（bool/i64/f64/bytes）
- VM fast-path整備（Instance/Plugin/Builtin）と統計サマリ強化

### ⏭️ 次（Phase 10）
- 10_a: JITブートストラップ ✅ 完了
- 10_b: Lower(Core-1) – Const/Move/BinOp/Cmp/Branch/Ret（最小emit仕上げ中）
- 10_c: ABI/呼出し – JIT→JIT/JIT→VM、例外バイアウト（実行経路を実体化）
- 10_d: コレクション基礎 – Array/Mapブリッジ
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
