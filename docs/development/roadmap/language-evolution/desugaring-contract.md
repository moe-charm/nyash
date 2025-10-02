# 🎯 De-sugaring Contract（デシュガリング契約）

**作成日**: 2025-10-02
**対象**: すべての新機能実装
**関連**: [言語進化ロードマップ v2.0](./README.md)

---

## 📖 概要

**De-sugaring Contract（デシュガリング契約）** は、Hakoruneの**最も重要な設計原則**です。

> **新構文は既存構文・既存Boxへ有限段で必ず落ちること。IR命令の追加は最後の手段。**

この原則により、**「コアは最小・糖衣は最強」** を実現します。

---

## 🎯 5つの不変ルール（赤線）

### **1️⃣ MIR14は増やさない**

**原則**: すべてデシュガリング/静的パス/標準ライブラリで実現

**現状のMIR14命令セット**:
- 基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
- メモリ(2): Load, Store
- 制御(4): Branch, Jump, Return, Phi
- 呼び出し(1): MirCall（Callee で Global/Extern/ModuleFunction/Method/Constructor/Closure/Value を表現）
- GC(2): Barrier, Safepoint
- 構造(2): Copy, Nop（最適化/検証用・意味論不変）

📖 **詳細**: [MIR Instruction Set](../../../reference/mir/INSTRUCTION_SET.md)

**なぜ凍結するか**:
- VM/LLVM/WASMすべてのバックエンドで恩恵を受ける
- セルフホストの保守性が維持される
- 実装の複雑度が爆発しない

---

### **2️⃣ Everything is Box/flow Main.main を維持**

**原則**: すべての値はBox、すべての関数はBox/flowのメソッド

```nyash
// ✅ 正しい設計
box OptionBox<T> {
    value: T
    is_some: BoolBox

    birth(v: T) {
        me.value = v
        me.is_some = true
    }

    unwrap(): T {
        if not me.is_some {
            panic("unwrap on None")
        }
        return me.value
    }
}

// T? → OptionBox<T> へデシュガリング
local x: IntegerBox? = getSomeValue()
// ↓
local x: OptionBox<IntegerBox> = getSomeValue()
```

**Pulse は下限**:
- 最小限の型として利用
- 言語機能には持ち込まない

---

### **3️⃣ Effects/Capabilities/Contracts は meta 側**

**原則**: 言語仕様には持ち込まない、metaレイヤーで処理

```nyash
// ❌ 言語機能化（採用しない）
effect IO {
    read(path: StringBox): ResultBox<StringBox>
    write(path: StringBox, data: StringBox): ResultBox<Unit>
}

fn process() with IO {
    // IO効果を要求
}

// ✅ meta側で処理（視覚糖衣のみ）
with net.out, fs.read {
    http.get("https://api.example.com")
    file.read("config.txt")
}
// ↓ 実際は何も変えない（ログに印を出すだけ）
```

**効果システムの取り扱い**:
- **静的解析ツール**で検出（Linter/型チェッカー）
- **ランタイムは関与しない**
- **視覚的な開発体験向上**のみ

---

### **4️⃣ 例外は導入しない**

**原則**: panicはVM/実装バグ用、通常はResultBoxで伝播

```nyash
// ✅ 正しいエラー処理（ResultBox + ? 演算子）
box FileBox {
    read(path: StringBox): ResultBox<StringBox> {
        // エラー時はResultBox::Err返却
    }
}

flow Main.main() {
    local content = FileBox::read("config.txt")?  // ? で伝播
    processContent(content)
}

// ❌ 例外（採用しない）
try {
    let content = FileBox::read("config.txt")
} catch(FileNotFoundError e) {
    // ...
}
```

**panic の用途**:
- VM内部エラー（配列境界外アクセス等）
- 実装バグ（unreachable!相当）
- **通常のエラーハンドリングには使わない**

---

### **5️⃣ dev→prod の挙動差は policy のみ**

**原則**: warn/audit/enforce で制御

```nyash
// ✅ policy制御
debug_assert(internal.isValid())  // dev: panic, prod: nop

// ❌ 挙動差を言語機能に（採用しない）
if DEBUG_MODE {
    checkInvariant()
}
```

**policy種別**:
- **warn**: 警告のみ
- **audit**: ログ記録
- **enforce**: エラー（dev/prod共通）

---

## 🍬 デシュガリング実例集

### **A. 型システム**

#### **Optional型 `T?`**
```nyash
// 糖衣構文
local x: IntegerBox? = getSomeValue()

// ↓ デシュガリング後
local x: OptionBox<IntegerBox> = getSomeValue()

// OptionBoxは標準ライブラリで実装
box OptionBox<T> {
    value: T
    is_some: BoolBox

    birth(v: T) { me.value = v; me.is_some = true }
    birth_none() { me.is_some = false }

    unwrap(): T { /* ... */ }
    is_some(): BoolBox { return me.is_some }
    is_none(): BoolBox { return not me.is_some }
}
```

**MIR命令増加**: なし（既存のMirCall, Branch, Return等で実現）

#### **Union型 `A|B`**
```nyash
// 糖衣構文
type Result = OkBox | ErrorBox

// ↓ デシュガリング後
type Result = SumBox<OkBox, ErrorBox>

// SumBoxは標準ライブラリで実装
box SumBox<A, B> {
    tag: StringBox  // "A" or "B"
    value_a: A
    value_b: B

    birth_left(a: A) { me.tag = "A"; me.value_a = a }
    birth_right(b: B) { me.tag = "B"; me.value_b = b }

    match_sum<R>(on_left: fn(A) -> R, on_right: fn(B) -> R): R {
        if me.tag == "A" {
            return on_left(me.value_a)
        } else {
            return on_right(me.value_b)
        }
    }
}
```

**MIR命令増加**: なし

---

### **B. 並行処理**

#### **`async {}` スコープ**
```nyash
// 糖衣構文
async {
    nowait task1 = longTask1()
    nowait task2 = longTask2()
    // スコープ終了で自動待機
}

// ↓ デシュガリング後
local group = new TaskGroupBox()
group.spawn(fn() { longTask1() })
group.spawn(fn() { longTask2() })
group.await_all()  // スコープ終了時自動

// TaskGroupBoxは標準ライブラリで実装
box TaskGroupBox {
    tasks: ArrayBox<FutureBox>

    spawn(task: fn()) {
        local future = nowait task()
        me.tasks.push(future)
    }

    await_all() {
        loop(me.tasks.len() > 0) {
            local future = me.tasks.pop()
            await future
        }
    }
}
```

**MIR命令増加**: なし（既存のnowait/awaitで実現）

#### **`select {}` 式**
```nyash
// 糖衣構文
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

// SelectBoxは標準ライブラリで実装
box SelectBox {
    channels: ArrayBox<ChannelBox>
    handlers: ArrayBox<fn(Any)>

    on(ch: ChannelBox, handler: fn(Any)) {
        me.channels.push(ch)
        me.handlers.push(handler)
    }

    on_timeout(ms: IntegerBox, handler: fn()) { /* ... */ }

    run() {
        // poll all channels, invoke first ready handler
    }
}
```

**MIR命令増加**: なし

---

### **C. 糖衣構文**

#### **パイプライン演算子 `|>`**
```nyash
// 糖衣構文
value |> f(_) |> g(_, 42) |> h(_)

// ↓ デシュガリング後
h(g(f(value), 42))

// 実用例
[1, 2, 3, 4, 5]
    |> map(_, fn(x) { x * 2 })
    |> filter(_, fn(x) { x % 3 == 0 })
    |> reduce(_, 0, fn(acc, x) { acc + x })

// ↓ デシュガリング後
reduce(filter(map([1,2,3,4,5], fn(x){x*2}), fn(x){x%3==0}), 0, fn(acc,x){acc+x})
```

**MIR命令増加**: なし（ただの関数呼び出し順序変更）

#### **名前付き引数**
```nyash
// 糖衣構文
download(url: u, timeout: 3.s, retry: 5)

// ↓ デシュガリング後（位置引数 + デフォルト補完）
download(u, 3.s, 5)

// 欠けた引数はデフォルト補完
download(url: u)
// ↓
download(u, DEFAULT_TIMEOUT, DEFAULT_RETRY)
```

**MIR命令増加**: なし（静的パスで位置引数に変換）

#### **`with capability` スコープ**
```nyash
// 糖衣構文
with net.out, fs.read {
    http.get("https://api.example.com")
    file.read("config.txt")
}

// ↓ デシュガリング後（何も変えない）
// metaレイヤーでログに印を出すだけ
http.get("https://api.example.com")
file.read("config.txt")
```

**MIR命令増加**: なし（完全に視覚糖衣）

---

### **D. マクロ**

#### **`@test` マクロ**
```nyash
// 糖衣構文
@test
it("should add numbers") {
    local calc = new Calculator()
    expect(calc.add(2, 3)).toBe(5)
}

// ↓ デシュガリング後（HIRパッチ）
TestRunnerBox::register("should add numbers", fn() {
    local calc = new Calculator()
    expect(calc.add(2, 3)).toBe(5)
})

// TestRunnerBoxは標準ライブラリ
static box TestRunnerBox {
    tests: ArrayBox<TestCase>

    register(name: StringBox, test: fn()) {
        me.tests.push(new TestCase(name, test))
    }

    run_all() {
        loop(me.tests.len() > 0) {
            local test = me.tests.pop()
            test.run()
        }
    }
}
```

**MIR命令増加**: なし（HIRレベルでパッチ）

#### **`@profile` マクロ**
```nyash
// 糖衣構文
@profile
calculate() {
    // 処理
}

// ↓ デシュガリング後（HIRパッチ）
calculate() {
    local _prof = ProfileBox::start("calculate")
    // 処理
    _prof.end()
}

// ProfileBoxは標準ライブラリ
box ProfileBox {
    name: StringBox
    start_time: TimeBox

    static start(name: StringBox): ProfileBox {
        local prof = new ProfileBox()
        prof.name = name
        prof.start_time = TimeBox::now()
        return prof
    }

    end() {
        local duration = TimeBox::now() - me.start_time
        ConsoleBox::log("Profile: " + me.name + " took " + duration.toString())
    }
}
```

**MIR命令増加**: なし

---

## 📊 デシュガリング規則一覧表

| 構文 | デシュガリング後 | 実装方法 | MIR命令増加 | Phase |
|------|----------------|---------|------------|-------|
| `T?` | `OptionBox<T>` | 標準ライブラリ | なし | 20 |
| `A\|B` | `SumBox<A,B>` | 標準ライブラリ | なし | 20 |
| `x \|> f(_)` | `f(x)` | 静的パス（AST変換） | なし | 17-18 |
| `f(a:1, b:2)` | `f(1,2)` | 静的パス（引数並び替え） | なし | 18-19 |
| `async { body }` | `TaskGroupBox::scoped(...)` | 標準ライブラリ | なし | 22 |
| `select { ... }` | `SelectBox::new()...run()` | 標準ライブラリ | なし | 23 |
| `@test it(...){...}` | `TestRunnerBox::register(...)` | マクロ（HIRパッチ） | なし | 16.1 |
| `@bench(iter:N) { ... }` | `BenchmarkBox::run(...)` | マクロ（HIRパッチ） | なし | 16.3 |
| `@profile { ... }` | `ProfileBox::start()` + `.end()` | マクロ（HIRパッチ） | なし | 20 |
| `repr(obj)` | `@derive(Debug)` 連携 | マクロ（HIRパッチ） | なし | 18 |
| `x.move()` | 所有権移転メソッド | 標準ライブラリ | なし | 28 |
| `x.share()` | Arc参照増加メソッド | 標準ライブラリ | なし | 28 |
| `x.weak()` | 弱参照取得メソッド | 標準ライブラリ | なし | 28 |
| `with cap { ... }` | （何も変えない） | 視覚糖衣（meta） | なし | 20-22 |
| `comptime { ... }` | 定数埋め込み | 静的パス（ビルド時評価） | なし | 22-25 |
| `pattern Some(x)` | `OptionBox::Some(x)` | 静的パス（パターン別名） | なし | 23-25 |

**重要**: すべてMIR命令増加なし！

---

## 🔍 デシュガリング検証方法

### **1. コンパイラフラグ**
```bash
# デシュガリング前のAST表示
$ hakorune --dump-ast program.hkr

# デシュガリング後のHIR表示
$ hakorune --dump-hir program.hkr

# 最終MIR表示（MIR14のみ）
$ hakorune --dump-mir program.hkr
```

### **2. スモークテスト**
```bash
# デシュガリング検証スイート
$ tools/smokes/v2/run.sh --profile desugaring
```

### **3. Linter検出**
```bash
# レガシー命令検出
$ HAKO_OPT_DIAG=1 hakorune program.hkr

# レガシー命令禁止（Fail-Fast）
$ HAKO_OPT_DIAG_FORBID_LEGACY=1 hakorune program.hkr
```

---

## 🚨 契約違反の例（やってはいけない）

### **❌ 悪い例1: MIR命令を追加**
```rust
// ❌ NG: Optional型のために新命令追加
enum MirInstruction {
    // ... 既存14命令 ...
    SomeCheck(ValueId),  // ← 追加してはいけない！
    NoneCheck(ValueId),  // ← 追加してはいけない！
}

// ✅ OK: OptionBoxで実現
box OptionBox<T> {
    is_some(): BoolBox { return me.is_some }
    // MirCall で呼び出し（既存命令）
}
```

### **❌ 悪い例2: 効果システムを言語機能に**
```nyash
// ❌ NG: 効果システムを型システムに統合
effect IO {
    read(path: StringBox): ResultBox<StringBox>
}

fn process() with IO {
    // VM/コンパイラが効果を追跡
}

// ✅ OK: meta側で処理（視覚糖衣）
with io {
    process()  // ログに印を出すだけ
}
```

### **❌ 悪い例3: 例外を導入**
```nyash
// ❌ NG: try-catch構文を実装
try {
    dangerousOp()
} catch(NetworkError e) {
    // ...
}

// ✅ OK: ResultBox + ? 演算子
local result = dangerousOp()?  // Err時は早期return
```

---

## 📚 参考資料

- [MIR Instruction Set](../../../reference/mir/INSTRUCTION_SET.md) - MIR14命令セット詳細
- [言語進化ロードマップ v2.0](./README.md) - 実装計画
- [Phase 16 Macro Revolution](../phases/phase-16-macro-revolution/README.md) - マクロシステム詳細
- [Discoverability問題分析](./discoverability-analysis.md) - 発見性問題と解決策

---

## 🎊 まとめ

**De-sugaring Contract（デシュガリング契約）** により、Hakoruneは：

1. ✅ **コアは最小** - MIR14命令セットを凍結
2. ✅ **糖衣は最強** - すべての新機能をデシュガリング/Box/マクロで実現
3. ✅ **保守性最高** - セルフホストの実装容易性維持
4. ✅ **一貫性抜群** - Everything is Box哲学貫徹
5. ✅ **拡張性無限** - Box/マクロでいくらでも拡張可能

この原則により、**次世代言語の標準** を打ち立てます。

---

**作成者**: Claude Sonnet 4.5 + ChatGPT Pro
**作成日**: 2025-10-02
**関連**: [言語進化ロードマップ v2.0](./README.md)
