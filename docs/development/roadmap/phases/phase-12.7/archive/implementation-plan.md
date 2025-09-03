# ANCP Implementation Plan

## 🎯 実装戦略：段階的アプローチ

### 基本方針
1. **最小実装から開始**: 20語の固定辞書でMVP
2. **段階的拡張**: 機能を少しずつ追加
3. **早期統合**: スモークテストと早期に統合
4. **継続的検証**: 各段階で往復テスト実施

## 📅 Week 1: 基礎実装

### Day 1-2: プロジェクトセットアップ
```toml
# Cargo.toml に追加
[features]
ancp = []

[dependencies]
phf = "0.11"  # 静的マップ用
tiktoken-rs = "0.5"  # トークン計測用（optional）
```

### Day 3-4: 基本Transcoder実装
```rust
// src/ancp/mod.rs
pub mod transcoder;
pub mod mappings;
pub mod error;

// src/ancp/transcoder.rs
use phf::phf_map;

static NYASH_TO_ANCP: phf::Map<&'static str, &'static str> = phf_map! {
    "box" => "$",
    "new" => "n",
    "me" => "m",
    "local" => "l",
    "return" => "r",
    // ... 初期20語
};

pub struct BasicTranscoder;

impl BasicTranscoder {
    pub fn encode(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let tokens = tokenize_simple(input);
        
        for token in tokens {
            match NYASH_TO_ANCP.get(token.text) {
                Some(ancp) => result.push_str(ancp),
                None => result.push_str(token.text),
            }
            result.push_str(&token.trailing_space);
        }
        
        result
    }
}
```

### Day 5-7: 基本往復テスト
```rust
// tests/ancp_roundtrip.rs
#[test]
fn test_basic_roundtrip() {
    let cases = vec![
        "box Test { }",
        "new StringBox()",
        "me.field = 42",
        "local x = 10",
        "return result",
    ];
    
    let transcoder = BasicTranscoder::new();
    
    for case in cases {
        let encoded = transcoder.encode(case);
        let decoded = transcoder.decode(&encoded);
        assert_eq!(case, decoded, "Failed for: {}", case);
    }
}
```

## 📅 Week 2: スマート変換

### Day 8-9: コンテキスト認識パーサー
```rust
// src/ancp/context_parser.rs
pub struct ContextAwareTranscoder {
    basic: BasicTranscoder,
}

impl ContextAwareTranscoder {
    pub fn encode(&self, input: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        
        // 文字列・コメント内は変換しない
        for (i, ch) in input.chars().enumerate() {
            match ch {
                '"' if !in_comment => in_string = !in_string,
                '/' if !in_string && peek_next(input, i) == Some('/') => {
                    in_comment = true;
                },
                '\n' => in_comment = false,
                _ => {}
            }
            
            // コンテキストに応じて処理
            if !in_string && !in_comment {
                // トークン変換
            } else {
                // そのまま出力
            }
        }
        
        result
    }
}
```

### Day 10-11: Lexer統合
```rust
// src/parser/lexer.rs に追加
impl Lexer {
    pub fn with_ancp_support(input: &str) -> Self {
        if input.starts_with(";ancp:") {
            // ANCPモードで初期化
            Self::new_ancp_mode(input)
        } else {
            Self::new(input)
        }
    }
    
    fn new_ancp_mode(input: &str) -> Self {
        // ANCP → Nyash変換してからレキシング
        let transcoder = get_transcoder();
        let nyash_code = transcoder.decode(input).unwrap();
        Self::new(&nyash_code)
    }
}
```

### Day 12-14: エラー位置マッピング
```rust
// src/ancp/source_map.rs
pub struct SourceMap {
    mappings: Vec<Mapping>,
}

impl SourceMap {
    pub fn translate_position(&self, ancp_pos: Position) -> Position {
        // ANCP位置 → Nyash位置への変換
        self.mappings
            .binary_search_by_key(&ancp_pos, |m| m.ancp_pos)
            .map(|i| self.mappings[i].nyash_pos)
            .unwrap_or(ancp_pos)
    }
}
```

## 📅 Week 3: ツール実装

### Day 15-16: CLIツール
```rust
// src/bin/nyash2ancp.rs
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    input: PathBuf,
    
    #[clap(short, long)]
    output: Option<PathBuf>,
    
    #[clap(long)]
    measure_tokens: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let content = fs::read_to_string(&args.input)?;
    
    let transcoder = AncpTranscoder::new();
    let encoded = transcoder.encode(&content)?;
    
    if args.measure_tokens {
        let reduction = measure_token_reduction(&content, &encoded);
        eprintln!("Token reduction: {:.1}%", reduction * 100.0);
    }
    
    match args.output {
        Some(path) => fs::write(path, encoded)?,
        None => print!("{}", encoded),
    }
    
    Ok(())
}
```

### Day 17-18: スモークテスト統合
```bash
#!/bin/bash
# tools/test_ancp_roundtrip.sh

test_file=$1
expected_pattern=$2

# 1. 通常実行
normal_output=$(./target/release/nyash "$test_file" 2>&1)

# 2. ANCP変換
ancp_file="${test_file%.nyash}.ancp"
./target/release/nyash2ancp -i "$test_file" -o "$ancp_file"

# 3. ANCP実行
ancp_output=$(./target/release/nyash "$ancp_file" 2>&1)

# 4. 出力比較
if [ "$normal_output" != "$ancp_output" ]; then
    echo "ERROR: Output mismatch for $test_file"
    exit 1
fi

# 5. パターン検証（既存のスモークテスト方式）
echo "$ancp_output" | grep -q "$expected_pattern"
```

### Day 19-21: VSCode拡張（基礎）
```typescript
// vscode-extension/src/extension.ts
import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    // ホバープロバイダー
    const hoverProvider = vscode.languages.registerHoverProvider(
        'ancp',
        {
            provideHover(document, position) {
                const word = document.getText(
                    document.getWordRangeAtPosition(position)
                );
                
                const original = ancpToNyash(word);
                if (original !== word) {
                    return new vscode.Hover(
                        `ANCP: \`${word}\` → Nyash: \`${original}\``
                    );
                }
            }
        }
    );
    
    context.subscriptions.push(hoverProvider);
}
```

## 📅 Week 4: 最適化と統合

### Day 22-23: tiktoken実測と最適化
```python
# tools/measure_ancp_efficiency.py
import tiktoken
import json

enc = tiktoken.get_encoding("cl100k_base")

def measure_file(nyash_path, ancp_path):
    with open(nyash_path) as f:
        nyash_code = f.read()
    with open(ancp_path) as f:
        ancp_code = f.read()
    
    nyash_tokens = len(enc.encode(nyash_code))
    ancp_tokens = len(enc.encode(ancp_code))
    
    return {
        "file": nyash_path,
        "nyash_tokens": nyash_tokens,
        "ancp_tokens": ancp_tokens,
        "reduction": 1 - (ancp_tokens / nyash_tokens),
        "nyash_chars": len(nyash_code),
        "ancp_chars": len(ancp_code),
    }

# 全サンプルファイルで測定
results = []
for nyash_file in glob.glob("examples/*.nyash"):
    ancp_file = nyash_file.replace(".nyash", ".ancp")
    results.append(measure_file(nyash_file, ancp_file))

# 統計出力
avg_reduction = sum(r["reduction"] for r in results) / len(results)
print(f"Average token reduction: {avg_reduction:.1%}")
```

### Day 24-25: CI/CD統合
```yaml
# .github/workflows/ancp.yml
name: ANCP Tests

on: [push, pull_request]

jobs:
  ancp-roundtrip:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build with ANCP
        run: cargo build --release --features ancp
      
      - name: Run roundtrip tests
        run: |
          for f in examples/*.nyash; do
            echo "Testing: $f"
            ./tools/test_ancp_roundtrip.sh "$f"
          done
      
      - name: Measure efficiency
        run: |
          python3 tools/measure_ancp_efficiency.py > ancp_report.json
          
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: ancp-efficiency-report
          path: ancp_report.json
```

### Day 26-28: ドキュメント・最終調整
- ユーザーガイド作成
- API ドキュメント生成
- パフォーマンスチューニング
- 最終テスト

## 🎯 成功指標

### Week 1終了時
- [ ] 基本20語で往復変換成功
- [ ] 単純なNyashプログラムが動作

### Week 2終了時
- [ ] コンテキスト認識変換
- [ ] Lexer統合完了
- [ ] エラー位置の正確なマッピング

### Week 3終了時
- [ ] CLI ツール完成
- [ ] スモークテスト統合
- [ ] VSCode基本機能

### Week 4終了時
- [ ] トークン削減率50%以上達成
- [ ] 全サンプルで往復テスト成功
- [ ] CI/CD完全統合
- [ ] ドキュメント完成

## 🚧 リスクと対策

### 技術的リスク
1. **パフォーマンス劣化**
   - 対策: 段階的実装で早期発見
   - 対策: プロファイリング継続実施

2. **互換性問題**
   - 対策: 既存テストスイートで検証
   - 対策: feature flagで段階的有効化

### 運用リスク
1. **採用障壁**
   - 対策: 分かりやすいドキュメント
   - 対策: 移行ツール提供

2. **メンテナンス負荷**
   - 対策: 自動テスト充実
   - 対策: CI/CDで品質保証

## 📝 チェックリスト

### 実装前
- [ ] tiktoken実測による記号選定完了
- [ ] 関係者への影響確認
- [ ] feature flag設計確認

### 実装中
- [ ] 日次で往復テスト実施
- [ ] パフォーマンス計測継続
- [ ] ドキュメント同時更新

### 実装後
- [ ] 全スモークテスト合格
- [ ] トークン削減率目標達成
- [ ] ユーザーガイド完成

---

この計画に従って実装を進めることで、4週間でANCPを完成させ、AIとの協働開発を革命的に改善します！