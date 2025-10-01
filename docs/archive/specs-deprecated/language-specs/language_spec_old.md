# 🚀 Nyash Language Reference 2025

**最終更新: 2025年8月11日 - デリゲーション革命完了！`from`統一構文＋`init`構文決定！**

## 📖 概要

Nyashは「Everything is Box」哲学に基づく革新的プログラミング言語です。
Rust製インタープリターによる高性能実行と、直感的な構文により、学習しやすく実用的な言語として完成しました。

---

## 🔤 **1. 予約語・キーワード完全リスト**

### **コア言語**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `box` | クラス定義 | `box MyClass { }` |
| `static` | 静的Box・関数定義 | `static box Main { }` |
| `interface` | インターフェース定義 | `interface Comparable { }` |
| `from` | デリゲーション指定 | `box Child from Parent { }` |
| `new` | オブジェクト生成 | `new ConsoleBox()` |
| `me`/`this` | 自己参照 | `me.field = value` |

### **変数・スコープ**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `local` | ローカル変数宣言 | `local x, y = 10` |
| `outbox` | 所有権移転変数 | `outbox result = compute()` |
| `global` | グローバル変数 | `global CONFIG = "dev"` |
| `init` | フィールド初期化ブロック | `init { name, age }` |

### **制御構文**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `if` | 条件分岐 | `if condition { }` |
| `else` | else節 | `else { }` |
| `loop` | ループ（唯一の形式） | `loop(condition) { }` |
| `break` | ループ脱出 | `break` |
| `return` | 関数リターン | `return value` |

### **論理・演算**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `not` | 論理否定 | `not condition` |
| `and` | 論理積 | `a and b` |
| `or` | 論理和 | `a or b` |
| `true`/`false` | 真偽値 | `flag = true` |

### **非同期・並行**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `nowait` | 非同期実行 | `nowait future = task()` |
| `await` | 待機・結果取得 | `result = await future` |

### **例外処理**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `try` | 例外捕獲開始 | `try { }` |
| `catch` | 例外処理 | `catch (e) { }` |
| `finally` | 最終処理 | `finally { }` |
| `throw` | 例外発生 | `throw error` |

### **その他**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `function` | 関数定義 | `function add(a,b) { }` |
| `print` | 出力 | `print("Hello")` |
| `include` | ファイル取り込み | `include "math.nyash"` |

---

## 📝 **2. 文法・構文仕様**

### **2.1 Box定義文法**

#### **基本Box**
```nyash
box ClassName {
    init { field1, field2, field3 }  # カンマ必須！CPU暴走防止
    
    # コンストラクタ
    init(param1, param2) {  # init構文に統一
        me.field1 = param1
        me.field2 = param2
        me.field3 = defaultValue()
    }
    
    # メソッド
    methodName(arg1, arg2) {
        return me.field1 + arg1
    }
    
    # デストラクタ
    fini() {
        print("Cleanup: " + me.field1)
    }
}
```

#### フィールドの可視性（public/private）
- `init { ... }` で宣言したフィールドは既定で public（外部から参照・代入可能）
- 追加で private フィールドを宣言する場合は `private { ... }` ブロックを使用
- 外部から private フィールドへ直接アクセスすることはできません（ゲッター/セッター経由）

```nyash
box User {
    // 既定: init は public フィールド宣言
    init { name }

    // private フィールド宣言
    private { age, passwordHash }

    birth(n, a) { me.name = n; me.age = a }

    setAge(a) { me.age = a }
    getAge() { return me.age }
}
```

補足: 旧仕様では `init` の可視性が明確ではありませんでしたが、現在は「public」に統一されています。

#### **デリゲーションBox**
```nyash
box Child from Parent interface Comparable {
    init { childField }
    
    init(parentParam, childParam) {  # init構文に統一
        from Parent.init(parentParam)  # 親コンストラクタ明示呼び出し
        me.childField = childParam
    }
    
    # メソッドオーバーライド
    override process(data) {  # override必須
        local result = from Parent.process(data)  # 親メソッド呼び出し
        return result + " (Child processed)"
    }
    
    # インターフェース実装
    compareTo(other) {
        return me.value - other.value
    }
}
```

#### **Static Box（推奨エントリーポイント）**
```nyash
static box Main {
    init { console, result }
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        return "Success"
    }
}
```

#### **ジェネリックBox**
```nyash
box Container<T> {
    init { value }
    
    Container(item) {
        me.value = item
    }
    
    getValue() {
        return me.value
    }
}
```

### **2.2 変数宣言**

#### **基本パターン**
```nyash
# 単一宣言
local x
local name = "初期値"

# 複数宣言
local a, b, c
local x = 10, y = 20, z  # 混合初期化

# 所有権移転（static関数内）
static function Factory.create() {
    outbox product  # 呼び出し側に所有権移転
    product = new Item()
    return product
}
```

#### **変数宣言厳密化システム（2025-08-09実装）**
```nyash
# ✅ 正しい - 明示宣言必須
local temp
temp = 42

# ❌ エラー - 未宣言変数への代入
x = 42  # RuntimeError: 未宣言変数 + 修正提案表示
```

### **2.3 制御構文**

#### **条件分岐**
```nyash
if condition {
    # 処理
} else if condition2 {
    # 処理2  
} else {
    # else処理
}
```

#### **ループ（統一構文）**
```nyash
# ✅ 唯一の正しい形式
loop(condition) {
    # ループ本体
    if exitCondition {
        break
    }
}

# ❌ 削除済み - 使用不可
while condition { }  # パーサーエラー
loop() { }          # パーサーエラー
```

### **2.4 演算子・式**

#### **🚀 新実装: 関数オーバーロードシステム**
```nyash
# Rust風トレイトベース演算子（2025-08-10実装完了）
sum = 10 + 20           # IntegerBox + IntegerBox = IntegerBox
concat = "Hi" + " !"    # StringBox + StringBox = StringBox  
repeat = "Ha" * 3       # StringBox * IntegerBox = "HaHaHa"
mixed = 42 + " answer"  # 混合型 → 自動文字列結合フォールバック
```

#### **演算子優先順位**
```nyash
result = a + b * c / d - e    # 算術演算子は標準的優先順位
logic = not a and b or c      # not > and > or
compare = (x > y) and (z <= w)  # 比較は括弧推奨
```

#### **論理演算子**
```nyash
# キーワード版（推奨）
canAccess = level >= 5 and hasKey
isValid = not (isEmpty or hasError)

# シンボル版（互換）
result = condition && other || fallback  # 利用可能だが非推奨
```

---

## 🏗️ **3. Box構文詳細ガイド**

### **3.1 Everything is Box 原則**

```nyash
# すべての値がBox
number = 42               # IntegerBox
text = "hello"           # StringBox
flag = true              # BoolBox
array = new ArrayBox()   # ArrayBox
console = new ConsoleBox() # ConsoleBox

# 統一的なメソッド呼び出し
print(number.to_string_box().value)  # "42"
print(array.length())               # 配列長
console.log("Everything is Box!")   # コンソール出力
```

### **3.2 コンストラクタパターン**

#### **パラメータ付きコンストラクタ**
```nyash
box Person {
    init { name, age, email }
    
    init(personName, personAge) {  # init構文に統一
        me.name = personName
        me.age = personAge  
        me.email = me.name + "@example.com"  # 計算フィールド
    }
    
    # ファクトリーメソッド
    static createGuest() {
        outbox guest
        guest = new Person("Guest", 0)
        return guest
    }
}

# 使用例
person = new Person("Alice", 25)
guest = Person.createGuest()
```

### **3.3 継承とインターフェース**

#### **デリゲーションチェーン**
```nyash
# 基底Box
box Animal {
    init { name, species }
    
    init(animalName, animalSpecies) {
        me.name = animalName
        me.species = animalSpecies
    }
    
    speak() {
        return me.name + " makes a sound"
    }
}

# デリゲーション
box Dog from Animal {
    init { breed }  # 追加フィールド
    
    init(dogName, dogBreed) {
        from Animal.init(dogName, "Canine")  # 親コンストラクタ呼び出し
        me.breed = dogBreed
    }
    
    override speak() {  # 明示的オーバーライド
        return me.name + " barks: Woof!"
    }
}

# インターフェース実装
box Cat from Animal interface Playful {
    # Playfulインターフェースの実装必須
}
```

### **3.4 Static Boxパターン**

#### **名前空間・ユーティリティ**
```nyash
static box MathUtils {
    init { PI, E }
    
    static {
        me.PI = 3.14159265
        me.E = 2.71828182
    }
    
    add(a, b) {
        return a + b
    }
    
    circleArea(radius) {
        return me.PI * radius * radius
    }
}

# 使用法
area = MathUtils.circleArea(5)
sum = MathUtils.add(10, 20)
pi = MathUtils.PI
```

#### **アプリケーションエントリーポイント**
```nyash
# 🎯 推奨: Static Box Main パターン
static box Main {
    init { console, result }
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("🚀 Starting application...")
        
        # アプリケーションロジック
        me.result = processData()
        
        return "Application completed successfully"
    }
}
```

---

## 🚀 **4. 最新機能・革新技術**

### **4.1 Arc<Mutex> Revolution（2025-08-10）**
```nyash
# 全16種類のBox型が統一Arc<Mutex>パターンで実装
# 完全なスレッドセーフティと高性能を両立

array = new ArrayBox()
array.push(10)           # スレッドセーフな追加
array.push(20)
item = array.get(0)      # スレッドセーフな取得

json = new JSONBox()
json.set("name", "Alice")    # 並行安全な操作
data = json.stringify()      # JSON文字列化
```

### **4.2 Rust風トレイトベース演算子（2025-08-10）**
```nyash
# AI大相談会で決定された最適設計
# 静的・動的ハイブリッドディスパッチによる高性能実現

# 整数演算
result = 100 - 25        # IntegerBox間演算 → IntegerBox
product = 6 * 7          # 高速静的ディスパッチ

# 文字列操作  
greeting = "Hello" + " World"    # 文字列結合
repeated = "Echo" * 3            # "EchoEchoEcho"

# 混合型フォールバック
message = "Answer: " + 42        # "Answer: 42"

# Boolean演算
boolSum = true + false           # 1 (IntegerBox)
```

### **4.3 変数宣言厳密化（2025-08-09）**
```nyash
# メモリ安全性・非同期安全性保証システム

static box Calculator {
    init { memory }  # 必須フィールド宣言
    
    calculate() {
        local temp       # 必須ローカル変数宣言
        temp = me.memory * 2
        return temp
    }
}
```

---

## ⚡ **5. 実装済みBox型ライブラリ**

### **5.1 基本型**
- `StringBox` - 文字列（split, find, replace, trim等）
- `IntegerBox` - 64bit整数
- `BoolBox` - 真偽値
- `VoidBox` - null/void値

### **5.2 コレクション**
- `ArrayBox` - 動的配列（push, pop, get, set, join等）
- `MapBox` - 連想配列・辞書

### **5.3 システム・I/O**
- `ConsoleBox` - コンソール入出力
- `DebugBox` - デバッグ支援・メモリ追跡
- `FileBox` - ファイルシステム操作

### **5.4 数学・時間**
- `MathBox` - 数学関数（sin, cos, log, sqrt等）
- `TimeBox` - 時刻操作・タイマー
- `RandomBox` - 乱数生成・選択・シャッフル

### **5.5 データ処理**
- `JSONBox` - JSON解析・生成（parse, stringify, get, set）
- `RegexBox` - 正規表現（test, find, replace, split）
- `BufferBox` - バイナリデータ処理
- `StreamBox` - ストリーム処理

### **5.6 ネットワーク・Web**
- `HttpClientBox` - HTTP通信
- `WebDisplayBox` - HTML表示（WASM）
- `WebConsoleBox` - ブラウザコンソール（WASM）
- `WebCanvasBox` - Canvas描画（WASM）

### **5.7 GUI・マルチメディア**
- `EguiBox` - デスクトップGUI（Windows/Linux）
- `SoundBox` - 音声再生

---

## 🎯 **6. パフォーマンス・デザイン原則**

### **6.1 メモリ安全性**
- Rust所有権システムによる完全なメモリ安全性
- Arc<Mutex>によるスレッドセーフな共有状態管理
- 自動参照カウント + 明示的デストラクタ（fini）

### **6.2 実行効率**
- 統一されたBox型システムによる最適化
- 静的・動的ハイブリッドディスパッチで高速演算
- パーサー無限ループ対策（--debug-fuel）

### **6.3 開発効率**
- 変数宣言厳密化による早期エラー検出
- 包括的デバッグ機能（DebugBox）
- 直感的な"Everything is Box"概念

---

## 📚 **7. 学習パス・ベストプラクティス**

### **7.1 初心者向け学習順序**
1. **基本概念**: Everything is Box哲学理解
2. **基本構文**: 変数宣言・制御構文・演算子
3. **Box定義**: 基本的なクラス作成
4. **Static Box Main**: アプリケーションエントリーポイント
5. **継承・インターフェース**: オブジェクト指向機能

### **7.2 推奨コーディングスタイル**
```nyash
# ✅ 推奨スタイル
static box Main {
    init { console, result }    # フィールド明示
    
    main() {
        me.console = new ConsoleBox()
        
        local data              # 変数事前宣言
        data = processInput()
        
        me.result = data        # 明確な代入
        return "Success"
    }
}
```

### **7.3 よくある間違いと対策**
```nyash
# ❌ よくある間違い
init { field1 field2 }      # カンマなし → CPU暴走
x = 42                      # 変数未宣言 → ランタイムエラー
while condition { }         # 非対応構文 → パーサーエラー

# ✅ 正しい書き方
init { field1, field2 }     # カンマ必須
local x = 42               # 事前宣言
loop(condition) { }        # 統一ループ構文
```

---

**🎉 Nyash 2025は、AI協働設計による最先端言語システムとして、シンプルさと強力さを完全に両立しました。**

*最終更新: 2025年8月10日 - Arc<Mutex> Revolution + AI大相談会成功記念*
