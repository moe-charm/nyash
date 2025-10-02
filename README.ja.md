# 🐱 HakoRune プログラミング言語（旧称 Nyash）
**超真面目に作っている趣味言語**  
**20日でゼロからネイティブバイナリへ - AI駆動の言語革命**

*[🇺🇸 English Version / 英語版はこちら](README.md)*

[![Selfhost Minimal](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml/badge.svg?branch=selfhosting-dev)](https://github.com/moe-charm/nyash/actions/workflows/selfhost-minimal.yml)
[![Core Smoke](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml/badge.svg)](https://github.com/moe-charm/nyash/actions/workflows/smoke.yml)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![Performance](https://img.shields.io/badge/Performance-13.5x%20高速化-ff6b6b.svg)](#performance)
[![JIT Ready](https://img.shields.io/badge/JIT-Cranelift%20搭載%20(実行封印)-orange.svg)](#execution-modes)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

実行状況・ポリシー（機能追加ポーズ）
- このフェーズでは「大きな機能追加」は一時停止しますが、バグ修正・正しさ/Fail‑Fastの向上は積極的に行います。
- 公開仕様の意味論は変えません。必要な追加は既定OFFのフラグで段階導入します（可逆・局所）。

開発者向けクイックスタート: `docs/guides/build/dev-quickstart.md`
ユーザーマクロ（Phase 2）: `docs/guides/user-macros.md`
AST JSON v0（マクロ/ブリッジ）: `docs/reference/ir/ast-json-v0.md`
セルフホスト1枚ガイド: `docs/guides/how-to/self-hosting.md`
ExternCall（env.*）と println 正規化: `docs/reference/runtime/externcall.md`
VM エンジン切替: `NYASH_VM_ENGINE={fallback|full}`（既定は fallback）。設計は `docs/guides/runtime-architecture.md` を参照してください。
Using/プラグイン（ENV要約）:
- Using: `NYASH_USING=0|1`（既定=1）、`NYASH_USING_STRATEGY={resolver|prelude}`（別名: `NYASH_USING_IMPL`）
- Plugins: `NYASH_PLUGIN_POLICY={auto|off|force}`（既定=auto）

### MIR 統一Call（既定ON）
- 呼び出しは中央（`emit_unified_call`）で集約。開発段階では既定ON（`0|false|off` で明示OFF）。
  - 早期正規化: `toString/stringify → str`
  - `equals/1`: Known 優先 → 一意候補（ユーザーBoxのみ）
  - Known→関数化: `obj.m(a) → Class.m(me,obj,a)` に統一
- レガシー関数化の重複を避ける開発ガード:
  - `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`
- JSON出力は unified ON で v1、OFF で v0（従来）
 - VMポリシー（Instance BoxCall）: ユーザーBoxのインスタンス呼び出しは開発時のみフォールバック許可。
   - 環境変数: `NYASH_VM_USER_INSTANCE_BOXCALL={0|1}`（既定: dev/ci=true, prod=false）
   - 規範: ビルダーが Instance→Function 関数化を行い、実行時の Instance BoxCall に依存しない設計（prod は禁止で漏れを早期検知）。

開発計測（任意）
- `resolve.choose` の Known 率をKPIとして出力
  - `NYASH_DEBUG_KPI_KNOWN=1`（有効化）
  - `NYASH_DEBUG_SAMPLE_EVERY=<N>`（N件ごとに出力）

レイヤー・ガード（origin→observe→rewrite の一方向）
- スクリプト: `tools/dev/check_builder_layers.sh`

開発ショートカット（Operator Boxes + JSON）
- JSON最小（Roundtrip/Nested を一発）: `./tools/opbox-json.sh`
- quick 全体（軽量プリフライト＋timeout 180s）: `./tools/opbox-quick.sh`
- 詳細: `docs/guides/operator-boxes.md`

開発モードと既定
- `nyash --dev script.nyash` で開発向け既定（AST using ON / Operator Boxes 観測ON / 診断の最小ON）を一括で有効化できます。`nyash script.nyash` は本番相当（静かで安定）。
- ワンコマンドの dev ショートカットも引き続き利用できます（`tools/opbox-json.sh` / `tools/opbox-quick.sh`）。
- using ガード: 同じファイルの重複 import（または alias の再バインド）はエラーになり、行番号付きで通知されます。
  - 例: `using: duplicate import of '<canon_path>' at file.nyash:12 (previous alias 'X' first seen at line 5)`
  - 重複を削除／統合して解消してください。

Phase‑15（2025‑09）アップデート
- LLVM は Python/llvmlite ハーネスを優先（`NYASH_LLVM_USE_HARNESS=1`）。Rust VM/JIT は保守・比較用途。
- パーサの改行処理は TokenCursor に統一中（`NYASH_PARSER_TOKEN_CURSOR=1`）。
- if/else の PHI は実際の遷移元（exit）を pred として使用（VM/LLVM パリティ緑）。
- 自己ホスト準備として Ny 製 JSON ライブラリと Ny Executor（最小命令）を既定OFFトグルで段階導入予定。
- 推奨トグル: `NYASH_LLVM_USE_HARNESS=1`, `NYASH_PARSER_TOKEN_CURSOR=1`, `NYASH_JSON_PROVIDER=ny`, `NYASH_SELFHOST_EXEC=1`。

仕様と既知制約
- 必須不変条件（Invariants）: `docs/reference/invariants.md`
- 制約（既知/一時/解消済み）: `docs/reference/constraints.md`
- PHI と SSA の設計: `docs/reference/architecture/phi-and-ssa.md`
  - 既定のPHI挙動: Phase‑15 で PHI-ON（MIR14）が標準になったよ。ループ・break/continue・構造化制御の合流で PHI を必ず生成するよ。
  - レガシー互換: `NYASH_MIR_NO_PHI=1`（必要なら `NYASH_VERIFY_ALLOW_NO_PHI=1` も）で PHI-OFF（エッジコピー）に切り替えできるよ。
- テスト行列（仕様→テスト対応）: `docs/guides/testing-matrix.md`
- 他言語との比較: `docs/guides/comparison/nyash-vs-others.md`

プロファイル（クイック）
- `--profile dev` → マクロON（strict）、開発向け VM 既定（必要に応じて環境で上書き可）
- `--profile lite` → マクロOFF の軽量実行
  - 例: `./target/release/hakorune --profile dev --backend vm apps/tests/ternary_basic.nyash`

## 目次
- [Self-Hosting（自己ホスト開発）](#self-hosting)
- [🚀 速報: ネイティブEXE達成！](#-速報-ネイティブexe達成)

<a id="self-hosting"></a>
## 🧪 Self-Hosting（自己ホスト開発）
- ガイド: `docs/how-to/self-hosting.md`
- 最小E2E: `./target/release/hakorune --backend vm apps/selfhost-minimal/main.nyash`
- スモーク: `bash tools/jit_smoke.sh` / `bash tools/selfhost_vm_smoke.sh`
- Makefile: `make run-minimal`, `make smoke-selfhost`

MIR注記: Core‑13 最小カーネルは既定で有効（NYASH_MIR_CORE13=1）。旧命令は正規化されます（Array/Ref→BoxCall、TypeCheck/Cast/Barrier/WeakRefの統一）。

Core‑13 は既定で有効です。旧「純化モード（Core‑13 pure）」は撤廃されました。

変更履歴（要点）: `CHANGELOG.md`

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

flow Main {  # 推奨: flow Main（エントリ: Strict=Main.main）
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

エントリ規約（Strict）: 既定のエントリは `Main.main` のみ。詳細は `docs/reference/language/entrypoints.md` を参照。

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

重要: 現在、JIT ランタイム実行は封印中です。実行は「Rust VM（MIR）」、配布は「LLVM AOT（llvmlite ハーネス）」が主軸です。PyVM は開発補助として段階的撤退中（`pyvm-bridge` feature でのみ有効）。ASTインタープリタはレガシー扱いでデフォルト無効（`interpreter-legacy`）。

Phase‑15（自己ホスト期）: ASTインタープリタは任意featureで明示ON
- 既定ビルド: `--backend vm` は Rust VM（MIR）。PyVM を使うには `--features pyvm-bridge` でビルドし、`NYASH_VM_USE_PY=1` を明示してください（非推奨）。
- レガシー AST インタープリタを有効化するには（通常は不要）:
  ```bash
  cargo build --release --features interpreter-legacy
  ```

### 1. **インタープリターモード** （開発用）
```bash
./target/release/hakorune program.hkr
```
- 即座に実行
- 完全なデバッグ情報
- 開発に最適

### 2. **VMモード（既定は Rust VM）**
```bash
# Rust VM（既定）
./target/release/hakorune --backend vm program.hkr

# （非推奨）PyVM を使う場合: pyvm-bridge を有効化して実行
cargo build --release --features pyvm-bridge
NYASH_VM_USE_PY=1 ./target/release/hakorune --backend vm program.hkr
```
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

### 4. **ネイティブバイナリ（LLVM AOT, llvmliteハーネス）**
```bash
# ハーネス＋CLI をビルド（LLVM_SYS_180_PREFIX不要）
cargo build --release -p nyash-llvm-compiler && cargo build --release --features llvm

# ハーネス経由で EXE を生成して実行
NYASH_LLVM_USE_HARNESS=1 \
NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc \
NYASH_EMIT_EXE_NYRT=target/release \
  ./target/release/hakorune --backend llvm --emit-exe myapp program.hkr
./myapp

# あるいは .o を出力して手動リンク
NYASH_LLVM_USE_HARNESS=1 \
NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc \
  ./target/release/hakorune --backend llvm program.hkr \
  -D NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o
cc nyash_llvm_temp.o -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o myapp
./myapp
```

簡易スモークテスト（VM と EXE の出力一致確認）:
```bash
tools/smoke_aot_vs_vm.sh examples/aot_min_string_len.nyash
```

### LLVM バックエンドの補足
- Python llvmlite を使用します。Python3 + llvmlite の用意と `ny-llvmc` のビルド（`cargo build -p nyash-llvm-compiler`）が必要です。`LLVM_SYS_180_PREFIX` は不要です。
- `NYASH_LLVM_OBJ_OUT`: `--backend llvm` 実行時に `.o` を出力するパス。
  - 例: `NYASH_LLVM_OBJ_OUT=$PWD/nyash_llvm_temp.o ./target/release/hakorune --backend llvm apps/ny-llvm-smoke/main.nyash`
- 削除された `NYASH_LLVM_ALLOW_BY_NAME=1`: すべてのプラグイン呼び出しがmethod_idベースに統一。
  - LLVMバックエンドは性能と型安全性のため、method_idベースのプラグイン呼び出しのみ対応。


### 5. **WebAssembly（ブラウザ）** — 現状: 一時停止 / 未整備
WASM/ブラウザ経路は現在メンテ対象外です（CI未対象）。古いプレイグラウンド/ガイドは歴史的資料として残置しています。

- ソース（アーカイブ）: `projects/nyash-wasm/`（ビルド保証なし）
- 現在の主軸: VM（Rust）と LLVM（llvmlite ハーネス）
- ローカルで試す場合は `projects/nyash-wasm/README.md` と `projects/nyash-wasm/build.sh` を参照（wasm-pack 必須、サポート無保証）。

---

## 🧰 タスク実行 (hako.toml)

`hako.toml`（互換: `nyash.toml`）の `[tasks]` と `[env]` で、ビルド/スモークなどのタスクを簡単に実行できます（MVP）。

例（hako.toml の末尾に追記）:

```
[env]
RUST_BACKTRACE = "1"

[tasks]
# llvmlite ハーネス＋CLI をビルド（LLVM_SYS_180_PREFIX不要）
build_llvm = "cargo build --release -p nyash-llvm-compiler && cargo build --release --features llvm"
smoke_obj_array = "NYASH_LLVM_USE_HARNESS=1 NYASH_NY_LLVM_COMPILER={root}/target/release/ny-llvmc NYASH_LLVM_OBJ_OUT={root}/nyash_llvm_temp.o ./target/release/nyash --backend llvm apps/ny-llvm-smoke/main.nyash"
```

実行:

```
./target/release/hakorune --run-task build_llvm
./target/release/hakorune --run-task smoke_obj_array
```

補足:
- `[env]` の値は実行前に環境へ適用されます。
- `{root}` は現在のプロジェクトルートに展開されます。
- 現状は最小機能（OS別/依存/並列は未対応）。

### ちいさなENVまとめ（VM vs LLVM ハーネス）
- VM 実行: 追加ENVなしでOK。
  - 例: `./target/release/hakorune --backend vm apps/tests/ternary_basic.nyash`
- LLVM ハーネス実行: 下記3つだけ設定してね。
  - `NYASH_LLVM_USE_HARNESS=1`
  - `NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc`
  - `NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release`
  - 例: `NYASH_LLVM_USE_HARNESS=1 NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc NYASH_EMIT_EXE_NYRT=target/release ./target/release/hakorune --backend llvm apps/ny-llvm-smoke/main.nyash`

### DebugHub かんたんガイド
- 有効化: `NYASH_DEBUG_ENABLE=1`
- 種別指定: `NYASH_DEBUG_KINDS=resolve,ssa`
- 出力先: `NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl`
- 例: `NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash.jsonl tools/smokes/v2/run.sh --profile quick --filter "userbox_*"`

### 開発用セーフティ（VM）
- stringify(Void) は "null" を返す（JSONフレンドリ／開発セーフティ。既定挙動は不変）。
- JsonScanner のデフォルト値（`NYASH_VM_SCANNER_DEFAULTS=1` 時のみ）: `is_eof/current/advance` 内に限定し、数値/テキストの不足を安全に埋める。
- VoidBox に対する length/size/get/push 等は、ガード下で中立なノーオペとして扱い、開発中のハードストップを回避。

---

## 🧰 一発ビルド（MVP）: `nyash --build`

`hako.toml`（互換: `nyash.toml`）を読み、プラグイン → コア → AOT → リンクまでを一発実行する最小ビルド機能です。

基本（Cranelift AOT）
```bash
./target/release/hakorune --build hako.toml \
  --app apps/egui-hello-plugin/main.nyash \
  --out app_egui
```

主なオプション（最小）
- `--build <path>`: hako.toml の場所（互換: nyash.toml）
- `--app <file>`: エントリ `.nyash`
- `--out <name>`: 出力EXE名（既定: `app`/`app.exe`）
- `--build-aot cranelift|llvm`（既定: cranelift）
- `--profile release|debug`（既定: release）
- `--target <triple>`（必要時のみ）

注意
- LLVM AOT は Python の llvmlite ハーネスを使用します。Python3 + llvmlite と `ny-llvmc` のビルド（`cargo build -p nyash-llvm-compiler`）が必要です。`LLVM_SYS_180_PREFIX` は不要です。
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
# hako.toml - プラグイン設定（互換: nyash.toml）
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
./target/release/hakorune hello.hkr
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
