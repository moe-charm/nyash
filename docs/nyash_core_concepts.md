# Nyash言語コアコンセプト AI向け速習ガイド

このドキュメントは、AIアシスタントがNyashプログラミング言語を迅速に理解するために、そのコアコンセプトを凝縮して提供します。

## 1. 基本哲学: Everything is a Box (すべてはBoxである)

- Nyashの基本原則は「すべてがBoxである」ということです。
- 単純な整数から複雑な構造体まで、すべてのデータ型は「Box」オブジェクトの一種です。これにより、純粋で一貫性のあるオブジェクトベースのシステムが実現されています。

## 2. オブジェクトモデルとデリゲーション (Nyash独自の方式)

Nyashは古典的な継承ではなく、デリゲーション（委譲）モデルを使用します。これは非常に重要な違いです。

- **Boxの定義:**
  ```nyash
  box MyBox {
      // フィールドはinitやpackのようなコンストラクタ内で宣言される
      // メソッドはこの場所に定義される
  }
  ```

- **コンストラクタ (優先度: `pack` > `init`)**
  - **`pack` (推奨):** Boxを構築するためのモダンで直感的な方法。オブジェクトのフィールドに値を「詰める」メソッドです。
    ```nyash
    box User {
        init { name, email } // フィールド宣言は依然として必要

        pack(userName, userEmail) {
            me.name = userName
            me.email = userEmail
        }
    }
    local user = new User("Alice", "alice@example.com") // packが呼び出される
    ```
  - **`init` (旧式/シンプル):** 引数を直接フィールドにマッピングする、よりシンプルなコンストラクタ。
    ```nyash
    box Point {
        init { x, y }
    }
    local p = new Point(10, 20) // initが呼び出される
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
  box Admin from User {
      override pack(name, email, perms) {
          // ... 実装 ...
      }
  }
  ```

- **デリゲートされたメソッドの呼び出し (`from`キーワード):** オーバーライドしたメソッド内から親の実装を呼び出すには、`from Parent.method()`を使用します。
  ```nyash
  override pack(name, email, perms) {
      // Userのpackメソッドを明示的に呼び出す
      from User.pack(name, email)
      me.permissions = perms
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

## 3. 構文クイックリファレンス

- **厳格な変数宣言:** すべての変数は使用前に宣言が必要です。
  - `local my_var`: ローカル変数を宣言します。
  - `me.field`: 現在のBoxインスタンスのフィールドにアクセスします。
  - `outbox product`: 静的関数内で使用され、所有権が呼び出し元に移転される変数を宣言します。

- **統一されたループ:** ループ構文は一種類のみです。
  ```nyash
  loop(condition) {
      // ...
  }
  ```

- **プログラムのエントリーポイント:** 実行は`static box Main`の`main`メソッドから開始されます。
  ```nyash
  static box Main {
      main() {
          // プログラムはここから開始
      }
  }
  ```

## 4. 演算子

- **論理演算子:** `and`, `or`, `not`
- **算術演算子:** `+`, `-`, `*`, `/` (ゼロ除算をハンドルします)
- **比較演算子:** `==`, `!=`, `<`, `>`, `<=`, `>=`

## 5. 主要なビルトインBox

- コア（環境を選ばず利用できる）
  - **`ConsoleBox`**: 基本的なI/O (例: `log()`)
  - **`ArrayBox` / `MapBox`**: 配列・マップ操作
  - **`TimeBox` / `RandomBox` / `RegexBox` / `JSONBox` / `StreamBox`**: 汎用ユーティリティ
  - **`DebugBox`**: イントロスペクション/デバッグ (例: `memoryReport()`)

- 環境依存（実行コンテキストに注意）
  - **`P2PBox`**: P2P通信
  - **`EguiBox`**: GUI（メインスレッド制約など）

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

## 11. クイック実行（ローカル）

- ビルド: `cargo build --bin nyash`
- 実行: `cargo run -- ./local_tests/sample.nyash`

