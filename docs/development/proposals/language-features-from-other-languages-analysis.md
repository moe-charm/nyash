# 他言語の優れた機能分析とHakoruneへの適用可能性調査

**作成日**: 2025-10-12
**作成者**: Task先生（プログラミング言語の賢者）
**目的**: 他言語の「これは良い！」機能をHakoruneに取り入れるべきか判断

---

## 分析対象言語

1. Python - 初心者に優しい言語の代表
2. Rust - メモリ安全性・並行性の最先端
3. Go - シンプルさと実用性の両立
4. Kotlin - Java改善の極致
5. Swift - モダン言語設計の集大成
6. Elixir - 関数型+並行性の融合
7. TypeScript - 型安全性の漸進的導入

---

## 機能1: if let / while let パターン（Rust, Swift）

### 概要
単一パターンマッチングを簡潔に書ける構文。`Option<T>` や `Result<T, E>` の値を扱う際に、完全な `match` を書かずに済む。

### 他言語での実装例

**Rust:**
```rust
// if let - 単一パターンマッチング
if let Some(value) = optional {
    println!("Got: {}", value);
}

// while let - ループとパターンマッチング結合
while let Some(item) = iterator.next() {
    process(item);
}

// Result型での使用
if let Ok(data) = read_file("config.json") {
    parse(data);
}
```

**Swift:**
```swift
// guard let - 早期リターンパターン
guard let user = currentUser else {
    return
}
// userはこのスコープで使用可能

// if let - 単一パターン
if let name = user?.profile?.name {
    print(name)
}
```

### Hakoruneでの実現可能性
**既にある（糖衣構文で可能）**

Hakoruneには以下が既存：
- `match` 式（Phase 12.7実装済み）
- `?` 演算子（Result伝播、Phase 12.7実装済み）

### Hakoruneに取り入れるべきか
**Partial（マクロでの実装を推奨）**

#### 理由
1. **完全なmatch式が既存** - 機能的には既に実現可能
2. **糖衣構文としての価値は高い** - 頻出パターンの簡略化
3. **コア拡張は不要** - マクロ実装で十分

#### 取り入れる場合の実装方針
```hakorune
// Phase 1: マクロ実装（@if_let, @while_let）
@if_let(Some(x), optional_value) {
    console.log(x)
}

// ↓ 展開後
match optional_value {
    Some(x) => {
        console.log(x)
    },
    _ => {}
}

// Phase 2: パーサー拡張（将来検討）
if let Some(x) = optional_value {
    console.log(x)
}
```

### Everything is Box との整合性
**整合性あり**

- ResultBox/OptionBox（将来実装）と自然に組み合わせ可能
- Box型のパターンマッチングとして統一的に実装

### 優先度
**Medium**

- 理由: match式で代替可能だが、頻出パターンの簡略化には価値
- 推奨: Phase 20（マクロシステム完成後）で `@if_let` マクロとして実装

---

## 機能2: guard文（Swift）

### 概要
早期リターンを強制する構文。条件が満たされない場合は必ずスコープを抜ける。ネストを減らし、「正常系」を左側に保つ。

### 他言語での実装例

**Swift:**
```swift
func processUser(_ user: User?) {
    guard let user = user else {
        return
    }
    // user is non-nil here

    guard user.isActive else {
        print("Inactive user")
        return
    }

    guard user.hasPermission("admin") else {
        return
    }

    // All checks passed, main logic here
    performAdminTask(user)
}
```

### Hakoruneでの実現可能性
**糖衣構文で可能**

現状のHakoruneでは以下で代替：
```hakorune
function processUser(user) {
    if user == null {
        return
    }

    if not user.isActive {
        console.log("Inactive user")
        return
    }

    // main logic
}
```

### Hakoruneに取り入れるべきか
**Yes（高優先度）**

#### 理由
1. **可読性の大幅向上** - 「ガード条件」と「メインロジック」の分離が明確
2. **ネスト地獄回避** - if-elseの多重ネストを防ぐ
3. **Fail-Fast哲学と一致** - Hakoruneの設計思想と完全に合致
4. **初心者に優しい** - 「異常系は先に排除」パターンの明示

#### 取り入れる場合の実装方針

**Phase 1: マクロ実装（即座に可能）**
```hakorune
@guard(user != null, "return")
@guard(user.isActive, "return")

// ↓ 展開後
if user == null {
    return
}
if not user.isActive {
    return
}
```

**Phase 2: パーサー拡張（Phase 20以降）**
```hakorune
guard user != null else {
    return
}

guard user.isActive else {
    console.log("Inactive")
    return
}
```

**Phase 3: guard let統合（さらに将来）**
```hakorune
guard let user = optional_user else {
    return
}
// userはここで使用可能
```

### Everything is Box との整合性
**完全に整合**

- すべての条件チェックはBox型の真偽値評価
- null安全性（NullBox）との相性が良い
- ResultBox/OptionBoxとの組み合わせで強力

### 優先度
**High**

- **即座にマクロ実装可能** - `@guard` マクロ（Phase 19-20で実装）
- **Fail-Fast文化の強化** - Hakoruneの設計思想を明示的にサポート
- **開発効率向上** - ガード条件の標準化によるコード品質向上

---

## 機能3: defer文（Go, Swift）

### 概要
関数終了時に必ず実行される処理を登録。リソース解放、クリーンアップ処理の保証。

### 他言語での実装例

**Go:**
```go
func readFile(filename string) error {
    file, err := os.Open(filename)
    if err != nil {
        return err
    }
    defer file.Close()  // 関数終了時に必ず実行

    // 処理中にエラーがあっても確実にClose
    data, err := ioutil.ReadAll(file)
    if err != nil {
        return err  // ここでreturnしてもCloseされる
    }

    return process(data)
}

// 複数のdefer - LIFO順（後入れ先出し）
func example() {
    defer fmt.Println("1")
    defer fmt.Println("2")
    defer fmt.Println("3")
    // 出力: 3, 2, 1
}
```

**Swift:**
```swift
func processFile() throws {
    let file = try open("data.txt")
    defer { file.close() }

    // 処理...
    let data = try file.read()
}
```

### Hakoruneでの実現可能性
**既にある（cleanup構文）**

Hakoruneには既に `cleanup` ブロックが実装済み（Stage 3、postfix）：

```hakorune
function readFile(path) {
    local file = FileBox.open(path)

    open(path) cleanup {
        file.close()
    }

    // 処理...
}
```

### Hakoruneに取り入れるべきか
**Already Implemented（機能拡張検討）**

#### 理由
1. **既存のcleanupで十分** - 基本機能は実現済み
2. **Goのdefer相当** - postfix cleanupがそれに該当
3. **拡張の余地あり** - 複数cleanup、LIFO順実行の保証

#### 取り入れる場合の実装方針

**現状（既存機能）:**
```hakorune
doWork() cleanup {
    console.log("Always executed")
}
```

**拡張案1: defer構文（別名として）**
```hakorune
function processFile(path) {
    local file = open(path)
    defer { file.close() }  // cleanup と同等の別名

    // 処理...
}
```

**拡張案2: 複数cleanupのLIFO保証**
```hakorune
function complex() {
    local a = acquire1()
    defer { release1(a) }

    local b = acquire2()
    defer { release2(b) }

    local c = acquire3()
    defer { release3(c) }

    // 実行順: release3 → release2 → release1（LIFO）
}
```

**拡張案3: fini()との統合検討**
```hakorune
box ResourceBox {
    handle: HandleBox

    birth(path) {
        me.handle = open(path)
        defer { me.handle.close() }  // コンストラクタ内defer
    }

    fini() {
        // deferで登録された処理も実行される？
    }
}
```

### Everything is Box との整合性
**完全に整合**

- Box型のリソース管理と自然に統合
- fini()メソッドとの役割分担明確化が必要
- 「決定論的リソース解放」哲学と一致

### 優先度
**Low（既存機能で十分、拡張は将来検討）**

- 現状の `cleanup` で基本機能は実現済み
- LIFO順実行の保証など、詳細仕様の明確化は価値あり
- Phase 20以降で、`defer` を `cleanup` の別名として導入検討

---

## 機能4: パイプ演算子（Elixir, F#）

### 概要
関数適用を左から右に読める形で連鎖。データ変換パイプラインを直感的に記述。

### 他言語での実装例

**Elixir:**
```elixir
# パイプ演算子 |>
result = data
  |> normalize()
  |> transform()
  |> validate()
  |> save()

# 引数を渡す場合
result = "hello"
  |> String.upcase()
  |> String.reverse()
  |> String.slice(0, 3)
```

**F#:**
```fsharp
let result =
    data
    |> List.filter (fun x -> x > 0)
    |> List.map (fun x -> x * 2)
    |> List.sum
```

### Hakoruneでの実現可能性
**既にある（糖衣構文で実装済み）**

Phase 12.7-Bで既に実装済み：
```hakorune
// パイプライン（既存機能）
result = data |> normalize() |> transform() |> process()

// ゲート: NYASH_SYNTAX_SUGAR_LEVEL=basic|full
```

### Hakoruneに取り入れるべきか
**Already Implemented**

#### 理由
- Phase 12.7-Bで既に実装済み
- ゲート管理下で安全に使用可能

#### 現在の実装状況
```hakorune
// 基本パイプライン
data = input
    |> normalize()
    |> validate()
    |> transform()

// メソッドチェーンとの併用
result = data
    |> process()
    |> toUpperCase()
    |> trim()
```

### Everything is Box との整合性
**完全に整合**

- すべてのBox型で統一的に使用可能
- メソッドチェーンとの使い分けが可能

### 優先度
**N/A（既存機能）**

- 既に実装済み
- ドキュメント充実化が必要

---

## 機能5: With文 / Context Manager（Python, Kotlin）

### 概要
リソースの自動管理。取得から解放までを構文レベルで保証。

### 他言語での実装例

**Python:**
```python
# with文 - コンテキストマネージャー
with open('file.txt') as f:
    data = f.read()
    process(data)
# ここでfは自動的にclose済み

# 複数リソース
with open('input.txt') as infile, open('output.txt', 'w') as outfile:
    outfile.write(infile.read())

# カスタムコンテキストマネージャー
class DatabaseConnection:
    def __enter__(self):
        self.conn = connect()
        return self.conn

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.conn.close()

with DatabaseConnection() as db:
    db.query("SELECT * FROM users")
```

**Kotlin:**
```kotlin
// use関数 - 自動クローズ
File("data.txt").inputStream().use { stream ->
    val data = stream.readBytes()
    process(data)
}

// 複数リソース
FileInputStream("input.txt").use { input ->
    FileOutputStream("output.txt").use { output ->
        input.copyTo(output)
    }
}
```

### Hakoruneでの実現可能性
**糖衣構文 + Boxパターンで可能**

Hakoruneでは以下で実現可能：
1. `cleanup` ブロック（既存）
2. `fini()` メソッド（既存）
3. 新規: `with` マクロ/構文

### Hakoruneに取り入れるべきか
**Yes（高優先度）**

#### 理由
1. **リソース管理の標準化** - File/DB/Networkなど頻出パターン
2. **既存機能の組み合わせ** - cleanup + fini() の統一的糖衣構文
3. **初心者に優しい** - 「リソースの取得と解放は必ずペア」を強制

#### 取り入れる場合の実装方針

**Phase 1: マクロ実装（即座に可能）**
```hakorune
// @with マクロ
@with(file, FileBox.open("data.txt")) {
    local data = file.read()
    process(data)
}

// ↓ 展開後
local file = FileBox.open("data.txt")
{
    local data = file.read()
    process(data)
} cleanup {
    file.close()
}
```

**Phase 2: パーサー拡張（Phase 20以降）**
```hakorune
// with構文
with file = FileBox.open("data.txt") {
    local data = file.read()
    process(data)
}
// ここでfile.close()が自動実行

// 複数リソース
with input = FileBox.open("input.txt"),
     output = FileBox.open("output.txt") {
    output.write(input.read())
}
```

**Phase 3: Box統合（Disposable Box Pattern）**
```hakorune
// IDisposable インターフェース（将来）
interface IDisposable {
    dispose()
}

box FileBox from IDisposable {
    birth(path) {
        me.handle = open_native(path)
    }

    dispose() {
        me.handle.close()
    }

    fini() {
        me.dispose()  // fini()からdispose()を呼ぶ
    }
}

// with構文は自動的にdispose()を呼ぶ
with file = FileBox.open("data.txt") {
    // ...
}
// ↓ 展開後
local file = FileBox.open("data.txt")
{
    // ...
} cleanup {
    file.dispose()
}
```

### Everything is Box との整合性
**完全に整合**

- すべてのBox型に `IDisposable` インターフェースを実装可能
- fini()との役割分担明確：
  - `fini()`: GC時の最終解放（Arc<Mutex>ドロップ時）
  - `dispose()`: 明示的リソース解放（with/cleanup時）

### 優先度
**High**

- **Phase 19-20でマクロ実装** - `@with` マクロ
- **Phase 21以降でパーサー拡張** - `with` 構文
- **IDisposableパターン確立** - 標準ライブラリ全体で統一

---

## 機能6: スコープ関数（Kotlin: let, apply, also, run, with）

### 概要
オブジェクトのスコープ内でコードブロックを実行。null安全性やビルダーパターンとの相性が良い。

### 他言語での実装例

**Kotlin:**
```kotlin
// let - null安全な操作
user?.let {
    println(it.name)
    sendEmail(it.email)
}

// apply - オブジェクト設定
val person = Person().apply {
    name = "Alice"
    age = 30
    email = "alice@example.com"
}

// also - 追加操作（ログなど）
val result = compute()
    .also { println("Result: $it") }
    .process()

// run - ブロック実行
val config = run {
    val env = System.getenv("ENV")
    loadConfig(env)
}

// with - 複数操作
with(canvas) {
    drawLine(0, 0, 100, 100)
    drawCircle(50, 50, 25)
    drawText("Hello", 10, 10)
}
```

### Hakoruneでの実現可能性
**一部既存、マクロで完全実現可能**

- `me` 構文（既存）- apply相当
- Lambda式（Phase 12.7）- let/also相当の基礎
- 必要な追加: スコープ関数マクロ

### Hakoruneに取り入れるべきか
**Partial（apply/withのみ推奨）**

#### 理由
1. **apply相当は`me`で実現可能** - 既存の `me` がほぼ同等
2. **let/alsoは冗長になる可能性** - Hakoruneのメソッドチェーンで十分
3. **withは価値あり** - レシーバーを明示しない複数操作

#### 取り入れる場合の実装方針

**Phase 1: マクロ実装**
```hakorune
// @with マクロ（既にwith構文で提案済み）
@with(canvas) {
    drawLine(0, 0, 100, 100)
    drawCircle(50, 50, 25)
    drawText("Hello", 10, 10)
}

// ↓ 展開後
local __recv = canvas
__recv.drawLine(0, 0, 100, 100)
__recv.drawCircle(50, 50, 25)
__recv.drawText("Hello", 10, 10)
```

**Phase 2: レシーバー省略構文（将来検討）**
```hakorune
with canvas {
    .drawLine(0, 0, 100, 100)    // .drawLine は canvas.drawLine
    .drawCircle(50, 50, 25)
    .drawText("Hello", 10, 10)
}
```

**既存機能での代替（apply相当）:**
```hakorune
// Hakoruneの現在の書き方
box Person {
    name: StringBox
    age: IntegerBox
    email: StringBox

    birth() {
        // 初期化はbirthで
    }

    configure(personName, personAge, personEmail) {
        me.name = personName
        me.age = personAge
        me.email = personEmail
        return me  // メソッドチェーン可能
    }
}

local person = new Person()
    .configure("Alice", 30, "alice@example.com")
```

### Everything is Box との整合性
**整合性あり（一部重複）**

- `me` 構文は既にBox型の自己参照として確立
- 追加のスコープ関数は「糖衣構文」としての位置づけ
- with構文のみが新しい価値を提供

### 優先度
**Low（withのみMedium）**

- **let/apply/also/run**: 既存機能で代替可能、優先度低
- **with**: レシーバー省略として価値あり、Phase 20以降で検討

---

## 機能7: goroutineとchannels（Go）

### 概要
軽量スレッド（goroutine）とメッセージパッシング（channel）による並行処理。

### 他言語での実装例

**Go:**
```go
// goroutine - 軽量スレッド
go func() {
    fmt.Println("Hello from goroutine")
}()

// channel - メッセージパッシング
ch := make(chan int)

go func() {
    ch <- 42  // 送信
}()

value := <-ch  // 受信
fmt.Println(value)

// select - 複数channel待機
select {
case msg1 := <-ch1:
    fmt.Println("Received from ch1:", msg1)
case msg2 := <-ch2:
    fmt.Println("Received from ch2:", msg2)
case <-time.After(1 * time.Second):
    fmt.Println("Timeout")
}

// buffered channel
ch := make(chan int, 10)  // バッファサイズ10
```

### Hakoruneでの実現可能性
**既にある（FutureBox + async/await）**

Hakoruneには既に非同期処理の基盤が存在：
- `FutureBox` - 非同期処理結果
- `nowait`/`await` キーワード（計画）
- RustのAsync/Awaitを基盤として実装

### Hakoruneに取り入れるべきか
**Partial（ChannelBoxの追加検討）**

#### 理由
1. **goroutine相当は既存** - `nowait` + `FutureBox`
2. **channelは価値あり** - メッセージパッシングパターンは強力
3. **selectは検討価値あり** - 複数Future待機の統一的構文

#### 取り入れる場合の実装方針

**Phase 1: ChannelBox実装（Box型として）**
```hakorune
box ChannelBox<T> {
    capacity: IntegerBox  // バッファサイズ

    birth(bufferSize) {
        me.capacity = bufferSize
        // 内部でRust mpsc::channel 使用
    }

    send(value: T) {
        // 送信（ブロック可能）
    }

    try_send(value: T): ResultBox<VoidBox, ErrorBox> {
        // 非ブロッキング送信
    }

    recv(): T {
        // 受信（ブロック可能）
    }

    try_recv(): ResultBox<T, ErrorBox> {
        // 非ブロッキング受信
    }
}

// 使用例
local ch = new ChannelBox(10)  // バッファサイズ10

nowait {
    ch.send(42)
}

local value = ch.recv()
console.log(value)
```

**Phase 2: select構文（将来検討）**
```hakorune
// 複数Futureやチャンネルを待機
select {
    case msg = ch1.recv() => {
        console.log("From ch1: " + msg)
    },
    case msg = ch2.recv() => {
        console.log("From ch2: " + msg)
    },
    timeout 1000 => {
        console.log("Timeout")
    }
}

// ↓ 展開後（内部実装）
local selector = new SelectorBox()
selector.add_channel(ch1)
selector.add_channel(ch2)
selector.set_timeout(1000)
local result = selector.wait()

match result {
    "ch1" => { ... },
    "ch2" => { ... },
    "timeout" => { ... }
}
```

**Phase 3: 非同期統合（async/await連携）**
```hakorune
// FutureBoxとChannelBoxの統合
nowait future = asyncTask()

select {
    case result = await future => {
        console.log("Task completed: " + result)
    },
    case msg = ch.recv() => {
        console.log("Message received: " + msg)
    }
}
```

### Everything is Box との整合性
**完全に整合**

- ChannelBoxは通常のBox型として実装
- FutureBoxとの自然な連携
- Rust mpsc::channelをバックエンドとして利用可能

### 優先度
**Medium（Phase 21-22で実装検討）**

- **ChannelBox**: メッセージパッシングパターンの標準化
- **select構文**: 複数非同期待機の統一的API
- **Rust async/await基盤活用**: 実装コストが比較的低い

---

## 機能8: panic/recover（Go）

### 概要
プログラムパニック時の復旧機構。予期しないエラーからの回復。

### 他言語での実装例

**Go:**
```go
// panic - プログラム異常終了
func riskyOperation() {
    if err := checkPrecondition(); err != nil {
        panic("Precondition failed")
    }
}

// recover - パニックから回復
func safeExecute() {
    defer func() {
        if r := recover(); r != nil {
            fmt.Println("Recovered from panic:", r)
        }
    }()

    riskyOperation()  // ここでpanicしても回復
}

// 実用例: サーバーハンドラー
func handler(w http.ResponseWriter, r *http.Request) {
    defer func() {
        if r := recover(); r != nil {
            log.Printf("Panic: %v", r)
            http.Error(w, "Internal Server Error", 500)
        }
    }()

    // ハンドラー処理...
}
```

### Hakoruneでの実現可能性
**既にある（throw/catch、ResultBox）**

Hakoruneには既に例外処理機構が存在：
- `throw` - 例外発生
- `catch` - 例外捕獲（postfix、Stage 3）
- `cleanup` - finally相当
- `ResultBox` - エラー値による処理（計画）

### Hakoruneに取り入れるべきか
**No（既存機能で十分）**

#### 理由
1. **panic相当は`throw`で実現** - 既存の例外機構
2. **recover相当は`catch`で実現** - postfix catch
3. **Fail-Fast哲学と一致** - 予期しないエラーは即座に失敗
4. **ResultBoxで十分** - 予測可能なエラーはResult型で処理

#### 既存機能での実現
```hakorune
// panic/recover相当（既存機能）
function riskyOperation() {
    if not checkPrecondition() {
        throw new ErrorBox("Precondition failed")
    }
}

function safeExecute() {
    riskyOperation() catch(e) {
        console.log("Caught error: " + e)
    }
}

// defer + recover相当（cleanup + catch）
function handler(request) {
    processRequest(request)
        catch(e) {
            console.log("Error: " + e)
            return new ErrorResponseBox(500)
        }
        cleanup {
            console.log("Request completed")
        }
}
```

### Everything is Box との整合性
**完全に整合**

- ErrorBoxは通常のBox型
- throw/catchは既存の例外機構
- ResultBox（将来）でエラー処理の選択肢を提供

### 優先度
**N/A（既存機能）**

- panic/recoverの追加実装は不要
- 既存のthrow/catch/cleanupで十分

---

## 機能9: Walrus演算子（Python := ）

### 概要
代入と式を同時に行う。条件式内での変数束縛。

### 他言語での実装例

**Python:**
```python
# 条件式内での代入
if (n := len(data)) > 10:
    print(f"Large dataset: {n} items")

# ループ条件での代入
while (line := file.readline()):
    process(line)

# リスト内包表記
results = [y for x in data if (y := transform(x)) is not None]

# with文での使用（注意が必要）
if (file := open('data.txt')):
    data = file.read()
# ※ context managerでは注意！
```

### Hakoruneでの実現可能性
**糖衣構文で可能**

現在のHakoruneでは以下が必要：
```hakorune
// 現状（分離）
local n = data.length()
if n > 10 {
    console.log("Large dataset: " + n)
}
```

### Hakoruneに取り入れるべきか
**No（可読性を損なう可能性）**

#### 理由
1. **可読性の低下** - 代入と条件を混在させると混乱
2. **デバッグの困難化** - 式の評価と代入が同時に起こる
3. **Hakoruneの哲学と不一致** - 「明示的が暗黙的より良い」
4. **必要性が低い** - 1行増えるだけで明確になる

#### 推奨する代替パターン
```hakorune
// ❌ Walrus演算子（非推奨）
if (local n = data.length()) > 10 {
    // ...
}

// ✅ 明示的な分離（推奨）
local n = data.length()
if n > 10 {
    console.log("Large dataset: " + n)
}

// ✅ スコープ最小化（必要なら）
{
    local n = data.length()
    if n > 10 {
        // ...
    }
}
```

### Everything is Box との整合性
**整合性あり（実装可能だが非推奨）**

- 技術的には実装可能
- しかし、Box理論の「明示的」哲学と合わない

### 優先度
**None（実装しない）**

- 可読性を損なう
- Hakoruneの設計思想に反する
- 明示的な変数宣言を推奨

---

## 機能10: Optional Chaining（Swift, TypeScript, JavaScript）

### 概要
null/undefinedチェックを簡潔に記述。深いネストされたプロパティアクセスを安全に。

### 他言語での実装例

**Swift:**
```swift
// Optional Chaining
let street = user?.address?.street

// メソッド呼び出し
user?.profile?.updateName("Alice")

// 配列アクセス
let firstItem = array?[0]
```

**TypeScript:**
```typescript
// Optional Chaining
const street = user?.address?.street

// 配列・メソッド
const firstUser = users?.[0]?.getName()

// Nullish Coalescing と組み合わせ
const name = user?.profile?.name ?? "Guest"
```

### Hakoruneでの実現可能性
**糖衣構文で可能**

現状のHakoruneでは：
```hakorune
// 現状（冗長）
local street
if user != null {
    if user.address != null {
        street = user.address.street
    }
}
```

### Hakoruneに取り入れるべきか
**Yes（高優先度）**

#### 理由
1. **null安全性の強化** - NullPointerErrorを防ぐ
2. **可読性の大幅向上** - ネストされたnullチェックの簡潔化
3. **初心者に優しい** - nullチェック忘れを防ぐ
4. **既存の`?`演算子と一貫** - Result伝播と統一的

#### 取り入れる場合の実装方針

**Phase 1: マクロ実装（即座に可能）**
```hakorune
// @optional_chain マクロ
local street = @optional_chain(user, .address, .street)

// ↓ 展開後
local street
if user != null {
    if user.address != null {
        street = user.address.street
    }
}
```

**Phase 2: パーサー拡張（Phase 20以降）**
```hakorune
// ?. 演算子
local street = user?.address?.street

// メソッド呼び出し
user?.profile?.updateName("Alice")

// ?? 演算子と組み合わせ（Nullish Coalescing）
local name = user?.profile?.name ?? "Guest"
```

**Phase 3: NullBox統合**
```hakorune
// NullBoxとの統合
box User {
    profile: ProfileBox  // nullableでない
    optionalData: NullBox<DataBox>  // nullable

    getName(): StringBox {
        // profile は常に存在
        return me.profile.name
    }

    getOptionalInfo(): StringBox {
        // optionalData はnullable
        return me.optionalData?.info ?? "No info"
    }
}
```

### Everything is Box との整合性
**完全に整合**

- NullBoxとの自然な統合
- すべてのBox型で統一的に使用可能
- `?` 演算子（Result伝播）との一貫性

### 優先度
**High**

- **Phase 19-20でマクロ実装** - `@optional_chain`
- **Phase 21以降でパーサー拡張** - `?.` 演算子
- **NullBox型の正式導入** - 型システムとの統合

---

## 機能11: Nullish Coalescing（JavaScript, TypeScript, Swift）

### 概要
null/undefinedの場合のみデフォルト値を返す。falsy値（0, ""）とnull/undefinedを区別。

### 他言語での実装例

**TypeScript:**
```typescript
// ?? 演算子 - null/undefined のみ
const value = input ?? "default"

// || との違い
const a = 0 || 10      // 10（0はfalsyなので）
const b = 0 ?? 10      // 0（0はnullでない）

const c = "" || "def"  // "def"（""はfalsyなので）
const d = "" ?? "def"  // ""（""はnullでない）
```

**Swift:**
```swift
// ?? 演算子
let name = user?.name ?? "Guest"
```

### Hakoruneでの実現可能性
**糖衣構文で可能**

現状のHakoruneでは：
```hakorune
// 現状（||はfalsyチェック）
local value = input or "default"

// null専用チェック（冗長）
local value
if input == null {
    value = "default"
} else {
    value = input
}
```

### Hakoruneに取り入れるべきか
**Yes（中優先度）**

#### 理由
1. **null/falsyの区別** - 0や空文字列を有効値として扱える
2. **`||`との使い分け** - 意図を明確化
3. **Optional Chainingと相性良** - `?.`と`??`の組み合わせ

#### 取り入れる場合の実装方針

**Phase 1: マクロ実装**
```hakorune
// @nullish マクロ
local value = @nullish(input, "default")

// ↓ 展開後
local value
if input == null {
    value = "default"
} else {
    value = input
}
```

**Phase 2: パーサー拡張（Phase 20以降）**
```hakorune
// ?? 演算子
local value = input ?? "default"

// Optional Chaining と組み合わせ
local name = user?.profile?.name ?? "Guest"

// || との使い分け
local a = input || "default"   // falsyチェック（0, "", false, null）
local b = input ?? "default"   // nullチェックのみ（null）
```

### Everything is Box との整合性
**完全に整合**

- NullBoxとの自然な統合
- 真偽値の統一的扱い（Truthiness、quick-reference.md 48-53行）
- IntegerBox, StringBox, BoolBoxとの明確な区別

### 優先度
**Medium**

- **Phase 20でマクロ実装** - `@nullish`
- **Phase 21以降でパーサー拡張** - `??` 演算子
- **Optional Chainingと同時実装が理想**

---

## 機能12: Discriminated Unions + Type Guards（TypeScript, Rust）

### 概要
タグ付きユニオン型。パターンマッチングと型の絞り込み（narrowing）。

### 他言語での実装例

**TypeScript:**
```typescript
// Discriminated Union
type Result<T, E> =
    | { type: 'ok', value: T }
    | { type: 'error', error: E }

function process(result: Result<number, string>) {
    // Type Guard - 型の絞り込み
    if (result.type === 'ok') {
        console.log(result.value)  // number型として扱える
    } else {
        console.log(result.error)  // string型として扱える
    }

    // switch での型絞り込み
    switch (result.type) {
        case 'ok':
            return result.value * 2
        case 'error':
            throw new Error(result.error)
    }
}

// never型による網羅性チェック
function assertNever(x: never): never {
    throw new Error("Unexpected value: " + x)
}

function handle(result: Result<number, string>) {
    switch (result.type) {
        case 'ok':
            return result.value
        case 'error':
            return 0
        default:
            return assertNever(result)  // 全ケース処理済みなら到達不能
    }
}
```

**Rust:**
```rust
// enum（代数的データ型）
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn process(result: Result<i32, String>) {
    match result {
        Ok(value) => println!("Value: {}", value),
        Err(error) => println!("Error: {}", error),
    }
}

// 網羅性チェック（コンパイル時）
match result {
    Ok(v) => v,
    Err(e) => 0,
    // 全パターン必須、漏れがあればコンパイルエラー
}
```

### Hakoruneでの実現可能性
**コア拡張必要（VariantBox、Phase 20計画済み）**

現在のHakorune:
- Phase 19: @enum/@match Macros（マクロのみ、進行中）
- Phase 20: VariantBox Core（本格的enum実装、計画中）

### Hakoruneに取り入れるべきか
**Yes（最高優先度、Phase 20で実装予定）**

#### 理由
1. **型安全性の大幅向上** - ランタイムエラーをコンパイル時に検出
2. **パターンマッチングの完成** - match式を真に実用的に
3. **Result/Option型の基盤** - エラー処理の標準化
4. **他言語との整合性** - Rust/TypeScript/Swiftの標準機能

#### 取り入れる場合の実装方針

**Phase 19（進行中）: マクロ版enum**
```hakorune
// @enum マクロ（現在実装中）
@enum Result<T, E> {
    Ok(T),
    Err(E)
}

// ↓ 展開後（Box + staticコンストラクタ）
box Result {
    __tag: StringBox
    __value: Box

    static Ok(value) {
        local r = new Result()
        r.__tag = "Ok"
        r.__value = value
        return r
    }

    static Err(error) {
        local r = new Result()
        r.__tag = "Err"
        r.__value = error
        return r
    }

    is_ok(): BoolBox {
        return me.__tag == "Ok"
    }

    unwrap(): T {
        if me.__tag == "Ok" {
            return me.__value
        }
        throw new ErrorBox("Called unwrap on Err")
    }
}
```

**Phase 20（計画中）: VariantBox Core**
```hakorune
// VariantBox - コアレベルのenum型
enum Result<T, E> {
    Ok(T),
    Err(E)
}

// パターンマッチング（型絞り込み付き）
function process(result: Result<IntegerBox, StringBox>) {
    match result {
        Ok(value) => {
            // valueはIntegerBox型として扱える
            console.log("Success: " + value)
        },
        Err(error) => {
            // errorはStringBox型として扱える
            console.log("Error: " + error)
        }
    }
    // 全パターン網羅必須（コンパイラチェック）
}

// ネストパターン
match result {
    Ok(Ok(value)) => console.log("Double success: " + value),
    Ok(Err(inner_error)) => console.log("Inner error: " + inner_error),
    Err(outer_error) => console.log("Outer error: " + outer_error)
}

// ガード条件
match result {
    Ok(value) if value > 0 => console.log("Positive"),
    Ok(value) => console.log("Non-positive"),
    Err(e) => console.log("Error")
}
```

**Phase 21以降: 標準ライブラリ統合**
```hakorune
// 標準Option型
enum Option<T> {
    Some(T),
    None
}

// 標準Result型
enum Result<T, E> {
    Ok(T),
    Err(E)
}

// 使用例
function divide(a: IntegerBox, b: IntegerBox): Result<IntegerBox, StringBox> {
    if b == 0 {
        return Result.Err("Division by zero")
    }
    return Result.Ok(a / b)
}

local result = divide(10, 2)
match result {
    Ok(value) => console.log("Result: " + value),
    Err(error) => console.log("Error: " + error)
}

// ? 演算子との統合
function complex_calculation(): Result<IntegerBox, StringBox> {
    local a = divide(10, 2)?     // Errなら早期return
    local b = divide(20, 4)?
    return Result.Ok(a + b)
}
```

### Everything is Box との整合性
**完全に整合**

- VariantBoxは特殊なBox型
- 各バリアントは通常のBox型を保持
- パターンマッチングはBox型の型チェック

### 優先度
**Critical（Phase 19-20で実装中/計画中）**

- **Phase 19（進行中）**: @enum/@match マクロ - 2-3 weeks
- **Phase 20（計画中）**: VariantBox Core - 18-27人日
- **最重要機能**: セルフホストコンパイラで即座に活用

---

## 機能13: Property Wrappers（Swift）

### 概要
プロパティアクセスに振る舞いを追加。バリデーション、遅延初期化、依存性注入などを宣言的に。

### 他言語での実装例

**Swift:**
```swift
// Property Wrapper 定義
@propertyWrapper
struct Clamped<Value: Comparable> {
    var value: Value
    let range: ClosedRange<Value>

    init(wrappedValue: Value, _ range: ClosedRange<Value>) {
        self.range = range
        self.value = max(range.lowerBound, min(range.upperBound, wrappedValue))
    }

    var wrappedValue: Value {
        get { value }
        set { value = max(range.lowerBound, min(range.upperBound, newValue)) }
    }
}

// 使用例
struct GameSettings {
    @Clamped(0...100) var volume: Int = 50
    @Clamped(0...1) var brightness: Double = 0.8
}

var settings = GameSettings()
settings.volume = 150  // 自動的に100にクランプ
print(settings.volume) // 100

// 他の例: @Published (SwiftUI)
class ViewModel: ObservableObject {
    @Published var text: String = ""  // 変更時にUI更新通知
}
```

### Hakoruneでの実現可能性
**マクロで実現可能**

現在のHakoruneには以下が既存：
- Unified Members（Phase 15）: computed, once, birth_once プロパティ
- フィールド型アノテーション（Phase 12.7）

### Hakoruneに取り入れるべきか
**Partial（基本的な機能のみマクロで実装）**

#### 理由
1. **Unified Membersで一部実現** - computed/once propertyが類似
2. **マクロで拡張可能** - @validatedなどの属性マクロ
3. **コア拡張は不要** - 既存機能の組み合わせで十分

#### 取り入れる場合の実装方針

**Phase 1: 既存機能（Unified Members）**
```hakorune
// computed property - Property Wrapper的な使い方
box GameSettings {
    _volume: IntegerBox
    _brightness: FloatBox

    volume: IntegerBox {
        // getter - クランプ処理
        if me._volume > 100 {
            return 100
        }
        if me._volume < 0 {
            return 0
        }
        return me._volume
    }

    setVolume(value: IntegerBox) {
        // setter - クランプ処理
        if value > 100 {
            me._volume = 100
        } else if value < 0 {
            me._volume = 0
        } else {
            me._volume = value
        }
    }
}

local settings = new GameSettings()
settings.setVolume(150)
console.log(settings.volume)  // 100
```

**Phase 2: マクロ実装（Phase 20以降）**
```hakorune
// @clamped マクロ
box GameSettings {
    @clamped(0, 100)
    volume: IntegerBox = 50

    @clamped(0.0, 1.0)
    brightness: FloatBox = 0.8
}

// ↓ 展開後
box GameSettings {
    _volume: IntegerBox = 50
    _brightness: FloatBox = 0.8

    volume: IntegerBox {
        return me._volume
    }

    setVolume(value: IntegerBox) {
        if value > 100 {
            me._volume = 100
        } else if value < 0 {
            me._volume = 0
        } else {
            me._volume = value
        }
    }

    // brightness同様...
}
```

**Phase 3: 高度なProperty Wrapper（将来検討）**
```hakorune
// @observable マクロ（ReactiveBoxとの連携）
box ViewModel {
    @observable
    text: StringBox = ""

    // ↓ 展開後、textの変更を監視・通知
}

// @lazy マクロ（遅延初期化）
box DataLoader {
    @lazy
    data: ArrayBox {
        loadFromFile("large_data.json")
    }
    // 初回アクセス時のみロード
}

// @validated マクロ（バリデーション）
box UserForm {
    @validated(EmailValidator)
    email: StringBox

    @validated(AgeRangeValidator(18, 120))
    age: IntegerBox
}
```

### Everything is Box との整合性
**完全に整合**

- Property Wrapperは「Box型を返すプロパティ」として実装
- Unified Membersの自然な拡張
- computed/once プロパティとの統一的扱い

### 優先度
**Low（Phase 20以降でマクロ実装検討）**

- 基本機能（clamped, validated）のみ
- Unified Membersで大部分は実現可能
- 高度な機能（@observable等）は長期計画

---

## 機能14: Async/Await（Rust, JavaScript, C#, Python）

### 概要
非同期処理を同期的な見た目で記述。コールバック地獄の回避。

### 他言語での実装例

**Rust:**
```rust
// async関数
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

// async mainも可能
#[tokio::main]
async fn main() {
    let data = fetch_data("https://api.example.com").await;
    println!("Data: {:?}", data);
}

// 複数async処理の並行実行
async fn parallel_fetch() {
    let (data1, data2) = tokio::join!(
        fetch_data("url1"),
        fetch_data("url2")
    );
}
```

**JavaScript:**
```javascript
// async/await
async function fetchUser(id) {
    const response = await fetch(`/users/${id}`)
    const user = await response.json()
    return user
}

// エラーハンドリング
async function safeOperation() {
    try {
        const result = await riskyAsync()
        return result
    } catch (error) {
        console.error(error)
        return null
    }
}

// Promise.all - 並行実行
async function parallel() {
    const [user1, user2, user3] = await Promise.all([
        fetchUser(1),
        fetchUser(2),
        fetchUser(3)
    ])
}
```

### Hakoruneでの実現可能性
**既にある（設計済み、FutureBox + nowait/await）**

言語仕様に既に組み込まれている：
- `nowait` キーワード - async相当
- `await` キーワード - await相当
- `FutureBox` - Promise/Future相当
- `TokenBox` - キャンセルトークン

### Hakoruneに取り入れるべきか
**Already Designed（実装強化が必要）**

#### 理由
1. **既に言語仕様に存在** - nowait/awaitキーワード
2. **実装は部分的** - 完全な実装が必要
3. **Rust async/await基盤** - バックエンドはRust async

#### 現在の設計
```hakorune
// 設計済み構文（LANGUAGE_REFERENCE_2025.md）
nowait future = asyncTask()
local result = await future

// FutureBox
box FutureBox {
    // 内部でRust Future<Output = Box> を保持
}

// TokenBox - キャンセル
box TokenBox {
    cancel()
    is_cancelled(): BoolBox
}
```

#### 実装強化案

**Phase 1: FutureBox完全実装**
```hakorune
// async関数の定義（将来構文）
async function fetchData(url: StringBox): FutureBox<StringBox> {
    local response = await HttpClientBox.get(url)
    local text = await response.text()
    return text
}

// 現状の代替（nowait + lambda）
function fetchData(url: StringBox): FutureBox<StringBox> {
    return FutureBox.spawn(fn() {
        local response = HttpClientBox.get(url)
        local text = response.text()
        return text
    })
}
```

**Phase 2: 並行実行サポート**
```hakorune
// join - 複数Futureの並行実行
local results = FutureBox.join([
    fetchData("url1"),
    fetchData("url2"),
    fetchData("url3")
])

local data1 = results.get(0)
local data2 = results.get(1)
local data3 = results.get(2)

// race - 最初に完了したものを返す
local first = FutureBox.race([
    fetchFromCache(),
    fetchFromNetwork(),
    fetchFromBackup()
])
```

**Phase 3: エラーハンドリング統合**
```hakorune
// await + Result + ? 演算子
async function complexOperation(): Result<DataBox, ErrorBox> {
    local data1 = await fetchData("url1")?
    local data2 = await fetchData("url2")?
    local combined = combine(data1, data2)
    return Result.Ok(combined)
}

// catch統合
local result = await asyncTask()
    catch(e) {
        console.log("Error: " + e)
        return default_value
    }
```

**Phase 4: select統合（channelとの統合）**
```hakorune
// 複数非同期ソースの待機
select {
    case result = await future1 => {
        console.log("Future1 completed: " + result)
    },
    case msg = ch.recv() => {
        console.log("Channel message: " + msg)
    },
    timeout 5000 => {
        console.log("Timeout")
    }
}
```

### Everything is Box との整合性
**完全に整合**

- FutureBox, TokenBoxは通常のBox型
- Rust async/awaitをバックエンドとして活用
- すべての非同期処理がBox型として統一

### 優先度
**High（Phase 21-22で完全実装）**

- **FutureBox強化**: Rust async基盤の完全統合
- **並行実行API**: join/race実装
- **エラーハンドリング**: Result型との統合
- **select構文**: Channel/Futureの統一的待機

---

## 機能15: Actor Model / GenServer（Elixir, Erlang）

### 概要
アクターパターンによる並行処理。状態を持つプロセス間のメッセージパッシング。

### 他言語での実装例

**Elixir:**
```elixir
# GenServer - アクターモデルの実装
defmodule Counter do
  use GenServer

  # Client API
  def start_link(initial_value) do
    GenServer.start_link(__MODULE__, initial_value, name: __MODULE__)
  end

  def increment do
    GenServer.cast(__MODULE__, :increment)
  end

  def get do
    GenServer.call(__MODULE__, :get)
  end

  # Server Callbacks
  def init(initial_value) do
    {:ok, initial_value}
  end

  def handle_cast(:increment, state) do
    {:noreply, state + 1}
  end

  def handle_call(:get, _from, state) do
    {:reply, state, state}
  end
end

# 使用例
{:ok, _pid} = Counter.start_link(0)
Counter.increment()
Counter.increment()
value = Counter.get()  # 2
```

### Hakoruneでの実現可能性
**Boxパターン + async/await で実現可能**

Hakoruneには以下が既存：
- Box型（状態を持つオブジェクト）
- nowait/await（非同期処理）
- ChannelBox（計画中、メッセージパッシング）

### Hakoruneに取り入れるべきか
**Partial（ActorBoxパターンとして実装）**

#### 理由
1. **Box型で状態管理** - 既存機能で実現可能
2. **メッセージパッシング** - ChannelBox実装必要
3. **並行実行** - FutureBoxで実現
4. **完全なActor Modelは過剰** - 軽量版で十分

#### 取り入れる場合の実装方針

**Phase 1: ActorBoxパターン（Box + Channel）**
```hakorune
// ActorBox基底クラス
box ActorBox {
    inbox: ChannelBox
    running: BoolBox

    birth() {
        me.inbox = new ChannelBox(100)
        me.running = true
    }

    spawn(): ActorHandleBox {
        nowait {
            loop(me.running) {
                local msg = me.inbox.recv()
                me.handle_message(msg)
            }
        }
        return new ActorHandleBox(me)
    }

    send(msg: Box) {
        me.inbox.send(msg)
    }

    // サブクラスでオーバーライド
    handle_message(msg: Box) {
        // デフォルト実装
    }

    stop() {
        me.running = false
    }
}

// 具体的なActor実装
box CounterActor from ActorBox {
    state: IntegerBox

    birth() {
        from ActorBox.birth()
        me.state = 0
    }

    override handle_message(msg: Box) {
        match msg {
            "increment" => {
                me.state = me.state + 1
            },
            "get" => {
                // 返信が必要な場合は別途設計
            }
        }
    }
}

// 使用例
local counter = new CounterActor()
local handle = counter.spawn()
handle.send("increment")
handle.send("increment")
```

**Phase 2: Request-Response パターン**
```hakorune
// Messageボックス - リプライチャンネル付き
box Message {
    type: StringBox
    payload: Box
    reply_to: ChannelBox  // 返信用

    birth(msgType, msgPayload) {
        me.type = msgType
        me.payload = msgPayload
        me.reply_to = new ChannelBox(1)
    }
}

// ActorBox拡張
box ActorBox {
    // ...

    call(msg_type: StringBox, payload: Box): Box {
        local msg = new Message(msg_type, payload)
        me.inbox.send(msg)
        return msg.reply_to.recv()  // 返信を待つ
    }

    cast(msg_type: StringBox, payload: Box) {
        local msg = new Message(msg_type, payload)
        me.inbox.send(msg)
        // 返信を待たない
    }
}

// CounterActorの改良版
box CounterActor from ActorBox {
    state: IntegerBox

    birth() {
        from ActorBox.birth()
        me.state = 0
    }

    override handle_message(msg: Message) {
        match msg.type {
            "increment" => {
                me.state = me.state + 1
                if msg.reply_to != null {
                    msg.reply_to.send("ok")
                }
            },
            "get" => {
                msg.reply_to.send(me.state)
            }
        }
    }
}

// 使用例
local counter = new CounterActor()
local handle = counter.spawn()

counter.cast("increment")
counter.cast("increment")
local value = counter.call("get")  // 2
```

**Phase 3: Supervisor パターン（将来）**
```hakorune
// SupervisorBox - アクターの監視・再起動
box SupervisorBox from ActorBox {
    children: ArrayBox  // 子アクター
    restart_strategy: StringBox  // "one_for_one", "all_for_one"

    birth(strategy) {
        from ActorBox.birth()
        me.children = new ArrayBox()
        me.restart_strategy = strategy
    }

    add_child(actor: ActorBox) {
        me.children.push(actor)
        local handle = actor.spawn()
        // 監視開始...
    }

    override handle_message(msg: Message) {
        match msg.type {
            "child_crashed" => {
                local crashed_actor = msg.payload
                me.restart_child(crashed_actor)
            }
        }
    }

    restart_child(actor: ActorBox) {
        // 再起動ロジック...
    }
}
```

### Everything is Box との整合性
**完全に整合**

- ActorBoxは通常のBox型
- メッセージはすべてBox型
- ChannelBoxを介した通信

### 優先度
**Medium（Phase 22以降で実装検討）**

- **ActorBoxパターン**: Box + Channel の組み合わせ
- **Request-Response**: call/castメソッド
- **Supervisor**: 高度な並行処理（長期計画）

---

## 機能16: LINQ / Stream API（C#, Java, Kotlin）

### 概要
コレクション操作の統一的API。遅延評価、メソッドチェーン。

### 他言語での実装例

**C# LINQ:**
```csharp
// LINQ - Language Integrated Query
var result = numbers
    .Where(x => x > 0)
    .Select(x => x * 2)
    .OrderBy(x => x)
    .Take(10)
    .ToList();

// クエリ構文
var result2 = from n in numbers
              where n > 0
              select n * 2;
```

**Java Stream:**
```java
List<Integer> result = numbers.stream()
    .filter(x -> x > 0)
    .map(x -> x * 2)
    .sorted()
    .limit(10)
    .collect(Collectors.toList());
```

**Kotlin:**
```kotlin
val result = numbers
    .filter { it > 0 }
    .map { it * 2 }
    .sorted()
    .take(10)
```

### Hakoruneでの実現可能性
**Boxメソッドとして実装可能**

現在のArrayBoxには一部のメソッドが既存：
- `push`, `pop`, `get`, `set`, `join`（既存）
- 追加必要: `filter`, `map`, `reduce`, `sort`等

### Hakoruneに取り入れるべきか
**Yes（高優先度、標準ライブラリ拡張）**

#### 理由
1. **データ処理の標準化** - 配列・コレクション操作が一貫
2. **関数型プログラミング** - Lambda式（Phase 12.7）との相性
3. **可読性向上** - 命令型よりも宣言的
4. **他言語経験者に優しい** - 広く普及している概念

#### 取り入れる場合の実装方針

**Phase 1: ArrayBox標準メソッド拡張**
```hakorune
// ArrayBoxに追加すべきメソッド
box ArrayBox {
    // 既存メソッド
    push(item: Box)
    pop(): Box
    get(index: IntegerBox): Box
    set(index: IntegerBox, value: Box)
    length(): IntegerBox
    join(separator: StringBox): StringBox

    // 追加メソッド（Phase 20以降）
    filter(predicate: FunctionBox): ArrayBox {
        local result = new ArrayBox()
        local i = 0
        loop(i < me.length()) {
            local item = me.get(i)
            if predicate.call(item) {
                result.push(item)
            }
            i = i + 1
        }
        return result
    }

    map(mapper: FunctionBox): ArrayBox {
        local result = new ArrayBox()
        local i = 0
        loop(i < me.length()) {
            local item = me.get(i)
            local mapped = mapper.call(item)
            result.push(mapped)
            i = i + 1
        }
        return result
    }

    reduce(accumulator: FunctionBox, initial: Box): Box {
        local acc = initial
        local i = 0
        loop(i < me.length()) {
            local item = me.get(i)
            acc = accumulator.call(acc, item)
            i = i + 1
        }
        return acc
    }

    sort(comparator: FunctionBox): ArrayBox {
        // ソートアルゴリズム実装
        return me  // in-place sort
    }

    take(n: IntegerBox): ArrayBox {
        local result = new ArrayBox()
        local i = 0
        loop(i < n and i < me.length()) {
            result.push(me.get(i))
            i = i + 1
        }
        return result
    }

    skip(n: IntegerBox): ArrayBox {
        local result = new ArrayBox()
        local i = n
        loop(i < me.length()) {
            result.push(me.get(i))
            i = i + 1
        }
        return result
    }
}

// 使用例
local numbers = new ArrayBox()
numbers.push(1)
numbers.push(2)
numbers.push(3)
numbers.push(4)
numbers.push(5)

local result = numbers
    .filter(fn(x) { x > 2 })
    .map(fn(x) { x * 2 })
    .take(2)

// result = [6, 8]
```

**Phase 2: StreamBox（遅延評価版）**
```hakorune
// StreamBox - 遅延評価版コレクション
box StreamBox {
    source: ArrayBox
    operations: ArrayBox  // 操作のリスト

    birth(array: ArrayBox) {
        me.source = array
        me.operations = new ArrayBox()
    }

    filter(predicate: FunctionBox): StreamBox {
        me.operations.push(new FilterOp(predicate))
        return me  // メソッドチェーン
    }

    map(mapper: FunctionBox): StreamBox {
        me.operations.push(new MapOp(mapper))
        return me
    }

    collect(): ArrayBox {
        // ここで初めて実行（遅延評価）
        local result = me.source
        local i = 0
        loop(i < me.operations.length()) {
            local op = me.operations.get(i)
            result = op.apply(result)
            i = i + 1
        }
        return result
    }
}

// 使用例
local stream = new StreamBox(numbers)
local result = stream
    .filter(fn(x) { x > 0 })
    .map(fn(x) { x * 2 })
    .collect()  // ここで初めて実行
```

**Phase 3: 標準ライブラリ全体への展開**
```hakorune
// MapBox拡張
box MapBox {
    filter(predicate: FunctionBox): MapBox
    map(mapper: FunctionBox): MapBox
    keys(): ArrayBox
    values(): ArrayBox
    entries(): ArrayBox  // [key, value]のペア
}

// StringBox拡張
box StringBox {
    split(delimiter: StringBox): ArrayBox
    chars(): ArrayBox  // 文字の配列
    lines(): ArrayBox  // 行の配列
}
```

### Everything is Box との整合性
**完全に整合**

- すべてのコレクション操作がBox型を返す
- Lambda式（FunctionBox）との自然な統合
- メソッドチェーンによる fluent API

### 優先度
**High（Phase 20-21で実装）**

- **Phase 20**: ArrayBox標準メソッド拡張（filter/map/reduce/sort）
- **Phase 21**: StreamBox（遅延評価版）
- **Phase 22**: 標準ライブラリ全体への展開

---

## 機能17: Extension Methods（C#, Kotlin, Swift）

### 概要
既存の型にメソッドを追加。型定義を変更せずに機能拡張。

### 他言語での実装例

**C#:**
```csharp
// 拡張メソッド
public static class StringExtensions {
    public static bool IsEmail(this string str) {
        return str.Contains("@");
    }

    public static string Reverse(this string str) {
        char[] arr = str.ToCharArray();
        Array.Reverse(arr);
        return new string(arr);
    }
}

// 使用例
string email = "test@example.com";
bool valid = email.IsEmail();  // true
string reversed = "hello".Reverse();  // "olleh"
```

**Kotlin:**
```kotlin
// 拡張関数
fun String.isEmail(): Boolean {
    return this.contains("@")
}

fun <T> List<T>.secondOrNull(): T? {
    return if (this.size >= 2) this[1] else null
}

// 使用例
val valid = "test@example.com".isEmail()  // true
val second = listOf(1, 2, 3).secondOrNull()  // 2
```

### Hakoruneでの実現可能性
**コア拡張必要（または、Boxパターンで代替）**

現在のHakoruneでは：
- 継承・デリゲーション（`from`構文）で一部実現可能
- 完全な拡張メソッドはコア変更が必要

### Hakoruneに取り入れるべきか
**Partial（Boxラッパーパターンで代替）**

#### 理由
1. **型定義を変更せずに拡張** - 便利だが、複雑性も増す
2. **名前空間汚染の危険** - どこでもメソッド追加可能
3. **Boxパターンで代替可能** - ラッパーBoxで実現
4. **コア拡張のコストが高い** - 実装複雑度が増す

#### 取り入れる場合の実装方針

**Phase 1: 現状の代替（Boxラッパー）**
```hakorune
// 拡張ユーティリティBox
box StringUtils {
    static isEmail(str: StringBox): BoolBox {
        return str.indexOf("@") >= 0
    }

    static reverse(str: StringBox): StringBox {
        local result = ""
        local i = str.length() - 1
        loop(i >= 0) {
            result = result + str.charAt(i)
            i = i - 1
        }
        return result
    }
}

// 使用例
local valid = StringUtils.isEmail("test@example.com")
local reversed = StringUtils.reverse("hello")
```

**Phase 2: using拡張（名前空間インポート）**
```hakorune
// string_extensions.hako
flow StringExtensions {
    isEmail(str: StringBox): BoolBox {
        return str.indexOf("@") >= 0
    }

    reverse(str: StringBox): StringBox {
        // ...
    }
}

// main.hako
using string_extensions as StrExt

local valid = StrExt.isEmail("test@example.com")
```

**Phase 3: 真の拡張メソッド（将来検討、コア拡張必要）**
```hakorune
// extension構文（仮想的）
extension StringBox {
    isEmail(): BoolBox {
        return me.indexOf("@") >= 0
    }

    reverse(): StringBox {
        // ...
    }
}

// 使用例
local valid = "test@example.com".isEmail()
local reversed = "hello".reverse()
```

### Everything is Box との整合性
**一部不整合の可能性**

- 真の拡張メソッドは型システムの複雑化
- Boxラッパーパターンの方が「Everything is Box」と整合
- コア型（StringBox等）の拡張は慎重に検討

### 優先度
**Low（Boxラッパーで代替推奨）**

- 真の拡張メソッドはコア拡張コストが高い
- 現状のBoxパターン、using拡張で十分
- 将来的な検討課題（Phase 23+）

---

## 機能18: Record Types / Data Classes（C#, Kotlin, Python）

### 概要
イミュータブルなデータ構造。等価比較、コピー、デストラクチャリングを自動生成。

### 他言語での実装例

**C#:**
```csharp
// record型
public record Person(string Name, int Age);

// 使用例
var person1 = new Person("Alice", 30);
var person2 = person1 with { Age = 31 };  // コピー with 変更

bool equal = person1 == person2;  // 値による等価比較
```

**Kotlin:**
```kotlin
// data class
data class Person(val name: String, val age: Int)

// 自動生成される:
// - equals(), hashCode()
// - toString()
// - copy()
// - componentN() (デストラクチャリング)

val person1 = Person("Alice", 30)
val person2 = person1.copy(age = 31)

val (name, age) = person1  // デストラクチャリング
```

**Python:**
```python
from dataclasses import dataclass

@dataclass
class Person:
    name: str
    age: int

person1 = Person("Alice", 30)
person2 = Person("Alice", 30)
print(person1 == person2)  # True（値による比較）
```

### Hakoruneでの実現可能性
**マクロで実現可能**

既存機能：
- Box型（データ構造）
- @derive マクロ（Phase 16、部分実装済み）

### Hakoruneに取り入れるべきか
**Yes（高優先度、@dataマクロとして実装）**

#### 理由
1. **ボイラープレートコード削減** - equals/toString/copy自動生成
2. **データ中心設計の促進** - イミュータブルなデータ構造
3. **既存マクロシステムと整合** - @deriveの拡張
4. **初心者に優しい** - データクラスは直感的

#### 取り入れる場合の実装方針

**Phase 1: @data マクロ（Phase 20実装予定）**
```hakorune
// @data マクロ
@data
box Person {
    name: StringBox
    age: IntegerBox
}

// ↓ 展開後
box Person {
    name: StringBox
    age: IntegerBox

    birth(personName: StringBox, personAge: IntegerBox) {
        me.name = personName
        me.age = personAge
    }

    // equals()自動生成
    equals(other: Person): BoolBox {
        if other == null {
            return false
        }
        return me.name == other.name and me.age == other.age
    }

    // toString()自動生成
    toString(): StringBox {
        return "Person(name=" + me.name + ", age=" + me.age + ")"
    }

    // copy()自動生成
    copy(newName: StringBox = null, newAge: IntegerBox = null): Person {
        local name = if newName != null { newName } else { me.name }
        local age = if newAge != null { newAge } else { me.age }
        return new Person(name, age)
    }

    // hashCode()自動生成（将来）
    hashCode(): IntegerBox {
        return me.name.hashCode() * 31 + me.age.hashCode()
    }
}

// 使用例
local person1 = new Person("Alice", 30)
local person2 = person1.copy(newAge = 31)

console.log(person1.toString())  // "Person(name=Alice, age=30)"
console.log(person1.equals(person2))  // false
```

**Phase 2: イミュータビリティ強制（将来検討）**
```hakorune
// @immutable + @data
@immutable
@data
box Person {
    name: StringBox
    age: IntegerBox
}

// ↓ 展開後、フィールドへの代入を禁止
// person.name = "Bob"  // エラー: イミュータブルフィールド
```

**Phase 3: デストラクチャリング（将来検討）**
```hakorune
// パターンマッチングでのデストラクチャリング
local person = new Person("Alice", 30)

match person {
    Person(name, age) => {
        console.log("Name: " + name + ", Age: " + age)
    }
}

// または、let構文（仮想的）
let Person(name, age) = person
console.log(name)  // "Alice"
```

### Everything is Box との整合性
**完全に整合**

- データクラスは通常のBox型
- 自動生成メソッドもすべてBox型を返す
- イミュータビリティはフィールドレベルで制御

### 優先度
**High（Phase 20で@dataマクロ実装）**

- **@data マクロ**: equals/toString/copy自動生成
- **@immutable**: イミュータビリティ強制（Phase 21以降）
- **デストラクチャリング**: パターンマッチング拡張（Phase 22以降）

---

## 機能19: Named Arguments / Default Parameters（Python, Kotlin, Swift）

### 概要
関数呼び出し時に引数名を指定。デフォルト値の設定。

### 他言語での実装例

**Python:**
```python
# 名前付き引数とデフォルト値
def create_user(name, age=18, email=None, admin=False):
    pass

# 呼び出し方
create_user("Alice")
create_user("Bob", age=25)
create_user("Charlie", email="charlie@example.com", admin=True)
create_user(name="David", admin=True, age=30)  # 順序自由
```

**Kotlin:**
```kotlin
// デフォルト引数
fun createUser(
    name: String,
    age: Int = 18,
    email: String? = null,
    admin: Boolean = false
) {
    // ...
}

// 名前付き引数
createUser("Alice")
createUser("Bob", age = 25)
createUser(name = "Charlie", admin = true, email = "charlie@example.com")
```

### Hakoruneでの実現可能性
**コア拡張必要（または、Boxパターンで代替）**

現在のHakoruneでは：
- デフォルト引数：未実装
- 名前付き引数：未実装
- 代替：Builderパターン、ConfigBox

### Hakoruneに取り入れるべきか
**Partial（デフォルト引数のみ推奨、名前付き引数は低優先度）**

#### 理由
1. **デフォルト引数は価値あり** - オプショナルパラメータの簡潔化
2. **名前付き引数は過剰** - 可読性向上は限定的
3. **Builderパターンで代替可能** - より柔軟
4. **コア拡張のコストが高い** - 実装複雑度

#### 取り入れる場合の実装方針

**Phase 1: デフォルト引数（パーサー拡張）**
```hakorune
// デフォルト引数構文
function createUser(name: StringBox, age: IntegerBox = 18, admin: BoolBox = false) {
    console.log("User: " + name + ", Age: " + age + ", Admin: " + admin)
}

// 呼び出し
createUser("Alice")                    // age=18, admin=false（デフォルト）
createUser("Bob", 25)                  // admin=false（デフォルト）
createUser("Charlie", 30, true)        // すべて指定
```

**Phase 2: Builderパターン（現状の推奨代替）**
```hakorune
// ConfigBox + Builderパターン
box UserConfig {
    name: StringBox
    age: IntegerBox = 18
    email: StringBox = null
    admin: BoolBox = false

    birth() {
        // デフォルト値は上で設定済み
    }

    withName(userName: StringBox): UserConfig {
        me.name = userName
        return me
    }

    withAge(userAge: IntegerBox): UserConfig {
        me.age = userAge
        return me
    }

    withEmail(userEmail: StringBox): UserConfig {
        me.email = userEmail
        return me
    }

    withAdmin(isAdmin: BoolBox): UserConfig {
        me.admin = isAdmin
        return me
    }
}

function createUser(config: UserConfig) {
    console.log("User: " + config.name + ", Age: " + config.age)
}

// 使用例
local config = new UserConfig()
    .withName("Alice")
    .withAge(25)
    .withAdmin(true)

createUser(config)
```

**Phase 3: 名前付き引数（将来検討、低優先度）**
```hakorune
// 名前付き引数構文（仮想的）
function createUser(name: StringBox, age: IntegerBox = 18, admin: BoolBox = false) {
    // ...
}

// 呼び出し
createUser(name = "Alice")
createUser(name = "Bob", age = 25)
createUser(admin = true, name = "Charlie", age = 30)  // 順序自由
```

### Everything is Box との整合性
**整合性あり（Builderパターン推奨）**

- ConfigBoxは通常のBox型
- メソッドチェーンによる fluent API
- デフォルト値はフィールド初期化で実現

### 優先度
**Medium（デフォルト引数のみ、Phase 21以降）**

- **デフォルト引数**: パーサー拡張必要、Phase 21で検討
- **Builderパターン**: 現状の推奨代替、即座に使用可能
- **名前付き引数**: 低優先度、Phase 23+で検討

---

## 機能20: String Interpolation（Kotlin, Swift, JavaScript）

### 概要
文字列内に式を埋め込む。テンプレートリテラル。

### 他言語での実装例

**Kotlin:**
```kotlin
val name = "Alice"
val age = 30
val message = "Hello, $name! You are $age years old."
val complex = "Next year: ${age + 1}"
```

**Swift:**
```swift
let name = "Alice"
let age = 30
let message = "Hello, \(name)! You are \(age) years old."
let complex = "Next year: \(age + 1)"
```

**JavaScript:**
```javascript
const name = "Alice"
const age = 30
const message = `Hello, ${name}! You are ${age} years old.`
const complex = `Next year: ${age + 1}`
```

### Hakoruneでの実現可能性
**パーサー拡張で実現可能**

現状のHakoruneでは：
```hakorune
// 現状（文字列連結）
local name = "Alice"
local age = 30
local message = "Hello, " + name + "! You are " + age + " years old."
```

### Hakoruneに取り入れるべきか
**Yes（高優先度）**

#### 理由
1. **可読性の大幅向上** - 文字列連結の煩雑さ解消
2. **エラー削減** - スペースや句読点の漏れ防止
3. **広く普及した機能** - 多くの言語で標準
4. **実装コスト低い** - パーサー拡張のみ

#### 取り入れる場合の実装方針

**Phase 1: 基本的な補間（パーサー拡張）**
```hakorune
// ${}構文
local name = "Alice"
local age = 30
local message = "Hello, ${name}! You are ${age} years old."

// ↓ パーサーで展開
local message = "Hello, " + name + "! You are " + age + " years old."
```

**Phase 2: 式の埋め込み**
```hakorune
// 式を直接埋め込み
local x = 10
local y = 20
local result = "The sum is ${x + y}"

// メソッド呼び出し
local user = new User("Alice")
local greeting = "Hello, ${user.getName()}!"

// 三項演算子（将来）
local status = "Status: ${isActive ? "Active" : "Inactive"}"
```

**Phase 3: 複数行文字列との統合**
```hakorune
// 複数行文字列 + 補間
local template = """
    User Information:
    Name: ${user.name}
    Age: ${user.age}
    Email: ${user.email}
"""

// ヒアドキュメント風（将来検討）
local html = <<HTML
    <div>
        <h1>${title}</h1>
        <p>${content}</p>
    </div>
HTML
```

**Phase 4: フォーマット指定（将来検討）**
```hakorune
// フォーマット指定子（Python f-string風）
local pi = 3.14159
local formatted = "Pi: ${pi:.2f}"  // "Pi: 3.14"

local count = 42
local hex = "Hex: ${count:x}"  // "Hex: 2a"
```

### Everything is Box との整合性
**完全に整合**

- 埋め込まれた式はすべてBox型として評価
- StringBoxへの自動変換（`toString()`/`str()`呼び出し）
- 既存の`+`演算子（文字列連結）との整合性

### 優先度
**High（Phase 20-21で実装）**

- **Phase 20**: 基本的な`${}`補間
- **Phase 21**: 式の埋め込み、複数行文字列統合
- **Phase 22**: フォーマット指定（長期計画）

---

## 優先度マトリックス（全20機能まとめ）

### Critical（最優先、Phase 19-20実装中/計画中）
| 機能 | 言語 | 優先度 | 実装Phase | 理由 |
|------|------|--------|----------|------|
| **Discriminated Unions + Type Guards** | Rust, TypeScript | **Critical** | Phase 19-20 | 型安全性、パターンマッチング完成、Result/Option基盤 |

### High（高優先度、Phase 20-21実装推奨）
| 機能 | 言語 | 優先度 | 実装Phase | 理由 |
|------|------|--------|----------|------|
| **guard文** | Swift | **High** | Phase 20 | Fail-Fast哲学、可読性、ネスト回避 |
| **with文 / Context Manager** | Python, Kotlin | **High** | Phase 20-21 | リソース管理標準化、初心者に優しい |
| **Optional Chaining** | Swift, TypeScript | **High** | Phase 20-21 | null安全性、可読性向上 |
| **LINQ / Stream API** | C#, Java, Kotlin | **High** | Phase 20-21 | データ処理標準化、関数型プログラミング |
| **Record Types / Data Classes** | C#, Kotlin, Python | **High** | Phase 20 | ボイラープレート削減、@dataマクロ |
| **String Interpolation** | Kotlin, Swift, JS | **High** | Phase 20-21 | 可読性大幅向上、実装コスト低 |
| **Async/Await強化** | Rust, JS, C# | **High** | Phase 21-22 | FutureBox完全実装、並行処理標準化 |

### Medium（中優先度、Phase 21-22実装検討）
| 機能 | 言語 | 優先度 | 実装Phase | 理由 |
|------|------|--------|----------|------|
| **if let / while let** | Rust, Swift | **Medium** | Phase 20 | match式で代替可能だが、頻出パターン簡略化に価値 |
| **Nullish Coalescing** | TypeScript, Swift | **Medium** | Phase 20-21 | null/falsyの区別、Optional Chainingと相性 |
| **goroutine + channels** | Go | **Medium** | Phase 21-22 | ChannelBox実装、メッセージパッシング |
| **Actor Model / GenServer** | Elixir, Erlang | **Medium** | Phase 22 | ActorBoxパターン、並行処理の高度化 |
| **Default Parameters** | Python, Kotlin, Swift | **Medium** | Phase 21 | デフォルト引数のみ、名前付き引数は低優先度 |

### Low（低優先度、Phase 22以降または実装しない）
| 機能 | 言語 | 優先度 | 実装Phase | 理由 |
|------|------|--------|----------|------|
| **Scope Functions** | Kotlin | **Low** | Phase 22+ | withのみMedium、他は既存機能で代替可能 |
| **Property Wrappers** | Swift | **Low** | Phase 22+ | Unified Membersで大部分実現可能 |
| **Extension Methods** | C#, Kotlin, Swift | **Low** | Phase 23+ | Boxラッパーで代替推奨、コア拡張コスト高 |

### N/A（既存機能または実装しない）
| 機能 | 言語 | 優先度 | 理由 |
|------|------|--------|------|
| **defer文** | Go, Swift | **N/A** | cleanup構文で既に実装済み |
| **パイプ演算子** | Elixir, F# | **N/A** | Phase 12.7-Bで既に実装済み |
| **panic/recover** | Go | **N/A** | throw/catch/cleanupで十分 |
| **Walrus演算子** | Python | **None** | 可読性を損なう、実装しない |

---

## 総合推奨実装ロードマップ

### Phase 19-20（現在〜近未来）
```
1. @enum/@match Macros（進行中） ← 最優先
2. @guard マクロ
3. @with マクロ
4. @data マクロ
5. Optional Chaining（?.）
6. String Interpolation（${}）
7. ArrayBox標準メソッド拡張（filter/map/reduce）
```

### Phase 21（中期）
```
1. Nullish Coalescing（??）
2. Default Parameters
3. Async/Await完全実装（FutureBox強化）
4. ChannelBox実装
5. StreamBox（遅延評価版コレクション）
```

### Phase 22-23（長期）
```
1. Actor Model / GenServer（ActorBox）
2. Property Wrappers（高度版）
3. Extension Methods（検討）
4. 複数行文字列 + 補間統合
```

---

## Everything is Box 哲学との整合性評価

### 完全に整合（推奨）
- Discriminated Unions（VariantBox）
- guard文
- with文 / Context Manager
- Optional Chaining / Nullish Coalescing
- LINQ / Stream API
- Record Types / Data Classes
- String Interpolation
- Async/Await
- goroutine + channels（ChannelBox）
- Actor Model（ActorBox）

### 一部整合（条件付き推奨）
- if let / while let（match式優先、糖衣構文として）
- defer文（既存cleanup拡張）
- Scope Functions（withのみ）
- Property Wrappers（Unified Members拡張）
- Default Parameters（名前付き引数は非推奨）

### 不整合または過剰（非推奨）
- Walrus演算子（可読性低下）
- Extension Methods（Boxラッパー推奨）
- 完全なpanic/recover（既存機能で十分）

---

## 結論

### 即座に実装すべき機能（Phase 19-20）
1. **Discriminated Unions**（Phase 19-20、実装中）
2. **guard文**（@guardマクロ）
3. **with文**（@withマクロ）
4. **Optional Chaining**（?.演算子）
5. **String Interpolation**（${}構文）
6. **@dataマクロ**（Record Types）

### 中期的に実装すべき機能（Phase 21-22）
1. **Nullish Coalescing**（??演算子）
2. **LINQ / Stream API**（ArrayBox/StreamBox拡張）
3. **Async/Await完全実装**（FutureBox強化）
4. **ChannelBox**（メッセージパッシング）

### 長期的に検討すべき機能（Phase 22+）
1. **Actor Model**（ActorBox）
2. **Property Wrappers**（高度版）
3. **Extension Methods**（慎重に検討）

### 実装しない機能
1. **Walrus演算子**（可読性低下）
2. **完全なpanic/recover**（既存機能で十分）

---

**Hakoruneの設計思想「Everything is Box」「Fail-Fast」「最小コア + 糖衣構文 + マクロ」に完全に沿った形で、他言語の優れた機能を選択的に取り入れることを推奨します。**
