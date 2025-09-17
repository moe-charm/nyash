# Nyashスクリプトプラグイン

## 📦 概要

Nyashスクリプトプラグインは、**最もNyashらしいプラグインシステム**です。特別な仕組みは不要で、ユーザー定義Boxがそのままプラグインとして機能します。

> 💡 **「Everything is Box」哲学の究極形**  
> プラグインも、ただのBoxです！

## 🎯 特徴

### 究極のシンプルさ
- **特別なAPIは不要** - 普通のNyashコードを書くだけ
- **学習コストゼロ** - Nyashを知っていれば書ける
- **即座に使える** - コンパイル不要、そのまま実行

### 完全な統合
- Nyashの全機能が使える
- 他のプラグイン（C ABI/Nyash ABI）も自由に組み合わせ可能
- デバッグも通常のNyashコードと同じ

### 高い生産性
- ホットリロード対応（開発中に即反映）
- テストが簡単（通常のNyashテストフレームワーク使用可）
- ドキュメント自動生成対応

## 📝 実装例

### 1. シンプルなユーティリティBox

```nyash
# math_utils.nyash - 数学ユーティリティプラグイン

box MathUtils {
    init { }
    
    # 階乗を計算
    factorial(n) {
        if n <= 1 {
            return 1
        }
        return n * me.factorial(n - 1)
    }
    
    # フィボナッチ数列
    fibonacci(n) {
        if n <= 1 {
            return n
        }
        return me.fibonacci(n - 1) + me.fibonacci(n - 2)
    }
    
    # 最大公約数
    gcd(a, b) {
        if b == 0 {
            return a
        }
        return me.gcd(b, a % b)
    }
}

# エクスポート（将来のexport構文）
# export MathUtils
```

### 2. 他のプラグインと組み合わせる例

```nyash
# data_processor.nyash - データ処理プラグイン

box DataProcessor {
    init { file, math, cache }
    
    birth(outputPath) {
        me.file = new FileBox()      # C ABIプラグイン
        me.math = new MathBox()      # C ABIプラグイン
        me.cache = new MapBox()      # C ABIプラグイン
    }
    
    # CSVデータを処理
    processCSV(inputPath, outputPath) {
        # ファイル読み込み
        local data = me.file.read(inputPath)
        local lines = data.split("\n")
        
        # 各行を処理
        local results = new ArrayBox()
        for line in lines {
            local values = line.split(",")
            local sum = 0
            
            for value in values {
                local num = value.toFloat()
                # 三角関数で変換（C ABIのMathBox使用）
                local transformed = me.math.sin(num)
                sum = sum + transformed
            }
            
            results.push(sum)
        }
        
        # 結果を保存
        me.file.write(outputPath, results.join("\n"))
        return results
    }
}
```

### 3. 高度なプラグイン - P2Pノード拡張

```nyash
# mesh_node.nyash - P2Pメッシュネットワークノード

box MeshNode from P2PBox {
    init { routing, peers, messageHandlers }
    
    pack(nodeId, transport) {
        # 親クラス（P2PBox）の初期化
        from P2PBox.pack(nodeId, transport)
        
        # 追加の初期化
        me.routing = new RoutingTable()
        me.peers = new MapBox()
        me.messageHandlers = new MapBox()
        
        # デフォルトハンドラー登録
        me.registerHandler("ping", me.handlePing)
        me.registerHandler("route", me.handleRoute)
    }
    
    # メッセージハンドラー登録
    registerHandler(messageType, handler) {
        me.messageHandlers.set(messageType, handler)
    }
    
    # オーバーライド: メッセージ送信時にルーティング
    override send(target, message) {
        # 最適なルートを探す
        local nextHop = me.routing.findBestRoute(target)
        
        if nextHop == null {
            # 直接送信を試みる
            return from P2PBox.send(target, message)
        }
        
        # ルーティング経由で送信
        local routedMessage = {
            type: "route",
            finalTarget: target,
            payload: message
        }
        
        return from P2PBox.send(nextHop, routedMessage)
    }
    
    # Pingハンドラー
    handlePing(sender, data) {
        me.send(sender, {
            type: "pong",
            timestamp: new TimeBox().now()
        })
    }
    
    # ルーティングハンドラー
    handleRoute(sender, data) {
        local finalTarget = data.finalTarget
        
        if finalTarget == me.nodeId {
            # 自分宛て
            me.processMessage(sender, data.payload)
        } else {
            # 転送
            me.send(finalTarget, data.payload)
        }
    }
}
```

## 🚀 プラグインの配布と使用

### 1. ローカルファイルとして

```nyash
# main.nyash
include "plugins/math_utils.nyash"

local utils = new MathUtils()
print(utils.factorial(5))  # 120
```

### 2. パッケージとして（将来）

```bash
# パッケージのインストール
nyash install awesome-math-utils

# パッケージの公開
nyash publish my-cool-plugin
```

```nyash
# パッケージの使用
import { MathUtils } from "awesome-math-utils"

local utils = new MathUtils()
```

### 3. 動的ロード

```nyash
# 実行時にプラグインをロード
local pluginCode = new FileBox().read("plugin.nyash")
eval(pluginCode)  # プラグインが利用可能に

local processor = new DataProcessor()
```

## 💡 ベストプラクティス

### 1. 単一責任の原則
```nyash
# ✅ 良い例：特定の機能に集中
box JSONParser {
    parse(text) { ... }
    stringify(obj) { ... }
}

# ❌ 悪い例：何でも詰め込む
box UtilityBox {
    parseJSON() { ... }
    sendEmail() { ... }
    calculateTax() { ... }
    playSound() { ... }
}
```

### 2. 依存性の明示
```nyash
# ✅ 良い例：必要な依存を明示
box DataAnalyzer {
    init { fileReader, mathLib, logger }
    
    birth() {
        me.fileReader = new FileBox()
        me.mathLib = new MathBox()
        me.logger = new LoggerBox()
    }
}
```

### 3. エラーハンドリング
```nyash
# ✅ 良い例：適切なエラー処理
box SafeCalculator {
    divide(a, b) {
        if b == 0 {
            throw new Error("Division by zero")
        }
        return a / b
    }
}
```

## 📊 他のプラグインシステムとの比較

| 特徴 | Nyashスクリプト | C ABI | Nyash ABI |
|------|----------------|-------|-----------|
| 実装言語 | Nyash | C/C++ | 任意 |
| 学習コスト | ゼロ | 中 | 高 |
| パフォーマンス | 中速 | 最速 | 高速 |
| 開発効率 | 最高 | 中 | 中 |
| デバッグ | 簡単 | 難しい | 中程度 |
| 配布 | .nyashファイル | .so/.dll | 任意 |

## 📚 まとめ

Nyashスクリプトプラグインは「**Everything is Box**」哲学の究極の実現です。

- **特別なAPIは不要** - 普通のNyashコードがプラグイン
- **完全な統合** - Nyashの全機能が使える
- **高い生産性** - 書いてすぐ使える

**迷ったらNyashスクリプトプラグインから始めましょう！**

必要に応じて、パフォーマンスが必要な部分だけC ABIに、他言語連携が必要な部分だけNyash ABIに移行すれば良いのです。