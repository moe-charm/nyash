# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

---

## 🔄 **現在の開発状況** (2025-09-28)

### 🎯 **Phase 15: セルフホスティング実行器統一化**
- **Rust VM + LLVM 2本柱体制**で開発中
- **Core Box統一化**: 3-tier → 2-tier 統一完了
- **MIR Callee型革新**: 型安全な関数解決システム実装済み

### 🤝 **AI協働開発体制**
```
Claude（私）: 戦略・分析・レビュー
ChatGPT: 実装・検証

現在の合意:
✅ Phase 15集中（セルフホスト優先）
✅ Builder根治は段階的（3 Phase戦略）
✅ 息が合っている状態: 良好
```

### 📚 **重要リソース**
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
- **現在のタスク**: [CURRENT_TASK.md](CURRENT_TASK.md)
- **Phase 15詳細**: [docs/development/roadmap/phases/phase-15/](docs/development/roadmap/phases/phase-15/)

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
./target/release/nyash --backend vm apps/APP/main.nyash
```

#### ⚡ llvmlite ライン（LLVMハーネス）
```bash
# 前提: Python3 + llvmlite
# 未導入なら: pip install llvmlite

# 一括スモークテスト（そのまま実行）
tools/smokes/v2/run.sh --profile integration

# 警告低減版（ビルド後に実行・推奨）
cargo build --release -p nyash-llvm-compiler && cargo build --release --features llvm
tools/smokes/v2/run.sh --profile integration

# 個別スモークテスト（フィルタ指定）
tools/smokes/v2/run.sh --profile integration --filter "<glob>"
# 例: --filter "json_*"     # JSON関連のみ
# 例: --filter "vm_llvm_*"  # VM/LLVM比較系のみ

# 単発実行
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/peek_expr_block.nyash

# 有効化確認
./target/release/nyash --version | rg -i 'features.*llvm'
```

**💡 ポイント**:
- **VM ライン**: 開発・デバッグ・検証用（高速・型安全）
- **llvmlite ライン**: 本番・最適化・配布用（実証済み安定性）
- 両方のテストが通ることで品質保証！

## ⚡ **WSL2高速化ガイド** - 開発速度3〜5倍！

### 🚀 **重要：WSL側にプロジェクトを配置すると爆速！**

**📊 実測データ**：
```
ファイルI/O: 3〜5倍高速
cargo build: 2〜3倍高速
スモークテスト: 6分 → 2分（3倍）
1日の開発: 累計30分以上の節約！
```

**🎯 推奨配置**：
```bash
# WSL側に配置（推奨）
/home/tomoaki/git/nyash-project/nekorune-wasm/

# Windowsエクスプローラーからアクセス
\\wsl.localhost\Ubuntu\home\tomoaki\git\nyash-project\

# VSCodeでも開ける
code \\wsl.localhost\Ubuntu\home\tomoaki\git\nyash-project\nekorune-wasm
```

**📂 移行手順**：
```bash
# 1. WSL側にディレクトリ作成
mkdir -p /home/tomoaki/git
cd /home/tomoaki/git

# 2. git clone（推奨）
git clone <リポジトリURL> nyash-project
cd nyash-project/nekorune-wasm
git checkout wasm-development

# 3. ビルド確認
cargo build --release
```

**💾 バックアップ設定**：
```batch
# rclone バックアップ設定例
# C:\git と WSL側を両方バックアップする場合
rclone sync "C:\git" "%BACKUP_ROOT%\git" ...
rclone sync "\\wsl.localhost\Ubuntu\home\tomoaki\git" "%BACKUP_ROOT%\wsl_git" ...

# WSL側に一本化する場合
rclone sync "\\wsl.localhost\Ubuntu\home\tomoaki\git" "%BACKUP_ROOT%\git" ...
```

**⚠️ 注意点**：
- `/mnt/c`（Windowsファイルシステム）は避ける
- WSL側のLinuxネイティブファイルシステムを使用
- Windowsからは`\\wsl.localhost\Ubuntu\`でアクセス可能

## Start Here (必ずここから)
- 現在のタスク: [CURRENT_TASK.md](CURRENT_TASK.md)
  - 📁 **Main**: [docs/development/current/main/](docs/development/current/main/)
  - 📁 **LLVM**: [docs/development/current/llvm/](docs/development/current/llvm/)
  - 📁 **Self**: [docs/development/current/self_current_task/](docs/development/current/self_current_task/)
- ドキュメントハブ: [README.md](README.md)
- 🚀 **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
 - 📊 **JIT統計JSONスキーマ(v1)**: [jit_stats_json_v1.md](docs/reference/jit/jit_stats_json_v1.md)

## 🧱 先頭原則: 「箱理論（Box-First）」で足場を積む
Nyashは「Everything is Box」。実装・最適化・検証のすべてを「箱」で分離・固定し、いつでも戻せる足場を積み木のように重ねる。

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

**現在のフェーズ：Phase 15 (Nyashセルフホスティング実行器統一化 - Rust VM + LLVM 2本柱体制)**

### 🏆 **Phase 15.5完了！アーキテクチャ革命達成**
- ✅ **Core Box Unification**: 3-tier → 2-tier 統一化完了
- ✅ **MIRビルダー統一化**: 約40行の特別処理削除
- ✅ **プラグインチェッカー**: ChatGPT5 Pro設計の安全性機能実装
- ✅ **StringBox問題根本解決**: slot_registry統一による完全修正

### 🚀 **Phase 15.8開始！LLVM→WASM実装** (2025-10-01 ~)
- 🌿 **専用ブランチ**: `wasm-development` (← `selfhost`からfork)
- 🎯 **目標**: MIR18命令 → WASM変換、ブラウザ/エッジ環境対応
- 📋 **戦略**: llvm_py拡張（既存800行活用）+ WASI runtime連携
- 📚 **計画書**: [Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)

#### **進捗管理方針（Phase 15.8専用）**
```
📊 進捗追跡に使用する3つのドキュメント:

1. CLAUDE.md（このファイル）
   - Phase 15.8の進捗サマリー
   - Week単位の完了状況
   - 重要なマイルストーン記録

2. CURRENT_TASK.md
   - 現在進行中のタスク詳細
   - 次のアクション項目
   - 問題・課題の記録

3. docs/development/roadmap/phases/phase-15.8/README.md
   - 全体計画・タイムライン
   - 技術詳細・設計方針
   - 成功条件・成果物

🔄 更新頻度:
- CLAUDE.md: Week完了時（週1回）
- CURRENT_TASK.md: タスク切り替え時（日次）
- Phase 15.8 README.md: 計画変更時のみ（不定期）
```

### 🚀 **Phase 15.8開始！LLVM→WASM実装** (2025-10-01 ~)
- 🌿 **専用ブランチ**: `wasm-development` (← `selfhost`からfork)
- 🎯 **目標**: MIR18命令 → WASM変換、ブラウザ/エッジ環境対応
- 📋 **戦略**: llvm_py拡張（既存800行活用）+ WASI runtime連携
- 📚 **計画書**: [Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)

#### **Week 1進捗** (2025-10-01 ~ 10-07) ✅ **完了！**
- ✅ **Phase 1.1**: llvmlite WASM初期化 [完了]
  - targetパラメータ追加（native/wasm32）
  - wasm32-unknown-wasi triple初期化
  - module.triple設定
  - 統合テストスクリプト作成
  - **成果**: native/WASM両方のコンパイル成功、triple検証PASS

- ✅ **Phase 1.2/1.3**: WASMビルドパイプライン [完了]
  - WASM calling convention調整（external linkage）
  - llvmliteで直接WASMバイナリ生成成功
  - tools/build_wasm.sh作成（MIR JSON → WASM）
  - tools/wasm_runner.js作成（Node.js実行）
  - **成果**: WASMバイナリ生成成功（102 bytes）
  - **発見**: llvmliteだけでWASM生成可能！LLC/wasm-ld不要
  - **制限**: 関数エクスポートにLLVM toolchain推奨

#### **Week 1総括**
🎉 **予定以上の成果！**
- 当初計画: llvmlite初期化 + calling convention + ビルドスクリプト
- 実際成果: 上記 + WASMバイナリ直接生成 + Node.js実行環境
- 重要発見: LLVMツールチェーン不要でWASM生成可能（llvmlite内蔵）

#### **Week 2進捗** (2025-10-08 ~ 10-14) ✅ **完了！**
- ✅ **Phase 2.1**: 関数エクスポート解決 + 基本パイプライン [完了]
  - WASI runtime実装（fd_write, proc_exit）
  - wasm_runner.js更新（BigInt対応）
  - Python自己完結型ツールチェーン確立

- ✅ **Phase 2.2**: WASI fd_write実装 [完了]
  - externcall.py更新（nyrt_print → nyash.console.log）

- ✅ **Phase 2.3**: 文字列constants処理実装 [完了]
  - function_lower.pyに40+行追加
  - グローバル文字列リテラル生成
  - **🎉 Hello World WASM実行成功！**（"Hello, WASM!" 出力確認）

- ✅ **Phase 2.4**: binop演算完全実装 [完了]
  - binop.py既存実装確認（269行）
  - 算術演算: +, -, *, /, %
  - ビット演算: &, |, ^, <<, >>
  - **🎉 全演算WASM動作確認完了！**（44 = 15+12+12+5）

- ✅ **Phase 2.5**: 箱理論LLVM/WASM分離設計 [完了]
  - targets/モジュール構造確立（base/wasm/native/factory）
  - 責任分離達成（WASM/Nativeの違いを局所化）
  - **🎉 箱理論実践完了！**

- ✅ **Phase 2.6**: compare/branch/jump実装 [完了]
  - compare演算: 10 > 5 = true (1) → 1 * 100 = 100 ✅
  - branch分岐: if (10 > 5) then 100 else 200 → 100 ✅
  - jump無条件: jump to block → 42 ✅
  - **🎉 制御フロー完全動作確認！**

- ✅ **Phase 2.7**: スモークテスト整備 [完了]
  - arithmetic_smoke.json, compare_smoke.json, control_flow_smoke.json
  - run_wasm_smoke_tests.sh（一括実行スクリプト）
  - **🎉 回帰テスト体制確立完了！**（3テスト全PASS）

#### **Week 2総括** (2025-10-01)
**達成事項**:
- 🎉 P0課題完全解決（関数エクスポート → Python自己完結型ツール）
- 🎉 Hello World WASM実行成功（完全パイプライン確立）
- 🎉 binop全演算WASM動作確認（既存実装そのまま動作）
- 🎉 箱理論実装完了（targets/モジュール分離）
- 🎉 制御フロー完全動作（compare/branch/jump）
- 🎉 回帰テスト体制確立（3スモークテスト + 自動実行）

**重要発見**:
- ✅ llvmliteだけでWASM生成可能（LLC/wasm-ld不要）
- ✅ 既存LLVM実装がそのままWASM動作（追加実装不要）
- ✅ Python自己完結型（LLVMツールチェーン不要）

#### **Week 3進捗** (2025-10-15 ~ 10-21) 🔥 **調査中**
- ✅ **Phase 3.1-A**: 根本原因特定完了 [2025-10-01]
  - block_lower.pyでPHI命令スキップを発見
  - resolver.pyで重複PHI生成を発見
  - vmapの参照不一致を特定
  - **手法**: ultrathink + LLVM IR直接確認 + PhiHandler verbose

- ✅ **Phase 3.1-B**: 箱化実装完了 [2025-10-01]
  - **PhiHandler** (197行) - PHI処理統一ハンドラー
  - **InstructionContext** (98行) - 命令コンテキスト箱化
  - block_lower.py, instruction_lower.py, resolver.py修正
  - **成果**: 完全な箱理論実践（分離・境界・見える化）

- ✅ **Phase 3.1-C**: テスト完全成功 [2025-10-01]
  - test_phi_if.json実行成功
  - 正しいLLVM IR生成: `%"phi_6" = phi i64 [100, %"bb1"], [200, %"bb2"]`
  - コンパイル成功: `tmp/nyash_llvm_py.o`
  - **成果**: PHI先頭配置・正しい値・重複なし・完全動作✅

- 🔥 **Phase 3.3**: ループPHI実装（調査中） [2025-10-02]
  - ✅ test_phi_loop.json実装済み（while/loop PHI + self-loop back-edge）
  - ✅ PhiHandler forward reference対応完了（incomplete_phis機構）
  - 🐛 **LLVM IR構文エラー発見**: `phi i64 [0, %"bb0"]` ← 生の数値問題
  - 📋 次の調査: val型確認、llvmlite内部動作、block_end_values取得確認

- 📋 **Phase 3.4**: ベンチマークシステム構築（ループPHI修正後）
- 📋 **Phase 3.5**: Parity確認（予定）

#### **Week 3総括** (2025-10-01完了) 🎉🎉🎉
**達成事項**:
- 🏆 **PHI処理完全修正**: 3つの根本原因特定・解決
- 🏆 **箱化実装完了**: 2つの新規箱クラス（295行）
- 🏆 **テスト完全成功**: if文PHIテストPASS
- 📝 **新規ファイル**: `phi_handler.py` (197行), `instruction_context.py` (98行), `test_phi_if.json`
- 🔧 **修正ファイル**: block_lower, instruction_lower, resolver, mir_call, llvm_builder (5ファイル)

**重要成果**:
- ✅ PHI命令が正しい位置（ブロック先頭）に生成
- ✅ PHI値が正しい（100, 200）
- ✅ 重複なし（1つのPHIのみ）
- ✅ コンパイル成功
- ✅ 箱理論実践完了（PhiHandler, InstructionContext）

**技術革新**:
- 箱化によるPHI処理の完全統一
- vmap二重登録による重複回避
- ultrathink調査手法の確立

### 🎉 **Phase 2.4完了！NyRT→NyKernelアーキテクチャ革命**
- ✅ **NyKernel化成功**: `crates/nyrt` → `crates/nyash_kernel` 完全移行
- ✅ **42%削減達成**: `with_legacy_vm_args` 11箇所系統的削除完了
- ✅ **Plugin-First統一**: 旧VM依存システム完全根絶
- ✅ **ビルド成功**: libnyash_kernel.a完全生成（0エラー・0警告）
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
./target/release/nyash program.nyash
./target/release/nyash --backend vm program.nyash

# ⚡ 本番・最適化・配布用 (LLVM)
./target/release/nyash --backend llvm program.nyash

# 🛡️ プラグインエラー対策
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash

# 🔍 詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/nyash program.nyash
```

### 🚀 **Phase 15 セルフホスティング専用**
```bash
# JSON v0ブリッジ（PyVM特殊用途）
NYASH_SELFHOST_EXEC=1 ./target/release/nyash program.nyash

# using処理確認
./target/release/nyash --enable-using program_with_using.nyash

# ラウンドトリップテスト
./tools/ny_roundtrip_smoke.sh
```

### 🐧 Linux/WSL版
```bash
# 標準ビルド（2本柱対応）
cargo build --release

# 開発・デバッグ実行（Rust VM）
./target/release/nyash program.nyash

# 本番・最適化実行（LLVM）
./target/release/nyash --backend llvm program.nyash
```

### 🪟 Windows版
```bash
# Windows実行ファイル生成
cargo build --release --target x86_64-pc-windows-msvc

# 生成された実行ファイル
target/x86_64-pc-windows-msvc/release/nyash.exe
```

### 🌐 **WASM/AOT版**（開発中）
```bash
# ⚠️ WASM機能: レガシーインタープリター削除により一時無効
# TODO: VM/LLVMベースのWASM実装に移行予定

# LLVM AOTコンパイル（実験的）
./target/release/nyash --backend llvm program.nyash  # 実行時最適化
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
./target/release/nyash program.nyash

# 2. LLVM実行 ✅（本番・最適化用, llvmliteハーネス）
cargo build --release --features llvm
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm program.nyash

# 3. プラグインテスト実証済み ✅
# CounterBox
echo 'local c = new CounterBox(); c.inc(); c.inc(); print(c.get())' > test.nyash
./target/release/nyash --backend llvm test.nyash

# StringBox
echo 'local s = new StringBox(); print(s.concat("Hello"))' > test.nyash
./target/release/nyash test.nyash

```

⚠️ **ビルド時間の注意**:
- 標準ビルド: 1-2分（高速）
- LLVMビルド: 3-5分（時間がかかる）
- 必ず十分な時間設定で実行してください

## 🚨 **Claude迷子防止ガイド** - 基本的な使い方で悩む君へ！

### 😵 **迷ったらこれ！**（Claude Code専用）

```bash
# 🎯 基本実行（まずこれ）- Rust VM
./target/release/nyash program.nyash

# ⚡ 本番・最適化実行 - LLVM
./target/release/nyash --backend llvm program.nyash

# 🛡️ プラグインエラー対策（緊急時のみ）
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash

# 🔍 詳細診断情報
NYASH_CLI_VERBOSE=1 ./target/release/nyash program.nyash

# ⚠️ PyVM特殊用途（JSON v0ブリッジ・セルフホスト専用）
NYASH_SELFHOST_EXEC=1 ./target/release/nyash program.nyash
```

### 🚨 **Phase 15戦略確定**
- ✅ **Rust VM + LLVM 2本柱体制**（開発集中）
- ✅ **PyVM特化保持**（JSON v0ブリッジ・using処理のみ）
- ✅ **レガシーインタープリター削除完了**（~350行削除済み）
- 🎯 **基本はRust VM、本番はLLVM、特殊用途のみPyVM**

### 📊 **環境変数優先度マトリックス**（Phase 15戦略版）

| 環境変数 | 必須度 | 用途 | 使用タイミング |
|---------|-------|-----|-------------|
| `NYASH_CLI_VERBOSE=1` | ⭐⭐⭐ | 詳細診断 | デバッグ時 |
| `NYASH_DISABLE_PLUGINS=1` | ⭐⭐ | エラー対策 | プラグインエラー時 |
| `NYASH_SELFHOST_EXEC=1` | ⭐ | セルフホスト | JSON v0ブリッジ専用 |
| ~~`NYASH_VM_USE_PY=1`~~ | ⚠️ | PyVM特殊用途 | ~~開発者明示のみ~~ |
| ~~`NYASH_ENABLE_USING=1`~~ | ✅ | using処理 | ~~デフォルト化済み~~ |

**💡 2本柱戦略**：基本は`./target/release/nyash`（Rust VM）、本番は`--backend llvm`！

**⚠️ PyVM使用制限**: [PyVM使用ガイドライン](docs/reference/pyvm-usage-guidelines.md)で適切な用途を確認

### ✅ **using system完全実装完了！** (2025-09-24 ChatGPT実装完了確認済み)

**🎉 歴史的快挙**: `using nyashstd`が完璧動作！環境変数なしでデフォルト有効！

**✅ 実装完了内容**：
- **ビルトイン名前空間解決**: `nyashstd` → `builtin:nyashstd`の自動解決
- **自動コード生成**: nyashstdのstatic box群（string, integer, bool, array, console）を動的生成
- **環境変数不要**: デフォルトで有効（--enable-using不要）

**✅ 動作確認済み**：
```bash
# 基本using動作（環境変数・フラグ不要！）
echo 'using nyashstd' > test.nyash
echo 'console.log("Hello!")' >> test.nyash
./target/release/nyash test.nyash
# 出力: Hello!

# 実装箇所
src/runner/pipeline.rs       # builtin:nyashstd解決
src/runner/modes/common_util/resolve/strip.rs  # コード生成
```

**📦 含まれるnyashstd機能**：
- `string.create(text)`, `string.upper(str)`
- `integer.create(value)`, `bool.create(value)`, `array.create()`
- `console.log(message)`

**🎯 完成状態**: ChatGPT実装で`using nyashstd`完全動作中！

## 🧪 テストスクリプト参考集（既存のを活用しよう！）
```bash
# 基本的なテスト
./target/release/nyash local_tests/hello.nyash              # Hello World
./target/release/nyash local_tests/test_array_simple.nyash  # ArrayBox
./target/release/nyash apps/tests/string_ops_basic.nyash    # StringBox

# MIR確認用テスト
./target/release/nyash --dump-mir apps/tests/loop_min_while.nyash
./target/release/nyash --dump-mir apps/tests/esc_dirname_smoke.nyash

# 統一Call テスト（Phase A完成！）
NYASH_MIR_UNIFIED_CALL=1 ./target/release/nyash --dump-mir test_simple_call.nyash
NYASH_MIR_UNIFIED_CALL=1 ./target/release/nyash --emit-mir-json test.json test.nyash
```

## 🚀 よく使う実行コマンド（忘れやすい）

### 🎯 基本実行方法
```bash
# VMバックエンド（デフォルト、高速）
./target/release/nyash program.nyash
./target/release/nyash --backend vm program.nyash

# LLVMバックエンド（最適化済み）
./target/release/nyash --backend llvm program.nyash

# プラグインテスト（LLVM）
./target/release/nyash --backend llvm program.nyash

# プラグイン無効（デバッグ用）
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash
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
NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh

# using/namespace E2E（要--enable-using）
./tools/using_e2e_smoke.sh
```

### 🐛 デバッグ用環境変数
```bash
# 詳細診断
NYASH_CLI_VERBOSE=1 ./target/release/nyash program.nyash

# JSON IR出力
NYASH_DUMP_JSON_IR=1 ./target/release/nyash program.nyash

# MIR出力（重要！）
NYASH_DUMP_MIR=1 ./target/release/nyash program.nyash
NYASH_VM_DUMP_MIR=1 ./target/release/nyash program.nyash  # VM実行時
./target/release/nyash --dump-mir program.nyash            # フラグ版

# PyVMデバッグ
NYASH_PYVM_DEBUG=1 ./target/release/nyash program.nyash

# パーサー無限ループ対策
./target/release/nyash --debug-fuel 1000 program.nyash

# プラグインなし実行
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash

# LLVMプラグイン実行（method_id使用）
./target/release/nyash --backend llvm program.nyash

# Python/llvmliteハーネス使用（開発中）
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash program.nyash

# 🚀 **Phase 15.5統一Call完全動作確認済み設定** (2025-09-24)
# ❌ モックルート回避 - 実際のLLVMハーネス使用
NYASH_MIR_UNIFIED_CALL=1 NYASH_DISABLE_PLUGINS=1 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT=/tmp/output.o ./target/release/nyash --backend llvm program.nyash

# 🔧 Python側で統一Call処理（llvmlite直接実行）
cd src/llvm_py && NYASH_MIR_UNIFIED_CALL=1 ./venv/bin/python llvm_builder.py input.json -o output.o
```

## 🔍 MIRデバッグ出力完全ガイド（必読！）

### 🎯 **確実にMIRを出力する方法**（優先順）

```bash
# 1️⃣ 最も確実: CLIフラグ使用
./target/release/nyash --dump-mir program.nyash
./target/release/nyash --dump-mir --mir-verbose program.nyash  # 詳細版

# 2️⃣ VM実行時のMIR出力
NYASH_VM_DUMP_MIR=1 ./target/release/nyash program.nyash

# 3️⃣ JSON形式でファイル出力
./target/release/nyash --emit-mir-json debug.json program.nyash
cat debug.json | jq .  # 整形表示

# 4️⃣ PyVM用JSON（自動生成）
NYASH_VM_USE_PY=1 ./target/release/nyash program.nyash
cat tmp/nyash_pyvm_mir.json | jq .
```

### 📋 **MIR関連環境変数一覧**

| 環境変数 | 用途 | 出力先 |
|---------|-----|-------|
| `NYASH_VM_DUMP_MIR=1` | VM実行前MIR出力 | stderr |
| `NYASH_DUMP_JSON_IR=1` | JSON IR出力 | stdout |
| `NYASH_CLI_VERBOSE=1` | 詳細診断（MIR含む） | stderr |
| `NYASH_DEBUG_MIR_PRINTER=1` | MIRプリンターデバッグ | stderr |

### 🚨 **MIRが出力されない時のチェックリスト**
1. ✅ `--dump-mir` フラグを使用（最も確実）
2. ✅ `--backend vm` を明示的に指定
3. ✅ `NYASH_DISABLE_PLUGINS=1` でプラグイン干渉を排除
4. ✅ `NYASH_CLI_VERBOSE=1` で詳細情報取得

### 💡 **実用的デバッグフロー**
```bash
# Step 1: 基本MIR確認
./target/release/nyash --dump-mir gemini_test_case.nyash

# Step 2: 詳細MIR + エフェクト情報
./target/release/nyash --dump-mir --mir-verbose --mir-verbose-effects gemini_test_case.nyash

# Step 3: VM実行時の挙動確認
NYASH_VM_DUMP_MIR=1 NYASH_CLI_VERBOSE=1 ./target/release/nyash gemini_test_case.nyash

# Step 4: JSON形式で詳細解析
./target/release/nyash --emit-mir-json mir.json gemini_test_case.nyash
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

### 🌟 完全明示デリゲーション
```nyash
// デリゲーション構文（すべてのBoxで統一的に使える！）
box Child from Parent {  // from構文でデリゲーション
    birth(args) {  // コンストラクタは「birth」に統一
        from Parent.birth(args)  // 親の初期化
    }
    
    override method() {  // 明示的オーバーライド必須
        from Parent.method()  // 親メソッド呼び出し
    }
}

// ✅ ビルトインBox、プラグインBox、ユーザー定義Boxすべてで可能！
box MyString from StringBox { }          // ビルトインBoxから
box MyFile from FileBox { }             // プラグインBoxから
box Employee from Person { }            // ユーザー定義Boxから
box Multi from StringBox, IntegerBox { } // 多重デリゲーションも可能！
```

### 🔄 統一ループ構文
```nyash
// ✅ 唯一の正しい形式
loop(condition) { }

// ❌ 削除済み構文
while condition { }  // 使用不可
loop() { }          // 使用不可
```

### 🌟 birth構文 - 生命をBoxに与える
```nyash
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

### 🌟 ビルトインBox継承
```nyash
// ✅ Phase 12.7以降: birthで統一（packは廃止）
box EnhancedP2P from P2PBox {
    additionalData: MapBox
    
    birth(nodeId, transport) {
        from P2PBox.birth(nodeId, transport)  // 親のbirth呼び出し
        me.additionalData = new MapBox()
    }
}
```

### 🎯 正統派Nyashスタイル
```nyash
// 🚀 Static Box Main パターン - エントリーポイントの統一スタイル
static box Main {
    console: ConsoleBox    // フィールド宣言
    result: IntegerBox
    
    main() {
        // ここから始まる！他の言語と同じエントリーポイント
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        
        // local変数も使用可能
        local temp
        temp = 42
        me.result = temp
        
        return "Revolution completed!"
    }
}
```

### 📝 変数宣言厳密化システム
```nyash
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

### ⚡ 実装済み演算子
```nyash
// 論理演算子（完全実装）
not condition    // NOT演算子
a and b         // AND演算子  
a or b          // OR演算子

// 算術演算子
a / b           // 除算（ゼロ除算エラー対応済み）
a + b, a - b, a * b  // 加算・減算・乗算
```

### 🎯 match式（パターンマッチング）
```nyash
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

### ⚠️ 重要な注意点
```nyash
// ✅ 正しい書き方（Phase 12.7文法改革後）
box MyBox {
    field1: TypeBox
    field2: TypeBox
    
    birth() {
        // 初期化処理
    }
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
- **[Phase 15 セルフホスティング計画](docs/development/roadmap/phases/phase-15/self-hosting-plan.txt)** - Nyashセルフホスティング実現
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
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)** - 基本構文・よくある間違い
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
- **[構文早見表](docs/quick-reference/syntax-cheatsheet.md)** - 基本構文・よくある間違い

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
NYASH_DISABLE_PLUGINS=1     # Core経路安定化（CI常時）
NYASH_LOAD_NY_PLUGINS=1     # nyash.tomlのny_pluginsを読み込む

# 言語機能
--enable-using              # using/namespace有効化
NYASH_ENABLE_USING=1        # 環境変数版

# パーサー選択
--parser ny                 # Nyパーサーを使用
NYASH_USE_NY_PARSER=1       # 環境変数版
NYASH_USE_NY_COMPILER=1     # NyコンパイラMVP経路

# デバッグ
NYASH_CLI_VERBOSE=1         # 詳細診断
NYASH_DUMP_JSON_IR=1        # JSON IR出力
```

### 🤖 AI相談
```bash
# Gemini CLIで相談
gemini -p "Nyashの実装で困っています..."

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
NYASH_SKIP_TOML_ENV=1 ./tools/smoke_plugins.sh

# using/namespace E2E（要--enable-using）
./tools/using_e2e_smoke.sh
```

**ルート汚染防止**: `local_tests/`ディレクトリを使う！


### 🐛 デバッグ

#### パーサー無限ループ対策
```bash
# 🔥 デバッグ燃料でパーサー制御
./target/release/nyash --debug-fuel 1000 program.nyash      # 1000回制限
./target/release/nyash --debug-fuel unlimited program.nyash  # 無制限
./target/release/nyash program.nyash                        # デフォルト10万回
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

## 🚨 コンテキスト圧縮時: 作業停止→状況確認→CURRENT_TASK.md確認→ユーザー確認

---

Notes:
- ここから先の導線は README.md に集約
- 詳細情報は各docsファイルへのリンクから辿る
- このファイルは500行以内が目安（あくまで目安であり、必要に応じて増減可）
- Phase 15セルフホスティング実装中！詳細は[Phase 15](docs/development/roadmap/phases/phase-15/)へ
