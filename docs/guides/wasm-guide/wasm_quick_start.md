# 🚀 Nyash WASM クイックスタート実装（注意: 現状メンテ外）

このガイドは歴史的資料です。WASM/ブラウザ経路は現在メンテ対象外で、記載手順は最新のNyashと一致しない場合があります。実験する場合は `projects/nyash-wasm/` を参考に自己責任でお試しください。

## Step 1: Cargo.toml修正

```toml
[dependencies]
wasm-bindgen = "0.2"
web-sys = "0.3"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
  "Document",
  "Element",
  "HtmlElement",
  "HtmlCanvasElement",
  "CanvasRenderingContext2d",
  "Window",
]
```

## Step 2: lib.rsにWASMエクスポート追加

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct NyashWasm {
    interpreter: NyashInterpreter,
}

#[wasm_bindgen]
impl NyashWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // panicをconsole.errorに
        console_error_panic_hook::set_once();
        
        let mut interpreter = NyashInterpreter::new();
        // WASMBox等を登録
        Self { interpreter }
    }
    
    #[wasm_bindgen]
    pub fn eval(&mut self, code: &str) -> String {
        match self.interpreter.eval(code) {
            Ok(result) => format!("{:?}", result),
            Err(e) => format!("Error: {}", e),
        }
    }
}
```

## Step 3: ConsoleBox実装

```rust
// src/boxes/console_box.rs
pub struct ConsoleBox;

impl NyashBox for ConsoleBox {
    fn box_type(&self) -> &'static str { "ConsoleBox" }
    
    fn call_method(&self, name: &str, args: Vec<Arc<dyn NyashBox>>) -> Result<Arc<dyn NyashBox>, String> {
        match name {
            "log" => {
                let msg = args[0].to_string();
                web_sys::console::log_1(&msg.into());
                Ok(Arc::new(VoidBox))
            }
            _ => Err(format!("Unknown method: {}", name))
        }
    }
}
```

## Step 4: 簡単なHTML

```html
<!DOCTYPE html>
<html>
<head>
    <title>Nyash in Browser!</title>
    <style>
        #editor { width: 100%; height: 200px; }
        #output { border: 1px solid #ccc; padding: 10px; }
    </style>
</head>
<body>
    <h1>🐱 Nyash Browser Playground</h1>
    <textarea id="editor">
// Nyashコードをここに書くにゃ！
console = new ConsoleBox()
console.log("Hello from Nyash in Browser!")

x = 10
y = 20
console.log("x + y = " + (x + y))
    </textarea>
    <br>
    <button onclick="runNyash()">実行！</button>
    <div id="output"></div>
    
    <script type="module">
        import init, { NyashWasm } from './nyash_wasm.js';
        
        let nyash;
        
        async function main() {
            await init();
            nyash = new NyashWasm();
            window.runNyash = () => {
                const code = document.getElementById('editor').value;
                const output = nyash.eval(code);
                document.getElementById('output').textContent = output;
            };
        }
        
        main();
    </script>
</body>
</html>
```

## ビルドコマンド

```bash
# wasm-packインストール
cargo install wasm-pack

# ビルド
wasm-pack build --target web --out-dir www

# ローカルサーバー起動
cd www && python3 -m http.server 8000
```

これで http://localhost:8000 でNyashがブラウザで動く！🎉
