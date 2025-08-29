# 🐱 Nyash プログラミング言語
**20日でゼロからネイティブバイナリへ - AI駆動の言語革命**

*[🇺🇸 English Version / 英語版はこちら](README.md)*

[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](#)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![Performance](https://img.shields.io/badge/Performance-13.5x%20高速化-ff6b6b.svg)](#performance)
[![JIT Ready](https://img.shields.io/badge/JIT-Cranelift%20搭載-orange.svg)](#execution-modes)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

## 🚀 **速報: ネイティブEXE達成！**

**2025年8月29日** - 誕生からわずか20日で、Nyashがネイティブ実行ファイルへのコンパイルを実現！

```bash
# Nyashソースからネイティブバイナリへ
./target/release/nyash --backend vm program.nyash  # JITコンパイル
./tools/build_aot.sh program.nyash -o app         # ネイティブEXE
./app                                              # スタンドアロン実行！
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

### 1. **インタープリターモード** （開発用）
```bash
./target/release/nyash program.nyash
```
- 即座に実行
- 完全なデバッグ情報
- 開発に最適

### 2. **VMモード** （本番用）
```bash
./target/release/nyash --backend vm program.nyash
```
- インタープリターより13.5倍高速
- 最適化されたバイトコード実行
- 本番環境対応のパフォーマンス

### 3. **JITモード** （高性能）
```bash
NYASH_JIT_EXEC=1 ./target/release/nyash --backend vm program.nyash
```
- Cranelift搭載JITコンパイル
- ほぼネイティブ性能
- ホット関数最適化

### 4. **ネイティブバイナリ** （配布用）
```bash
./tools/build_aot.sh program.nyash -o myapp
./myapp  # スタンドアロン実行！
```
- 依存関係ゼロ
- 最高性能
- 簡単配布

### 5. **WebAssembly** （ブラウザ用）
```bash
./target/release/nyash --compile-wasm program.nyash
```
- ブラウザで実行
- デフォルトでクロスプラットフォーム
- Webファースト開発

---

## 📊 **パフォーマンスベンチマーク**

実世界ベンチマーク結果 (ny_bench.nyash)：

```
モード           | 時間      | 相対速度
----------------|-----------|---------------
インタープリター | 110.10ms  | 1.0x (基準)
VM              | 8.14ms    | 13.5倍高速
VM + JIT        | 5.8ms     | 19.0倍高速
ネイティブ      | ~4ms      | ~27倍高速
```

---

## 🎮 **言語機能**

### クリーンな構文
```nyash
box GameCharacter {
    init { name, health, skills }
    
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
    init { logger }
    
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
    init { x, y, sprite }
    
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

## 📄 **ライセンス**

MIT ライセンス - プロジェクトで自由に使用してください！

---

## 👨‍💻 **作者**

**Tomoaki** - 言語設計者＆革命家
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