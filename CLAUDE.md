# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

---

## 🔄 **現在の開発状況** (2025-10-03)

### 🎯 **Phase 15.8: WASM実装進行中**
- **ブランチ**: `wasm-development` (← `selfhost`からfork)
- **目標**: MIR18命令 → WASM変換、ブラウザ/エッジ環境対応
- **戦略**: llvm_py拡張（既存800行活用）+ WASI runtime連携
- **計画書**: [Phase 15.8 README](docs/development/roadmap/phases/phase-15.8/README.md)

### 🎉 **Week 4完了！ExternCall Registry革命**
- ✅ ExternCallRegistry 2層分離アーキテクチャ実装完了
- ✅ CSE Fail-Fast根本修正（TimerBox CSEバグ解決）
- ✅ call命令完全動作（add_two(10) = 12n）
- ✅ ベンチマーク7/7テストPASS

### 📊 **WASM対応状況**（MIR18命令）
**✅ 実装済み（8/18命令）**:
1. const（定数）
2. binop（二項演算）
3. compare（比較演算）
4. branch（条件分岐）
5. jump（無条件ジャンプ）
6. ret（戻り値）
7. phi（値合流：if/loop両対応）
8. copy（値コピー）
9. **call**（関数呼び出し）← Phase 3.5完了！

**❌ 未実装（9/18命令）**:
- boxcall, newbox, load/store, externcall拡充, typeop, safepoint, barrier, loopform, unop

**🚨 発見された問題**:
- Hakoコンパイラ: 不正PHI命令生成バグ（到達不能ブロックがPHI predecessorに含まれる）

### 📚 **重要リソース**
- **開発マスタープラン**: [00_MASTER_ROADMAP.md](docs/development/roadmap/phases/00_MASTER_ROADMAP.md)
- **現在のタスク**: [CURRENT_TASK.md](CURRENT_TASK.md)
- **Phase 15.8詳細**: [docs/development/roadmap/phases/phase-15.8/](docs/development/roadmap/phases/phase-15.8/)

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

### 🚀 **Phase 15戦略: Rust VM + LLVM 2本柱**
```
【Rust VM】  開発・デバッグ・検証用（712行、高品質・型安全）
【LLVM】     本番・最適化・配布用（Python/llvmlite、実証済み）
【WASM】     Phase 15.8実験的（llvm_py拡張）
```

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

## 🔍 MIRデバッグ出力完全ガイド

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
- **MIR命令**: 18個の命令で全機能実現！

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
