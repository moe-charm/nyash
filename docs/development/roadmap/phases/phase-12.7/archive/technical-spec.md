# ANCP Technical Specification v1.0

## 1. プロトコル概要

### 1.1 設計原則
- **可逆性**: 100%の双方向変換保証
- **効率性**: 50-70%のトークン削減
- **可読性**: 人間も慣れれば読み書き可能
- **拡張性**: バージョニングによる将来対応

### 1.2 プロトコルヘッダー
```
;ancp:1.0 nyash:0.5.0;
```
- `ancp:1.0`: ANCPプロトコルバージョン
- `nyash:0.5.0`: 対応Nyashバージョン

## 2. 記号マッピング仕様

### 2.1 予約語マッピング（優先度順）

#### Tier 1: 超高頻度（1文字）
| Nyash | ANCP | 頻度 | tiktoken削減 |
|-------|------|------|--------------|
| me | m | 極高 | 2→1 (50%) |
| new | n | 高 | 3→1 (67%) |
| return | r | 高 | 6→1 (83%) |
| local | l | 高 | 5→1 (80%) |

#### Tier 2: 高頻度（1文字特殊）
| Nyash | ANCP | 理由 |
|-------|------|------|
| box | $ | 金庫のメタファー |
| from | @ | 接続を表現 |
| init | # | 初期化のハッシュ |
| if | ? | 疑問・条件 |
| else | : | 条件の区切り |

#### Tier 3: 中頻度（1-2文字）
| Nyash | ANCP | 理由 |
|-------|------|------|
| static | S | 大文字で静的を表現 |
| loop | L | ループのL |
| birth | b | 誕生のb |
| override | O | 上書きのO |
| pack | p | パックのp |

### 2.2 演算子・記号の扱い
- 算術演算子（+, -, *, /）: そのまま
- 比較演算子（==, !=, <, >）: そのまま
- 論理演算子（and, or, not）: 検討中
  - `and` → `&`
  - `or` → `|`
  - `not` → `!`

### 2.3 複合パターン
```nyash
// 元のコード
box Cat from Animal {
    init { name, age }
}

// ANCP変換後
$Cat@Animal{#{name,age}}
```

## 3. パース規則

### 3.1 トークン境界
- 記号の前後に空白は不要（`$Cat`でOK）
- 識別子の区切りは既存ルール継承
- 文字列・コメント内は変換しない

### 3.2 優先順位
1. 文字列リテラル内: 変換なし
2. コメント内: 変換なし
3. 識別子の一部: 変換なし（`method_name`の`me`は変換しない）
4. 独立トークン: 変換対象

### 3.3 コンテキスト認識
```rust
enum TokenContext {
    Normal,           // 通常（変換対象）
    StringLiteral,    // 文字列内
    Comment,          // コメント内
    Identifier,       // 識別子の一部
}
```

## 4. 実装仕様

### 4.1 Transcoder API
```rust
pub trait AncpTranscoder {
    // 基本変換
    fn encode(&self, nyash: &str) -> Result<String, AncpError>;
    fn decode(&self, ancp: &str) -> Result<String, AncpError>;
    
    // ストリーミング変換
    fn encode_stream(&self, input: impl Read) -> impl Read;
    fn decode_stream(&self, input: impl Read) -> impl Read;
    
    // 位置情報保持
    fn encode_with_map(&self, nyash: &str) -> Result<(String, SourceMap), AncpError>;
    fn decode_with_map(&self, ancp: &str) -> Result<(String, SourceMap), AncpError>;
}
```

### 4.2 SourceMap仕様
```rust
pub struct SourceMap {
    mappings: Vec<Mapping>,
}

pub struct Mapping {
    // 元の位置
    original_line: u32,
    original_column: u32,
    original_token: String,
    
    // 変換後の位置
    generated_line: u32,
    generated_column: u32,
    generated_token: String,
}
```

### 4.3 エラーハンドリング
```rust
pub enum AncpError {
    // 構文エラー
    InvalidSyntax { position: Position, expected: String },
    
    // バージョン非互換
    VersionMismatch { required: Version, found: Version },
    
    // 変換不可能
    UnsupportedConstruct { construct: String, reason: String },
}
```

## 5. 統合仕様

### 5.1 Lexer統合
```rust
// Lexerに追加
pub enum InputDialect {
    Nyash,
    Ancp(Version),
}

impl Lexer {
    pub fn new_with_dialect(input: &str, dialect: InputDialect) -> Self {
        // ヘッダー検出で自動判定も可能
        let dialect = detect_dialect(input).unwrap_or(dialect);
        // ...
    }
}
```

### 5.2 CLI統合
```bash
# 変換コマンド
nyash --to-ancp input.nyash > output.ancp
nyash --from-ancp input.ancp > output.nyash

# 直接実行
nyash --dialect=ancp script.ancp

# フォーマット表示
nyash --view=ancp script.nyash    # Nyashファイルをancp形式で表示
nyash --view=hybrid script.ancp   # 並列表示
```

### 5.3 VSCode統合
```typescript
// 言語サーバープロトコル拡張
interface AncpHoverInfo {
    original: string;      // Nyash形式
    compressed: string;    // ANCP形式
    savings: number;       // 削減率
}

// リアルタイム変換
interface AncpLens {
    showOriginal: boolean;
    showCompressed: boolean;
    showSavings: boolean;
}
```

## 6. テスト仕様

### 6.1 往復テスト
```rust
#[test]
fn roundtrip_all_constructs() {
    let test_cases = vec![
        // 基本構造
        "box Test { }",
        "box Child from Parent { }",
        
        // メソッド定義
        "birth() { me.x = 1 }",
        "override method() { from Parent.method() }",
        
        // 制御構造
        "if x == 1 { } else { }",
        "loop(i < 10) { i = i + 1 }",
        
        // 複雑な例
        include_str!("../examples/complex.nyash"),
    ];
    
    for case in test_cases {
        let encoded = transcoder.encode(case).unwrap();
        let decoded = transcoder.decode(&encoded).unwrap();
        assert_eq!(case, decoded);
    }
}
```

### 6.2 トークン効率テスト
```rust
#[test]
fn measure_token_reduction() {
    let encoder = tiktoken::get_encoding("cl100k_base");
    
    let original = "box Cat from Animal { init { name } }";
    let ancp = "$Cat@Animal{#{name}}";
    
    let original_tokens = encoder.encode(original).len();
    let ancp_tokens = encoder.encode(ancp).len();
    
    let reduction = 1.0 - (ancp_tokens as f64 / original_tokens as f64);
    assert!(reduction >= 0.5); // 50%以上の削減を保証
}
```

### 6.3 エラー位置テスト
```rust
#[test]
fn error_position_mapping() {
    let ancp = "$Test{invalid syntax here}";
    let result = transcoder.decode_with_map(ancp);
    
    match result {
        Err(AncpError::InvalidSyntax { position, .. }) => {
            // エラー位置が正しくマッピングされているか
            assert_eq!(position.line, 1);
            assert_eq!(position.column, 14);
        }
        _ => panic!("Expected syntax error"),
    }
}
```

## 7. パフォーマンス目標

### 7.1 変換性能
- エンコード: 100MB/s以上
- デコード: 150MB/s以上
- メモリ使用: 入力サイズの2倍以内

### 7.2 実行時性能
- パース時間増加: 10%以内
- 実行時オーバーヘッド: なし（パース後は同一AST）

## 8. セキュリティ考慮事項

### 8.1 インジェクション対策
- ANCP記号が既存コードを破壊しないよう検証
- 文字列エスケープの適切な処理

### 8.2 バージョン互換性
- 古いANCPバージョンの適切なエラー表示
- 将来の拡張に備えた設計

## 9. 将来の拡張

### 9.1 ANCP v2.0候補
- 文脈依存圧縮（頻出パターンの動的割当）
- カスタム辞書サポート
- バイナリ形式（BANCTP）

### 9.2 AI特化拡張
- モデル別最適化プロファイル
- トークナイザー直接統合
- 意味保持圧縮

---

この仕様書は、ANCPの技術的実装の基準となる文書です。実装時はこの仕様に従い、必要に応じて更新してください。