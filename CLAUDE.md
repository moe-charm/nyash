# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

**⚠️ 重要**: このファイルの「開発状況」は**成功報告が中心**です。実際の開発では**失敗・問題点の報告が最も重要**です。失敗報告については [🚨 失敗報告の重要性](#-失敗報告の重要性最優先) セクションを必ず参照してください。

---

## 🔄 **現在の開発状況** (2025-10-06)

**注**: 以下は主に成功報告です。各Phaseの問題点・失敗・学びについては、個別のドキュメントや issue を参照してください。

### ⚠️ **Phase 2.1（dep_tree統合）の問題点・失敗** (2025-10-06)

#### ❌ **主要な問題点**

**1️⃣ テスト実行完全失敗**
- **問題**: 3ファイル統合したが、1回も動作確認できていない
- **実際**: FileBoxエラー、usingパースエラー、原因調査なし
- **影響**: commit前に動作検証必須（現在未検証状態）

**2️⃣ 見積もりの大誤算**
- **見積もり**: 108-150行削減
- **実際**: 20行削減のみ（**見積もりの18%**）
- **原因**: Hakoruneの構文制約（セミコロン区切り不可）を考慮せず
- **内訳**: 重複削減-75行、構文修正+23行、Core追加+55行 → 純削減20行

**3️⃣ 構文エラー連発（4回修正）**
- Line 47: `continue` 使用 → Hakoruneは未サポート
- Line 94, 111, 172, 177, 183, 226: セミコロン区切り → すべて複数行展開

**4️⃣ using文の混乱**
- `using selfhost.tools.dep_tree_core` → ❌ "Unsupported namespace"
- `using "./dep_tree_core.hako"` → ❌ "Expected identifier"
- hako.tomlに追加したのに動かない理由、**調査していない**

**5️⃣ 背景プロセス放置**
- LLVMベンチマーク（Bash 15351f）実行中のまま、結果確認なし

#### 📊 **実際の成果（客観的）**

**変更ファイル**:
- 新規: `dep_tree_core.hako` (55行)
- 変更: `dep_tree.nyash` (253→247行, -6行)
- 変更: `dep_tree_simple.nyash` (265→233行, -32行)
- 変更: `dep_tree_min_string.nyash` (159→122行, -37行)
- 変更: `hako.toml` (+1行エイリアス)

**合計**: 純削減20行（見積もり比18%）

#### 🎓 **学び**
1. **事前確認**: Hakoruneの構文制約を確認してから見積もるべき
2. **中間テスト**: コード編集中に最低1回は動作確認すべき
3. **調査優先**: エラーが出たら、試行錯誤より根本原因調査を優先
4. **背景プロセス**: 長時間実行プロセスは定期的に確認

---

### 🎉 **Phase 15.11完了！StringHelpers共通ライブラリ箱化成功** (2025-10-05)
**セルフホストコード重複削減 - 14ファイル統合で335行純削減**

#### ✅ **StringHelpers共通ライブラリ作成**
**新規ファイル**:
- `apps/selfhost/common/string_helpers.hako` (86行)
  - `int_to_str(n)` - 整数→文字列変換
  - `to_i64(x)` - 文字列/数値→i64パース（負数対応）
  - `json_quote(s)` - JSON文字列エスケープ
  - `is_numeric_str(s)` - 数値文字列判定
  - `read_digits(text, pos)` - 連続数字読み取り
- `apps/selfhost/test_string_helpers.hako` - 包括的テストスイート

#### ✅ **14ファイル更新完了**
**JSON builders** (3ファイル):
- mir_builder2.hako
- mir_builder_min.hako
- mir_builder_min.hako

**JSON utilities** (2ファイル) - **Phase 15.11.1追加**:
- json_scan.hako (_str_to_int委譲)
- json_frag.hako (read_digits + _str_to_int委譲)

**Mini-VM components** (5ファイル):
- mini_vm_scan.hako
- mir_vm_min.hako
- mir_vm_m2.hako
- op_handlers.hako
- flow_debugger.hako

**Other tools** (4ファイル):
- seam_inspector.hako
- collect_mixed_smoke.hako
- mini_vm_if_branch.hako
- mini_vm_lib.hako

#### 📊 **統計**
- **Phase 15.11**: 319行削減 (380削除 - 61追加)
- **Phase 15.11.1**: 15行削減 (22削除 - 7追加) - ChatGPT協力
- **合計削減**: 335行
- **重複削除**: 7種類のヘルパー関数を統合
- **コミット**: `6ba6b026` (本体), `d07f3af3` (追加統合), `0de80fa6` (docs)

#### 🎯 **次のステップ（Phase 15.12候補）**
- `index_of_from` → CfgNavigatorBox統合 (60-100行削減見込み)
- 詳細: `docs/development/proposals/ideas/improvements/phase-15-12-index-of-from-consolidation.md`

#### 🐛 **既知の問題**
- `--dump-mir`フラグがusing文でパースエラー（別issue記録済み）
- 通常実行は完全動作

---

### ✅ **Phase 15.10完了** (2025-10-05)
Legacy Code大掃除 - 2大ファイル→8小ファイル分割、デッドコード470行削除、純削減400行。コミット: `43679766`, `f6cbbf48`, `f1f3b83e`

### ✅ **Phase 15.9完了** (2025-10-05)
VmConfig集約化 - 環境変数42ファイル散在→1箇所集約、パフォーマンス向上。コミット: `f1874b3b`

### ✅ **Birth Lifecycle完全統合** (2025-10-05)
3つのcalling convention統一契約化、58ファイル843行修正、production環境バグ解消
1. ~~BuilderConfigBox実装（MIR Builder用環境変数約15種類）~~ → 保留（現状維持）
2. ✅ legacy.rs分割（calls:617行 + boxes:518行） → **Phase 15.10で完了**
3. ~~boxes_* → builtin_boxes/ 移動~~ → 現状維持（移動のメリットなし）

---

### ✅ **Phase 15.8** (2025-10-04)
WASM実装進行中 - MIR16命令完全対応。詳細: [Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)

### ✅ **Phase 3.4-3.5完了** (2025-10-03~04)
統合ベンチマーク＋固定時間測定実装。言語対決: Nyash LLVM版39.4M ops/sec（C言語の68%）達成。詳細: [benchmark-implementation.md](docs/development/current/wasm/benchmark-implementation.md)

### 📚 **重要リソース**
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
- **現在のタスク**: [CURRENT_TASK.md](CURRENT_TASK.md)
- **MIR命令セット**: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)

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
2. **失敗・問題点を記録**（最重要！）
3. **改善アイデアは `docs/development/proposals/ideas/` フォルダに記録**（20%）
4. **優先度に応じて後から改善**

**⚠️ 注意**: 80%で完了とするのは「機能」だけです。**失敗・問題点の記録は100%必須**です。

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

## 🚨 **失敗報告の重要性（最優先！）**

### **プログラム開発では失敗報告が一番大事**

**成功報告より失敗報告が重要な理由**:
- ✅ 失敗は**次の改善の種**（成功は既に終わったこと）
- ✅ 失敗は**学習の最大の機会**（同じミスを繰り返さない）
- ✅ 失敗は**システムの脆弱性を教えてくれる**（本番障害を未然に防ぐ）
- ✅ 失敗は**見積もり精度を上げる**（楽観的予測を修正）

### **報告すべき失敗の種類**

#### 1️⃣ **実行失敗・テスト失敗**
```
❌ テスト実行0回成功
❌ コンパイルエラー4回連続
❌ 動作確認できていない状態でcommit提案
```

#### 2️⃣ **見積もりの失敗**
```
当初見積もり: 108-150行削減
実際の結果:   20行削減のみ（見積もりの18%）

原因: 構文制約による増加分を考慮していなかった
```

#### 3️⃣ **設計判断の失敗**
```
判断: セミコロン区切り1行文で書く
結果: Hakoruneでパースエラー → 全部複数行に書き直し (+23行)

原因: Hakoruneの構文制約を忘れていた
```

#### 4️⃣ **理解不足・調査不足**
```
問題: using文でパースエラー
対応: 3通りの書き方を試す → すべて失敗
根本原因: **調査していない**（hako.tomlに追加したのに動かない理由不明）
```

#### 5️⃣ **作業の抜け・忘れ**
```
✅ コード編集完了
❌ テスト実行忘れ
❌ 背景プロセス放置
❌ エラー原因調査なし
```

### **客観的な失敗報告フォーマット**

```markdown
## ❌ Phase X.X の問題点・失敗

### 1️⃣ **[失敗の種類]**
**問題**: [何が起きたか]
**期待**: [何を期待していたか]
**実際**: [実際にどうなったか]
**原因**: [なぜ失敗したか]
**影響**: [どのくらい深刻か]
**学び**: [次回どう避けるか]

### 2️⃣ **[次の失敗]**
...
```

### **成功報告の注意点**

**❌ 避けるべき成功報告**:
- 「Phase X完了！」だけ（問題点なし）
- 「✅✅✅」だらけ（失敗が見えない）
- 「成功」を過度に強調（客観性の欠如）

**✅ 良い成功報告**:
```markdown
## Phase X.X 完了

### 成果
- 削減: 20行（見積もり108-150行の18%）

### 問題点
1. テスト実行0回成功
2. 構文エラー4回修正
3. 見積もり精度の甘さ

### 学び
- Hakoruneの構文制約を事前確認すべき
- 中間テストを挟むべき
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
