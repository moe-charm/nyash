# Nyash Playground Guide（ブラウザデモ活用ガイド）

最終更新: 2025-08-13

## リンク
- Playground: https://moe-charm.github.io/nyash/projects/nyash-wasm/nyash_playground.html

## ねらい
- 言語の要点（init/fini/weak/スコープ解放）を“動く例”で体感
- レビュワーやチームへの共有・再現を容易化（ゼロインストール）

## 使い方（最短）
1) 上記リンクを開く
2) 下のサンプルコードをコピー＆ペースト
3) Run/実行ボタンで結果確認（ログ・出力・エラー挙動）

---

## シナリオ1: 循環参照 vs weak（自動nil化）

```nyash
# Parent↔Child の双方向参照。
# 子→親は weak 参照にして、リークとダングリングを防ぐ。

box Parent {
    init { child }
    pack() {
        me.child = new Child()
        me.child.setParent(me)
    }
    getName() { return "P" }
    fini() { print("Parent.fini") }
}

box Child {
    init { weak parent }
    setParent(p) { me.parent = p }
    show() {
        if (me.parent != null) { print("parent=" + me.parent.getName()) }
        else { print("parent is gone") }
    }
    fini() { print("Child.fini") }
}

p = new Parent()
p.child.show()      # => parent=P

# 親を明示的に破棄（fini 呼出しが発火する環境であればここで解放）
p.fini()

# 親が破棄済みなので、weak 参照は自動的に nil 化される
p.child.show()      # => parent is gone（想定）
```

ポイント:
- `init { weak parent }`で弱参照を宣言
- 参照先が破棄されるとアクセス時に自動で`null`扱い

---

## シナリオ2: 再代入時の fini 発火（予備解放）

```nyash
box Holder { init { obj } }
box Thing  { fini() { print("Thing.fini") } }

h = new Holder()
h.obj = new Thing()
h.obj = new Thing()  # 旧 obj に対して fini() が呼ばれる（ログで確認）
```

ポイント:
- フィールド再代入の節目で `fini()` が自動呼出し
- 二重解放は内部で抑止（安全側）

---

## シナリオ3: スコープ抜けでローカル解放

```nyash
function make() {
    local t
    t = new Thing()
    # 関数を抜けるとスコープ追跡により t が解放される
}

box Thing { fini() { print("Thing.fini (scope)") } }

make()  # => Thing.fini (scope)
```

ポイント:
- `local`で宣言された変数はスコープ終了時に一括解放
- 暗黙ではなく構文規約で明示（未宣言代入はエラー）

---

## Tips（レビュー/論文向け）
- 論文やREADMEから本ガイドへリンクし、コピー＆ペーストで再現
- 期待ログ（例: `Thing.fini`）を明記して、挙動の確認を容易に
- 比較のため「weakなし版」と「weakあり版」を並記

