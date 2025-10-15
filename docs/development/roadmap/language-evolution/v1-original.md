# 🚀 Nyash/Hakorune 言語進化ロードマップ

**作成日**: 2025-10-02
**対象**: Phase 16-30（言語機能・標準ライブラリの進化）
**関連**: [アーキテクチャ戦略](./architecture-strategy.md) - Rust vs セルフホスト実装戦略

---

## 📖 概要

Phase 15までの実装により、Nyash/Hakoruneは強力な基礎を確立しました。
このドキュメントは、糖衣構文・マクロ以外の体系的な言語機能進化を定義します。

**実装方針**: [アーキテクチャ戦略](./architecture-strategy.md)に従い、Phase 19以降の新機能は**セルフホストのみ**で実装します。

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
- **圧縮率: 約7.8%（13分の1）**

---

## 🎯 進化計画（9の柱）

**注**: マクロシステム（旧🔟）はPhase 16で既に実装中のため、残り9項目を進化計画とします。

### **1️⃣ 型システムの段階的強化**

**現状**: 型アノテーションはP0では無視、実行時型エラー

**Phase**: 20-25
**優先度**: 🟡 中

**改善案**:
```nyash
// Phase A: 静的型チェック（opt-in）
@strict_types
box Calculator {
    value: IntegerBox  // 型検査ON

    add(a: IntegerBox, b: IntegerBox): IntegerBox {
        return a + b  // 型不一致はコンパイルエラー
    }
}

// Phase B: ジェネリクス/型パラメータ
box Container<T> {
    items: ArrayBox<T>

    add(item: T) {
        me.items.push(item)
    }

    get(index: IntegerBox): T? {  // Optional型
        return me.items.get(index)
    }
}

// Phase C: Union型・Intersection型
type Result = OkBox | ErrorBox
type Loggable = ConsoleBox & FileBox
```

**実装ステップ**:
1. Phase 20: 基本型チェック（opt-in）
2. Phase 22: ジェネリクス基礎
3. Phase 24: Union/Intersection型

---

### **2️⃣ エラーハンドリングの体系化**

**現状**: ResultBox, postfix catch/cleanup（基礎は良い）

**Phase**: 26-28
**優先度**: 🟢 低（現状で十分実用的）

**改善案**:
```nyash
// カスタムエラー型階層
box NetworkError from ErrorBox {
    code: IntegerBox
    message: StringBox
}

box TimeoutError from NetworkError { }
box ConnectionError from NetworkError { }

// エラー変換チェーン
fetchData()
    .mapError(fn(e) { new AppError(e) })
    catch(NetworkError e) {
        retry()
    }
    catch(TimeoutError e) {
        useCache()
    }
    cleanup {
        closeConnection()
    }

// panic/recover明確化
panic("critical error")  // プログラム停止
recover(fn(e) { log(e); return defaultValue() })  // panic回復
```

---

### **3️⃣ テストフレームワーク統合** 🔴

**現状**: テスト機能なし（**最大の欠落！**）

**Phase**: 16-17
**優先度**: 🔴 最高（開発体験向上の鍵！）

**改善案**:
```nyash
// ビルトインテスト構文
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

// モック・スタブ
@mock
box MockHttpClient from HttpClient {
    override get(url) {
        return new MockResponse(200, "OK")
    }
}

// プロパティベーステスト
@property_test(iterations: 1000)
it("commutative addition") forall(a: IntegerBox, b: IntegerBox) {
    expect(a + b).toBe(b + a)
}
```

**実装ステップ**:
1. Phase 16.1: 基本テストフレームワーク（describe/it/expect）
2. Phase 16.2: アサーション拡張（toBe/toEqual/toThrow等）
3. Phase 16.3: モック・スタブ機能
4. Phase 17: プロパティベーステスト

**重要性**:
- セルフホスティング進行中の**今こそ**導入すべき
- Rust実装で先行実装 → セルフホストで再実装が理想
- TDD開発体験が言語採用の鍵

---

### **4️⃣ 並行処理の完成度向上**

**現状**: nowait/await/FutureBox（基礎のみ）

**Phase**: 21-25
**優先度**: 🟡 中

**改善案**:
```nyash
// チャネル/メッセージパッシング
local ch = new ChannelBox<IntegerBox>()
nowait producer = loop() {
    ch.send(randomInt())
}
local value = ch.receive()  // await不要

// 構造化並行性（Structured Concurrency）
async {
    nowait task1 = longTask1()
    nowait task2 = longTask2()

    // すべて完了するまで待機（スコープ終了で自動）
    // task1/task2のどちらかがエラー → 両方キャンセル
}

// Select式（複数Future待機）
match select {
    ch1.receive() => { handle1() }
    ch2.receive() => { handle2() }
    timeout(1000) => { handleTimeout() }
}

// キャンセルトークン標準化
local token = new CancelTokenBox()
nowait task = longTask(token)
token.cancel()  // タスク中断
```

**実装ステップ**:
1. Phase 21: ChannelBox実装
2. Phase 22: 構造化並行性（async block）
3. Phase 23: Select式
4. Phase 24: キャンセルトークン標準化

---

### **5️⃣ メモリ管理の可視性向上**

**現状**: Arc<Mutex>暗黙共有、outbox/weak未活用

**Phase**: 28-30
**優先度**: 🟢 低（現状で安全）

**改善案**:
```nyash
// 所有権移転の明示
outbox result = expensiveComputation()  // moveセマンティクス
useResult(result)  // resultは無効化

// 弱参照の標準化
box Parent {
    weak child: ChildBox  // 循環参照防止
}

// ライフタイムヒント（optional）
box Borrower<'a> {
    reference: &'a StringBox  // 借用明示
}

// クローン明示化
local copy = original.clone()  // 深いコピー
local ref = original.share()   // Arc参照増加
```

---

### **6️⃣ 標準ライブラリの体系化**

**現状**: 豊富だが命名・構造が不統一

**Phase**: 16-20
**優先度**: 🟡 中

**改善案**:
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

// 命名規則統一
// ❌ Before: array.length(), string.len(), map.size()
// ✅ After:  collection.len() （統一）

// Prelude自動インポート
// StringBox, IntegerBox, ArrayBox等は自動利用可能
```

**実装ステップ**:
1. Phase 16: 命名規則統一計画策定
2. Phase 17: std.collections体系化
3. Phase 18: std.io/net/sync体系化
4. Phase 19-20: その他標準ライブラリ体系化

---

### **7️⃣ デバッグ・診断機能強化**

**現状**: DebugBox、基本的なprint

**Phase**: 18-20
**優先度**: 🟡 中（開発体験向上）

**改善案**:
```nyash
// repr()関数実装（構造的表示）
print(str(obj))   // "MyBox instance"
print(repr(obj))  // "MyBox { field1: 42, field2: "test" }"

// スタックトレース改善
try {
    deepFunction()
} catch(e) {
    print(e.stackTrace())  // 完全なコールスタック
}

// プロファイラ統合
@profile
calculate() {
    // 実行時間・メモリ使用量を自動計測
}

// アサーション強化
assert(value > 0, "value must be positive")
debug_assert(internalState.isValid())  // devのみ
static_assert(SIZE == 64, "size must be 64")  // コンパイル時
```

**実装ステップ**:
1. Phase 18: repr()関数実装
2. Phase 19: スタックトレース改善
3. Phase 20: プロファイラ統合・アサーション強化

---

### **8️⃣ ドキュメント生成機能**

**現状**: ドキュメント生成機能なし

**Phase**: 30+
**優先度**: 🟢 低

**改善案**:
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

// コマンド
$ hakorune doc --generate  # HTML/Markdown生成
```

---

### **9️⃣ パッケージマネージャ完成**

**現状**: using system基礎実装

**Phase**: 18-20
**優先度**: 🟡 中

**改善案**:
```toml
# hako.toml（Cargo風）
[package]
name = "my-app"
version = "0.1.0"
authors = ["Me"]

[dependencies]
http-client = "1.2.0"
json-parser = "2.0.1"

[dev-dependencies]
test-framework = "0.5.0"
```

```bash
# コマンド
$ hako new my-project
$ hako build
$ hako test
$ hako run
$ hako publish
```

**実装ステップ**:
1. Phase 18: hako.toml依存関係解決
2. Phase 19: パッケージレジストリ設計
3. Phase 20: hako CLIコマンド実装

---

### **🔟 マクロシステム拡張**

**現状**: ✅ **Phase 16で実装中** - Box-Based Macro革命

**実装済み**:
- @derive(Equals, ToString, Clone, Debug)
- @test マクロ + テストランナー
- AST Pattern Matching基盤
- Quote/Unquote システム
- HIRパッチ式マクロエンジン

📖 **詳細**: [Phase 16 Macro Revolution](./phases/phase-16-macro-revolution/README.md)

**今後の拡張**:
```nyash
// さらなる@deriveトレイト追加
@derive(Hash, Ord, Default, Serialize)
box MyBox { }

// カスタムマクロ定義
macro benchmark(iterations) {
    // パフォーマンス計測マクロ
}

// comptime計算拡張
const CONFIG = comptime {
    readFile("config.toml").parse()
}
```

**Phase**: 16（実装中）→ 17-20（拡張）
**優先度**: 🟡 中（コア実装済み、拡張は要望次第）

---

## 📅 実装タイムライン

### **Phase 16-17: 基礎固め**（最優先）
1. 🚀 **マクロシステム完成** - @derive/@test完全動作（Phase 16実装中）
2. 🟡 **標準ライブラリ体系化開始** - 命名規則統一

### **Phase 18-20: 開発体験向上**
3. 🟡 **デバッグ機能強化** - repr/スタックトレース
4. 🟡 **パッケージマネージャ完成** - 依存関係管理
5. 🟡 **標準ライブラリ体系化完成**

### **Phase 21-25: 高度機能**
6. 🟡 **並行処理完成** - チャネル/構造化並行性
7. 🟡 **型システム強化** - 静的型チェック opt-in/ジェネリクス

### **Phase 26-30: 洗練**
8. 🟢 **エラー体系化**
9. 🟢 **メモリ管理可視性**
10. 🟢 **ドキュメント生成**

---

## 💡 重要な設計原則

### **1. YAGNI原則遵守**
- マクロシステムは実際の要望が出てから
- 型システム強化はopt-in（段階的導入）
- メモリ管理可視性は現状で十分安全

### **2. Everything is Box哲学の維持**
- すべての新機能はBox化して提供
- 一貫性を最優先

### **3. セルフホスティング優先**
- Rust実装で先行実装 → セルフホストで再実装
- セルフホストでの実装容易性を考慮した設計

### **4. 後方互換性**
- 既存コードを壊さない
- 新機能はopt-inまたは新構文で提供

---

## 🎊 まとめ

### **Phase 16: マクロシステム実装中**
✅ @derive/@test等のBox-Based Macro革命が進行中！
セルフホスティング進行中の今、言語機能が加速度的に進化している。

### **現在の設計の優秀さ**
- **プロパティシステム**（once/birth_once）は**業界最先端レベル**
- **マクロシステム**（Phase 16実装中）はBox-Based設計で革新的
- **postfix catch/cleanup**は非常に直感的
- **Everything is Box哲学**が一貫している
- **コンパクトな実装**（Rust実装の13分の1）

### **長期ビジョン**
Phase 16-30を通じて、実用的で表現力豊かな言語へと進化。
マクロシステム・プロパティシステム・型システムの三位一体で、
次世代言語の標準を打ち立てる。

---

**作成者**: Claude Sonnet 4.5
**ベース**: ドキュメント深層分析（LANGUAGE_REFERENCE_2025.md, quick-reference.md, using.md）
**修正**: 2025-10-02 - Phase 16マクロシステム実装中の事実を反映
