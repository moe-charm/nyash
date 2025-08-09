# 🐱 Nyash プログラミング言語
**次世代ブラウザーネイティブ開発体験**

*[🇺🇸 English Version / 英語版はこちら](README.md)*

[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)](#)
[![Everything is Box](https://img.shields.io/badge/Philosophy-Everything%20is%20Box-blue.svg)](#philosophy)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Ready-orange.svg)](#webassembly)
[![Try Now](https://img.shields.io/badge/Try%20Now-Browser%20Playground-ff6b6b.svg)](https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html)
[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](#license)

---

## 🚀 **今すぐNyashを体験！**

**インストール不要、設定不要 - ブラウザーを開くだけ！**

👉 **[🎮 Nyashブラウザープレイグラウンド起動](https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html)** 👈

体験できる機能：
- 🎨 **アーティスト協同制作デモ** - 複数Boxインスタンスの連携
- ⚡ **非同期計算処理** - シンプルな並列処理
- 🎮 **Canvas ゲームグラフィック** - ブラウザーでの直接グラフィック描画
- 🔍 **ライブデバッグ可視化** - プログラムのメモリ状態をリアルタイム表示

---

## ✨ **Nyashが革命を起こす理由**

### 🎯 **メモリ安全性の革命**
```nyash
// 従来の言語: 手動メモリ管理、クラッシュ、セキュリティ問題
// Nyash: Everything is Box - 自動的、安全、エレガント

static box Main {
    init { player, enemies, canvas }
    
    main() {
        me.player = new PlayerBox("勇者", 100)
        me.canvas = new WebCanvasBox("game", 800, 600)
        
        // メモリは自動管理 - クラッシュなし、メモリリークなし！
        me.player.render(me.canvas)
        return "ゲーム安全実行中！"
    }
}
```

### 🌐 **ブラウザーファースト設計**
- **ゼロインストール**: WebAssembly経由でWebブラウザーで直接実行
- **Web API内蔵**: Canvas、DOM、ストレージ - すべてが言語ネイティブ機能
- **リアルタイム協業**: コードを即座に共有、どこでも実行
- **モバイル対応**: スマートフォン、タブレット、すべての現代デバイスで動作

### 🎨 **創作プログラミングが簡単に**
```nyash
// コードでアートを作る - 自然に！
box Artist {
    init { name, color }
    
    paintMasterpiece(canvas) {
        canvas.fillCircle(100, 100, 50, me.color)
        canvas.fillText("Art by " + me.name, 10, 200, "24px Arial", me.color)
    }
}

// 複数のアーティストが協力
picasso = new Artist("ピカソ", "red")
monet = new Artist("モネ", "blue")
// 各Boxが独自の状態と動作を維持！
```

### ⚡ **非同期処理の簡潔性**
```nyash
// 複雑さなしの並列処理
nowait future1 = heavyComputation(10000)
nowait future2 = renderGraphics()

// 実行中に他の作業を...
setupUI()

// 準備ができたら結果を取得
result1 = await future1
result2 = await future2
```

---

## 🏗️ **革命的アーキテクチャ**

### Everything is Box 哲学
Nyashのすべての値は **Box** - 統一された、メモリ安全なコンテナです：

| 従来の言語 | Nyash |
|-----------|-------|
| `int x = 42;` | `x = new IntegerBox(42)` |
| `string name = "Hello";` | `name = new StringBox("Hello")` |
| 複雑なcanvas設定 | `canvas = new WebCanvasBox("game", 800, 600)` |
| 手動メモリ管理 | 自動Boxライフサイクル管理 |

### Static Box Main パターン
```nyash
// クリーンで予測可能なプログラム構造
static box Main {
    init { database, ui, gameState }  // すべてのフィールドを事前宣言
    
    main() {
        // 論理的順序で初期化
        me.database = new DatabaseBox("save.db")
        me.ui = new UIManagerBox()
        me.gameState = new GameStateBox()
        
        // プログラムロジックここに
        return runGameLoop()
    }
}
```

### 視覚的デバッグ統合
```nyash
debug = new DebugBox()
debug.startTracking()

player = new PlayerBox("勇者")
debug.trackBox(player, "メインキャラクター")

// ブラウザーでリアルタイムメモリ可視化！
print(debug.memoryReport())  // ライブ統計、デバッグ地獄なし
```

---

## 🎮 **創作コーディングに最適**

### ゲーム開発
- **内蔵Canvas API**: 外部ライブラリなしでグラフィック
- **入力ハンドリング**: マウス、キーボード、タッチ - すべてネイティブ
- **オーディオサポート**: 音楽と効果音用のSoundBox
- **物理準備済み**: 数学演算最適化

### 教育的プログラミング
- **視覚的フィードバック**: コードの効果を即座に確認
- **メモリ可視化**: プログラムの動作を理解
- **設定バリアなし**: 学生はブラウザーで即座にコーディング
- **段階的学習**: 簡単なスクリプトから複雑なアプリケーションまで

### Webアプリケーション
- **直接DOM制御**: WebDisplayBoxでHTML操作
- **フレームワーク不要**: 言語がネイティブでWeb相互作用を処理
- **リアルタイム更新**: 変更が即座に反映
- **クロスプラットフォーム**: 同じコード、どこでも

---

## 📖 **言語の特徴**

### クリーンで表現力豊かな構文
```nyash
// 自然なオブジェクト指向プログラミング
box Player {
    init { name, health, inventory }
    
    Player(playerName) {
        me.name = playerName
        me.health = 100
        me.inventory = new ArrayBox()
    }
    
    takeDamage(amount) {
        me.health = me.health - amount
        if me.health <= 0 {
            me.respawn()
        }
    }
    
    respawn() {
        me.health = 100
        print(me.name + " がリスポーンしました！")
    }
}
```

### 強力な演算子
```nyash
// 明確性のための自然言語演算子
isAlive = health > 0 and not poisoned
canCast = mana >= spellCost or hasItem("魔法の指輪")
gameOver = playerDead or timeUp

// 内蔵数学演算
distance = sqrt((x2 - x1)^2 + (y2 - y1)^2)
angle = atan2(deltaY, deltaX)
```

### ジェネリックプログラミング
```nyash
// 型安全なジェネリックコンテナ
box Container<T> {
    init { value }
    
    Container(item) { me.value = item }
    getValue() { return me.value }
}

numbers = new Container<IntegerBox>(42)
texts = new Container<StringBox>("こんにちは")
```

---

## 🛠️ **使い始める**

### ブラウザー開発（推奨）
```bash
# 1. リポジトリクローン
git clone https://github.com/moe-charm/nyash.git
cd nyash

# 2. WebAssemblyバージョンビルド
cd projects/nyash-wasm
./build.sh

# 3. ブラウザーでプレイグラウンドを開く
# 任意の現代ブラウザーでnyash_playground.htmlを開く
```

### ネイティブ開発
```bash
# ネイティブバージョンビルド
cargo build --release

# プログラムをローカルで実行
./target/release/nyash program.nyash

# 例を試す
./target/release/nyash test_async_demo.nyash
./target/release/nyash app_dice_rpg.nyash
```

---

## 🎯 **対象ユーザー**

- 🎨 **クリエイター**: アーティスト、ゲーム開発者
- 🎓 **教育者**: プログラミング講師、学生
- 🌐 **Web開発者**: インタラクティブコンテンツ制作者
- 🔬 **研究者**: 新しいプログラミングパラダイムの探求者

---

## 🤝 **貢献**

Nyashはオープンソースで、貢献を歓迎しています！

- **Issues**: バグ報告、機能リクエスト
- **Pull Requests**: コード改善、新しい例
- **ドキュメント**: ガイドと例の改善支援
- **コミュニティ**: Nyash作品を共有！

## 📄 **ライセンス**

MIT ライセンス - 個人および商用利用無料。

---

## 🔗 **リンク**

- **[🎮 今すぐ試す - ブラウザープレイグラウンド](https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html)**
- **[📚 ドキュメント](docs/)**
- **[🎯 例](examples/)**
- **[💬 コミュニティディスカッション](https://github.com/moe-charm/nyash/discussions)**

## 👨‍💻 **作者**

**Moe Charm** - プログラミング言語デザイナー・開発者
- 🐙 GitHub: [@moe-charm](https://github.com/moe-charm)  
- 🐦 Twitter/X: [@CharmNexusCore](https://x.com/CharmNexusCore)
- ☕ 開発サポート: [coff.ee/moecharmde6](http://coff.ee/moecharmde6)

*AI支援と献身的開発で革新的プログラミング言語を創造 🤖*

---

## 🤖 **プロジェクトのサポート**

Nyashは最先端のAI協業で開発されています！

継続的な開発をサポートしたい場合：

**☕ [開発サポート](http://coff.ee/moecharmde6)** - イノベーションの燃料を！

*Claude Code による支援 - 高度なAI開発ツールは無料ではありません！ 🤖*

あなたのサポートはプロジェクトの維持、新機能の開発、プログラミング言語設計の境界を押し広げることに役立ちます。すべての貢献が違いを生みます！ 🙏

---

*❤️、🤖 Claude Code、そしてEverything is Box哲学で構築*

**Nyash - すべての値がBoxであり、すべてのBoxが物語を語る場所。**