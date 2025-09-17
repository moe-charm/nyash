# CURRENT TASK – Phase 11.7（JIT Complete / Semantics Layer）

Focus Window: Semantics導入＋jit-direct安定化の確定報告 → GC/Sync/非同期のJIT降下着手

合意事項（要約）

- LLVM AOT は一旦クローズ。Windows 依存と重量を回避し、Cranelift に集中。
- 単一意味論層（Semantics）を導入し、Nyashスクリプト/VM/JIT(exe)を同一動作に揃える。
- VM は参照実装。JIT は実行/生成を担い、VM→JITのランタイムフォールバックは行わない。

現状ステータス（2025-09-01）

- jit-direct 分岐/PHI 合流：単一出口＋BlockParam合流で安定化を確認。
  - テスト: `mir-branch-ret`, `mir-phi-min`, `mir-branch-multi`, `mir-nested-branch`, `mir-phi-two` で VM/JIT 一致（tag=201/200 一致）。
- Semantics 層：`src/semantics/{mod.rs, eval.rs}` にトレイトとPoCインタプリタの骨組みを追加済（未配線）。
- C ABI（NyRT）：`crates/nyrt` の `libnyrt.a` に必要シンボル実装済（console/array/string/plugin_invoke/checkpoint/gc_barrier 等）。
- VM 側：Safepoint/書込バリア/簡易スケジューラ（SingleThread）連携は稼働。
- JIT 側：Safepoint/バリア/await はまだスタブまたは未emit（要降下）。

直近タスク（このフェーズでやること）

1) Semantics 実用化配線（VM/JITの動作一致の“芯”）
   - `SemanticsVM`（VM実行での実装）と `SemanticsClif`（LowerCore+IRBuilder委譲）を用意。
   - `semantics::MirInterpreter` で両者を同一MIRへ適用し、差分検出の土台を作る。
2) JIT へ GC/Sync/非同期の降下
   - Safepoint: `I::Safepoint` を `nyash.rt.checkpoint` emit。`nyrt` 側で `gc.safepoint()` と `scheduler.poll()` に橋渡し。
   - Write Barrier: Array/Map の set/push 等をlowerする箇所で `nyash.gc.barrier_write` を emit（CountingGc で検証）。
   - Await: PoC として FutureBox の同期 get にlower（動作一致優先）。
3) パリティ検証
   - `NYASH_GC_COUNTING=1` で VM/JIT ともに safepoint/barrier カウントが増えることを確認。
   - 既存 smokes（分岐/PHI/配列/外部呼び出し）で一致を継続監視。

実行メモ

- Build（JIT）: `cargo build --release --features cranelift-jit`
- jit-direct: `NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct <app>`
- 追跡: `NYASH_JIT_TRACE_RET/SEL/BLOCKS=1`、GC: `NYASH_GC_COUNTING=1`（必要時）

備考

- LLVM AOT のドキュメント/ツールは維持するが、Windows 前提の依存導入は行わない。Cranelift で“がっちり作る”。
