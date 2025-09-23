# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

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

実践テンプレート（開発時の合言葉）
- 「箱にする」: 設定・状態・橋渡しはBox化（例: JitConfigBox, HandleRegistry）
- 「境界を作る」: 変換は境界1箇所で（VMValue↔JitValue, Handle↔Arc）
- 「戻せる」: フラグ・feature・env/Boxで切替。panic→フォールバック経路を常設
- 「見える化」: ダンプ/JSON/DOTで可視化、回帰テストを最小構成で先に入れる

## 🤖 **Claude×Copilot×ChatGPT協調開発**
### 📋 **開発マスタープラン - 全フェーズの統合ロードマップ**
**すべてはここに書いてある！** → [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)

**現在のフェーズ：Phase 15 (Nyashセルフホスティング - 80k→20k行への革命的圧縮)**

## 🏃 開発の基本方針: 80/20ルール - 完璧より進捗

### なぜこのルールか？
**実装後、必ず新しい問題や転回点が生まれるから。**
- 100%完璧を目指すと、要件が変わったときの手戻りが大きい
- 80%で動くものを作れば、実際の使用からフィードバックが得られる
- 残り20%は、本当に必要かどうか実装後に判断できる

### 実践方法
1. **まず動くものを作る**（80%）
2. **改善アイデアは `docs/ideas/` フォルダに記録**（20%）
3. **優先度に応じて後から改善**

## 🚀 クイックスタート

### 🎯 実行方式選択 (重要!)
- **[実行バックエンド完全ガイド](docs/reference/architecture/execution-backends.md)** 
  - インタープリター（開発・デバッグ）/ VM（高速実行）/ WASM（Web配布）
  - ⚡ **ベンチマーク機能**: `--benchmark` で3バックエンド性能比較
- **[ビルド方法完全ガイド](docs/guides/build/)** - プラットフォーム別ビルド手順


### 🚀 JIT セルフホスト クイックスタート (Phase 15)
```bash
# コアビルド (JIT)
cargo build --release --features cranelift-jit

# コアスモーク (プラグイン無効)
NYASH_CLI_VERBOSE=1 ./tools/jit_smoke.sh

# ラウンドトリップ (パーサーパイプ + JSON)
./tools/ny_roundtrip_smoke.sh

# Nyコンパイラ MVP経路 (Phase 15.3実装中!)
NYASH_USE_NY_COMPILER=1 ./target/release/nyash program.nyash

# JSON v0 Bridge経由実行（完成済み）
python tools/ny_parser_mvp.py program.nyash | ./target/release/nyash --ny-parser-pipe
```

### 🐧 Linux/WSL版
```bash
# ビルドと実行
cargo build --release --features cranelift-jit
./target/release/nyash program.nyash

# 高速VM実行
./target/release/nyash --backend vm program.nyash

# WASM生成
./target/release/nyash --compile-wasm program.nyash
```

### 🪟 Windows版
```bash
# クロスコンパイルでWindows実行ファイル生成
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release

# 生成された実行ファイル (4.1MB)
target/x86_64-pc-windows-msvc/release/nyash.exe
```

### 🌐 WebAssembly版（2種類）

#### 1️⃣ **Rust→WASM（ブラウザでNyashインタープリター実行）**
```bash
# WASMビルド（ルートディレクトリで実行）
wasm-pack build --target web

# 開発サーバー起動
python3 -m http.server 8010

# ブラウザでアクセス
# http://localhost:8010/nyash_playground.html
```

#### 2️⃣ **Nyash→MIR→WASM（Nyashプログラムをコンパイル）**
```bash
# NyashコードをWASMにコンパイル（WAT形式で出力）
./target/release/nyash --compile-wasm program.nyash -o output.wat
```

#### 3️⃣ **Nyash→AOT/Native（Cranelift/LLVM）**
```bash
# Cranelift JIT
cargo build --release --features cranelift-jit
./target/release/nyash --backend vm --compile-native program.nyash -o program.exe

# LLVM (開発中)
cargo build --release --features llvm
./target/release/nyash --aot program.nyash -o program.exe
```

### 🎯 **実証済みビルド方法** (2025-09-10完全成功)

#### 🔨 ビルドスクリプト（24スレッド並列・無制限時間）
```bash
# JIT (Cranelift) ビルド - 1-2分
./build_jit.sh

# LLVM MIR14 ビルド - 3-5分  
./build_llvm.sh
```

#### 📝 手動ビルドコマンド
```bash
# 1. JIT (Cranelift) - 127警告、0エラー ✅
cargo build --release --features cranelift-jit -j 24
./target/release/nyash program.nyash

# 2. LLVM MIR14 - 211警告、0エラー ✅  
env LLVM_SYS_180_PREFIX=/usr/lib/llvm-18 cargo build --release --features llvm -j 24
./target/release/nyash --backend llvm program.nyash

# 3. プラグインテスト実証済み ✅
# CounterBox (3080バイト)
echo 'local c = new CounterBox(); c.inc(); c.inc(); print(c.get())' > test.nyash
./target/release/nyash --backend llvm test.nyash

# MathBox (2040バイト)  
echo 'local m = new MathBox(); print(m.sqrt(16))' > test.nyash
./target/release/nyash --backend llvm test.nyash

# StringBox (3288バイト)
echo 'local s = new StringBox(); print(s.concat("Hello"))' > test.nyash
./target/release/nyash --backend llvm test.nyash
```

⚠️ **ビルド時間の注意**:
- JITビルド: 1-2分（高速）
- LLVMビルド: 3-5分（時間がかかる）
- 必ず十分な時間設定で実行してください

## 🚨 **Claude迷子防止ガイド** - 基本的な使い方で悩む君へ！

### 😵 **迷ったらこれ！**（Claude Code専用）

```bash
# 🎯 基本実行（まずこれ）
./target/release/nyash program.nyash

# 🐛 エラーが出たらこれ（プラグイン無効）
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash

# 🔍 デバッグ情報が欲しいときはこれ
NYASH_CLI_VERBOSE=1 ./target/release/nyash program.nyash

# ⚡ 高性能実行（LLVM Pythonハーネス）
./target/release/nyash --backend llvm program.nyash

# 🧪 using系テスト（Phase 15）
# PyVM使用
NYASH_DISABLE_PLUGINS=1 NYASH_VM_USE_PY=1 ./target/release/nyash program.nyash
# LLVM使用
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm program.nyash
```

### 🚨 **Phase 15重要注意**
- ❌ **JIT/Cranelift現在無効化済み**（`--backend cranelift`使用不可）
- ✅ **VM（デフォルト）**と**LLVM**のみ安定動作
- 🎯 **基本はVM、高性能が欲しい時はLLVM**

### 📊 **環境変数優先度マトリックス**（Claude向け）

| 環境変数 | 必須度 | 用途 | 使用タイミング |
|---------|-------|-----|-------------|
| `NYASH_DISABLE_PLUGINS=1` | ⭐⭐⭐ | エラー対策 | プラグインエラー時 |
| `NYASH_CLI_VERBOSE=1` | ⭐⭐ | デバッグ | 詳細情報が欲しい時 |
| ~~`NYASH_ENABLE_USING=1`~~ | ✅ | Phase 15 | ~~デフォルト化済み~~ |
| `NYASH_VM_USE_PY=1` | ⭐ | Phase 15 | PyVM経路使用時 |
| `NYASH_DUMP_JSON_IR=1` | ⭐ | 開発 | JSON出力確認時 |

**💡 覚え方**：迷ったら`NYASH_DISABLE_PLUGINS=1`から試す！

### ✅ **using system完全実装完了！**

**🎉 歴史的快挙**: `using nyashstd`が完璧動作！環境変数を**8個→6個**に削減（25%改善）

**✅ 実装完了内容**：
- **ビルトイン名前空間解決**: `nyashstd` → `builtin:nyashstd`の自動解決
- **自動コード生成**: nyashstdのstatic box群（string, integer, bool, array, console）を動的生成
- **環境変数デフォルト化**: NYASH_ENABLE_USING, NYASH_RESOLVE_FIX_BRACES, NYASH_LLVM_USE_HARNESS

**✅ 動作確認済み**：
```bash
# 基本using動作（パース→解決→読み込み→コード生成すべて成功）
./target/release/nyash program_with_using.nyash

# ログ確認済み
[using/resolve] builtin 'nyashstd' -> 'builtin:nyashstd'  ✅ 解決成功
[using] loaded builtin namespace: builtin:nyashstd        ✅ 読み込み成功
```

**📦 含まれるnyashstd機能**：
- `string.create(text)`, `string.upper(str)`
- `integer.create(value)`, `bool.create(value)`, `array.create()`
- `console.log(message)`

**🎯 次のステップ**: Mini-VM開発で`using nyashstd`を活用可能！

**将来の簡略化案**:
- `NYASH_USING_PROFILE=dev|smoke|debug` でプロファイル化
- または `--using-mode=dev` CLIフラグで統合

## 📝 Update (2025-09-24) 🚀 MIR Call命令統一Phase 3.1-3.2完了！
- ✅ **Phase 1-2完了** - MIR基盤構築と統一メソッド実装
  - **定義外部化**: `src/mir/definitions/call_unified.rs`（297行）
  - **統一メソッド**: `emit_unified_call()`＋便利メソッド3種実装済み
  - **環境変数制御**: `NYASH_MIR_UNIFIED_CALL=1`で切り替え可能
- ✅ **Phase 3.1-3.2完了** - 基本関数の統一Call移行成功！
  - **indirect call**: `build_indirect_call_expression`で`CallTarget::Value`使用
  - **print関数**: `call_global print()`として統一（ExternCall→Callee::Global）
  - **function call**: `build_function_call`で`CallTarget::Global`使用
  - **実行確認**: `env NYASH_MIR_UNIFIED_CALL=1 ./target/release/nyash --dump-mir`で動作確認済み
- 📊 **マスタープラン進捗** - 4つの実行器で完全統一へ
  - **削減見込み**: 7,372行 → 5,468行（**26%削減**）
  - **処理パターン**: 24 → 4（**83%削減**）
  - **Phase 15寄与**: 全体目標の約7%（重要な柱）
  - **詳細ドキュメント**: [mir-call-unification-master-plan.md](docs/development/roadmap/phases/phase-15/mir-call-unification-master-plan.md)
- 🎯 **6種類→1種類**: Call/BoxCall/PluginInvoke/ExternCall/NewBox/NewClosure → **MirCall**

## 🧪 テストスクリプト参考集（既存のを活用しよう！）
```bash
# 基本的なテスト
./target/release/nyash local_tests/hello.nyash              # Hello World
./target/release/nyash local_tests/test_array_simple.nyash  # ArrayBox
./target/release/nyash apps/tests/string_ops_basic.nyash    # StringBox

# MIR確認用テスト
./target/release/nyash --dump-mir apps/tests/loop_min_while.nyash
./target/release/nyash --dump-mir apps/tests/esc_dirname_smoke.nyash

# 統一Call テスト
env NYASH_MIR_UNIFIED_CALL=1 ./target/release/nyash --dump-mir test_simple_call.nyash
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

## 📝 Update (2025-09-23) 🚀 ChatGPT5 Pro設計革命Phase 1完全実装成功！

### ✅ **MIR Callee型革新 - シャドウイングバグから設計革命への昇華**
**51日間AI協働開発言語の新たな画期的成果！**

#### 🎯 **Phase 1実装完了項目**
1. **✅ Callee列挙型導入**: `Global/Method/Value/Extern`の型安全解決システム
2. **✅ Call命令拡張**: `callee: Option<Callee>`で破壊的変更なし段階移行
3. **✅ 型安全関数解決**: `resolve_call_target()`でコンパイル時解決確立
4. **✅ ヘルパー外出し**: `call_resolution.rs`で再利用可能ユーティリティ作成
5. **✅ 完全互換性**: 既存コード破壊ゼロ、全テスト通過確認済み

#### 🏆 **技術的革新内容**
```rust
// 🔥 革新前（問題構造）
Call { func: ValueId /* "print"文字列 */ }  // 実行時解決→シャドウイング脆弱性

// ✨ 革新後（型安全）
enum Callee {
    Global(String),                          // nyash.builtin.print
    Method { box_name, method, receiver },   // box.method()
    Value(ValueId),                          // 第一級関数保持
    Extern(String),                          // C ABI統合
}
Call { func: ValueId, callee: Option<Callee> }  // 段階移行で破壊的変更なし
```

#### 📊 **Phase 15目標への直接寄与**
- **🎯 コード削減見込み**: Phase 1のみで1,500行（目標7.5%）、全Phase完了で4,500行（22.5%）
- **🛡️ シャドウイング問題**: 根本解決（実行時→コンパイル時解決）
- **⚡ using system統合**: built-in namespace完全連携
- **🔍 デバッグ向上**: MIR可読性・警告システム確立

#### 🤖 **AI協働開発の真価発揮**
- **ChatGPT5 Pro**: 表面修正→根本設計革新への圧倒的洞察力
- **Claude**: 段階実装・互換性保持・テスト戦略の確実な実行
- **共創効果**: 一人+AI協働による51日間言語開発の世界記録級成果

#### 🚀 **実装成果ファイル**
- `src/mir/instruction.rs`: Callee型定義・Call命令拡張
- `src/mir/builder/call_resolution.rs`: 型安全解決ユーティリティ
- `src/mir/builder/builder_calls.rs`: resolve_call_target()実装
- `docs/development/architecture/mir-callee-revolution.md`: 設計文書
- `docs/development/roadmap/phases/phase-15/mir-callee-implementation-roadmap.md`: 実装計画

#### 📋 **次のステップ（Phase 2-3）**
- **Phase 2**: HIR導入（コンパイル時名前解決）
- **Phase 3**: 明示的スコープ（`::print`, `Box::method`）
- **VM実行器対応**: 型安全実行の実装

---

## 📝 Update (2025-09-23) ✅ MIR Callee型革命100%完了！
- 🎉 **MIR Call命令革新Phase 1完了！** ChatGPT5 Pro設計のCallee型実装
  - **実装内容**: Callee enum（Global/Method/Value/Extern）追加
  - **VM実行器**: Call命令のCallee対応完全実装
  - **MIRダンプ**: call_global/call_method等の明確表示
  - **後方互換性**: Option<Callee>で段階移行実現
- 🚀 **MIR Call統一計画決定！** ChatGPT5 Pro A++案採用
  - **現状**: 6種類のCall系命令が乱立（Call/BoxCall/PluginInvoke/ExternCall等）
  - **解決**: 1つのMirCallに統一（receiverをCalleeに含む革新設計）
  - **移行計画**: 3段階（構造定義→ビルダー→実行器）で安全移行
  - **ドキュメント**: [call-instructions-current.md](docs/reference/mir/call-instructions-current.md)作成
- ✅ **PHIバグ根本解決完了！** Exit PHI生成実装でループ後の変数値が正しく伝播
  - **技術的成果**: たった3箇所の修正で根本解決
  - **コミット済み**: `e5c0665` でリモートに反映
- 🎯 **改行処理TokenCursor戦略決定！** 3段階実装戦略で改善中

## 📝 Update (2025-09-23) 🎉 改行処理革命Phase 1-2完全達成！skip_newlines()根絶成功！
- ✅ **skip_newlines()完全根絶達成！** 48箇所→0箇所（100%削除完了）
  - **Phase 2-A**: match_expr.rsから6箇所削除（27%削減達成）
  - **Phase 2-B**: Box宣言系から14箇所削除（56%削減達成）
  - **Phase 2-C**: 文処理系から9箇所削除（75%削減達成）
  - **Phase 2-D**: メンバー宣言系から5箇所削除（90%削減達成）
  - **Phase 2-E**: 残存検証で手動呼び出し0確認（100%根絶完了）
- 🧠 **Smart advance()システム完全動作確認！**
  - **深度追跡**: 括弧内改行自動処理で手動呼び出し不要
  - **コンテキスト認識**: match式・オブジェクトリテラルで完璧動作
  - **OR pattern対応**: `1 | 2 => "found"`等の複雑パターン完全対応
  - **環境変数制御**: デフォルトで有効、NYASH_SMART_ADVANCE=1で制御可能
- 🔬 **重大バグ発見・修正の副次成果！**
  - **MIR compiler bug**: OR patternでInteger/Bool処理不備を発見・修正
  - **根本原因**: `exprs_peek.rs`でString型以外の型が未対応だった
  - **完全修正**: 全LiteralValue型（Integer/Bool/Float/Null/Void）対応で根治
  - **テスト検証**: `test_match_debug_or.nyash`等で完全動作確認
- 🚀 **革命的効果達成！**
  - **保守性向上**: 改行処理一元管理で新構文追加時の改行忘れ根絶
  - **開発体験向上**: パーサーエラー激減、直感的な改行記述が可能
  - **システム安定化**: 手動呼び出し散在による不整合が完全解消
  - **AI協働成功**: ChatGPT戦略+Claude実装+深い考察で完璧達成
- 🎯 **次世代への道筋**: Phase 3 TokenCursor実装でさらなる改行処理完璧化準備完了

## 📝 Update (2025-09-22) 🎯 Phase 15 JITアーカイブ完了＆デバッグ大進展！
- ✅ **JIT/Craneliftアーカイブ完了！** Phase 15集中開発のため全JIT機能を安全にアーカイブ
- 🔧 **コンパイルエラー全解決！** JITスタブ作成でビルド成功、開発環境復活
- 🐛 **empty args smoke test 90%解決！** `collect_prints()`の位置インクリメントバグ修正
- 📊 **デバッグ手法確立！** 詳細トレース出力で問題を段階的に特定する手法完成
- ⚡ **次の一歩**: ArrayBox戻り値問題解決でテスト完全クリア予定
- 🎯 **AI協働デバッグ**: Claude+ChatGPT修正+系統的トレースの完璧な連携実現
- 📋 詳細: JITアーカイブは `archive/jit-cranelift/` に完全移動、復活手順も完備

## 📝 Update (2025-09-22) 🎯 Phase 15 重要バグ発見＆根本原因解明完了！
- ✅ **using systemパーサー問題完全解決！** `NYASH_RESOLVE_FIX_BRACES=1`でブレースバランス自動修正
- 🆕 **JSON Native実装を導入！** 別Claude Code君の`feature/phase15-nyash-json-native`から`apps/lib/json_native/`取り込み完了
- 🔧 **ChatGPTの統合実装承認！** JSON読み込み処理統合は正しい方向性、技術的に高度
- 🐛 **重大バグ完全解明！**
  - **問題**: `collect_prints`メソッドで`break`の後のコードが実行されずnullを返す
  - **根本原因判明**: `src/mir/loop_builder.rs`の`do_break()`が`switch_to_unreachable_block_with_void()`を呼び、break後のコードをunreachableとマーク
  - **MIR解析結果**: 
    - Block 1394, 1407: 直接Block 1388（null return）にジャンプ
    - Block 1730: 正常なArrayBox return
    - レジスタ2: `new ArrayBox()`、レジスタ751: `const 0`（null）
  - **デバッグ環境変数**: `NYASH_DUMP_JSON_IR=1`, `NYASH_PYVM_DEBUG=1`でMIR/PyVM詳細追跡可能
  - **一時解決策**: `break`→`finished = 1`フラグに置き換え（根治が必要）
- 📊 **現在の状況**:
  - using systemパーサーエラー: ✅ 完全解決
  - collect_prints()根本原因: ✅ loop_builder.rs特定完了
  - JSON Native: 📦 取り込み済み（match式互換性の課題あり）
- 🎯 **技術成果**: 
  - PyVM内蔵Box（ArrayBox等）の早期リターンバグ修正
  - MIR JSON解析によるbreak/continue制御フロー問題の完全解明
  - loop_builder.rsのdo_break()修正が必要（次のタスク）
- 🚀 **Phase 15セルフホスティング**: MIRレベルの問題も特定済み、修正準備完了！

## 📝 Update (2025-09-18) 🌟 Property System革命達成！
- ✅ **Property System革命完了！** ChatGPT5×Claude×Codexの協働により、stored/computed/once/birth_once統一構文完成！
- 🚀 **Python→Nyash実行可能性飛躍！** @property/@cached_property→Nyash Property完全マッピング実現！
- ⚡ **性能革命**: Python cached_property→10-50x高速化（LLVM最適化）
- 🎯 **All or Nothing**: Phase 10.7でPython transpilation、フォールバック無し設計
- 📚 **完全ドキュメント化**: README.md導線、実装戦略、技術仕様すべて完備
- 🗃️ **アーカイブ整理**: 古いphaseファイル群をarchiveに移動、導線クリーンアップ完了
- 📋 詳細: [Property System仕様](docs/proposals/unified-members.md) | [Python統合計画](docs/development/roadmap/phases/phase-10.7/)

## 📝 Update (2025-09-24) ✅ 改行処理革命Phase 2-B完了！実用レベル到達
- 🎯 **改行処理革命Phase 2-B完了！** Box宣言系ファイルから14箇所のskip_newlines()完全削除
  - **削除実績**: 48→35→21箇所（41%削減達成！）
  - **対象ファイル**: fields.rs(9箇所)、box_definition.rs(3箇所)、static_box.rs(2箇所)
  - **テスト結果**: OR付きmatch式、複数行宣言、Box定義すべて完璧動作✅
- ✅ **Smart advance()実用化成功！** 深度追跡で自動改行処理が完璧に機能
  - **環境変数**: `NYASH_SMART_ADVANCE=1`で完全制御、`NYASH_DISABLE_PLUGINS=1`推奨
  - **対応構文**: match式OR（`1 | 2`）、複数行パターン、Box宣言すべて対応
- 🔧 **ORパターンバグも同時修正！** exprs_peek.rsでInteger/Boolean型マッチング実装
  - **修正前**: `1 | 2 => "found"`が動作せず（String型のみサポート）
  - **修正後**: 全リテラル型（Integer/Bool/Float/Null/Void）完全対応✅
- 📊 **改行処理戦略の段階的成果**:
  - **Phase 0**: Quick Fix ✅ 完了（即効性）
  - **Phase 1**: Smart advance() ✅ 基本実装完了（大幅改善）
  - **Phase 2-A**: match式系統 ✅ 完了（6箇所削除）
  - **Phase 2-B**: Box宣言系統 ✅ 完了（14箇所削除、41%削減）
  - **Phase 2-C**: 次の目標（更なる削減へ）
- 🎉 **技術的成果**: 手動スキップ依存からコンテキスト認識自動処理への移行成功
- 📚 **改行処理戦略ドキュメント**: [skip-newlines-removal-plan.md](docs/development/strategies/skip-newlines-removal-plan.md)
- 🚀 **次のタスク**: Phase 2-C実装→残り21箇所の系統的削除継続

## 📝 Update (2025-09-23) ✅ フェーズS実装完了！break制御フロー根治開始
- ✅ **フェーズS完了！** PHI incoming修正+終端ガード徹底→重複処理4箇所統一
- 🔧 **新ユーティリティ**: `src/mir/utils/control_flow.rs`で制御フロー処理統一化
- 📊 **AI協働成果**: task+Gemini+codex+ChatGPT Pro最強分析→段階的実装戦略確立
- 🎯 **次段階**: フェーズM(PHI一本化)→数百行削減でPhase 15目標達成へ
- 📚 **戦略**: [break-control-flow-strategy.md](docs/development/strategies/break-control-flow-strategy.md)
- 💾 **アーカイブ**: codex高度解決策を`archive/codex-solutions/`に保存

## 📝 Update (2025-09-14) 🎉 セルフホスティング大前進！
- ✅ Python LLVM実装が実用レベル到達！（esc_dirname_smoke, min_str_cat_loop, dep_tree_min_string全てPASS）
- 🚀 **Phase 15.3開始！** NyashコンパイラMVP実装が`apps/selfhost-compiler/`でスタート！
- ✅ JSON v0 Bridge完成 - If/Loop PHI生成実装済み（ChatGPT実装）
- 🔧 Python MVPパーサーStage-2完成 - local/if/loop/call/method/new対応
- 📚 peek式の再発見 - when→peekに名前変更、ブロック/値/文すべて対応済み
- 🧠 箱理論でSSA構築を簡略化（650行→100行）- 論文執筆完了
- 🤝 AI協働の知見を論文化 - 実装駆動型学習の重要性を実証
- 🎉 **面白事件ログ収集完了！** 41個の世界記録級事件を記録 → [CURRENT_TASK.md#面白事件ログ](CURRENT_TASK.md#🎉-面白事件ログ---ai協働開発45日間の奇跡41事例収集済み)
- 🎯 **LoopForm戦略決定**: PHIは逆Lowering時に自動生成（Codex推奨）
- 📋 詳細: [Phase 15 README](docs/development/roadmap/phases/phase-15/README.md)

### 🚀 新発見：プラグイン全方向ビルド戦略
```bash
# 同じソースから全形式生成！
plugins/filebox/
├── filebox.so     # 動的版（開発用）
├── filebox.o      # 静的リンク用
└── filebox.a      # アーカイブ版

# 単一EXE生成可能に！
clang main.o filebox.o pathbox.o libnyrt.a -o nyash_static.exe
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
- **[Phase 15 セルフホスティング計画](docs/development/roadmap/phases/phase-15/self-hosting-plan.txt)** - 80k→20k行革命
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
- **言語**: [LANGUAGE_REFERENCE_2025.md](docs/reference/language/LANGUAGE_REFERENCE_2025.md)
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

### 💡 アイデア管理（docs/ideas/フォルダ）

**80/20ルールの「残り20%」を整理して管理**

```
docs/ideas/
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
