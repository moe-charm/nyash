# Claude Quick Start (Minimal Entry)

このファイルは最小限の入口だよ。詳細はREADMEから辿ってねにゃ😺

## Start Here (必ずここから)
- 現在のタスク: [CURRENT_TASK.md](CURRENT_TASK.md)
  - 📁 **Main**: [docs/current_task/main/](docs/current_task/main/)
  - 📁 **LLVM**: [docs/current_task/llvm/](docs/current_task/llvm/)
  - 📁 **Self**: [docs/current_task/self_current_task/](docs/current_task/self_current_task/)
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

# Nyコンパイラ MVP経路 (実験的)
NYASH_USE_NY_COMPILER=1 ./target/release/nyash program.nyash
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

# パーサー無限ループ対策
./target/release/nyash --debug-fuel 1000 program.nyash

# プラグインなし実行
NYASH_DISABLE_PLUGINS=1 ./target/release/nyash program.nyash

# LLVMプラグイン実行（method_id使用）
./target/release/nyash --backend llvm program.nyash

# Python/llvmliteハーネス使用（開発中）
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash program.nyash
```

## 📝 Update (2025-09-13) 🎉 LLVM大進展！
- ✅ dep_tree_min_string.nyashのオブジェクト生成成功！（10.4KB）
- ✅ LLVM verifier green - dominance違反解決！
- 🐍 Python/llvmlite版を正式採用予定（開発速度10倍）
- 🎯 Phase 15セルフホスティング継続中（80k→20k行目標）
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

### 🐍 Python LLVM バックエンド (実験的実装)
**場所**: `/src/llvm_py/`

ChatGPTの調査待ち中に作成した、llvmliteベースのLLVMバックエンド実装にゃ。
Rust/inkwellの複雑さを回避して、シンプルに2000行程度でMIR14→LLVM変換を実現。

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
- このファイルは500行以内を維持する
- Phase 15セルフホスティング実装中！詳細は[Phase 15](docs/development/roadmap/phases/phase-15/)へ
