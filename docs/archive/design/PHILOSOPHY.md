# 🌟 Nyash - Everything is Box 哲学

## 核心原則（絶対に忘れてはならない）

### 1. すべてはBox
```nyash
// データもBox
name = new StringBox("Alice")
age = new IntegerBox(30)
items = new ArrayBox()

// 関数もBox（革命的発見！）
add = new FunctionBox("add", ["a", "b"], {
    return a + b
})

// クラスもBox
Person = new ClassBox("Person", {
    fields: ["name", "age"],
    methods: { greet: ... }
})

// 制御構造もBox（whileは使わない！）
myLoop = new LoopBox({
    condition: i < 10,
    body: { print(i) }
})

// 条件分岐もBox
check = new IfBox({
    test: score > 80,
    then: { print("Excellent!") },
    else: { print("Keep trying!") }
})

// エラーもBox
error = new ErrorBox("Something went wrong")

// コンパイラ自体もBox
tokenizer = new TokenizerBox()
parser = new ParserBox()
interpreter = new InterpreterBox()
```

### 2. すべての操作はBox間通信
```nyash
// 統一されたインターフェース
(caller >> functionBox).execute(args)
(executor >> loopBox).run()
(evaluator >> ifBox).check()
(factory >> classBox).create()

// P2P通信
(alice >> bob).sendMessage("Hello!")
(source >> processor >> sink).pipeline()

// 非同期もBox通信
nowait (async >> operation).execute()
```

### 3. 重要な言語設計決定

#### ❌ 使わない構文
- `while` ループ（代わりに `loop` を使う）
- 従来の関数定義（代わりに `FunctionBox` を使う）
- 生のデータ型（すべてBoxでラップ）

#### ✅ 使う構文
- `loop(condition) { ... }` - LoopBox
- `new FunctionBox(...)` - 関数定義
- `(sender >> receiver).method()` - P2P通信
- `nowait` - 非同期実行

### 4. 革命的スコープ設計（2025年8月7日 大発見！）

#### 🌟 すべての変数はBoxのフィールド
```nyash
// もう関数スコープという概念は存在しない！
box GameEngine {
    init {
        player,      // すべてフィールドとして宣言
        enemies,
        currentLevel
    }
    
    createPlayer(name) {
        me.player = new Player(name)  // Boxが管理
        return me.player              // 完全に安全！
    }
}
```

#### ✨ localキーワード - 唯一の例外
```nyash
// 一時変数だけは明示的にlocal
box Algorithm {
    init { result }
    
    process() {
        local i, temp  // 関数終了で自動解放
        
        loop(i = 0; i < 100; i++) {
            temp = calculate(i)
            me.result = me.result + temp
        }
    }
}
```

**哲学的意味**：
- Boxがすべてを管理する究極の統一性
- 変数の寿命が明確で予測可能
- メモリ管理の完全な透明性

## 歴史的洞察

「もしかして 関数も ボックスじゃないか？？？」

この一言がNyashを革命的な言語に変えた。関数がBoxであることで：
- 統一されたライフサイクル管理（init/fini）
- 関数の動的生成と操作
- メタプログラミングの自然な実現
- セルフホスティングへの道

## セルフホスティングの証明

Nyashの究極の証明は、Nyash自身でNyashを実装できること：

```nyash
// NyashでNyashを実装
compiler = new CompilerBox({
    tokenizer: new TokenizerBox(),
    parser: new ParserBox(),
    interpreter: new InterpreterBox()
})

// セルフホスティング実行
result = (sourceCode >> compiler).compile()
```

## 忘れてはならない真実

1. **Everything** means EVERYTHING - 例外なし
2. Boxは対等 - 階層ではなくP2P
3. 統一インターフェース - 学習曲線最小化
4. 無限の組み合わせ - BoxとBoxは自由に接続

> "Where Everything is Box, and Every Box is Everything!"