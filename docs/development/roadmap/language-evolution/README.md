# 🚀 Nyash/Hakorune 言語進化ロードマップ v2.0

**作成日**: 2025-10-02
**更新日**: 2025-10-02 - ChatGPT Pro + Claude 統合改良版
**対象**: Phase 16-30（言語機能・標準ライブラリの進化）
**関連**: [アーキテクチャ戦略](../architecture-strategy.md) - Rust vs セルフホスト実装戦略

---

## 📖 概要

Phase 15までの実装により、Nyash/Hakoruneは強力な基礎を確立しました。
このドキュメントは、**「コアは最小・糖衣は最強」**方針に基づく体系的な言語機能進化を定義します。

**実装方針**: [アーキテクチャ戦略](../architecture-strategy.md)に従い、Phase 19以降の新機能は**セルフホストのみ**で実装します。

---

## 🎯 **不変の設計原則（赤線）**

### **De-sugaring Contract**（最重要原則）

> **新構文は既存構文・既存Boxへ有限段で必ず落ちること。IR命令の追加は最後の手段。**

**具体的ルール**:
1. **MIR14は増やさない** - すべてデシュガリング/静的パス/標準ライブラリで実現
2. **Everything is Box/flow Main.main を維持** - Pulse は下限として利用
3. **Effects/Capabilities/Contracts は meta 側** - 言語仕様には持ち込まない
4. **例外は導入しない** - panicはVM/実装バグ用、通常はResultBoxで伝播
5. **dev→prod の挙動差は policy のみ** - warn/audit/enforce で制御

📖 **詳細**: [De-sugaring Contract](./desugaring-contract.md) に別途明記

---

## 📊 現在の強み（Phase 15-16時点）

### ✅ **言語機能の充実度**
1. **プロパティシステム** (stored/computed/once/birth_once) - **業界最先端レベル**
2. **🚀 マクロシステム (Phase 16実装中)** - @derive/@test等、Box-Based Macro革命
3. **postfix catch/cleanup** - 非常にモダン（Swift/Kotlin風）
4. **Result伝播 (? 演算子)** - Rust風エラー処理
5. **match式** - パターンマッチング
6. **Lambda式** - 高階関数サポート
7. **using/namespace** - モジュールシステム基礎
8. **flow** - stateless namespace（静的関数グループ化）
9. **非同期** (nowait/await/FutureBox) - 並行処理基礎
10. **演算子オーバーロード** - トレイトベース
11. **変数宣言厳密化** - メモリ・非同期安全性保証

### ✅ **Box型ライブラリ充実度**
30種類以上のBox（String/Integer/Array/Map/JSON/Regex/HTTP/GUI等）

### ✅ **コンパクトな実装**
- セルフホスティングコンパイラ: **3,771行**
- Rust実装: 48,344行
- **圧縮率: 約10.7%（約10分の1）**

---

## 🎯 進化計画（9の柱 + 糖衣5つ）

**注**: マクロシステム（旧🔟）はPhase 16で既に実装中のため、残り9項目 + 糖衣構文5つを進化計画とします。

---

### **1️⃣ 型システムの段階的強化（デシュガリング優先）**

**現状**: 型アノテーションはP0では無視、実行時型エラー

**Phase**: 20-25
**優先度**: 🟡 中
**実装方針**: **完全デシュガリング** - 新しいMIR命令は追加しない

#### **Phase 20: Optional/Union型（純糖衣）**
```nyash
// T? → OptionBox<T> へデシュガリング
box Container {
    value: IntegerBox?  // 糖衣構文

    get(): IntegerBox? {  // 戻り値も糖衣
        return me.value
    }
}

// ↓ デシュガリング後
box Container {
    value: OptionBox<IntegerBox>

    get(): OptionBox<IntegerBox> {
        return me.value
    }
}

// Union型: A|B → SumBox<A,B> へデシュガリング
type Result = OkBox | ErrorBox  // 糖衣構文
// ↓
type Result = SumBox<OkBox, ErrorBox>
```

**match網羅性チェック**: 静的パス（コンパイラ検査）で提供
**実装**: 既存のOptionBox/SumBoxを活用（新しいBox不要）

#### **Phase 22: ジェネリクス（モノモーフィック展開）**
```nyash
// Phase A: マクロ的モノモーフィック展開から開始
box Container<T> {
    items: ArrayBox<T>

    add(item: T) {
        me.items.push(item)
    }

    get(index: IntegerBox): T? {
        return me.items.get(index)
    }
}

// 実体化: 型パラメータごとにコード生成
// Container<IntegerBox> → Container_IntegerBox
// Container<StringBox> → Container_StringBox
```

**制約**: 最初は単純型パラメータのみ（高度な制約は後回し）

#### **Phase 24: Intersection型（型検査のみ）**
```nyash
// Intersection: 型検査側の"特性集合"のみ
type Loggable = ConsoleBox & FileBox  // capability の交差

// ランタイム表現は作らない（静的検査のみ）
```

**やらないこと**: HM型推論、row多相（コスト > 便益）

**実装ステップ**:
1. Phase 20: Optional/Union型糖衣 + 網羅性チェック
2. Phase 22: ジェネリクス基礎（モノモーフィック）
3. Phase 24: Intersection型（静的検査のみ）

---

### **2️⃣ エラーハンドリング（現状で十分・糖衣のみ）**

**現状**: ResultBox, postfix catch/cleanup（基礎は良い）

**Phase**: 26-28
**優先度**: 🟢 低（現状で十分実用的）

**追加する糖衣（2つのみ）**:
```nyash
// 1. try-else 糖衣 → Result連鎖へデシュガリング
try {
    dangerousOperation()
} else { e ->
    handleError(e)
}

// ↓ デシュガリング後
dangerousOperation()
    .catch(fn(e) { handleError(e) })

// 2. mapError → 標準ライブラリで提供（構文は増やさない）
result.mapError(fn(e) { new AppError(e) })
```

**やらないこと**:
- カスタムエラー型階層（Boxで実現可能）
- panic/recover（現状のpanicで十分）

---

### **3️⃣ テストフレームワーク統合（最優先！）** 🔴

**現状**: テスト機能なし（**最大の欠落！**）

**Phase**: 16.1-17
**優先度**: 🔴 最高（開発体験向上の鍵！言語採用の決定要因）

**実装方針**: **すべてマクロで実装**（MIR命令増やさない）

#### **Phase 16.1: 基本テストフレームワーク**
```nyash
// @test/@describe/@it マクロ実装
@test
describe("Calculator") {
    @test
    it("should add numbers") {
        local calc = new Calculator()
        expect(calc.add(2, 3)).toBe(5)
        assert(calc.value == 0)
    }

    @test
    it("should handle errors") {
        expectThrow(fn() { calc.divide(1, 0) })
    }
}
```

**実体**: TestRunnerBox（標準ライブラリ）

#### **Phase 16.2: アサーション拡張**
```nyash
expect(value).toBe(expected)         // 厳密等価
expect(value).toEqual(expected)      // 深い等価
expect(value).toThrow()              // 例外期待
expect(value).toBeGreaterThan(n)     // 比較
expect(value).toContain(item)        // 含有
```

#### **Phase 16.3: Benchmark統合** 🎯
```nyash
@bench(iterations: 1000)
benchmark_fibonacci() {
    fibonacci(20)
}

// 実行: hako bench
// 出力: 平均実行時間、メモリ使用量等
```

**実体**: BenchmarkBox（標準ライブラリ） + `hako bench` コマンド

#### **Phase 16.4 または 17.1: Property-based testing**
```nyash
@property_test(iterations: 1000)
test_commutative_addition() forall(a: IntegerBox, b: IntegerBox) {
    expect(a + b).toBe(b + a)
}
```

**生成器**: GenBox<T>（シード固定可）

#### **Phase 17: Snapshot/Mock**
```nyash
// Snapshot testing
@test
test_output() {
    local result = generateReport()
    snapshot("report_v1", result)  // JSON保存、再現可能
}

// Mock
@mock
box MockHttpClient from HttpClient {
    override get(url) {
        return new MockResponse(200, "OK")
    }
}
```

**重要性**:
- セルフホスティング進行中の**今こそ**導入すべき
- TDD開発体験が言語採用の鍵
- Benchmark統合で性能回帰を防止

---

### **4️⃣ 並行処理の完成度向上（糖衣100%）**

**現状**: nowait/await/FutureBox（基礎のみ）

**Phase**: 21-25
**優先度**: 🟡 中

**実装方針**: **構文は最小、すべて標準ライブラリで実現**

#### **Phase 21: ChannelBox実装**
```nyash
// 標準ライブラリ実装（新構文なし）
using std.sync

local ch = new ChannelBox<IntegerBox>()
nowait producer = loop(true) {
    ch.send(randomInt())
}
local value = ch.receive()  // await不要
```

#### **Phase 22: 構造化並行性（糖衣）**
```nyash
// async{} → TaskGroupBox へデシュガリング
async {
    nowait task1 = longTask1()
    nowait task2 = longTask2()

    // スコープ終了で自動待機
    // エラー時は両方キャンセル
}

// ↓ デシュガリング後
local group = new TaskGroupBox()
group.spawn(fn() { longTask1() })
group.spawn(fn() { longTask2() })
group.await_all()  // スコープ終了時自動
```

#### **Phase 23: Select式（API合成）**
```nyash
// select{} → SelectBox へデシュガリング
match select {
    ch1.receive() => { handle1() }
    ch2.receive() => { handle2() }
    timeout(1000) => { handleTimeout() }
}

// ↓ デシュガリング後
local sel = new SelectBox()
sel.on(ch1, fn(v) { handle1() })
sel.on(ch2, fn(v) { handle2() })
sel.on_timeout(1000, fn() { handleTimeout() })
sel.run()
```

#### **Phase 24: CancelTokenBox標準化**
```nyash
// 標準ライブラリ実装
local token = new CancelTokenBox()
nowait task = longTask(token)
token.cancel()  // タスク中断
```

**実装ステップ**:
1. Phase 21: ChannelBox実装（標準ライブラリ）
2. Phase 22: TaskGroupBox + `async{}` 糖衣
3. Phase 23: SelectBox + `select{}` 糖衣
4. Phase 24: CancelTokenBox標準化

---

### **5️⃣ メモリ管理の可視性向上（メソッド化）**

**現状**: Arc<Mutex>暗黙共有、outbox/weak未活用

**Phase**: 28-30
**優先度**: 🟢 低（現状で安全）

**実装方針**: **キーワードにしない、メソッドで提供**

```nyash
// ❌ キーワード版（採用しない）
outbox result = expensiveComputation()
weak reference: ChildBox

// ✅ メソッド版（採用）
local result = expensiveComputation()
useResult(result.move())  // 所有権移転

local ref = original.share()   // Arc参照増加
local weak = original.weak()    // 弱参照取得

local copy = original.clone()  // 深いコピー
```

**注釈（meta）**:
```nyash
// unique ヒントはmeta（AOT最適化用）
// 言語機能にはしない
@unique
local exclusive = createResource()
```

---

### **6️⃣ 標準ライブラリの体系化**

**現状**: 豊富だが命名・構造が不統一

**Phase**: 16-20
**優先度**: 🟡 中（早期実施推奨）

**実装方針**: Rust std風階層 + 命名統一

#### **Phase 16: 命名規則統一計画策定**
```nyash
// ❌ Before（不統一）
array.length()    // ArrayBox
string.len()      // StringBox
map.size()        // MapBox

// ✅ After（統一）
array.len()       // すべて len() に統一
string.len()
map.len()
```

#### **Phase 17-20: 階層構造実装**
```nyash
// Rust std風階層構造
using std.collections  // ArrayBox, MapBox, SetBox, QueueBox
using std.io           // FileBox, StreamBox, BufferBox
using std.net          // HttpClientBox, TcpBox, UdpBox
using std.sync         // MutexBox, ChannelBox, AtomicBox
using std.time         // TimeBox, DurationBox, TimerBox
using std.json         // JSONBox（parse/stringify/schema）
using std.regex        // RegexBox
using std.math         // MathBox
using std.console      // ConsoleBox
using std.effect       // 🆕 効果のNo-op/記録デバイス
using std.profile      // 🆕 軽量プロファイルAPI

// Prelude自動インポート（最小限）
// StringBox, IntegerBox, ArrayBox, MapBox, ResultBox, OptionBox
```

**実装ステップ**:
1. Phase 16: 命名規則統一計画策定
2. Phase 17: std.collections体系化 + 命名統一実施
3. Phase 18: std.io/net/sync体系化
4. Phase 19-20: その他標準ライブラリ体系化 + std.effect/profile追加

---

### **7️⃣ デバッグ・診断機能強化**

**現状**: DebugBox、基本的なprint

**Phase**: 18-20
**優先度**: 🟡 中（開発体験向上）

**実装方針**: **すべてマクロまたはBox化**

#### **Phase 18: repr()実装**
```nyash
// repr() → @derive(Debug) マクロと連携
@derive(Debug)
box Person {
    name: StringBox
    age: IntegerBox
}

print(str(person))   // "Person instance"
print(repr(person))  // "Person { name: "Alice", age: 25 }"
```

**実装**: `@derive(Debug)` マクロ（Phase 16実装済み）と連携

#### **Phase 19: スタックトレース改善**
```nyash
// スタックトレースはBox化
try {
    deepFunction()
} catch(e) {
    print(e.stack())  // ErrorBox.stack() メソッド
}
```

**実装**: VM側で改善、ErrorBoxにメソッド追加

#### **Phase 20: プロファイラ統合**
```nyash
// @profile マクロ → std.profile API注入
@profile
calculate() {
    // 実行時間・メモリ使用量を自動計測
}

// ↓ デシュガリング後
calculate() {
    local _prof = ProfileBox::start("calculate")
    // 元の処理
    _prof.end()
}
```

**実装**: `@profile` マクロ + std.profile（標準ライブラリ）

**アサーション強化**:
```nyash
assert(value > 0, "value must be positive")
debug_assert(internal.isValid())  // devのみ（policy制御）
static_assert(SIZE == 64, "size must be 64")  // コンパイル時
```

**実装ステップ**:
1. Phase 18: repr() + @derive(Debug)連携
2. Phase 19: スタックトレース改善（ErrorBox強化）
3. Phase 20: @profile マクロ + アサーション強化

---

### **8️⃣ ドキュメント生成機能**

**現状**: ドキュメント生成機能なし

**Phase**: 25-30
**優先度**: 🟢 低

**実装方針**: **ツール側で処理**（言語拡張は不要）

```nyash
/// Calculator Box provides basic arithmetic operations.
///
/// # Examples
/// ```nyash
/// local calc = new Calculator()
/// calc.add(2, 3)  // => 5
/// ```
box Calculator {
    /// The current calculation result
    result: IntegerBox

    /// Adds two numbers
    /// # Arguments
    /// - `a`: First number
    /// - `b`: Second number
    /// # Returns
    /// Sum of a and b
    add(a: IntegerBox, b: IntegerBox): IntegerBox {
        return a + b
    }
}
```

**コマンド**:
```bash
$ hako doc --generate  # JSON IR → HTML 二段階生成
```

**二段階出力の利点**: テーマ差し替えが容易

---

### **9️⃣ パッケージマネージャ完成（段階導入）**

**現状**: using system基礎実装

**Phase**: 18-20
**優先度**: 🟡 中

**実装方針**: **段階導入** - git直参照 → index-less publish → レジストリ

#### **Phase 18: git直参照 + lock.hako**
```toml
# hako.toml（Cargo風）
[package]
name = "my-app"
version = "0.1.0"
authors = ["Me"]

[dependencies]
http-client = { git = "https://github.com/user/http-client", tag = "v1.2.0" }
json-parser = { git = "https://github.com/user/json-parser", branch = "main" }

[dev-dependencies]
test-framework = { git = "https://github.com/hakorune/test-framework", tag = "v0.5.0" }
```

**lock.hako**: deterministic ビルド保証

#### **Phase 19: index-less publish**
```bash
$ hako publish  # GitHub Releases等に直接公開
```

**レジストリ不要**: git tag ベースで依存関係解決

#### **Phase 20: レジストリ（必要が生じたら）**
```toml
[dependencies]
http-client = "1.2.0"  # レジストリから取得
```

**コマンド**:
```bash
$ hako new my-project
$ hako build
$ hako test
$ hako bench  # 🎯 最初から統合！
$ hako run
$ hako publish
```

**実装ステップ**:
1. Phase 18: git直参照 + lock.hako（deterministic）
2. Phase 19: index-less publish（GitHub Releases）
3. Phase 20: レジストリ（必要なら）

---

### **🔟 マクロシステム拡張**

**現状**: ✅ **Phase 16で実装中** - Box-Based Macro革命

**実装済み**:
- @derive(Equals, ToString, Clone, Debug)
- @test マクロ + テストランナー
- AST Pattern Matching基盤
- Quote/Unquote システム
- HIRパッチ式マクロエンジン

📖 **詳細**: [Phase 16 Macro Revolution](../phases/phase-16-macro-revolution/README.md)

**Phase 17-20: 拡張**:
```nyash
// さらなる@deriveトレイト追加
@derive(Hash, Ord, Default, Serialize)
box MyBox { }

// カスタムマクロ定義（限定版）
macro benchmark(iterations) {
    // パフォーマンス計測マクロ
}

// comptime計算（限定版・安全領域のみ）
const CONFIG = comptime {
    readFile("config.toml").parse()  // ビルド時評価
}
```

**制約**: Turing完全にしない（安全領域のみ）

**Phase**: 16（実装中）→ 17-20（拡張）
**優先度**: 🟡 中（コア実装済み、拡張は要望次第）

---

## 🍬 **糖衣構文5つ（追加提案）**

### **A. パイプライン演算子 `|>`** 🎯

**Phase**: 17-18（テストの次）
**優先度**: 🔴 高（発見性問題の解決にも効く！）

```nyash
// 読みやすさ爆増！
value |> f(_) |> g(_, 42) |> h(_)

// ↓ デシュガリング後
h(g(f(value), 42))

// 実用例
[1, 2, 3, 4, 5]
    |> map(_, fn(x) { x * 2 })
    |> filter(_, fn(x) { x % 3 == 0 })
    |> reduce(_, 0, fn(acc, x) { acc + x })
```

**効果**: Cookbook/Recipe集の可読性向上、if連鎖問題の緩和

---

### **B. 名前付き引数**

**Phase**: 18-19
**優先度**: 🟡 中

```nyash
// 可読性UP
download(url: u, timeout: 3.s, retry: 5)

// ↓ デシュガリング後（位置引数 + デフォルト補完）
download(u, 3.s, 5)

// デフォルト引数との相性良い
download(url: u)  // timeout, retry はデフォルト値
```

---

### **C. `with capability` スコープ（視覚糖衣）**

**Phase**: 20-22
**優先度**: 🟢 低（開発体験向上）

```nyash
// 効果システムを言語機能にしない賢い方法
with net.out, fs.read {
    http.get("https://api.example.com")
    file.read("config.txt")
}

// ↓ 実際は何も変えない（ログに印を出すだけ）
// metaレイヤーで処理
```

**効果**: 視覚的な開発体験向上、capability明示

---

### **D. `comptime`（限定版）**

**Phase**: 22-25
**優先度**: 🟡 中

```nyash
// ビルド時評価（限定版）
const CONFIG = comptime {
    readFile("config.toml").parse()
}

// 定数埋め込み
const VERSION = comptime { "1.0.0" }
```

**制約**:
- Turing完全にしない
- 読み込みと定数計算のみ
- YAGNI原則遵守

---

### **E. パターン別名（Pattern Synonyms）**

**Phase**: 23-25
**優先度**: 🟢 低

```nyash
// パターン別名で可読性UP
pattern Some(x) = OptionBox::Some(x)
pattern None    = OptionBox::None

match value {
    Some(x) => process(x)
    None    => handleError()
}
```

---

## 🎯 **デシュガリング規則一覧**

| 構文 | デシュガリング後 | Phase |
|------|----------------|-------|
| `T?` | `OptionBox<T>` | 20 |
| `A\|B` | `SumBox<A,B>` | 20 |
| `x \|> f(_)` | `f(x)` | 17-18 |
| `f(a:1, b:2)` | `f(1,2)` | 18-19 |
| `async { body }` | `TaskGroupBox::scoped(fn() { ... })` | 22 |
| `select { a=>x; b=>y }` | `SelectBox::new().on(a, fn(v){x}).on(b, fn(v){y}).run()` | 23 |
| `@test it("..."){...}` | `TestRunnerBox::register(fn(){...})` | 16.1 |
| `@bench(iter:N) { ... }` | ベンチハーネス呼び出し | 16.3 |
| `@profile { ... }` | `ProfileBox::start()` + 処理 + `.end()` | 20 |
| `repr(obj)` | `@derive(Debug)` マクロ連携 | 18 |
| `x.move()` | 所有権移転メソッド | 28 |
| `x.share()` | Arc参照増加メソッド | 28 |
| `x.weak()` | 弱参照取得メソッド | 28 |

---

## 📅 実装タイムライン（優先順位付き）

### **Phase 16-17: 基礎固め + 発見性問題解決** 🔴

**Phase 16（現在）**:
1. 🚀 **マクロシステム完成** - @derive/@test完全動作
   - 16.1: 基本テストフレームワーク（describe/it/expect）
   - 16.2: アサーション拡張（toBe/toEqual/toThrow）
   - 16.3: **Benchmark統合** (`@bench`, `hako bench`)
   - 16.4: Property-based testing（GenBox）
2. 🟡 **標準ライブラリ命名規則統一計画策定**

**Phase 17（次の大目標）**:
3. 🔴 **Cookbook/Recipe集作成** - [Discoverability問題](./discoverability-analysis.md)解決の要
   - `docs/cookbook/patterns/` - ルックアップテーブル、文字列解析等
   - `docs/cookbook/anti-patterns/` - if連鎖のアンチパターン等
   - `docs/cookbook/refactoring/` - if→match変換等
4. 🔴 **Linter基礎実装** - Anti-Pattern検出開始
5. 🟡 **パイプライン演算子 `|>`** - 可読性向上
6. 🟡 **std.collections体系化 + 命名統一実施** (`len()` 統一)
7. 🟢 **Snapshot/Mock機能** - テスト完成

### **Phase 18-20: 開発体験向上** 🟡

**Phase 18**:
8. 🟡 **名前付き引数**
9. 🟡 **repr() + @derive(Debug)連携**
10. 🟡 **std.io/net/sync体系化**
11. 🟡 **パッケージマネージャ（git直参照 + lock.hako）**

**Phase 19**:
12. 🟡 **スタックトレース改善**（ErrorBox強化）
13. 🟡 **Linter拡充** - 自動リファクタリング提案
14. 🟡 **パッケージマネージャ（index-less publish）**
15. 🟡 **その他標準ライブラリ体系化** + std.effect/profile追加

**Phase 20**:
16. 🟡 **@profile マクロ + アサーション強化**
17. 🟡 **Optional/Union型糖衣** (`T?`, `A|B`)
18. 🟡 **パッケージマネージャ完成**（必要ならレジストリ）

### **Phase 21-25: 高度機能** 🟡

**Phase 21-24**:
19. 🟡 **並行処理完成**
    - 21: ChannelBox
    - 22: TaskGroupBox + `async{}` 糖衣 + ジェネリクス基礎
    - 23: SelectBox + `select{}` 糖衣 + パターン別名
    - 24: CancelTokenBox + Intersection型（静的検査のみ）
20. 🟢 **`with capability` スコープ**（視覚糖衣）
21. 🟢 **`comptime`（限定版）**

**Phase 25**:
22. 🟡 **ドキュメント生成** (`hako doc`)

### **Phase 26-30: 洗練** 🟢

23. 🟢 **エラーハンドリング糖衣** (`try-else`, `mapError`)
24. 🟢 **メモリ管理メソッド** (`move()`, `share()`, `weak()`)
25. 🟢 **マクロシステム拡張** (@derive拡充、カスタムマクロ)

---

## 💡 設計原則の再確認

### **1. De-sugaring Contract 遵守**
- 新構文は既存構文・既存Boxへ有限段で必ず落ちる
- MIR14命令セットは増やさない

### **2. Everything is Box哲学の維持**
- すべての新機能はBox化して提供
- 一貫性を最優先

### **3. セルフホスティング優先**
- Rust実装で先行実装 → セルフホストで再実装
- セルフホストでの実装容易性を考慮した設計

### **4. YAGNI原則遵守**
- マクロシステムは実際の要望が出てから拡張
- 型システム強化はopt-in（段階的導入）
- メモリ管理可視性は現状で十分安全

### **5. 後方互換性**
- 既存コードを壊さない
- 新機能はopt-inまたは新構文で提供

---

## 🎊 まとめ

### **Phase 16-17: テスト+発見性問題解決**
✅ **@test/@bench/@derive完全動作** - Box-Based Macro革命完成
✅ **Cookbook/Recipe集 + Linter** - 発見性問題の根本解決
✅ **パイプライン演算子 `|>`** - 可読性爆増
✅ **標準ライブラリ命名統一** - 開発体験の一貫性向上

### **現在の設計の優秀さ**
- **プロパティシステム**（once/birth_once）は**業界最先端レベル**
- **マクロシステム**（Phase 16実装中）はBox-Based設計で革新的
- **postfix catch/cleanup**は非常に直感的
- **Everything is Box哲学**が一貫している
- **コンパクトな実装**（Rust実装の約10分の1）

### **長期ビジョン**
Phase 16-30を通じて、実用的で表現力豊かな言語へと進化。
**マクロシステム・プロパティシステム・型システムの三位一体**で、
**「コアは最小・糖衣は最強」** - 次世代言語の標準を打ち立てる。

---

**作成者**: Claude Sonnet 4.5 + ChatGPT Pro
**ベース**:
- [language-evolution.md v1](./v1-original.md) - Claude初版
- ChatGPT Pro深層分析・改良提案
- [Discoverability問題分析](./discoverability-analysis.md)
- [De-sugaring Contract](./desugaring-contract.md)

**更新履歴**:
- 2025-10-02: Claude初版作成
- 2025-10-02: ChatGPT Pro + Claude 統合改良版作成
