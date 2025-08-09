# 🎯 Nyash LISP インタープリタ設計書

## 概要
Nyash上で動作するLISPインタープリタを実装する。「Everything is Box」哲学に基づき、LISPのS式をNyashのBoxで表現する。

## 🏗️ アーキテクチャ

### 1. コアBox実装

#### ConsBox - cons cell（ペア）
```nyash
box ConsBox {
    car    // 最初の要素
    cdr    // 残りの要素（通常は別のConsBoxかNullBox）
    
    init { car, cdr }
    
    func getCar() { return me.car }
    func getCdr() { return me.cdr }
    func setCar(value) { me.car = value }
    func setCdr(value) { me.cdr = value }
    
    func toString() {
        if (NullBox.isNull(me.cdr)) {
            return "(" + me.car.toString() + ")"
        }
        // TODO: 適切なリスト表示
        return "(" + me.car.toString() + " . " + me.cdr.toString() + ")"
    }
}
```

#### SymbolBox - シンボル
```nyash
box SymbolBox {
    name
    
    init { name }
    
    func getName() { return me.name }
    func toString() { return me.name }
}
```

#### LispEnvironment - 変数環境
```nyash
box LispEnvironment {
    bindings  // MapBox: symbol name -> value
    parent    // 親環境（スコープチェーン用）
    
    init { parent }
    
    func define(symbol, value) {
        me.bindings.set(symbol.getName(), value)
    }
    
    func lookup(symbol) {
        name = symbol.getName()
        if (me.bindings.has(name)) {
            return me.bindings.get(name)
        }
        if (not NullBox.isNull(me.parent)) {
            return me.parent.lookup(symbol)
        }
        return new ErrorBox("Unbound variable: " + name)
    }
}
```

### 2. S式パーサー

```nyash
box SExpressionParser {
    tokens
    position
    
    init { input }
    
    func parse() {
        me.tokens = me.tokenize(input)
        me.position = 0
        return me.parseExpression()
    }
    
    func parseExpression() {
        token = me.currentToken()
        
        if (token == "(") {
            return me.parseList()
        }
        if (token.isNumber()) {
            return new IntegerBox(token.toNumber())
        }
        if (token.isString()) {
            return new StringBox(token.getValue())
        }
        // シンボル
        return new SymbolBox(token)
    }
    
    func parseList() {
        me.consume("(")
        elements = new ArrayBox()
        
        loop(me.currentToken() != ")") {
            elements.push(me.parseExpression())
        }
        
        me.consume(")")
        return me.arrayToConsList(elements)
    }
}
```

### 3. eval関数

```nyash
box LispEvaluator {
    globalEnv
    
    init {}
    
    func eval(expr, env) {
        // 自己評価的な値
        if (expr.isNumber() or expr.isString()) {
            return expr
        }
        
        // シンボル
        if (expr.isSymbol()) {
            return env.lookup(expr)
        }
        
        // リスト（関数適用か特殊形式）
        if (expr.isCons()) {
            car = expr.getCar()
            
            // 特殊形式のチェック
            if (car.isSymbol()) {
                name = car.getName()
                
                if (name == "quote") {
                    return me.evalQuote(expr, env)
                }
                if (name == "if") {
                    return me.evalIf(expr, env)
                }
                if (name == "define") {
                    return me.evalDefine(expr, env)
                }
                if (name == "lambda") {
                    return me.evalLambda(expr, env)
                }
                // ... 他の特殊形式
            }
            
            // 通常の関数適用
            func = me.eval(car, env)
            args = me.evalList(expr.getCdr(), env)
            return me.apply(func, args)
        }
        
        return expr
    }
    
    func apply(func, args) {
        // プリミティブ関数
        if (func.isPrimitive()) {
            return func.applyPrimitive(args)
        }
        
        // ラムダ式
        if (func.isLambda()) {
            newEnv = new LispEnvironment(func.getEnv())
            params = func.getParams()
            
            // パラメータをバインド
            // ... 実装
            
            return me.eval(func.getBody(), newEnv)
        }
        
        return new ErrorBox("Not a function: " + func.toString())
    }
}
```

### 4. 基本関数の実装

```nyash
box LispPrimitives {
    func setupGlobalEnv(env) {
        // 算術演算
        env.define(new SymbolBox("+"), new PrimitiveBox(me.add))
        env.define(new SymbolBox("-"), new PrimitiveBox(me.subtract))
        env.define(new SymbolBox("*"), new PrimitiveBox(me.multiply))
        env.define(new SymbolBox("/"), new PrimitiveBox(me.divide))
        
        // リスト操作
        env.define(new SymbolBox("cons"), new PrimitiveBox(me.cons))
        env.define(new SymbolBox("car"), new PrimitiveBox(me.car))
        env.define(new SymbolBox("cdr"), new PrimitiveBox(me.cdr))
        env.define(new SymbolBox("list"), new PrimitiveBox(me.list))
        
        // 述語
        env.define(new SymbolBox("null?"), new PrimitiveBox(me.isNull))
        env.define(new SymbolBox("pair?"), new PrimitiveBox(me.isPair))
        env.define(new SymbolBox("number?"), new PrimitiveBox(me.isNumber))
        
        // 比較
        env.define(new SymbolBox("="), new PrimitiveBox(me.equal))
        env.define(new SymbolBox("<"), new PrimitiveBox(me.lessThan))
        env.define(new SymbolBox(">"), new PrimitiveBox(me.greaterThan))
    }
    
    func add(args) {
        sum = 0
        current = args
        loop(not NullBox.isNull(current)) {
            sum = sum + current.getCar().getValue()
            current = current.getCdr()
        }
        return new IntegerBox(sum)
    }
    
    // ... 他のプリミティブ関数
}
```

## 🎮 使用例

```lisp
; Nyash LISPでの階乗計算
(define factorial
  (lambda (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1))))))

(factorial 5)  ; => 120

; リスト操作
(define map
  (lambda (f lst)
    (if (null? lst)
        '()
        (cons (f (car lst))
              (map f (cdr lst))))))

(map (lambda (x) (* x x)) '(1 2 3 4 5))  ; => (1 4 9 16 25)
```

## 📋 実装ステップ

1. **Phase 1: 基本データ構造**
   - ConsBox実装
   - SymbolBox実装
   - 基本的なリスト操作

2. **Phase 2: パーサー**
   - トークナイザー
   - S式パーサー
   - 文字列→Box変換

3. **Phase 3: 評価器**
   - eval関数の基本実装
   - 環境（Environment）管理
   - 特殊形式の処理

4. **Phase 4: 基本関数**
   - 四則演算
   - リスト操作（cons, car, cdr）
   - 述語関数

5. **Phase 5: 高度な機能**
   - lambda式
   - クロージャ
   - 再帰関数のサポート

6. **Phase 6: 最適化とデバッグ**
   - DebugBoxとの統合
   - エラーハンドリングの改善
   - パフォーマンス最適化

## 🎯 成功基準

- 基本的なLISPプログラムが動作する
- 再帰関数が正しく実行される
- リスト操作が適切に機能する
- Nyashの他のBox機能と統合できる

## 💡 技術的課題

1. **末尾再帰最適化**: NyashはTCOをサポートしていないため、深い再帰でスタックオーバーフローの可能性
2. **ガベージコレクション**: Nyashのfini()との統合方法
3. **マクロシステム**: 将来的な実装検討事項

---

「Everything is Box」の究極の実証 - LISPインタープリタ on Nyash！