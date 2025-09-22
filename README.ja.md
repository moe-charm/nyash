# 🐱 Nyash プログラミング言語
**超真面目に作っている趣味言語**  
**20日でゼロからネイティブバイナリへ - AI駆動の言語革命**

*[🇺🇸 English Version / 英語版はこちら](README.md)*

[![Selfhost Minimal](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml/badge.svg?branch=selfhosting-dev)](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml)
[![Core Smoke](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml/badge.svg)](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![Performance](https://img.shields.io/badge/Performance-13.5x%20高速化-ff6b6b.svg)](#performance)
[![JIT Ready](https://img.shields.io/badge/JIT-Cranelift%20搭載%20(実行封印)-orange.svg)](#execution-modes)
[![ブラウザで試す](https://img.shields.io/badge/今すぐ試す-ブラウザプレイグラウンド-ff6b6b.svg)](projects/nyash-wasm/nyash_playground.html)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

開発者向けクイックスタート: `docs/DEV_QUICKSTART.md`
ユーザーマクロ（Phase 2）: `docs/guides/user-macros.md`
AST JSON v0（マクロ/ブリッジ）: `docs/reference/ir/ast-json-v0.md`
セルフホスト1枚ガイド: `docs/how-to/self-hosting.md`
ExternCall（env.*）と println 正規化: `docs/reference/runtime/externcall.md`

仕様と既知制約
- 必須不変条件（Invariants）: `docs/reference/invariants.md`
- 制約（既知/一時/解消済み）: `docs/reference/constraints.md`
- PHI と SSA の設計: `docs/architecture/phi-and-ssa.md`
  - 既定のPHI挙動: ビルドが `phi-legacy` を有効化している場合は PHI-ON（推奨）。未有効時は安定性のため PHI-OFF（エッジコピー）にフォールバック。
  - 実行時切替: `NYASH_MIR_NO_PHI=0`（PHI-ON）、`NYASH_MIR_NO_PHI=1`（PHI-OFF）。
- テスト行列（仕様→テスト対応）: `docs/guides/testing-matrix.md`
- 他言語との比較: `docs/comparison/nyash-vs-others.md`

プロファイル（クイック）
- `--profile dev` → マクロON（strict）、PyVM 開発向けの既定を適用（必要に応じて環境で上書き可）
- `--profile lite` → マクロOFF の軽量実行
  - 例: `./target/release/nyash --profile dev --backend vm apps/tests/ternary_basic.nyash`

## 目次
- [Self-Hosting（自己ホスト開発）](#self-hosting)
- [今すぐ試す（ブラウザ）](#-今すぐブラウザでnyashを試そう)

<a id="self-hosting"></a>
## 🧪 Self-Hosting（自己ホスト開発）
- ガイド: `docs/how-to/self-hosting.md`
- 最小E2E: `NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`
- スモーク: `bash tools/jit_smoke.sh` / `bash tools/selfhost_vm_smoke.sh`
- Makefile: `make run-minimal`, `make smoke-selfhost`

MIR注記: Core‑13 最小カーネルは既定で有効（NYASH_MIR_CORE13=1）。旧命令は正規化されます（Array/Ref→BoxCall、TypeCheck/Cast/Barrier/WeakRefの統一）。

純化モード: `NYASH_MIR_CORE13_PURE=1` を有効にすると、Optimizer が Load/Store/NewBox/Unary を Core‑13 形に書き換え、残存する非Core‑13命令があればコンパイルを失敗させます。あえて実行が壊れる可能性がありますが、MIR違反を早期に発見するための設計です。

変更履歴（要点）: `CHANGELOG.md`

## 🎮 **今すぐブラウザでNyashを試そう！**

👉 **[ブラウザプレイグラウンドを起動](projects/nyash-wasm/nyash_playground.html)** 👈

インストール不要 - ウェブブラウザで即座にNyashを体験！

---

## 🚀 **速報: ネイティブEXE達成！**

**2025年8月29日** - 誕生からわずか20日で、Nyashがネイティブ実行ファイルへのコンパイルを実現！

```bash
# Nyashソースからネイティブバイナリへ（Craneliftが必要）
cargo build --release --features cranelift-jit
./tools/build_aot.sh program.nyash -o app         # ネイティブEXE
./app                                             # スタンドアロン実行！
```

**20日間で達成したこと：**
- ✅ インタープリター付き完全プログラミング言語
- ✅ 13.5倍高速化を実現したVM
- ✅ JITコンパイラ（Cranelift統合）
- ✅ WebAssemblyサポート
- ✅ プラグインシステム（C ABI）
- ✅ ネイティブバイナリ生成
- ✅ プラグイン経由のPython統合

---

## ✨ **なぜNyashなのか？**

### 🎯 **Everything is Box 哲学**
```nyash
// 従来の言語は複雑な型システムを持つ
// Nyash: 一つの概念がすべてを支配する - Box

static box Main {
    main() {
        // すべての値はBox - 統一、安全、シンプル
        local name = new StringBox("Nyash")
        local count = new IntegerBox(42)
        local data = new MapBox()
        
        // PythonオブジェクトもBox！
        local py = new PyRuntimeBox()
        local math = py.import("math")
        print("sqrt(9) = " + math.getattr("sqrt").call(9).str())
        
        return 0
    }
}
```

### ⚡ **前例のない開発速度**
- **1日目**: 基本インタープリター動作
- **4日目**: すでにJIT計画開始
- **13日目**: VMが13.5倍高速化達成
- **20日目**: ネイティブ実行ファイル生成！

### 🔌 **プラグインファースト・アーキテクチャ**
```nyash
// あらゆる機能がプラグインBoxになれる
local file = new FileBox()          // ファイルI/Oプラグイン
local http = new HttpClientBox()    // ネットワークプラグイン
local py = new PyRuntimeBox()       // Pythonプラグイン

// プラグインもネイティブコードにコンパイル！
```

---

## 🏗️ **複数の実行モード**

重要: 現在、JIT ランタイム実行は封印中です。実行は「PyVM（既定）／VM（任意でレガシー有効）」、配布は「Cranelift AOT(EXE)／LLVM AOT(EXE)」の4体制です。

Phase‑15（自己ホスト期）: VM/インタープリタはフィーチャーで切替
- 既定ビルド: `--backend vm` は PyVM 実行（python3 + `tools/pyvm_runner.py` が必要）
- レガシー Rust VM/インタープリターを有効化するには:
  ```bash
  cargo build --release --features vm-legacy,interpreter-legacy
  ```
  以降、`--backend vm`/`--backend interpreter` が従来経路で動作します。

### 1. **インタープリターモード** （開発用）
```bash
./target/release/nyash program.nyash
```
- 即座に実行
- 完全なデバッグ情報
- 開発に最適

### 2. **VMモード（既定は PyVM／レガシーは任意）**
```bash
# 既定: PyVM ハーネス（python3 必要）
./target/release/nyash --backend vm program.nyash

# レガシー Rust VM を使う場合
cargo build --release --features vm-legacy
./target/release/nyash --backend vm program.nyash
```
- 既定（vm-legacy OFF）: MIR(JSON) を出力して `tools/pyvm_runner.py` で実行
- レガシー VM: インタープリター比で 13.5x（歴史的実測）。比較・検証用途で維持
 - 補足: `--benchmark` はレガシー VM（`vm-legacy`）が必要です。実行前に `cargo build --release --features vm-legacy` を行ってください。

### 3. **ネイティブバイナリ（Cranelift AOT）** （配布用）
```bash
# 事前ビルド（Cranelift）
cargo build --release --features cranelift-jit

./tools/build_aot.sh program.nyash -o myapp
./myapp  # スタンドアロン実行！
```
- 依存関係ゼロ
- 最高性能
- 簡単配布

### 4. **ネイティブバイナリ（LLVM AOT）**
```bash
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) \
  cargo build --release --features llvm
NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o \
  ./target/release/nyash --backend llvm program.nyash
# リンクして実行
cc nyash_llvm_temp.o -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o myapp
./myapp
```

簡易スモークテスト（VM と EXE の出力一致確認）:
```bash
tools/smoke_aot_vs_vm.sh examples/aot_min_string_len.nyash
```

### LLVM バックエンドの補足
- `NYASH_LLVM_OBJ_OUT`: `--backend llvm` 実行時に `.o` を出力するパス。
  - 例: `NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o ./target/release/nyash --backend llvm apps/ny-llvm-smoke/main.nyash`
- 削除された `NYASH_LLVM_ALLOW_BY_NAME=1`: すべてのプラグイン呼び出しがmethod_idベースに統一。
  - LLVMバックエンドは性能と型安全性のため、method_idベースのプラグイン呼び出しのみ対応。


### 5. **WebAssembly** （ブラウザ用）
```bash
cargo build --release --features wasm-backend
./target/release/nyash --compile-wasm program.nyash
```
- ブラウザで実行
- デフォルトでクロスプラットフォーム
- Webファースト開発

---

## 🧰 タスク実行 (nyash.toml)

`nyash.toml` の `[tasks]` と `[env]` で、ビルド/スモークなどのタスクを簡単に実行できます（MVP）。

例（nyash.toml の末尾に追記）:

```
[env]
RUST_BACKTRACE = "1"

[tasks]
build_llvm = "LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm"
smoke_obj_array = "NYASH_LLVM_OBJ_OUT={root}/nyash_llvm_temp.o ./target/release/nyash --backend llvm apps/ny-llvm-smoke/main.nyash"
```

実行:

```
./target/release/nyash --run-task build_llvm
./target/release/nyash --run-task smoke_obj_array
```

補足:
- `[env]` の値は実行前に環境へ適用されます。
- `{root}` は現在のプロジェクトルートに展開されます。
- 現状は最小機能（OS別/依存/並列は未対応）。

---

## 🧰 一発ビルド（MVP）: `nyash --build`

`nyash.toml` を読み、プラグイン → コア → AOT → リンクまでを一発実行する最小ビルド機能です。

基本（Cranelift AOT）
```bash
./target/release/nyash --build nyash.toml \
  --app apps/egui-hello-plugin/main.nyash \
  --out app_egui
```

主なオプション（最小）
- `--build <path>`: nyash.toml の場所
- `--app <file>`: エントリ `.nyash`
- `--out <name>`: 出力EXE名（既定: `app`/`app.exe`）
- `--build-aot cranelift|llvm`（既定: cranelift）
- `--profile release|debug`（既定: release）
- `--target <triple>`（必要時のみ）

注意
- LLVM AOT には LLVM 18 が必要（`LLVM_SYS_180_PREFIX` を設定）。
- GUIを含む場合、AOTのオブジェクト出力時にウィンドウが一度開きます（閉じて続行）。
- WSL で表示されない場合は `docs/guides/cranelift_aot_egui_hello.md` のWSL Tips（Wayland→X11切替）を参照。


## 📊 **パフォーマンスベンチマーク**

実世界ベンチマーク結果 (ny_bench.nyash)：

```
モード           | 時間      | 相対速度
----------------|-----------|---------------
インタープリター | 110.10ms  | 1.0x (基準)
VM              | 8.14ms    | 13.5倍高速
Cranelift AOT   | ~4–6ms    | ~20–27倍高速
ネイティブ(LLVM)| ~4ms      | ~27倍高速
```

---

## 🎮 **言語機能**

### クリーンな構文
```nyash
box GameCharacter {
    private { name, health, skills }
    
    // birthコンストラクタ - Boxに生命を与える！
    birth(characterName) {
        me.name = characterName
        me.health = 100
        me.skills = new ArrayBox()
        print("🌟 " + characterName + " が誕生しました！")
    }
    
    learnSkill(skill) {
        me.skills.push(skill)
        return me  // メソッドチェーン
    }
}

// 使用例
local hero = new GameCharacter("ネコ")
hero.learnSkill("火魔法").learnSkill("回復")
```

### モダンなAsync/Await
```nyash
// シンプルな並行処理
nowait task1 = fetchDataFromAPI()
nowait task2 = processLocalFiles()

// 待機中に他の作業
updateUI()

// 結果収集
local apiData = await task1
local files = await task2
```

### デリゲーションパターン
```nyash
// 継承よりコンポジション
box EnhancedArray from ArrayBox {
    private { logger }
    
    override push(item) {
        me.logger.log("追加中: " + item)
        from ArrayBox.push(item)  // 親に委譲
    }
}
```

---

## 🔌 **プラグインシステム**

Nyashは「Everything is Plugin」アーキテクチャを開拓：

```toml
# nyash.toml - プラグイン設定
[libraries."libnyash_python_plugin.so"]
boxes = ["PyRuntimeBox", "PyObjectBox"]

[libraries."libnyash_net_plugin.so"]  
boxes = ["HttpServerBox", "HttpClientBox", "WebSocketBox"]
```

C/Rustで独自のBox型を作成してシームレスに統合！

---

## 🛠️ **はじめる**

### クイックインストール (Linux/Mac/WSL)
```bash
# クローンとビルド
git clone https://github.com/moe-charm/nyash.git
cd nyash
cargo build --release --features cranelift-jit

# 最初のプログラムを実行
echo 'print("Hello Nyash!")' > hello.nyash
./target/release/nyash hello.nyash
```

### Windows
```bash
# Windows向けクロスコンパイル
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc --release
# target/x86_64-pc-windows-msvc/release/nyash.exe を使用

# WindowsでのネイティブEXE（AOT）ビルド（Cranelift と MSYS2/WSL が必要）
cargo build --release --features cranelift-jit
powershell -ExecutionPolicy Bypass -File tools\build_aot.ps1 -Input examples\aot_min_string_len.nyash -Out app.exe
./app.exe
```

---

## 🌟 **独自のイノベーション**

### 1. **AI駆動開発**
- Claude、ChatGPT、Codexの協力で開発
- コンセプトからネイティブコンパイルまで20日間の旅
- AIが言語開発を30倍加速できることを証明

### 2. **Box-Firstアーキテクチャ**
- すべての最適化がBox抽象を保持
- プラグインもBox、JITもBoxを保持、ネイティブコードもBoxを尊重
- すべての実行モードで前例のない一貫性

### 3. **観測可能な設計**
- 組み込みのデバッグとプロファイリング
- JITコンパイルのJSONイベントストリーム
- 最適化のDOTグラフ可視化

---

## 📚 **例**

### Python統合
```nyash
// NyashからPythonライブラリを使用！
local py = new PyRuntimeBox()
local np = py.import("numpy")
local array = np.getattr("array").call([1, 2, 3])
print("NumPy配列: " + array.str())
```

### Webサーバー
```nyash
local server = new HttpServerBox()
server.start(8080)

loop(true) {
    local request = server.accept()
    local response = new HttpResponseBox()
    response.setStatus(200)
    response.write("Nyashからこんにちは！")
    request.respond(response)
}
```

### ゲーム開発
```nyash
box GameObject {
    public { x, y, sprite }
    
    update(deltaTime) {
        // 物理シミュレーション
        me.y = me.y + gravity * deltaTime
    }
    
    render(canvas) {
        canvas.drawImage(me.sprite, me.x, me.y)
    }
}
```

---

## 🤝 **貢献**

革命に参加しよう！以下を歓迎します：
- 🐛 バグ報告と修正
- ✨ プラグイン経由の新しいBox型
- 📚 ドキュメントの改善
- 🎮 クールなサンプルプログラム

詳細は `AGENTS.md`（Repository Guidelines）をご参照ください。プロジェクト構成、ビルド/テスト手順、PRの要件を簡潔にまとめています。

## 📄 **ライセンス**

MIT ライセンス - プロジェクトで自由に使用してください！

---

## 👨‍💻 **作者**

**charmpic** - 趣味で言語作ってる人
- 🐱 GitHub: [@moe-charm](https://github.com/moe-charm)
- 🌟 協力: Claude、ChatGPT、Codexとのコラボレーション

---

## 🎉 **歴史的タイムライン**

- **2025年8月9日**: 最初のコミット - "Hello Nyash!"
- **2025年8月13日**: JIT計画開始（4日目！）
- **2025年8月20日**: VMが13.5倍性能達成
- **2025年8月29日**: ネイティブEXEコンパイル実現！

*ゼロからネイティブバイナリまで20日間 - 言語開発の新記録！*

---

**🚀 Nyash - すべてがBoxであり、Boxがネイティブコードにコンパイルされる場所！**

*❤️、🤖 AIコラボレーション、そしてプログラミング言語は思考の速度で作れるという信念で構築*
