# Nyash Export/Import構文仕様 v1.0

## 🎯 概要

Nyashのコード共有エコシステムを実現するための`export`/`import`構文仕様。「Everything is Box」哲学に基づき、Boxを中心とした明快な構文を提供する。

## 📝 基本構文

### Export構文

```nyash
# 単一Boxのエクスポート
export box MathUtils {
    init { precision }
    
    factorial(n) {
        if n <= 1 { return 1 }
        return n * me.factorial(n - 1)
    }
    
    fibonacci(n) {
        if n <= 1 { return n }
        return me.fibonacci(n - 1) + me.fibonacci(n - 2)
    }
}

# Static Boxのエクスポート
export static box Constants {
    init { }
    
    PI = 3.14159265359
    E = 2.71828182846
    GOLDEN_RATIO = 1.61803398875
}

# 複数Boxの名前付きエクスポート
export {
    MathUtils,
    Constants,
    StringHelpers as StrUtils  # エイリアス付き
}

# デフォルトエクスポート
export default box Calculator {
    init { display }
    // ...
}
```

### Import構文

```nyash
# 名前付きインポート
import { MathUtils } from "math_utils.ny"
import { MathUtils, Constants } from "math_lib.ny"

# エイリアス付きインポート
import { MathUtils as Math } from "math_utils.ny"

# デフォルトインポート
import Calculator from "calculator.ny"

# 全体インポート（名前空間）
import * as MathLib from "math_lib.ny"

# 複合インポート
import Calculator, { MathUtils, Constants } from "advanced_calc.ny"
```

## 🔧 モジュール解決

### ファイルパス解決

```nyash
# 相対パス
import { Utils } from "./utils.ny"
import { Common } from "../common/helpers.ny"

# パッケージ名（nyash_modules/から）
import { Logger } from "awesome-logger"

# 絶対パス（非推奨、移植性のため）
import { Config } from "/home/user/project/config.ny"
```

### 解決順序

1. 相対パス（`./`または`../`で始まる）
2. `nyash_modules/`ディレクトリ
3. グローバルパッケージディレクトリ（設定可能）
4. 絶対パス

## 📦 パッケージ構造

### 基本的なパッケージ構成

```
my-math-package/
├── nyash.toml          # パッケージメタデータ
├── src/
│   ├── index.ny        # メインエントリーポイント
│   ├── utils.ny
│   └── advanced.ny
├── tests/
│   └── test_math.ny
└── README.md
```

### nyash.toml

```toml
[package]
name = "awesome-math"
version = "1.0.0"
description = "素晴らしい数学ユーティリティ"
author = "Nyash Developer"
license = "MIT"

[dependencies]
# 他のNyashパッケージへの依存
basic-utils = "^2.0.0"

[export]
# パッケージのメインエクスポート
main = "src/index.ny"
```

### index.ny（エントリーポイント）

```nyash
# 内部モジュールをインポート
import { InternalUtils } from "./utils.ny"
import { AdvancedMath } from "./advanced.ny"

# 外部にエクスポート
export {
    InternalUtils as Utils,
    AdvancedMath
}

# デフォルトエクスポート
export default box MathPackage {
    init {
        me.utils = new Utils()
        me.advanced = new AdvancedMath()
    }
}
```

## 🚀 高度な機能

### 条件付きエクスポート

```nyash
# プラットフォーム別エクスポート
if PLATFORM == "web" {
    export { WebLogger as Logger } from "./web_logger.ny"
} else {
    export { ConsoleLogger as Logger } from "./console_logger.ny"
}
```

### 再エクスポート

```nyash
# 他のモジュールから再エクスポート
export { MathUtils } from "./math_utils.ny"
export * from "./string_helpers.ny"
```

### 動的インポート（将来拡張）

```nyash
# 実行時に動的にインポート
local dynamicModule = await import("./heavy_module.ny")
local HeavyBox = dynamicModule.HeavyBox
```

## 🔒 スコープとアクセス制御

### プライベートメンバー

```nyash
export box SecureBox {
    init { 
        _privateData  # アンダースコアプレフィックスは慣習的にプライベート
        publicData
    }
    
    # プライベートメソッド（エクスポートされない）
    _internalProcess() {
        // 内部処理
    }
    
    # パブリックメソッド
    process() {
        return me._internalProcess()
    }
}
```

## 🎯 実装優先順位

### Phase 1: 基本機能（必須）
- [ ] `export box`構文
- [ ] `import { Box } from "file"`構文
- [ ] 相対パス解決
- [ ] 基本的な循環参照チェック

### Phase 2: 拡張機能（推奨）
- [ ] `export default`
- [ ] `import * as namespace`
- [ ] エイリアス（`as`）
- [ ] nyash_modules/ディレクトリサポート

### Phase 3: 高度な機能（オプション）
- [ ] 条件付きエクスポート
- [ ] 再エクスポート
- [ ] 動的インポート
- [ ] パッケージマネージャー統合

## ⚠️ 制約事項

1. **循環参照の禁止**
   ```nyash
   # ❌ エラー: 循環参照
   # a.ny: import { B } from "./b.ny"
   # b.ny: import { A } from "./a.ny"
   ```

2. **トップレベルでのみ許可**
   ```nyash
   # ✅ OK
   import { Utils } from "./utils.ny"
   
   # ❌ エラー: 関数内でのインポート
   box MyBox {
       method() {
           import { Helper } from "./helper.ny"  # エラー！
       }
   }
   ```

3. **export前の参照禁止**
   ```nyash
   # ❌ エラー: 定義前のエクスポート
   export { UndefinedBox }  # エラー！
   
   box UndefinedBox { }
   ```

## 🔄 他言語との比較

| 機能 | Nyash | JavaScript | Python | Rust |
|------|-------|------------|--------|------|
| 名前付きexport | ✅ | ✅ | ✅ | ✅ |
| デフォルトexport | ✅ | ✅ | ❌ | ❌ |
| 名前空間import | ✅ | ✅ | ✅ | ✅ |
| 動的import | 🔄 | ✅ | ✅ | ❌ |
| 再export | ✅ | ✅ | ✅ | ✅ |

## 📚 使用例

### 数学ライブラリ

```nyash
# math_lib.ny
export box Vector2D {
    init { x, y }
    
    add(other) {
        return new Vector2D(me.x + other.x, me.y + other.y)
    }
    
    magnitude() {
        return Math.sqrt(me.x * me.x + me.y * me.y)
    }
}

export static box MathConstants {
    init { }
    TAU = 6.28318530718
}
```

### 使用側

```nyash
# game.ny
import { Vector2D, MathConstants } from "./math_lib.ny"

box Player {
    init { 
        me.position = new Vector2D(0, 0)
        me.velocity = new Vector2D(1, 1)
    }
    
    update() {
        me.position = me.position.add(me.velocity)
        local angle = MathConstants.TAU / 4  # 90度
    }
}
```

---

*Everything is Box - そしてBoxは共有される*