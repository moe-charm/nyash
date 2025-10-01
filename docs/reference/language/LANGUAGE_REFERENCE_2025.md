# 🚀 Nyash Language Reference 2025

**最終更新: 2025年9月18日 - Property System Revolution + Method-Level Exception Handling**

## 📖 概要

Nyashは「Everything is Box」哲学に基づく革新的プログラミング言語です。
Rust製インタープリターによる高性能実行と、直感的な構文により、学習しやすく実用的な言語として完成しました。

---

## 🔤 **1. 予約語・キーワード完全リスト**

### **Phase 15で確定した17個の予約語**
| 予約語 | 用途 | 例 |
|-------|------|---|
| `box` | クラス定義 | `box MyClass { }` |
| `new` | オブジェクト生成 | `new ConsoleBox()` |
| `me` | 自己参照（thisの代わり） | `me.field = value` |
| `local` | ローカル変数宣言 | `local x, y = 10` |
| `return` | 関数リターン | `return value` |
| `from` | デリゲーション・親メソッド呼び出し | `box Child from Parent` / `from Parent.method()` |
| `birth` | コンストラクタ（統一名） | `birth(param) { }` |
| `static` | 静的Box・関数定義 | `static box Main { }` |
| `if` | 条件分岐 | `if condition { }` |
| `else` | else節 | `else { }` |
| `loop` | ループ（唯一の形式） | `loop(condition) { }` |
| `continue` | ループ継続 | `continue` |
| `match` | パターンマッチング（構造/型/ガード） | `match value { "A" => 1, _ => 0 }` |
| `try` | 例外捕獲開始 | `try { }`（非推奨。postfix `catch/cleanup` を使用） |
| `interface` | インターフェース定義 | `interface Comparable { }` |
| `once` | **NEW** 遅延評価プロパティ | `once cache: CacheBox { build() }` |
| `birth_once` | **NEW** 即座評価プロパティ | `birth_once config: ConfigBox { load() }` |

### **その他の重要キーワード（予約語ではない）**
| キーワード | 用途 | 例 |
|-------|------|---|
| `override` | 明示的オーバーライド | `override speak() { }` |
| `break` | ループ脱出 | `break` |
| `catch` | 例外処理 | `catch (e) { }`（式/呼び出しの後置も可・Stage‑3） |
| `cleanup` | 最終処理（finally の後継） | `cleanup { }`（式/呼び出しの後置も可・Stage‑3） |
| `throw` | 例外発生 | `throw error` |
| `nowait` | 非同期実行 | `nowait future = task()` |
| `await` | 待機・結果取得 | `result = await future` |
| `include` | ファイル取り込み | `include "math.nyash"` |
| `print` | 出力（デバッグ用） | `print("Hello")` |
| `function`/`fn` | 関数定義 | `fn add(a,b) { }` |
| `init` | 初期化ブロック | `init { field1, field2 }` |
| `pack` | 旧コンストラクタ（互換性） | `pack(param) { }` |
| `outbox` | 所有権移転変数 | `outbox result = compute()` |
| `global` | グローバル変数 | `global CONFIG = "dev"` |
| `weak` | 弱参照修飾子 | `weak reference` |
| `using` | 名前空間インポート | `using namespace` |

### **演算子・論理**
| 演算子/キーワード | 用途 | 例 |
|-------|------|---|
| `not` / `!` | 論理否定 | `not condition` / `!condition` |
| `and` | 論理積 | `a and b` |
| `or` | 論理和 | `a or b` |
| `true`/`false` | 真偽値 | `flag = true` |
| `null` | null値 | `value = null` |


---

## 📝 **2. 文法・構文仕様**

### **2.1 Box定義文法**

#### **基本Box**
```nyash
box ClassName {
    # フィールド宣言（Phase 12.7形式）
    field1: TypeBox              # フィールド型アノテーション（P0では無視）
    field2: TypeBox
    field3                       # 型なしも可
    
    # コンストラクタ
    birth(param1, param2) {      # birth構文に統一
        me.field1 = param1
        me.field2 = param2
        me.field3 = defaultValue()
    }
    
    # メソッド
    methodName(arg1, arg2) {
        return me.field1 + arg1
    }
    
    # デストラクタ（fini）
    fini() {
        print("Cleanup: " + me.field1)
    }
}
```

#### **デリゲーションBox**
```nyash
box Child from Parent interface Comparable {
    childField: TypeBox          # 追加フィールド
    
    birth(parentParam, childParam) {  # birth構文に統一
        from Parent.birth(parentParam)  # 親コンストラクタ明示呼び出し
        me.childField = childParam
    }
    
    # メソッドオーバーライド
    override process(data) {     # overrideキーワード必須
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
    console: ConsoleBox
    result: IntegerBox
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("🎉 Everything is Box!")
        return "Success"
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

#### **設計方針（var/let について）**
- Nyash は `var`/`let` を導入しません。ローカル変数は常に `local` で明示宣言します。
- 目的: 代入点と定義点を一致させ、Loop‑Form/SSA と解析（Known/Union）を簡潔に保つためです。
- 補足: 行頭 `@name[:T] = expr` は標準ランナーで `local name[:T] = expr` に自動展開されます（構文糖衣、言語意味は不変）。

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
    if skipCondition {
        continue  # Phase 12.7で追加
    }
}

# ❌ 採用しない構文（設計方針）
while condition { }      # 先頭条件は `loop(condition){}` へ統一
do { body } while(cond)  # do‑while は不採用。`repeat/ until` 糖衣で表現し、先頭条件に正規化
loop() { }               # 無条件ループは `loop(true){}` を意図明確に書く

> 設計メモ: Nyashは「単一入口・先頭条件」の制御フロー規律を重視するため、do‑whileは採用しません。必ず実行の表現は `loop(1)` ラッパーや `repeat/until` 糖衣からゼロコストで正規化します。
```

#### **Match式（Phase 12.7で追加）**
```nyash
# パターンマッチング風の分岐
local result = match value {
    "A" => 100,
    "B" => 200,
    "C" => 300,
    _ => 0  # _はデフォルトパターン
}

# 文の形式も可
match status {
    "error" => {
        print("Error occurred")
        return null
    },
    "success" => {
        print("All good")
    },
    _ => {
        print("Unknown status")
    }
}
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

#### **特殊演算子（Phase 12.7実装済み）**
```nyash
# ? 演算子 - Result伝播
local data = readFile(path)?  # エラーなら早期return

# ラムダ式
local add = fn(x, y) { return x + y }
local double = fn(x) { x * 2 }  # 単一式なら省略可

# await式  

### **2.5 プロパティ（統一メンバ — Phase 15、既定ON: NYASH_ENABLE_UNIFIED_MEMBERS）**

概要
- Box 内のメンバを「格納/計算/一度だけ（遅延 or 生成時）」で統一的に扱います。JSON v0/MIR は変更せず、ローワで既存の slot/method に展開します。
- 環境変数 `NYASH_ENABLE_UNIFIED_MEMBERS` で制御（Phase 15 では既定ON、`0/false/off` で無効化）。

分類と構文（header‑first）
- stored（格納・読み書き可）
  - `name: Type` または `name: Type = expr`（初期値は生成時に一度だけ評価）
- computed（計算・読み専用）
  - `name: Type { /* body */ }`（読むたびに計算。代入不可）
- once（初回アクセス時に一度だけ計算 → 以後は保存値）
  - `once name: Type { /* body */ }` または `once name: Type => expr`
- birth_once（生成時に一度だけ計算 → 以後は保存値）
  - `birth_once name: Type { /* body */ }` または `birth_once name: Type => expr`

nyashモード（block‑first、オプション）
- `{"..."}` の直後に `as` を置く統一構文を、Box メンバ領域で受理
  - computed: `{ body } as name: Type`
  - once: `{ body } as once name: Type`
  - birth_once: `{ body } as birth_once name: Type`
  - stored は block‑first では宣言しない（header‑first を使用）

共通ルール
- 読みは全て `obj.name` で同一表記。違いは書き込み可否と計算タイミングのみ。
- 代入:
  - stored のみ許可（`obj.name = v`）。
  - computed/once/birth_once は代入不可（エラー）。setter を定義した場合のみ糖衣で許可（`obj.name = v` → `__set_name(v)`）。

例
```nyash
box MyBox {
  name: StringBox                 # stored
  size: IntegerBox { me.items.len() }   # computed
  once cache: CacheBox { buildCache() } # once
  birth_once token: StringBox { readEnv("TOKEN") } # eager once
}
```

例外・ハンドラ（Stage‑3, `NYASH_PARSER_STAGE3=1`）
- stored 以外のブロック末尾に `catch`/`cleanup` を付与可能（header‑first / block‑first 両対応）。
  - computed: `name: T { body } catch(e) { ... } cleanup { ... }`
  - once: `once name: T { body } catch { ... } cleanup { ... }`
  - birth_once: `birth_once name: T { body } catch { ... } cleanup { ... }`
- once の例外ポリシー（catch が無い場合）: 例外をその場で伝播し、プロパティは poison 状態となり以後の読みでも同じ例外を再スロー（再実行しない）。
- birth_once の実行順: ユーザ `birth` 本体の前、宣言順で実行。未捕捉例外はコンストラクタ失敗として伝播。自己参照はエラー。相互依存の循環は検出してエラー。

ローワ（下ろし先の概要）
- stored → slot（初期化子は生成時に一度だけ評価）。
- computed → `__get_name():T` メソッドを合成し、`obj.name` 読みを呼び出しに解決。
- once → `__name: Option<T>` + `__get_name()`（初回のみ評価・保存）。未捕捉例外で poison し、以後は即 rethrow。
- birth_once → `__name: T` を用意し、`birth` 直前に宣言順で初期化コードを挿入。未捕捉例外は `new` 失敗。

注意
- JSON v0 は unchanged。Unified Members はパーサ/ローワの砂糖として振る舞います。
- stored の初期化子は式のみ（`catch/cleanup` は不可）。
local result = await asyncTask()
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
    public name: StringBox
    public email: StringBox
    private age: IntegerBox
    
    birth(personName, personAge) {  # birth構文に統一
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
    public name: StringBox
    public species: StringBox
    
    birth(animalName, animalSpecies) {
        me.name = animalName
        me.species = animalSpecies
    }
    
    speak() {
        return me.name + " makes a sound"
    }
}

# デリゲーション
box Dog from Animal {
    breed: StringBox  # 追加フィールド
    
    birth(dogName, dogBreed) {
        from Animal.birth(dogName, "Canine")  # 親コンストラクタ呼び出し
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
    PI: FloatBox
    E: FloatBox
    
    # 注意: static初期化ブロックは未実装
    # 初期化はメソッド内で行う
    
    add(a, b) {
        return a + b
    }
    
    circleArea(radius) {
        # 初回アクセスで初期化パターン
        if me.PI == null {
            me.PI = 3.14159265
        }
        return me.PI * radius * radius
    }
}

# 使用法
area = MathUtils.circleArea(5)
sum = MathUtils.add(10, 20)
```

#### **アプリケーションエントリーポイント（Strict）**
既定のエントリは「`Main.main` のみ」です。Hakorune では入門者向けに `flow Main` を推奨します。静的状態が必要な場合のみ `static box Main` を採用してください。

詳細は `docs/reference/language/entrypoints.md` を参照してください。

```nyash
# 🎯 推奨: Static Box Main パターン
static box Main {
    console: ConsoleBox
    result: IntegerBox
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("🚀 Starting application...")
        
        # アプリケーションロジック
        me.result = processData()
        
        return "Application completed successfully"
    }
}
```

トップレベル `main()` を用いる場合（既定で許可）:
```nyash
main() {
  println("Hello")
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
    private memory: ArrayBox  # 必須フィールド宣言
    
    calculate() {
        local temp       # 必須ローカル変数宣言
        temp = me.memory * 2
        return temp
    }
}
```

---

## 🚀 **4.4 Phase 12.7実装済み機能**

### **Peek式 - パターンマッチング風分岐**
```nyash
# 式として使用（値を返す）
local grade = match score {
    100 => "Perfect",
    90 => "Excellent", 
    80 => "Good",
    _ => "Needs improvement"
}

# 文として使用（アクション実行）
match command {
    "save" => {
        saveFile()
        print("Saved!")
    },
    "quit" => {
        cleanup()
        return
    },
    _ => print("Unknown command")
}
```

### **Continue文**
```nyash
loop(i < 100) {
    if i % 2 == 0 {
        continue  # 偶数をスキップ
    }
    process(i)
}
```

### **フィールド型アノテーション**
```nyash
box Person {
    name: StringBox      # 型情報を明記（P0では無視）
    age: IntegerBox
    email               # 型なしも可
}
```

### **?演算子（Result伝播）**
```nyash
# ResultBoxのエラーを自動的に早期return
local data = readFile("config.json")?
local parsed = parseJSON(data)?
return parsed.get("version")
```

### **Lambda式**
```nyash
# 基本形
local add = fn(x, y) { return x + y }

# 単一式の場合（returnは省略可）
local double = fn(x) { x * 2 }

# 高階関数での使用
array.map(fn(x) { x * x })
```

---

## ⚡ **5. 実装済みBox型ライブラリ**

### **5.1 基本型**
- `StringBox` - 文字列（split, find, replace, trim等）
- `IntegerBox` - 64bit整数
- `FloatBox` - 64bit浮動小数点数
- `BoolBox` - 真偽値
- `NullBox` - null値
- `VoidBox` - void値

### **5.2 コレクション**
- `ArrayBox` - 動的配列（push, pop, get, set, join等）
- `MapBox` - 連想配列・辞書

### **5.3 システム・I/O**
- `ConsoleBox` - コンソール入出力
- `DebugBox` - デバッグ支援・メモリ追跡
- `FileBox` - ファイルシステム操作（プラグイン）

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
- `HttpClientBox` - HTTP通信（プラグイン）
- `WebDisplayBox` - HTML表示（WASM）
- `WebConsoleBox` - ブラウザコンソール（WASM）
- `WebCanvasBox` - Canvas描画（WASM）

### **5.7 GUI・マルチメディア**
- `EguiBox` - デスクトップGUI（Windows/Linux、プラグイン）
- `SoundBox` - 音声再生

### **5.8 特殊用途**
- `FutureBox` - 非同期処理結果
- `ResultBox` - エラー処理（Ok/Err）
- `TokenBox` - キャンセルトークン
- `FunctionBox` - 第一級関数
- `P2PBox` - P2P通信（プラグイン）

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
    public console: ConsoleBox    # 公開フィールド明示
    public result: IntegerBox
    
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
public { field1 field2 }    # 旧構文 → 使用不可
x = 42                      # 変数未宣言 → ランタイムエラー
while condition { }         # 非対応構文 → パーサーエラー
this.field                  # thisは使用不可 → me.fieldを使用

# ✅ 正しい書き方（Phase 12.7後）
field1: TypeBox             # フィールド宣言（型は省略可）
field2                      # 型なしフィールド
local x = 42               # 事前宣言必須
loop(condition) { }        # 統一ループ構文
me.field                   # self参照はmeのみ
```

---

## 📌 **8. 糖衣構文（Phase 12.7-B）**

### 実装済み（ゲート: `NYASH_SYNTAX_SUGAR_LEVEL=basic|full`）
```nyash
# パイプライン
result = data |> normalize() |> transform() |> process

# セーフアクセス + デフォルト
name = user?.profile?.name ?? "guest"

# 複合代入
x += 1; y *= 2

# 範囲（内部的には Range(a,b)）
loop(i in 1 .. 5) { /* ... */ }
```

### 拡張（段階適用予定・設計済み）
```nyash
let {x, y} = point
let [first, second, ...rest] = array
```

---

**🎉 Nyash 2025は、AI協働設計による最先端言語システムとして、シンプルさと強力さを完全に両立しました。**

*最終更新: 2025年9月4日 - Phase 12.7実装済み機能の正確な反映*
### 2.x 例外・エラーハンドリング（postfix / cleanup）

方針
- try は非推奨。postfix `catch` と `cleanup` を用いる。
- `catch` は直前の式/呼び出しで発生した例外を処理。
- `cleanup` は常に実行（finally の後継）。

例（式レベルの postfix）
```
do_work() catch(Error e) { env.console.log(e) }
open(path) cleanup { env.console.log("close") }
connect(url)
  catch(NetworkError e) { env.console.warn(e) }
  cleanup { env.console.log("done") }
```

注: Phase 1 は正規化（ゲート `NYASH_CATCH_NEW=1`）で legacy TryCatch へ展開。Phase 2 でパーサが直接受理。
