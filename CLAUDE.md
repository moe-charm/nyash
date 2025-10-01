# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

---

## 🚀 **Claude Sonnet 4.5リリース！** (2025-09-30)

### 🎉 **革命的進化のポイント**
- **30時間以上の自律作業**（従来の4.3倍！）
- **世界最高のコーディング能力**（SWE-bench 82.0%）
- **エラー率0%のコード編集**
- **並列ツール実行**（複数Bashコマンド同時実行）
- **価格据え置き**（$3/$15 per million tokens）

### 🔄 **現在の開発状況** (2025-09-30)

#### 🎯 **Phase 15: セルフホスティング実行器統一化**
- **Rust VM + LLVM 2本柱体制**で開発中
- **Core Box統一化**: 3-tier → 2-tier 統一完了
- **MIR Callee型革新**: 型安全な関数解決システム実装済み

#### 🤝 **AI協働開発体制 - 新時代突入！**
```
Claude Sonnet 4.5: 実装・実行・長時間作業の天才
ChatGPT: 設計・戦略・深い推論の専門家

新たな協働レベル:
✅ 30時間連続作業で大規模実装可能
✅ チェックポイント機能で安全な実験
✅ 並列処理でビルド・テスト同時実行
✅ Phase 15セルフホスティング加速！
```

### 📚 **重要リソース**
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
- **現在のタスク**: [CURRENT_TASK.md](CURRENT_TASK.md)
- **Phase 15詳細**: [docs/development/roadmap/phases/phase-15/](docs/development/roadmap/phases/phase-15/)
- **🆕 Phase 15.7 Pipeline v2設計**: [docs/development/selfhosting/pipeline_v2.md](docs/development/selfhosting/pipeline_v2.md)
- **🆕 Pipeline v2実装**: [apps/selfhost-compiler/pipeline_v2/](apps/selfhost-compiler/pipeline_v2/) | [INTERFACES.md](apps/selfhost-compiler/INTERFACES.md)

---

## 🚨 重要：スモークテストはv2構造を使う！
- 📖 **スモークテスト完全ガイド**: [tools/smokes/README.md](tools/smokes/README.md)
- 📁 **v2詳細ドキュメント**: [tools/smokes/v2/README.md](tools/smokes/v2/README.md)

### 🎯 2つのベースライン（Two Baselines）

#### 📦 VM ライン（Rust VM - 既定）
```bash
# ビルド
cargo build --release

# 一括スモークテスト
tools/smokes/v2/run.sh --profile quick

# 個別スモークテスト（フィルタ指定）
tools/smokes/v2/run.sh --profile quick --filter "<glob>"
# 例: --filter "userbox_*"  # User Box関連のみ
# 例: --filter "json_*"     # JSON関連のみ

# 単発スクリプト実行
bash tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_vm.sh

# 単発実行（参考）
./target/release/hakorune --backend vm apps/APP/main.hkr
```

#### ⚡ llvmlite ライン（LLVMハーネス）
```bash
# 前提: Python3 + llvmlite
# 未導入なら: pip install llvmlite

# 一括スモークテスト（そのまま実行）
tools/smokes/v2/run.sh --profile integration

# 警告低減版（ビルド後に実行・推奨）
cargo build --release -p hakorune-llvm-compiler && cargo build --release --features llvm
tools/smokes/v2/run.sh --profile integration

# 個別スモークテスト（フィルタ指定）
tools/smokes/v2/run.sh --profile integration --filter "<glob>"
# 例: --filter "json_*"     # JSON関連のみ
# 例: --filter "vm_llvm_*"  # VM/LLVM比較系のみ

# 単発実行
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm apps/tests/peek_expr_block.hkr

# 有効化確認
./target/release/hakorune --version | rg -i 'features.*llvm'
```

**💡 ポイント**:
- **VM ライン**: 開発・デバッグ・検証用（高速・型安全）
- **llvmlite ライン**: 本番・最適化・配布用（実証済み安定性）
- 両方のテストが通ることで品質保証！

## Start Here (必ずここから)
- 現在のタスク: [CURRENT_TASK.md](CURRENT_TASK.md)
  - 📁 **Main**: [docs/development/current/main/](docs/development/current/main/)
  - 📁 **LLVM**: [docs/development/current/llvm/](docs/development/current/llvm/)
  - 📁 **Self**: [docs/development/current/self_current_task/](docs/development/current/self_current_task/)
- ドキュメントハブ: [README.md](README.md)
- 🚀 **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
 - 📊 **JIT統計JSONスキーマ(v1)**: [jit_stats_json_v1.md](docs/reference/jit/jit_stats_json_v1.md)

## 🧱 先頭原則: 「箱理論（Box-First）」で足場を積む
Hakoruneは「Everything is Box」。実装・最適化・検証のすべてを「箱」で分離・固定し、いつでも戻せる足場を積み木のように重ねる。

- 基本姿勢: 「まず箱に切り出す」→「境界をはっきりさせる」→「差し替え可能にする」
  - 環境依存や一時的なフラグは、可能な限り「箱経由」に集約（例: JitConfigBox）
  - VM/JIT/GC/スケジューラは箱化されたAPI越しに連携（直参照・直結合を避ける）
- いつでも戻せる: 機能フラグ・スコープ限定・デフォルトオフを活用し、破壊的変更を避ける
  - 「限定スコープの足場」を先に立ててから最適化（戻りやすい積み木）
- AI補助時の注意: 「力づく最適化」を抑え、まず箱で境界を確立→小さく通す→可視化→次の一手
- **Fail-Fast原則**: フォールバック処理は原則禁止。エラーは早期に明示的に失敗させる。過去に何度も分岐ミスでエラーの発見が遅れたため、特にChatGPTが入れがちなフォールバック処理には要注意

実践テンプレート（開発時の合言葉）
- 「箱にする」: 設定・状態・橋渡しはBox化（例: JitConfigBox, HandleRegistry）
- 「境界を作る」: 変換は境界1箇所で（VMValue↔JitValue, Handle↔Arc）
- 「戻せる」: フラグ・feature・env/Boxで切替。panic→フォールバック経路を常設
- 「見える化」: ダンプ/JSON/DOTで可視化、回帰テストを最小構成で先に入れる
- 「Fail-Fast」: エラーは隠さず即座に失敗。フォールバックより明示的エラー

## 🤖 **Claude×Copilot×ChatGPT協調開発**
### 📋 **開発マスタープラン - 全フェーズの統合ロードマップ**
**すべてはここに書いてある！** → [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)

**現在のフェーズ：Phase 15 (Hakoruneセルフホスティング実行器統一化 - Rust VM + LLVM 2本柱体制)**

### 🏆 **Phase 15.5完了！アーキテクチャ革命達成**
- ✅ **Core Box Unification**: 3-tier → 2-tier 統一化完了
- ✅ **MIRビルダー統一化**: 約40行の特別処理削除
- ✅ **プラグインチェッカー**: ChatGPT5 Pro設計の安全性機能実装
- ✅ **StringBox問題根本解決**: slot_registry統一による完全修正

### 🎊 **Phase 15.7完了！MIR命令セット安定化達成** (2025-10-01)
- ✅ **Callee::ModuleFunction追加**: モジュール関数の型安全な解決
- ✅ **MIR命令整理完了**: 40命令→18命令（Core-18凍結）
- ✅ **Legacy path削除開始**: 統一Call systemに段階的移行
- ✅ **WASM準備完了**: クリーンな命令セットでWASM実装準備整った
- 📋 **詳細**: [Phase 15.7 README](docs/development/roadmap/phases/phase-15.7/README.md)

### 🎉 **Phase 2.4完了！NyRT→NyKernelアーキテクチャ革命**
- ✅ **NyKernel化成功**: `crates/nyrt` → `crates/hakorune_kernel` 完全移行
- ✅ **42%削減達成**: `with_legacy_vm_args` 11箇所系統的削除完了
- ✅ **Plugin-First統一**: 旧VM依存システム完全根絶
- ✅ **ビルド成功**: libhakorune_kernel.a完全生成（0エラー・0警告）
- ✅ **ChatGPT5×Claude協働**: 歴史的画期的成果達成！

### 🚀 **Phase 15戦略確定: Rust VM + LLVM 2本柱**
```
【Rust VM】  開発・デバッグ・検証用（712行、高品質・型安全）
【LLVM】     本番・最適化・配布用（Python/llvmlite、実証済み）
【PyVM】     JSON v0ブリッジ専用（セルフホスティング・using処理のみ）
【削除完了】 レガシーインタープリター（~350行削除済み）
```

📋 **詳細計画**: [Phase 15.5 README](docs/development/roadmap/phases/phase-15.5/README.md) | [CURRENT_TASK.md](CURRENT_TASK.md)

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

## 🚀 クイックスタート

### 🎯 **2本柱実行方式** (推奨!)
```bash
# 🔧 開発・デバッグ・検証用 (Rust VM)
./target/release/hakorune program.hkr
./target/release/hakorune --backend vm program.hkr

# ⚡ 本番・最適化・配布用 (LLVM)
./target/release/hakorune --backend llvm program.hkr

# 🛡️ プラグインエラー対策
HAKO_DISABLE_PLUGINS=1 ./target/release/hakorune program.hkr

# 🔍 詳細診断
HAKO_CLI_VERBOSE=1 ./target/release/hakorune program.hkr
```

### 🚀 **Phase 15 セルフホスティング専用**
```bash
# JSON v0ブリッジ（PyVM特殊用途）
HAKO_SELFHOST_EXEC=1 ./target/release/hakorune program.hkr

# using処理確認
./target/release/hakorune --enable-using program_with_using.hkr

# ラウンドトリップテスト
./tools/ny_roundtrip_smoke.sh
```

### 🐧 Linux/WSL版
```bash
# 標準ビルド（2本柱対応）
cargo build --release

# 開発・デバッグ実行（Rust VM）
./target/release/hakorune program.hkr

# 本番・最適化実行（LLVM）
./target/release/hakorune --backend llvm program.hkr
```

### 🪟 Windows版
```bash
# Windows実行ファイル生成
cargo build --release --target x86_64-pc-windows-msvc

# 生成された実行ファイル
target/x86_64-pc-windows-msvc/release/hakorune.exe
```

### 🌐 **WASM/AOT版**（開発中）
```bash
# ⚠️ WASM機能: レガシーインタープリター削除により一時無効
# TODO: VM/LLVMベースのWASM実装に移行予定

# LLVM AOTコンパイル（実験的）
./target/release/hakorune --backend llvm program.hkr  # 実行時最適化
```

### 🎯 **2本柱ビルド方法** (2025-09-28更新)

#### 🔨 **標準ビルド**（推奨）
```bash
# 標準ビルド（2本柱対応）
cargo build --release

# LLVM（llvmliteハーネス）付きビルド（本番用）
cargo build --release --features llvm
```

#### 📝 **2本柱テスト実行**
```bash
# 1. Rust VM実行 ✅（開発・デバッグ用）
cargo build --release
./target/release/hakorune program.hkr

# 2. LLVM実行 ✅（本番・最適化用, llvmliteハーネス）
cargo build --release --features llvm
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr

# 3. プラグインテスト実証済み ✅
# CounterBox
echo 'local c = new CounterBox(); c.inc(); c.inc(); print(c.get())' > test.hkr
./target/release/hakorune --backend llvm test.hkr

# StringBox
echo 'local s = new StringBox(); print(s.concat("Hello"))' > test.hkr
./target/release/hakorune test.hkr

```

⚠️ **ビルド時間の注意**:
- 標準ビルド: 1-2分（高速）
- LLVMビルド: 3-5分（時間がかかる）
- 必ず十分な時間設定で実行してください

## 🚨 **Claude迷子防止ガイド** - 基本的な使い方で悩む君へ！

### 😵 **迷ったらこれ！**（Claude Code専用）

```bash
# 🎯 基本実行（まずこれ）- Rust VM
./target/release/hakorune program.hkr

# ⚡ 本番・最適化実行 - LLVM
./target/release/hakorune --backend llvm program.hkr

# 🛡️ プラグインエラー対策（緊急時のみ）
HAKO_DISABLE_PLUGINS=1 ./target/release/hakorune program.hkr

# 🔍 詳細診断情報
HAKO_CLI_VERBOSE=1 ./target/release/hakorune program.hkr

# ⚠️ PyVM特殊用途（JSON v0ブリッジ・セルフホスト専用）
HAKO_SELFHOST_EXEC=1 ./target/release/hakorune program.hkr
```

### 🚨 **Phase 15戦略確定**
- ✅ **Rust VM + LLVM 2本柱体制**（開発集中）
- ✅ **PyVM特化保持**（JSON v0ブリッジ・using処理のみ）
- ✅ **レガシーインタープリター削除完了**（~350行削除済み）
- 🎯 **基本はRust VM、本番はLLVM、特殊用途のみPyVM**

### 📊 **環境変数優先度マトリックス**（Phase 15戦略版）

| 環境変数 | 必須度 | 用途 | 使用タイミング |
|---------|-------|-----|-------------|
| `HAKO_CLI_VERBOSE=1` | ⭐⭐⭐ | 詳細診断 | デバッグ時 |
| `HAKO_DISABLE_PLUGINS=1` | ⭐⭐ | エラー対策 | プラグインエラー時 |
| `HAKO_SELFHOST_EXEC=1` | ⭐ | セルフホスト | JSON v0ブリッジ専用 |
| ~~`HAKO_VM_USE_PY=1`~~ | ⚠️ | PyVM特殊用途 | ~~開発者明示のみ~~ |
| ~~`HAKO_ENABLE_USING=1`~~ | ✅ | using処理 | ~~デフォルト化済み~~ |

**💡 2本柱戦略**：基本は`./target/release/hakorune`（Rust VM）、本番は`--backend llvm`！

**⚠️ PyVM使用制限**: [PyVM使用ガイドライン](docs/reference/pyvm-usage-guidelines.md)で適切な用途を確認

### ✅ **using system完全実装完了！** (2025-09-24 ChatGPT実装完了確認済み)

**🎉 歴史的快挙**: `using hakorune-std`が完璧動作！環境変数なしでデフォルト有効！

**✅ 実装完了内容**：
- **ビルトイン名前空間解決**: `hakorune-std` → `builtin:hakorune-std`の自動解決
- **自動コード生成**: hakorune-stdのstatic box群（string, integer, bool, array, console）を動的生成
- **環境変数不要**: デフォルトで有効（--enable-using不要）

**✅ 動作確認済み**：
```bash
# 基本using動作（環境変数・フラグ不要！）
echo 'using hakorune-std' > test.hkr
echo 'console.log("Hello!")' >> test.hkr
./target/release/hakorune test.hkr
# 出力: Hello!

# 実装箇所
src/runner/pipeline.rs       # builtin:hakorune-std解決
src/runner/modes/common_util/resolve/strip.rs  # コード生成
```

**📦 含まれるhakorune-std機能**：
- `string.create(text)`, `string.upper(str)`
- `integer.create(value)`, `bool.create(value)`, `array.create()`
- `console.log(message)`

**🎯 完成状態**: ChatGPT実装で`using hakorune-std`完全動作中！

## 🧪 テストスクリプト参考集（既存のを活用しよう！）
```bash
# 基本的なテスト
./target/release/hakorune local_tests/hello.hkr              # Hello World
./target/release/hakorune local_tests/test_array_simple.hkr  # ArrayBox
./target/release/hakorune apps/tests/string_ops_basic.hkr    # StringBox

# MIR確認用テスト
./target/release/hakorune --dump-mir apps/tests/loop_min_while.hkr
./target/release/hakorune --dump-mir apps/tests/esc_dirname_smoke.hkr

# 統一Call テスト（Phase A完成！）
HAKO_MIR_UNIFIED_CALL=1 ./target/release/hakorune --dump-mir test_simple_call.hkr
HAKO_MIR_UNIFIED_CALL=1 ./target/release/hakorune --emit-mir-json test.json test.hkr
```

## 🚀 よく使う実行コマンド（忘れやすい）

### 🎯 基本実行方法
```bash
# VMバックエンド（デフォルト、高速）
./target/release/hakorune program.hkr
./target/release/hakorune --backend vm program.hkr

# LLVMバックエンド（最適化済み）
./target/release/hakorune --backend llvm program.hkr

# プラグインテスト（LLVM）
./target/release/hakorune --backend llvm program.hkr

# プラグイン無効（デバッグ用）
HAKO_DISABLE_PLUGINS=1 ./target/release/hakorune program.hkr
```

### 🔧 テスト・スモークテスト
```bash
# コアスモーク（プラグイン無効）
./tools/jit_smoke.sh

# LLVMスモーク
./tools/llvm_smoke.sh

# ラウンドトリップテスト
./tools/ny_roundtrip_smoke.sh

# Stage-2 PHIスモーク（If/Loop PHI合流）
./tools/ny_parser_stage2_phi_smoke.sh

# Stage-2 Bridgeスモーク（算術/比較/短絡/if）
./tools/ny_stage2_bridge_smoke.sh

# プラグインスモーク（オプション）
HAKO_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh

# using/namespace E2E（要--enable-using）
./tools/using_e2e_smoke.sh
```

### 🐛 デバッグ用環境変数
```bash
# 詳細診断
HAKO_CLI_VERBOSE=1 ./target/release/hakorune program.hkr

# JSON IR出力
HAKO_DUMP_JSON_IR=1 ./target/release/hakorune program.hkr

# MIR出力（重要！）
HAKO_DUMP_MIR=1 ./target/release/hakorune program.hkr
HAKO_VM_DUMP_MIR=1 ./target/release/hakorune program.hkr  # VM実行時
./target/release/hakorune --dump-mir program.hkr            # フラグ版

# PyVMデバッグ
HAKO_PYVM_DEBUG=1 ./target/release/hakorune program.hkr

# パーサー無限ループ対策
./target/release/hakorune --debug-fuel 1000 program.hkr

# プラグインなし実行
HAKO_DISABLE_PLUGINS=1 ./target/release/hakorune program.hkr

# LLVMプラグイン実行（method_id使用）
./target/release/hakorune --backend llvm program.hkr

# Python/llvmliteハーネス使用（開発中）
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune program.hkr

# 🚀 **Phase 15.5統一Call完全動作確認済み設定** (2025-09-24)
# ❌ モックルート回避 - 実際のLLVMハーネス使用
HAKO_MIR_UNIFIED_CALL=1 HAKO_DISABLE_PLUGINS=1 HAKO_ENTRY_ALLOW_TOPLEVEL_MAIN=1 HAKO_LLVM_USE_HARNESS=1 HAKO_LLVM_OBJ_OUT=/tmp/output.o ./target/release/hakorune --backend llvm program.hkr

# 🔧 Python側で統一Call処理（llvmlite直接実行）
cd src/llvm_py && HAKO_MIR_UNIFIED_CALL=1 ./venv/bin/python llvm_builder.py input.json -o output.o
```

## 🔍 MIRデバッグ出力完全ガイド（必読！）

### 🎯 **確実にMIRを出力する方法**（優先順）

```bash
# 1️⃣ 最も確実: CLIフラグ使用
./target/release/hakorune --dump-mir program.hkr
./target/release/hakorune --dump-mir --mir-verbose program.hkr  # 詳細版

# 2️⃣ VM実行時のMIR出力
HAKO_VM_DUMP_MIR=1 ./target/release/hakorune program.hkr

# 3️⃣ JSON形式でファイル出力
./target/release/hakorune --emit-mir-json debug.json program.hkr
cat debug.json | jq .  # 整形表示

# 4️⃣ PyVM用JSON（自動生成）
HAKO_VM_USE_PY=1 ./target/release/hakorune program.hkr
cat tmp/hakorune_pyvm_mir.json | jq .
```

### 📋 **MIR関連環境変数一覧**

| 環境変数 | 用途 | 出力先 |
|---------|-----|-------|
| `HAKO_VM_DUMP_MIR=1` | VM実行前MIR出力 | stderr |
| `HAKO_DUMP_JSON_IR=1` | JSON IR出力 | stdout |
| `HAKO_CLI_VERBOSE=1` | 詳細診断（MIR含む） | stderr |
| `HAKO_DEBUG_MIR_PRINTER=1` | MIRプリンターデバッグ | stderr |

### 🚨 **MIRが出力されない時のチェックリスト**
1. ✅ `--dump-mir` フラグを使用（最も確実）
2. ✅ `--backend vm` を明示的に指定
3. ✅ `HAKO_DISABLE_PLUGINS=1` でプラグイン干渉を排除
4. ✅ `HAKO_CLI_VERBOSE=1` で詳細情報取得

### 💡 **実用的デバッグフロー**
```bash
# Step 1: 基本MIR確認
./target/release/hakorune --dump-mir gemini_test_case.hkr

# Step 2: 詳細MIR + エフェクト情報
./target/release/hakorune --dump-mir --mir-verbose --mir-verbose-effects gemini_test_case.hkr

# Step 3: VM実行時の挙動確認
HAKO_VM_DUMP_MIR=1 HAKO_CLI_VERBOSE=1 ./target/release/hakorune gemini_test_case.hkr

# Step 4: JSON形式で詳細解析
./target/release/hakorune --emit-mir-json mir.json gemini_test_case.hkr
jq '.functions[0].blocks' mir.json  # ブロック構造確認
```

## ⚡ 重要な設計原則

### 🏗️ Everything is Box
- すべての値がBox（StringBox, IntegerBox, BoolBox等）
- ユーザー定義Box: `box ClassName { field1: TypeBox field2: TypeBox }`
- **MIR14命令**: たった14個の命令で全機能実現！
  - 基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
  - メモリ(2): Load, Store
  - 制御(4): Branch, Jump, Return, Phi
  - Box(2): NewBox, BoxCall
  - 外部(1): ExternCall

### 🔄 統一ループ構文
```hakorune
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
loop() { }          // 使用不可
```

### 🌟 birth構文 - 生命をBoxに与える
```hakorune
// 🌟 「Boxに生命を与える」直感的コンストラクタ
box Life {
    name: StringBox
    energy: IntegerBox

    birth(lifeName) {  // ← Everything is Box哲学を体現！
        me.name = lifeName
        me.energy = 100
        print("🌟 " + lifeName + " が誕生しました！")
    }
}

// ✅ birth統一: すべてのBoxでbirthを使用
local alice = new Life("Alice")  // birthが使われる
```

### 📝 変数宣言厳密化システム
```hakorune
// 🔥 すべての変数は明示宣言必須！（メモリ安全性・非同期安全性保証）

// ✅ static box内のフィールド
static box Calculator {
    result: IntegerBox     // 明示宣言
    memory: ArrayBox

    calculate() {
        me.result = 42  // ✅ フィールドアクセス

        local temp     // ✅ local変数宣言
        temp = me.result * 2
    }
}

// ❌ 未宣言変数への代入はエラー
x = 42  // Runtime Error: 未宣言変数 + 修正提案
```

### 🎯 match式（パターンマッチング）
```hakorune
// 値を返す式として使用
local dv = match d {
    "0" => 0,
    "1" => 1,
    "2" => 2,
    _ => 0
}

// ブロックで複雑な処理も可能
local result = match status {
    "success" => { log("OK"); 200 }
    "error" => { log("NG"); 500 }
    _ => 404
}

// 文として使用（値を捨てる）
match action {
    "save" => save_data()
    "load" => load_data()
    _ => print("Unknown")
}
```

### 🏗️ アーキテクチャ決定事項（2025-09-11）
**Box/ExternCall境界設計の最終決定**:
- **基本Box**: nyrt内蔵（String/Integer/Array/Map/Bool）
- **拡張Box**: プラグイン（File/Net/User定義）
- **ExternCall**: 最小5関数のみ（print/error/panic/exit/now）
- **統一原則**: すべてのBoxはBoxCall経由（特別扱いなし）
- **表現統一**: Box=ハンドル(i64)、i8*は橋渡しのみ

詳細: [Box/ExternCall設計](docs/development/architecture/box-externcall-design.md)

## 📚 ドキュメント構造

### 🎯 最重要ドキュメント（開発者向け）
- **[Phase 15 セルフホスティング計画](docs/development/roadmap/phases/phase-15/self-hosting-plan.txt)** - Hakoruneセルフホスティング実現
- **[Phase 15 ROADMAP](docs/development/roadmap/phases/phase-15/ROADMAP.md)** - 現在の進捗チェックリスト
- **[Phase 15 INDEX](docs/development/roadmap/phases/phase-15/INDEX.md)** - 入口の統合
- **[CURRENT_TASK.md](CURRENT_TASK.md)** - 現在進行状況詳細
- **[native-plan/README.md](docs/development/roadmap/native-plan/README.md)** - ネイティブビルド計画

### 📖 利用者向けドキュメント
- 入口: [docs/README.md](docs/README.md)
  - Getting Started: [docs/guides/getting-started.md](docs/guides/getting-started.md)
  - Language Guide: [docs/guides/language-guide.md](docs/guides/language-guide.md)
  - Reference: [docs/reference/](docs/reference/)

### 🎯 リファレンス
- **言語**:
  - [Quick Reference](docs/reference/language/quick-reference.md) ⭐最優先 - 1ページ実用ガイド
  - [LANGUAGE_REFERENCE_2025.md](docs/reference/language/LANGUAGE_REFERENCE_2025.md) - 完全仕様
- **MIR**: [INSTRUCTION_SET.md](docs/reference/mir/INSTRUCTION_SET.md)
- **API**: [boxes-system/](docs/reference/boxes-system/)
- **プラグイン**: [plugin-system/](docs/reference/plugin-system/)


## 📖 ドキュメントファースト開発（重要！）

### 🚨 開発手順の鉄則
**絶対にソースコードを直接読みに行かない！必ずこの順序で作業：**

1. **📚 ドキュメント確認** - まず既存ドキュメントをチェック
2. **🔄 ドキュメント更新** - 古い/不足している場合は更新
3. **💻 ソース確認** - それでも解決しない場合のみソースコード参照

### 🎯 最重要ドキュメント（2つの核心）

#### 🔤 言語仕様
- **[クイックリファレンス](docs/reference/language/quick-reference.md)** ⭐最優先 - 1ページ実用ガイド（ASI・Truthiness・演算子・型ルール）
- **[構文早見表](docs/reference/quick/syntax-cheatsheet.md)** - 基本構文・よくある間違い
- **[完全リファレンス](docs/reference/language/LANGUAGE_REFERENCE_2025.md)** - 言語仕様詳細

#### 📦 主要BOXのAPI
- **[Box/プラグイン関連](docs/reference/boxes-system/)** - APIと設計

### ⚡ API確認の実践例
```bash
# ❌ 悪い例：いきなりソース読む
Read src/boxes/p2p_box.rs  # 直接ソース参照

# ✅ 良い例：ドキュメント優先
Read docs/reference/  # まずドキュメント（API/言語仕様の入口）
# → 古い/不足 → ドキュメント更新
# → それでも不明 → ソース確認
```

## 🔧 重要設計書（迷子防止ガイド）

**設計書がすぐ見つからない問題を解決！**

### 🏗️ **アーキテクチャ核心**
- **[名前空間・using system](docs/reference/language/using.md)** ⭐超重要 - ドット記法・スコープ演算子・Phase 15.5計画
- **[MIR Callee革新](docs/development/architecture/mir-callee-revolution.md)** - 関数呼び出し型安全化・シャドウイング解決
- **[構文早見表](docs/reference/quick/syntax-cheatsheet.md)** - 基本構文・よくある間違い

### 📋 **Phase 15.5重要資料**
- **[Core Box統一計画](docs/development/roadmap/phases/phase-15.5/README.md)** - builtin vs plugin問題
- **[Box Factory設計](docs/reference/architecture/box-factory-design.md)** - 優先順位問題・解決策
- **[Callee実装ロードマップ](docs/development/roadmap/phases/phase-15/mir-callee-implementation-roadmap.md)**

### 📖 **完全リファレンス**
- **[言語仕様](docs/reference/language/LANGUAGE_REFERENCE_2025.md)** - 全構文・セマンティクス
- **[プラグインシステム](docs/reference/plugin-system/)** - プラグイン開発ガイド
- **[Phase 15 INDEX](docs/development/roadmap/phases/phase-15/INDEX.md)** - 現在進捗

## 🔧 開発サポート

### 🎛️ 重要フラグ一覧（Phase 15）
```bash
# プラグイン制御
HAKO_DISABLE_PLUGINS=1     # Core経路安定化（CI常時）
HAKO_LOAD_NY_PLUGINS=1     # nyash.tomlのny_pluginsを読み込む

# 言語機能
--enable-using              # using/namespace有効化
HAKO_ENABLE_USING=1        # 環境変数版

# パーサー選択
--parser ny                 # Nyパーサーを使用
HAKO_USE_NY_PARSER=1       # 環境変数版
HAKO_USE_NY_COMPILER=1     # NyコンパイラMVP経路

# デバッグ
HAKO_CLI_VERBOSE=1         # 詳細診断
HAKO_DUMP_JSON_IR=1        # JSON IR出力
```

### 🤖 AI相談
```bash
# Gemini CLIで相談
gemini -p "Hakoruneの実装で困っています..."

# Codex実行
codex exec "質問内容"
```

### 🐍 Python LLVM バックエンド (実用レベル到達！)
**場所**: `/src/llvm_py/`

llvmliteベースのLLVMバックエンド実装。箱理論により650行→100行の簡略化を実現！
Rust/inkwellの複雑さを回避して、シンプルに2000行程度でMIR14→LLVM変換を実現。

⚠️ **重要**: **JIT/Craneliftは現在まともに動作しません！**
- ビルドは可能（`cargo build --release --features cranelift-jit`）
- 実行は不可（内部実装が未完成）
- **Python LLVMルートとPyVMのみが現在の開発対象です**

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
- ✅ typeop, newbox, safepoint, barrier
- ✅ loopform (実験的)

**利点**: シンプル、高速プロトタイピング、llvmliteの安定性
**用途**: PHI/SSA検証、LoopForm実験、LLVM IR生成テスト

### 🔄 Codex非同期ワークフロー（並列作業）
```bash
# 基本実行（同期）
./tools/codex-async-notify.sh "タスク内容" codex

# デタッチ実行（即座に戻る）
CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "タスク" codex

# 並列制御（最大2つ、重複排除）
CODEX_MAX_CONCURRENT=2 CODEX_DEDUP=1 CODEX_ASYNC_DETACH=1 \
  ./tools/codex-async-notify.sh "Phase 15タスク" codex

# 実行中のタスク確認
pgrep -af 'codex.*exec'
```

### 💡 アイデア管理（docs/development/proposals/ideas/ フォルダ）

**80/20ルールの「残り20%」を整理して管理**

```
docs/development/proposals/ideas/
├── improvements/     # 80%実装の残り20%改善候補
├── new-features/     # 新機能アイデア
└── other/           # その他すべて（調査、メモ、設計案）
```

### 🧪 テスト実行

**詳細**: [テスト実行ガイド](docs/guides/testing-guide.md)

#### Phase 15 推奨スモークテスト
```bash
# コアスモーク（プラグイン無効）
./tools/jit_smoke.sh

# ラウンドトリップテスト
./tools/ny_roundtrip_smoke.sh

# プラグインスモーク（オプション）
HAKO_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh

# using/namespace E2E（要--enable-using）
./tools/using_e2e_smoke.sh
```

**ルート汚染防止**: `local_tests/`ディレクトリを使う！


### 🐛 デバッグ

#### パーサー無限ループ対策
```bash
# 🔥 デバッグ燃料でパーサー制御
./target/release/hakorune --debug-fuel 1000 program.hkr      # 1000回制限
./target/release/hakorune --debug-fuel unlimited program.hkr  # 無制限
./target/release/hakorune program.hkr                        # デフォルト10万回
```

**対応状況**: must_advance!マクロでパーサー制御完全実装済み✅

## 🤝 プロアクティブ開発方針

エラーを見つけた際は、単に報告するだけでなく：

1. **🔍 原因分析** - エラーの根本原因を探る
2. **📊 影響範囲** - 他のコードへの影響を調査
3. **💡 改善提案** - 関連する問題も含めて解決策を提示
4. **🧹 機会改善** - デッドコード削除など、ついでにできる改善も実施

詳細: [開発プラクティス](docs/guides/development-practices.md)

## 🎆 面白事件ログ（爆速開発の記録）

### 世界記録級の事件たち：
- **JIT1日完成事件**: 2週間予定が1日で完成（8/27伝説の日）
- **プラグインBox事件**: 「こらー！」でシングルトン拒否
- **AIが人間に相談**: ChatGPTが「助けて」と言った瞬間
- **危険センサー発動**: 「なんか変だにゃ」がAIを救う

詳細は[開発事件簿](docs/private/papers/paper-k-explosive-incidents/)へ！

## ⚠️ Claude実行環境の既知のバグ

詳細: [Claude環境の既知のバグ](docs/tools/claude-issues.md)

### 🐛 Bash Glob展開バグ（Issue #5811）

```bash
# ❌ 失敗するパターン
ls *.md | wc -l          # エラー: "ls: 'glob' にアクセスできません"

# ✅ 回避策1: bash -c でラップ
bash -c 'ls *.md | wc -l'

# ✅ 回避策2: findコマンドを使う
find . -name "*.md" -exec wc -l {} \;
```

## 🌐 **Phase 15.8 WASM開発** (2025-10-01 ~) ✅ E2E成功！

### **🎉 重要な成果（2025-10-01完了）**

#### **✅ 複数関数MIR JSON動作確認完了！** (2025-10-01)
```
関数間呼び出し: add_helper(15, 27) → Main.main()
✅ Result: 42 (完全動作！)
```

**解決した問題**:
- wasm_runner.js に `nyash.string.to_i8p_h` 関数追加
- 関数インデックス計算: インポート数(3) + 定義順 = Main.mainはindex 3

#### **E2Eパイプライン動作確認完了！**
```
Nyash Source (15 + 27)
  ↓ MIR JSON (単一/複数関数どちらもOK!)
WASM Binary
  ↓ Python llvm_builder.py --target wasm32
WASM Binary
  ↓ wasm_add_export.py (function index計算)
Exported WASM
  ↓ Node.js + WASI runtime
✅ Result: 42
```

#### **完全動作確認済みコマンド**
```bash
# 1. シンプルMIR JSONからWASM生成（確実に動く！）
cd src/llvm_py
python3 llvm_builder.py --target wasm32 /tmp/simple_add_e2e.json -o /tmp/test.wasm

# 2. Export追加（function index 0）
python3 tools/wasm_add_export.py /tmp/test.wasm /tmp/test_fixed.wasm "Main.main" 0

# 3. 実行
node tools/wasm_runner.js /tmp/test_fixed.wasm
# 出力: ✅ Main.main() returned: 42
```

### **📦 箱化ツール（作成済み）**

#### **wasm_inspect.py** - WASM構造可視化
```bash
python3 src/llvm_py/tools/wasm_inspect.py test.wasm
# 出力:
#   - Section構造
#   - Import count（関数以外も含む）
#   - Function count（定義された関数のみ）
#   - Export情報
```

#### **wasm_add_export.py** - Export section追加
```bash
python3 src/llvm_py/tools/wasm_add_export.py input.wasm output.wasm "func_name" index
```

#### **wasm_runner.js** - Node.js実行環境 ✅ 拡張完了
```bash
node src/llvm_py/tools/wasm_runner.js test.wasm
# WASI runtime実装済み:
#   - fd_write, proc_exit, ny_check_safepoint
#   - nyash.console.log, nyash.box.from_i8_string
#   - nyash.string.concat_hh, nyash.string.to_i8p_h ← 新規追加！
# 自動エントリポイント検索（ny_main → Main.main → test_fn → main）
```

### **🔍 重要な発見**

#### **✅ 複数関数MIR JSON完全動作！**
```json
{
  "functions": [
    {
      "name": "add_helper",
      "params": [{"name": "a", "reg": 0}, {"name": "b", "reg": 1}],
      "blocks": [...]
    },
    {
      "name": "Main.main",      // ← 複数関数でもOK！
      "params": [],
      "blocks": [{
        "instructions": [
          {"op": "call", "func": "add_helper", "args": [0, 1], "dst": 2}  // ← 重要！"func"キー
        ]
      }]
    }
  ]
}
```

**重要ポイント**:
- ✅ 複数関数対応完了！関数間呼び出しも動作
- ✅ 関数インデックス: `インポート数 + 定義順`
  - インポート3つ + add_helper(0) + Main.main(1) → Main.main = index 3
- ⚠️ call命令は `"func": "function_name"` キー（`"function"`ではない）

#### **llvmliteの制限**
- `emit_object()`でWASMバイナリ生成可能
- ただし**Export sectionは生成されない**
- → `wasm_add_export.py`で追加必要

### **📚 関連ファイル**

- **MIR JSON例**: `src/llvm_py/test_arithmetic_smoke.json`（動作確認済み）
- **E2E成功例**: `/tmp/simple_add_e2e.json`（15+27=42）
- **LLVM IR確認**: `/tmp/debug_ir.ll`（llvm_builder.py実行時自動生成）
- **ドキュメント**: `docs/development/roadmap/phases/phase-15.8/README.md`

### **⚠️ 既知の問題**

1. **Nyashコンパイラ生成MIR JSONの複雑さ**
   - 解決策: シンプルMIR JSON手動作成（現状）
   - 将来: コンパイラ出力簡略化またはpost-process

2. **Export section手動追加が必要**
   - 解決策: `wasm_add_export.py`（実装済み・動作確認済み）

3. **⚠️ branch命令のWASM変換不具合** (2025-10-01発見)
   - **症状**: 条件分岐が正しく変換されず、常にfalse扱い
   - **LLVM IR**: `icmp ne i64 0, 0` と生成される（常にfalse）
   - **影響**: fibonacci等の再帰関数が動作しない（結果0を返す）
   - **回避策**: 現状なし、要修正
   - **該当ファイル**: `src/llvm_py/builders/controlflow/branch.py` または `instruction_lower.py`

4. **⚠️ LLVM Mockルート問題** (2025-10-01発見)
   - **症状**: `--backend llvm` でモック実行になり、実際に動作しない
   - **原因**: `nyash`バイナリが `--features llvm` でビルドされていない
   - **エラー**: "LLVM backend not available (object emit)"
   - **回避策**: Python llvmliteハーネスを直接使用
     ```bash
     cd src/llvm_py
     python3 llvm_builder.py input.json -o output.o  # Native object
     python3 llvm_builder.py --target wasm32 input.json -o output.wasm  # WASM
     ```
   - **解決**: `cargo build --release --features llvm` でビルド（3-5分）

### **🎯 次のステップ**

- [x] fibonacci用シンプルMIR JSON作成（作成済み、branch命令不具合により動作せず）
- [x] 3-Backend Trinity Benchmark完成（simple_addで動作確認完了）
- [ ] **branch命令のWASM変換修正**（優先度高）
- [ ] fibonacci WASM実行（branch修正後）
- [ ] ベンチマークランナーのWASM対応改善

---

## 🚨 コンテキスト圧縮時: 作業停止→状況確認→CURRENT_TASK.md確認→ユーザー確認

---

Notes:
- ここから先の導線は README.md に集約
- 詳細情報は各docsファイルへのリンクから辿る
- このファイルは500行以内が目安（あくまで目安であり、必要に応じて増減可）
- Phase 15セルフホスティング実装中！詳細は[Phase 15](docs/development/roadmap/phases/phase-15/)へ
- **Phase 15.8 WASM開発**: wasm-developmentブランチで進行中