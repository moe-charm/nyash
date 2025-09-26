# Nyash 2モード設計：スクリプトモード vs アプリケーションモード

## 🎯 設計理念

Nyashは「明示性重視」と「使いやすさ」の両方を実現するため、**2つの実行モード**を永続的にサポートする。移行や非推奨化は行わず、それぞれのモードが異なる用途で輝く設計とする。

## 📊 モード比較

### スクリプトモード（Script Mode）
```nyash
// ファイル：hello.nyash
name = "Nyash"
count = 42
print("Hello, " + name + "!")
print("The answer is " + count)

// 関数も自由に定義
function greet(who) {
    print("Greetings, " + who)
}

greet("World")
```

### アプリケーションモード（Application Mode）
```nyash
// ファイル：app.nyash
static box Main {
    init { name, count }
    
    main() {
        me.name = "Nyash"
        me.count = 42
        me.greet()
    }
    
    greet() {
        print("Hello, " + me.name + "!")
        print("The answer is " + me.count)
    }
}
```

## 🔍 自動検出ルール

### 基本原則：「見た目で分かる」
1. **`static box`が1つでもある** → アプリケーションモード
2. **`static box`が1つもない** → スクリプトモード
3. **明示的な指定は不要**（ファイル内容で自動判定）

### 検出アルゴリズム
```rust
fn detect_mode(ast: &AST) -> ExecutionMode {
    // ASTを走査してstatic boxの存在をチェック
    for node in ast.walk() {
        match node {
            ASTNode::StaticBoxDeclaration { .. } => {
                return ExecutionMode::Application;
            }
            _ => continue,
        }
    }
    ExecutionMode::Script
}
```

## 🌟 各モードの特徴

### スクリプトモード

#### 利点
- **素早いプロトタイピング** - アイデアをすぐ試せる
- **学習曲線が緩やか** - 初心者に優しい
- **REPL完全互換** - コピペで動く
- **ワンライナー対応** - `nyash -e "print(2+2)"`

#### 変数スコープ
```nyash
// グローバルスコープに直接定義
x = 10
y = 20

// 関数内でもアクセス可能
function sum() {
    return x + y  // グローバル変数を参照
}

// ローカル変数
function calculate() {
    local temp = 100  // 関数ローカル
    return temp + x
}
```

#### エラーハンドリング
```nyash
// 未定義変数アクセス → 実行時エラー
print(undefined_var)  // Runtime Error: undefined variable 'undefined_var'

// 型エラーも実行時
x = "hello"
y = x + 10  // Runtime Error: cannot add String and Integer
```

### アプリケーションモード

#### 利点
- **明示的な変数管理** - どこで何が定義されているか明確
- **静的解析可能** - IDEサポートが充実
- **大規模開発対応** - チーム開発で威力を発揮
- **名前空間の分離** - static box単位でスコープ管理

#### 変数スコープ
```nyash
static box Calculator {
    init { x, y, result }  // 必ず宣言
    
    calculate() {
        // me.を通じてアクセス（明示的）
        me.result = me.x + me.y
        
        // ローカル変数
        local temp = me.result * 2
        return temp
    }
}

// ❌ エラー：グローバル変数は定義できない
global_var = 10  // Error: Global variables not allowed in application mode
```

#### コンパイル時チェック
```nyash
static box TypedCalc {
    init { value }
    
    setValue(v) {
        // 将来的に型推論や型チェックも可能
        me.value = v
    }
    
    calculate() {
        // 未定義フィールドアクセス → コンパイルエラー
        return me.undefined_field  // Error: 'undefined_field' not declared in init
    }
}
```

## 🚀 REPL統合

### REPLの動作
```bash
$ nyash
Nyash REPL v1.0 (Script Mode)
> x = 42
> print(x)
42
> function double(n) { return n * 2 }
> print(double(x))
84

# アプリケーションモードのコードも貼り付け可能
> static box Counter {
.     init { count }
.     increment() { me.count = me.count + 1 }
. }
> c = new Counter()
> c.increment()
> print(c.count)
1
```

### モード切り替え
```bash
# 通常起動（スクリプトモード）
$ nyash

# アプリケーションモードでREPL起動（将来実装）
$ nyash --app-mode
Nyash REPL v1.0 (Application Mode)
> x = 42  # Error: Global variables not allowed
> static box Main { ... }  # OK
```

## 📝 実装詳細

### パーサーの拡張
```rust
pub struct Parser {
    mode: ExecutionMode,
    // スクリプトモードでは緩い解析
    // アプリケーションモードでは厳格な解析
}

impl Parser {
    pub fn parse(&mut self, source: &str) -> Result<AST, ParseError> {
        let ast = self.parse_initial(source)?;
        self.mode = detect_mode(&ast);
        
        match self.mode {
            ExecutionMode::Script => self.validate_script_mode(&ast),
            ExecutionMode::Application => self.validate_app_mode(&ast),
        }
    }
}
```

### インタープリターの対応
```rust
impl Interpreter {
    pub fn execute(&mut self, ast: &AST, mode: ExecutionMode) -> Result<Value, RuntimeError> {
        match mode {
            ExecutionMode::Script => {
                // グローバル変数を許可
                self.allow_globals = true;
                self.execute_script(ast)
            }
            ExecutionMode::Application => {
                // グローバル変数を禁止
                self.allow_globals = false;
                self.execute_application(ast)
            }
        }
    }
}
```

## 🎨 ベストプラクティス

### いつスクリプトモードを使うか
1. **データ処理スクリプト**
   ```nyash
   data = readFile("input.csv")
   lines = data.split("\n")
   for line in lines {
       fields = line.split(",")
       print(fields[0] + ": " + fields[1])
   }
   ```

2. **設定ファイル**
   ```nyash
   # config.nyash
   server_host = "localhost"
   server_port = 8080
   debug_mode = true
   ```

3. **テストコード**
   ```nyash
   # test_math.nyash
   assert(2 + 2 == 4)
   assert(10 / 2 == 5)
   print("All tests passed!")
   ```

### いつアプリケーションモードを使うか
1. **Webアプリケーション**
   ```nyash
   static box WebServer {
       init { port, routes }
       
       start() {
           me.port = 8080
           me.routes = new MapBox()
           me.setupRoutes()
           me.listen()
       }
   }
   ```

2. **ゲーム開発**
   ```nyash
   static box Game {
       init { player, enemies, score }
       
       main() {
           me.initialize()
           loop(me.isRunning()) {
               me.update()
               me.render()
           }
       }
   }
   ```

3. **ライブラリ開発**
   ```nyash
   static box MathLib {
       init { precision }
       
       static factorial(n) {
           if n <= 1 { return 1 }
           return n * MathLib.factorial(n - 1)
       }
   }
   ```

## 🔮 将来の拡張

### 混在モード（検討中）
```nyash
#!script
// ファイルの最初の部分はスクリプトモード
config = loadConfig()
debug = true

#!application
// ここからアプリケーションモード
static box Main {
    init { config }
    
    main() {
        // スクリプト部分の変数を参照できる？
        me.config = ::config  // グローバルスコープ参照構文
    }
}
```

### プロジェクト設定
```toml
# nyash.toml
[project]
name = "my-app"
default_mode = "application"  # プロジェクト全体のデフォルト

[scripts]
# 特定のファイルはスクリプトモードで実行
mode = "script"
files = ["scripts/*.nyash", "tests/*.nyash"]
```

## 🌈 他言語との比較

### Python
- **類似点**: REPLとファイル実行で同じ動作
- **相違点**: Nyashはモードを明確に分離

### JavaScript/TypeScript
- **類似点**: strictモードの概念
- **相違点**: Nyashは見た目で判定（`"use strict"`不要）

### C#
- **優位性**: 
  - トップレベルステートメントより柔軟
  - 明示的なモード分離で混乱なし
  - REPLとの完全な統合

### Ruby
- **類似点**: スクリプトとクラスベースの両対応
- **相違点**: Nyashはより明確なモード分離

## 🎯 まとめ

Nyashの2モード設計は：
1. **移行不要** - 両モード永続サポート
2. **自動判定** - ファイル内容で自動切り替え
3. **REPL対応** - 同じルールで直感的
4. **いいとこ取り** - 用途に応じて最適なモードを選択

この設計により、Nyashは「**初心者に優しく、プロに強力**」な言語となる。