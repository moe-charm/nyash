# CURRENT TASK (Phase 10.8: MIR Core-15 確定 → LLVM 準備)

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
