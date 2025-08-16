# Nyash言語コアコンセプト AI向け速習ガイド

このドキュメントは、AIアシスタントがNyashプログラミング言語を迅速に理解するために、そのコアコンセプトを凝縮して提供します。

## 1. 基本哲学: Everything is a Box (すべてはBoxである)

- Nyashの基本原則は「すべてがBoxである」ということです。
- 単純な整数から複雑な構造体まで、すべてのデータ型は「Box」オブジェクトの一種です。これにより、純粋で一貫性のあるオブジェクトベースのシステムが実現されています。

### 🌟 **革命的改善: 自動リテラル変換（Phase 9.75h完了）**

Nyashでは、Everything is Box哲学を維持しながら、使いやすさを大幅に向上させる自動リテラル変換機能を提供します：

```nyash
// 🎉 新しい書き方 - 自動変換で超使いやすい！
local text = "Hello"       // "Hello" → StringBox::new("Hello") に自動変換
local name = "Alice"       // "Alice" → StringBox::new("Alice") に自動変換  
local age = 30             // 30 → IntegerBox::new(30) に自動変換
local active = true        // true → BoolBox::new(true) に自動変換
local pi = 3.14159         // 3.14159 → FloatBox::new(3.14159) に自動変換

// ❌ 古い書き方（まだサポート）
local oldText = new StringBox("Hello")
local oldAge = new IntegerBox(30)

// ✅ Everything is Box哲学 + 書きやすさ革命達成！
```

**重要**: この自動変換はパーサーレベルで行われるため、実行時オーバーヘッドはありません。すべてが内部的にBoxとして処理されます。

## 2. オブジェクトモデルとデリゲーション (Nyash独自の方式)

Nyashは古典的な継承ではなく、デリゲーション（委譲）モデルを使用します。これは非常に重要な違いです。

- **Boxの定義:**
  ```nyash
  box MyBox {
      // フィールドはinitやpackのようなコンストラクタ内で宣言される
      // メソッドはこの場所に定義される
  }
  ```

- **コンストラクタ** (優先順位: birth > pack > init > Box名形式)
  - **`birth` (推奨・統一):** 「Boxに生命を与える」直感的コンストラクタ。Everything is Box哲学を体現する最新の統一構文。
    ```nyash
    box Life {
        init { name, energy }  // フィールド宣言
        
        birth(lifeName) {  // ← 「生命を与える」哲学的コンストラクタ
            me.name = lifeName
            me.energy = 100
            print("🌟 " + lifeName + " が誕生しました！")
        }
    }
    local alice = new Life("Alice")  // birthが呼び出される
    ```
  - **`init` (基本):** 従来のユーザー定義Boxのコンストラクタ。フィールド宣言と基本的な初期化。
    ```nyash
    box User {
        init { name, email }  // フィールド宣言のみ
        // new時に直接フィールドに値が設定される
    }
    local user = new User("Alice", "alice@example.com")  // initが呼び出される
    ```
  - **`pack` (ビルトインBox継承専用):** ビルトインBox（P2PBox、MathBox等）を継承する際の特別なコンストラクタ。ユーザー定義Boxでは使用禁止。
    ```nyash
    box ChatNode from P2PBox {
        init { chatHistory }  // 追加フィールド宣言
        
        pack(nodeId, world) {
            from P2PBox.pack(nodeId, world)  // ビルトインBoxの初期化
            me.chatHistory = new ArrayBox()   // 自分の追加フィールド初期化
        }
    }
    local node = new ChatNode("node1", "tcp")  // packが呼び出される
    ```

- **デリゲーション (`from`キーワード):** あるオブジェクトが、メソッド呼び出しやフィールドアクセスを別のオブジェクトに委譲できます。
  ```nyash
  // AdminがUserにデリゲートする
  box Admin from User {
      init { permissions } // Adminの追加フィールド
  }
  ```

- **明示的なオーバーライド (`override`キーワード):** 子Boxが親Boxのメソッドを再実装する場合、必ず`override`でマークしなければなりません。
  ```nyash
  box AdminUser from User {
      init { permissions }  // 追加フィールド
      
      birth(name, email, permissions) {  // birth構文使用
          from User.birth(name, email)     // 親のbirthを呼び出し
          me.permissions = permissions     // 追加フィールド初期化
          print("🎉 管理者 " + name + " が誕生しました")
      }
      
      override greet() {
          from User.greet()                // 親の処理を実行
          print("(Administrator)")         // 追加の処理
      }
  }
  ```

- **デリゲートされたメソッドの呼び出し (`from`キーワード):** オーバーライドしたメソッド内から親の実装を呼び出すには、`from Parent.method()`を使用します。
  ```nyash
  box ScientificCalc from MathBox {
      init { history }
      
      pack() {
          from MathBox.pack()              // ビルトインBoxの初期化
          me.history = new ArrayBox()      // 自分の追加フィールド
      }
      
      override sin(x) {
          local result = from MathBox.sin(x)  // 親のメソッド呼び出し
          me.history.push("sin(" + x + ") = " + result)
          return result
      }
  }
  ```

- **ファイナライズ (`fini`キーワード):**
  - `fini()`は「論理的な解放フック」として機能する特別なメソッドです。
  - インスタンスに対して呼び出されると、そのインスタンスがもはや使用されるべきではないことを示します。
  - クリーンアップ処理を実行し、所有するすべてのフィールドに対して再帰的に`fini()`を呼び出します。
  - ファイナライズされたオブジェクトを使用しようとすると（`fini`の再呼び出しを除く）、実行時エラーが発生します。
  ```nyash
  box ManagedResource {
      init { handle }
      fini() {
          // ハンドルを解放したり、他のクリーンアップ処理を実行
          me.console.log("リソースをファイナライズしました。")
      }
  }
  ```

## 3. 標準ライブラリアクセス (using & namespace) 🎉 **Phase 9.75e完了**

Nyashは組み込み標準ライブラリ`nyashstd`と、using文による名前空間インポートをサポートします。

### **🌟 using nyashstd - 完全実装済み**

**基本構文:**
```nyash
using nyashstd

// ✅ 実際に動作確認済みの標準ライブラリ機能
local result = string.create("Hello World")  // → "Hello World"
local upper = string.upper(result)           // → "HELLO WORLD"  
local number = integer.create(42)            // → 42
local flag = bool.create(true)               // → true
local arr = array.create()                   // → []
console.log("✅ using nyashstd test completed!")  // ✅ 出力成功
```

### **🎯 実装済み名前空間モジュール:**

- **string.*** - 文字列操作
  ```nyash
  string.create("text")     // 文字列Box作成
  string.upper("hello")     // "HELLO" - 大文字変換
  string.lower("WORLD")     // "world" - 小文字変換
  ```

- **integer.*** - 整数操作
  ```nyash
  integer.create(42)        // 整数Box作成
  // 将来: integer.add(), integer.multiply() 等
  ```

- **bool.*** - 真偽値操作
  ```nyash
  bool.create(true)         // 真偽値Box作成
  // 将来: bool.and(), bool.or(), bool.not() 等
  ```

- **array.*** - 配列操作
  ```nyash
  array.create()            // 空配列Box作成
  // 将来: array.push(), array.length() 等
  ```

- **console.*** - コンソール出力
  ```nyash
  console.log("message")    // コンソール出力
  // 将来: console.error(), console.debug() 等
  ```

### **⚡ 自動リテラル変換との連携**

using nyashstdと自動リテラル変換を組み合わせると、極めてシンプルなコードが書けます：

```nyash
using nyashstd

// 🌟 革命的シンプルさ！
local name = "Nyash"              // 自動StringBox変換
local year = 2025                 // 自動IntegerBox変換
local upper = string.upper(name)  // nyashstd + 自動変換連携
console.log("🚀 " + upper + " " + year.toString() + " Ready!")
// 出力: "🚀 NYASH 2025 Ready!" ✅
```

### **📋 名前空間の特徴:**
- **✅ Phase 9.75e完了**: `nyashstd`完全実装・動作確認済み
- **IDE補完対応**: `string.`で標準機能の補完が可能（将来）
- **明示的インポート**: プレリュード（自動インポート）よりIDE補完に適した設計
- **拡張可能**: 将来的にユーザー定義名前空間もサポート予定

## 4. 構文クイックリファレンス

### **🎯 現代的Nyash構文（Phase 9.75h対応）**

- **厳格な変数宣言:** すべての変数は使用前に宣言が必要です。
  ```nyash
  // 🌟 自動リテラル変換 + 宣言
  local text = "Hello"        // 自動StringBox変換 + ローカル宣言
  local count = 42            // 自動IntegerBox変換 + ローカル宣言
  local flag = true           // 自動BoolBox変換 + ローカル宣言
  
  // Box内フィールドアクセス
  me.field = "value"          // 現在のBoxインスタンスのフィールド
  
  // 静的関数内での所有権移転
  outbox product = new Item() // 所有権が呼び出し元に移転
  ```

- **統一されたループ:** ループ構文は一種類のみです。
  ```nyash
  loop(condition) {
      // 条件がtrueの間ループ
  }
  ```

- **プログラムのエントリーポイント:** 実行は`static box Main`の`main`メソッドから開始されます。
  ```nyash
  using nyashstd  // 標準ライブラリインポート
  
  static box Main {
      init { console }  // フィールド宣言
      
      main() {
          me.console = new ConsoleBox()
          
          // 🌟 現代的Nyash書法
          local message = "Hello Nyash 2025!"  // 自動変換
          console.log(message)                 // 標準ライブラリ使用
      }
  }
  ```

### **🎉 実用的なコード例（最新機能活用）**

```nyash
using nyashstd

static box Main {
    init { console }
    
    main() {
        me.console = new ConsoleBox()
        
        // 🌟 すべて自動変換 + 標準ライブラリ
        local name = "Nyash"               // 自動StringBox
        local version = 2025               // 自動IntegerBox  
        local isStable = true              // 自動BoolBox
        local pi = 3.14159                 // 自動FloatBox
        
        // string標準ライブラリ活用
        local upper = string.upper(name)
        
        // コンソール出力
        console.log("🚀 " + upper + " " + version.toString() + " Ready!")
        console.log("円周率: " + pi.toString())
        console.log("安定版: " + isStable.toString())
    }
}
```

## 4. 演算子

- **論理演算子:** `and`, `or`, `not`
- **算術演算子:** `+`, `-`, `*`, `/` (ゼロ除算をハンドルします)
- **比較演算子:** `==`, `!=`, `<`, `>`, `<=`, `>=`

## 5. 主要なビルトインBox（実装済み）

- **基本型**
  - **`StringBox`**: 文字列操作
  - **`IntegerBox`**: 整数値
  - **`BoolBox`**: 真偽値
  - **`NullBox`**: null値

- **計算・データ処理系**
  - **`MathBox`**: 数学関数（sin, cos, sqrt等）
  - **`RandomBox`**: 乱数生成
  - **`TimeBox`**: 時間・日付操作
  - **`ArrayBox`**: 配列操作
  - **`MapBox`**: 連想配列（辞書）操作

- **I/O・デバッグ系**
  - **`ConsoleBox`**: 基本的なI/O (例: `log()`)
  - **`DebugBox`**: イントロスペクション/デバッグ (例: `memoryReport()`)
  - **`SoundBox`**: 音声出力

- **GUI・Web系（環境依存）**
  - **`EguiBox`**: GUI（メインスレッド制約など）
  - **`WebDisplayBox`**: Web表示
  - **`WebConsoleBox`**: Webコンソール
  - **`WebCanvasBox`**: Web Canvas操作

- **通信系**
  - **`P2PBox`**: P2P通信
  - **`SimpleIntentBox`**: 簡単なインテント通信

**注意**: using nyashstdで標準ライブラリ経由でのアクセスも可能です。

## 6. データ構造 (Data Structures)

現行バージョンでは配列/マップのリテラル構文（`[]`, `{}`）は未実装です（将来計画）。
利用時はビルトインBoxを用います。

- **配列 (ArrayBox):**
  ```nyash
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  a.push(3)
  // 取得
  local first = a.get(0)
  // サイズ（実装に依存）
  // 例: a.length() または length はAPIに従って利用
  ```

- **マップ (MapBox):**
  ```nyash
  local m = new MapBox()
  m.set("name", "Alice")
  m.set("age", 30)
  local name = m.get("name")
  ```

## 7. エラーハンドリング (Error Handling)

実行時エラーによってプログラムがクラッシュするのを防ぐために、`try...catch`ブロックを使用します。

- **構文:**
  ```nyash
  try {
      // エラーが発生する可能性のあるコード
      local result = 10 / 0
  } catch {
      // エラーが発生した場合に実行されるコード
      print("エラーが発生しましたが、プログラムは続行します。")
  }
  ```

- **finally/throw の補足:**
  ```nyash
  try {
      if (x < 0) { throw "negative" }
  } catch {
      print("error")
  } finally {
      print("always")
  }
  ```

## 8. メモリ管理と弱参照 (Memory Management & Weak References)

- **`weak` キーワード:** `init`ブロック内でフィールドを`weak`として宣言することで、弱参照を作成します。これは主に循環参照を防ぐために使用されます。
  - 弱参照はオブジェクトの所有権を持ちません。
  - 参照先のオブジェクトが解放されると、弱参照フィールドは自動的に`null`になります。
  ```nyash
  box Node {
      init { id, weak next } // 'next'は弱参照
  }

  local node1 = new Node("A", null)
  local node2 = new Node("B", node1) // node2はnode1への弱参照を持つ
  node1.next = node2 // node1はnode2への強参照を持つ
  // この場合、node1とnode2が互いを所有しないため、安全に解放される
  ```

不変条件（重要）
- weak フィールドに対して `fini()` を直接呼ぶことはできません（エラーになります）。
- インスタンスで `fini()` 呼び出し後は、そのオブジェクトの使用はすべて禁止です（アクセス時にエラー）。
- `fini()` のカスケードは init 宣言順の「逆順」で実行され、weak フィールドはスキップされます。

## 9. 非同期処理 (Asynchronous Processing)

- **`nowait` 文:** 式を別スレッドで非同期実行し、その結果を表す `FutureBox` を変数に格納します。
  - 構文: `nowait future = expression`
  - 挙動: 内部でスレッドを生成し、完了時に `future.set_result(...)` が呼ばれます。

- **`await` 式:** `FutureBox` の完了を待機し、結果を取り出します。
  - 構文: `result = await future`
  - 実装: `FutureBox.wait_and_get()` を通じて結果を返します。

使用例:
```nyash
// 非同期に3つの処理を開始
nowait f1 = heavyComputation(5000)
nowait f2 = heavyComputation(3000)
nowait f3 = heavyComputation(4000)

// 結果を待機
r1 = await f1
r2 = await f2
r3 = await f3
```

備考（現実装の特性）
- 実装はスレッドベースの簡易非同期（イベントループ無し）。
- FutureBox は簡易 busy-wait を用います（将来 condvar 等で改善予定）。
- 例外は `ErrorBox` として `FutureBox` に格納されます（`await` 側で結果を取り出す設計）。

## 10. 静的Box/関数と所有権移転（outbox）

- **静的エントリーポイント:**
  ```nyash
  static box Main {
      main() {
          print("Hello Nyash")
      }
  }
  ```

- **静的関数の定義/呼び出し:**
  ```nyash
  static function Math.min(a, b) {
      if (a < b) { return a } else { return b }
  }
  local m = Math.min(1, 2)
  ```

- **所有権移転（outbox, static関数内のみ）:**
  ```nyash
  static function Factory.create() {
      outbox product
      product = new Item()
      return product
  }
  ```

## 11. 実行バックエンド選択 (2025-08-14追加)

Nyashは3つの実行方式をサポート。用途に応じて選択可能：

```bash
# インタープリター実行（開発・デバッグ重視）
nyash program.nyash

# VM実行（高速実行・本番環境）
nyash --backend vm program.nyash

# WASM生成（Web配布・最高性能）
nyash --compile-wasm program.nyash

# ベンチマーク実行（性能比較）
nyash --benchmark --iterations 100
```

**性能比較（実行速度）:**
- **WASM**: 13.5倍高速化（真の実行性能）
- **VM**: 20.4倍高速化（高速実行・本番環境）
- **Interpreter**: ベースライン（開発・デバッグ重視）

**注意**: 280倍高速化はコンパイル性能（ビルド時間）であり、実行性能とは異なります。

詳細: [docs/execution-backends.md](execution-backends.md)

## 12. クイック実行（ローカル）

- ビルド: `cargo build --release -j32`
- 実行: `./target/release/nyash program.nyash`
- WASM: `./target/release/nyash --compile-wasm program.nyash`

---

**最終更新: 2025年8月16日** - **Phase 9.75h完了記念 大幅更新**
- 🌟 **自動リテラル変換実装**: 文字列・数値・真偽値の自動Box変換（革命的ユーザビリティ向上）
- ✅ **using nyashstd完全実装**: 標準ライブラリアクセス機能完成
- ✅ **birth構文追加**: 「生命をBoxに与える」統一コンストラクタ
- ✅ **現代的構文例追加**: 最新機能を活用した実用コード例
- ✅ **性能数値修正**: WASM 13.5倍（実行性能）・280倍（コンパイル性能）
- ✅ **ビルトインBoxリスト最新化**: 実装済み17種類のBox完全リスト

### 🚀 **今回の革命的改善**
**Everything is Box哲学 + 使いやすさ** を完全両立達成！
- **Before**: `local text = new StringBox("Hello")`（冗長）
- **After**: `local text = "Hello"`（シンプル、自動変換）
- **結果**: パーサーレベル変換により実行時オーバーヘッドゼロ

