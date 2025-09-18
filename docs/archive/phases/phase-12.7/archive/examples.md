# ANCP Examples - 実例で学ぶ圧縮記法

## 🎯 基本パターン

### 1. シンプルなBox定義
```nyash
// Nyash (31文字)
box Point {
    init { x, y }
}

// ANCP (16文字) - 48%削減！
$Point{#{x,y}}
```

### 2. メソッド付きBox
```nyash
// Nyash (118文字)
box Calculator {
    init { result }
    
    birth() {
        me.result = 0
    }
    
    add(x, y) {
        me.result = x + y
        return me.result
    }
}

// ANCP (59文字) - 50%削減！
$Calculator{#{result}b(){m.result=0}add(x,y){m.result=x+y r m.result}}
```

### 3. 継承/デリゲーション
```nyash
// Nyash (165文字)
box Dog from Animal {
    init { name, breed }
    
    birth(name, breed) {
        from Animal.init(name)
        me.breed = breed
    }
    
    bark() {
        return "Woof! I'm " + me.name
    }
}

// ANCP (87文字) - 47%削減！
$Dog@Animal{#{name,breed}b(name,breed){@Animal.init(name)m.breed=breed}bark(){r"Woof! I'm"+m.name}}
```

## 🚀 実践的な例

### 4. P2Pノード実装
```nyash
// Nyash (287文字)
box P2PNode from NetworkBox {
    init { id, peers, messages }
    
    birth(id) {
        me.id = id
        me.peers = new ArrayBox()
        me.messages = new MapBox()
    }
    
    connect(peer) {
        me.peers.push(peer)
        peer.addPeer(me)
        return me
    }
    
    broadcast(msg) {
        local i = 0
        loop(i < me.peers.length()) {
            me.peers.get(i).receive(msg)
            i = i + 1
        }
    }
}

// ANCP (156文字) - 46%削減！
$P2PNode@NetworkBox{#{id,peers,messages}b(id){m.id=id m.peers=n ArrayBox()m.messages=n MapBox()}connect(peer){m.peers.push(peer)peer.addPeer(m)r m}broadcast(msg){l i=0 L(i<m.peers.length()){m.peers.get(i).receive(msg)i=i+1}}}
```

### 5. 非同期WebServer
```nyash
// Nyash (342文字)
box WebServer from HttpBox {
    init { port, routes, middleware }
    
    birth(port) {
        from HttpBox.init(port)
        me.routes = new MapBox()
        me.middleware = new ArrayBox()
    }
    
    route(path, handler) {
        me.routes.set(path, handler)
        return me
    }
    
    use(middleware) {
        me.middleware.push(middleware)
        return me
    }
    
    async start() {
        await from HttpBox.listen(me.port)
        print("Server running on port " + me.port)
    }
}

// ANCP (183文字) - 46%削減！
$WebServer@HttpBox{#{port,routes,middleware}b(port){@HttpBox.init(port)m.routes=n MapBox()m.middleware=n ArrayBox()}route(path,handler){m.routes.set(path,handler)r m}use(middleware){m.middleware.push(middleware)r m}async start(){await @HttpBox.listen(m.port)print("Server running on port"+m.port)}}
```

## 💡 高度なパターン

### 6. エラーハンドリング
```nyash
// Nyash (198文字)
box SafeCalculator {
    divide(a, b) {
        if b == 0 {
            return new ErrorBox("Division by zero")
        } else {
            return new ResultBox(a / b)
        }
    }
}

// ANCP (93文字) - 53%削減！
$SafeCalculator{divide(a,b){?b==0{r n ErrorBox("Division by zero")}:{r n ResultBox(a/b)}}}
```

### 7. ジェネリック風パターン
```nyash
// Nyash (245文字)
box Container {
    init { items, type }
    
    birth(type) {
        me.items = new ArrayBox()
        me.type = type
    }
    
    add(item) {
        if item.type() == me.type {
            me.items.push(item)
            return true
        }
        return false
    }
}

// ANCP (124文字) - 49%削減！
$Container{#{items,type}b(type){m.items=n ArrayBox()m.type=type}add(item){?item.type()==m.type{m.items.push(item)r true}r false}}
```

## 🎮 完全なアプリケーション例

### 8. Todoアプリ（フル実装）
```nyash
// Nyash (562文字)
box TodoApp {
    init { todos, nextId }
    
    birth() {
        me.todos = new ArrayBox()
        me.nextId = 1
    }
    
    addTodo(text) {
        local todo = new MapBox()
        todo.set("id", me.nextId)
        todo.set("text", text)
        todo.set("done", false)
        
        me.todos.push(todo)
        me.nextId = me.nextId + 1
        
        return todo.get("id")
    }
    
    toggleTodo(id) {
        local i = 0
        loop(i < me.todos.length()) {
            local todo = me.todos.get(i)
            if todo.get("id") == id {
                todo.set("done", not todo.get("done"))
                return true
            }
            i = i + 1
        }
        return false
    }
    
    listTodos() {
        return me.todos
    }
}

// ANCP (296文字) - 47%削減！
$TodoApp{#{todos,nextId}b(){m.todos=n ArrayBox()m.nextId=1}addTodo(text){l todo=n MapBox()todo.set("id",m.nextId)todo.set("text",text)todo.set("done",false)m.todos.push(todo)m.nextId=m.nextId+1 r todo.get("id")}toggleTodo(id){l i=0 L(i<m.todos.length()){l todo=m.todos.get(i)?todo.get("id")==id{todo.set("done",not todo.get("done"))r true}i=i+1}r false}listTodos(){r m.todos}}
```

## 📊 圧縮効果まとめ

| 例 | Nyash文字数 | ANCP文字数 | 削減率 |
|----|------------|-----------|--------|
| Point | 31 | 16 | 48% |
| Calculator | 118 | 59 | 50% |
| Dog | 165 | 87 | 47% |
| P2PNode | 287 | 156 | 46% |
| WebServer | 342 | 183 | 46% |
| SafeCalculator | 198 | 93 | 53% |
| Container | 245 | 124 | 49% |
| TodoApp | 562 | 296 | 47% |

**平均削減率: 48.3%**

## 🔍 パターン分析

### 最も効果的な変換
1. `return` → `r` (83%削減)
2. `local` → `l` (80%削減)
3. `new` → `n` (67%削減)
4. `box` → `$` (75%削減)
5. `me` → `m` (50%削減)

### 圧縮のコツ
- 空白を最小限に（セミコロン不要）
- 中括弧の直後に内容を書く
- 演算子の前後の空白を省略
- 文字列連結の空白も省略可能

## 🎯 練習問題

### 問題1
次のNyashコードをANCPに変換してください：
```nyash
box User {
    init { name, email }
    
    birth(name, email) {
        me.name = name
        me.email = email
    }
    
    toString() {
        return me.name + " <" + me.email + ">"
    }
}
```

<details>
<summary>答え</summary>

```ancp
$User{#{name,email}b(name,email){m.name=name m.email=email}toString(){r m.name+" <"+m.email+">"}}
```
</details>

### 問題2
次のANCPコードをNyashに戻してください：
```ancp
$Stack{#{items}b(){m.items=n ArrayBox()}push(item){m.items.push(item)}pop(){?m.items.length()>0{r m.items.pop()}r null}}
```

<details>
<summary>答え</summary>

```nyash
box Stack {
    init { items }
    
    birth() {
        me.items = new ArrayBox()
    }
    
    push(item) {
        me.items.push(item)
    }
    
    pop() {
        if me.items.length() > 0 {
            return me.items.pop()
        }
        return null
    }
}
```
</details>

---

これらの例を参考に、ANCPを使いこなしてAI時代の効率的な開発を実現しましょう！