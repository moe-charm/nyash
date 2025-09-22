# Nyash JSON Native Architecture

> 美しさ重視のモジュラー設計 - yyjsonの最適化と対極のアプローチ

## 🎨 設計哲学

### yyjson vs Nyash JSON
| 項目 | yyjson | Nyash JSON |
|------|--------|------------|
| **構造** | 単一巨大ファイル | モジュラー分離 |
| **最適化** | 最高速追求 | 理解しやすさ優先 |
| **保守性** | 専門家向け | チーム開発向け |
| **拡張性** | 困難 | 容易 |

### 80/20ルール適用
- **80%**: 美しく理解しやすい構造で動くものを作る
- **20%**: 後で必要に応じて最適化（ホットパス統合等）

## 📂 モジュラー構造

```
apps/lib/json_native/
├── README.md           # プロジェクト概要
├── ARCHITECTURE.md     # この設計ドキュメント
│
├── core/               # 🌟 核心データ構造
│   ├── node.nyash      # JsonNode - JSON値表現
│   ├── value.nyash     # JsonValue - 型安全ラッパー
│   └── error.nyash     # JsonError - エラーハンドリング
│
├── lexer/              # 🔍 字句解析層
│   ├── tokenizer.nyash # トークナイザー本体
│   ├── token.nyash     # トークン定義
│   └── scanner.nyash   # 文字スキャナー
│
├── parser/             # 🏗️ 構文解析層
│   ├── parser.nyash    # メインパーサー
│   ├── recursive.nyash # 再帰下降パーサー
│   └── validator.nyash # JSON妥当性検証
│
├── utils/              # 🛠️ ユーティリティ
│   ├── string.nyash    # 文字列処理ヘルパー
│   ├── escape.nyash    # エスケープ処理
│   └── pretty.nyash    # 整形出力
│
├── tests/              # 🧪 テストスイート
│   ├── unit/           # 単体テスト
│   ├── integration/    # 統合テスト
│   └── performance/    # 性能テスト
│
└── examples/           # 📖 使用例
    ├── basic.nyash     # 基本的な使用例
    ├── advanced.nyash  # 高度な使用例
    └── benchmark.nyash # ベンチマーク例
```

## 🎯 各モジュールの責務

### Core層 - データ構造の基盤
```nyash
// core/node.nyash - JSON値の抽象表現
box JsonNode {
    kind: StringBox     // "null"|"bool"|"int"|"string"|"array"|"object"
    value: Box          // 実際の値
    meta: Box           // メタデータ（位置情報等）
}

// core/value.nyash - 型安全なアクセス
box JsonValue {
    node: JsonNode      // 内部ノード
    // as_string(), as_int(), as_bool() 等の型安全メソッド
}

// core/error.nyash - エラー情報
box JsonError {
    code: StringBox     // エラーコード
    message: StringBox  // エラーメッセージ
    position: IntegerBox // エラー位置
}
```

### Lexer層 - 文字列をトークンに分解
```nyash
// lexer/token.nyash - トークン定義
box JsonToken {
    type: StringBox     // "STRING"|"NUMBER"|"LBRACE"|"RBRACE"等
    value: StringBox    // トークンの値
    start: IntegerBox   // 開始位置
    end: IntegerBox     // 終了位置
}

// lexer/tokenizer.nyash - メイントークナイザー
box JsonTokenizer {
    scanner: JsonScanner // 文字スキャナー
    tokens: ArrayBox     // 生成されたトークン配列
}
```

### Parser層 - トークンをASTに変換
```nyash
// parser/parser.nyash - メインパーサー
box JsonParser {
    tokenizer: JsonTokenizer  // 字句解析器
    current: IntegerBox       // 現在のトークン位置
    // parse() -> JsonNode
}

// parser/recursive.nyash - 再帰下降実装
static box RecursiveParser {
    parse_value(tokens, pos)    // 値をパース
    parse_object(tokens, pos)   // オブジェクトをパース
    parse_array(tokens, pos)    // 配列をパース
}
```

### Utils層 - 共通ユーティリティ
```nyash
// utils/string.nyash - 文字列処理
static box StringUtils {
    trim(s)                    // 空白トリム
    is_whitespace(ch)          // 空白文字判定
    is_digit(ch)               // 数字判定
}

// utils/escape.nyash - エスケープ処理
static box EscapeUtils {
    escape_string(s)           // JSON文字列エスケープ
    unescape_string(s)         // JSONエスケープ解除
    validate_string(s)         // 文字列妥当性検証
}
```

## 🔄 モジュール間の依存関係

```
Core ←── Lexer ←── Parser ←── API
 ↑        ↑         ↑
Utils ────┴─────────┴─ (共通ユーティリティ)
```

### 依存性の最小化
- **Core**: 他モジュールに依存しない独立基盤
- **Lexer**: CoreとUtilsのみに依存
- **Parser**: Core, Lexer, Utilsに依存
- **Utils**: 完全独立（どこからでも使用可能）

## 🚀 段階的実装戦略

### Phase 1: Core基盤 (完了)
- [x] JsonNode基本実装
- [x] 基本的なJSON生成機能

### Phase 2: Utils基盤
- [ ] StringUtils実装
- [ ] EscapeUtils実装
- [ ] PrettyPrint実装

### Phase 3: Lexer実装
- [ ] JsonToken定義
- [ ] JsonScanner実装
- [ ] JsonTokenizer実装

### Phase 4: Parser実装
- [ ] RecursiveParser実装
- [ ] JsonParser統合
- [ ] エラーハンドリング

### Phase 5: 統合＆テスト
- [ ] 全モジュール統合
- [ ] 包括的テストスイート
- [ ] 性能測定＆調整

## 💡 美しさの利点

### 1. **理解しやすさ**
- モジュール境界が明確
- 各ファイルが単一責任
- 新規開発者でもすぐ理解

### 2. **保守性**
- バグ修正が局所化
- 機能追加が容易
- リファクタリングが安全

### 3. **テスト性**
- 各モジュール独立テスト可能
- モックによる分離テスト
- デバッグが簡単

### 4. **拡張性**
- 新機能を独立モジュールで追加
- 既存コードを破壊せずに拡張
- プラグイン機構の基盤

## 🎯 美しいコードの指針

### ファイルサイズ制限
- 各ファイル: **200行以下**を目標
- 複雑な機能は複数ファイルに分割
- 読みやすさを最優先

### 命名規則
- **Box名**: PascalCase (JsonNode, JsonParser)
- **メソッド名**: snake_case (parse_value, as_string)
- **ファイル名**: snake_case (tokenizer.nyash, recursive.nyash)

### コメント戦略
- **なぜ**: 設計の意図を説明
- **何**: 複雑なアルゴリズムを説明
- **注意**: エッジケースや制限事項

この美しいアーキテクチャで、yyjsonとは対極のアプローチを実践し、
保守しやすく理解しやすいJSON実装を作り上げます！