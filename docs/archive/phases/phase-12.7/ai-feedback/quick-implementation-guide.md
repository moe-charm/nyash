# ANCP即座実装ガイド - 今すぐ始める！

Date: 2025-09-03

## 🚀 30分で作る最小プロトタイプ

### Step 1: P*正規化ルール（5分で決める）
```rust
// 最小限の正規化ルール
pub struct MinimalCanonicalizer {
    rules: Rules {
        comments: Remove,        // v1では削除
        whitespace: SingleSpace, // 連続空白→単一空白
        newlines: Preserve,      // 改行は保持
        semicolons: Required,    // セミコロン必須
    }
}
```

### Step 2: 最小記号マッピング（10分）
```rust
// 超シンプルマッピング
const KEYWORD_MAP: &[(&str, &str)] = &[
    ("box", "$"),
    ("new", "@"),
    ("me", "."),
    ("init", "#"),
    ("return", "^"),
    ("local", "l"),
    ("public", "+"),
    ("if", "?"),
    ("else", ":"),
];

const BUILTIN_MAP: &[(&str, &str)] = &[
    ("StringBox", "S"),
    ("IntegerBox", "I"),
    ("MapBox", "M"),
    ("ArrayBox", "A"),
];
```

### Step 3: 最小エンコーダー（15分）
```rust
// Boxだけ動けばOK！
fn encode_minimal(code: &str) -> String {
    let mut result = code.to_string();
    
    // 1. キーワード置換
    for (from, to) in KEYWORD_MAP {
        result = result.replace(from, to);
    }
    
    // 2. 型名短縮
    for (from, to) in BUILTIN_MAP {
        result = result.replace(from, to);
    }
    
    // 3. 空白圧縮
    result = compress_whitespace(result);
    
    result
}
```

## 📝 1時間で作る実用版

### ソースマップ最小実装
```rust
#[derive(Serialize, Deserialize)]
struct SimpleSourceMap {
    version: u8,
    mappings: Vec<Mapping>,
}

struct Mapping {
    f_pos: usize,  // Fusion位置
    p_pos: usize,  // Pretty位置  
    len: usize,    // 長さ
}
```

### CLI最小実装
```bash
#!/bin/bash
# ancp.sh - 超簡易版

case $1 in
    encode)
        cargo run --bin ancp-encoder < $2
        ;;
    decode)
        cargo run --bin ancp-decoder < $2
        ;;
    *)
        echo "Usage: ancp encode|decode file"
        ;;
esac
```

## 🧪 今すぐ試せるテストケース

### Test 1: 最小Box
```nyash
# input.nyash
box Test {
    init { value }
}

# 期待出力
$Test{#{value}}
```

### Test 2: 簡単な関数
```nyash
# input.nyash
box Calculator {
    add(a, b) {
        return a + b
    }
}

# 期待出力
$Calculator{add(a,b){^a+b}}
```

## 🎯 今日中に達成可能な目標

### 午前（2時間）
1. [ ] P*ルール仕様書（1ページ）
2. [ ] 記号マッピング表完成
3. [ ] Rustプロジェクト作成

### 午後（3時間）
1. [ ] 最小エンコーダー実装
2. [ ] 10個のテストケース作成
3. [ ] 圧縮率測定スクリプト

### 夕方（1時間）
1. [ ] README.md作成
2. [ ] 初期ベンチマーク実行
3. [ ] 明日の計画立案

## 💡 すぐ使えるコードスニペット

### Rust Cargo.toml
```toml
[package]
name = "ancp"
version = "0.1.0"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[[bin]]
name = "ancp-cli"
path = "src/main.rs"
```

### 最初のmain.rs
```rust
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    
    // 超簡易圧縮
    let compressed = input
        .replace("box", "$")
        .replace("init", "#")
        .replace("me.", ".")
        .replace("  ", " ");
        
    println!("{}", compressed);
}
```

## 🏃 動作確認コマンド

```bash
# 1. プロジェクト作成
cargo new ancp-prototype
cd ancp-prototype

# 2. 最小実装
echo 'box Test { init { x } }' | cargo run

# 3. 圧縮率確認
echo 'box Test { init { x } }' | wc -c  # 元
echo '$Test{#{x}}' | wc -c              # 後

# 4. 成功！🎉
```

---

**15分後には動くものができる！さあ始めよう！**