# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

---

## 🔄 **現在の開発状況** (2025-10-03)

### 🎯 **Phase 15.8: WASM実装進行中**
- **ブランチ**: `wasm-development` (← `selfhost`からfork)
- **目標**: MIR18命令 → WASM変換、ブラウザ/エッジ環境対応
- **戦略**: llvm_py拡張（既存800行活用）+ WASI runtime連携
- **計画書**: [Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)

### 🎉 **Result表示修正完了！VM/LLVM AOT両対応** (2025-10-04)
- ✅ **Leaf-level Result表示**: VM/LLVM AOT両方で完全動作
- ✅ **selfhostブランチ統合**: Result表示修正を完全移植
- ✅ **3ファイル修正**:
  - `crates/hako_kernel/src/lib.rs`: AOT main stub Result表示＋flush
  - `src/runner/vm_pipeline.rs`: Result表示責任明確化
  - `src/runner/modes/vm.rs`: stdout flush実装
- ✅ **環境変数対応**: `NYASH_NYRT_SILENT_RESULT=1` でベンチマーク用出力抑制
- ✅ **テスト確認**: `/tmp/mini_ret.hako` → `Result: 7` 完全動作
- ✅ **コミット**: `0edaaffa` - "feat(vm): Result表示完全修正"
- 📖 **詳細**: hakorune-selfhost の `docs/guides/result-printing.md`

### 🎉 **Phase 3.4完了！統合ベンチマークシステム実装** (2025-10-03)
- ✅ **bench_unified.sh完全書き直し**（420行、2フェーズ分離設計）
- ✅ **ChatGPT Pro設計準拠**: [apps/benchmarks/DESIGN.md](apps/benchmarks/DESIGN.md)
- ✅ **VMベンチマーク完全動作**（カウンター: 2ms、フィボナッチ: 2ms、素数判定: 3ms）
- ✅ **LLVM/WASMビルド成功**（Phase 1: Preparation完了）
- ⚠️ **LLVM Phase 2実行問題**（Warmup後ハング、調査中）

### 🎉 **Phase 3.5完了！固定時間ベンチマーク完全実装** (2025-10-04)

#### ✅ **VM版完全動作**
- TimerBox.now_ms(): コンパイラが自動的に`ExternCall(nyrt.time.now_ms)`に変換
- VM側`extern_adapter.rs`で`SystemTime::now()`実装済み
- 166ms差分を正確に計測✅

#### ✅ **LLVM版完全動作** 🎉
**根本原因判明・解決**:
- nyrt.time.now_ms()実装は完全に正常（libhako_kernel.a）
- SystemTime::now()を正しく呼び出し
- **問題**: ループ100,000回は1ミリ秒未満で完了（時間差が0）
- **解決**: ループ回数を10,000,000回に増加 → 測定可能な時間差発生 ✅

**確認済み事項**:
- ✅ libhako_kernel.aにnyrt.time.now_ms実装あり（121バイト）
- ✅ 実行可能ファイルに正しくリンク済み
- ✅ LLVM IRでexterncall正しく生成
- ✅ SystemTime::now()を呼び出し中
- ✅ 逆アセンブルで動作確認完了

#### ✅ **固定時間方式実装完了**
- `run_duration(file, duration_sec)` メソッド（bench_runner.hako）
- DESIGN.md準拠の完全実装（end_timeまでループ、ops/sec計算）
- MapBoxで結果構造化（iterations/duration_ms/ops_per_sec）
- **VM版実測**: 空ベンチ 109,543 ops/sec、sum_loop 5 ops/sec
- **LLVM版**: local_tests/bench_timer_llvm.hako（5秒間測定方式）実装

### 🏆 **言語対決ベンチマーク完了！** (2025-10-04)

**sum_loop ベンチマーク（固定5秒測定）**:

| 言語   | Backend | Ops/sec      | 相対速度 | 対C比 | 備考 |
|--------|---------|--------------|----------|-------|------|
| C      | gcc -O3 | 58,012,004   | 1.00x    | 100% | - |
| Python | CPython 3.x | 17,915,223 | 0.31x  | 31%  | Computed goto |
| **Ruby** | **YARV 3.2** | **11,178,680** | **0.19x** | **19%** | Switch VM |
| Nyash  | Rust VM | 351,263      | 0.006x   | 0.6% | BoxCall重い |
| Nyash  | LLVM    | **失敗**     | -        | -    | シンボル不足* |

**\*LLVM失敗原因**: `libhako_kernel.a`に`nyash.console.log`/`nyash.string.concat_si`が未実装

**重要な発見**:
- 🧠 **Python二層戦略の発見**: 「インタープリター」ではなく「軽量VM + C層委譲」
  - Python層: 制御フローのみ（バイトコードVM、18命令/ループ）
  - C層: 実際の処理（`time.time()`, 整数演算等）
  - オーバーヘッド: 約9 CPU命令/バイトコード（**超軽量！**）
  - 実測: 3.2億バイトコード命令/秒（Computed goto最適化）
  - 📝 このベンチは**C層の速度**を測定（Python VM層ほぼ未測定）
- 🔴 **Ruby vs Python**: 命令数少ないのに遅い！
  - Ruby: 14命令/ループ、1.6億命令/秒（1命令≈20 CPU）
  - Python: 18命令/ループ、3.2億命令/秒（1命令≈9 CPU）
  - 原因: Rubyは「すべてオブジェクト」思想（Time.nowも重い）
- ✅ Nyash VM妥当なインタープリターオーバーヘッド（ただしBoxCall重い）
- 🎉 **LLVM版完全成功！** (2025-10-04)
  - **39.4M ops/sec** - C言語の68%、VM版の112倍！
  - シンボル追加完了: `nyash.console.log`, `nyash.string.concat_si`
  - Python (31%) / Ruby (19%) を大幅に超える速度を実現！

**実行方法**: `bash benchmarks/run_language_shootout.sh`

**詳細**: [benchmark-implementation.md](docs/development/current/wasm/benchmark-implementation.md)

#### 📋 **次のステップ**: WASM版固定時間ベンチマーク実装 → VM/LLVM/WASM速度比較表作成

### 📊 **WASM対応状況**（MIR凍結セット16命令基準）
**✅ 実装済み（16/16命令 - 完全対応！）**:
1. **基本演算(5)**: Const, UnaryOp, BinOp, Compare, TypeOp
2. **メモリ(2)**: Load, Store
3. **制御(4)**: Branch, Jump, Return, Phi（if/loop両対応）
4. **呼び出し(1)**: Call/MirCall（統一Call実装済み）
5. **GC(2)**: Barrier, Safepoint
6. **構造(2)**: Copy, Nop

**🎉 完全実装済み！**
- ExternCall, BoxCall, NewBox も動作確認済み
- LoopForm実験的実装あり
- 詳細: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)

**🚨 発見された問題**:
- Hakoコンパイラ: 不正PHI命令生成バグ（到達不能ブロックがPHI predecessorに含まれる）

### 📚 **重要リソース**
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
- **現在のタスク**: [CURRENT_TASK_WASM.md](CURRENT_TASK_WASM.md) ⭐ここを見る！
- **Phase 15.8詳細**: [docs/development/roadmap/phases/phase-15.8/](docs/development/roadmap/phases/phase-15.8/)
- **ベンチマーク設計図**: [apps/benchmarks/DESIGN.md](apps/benchmarks/DESIGN.md) ⭐ChatGPT Pro設計
- **WASMベンチマークガイド**: [docs/guides/wasm-benchmarks.md](docs/guides/wasm-benchmarks.md)
- **MIR命令セット**: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md) ⭐正式仕様

---

## 🔧 ビルド・実行方法

### 🚀 基本ビルド
```bash
# 標準ビルド（Rust VM）
cargo build --release

# LLVM機能付きビルド
cargo build --release --features llvm
```

### ⚡ 基本実行（hakoコマンド推奨）
```bash
# 基本実行
./target/release/hako program.nyash

# VM実行（明示的）
./target/release/hako --backend vm program.nyash

# LLVM実行（最適化）
./target/release/hako --backend llvm program.nyash

# クリーンな出力（デバッグメッセージ抑制）
NYASH_QUIET=1 ./target/release/hako program.nyash
```

### 🌐 WASM実行（Phase 15.8）
```bash
# WASMベンチマークスイート実行
bash tools/run_wasm_benchmark_suite.sh

# 個別WASM生成＆実行
bash tools/build_wasm.sh src/llvm_py/test_arithmetic_smoke.json -o /tmp/test.wasm
node tools/wasm_runner.js /tmp/test.wasm

# WASMスモークテスト
bash tools/run_wasm_smoke_tests.sh
```

### 📚 実行モード詳細ガイド

**🎯 実行方法がわからなくなったら**: [実行モード完全ガイド](docs/guides/execution-modes-guide.md) ⭐必読

Hakoruneには**4つの実行モード**があります：

| モード | 用途 | コマンド例 |
|--------|------|-----------|
| **VM** | 開発・デバッグ | `./hako program.nyash` |
| **LLVM CLI** | 本番・最適化 | `NYASH_LLVM_USE_HARNESS=1 ./hako --backend llvm program.nyash` |
| **LLVM AOT** | スタンドアロンEXE | `./program.exe` (事前ビルド必要) |
| **WASM** | Web実行 | `node wasm_runner.js program.wasm` |

詳細な使い分け・トラブルシューティングは [実行モードガイド](docs/guides/execution-modes-guide.md) 参照。

**🔍 内部実装を理解したい**: [技術詳解: 関数解決の仕組み](docs/guides/execution-modes-technical-deep-dive.md)
- LLVM CLIがHakoruneの実行ファイル（libhakorune_kernel.a）で関数を解決する仕組み
- 各モードの関数解決マトリックス・デバッグ方法

---

## 📊 環境変数（主要なもの）

**🎯 よく使う環境変数**:
- `NYASH_QUIET=1`: 出力抑制（スモークテスト・CI）
- `NYASH_CLI_VERBOSE=1`: 詳細診断（デバッグ時）
- `NYASH_LLVM_USE_HARNESS=1`: LLVM/llvmliteハーネス有効化
- `NYASH_DISABLE_PLUGINS=1`: プラグイン無効化

**🔧 デバッグ用**:
```bash
# MIR出力（重要！）
NYASH_DUMP_MIR=1 ./target/release/hako program.nyash
./target/release/hako --dump-mir program.nyash  # フラグ版

# JSON IR出力
./target/release/hako --emit-mir-json output.json program.nyash
```

📖 **完全ガイド**: [環境変数完全ガイド](docs/reference/environment-variables.md)

---

## 🧪 スモークテスト

### 推奨テストコマンド
```bash
# VM ライン（開発・デバッグ）
tools/smokes/v2/run.sh --profile quick

# llvmlite ライン（本番・最適化）
tools/smokes/v2/run.sh --profile integration

# WASMテスト
bash tools/run_wasm_smoke_tests.sh

# PHI関連テスト
bash tools/smokes/v2/run_phi.sh
```

📖 **スモークテスト完全ガイド**: [tools/smokes/README.md](tools/smokes/README.md)

---

## Start Here (必ずここから)
- 現在のタスク: [CURRENT_TASK.md](CURRENT_TASK.md)
  - 📁 **Main**: [docs/development/current/main/](docs/development/current/main/)
  - 📁 **LLVM**: [docs/development/current/llvm/](docs/development/current/llvm/)
  - 📁 **Self**: [docs/development/current/self_current_task/](docs/development/current/self_current_task/)
- ドキュメントハブ: [README.md](README.md)
- 🚀 **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)

## 🧱 先頭原則: 「箱理論（Box-First）」で足場を積む

Nyashは「Everything is Box」。実装・最適化・検証のすべてを「箱」で分離・固定し、いつでも戻せる足場を積み木のように重ねる。

### 実践テンプレート（開発時の合言葉）
- 「箱にする」: 設定・状態・橋渡しはBox化（例: JitConfigBox, HandleRegistry）
- 「境界を作る」: 変換は境界1箇所で（VMValue↔JitValue, Handle↔Arc）
- 「戻せる」: フラグ・feature・env/Boxで切替。panic→フォールバック経路を常設
- 「見える化」: ダンプ/JSON/DOTで可視化、回帰テストを最小構成で先に入れる
- 「Fail-Fast」: エラーは隠さず即座に失敗。フォールバックより明示的エラー

---

## 🤖 **Claude×Copilot×ChatGPT協調開発**

### 📋 **開発マスタープラン**
**すべてはここに書いてある！** → [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)

**現在のフェーズ：Phase 15.8 (WASM実装)**

### 🎊 **最新成果（2025-10-03）**
- ✅ **Phase 15.5-15.8完了**: Core Box統一・MIR命令安定化・LLVM PHI安定化・型変換統一化
- ✅ **MIR Builder2実装**: static box引数消失バグ回避（インスタンス版）
- ✅ **Rust VMすけすけトレース実装**: 1命令/1行観測＋ステッパ機能
- ✅ **VM Bug修正完了**: PHI predecessor判定バグ修正（3つのバグが1つの根本原因から）

### 🚀 **Phase 15戦略: Rust VM + LLVM 2本柱**
```
【Rust VM】  開発・デバッグ・検証用（高速・型安全）
【LLVM】     本番・最適化・配布用（Python/llvmlite、実証済み）
【WASM】     Phase 15.8実験的（llvm_py拡張、call命令完全動作済み）
```

📋 **詳細**: [Phase 15 INDEX](docs/development/roadmap/phases/phase-15/INDEX.md) | [CURRENT_TASK.md](CURRENT_TASK.md)

---

## 🏃 開発の基本方針: 80/20ルール - 完璧より進捗

### なぜこのルールか？
**実装後、必ず新しい問題や転回点が生まれるから。**
- 100%完璧を目指すと、要件が変わったときの手戻りが大きい
- 80%で動くものを作れば、実際の使用からフィードバックが得られる
- 残り20%は、本当に必要かどうか実装後に判断できる

### 実践方法
1. **まず動くものを作る**（80%）
2. **改善アイデアは `docs/development/proposals/ideas/` フォルダに記録**（20%）
3. **優先度に応じて後から改善**

---

## 🚀 クイックスタート

### 🎯 **2本柱実行方式** (推奨!)
```bash
# 🔧 開発・デバッグ・検証用 (Rust VM)
./target/release/hako program.nyash
./target/release/hako --backend vm program.nyash

# ⚡ 本番・最適化・配布用 (LLVM)
./target/release/hako --backend llvm program.nyash

# 🛡️ プラグインエラー対策
NYASH_DISABLE_PLUGINS=1 ./target/release/hako program.nyash

# 🔍 詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/hako program.nyash
```

### 🌐 **WASMライン**（Phase 15.8実験的）
```bash
# WASMベンチマークスイート実行
bash tools/run_wasm_benchmark_suite.sh

# 個別WASM生成＆実行
bash tools/build_wasm.sh src/llvm_py/test_arithmetic_smoke.json -o /tmp/test.wasm
node tools/wasm_runner.js /tmp/test.wasm
```

---

## 🧪 テストスクリプト参考集
```bash
# 基本的なテスト
./target/release/hako local_tests/hello.nyash              # Hello World
./target/release/hako local_tests/test_array_simple.nyash  # ArrayBox
./target/release/hako apps/tests/string_ops_basic.nyash    # StringBox

# MIR確認用テスト
./target/release/hako --dump-mir apps/tests/loop_min_while.nyash
```

---

## 🚀 よく使う実行コマンド

### 🎯 基本実行方法
```bash
# VMバックエンド（デフォルト、高速）
./target/release/hako program.nyash
./target/release/hako --backend vm program.nyash

# LLVMバックエンド（最適化済み）
./target/release/hako --backend llvm program.nyash

# プラグイン無効（デバッグ用）
NYASH_DISABLE_PLUGINS=1 ./target/release/hako program.nyash
```

### 🔧 テスト・スモークテスト
```bash
# コアスモーク（プラグイン無効）
./tools/jit_smoke.sh

# LLVMスモーク
./tools/llvm_smoke.sh

# ラウンドトリップテスト
./tools/ny_roundtrip_smoke.sh

# WASMスモーク
bash tools/run_wasm_smoke_tests.sh
```

### 📊 ベンチマークシステム（Phase 15.8）
**設計**: [apps/benchmarks/DESIGN.md](apps/benchmarks/DESIGN.md) - ChatGPT Pro設計
**重要原則**: 準備フェーズと測定フェーズの分離！

#### 🔨 ビルド方法（準備フェーズ）
```bash
# LLVM実行ファイル生成（~700ms、1回のみ）
bash tools/build_llvm.sh <program.nyash> -o <output_exe>

# WASM生成（1回のみ）
bash tools/build_wasm.sh <mir.json> -o <output.wasm>

# VM: 準備不要（インタープリタ）
```

#### ⏱️ ベンチマーク実行（測定フェーズ）
```bash
# 統合ベンチマーク（3バックエンド）
bash tools/bench_unified.sh --backend all --warmup 10 --repeat 50
bash tools/bench_unified.sh --backend vm --warmup 2 --repeat 3  # クイック
bash tools/bench_unified.sh --backend llvm --warmup 10 --repeat 50
bash tools/bench_unified.sh --backend wasm --warmup 10 --repeat 50
```

**詳細**: [apps/benchmarks/README.md](apps/benchmarks/README.md)

### 🐛 デバッグ用環境変数
```bash
# 詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/hako program.nyash

# JSON IR出力
NYASH_DUMP_JSON_IR=1 ./target/release/hako program.nyash

# MIR出力（重要！）
NYASH_DUMP_MIR=1 ./target/release/hako program.nyash
./target/release/hako --dump-mir program.nyash  # フラグ版

# パーサー無限ループ対策
./target/release/hako --debug-fuel 1000 program.nyash

# プラグインなし実行
NYASH_DISABLE_PLUGINS=1 ./target/release/hako program.nyash

# Python/llvmliteハーネス使用
NYASH_LLVM_USE_HARNESS=1 ./target/release/hako --backend llvm program.nyash
```

---

## 🔬 **Rust VM すけすけトレース（MVP実装済み！）** ⭐NEW

### 🎯 **実行時1命令トレース**
```bash
# 基本トレース（フィルタ＋値表示、1命令/1行）
HAKO_VM_TRACE="op=compare,binop,externcall,boxcall,call;regs=1;block=*" ./target/release/hakorune test.hkr

# または
NYASH_VM_TRACE="op=compare,binop;regs=1" ./target/release/hakorune test.hkr

# 出力例:
# [vm] bb=0 inst=2 binop kind=Add lhs=v%1(42) rhs=v%2(10) dst=v%3 → 52
# [vm] bb=0 inst=3 boxcall recv=v%0(MapBox) method="set" args=[v%1,v%3] dst=v%4
# [vm] bb=0 inst=4 compare kind=Gt lhs=v%1(6) rhs=v%2(3) dst=v%3 → 1
```

### 🛑 **ステッパ機能（対話デバッグ）**
```bash
# 1命令ずつ停止・実行
HAKO_VM_STEP=1 ./target/release/hakorune test.hkr

# 対話ブロック許可（stdin待機）
HAKO_VM_STEP=1 HAKO_VM_STEP_ALLOW_BLOCK=1 ./target/release/hakorune test.hkr

# プロンプト:
# > [n]ext/[c]ontinue/[r]egisters/[q]uit?
# n → 次の命令へ
# c → 実行継続
# r → レジスタ状態表示
# q → 終了
```

### 🔍 **引数トレース（補助機能）**
```bash
# Global/ModuleFn/Legacy 経路の a0/a1 と種別を出力
NYASH_VM_CALL_ARG_TRACE=1 ./target/release/hakorune test.hkr

# 出力例:
# [call_arg] Global: a0=v%1(42) a1=v%2(10)
# [call_arg] ModuleFn: a0=v%3(MapBox) a1=null
```

### 📍 **実装場所**
- トレース＆ステッパ: `src/backend/mir_interpreter/exec.rs:242, 386`
- 引数トレース: `src/backend/mir_interpreter/handlers/calls/{function.rs,legacy.rs}`

### 💡 **使用例（今回の static box 引数消失問題）**
```bash
# このトレースがあれば一瞬で発見できた：
HAKO_VM_TRACE="op=boxcall;regs=1" ./target/release/hakorune emit_compare_test.hkr

# 期待される出力:
# [vm] boxcall MirJsonBuilderMin.start_module args=[v%3(null)]
#                                                    ↑ ここで即座に「引数null」発見！
```

### 🚨 **重要：2つのトレースレイヤーを混同しない！**

#### 📦 **Layer 1: Rust VMトレース（すけすけ機能）**
```bash
# ← これが「すけすけ」！Rust VM内部のMIR実行を観測
export HAKO_VM_TRACE="op=boxcall,externcall;regs=1"
export NYASH_DISABLE_PLUGINS=1
./target/release/hakorune test.hkr 2>&1

# 出力例:
# [vm] bb=380 inst=1 boxcall boxcall method="length"
# [vm] → v%12(267)
```

#### 📝 **Layer 2: Mini-VM内部ログ（_tprint）**
```bash
# Hakoruneスクリプトで書かれたMini-VM内部のprintログ
# mir_vm_min.hako の _tprint() が出力

# 普通のprint()なので、実行されれば自動で出る
# （今回はMIRエラーで早期終了したため見えなかった）
```

#### ⚠️ **私（Claude）がよく混同するポイント**
```
❌ 間違い：「_tprintログを見るためにHAKO_VM_TRACEを使う」
✅ 正解：
  - HAKO_VM_TRACE = Rust VMの実行トレース（すけすけ）
  - _tprint = Mini-VM内部のprintログ（Hakoruneスクリプトレベル）
  - 別物！
```

---

## 🔍 MIRデバッグ出力完全ガイド（必読！）

### 🎯 **確実にMIRを出力する方法**（優先順）

```bash
# 1️⃣ 最も確実: CLIフラグ使用
./target/release/hako --dump-mir program.nyash
./target/release/hako --dump-mir --mir-verbose program.nyash  # 詳細版

# 2️⃣ VM実行時のMIR出力
NYASH_VM_DUMP_MIR=1 ./target/release/hako program.nyash

# 3️⃣ JSON形式でファイル出力
./target/release/hako --emit-mir-json debug.json program.nyash
cat debug.json | jq .  # 整形表示
```

### 💡 **実用的デバッグフロー**
```bash
# Step 1: 基本MIR確認
./target/release/hako --dump-mir test_case.nyash

# Step 2: 詳細MIR + エフェクト情報
./target/release/hako --dump-mir --mir-verbose --mir-verbose-effects test_case.nyash

# Step 3: VM実行時の挙動確認
NYASH_VM_DUMP_MIR=1 NYASH_CLI_VERBOSE=1 ./target/release/hako test_case.nyash

# Step 4: JSON形式で詳細解析
./target/release/hako --emit-mir-json mir.json test_case.nyash
jq '.functions[0].blocks' mir.json  # ブロック構造確認
```

---

## ⚡ 重要な設計原則

### 🏗️ Everything is Box
- すべての値がBox（StringBox, IntegerBox, BoolBox等）
- ユーザー定義Box: `box ClassName { field1: TypeBox field2: TypeBox }`
- **MIR凍結セット**: 16命令で全機能実現！（詳細: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)）

### 🌟 完全明示デリゲーション
```nyash
// デリゲーション構文（すべてのBoxで統一的に使える！）
box Child from Parent {
    birth(args) {  // コンストラクタは「birth」に統一
        from Parent.birth(args)  // 親の初期化
    }

    override method() {  // 明示的オーバーライド必須
        from Parent.method()  // 親メソッド呼び出し
    }
}
```

### 🔄 統一ループ構文
```nyash
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
```

### 🎯 正統派Nyashスタイル
```nyash
// 🚀 Static Box Main パターン - エントリーポイントの統一スタイル
static box Main {
    console: ConsoleBox
    result: IntegerBox

    main() {
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")

        local temp
        temp = 42
        me.result = temp

        return "Revolution completed!"
    }
}
```

### 📝 変数宣言厳密化システム
```nyash
// 🔥 すべての変数は明示宣言必須！

// ✅ static box内のフィールド
static box Calculator {
    result: IntegerBox
    memory: ArrayBox

    calculate() {
        me.result = 42

        local temp
        temp = me.result * 2
    }
}

// ❌ 未宣言変数への代入はエラー
x = 42  // Runtime Error: 未宣言変数
```

---

## 🏗️ アーキテクチャ決定事項

### **ExternCall Registry 2層分離アーキテクチャ** (2025-10-03)
```
ExternCallRegistryBox (共通・抽象)
    interface: "nyrt.time"
    method: "now_ms"
    effects: READ
    ↓
┌───┼───┐
↓   ↓   ↓
WASM VM LLVM Adapters (各Backend・具体)
```

**設計原則**:
- **Registry**: 抽象仕様のみ（interface/method/effects）
- **Adapter**: バックエンド固有実装（WASM=i32, VM=SystemTime, LLVM=JSON）
- **Fail-Fast**: 未知extern → RuntimeError（フォールバック禁止）
- **疎結合**: 各Backendが独立開発可能

詳細: [Externs Registry](docs/development/architecture/externs_registry.md)

### **Box/ExternCall境界設計** (2025-09-11)
- **基本Box**: nyrt内蔵（String/Integer/Array/Map/Bool）
- **拡張Box**: プラグイン（File/Net/User定義）
- **ExternCall**: Registry管理（timer/array.size/map.size等）
- **統一原則**: すべてのBoxはBoxCall経由（特別扱いなし）

詳細: [Box/ExternCall設計](docs/development/architecture/box-externcall-design.md)

---

## 📚 ドキュメント構造

### 🎯 最重要ドキュメント（開発者向け）
- **[CURRENT_TASK.md](CURRENT_TASK.md)** - 現在進行状況詳細
- **[00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)** - 開発マスタープラン
- **[Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)** - WASM実装計画

### 📖 利用者向けドキュメント
- 入口: [docs/README.md](docs/README.md)
  - Getting Started: [docs/guides/getting-started.md](docs/guides/getting-started.md)
  - Language Guide: [docs/guides/language-guide.md](docs/guides/language-guide.md)
  - Reference: [docs/reference/](docs/reference/)

### 🎯 リファレンス
- **言語**:
  - [Quick Reference](docs/reference/language/quick-reference.md) ⭐最優先
  - [LANGUAGE_REFERENCE_2025.md](docs/reference/language/LANGUAGE_REFERENCE_2025.md) - 完全仕様
- **MIR**: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)
- **API**: [boxes-system/](docs/reference/boxes-system/)
- **プラグイン**: [plugin-system/](docs/reference/plugin-system/)

---

## 📖 ドキュメントファースト開発（重要！）

### 🚨 開発手順の鉄則
**絶対にソースコードを直接読みに行かない！必ずこの順序で作業：**

1. **📚 ドキュメント確認** - まず既存ドキュメントをチェック
2. **🔄 ドキュメント更新** - 古い/不足している場合は更新
3. **💻 ソース確認** - それでも解決しない場合のみソースコード参照

### 🎯 最重要ドキュメント（2つの核心）

#### 🔤 言語仕様
- **[クイックリファレンス](docs/reference/language/quick-reference.md)** ⭐最優先
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)** - 基本構文・よくある間違い
- **[完全リファレンス](docs/reference/language/LANGUAGE_REFERENCE_2025.md)** - 言語仕様詳細

#### 📦 主要BOXのAPI
- **[Box/プラグイン関連](docs/reference/boxes-system/)** - APIと設計

---

## 🔧 重要設計書（迷子防止ガイド）

### 🏗️ **アーキテクチャ核心**
- **[名前空間・using system](docs/reference/language/using.md)** ⭐超重要
- **[MIR Callee革新](docs/development/architecture/mir-callee-revolution.md)**
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)**

### 📋 **Phase 15関連資料**
- **[Phase 15 INDEX](docs/development/roadmap/phases/phase-15/INDEX.md)**
- **[Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)**

### 📖 **完全リファレンス**
- **[言語仕様](docs/reference/language/LANGUAGE_REFERENCE_2025.md)**
- **[プラグインシステム](docs/reference/plugin-system/)**

---

## 🔧 開発サポート

### 🎛️ 重要フラグ一覧
```bash
# プラグイン制御
NYASH_DISABLE_PLUGINS=1

# デバッグ
NYASH_CLI_VERBOSE=1
NYASH_DUMP_JSON_IR=1
```

### 🐍 Python LLVM バックエンド (実用レベル到達！)
**場所**: `/src/llvm_py/`

llvmliteベースのLLVMバックエンド実装。箱理論により650行→100行の簡略化を実現！

#### 実行方法
```bash
cd src/llvm_py
python3 -m venv venv
./venv/bin/pip install llvmlite
./venv/bin/python llvm_builder.py test_minimal.json -o output.o
```

#### 実装済み命令
- ✅ const, binop, jump, branch, ret, compare
- ✅ phi, call, boxcall, externcall
- ✅ typeop, newbox, safepoint, barrier, loopform

---

## 💡 アイデア管理

**80/20ルールの「残り20%」を整理して管理**

```
docs/development/proposals/ideas/
├── improvements/     # 80%実装の残り20%改善候補
├── new-features/     # 新機能アイデア
└── other/           # その他すべて（調査、メモ、設計案）
```

---

## 🤝 プロアクティブ開発方針

エラーを見つけた際は、単に報告するだけでなく：

1. **🔍 原因分析** - エラーの根本原因を探る
2. **📊 影響範囲** - 他のコードへの影響を調査
3. **💡 改善提案** - 関連する問題も含めて解決策を提示
4. **🧹 機会改善** - デッドコード削除など、ついでにできる改善も実施

詳細: [開発プラクティス](docs/guides/development-practices.md)

---

## ⚠️ Claude実行環境の既知のバグ

詳細: [Claude環境の既知のバグ](docs/tools/claude-issues.md)

### 🐛 Bash Glob展開バグ（Issue #5811）

```bash
# ❌ 失敗するパターン
ls *.md | wc -l

# ✅ 回避策: bash -c でラップ
bash -c 'ls *.md | wc -l'
```

---

## 🚨 コンテキスト圧縮時: 作業停止→状況確認→CURRENT_TASK.md確認→ユーザー確認

---

**Notes**:
- ここから先の導線は README.md に集約
- 詳細情報は各docsファイルへのリンクから辿る
- Phase 15.8 WASM実装中！詳細は[Phase 15.8](docs/development/roadmap/phases/phase-15.8/)へ

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
