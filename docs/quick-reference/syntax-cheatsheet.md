# 🚀 Nyash構文早見表（Syntax Cheatsheet）

## 📝 基本構文

### 変数宣言
```nyash
local x                    # 変数宣言
local x = 42              # 初期化付き宣言
local a, b, c             # 複数宣言
local x = 10, y = 20, z   # 混合初期化
```

注意: Nyash は `var`/`let` を採用していません。常に `local` で明示宣言してください（未宣言名への代入はエラー）。
補足: 行頭 `@name[:T] = expr` は標準ランナーで `local name[:T] = expr` に自動展開されます（意味は不変）。

### Box定義（クラス）
```nyash
box ClassName {
    # フィールド宣言（Phase 12.7形式）
    field1: TypeBox           # デフォルト非公開
    public field2: TypeBox    # 公開フィールド
    private field3: TypeBox   # 明示的非公開
    
    birth(args) {            # コンストラクタ（birth統一）
        me.field1 = args
    }
}
```

### エントリーポイント（優先順）

Nyash はエントリを以下の順で解決します。

1) `Main.main` があれば優先
2) なければトップレベル `main()`

両方ある場合は `Main.main` が使われます。トップレベル `main` は既定で許可されています（無効化したい場合は `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=0`）。

### Static Box（エントリーポイント）
```nyash
static box Main {
    console: ConsoleBox
    
    main() {
        me.console = new ConsoleBox()
        me.console.log("Hello Nyash!")
        return 0
    }
}
```

### トップレベル main（既定で許可）
```nyash
main() {
  println("Hello Nyash!")
  return 0
}
```

### プロパティ（stored / computed / once / birth_once）
```nyash
box MyBox {
  # stored（格納・読み書き可）
  name: StringBox
  id: IntegerBox = 42   # 初期値は生成時に一度だけ評価

  # computed（計算・読み専用）
  greeting: StringBox { return "Hello, " + me.name }

  # once（初回アクセス時に一度だけ計算→以降は保存値）
  once cache: ResultBox { return heavyWork() }

  # birth_once（生成時に一度だけ計算→以降は保存値）
  birth_once token: StringBox { return readEnv("TOKEN") }
}

# 読みは共通、書きは stored のみ可能
local b = new MyBox()
print(b.greeting)   # OK（computed）
b.name = "A"        # OK（stored）
b.greeting = "x"    # エラー（computed には代入不可）
```

## 🔄 制御構文

### 条件分岐
```nyash
if condition {
    # 処理
} else if condition2 {
    # 処理2
} else {
    # else処理
}
```

### ループ（統一構文）
```nyash
# ✅ 唯一の正しい形式
loop(condition) {
    if exitCondition {
        break
    }
}

# ❌ 使えない構文
while condition { }  # エラー！
loop() { }          # エラー！
```

## 🌟 デリゲーション（継承）

### 基本デリゲーション
```nyash
box Child from Parent {
    birth(args) {
        from Parent.birth(args)  # 親のbirth呼び出し
    }
    
    method() {                  # メソッド定義
        from Parent.method()    # 親メソッド呼び出し
    }
}
```

### 多重デリゲーション
```nyash
box Multi from StringBox, IntegerBox {
    # 複数の親から機能を継承
}
```

## ⚡ 演算子

### 算術演算子
```nyash
a + b    # 加算
a - b    # 減算
a * b    # 乗算
a / b    # 除算（ゼロ除算チェック済）
a % b    # 剰余
```

### 論理演算子
```nyash
not condition    # NOT（推奨）
a and b         # AND（推奨）
a or b          # OR（推奨）

# シンボル版（互換性のため存在するが非推奨）
!condition      # NOT
a && b          # AND
a || b          # OR
```

### 比較演算子
```nyash
a == b    # 等しい
a != b    # 等しくない
a < b     # より小さい
a > b     # より大きい
a <= b    # 以下
a >= b    # 以上
```

## 📦 オブジェクト生成

### new構文
```nyash
# 基本形
obj = new ClassName()
obj = new ClassName(arg1, arg2)

# 組み込みBox
console = new ConsoleBox()
array = new ArrayBox()
map = new MapBox()
```

### 特殊なコンストラクタ優先順位
```nyash
box Life {
    birth(name) { }     # 最優先
    pack(args) { }      # ビルトインBox継承用
    init(args) { }      # 通常コンストラクタ
    Life(args) { }      # Box名形式（非推奨）
}

# birthが定義されていればbirthが呼ばれる
obj = new Life("Alice")
```

## 🚨 よくある間違い

### ❌ カンマ忘れ（CPU暴走の原因！）
```nyash
# ❌ 間違い
init { field1 field2 }    # カンマなし→CPU暴走！

# ✅ 正しい
init { field1, field2 }   # カンマ必須
```

### ❌ 未宣言変数
```nyash
# ❌ 間違い
x = 42    # Runtime Error: 未宣言変数

# ✅ 正しい
local x
x = 42
```

### ❌ 計算プロパティへの代入
```nyash
box B {
  value: IntegerBox { return 1 }  # computed
}

local b = new B()
b.value = 2   # ❌ エラー: 計算プロパティには代入できません（setter を定義するか stored にしてください）
```

### ❌ 削除された構文
```nyash
# ❌ 使えない
while condition { }       # whileは削除済み
super.method()           # superは使えない

# ✅ 代わりに
loop(condition) { }      # loop構文を使う
from Parent.method()     # fromで親を呼ぶ
```

## 🎯 実用例

### Hello World
```nyash
static box Main {
    main() {
        print("Hello, Nyash!")
        return 0
    }
}
```

### 配列操作
```nyash
local array
array = new ArrayBox()
array.push(10)
array.push(20)
array.push(30)

local sum = 0
local i = 0
loop(i < array.length()) {
    sum = sum + array.get(i)
    i = i + 1
}
print("Sum: " + sum)  # Sum: 60
```

### エラーハンドリング
```nyash
try {
    local result = riskyOperation()
} catch (error) {
    print("Error: " + error.message)
} cleanup {
    cleanup()
}
```

### 非同期処理
```nyash
nowait future = longTask()    # 非同期実行
# 他の処理...
local result = await future   # 結果待機
```

## 📚 組み込みBox一覧（抜粋）

| Box名 | 用途 | 主要メソッド |
|-------|------|-------------|
| StringBox | 文字列 | split(), find(), replace(), trim() |
| IntegerBox | 整数 | to_string_box() |
| ArrayBox | 配列 | push(), pop(), get(), set(), length() |
| MapBox | 辞書 | set(), get(), has(), keys() |
| ConsoleBox | コンソール | log(), error(), read() |
| MathBox | 数学 | sin(), cos(), sqrt(), random() |
| FileBox | ファイル | read(), write(), exists() |
| JSONBox | JSON | parse(), stringify(), get(), set() |

---

**Tips**: 
- すべての値はBox（Everything is Box）
- 変数は必ず宣言してから使う
- ループは`loop(condition)`のみ
- 親メソッドは`from Parent.method()`で呼ぶ
- カンマ忘れに注意！
### ビット演算子（整数限定）
```nyash
a & b     # ビットAND
a | b     # ビットOR
a ^ b     # ビットXOR
a << n    # 左シフト（n は 0..63 にマスク）
a >> n    # 右シフト（現在は論理シフト相当の実装）
```

注意: 旧来の `>>`（ARROW 演算子）は廃止されました。パイプラインは `|>` を使用してください。
