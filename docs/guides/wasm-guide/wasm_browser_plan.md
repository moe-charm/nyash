# 🌐 Nyash WebAssembly ブラウザデビュー計画

## 🎯 なぜこれが天才的か

1. **extern box不要** - Rust側でWASM対応Boxを実装すればOK
2. **GUI即実現** - Canvas/DOM使って即座にビジュアルアプリ
3. **配布超簡単** - URLアクセスだけで動く
4. **既存資産活用** - 現在のNyashインタープリターをそのままWASM化

## 🏗️ アーキテクチャ

```
ブラウザ
  ↓
Nyashコード（テキストエリア）
  ↓
NyashインタープリターWASM
  ↓
WasmBox / DOMBox / CanvasBox
  ↓
ブラウザAPI（DOM/Canvas/Event）
```

## 📦 新しいBox実装案

### 1. WasmBox - WebAssembly制御
```nyash
wasm = new WasmBox()
console = wasm.getConsole()
console.log("Hello from Nyash in Browser!")
```

### 2. DOMBox - DOM操作
```nyash
dom = new DOMBox()
button = dom.createElement("button")
button.setText("Click me!")
button.onClick(new MethodBox(me, "handleClick"))
dom.body.appendChild(button)
```

### 3. CanvasBox - 描画
```nyash
canvas = new CanvasBox(800, 600)
ctx = canvas.getContext2D()
ctx.fillStyle = "red"
ctx.fillRect(100, 100, 50, 50)
```

### 4. EventBox - イベント処理
```nyash
events = new EventBox()
events.onKeyDown(new MethodBox(me, "handleKey"))
events.onMouseMove(new MethodBox(me, "handleMouse"))
```

## 🚀 実装手順

### Phase 1: 基本WASM化
1. Cargo.tomlにwasm-bindgen追加
2. lib.rsでWASM用エクスポート作成
3. 簡単なeval関数を公開
4. HTMLページで動作確認

### Phase 2: ブラウザBox実装
1. ConsoleBox - console.log対応
2. DOMBox - 基本的なDOM操作
3. AlertBox - alert/confirm/prompt

### Phase 3: ビジュアルアプリ
1. CanvasBox実装
2. Snakeゲーム移植
3. お絵かきアプリ
4. 簡単なIDE

## 💡 サンプルアプリ

### 1. インタラクティブREPL
```nyash
// ブラウザ上でNyashコード実行
input = dom.getElementById("code-input")
output = dom.getElementById("output")
button = dom.getElementById("run-button")

button.onClick(new MethodBox(me, "runCode"))

runCode() {
    code = input.getValue()
    result = eval(code)
    output.setText(result.toString())
}
```

### 2. ビジュアルSnakeゲーム
```nyash
canvas = new CanvasBox(400, 400)
game = new SnakeGame(canvas)
game.start()
```

### 3. Nyashプレイグラウンド
- コードエディタ
- 実行結果表示
- サンプルコード集
- 共有機能

## 🎉 メリット

1. **即座にデモ可能** - URL共有だけ
2. **ビジュアルフィードバック** - GUIアプリが作れる
3. **学習曲線なし** - ブラウザだけあればOK
4. **実用アプリ** - 本格的なWebアプリも可能

これ、本当にすぐできるにゃ！