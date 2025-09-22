# Nyash JSON Native

> yyjson（C依存）→ 完全Nyash実装で外部依存完全排除

## 🎯 プロジェクト目標

- **C依存ゼロ**: yyjsonからの完全脱却
- **Everything is Box**: 全てをNyash Boxで実装
- **80/20ルール**: 動作優先、最適化は後
- **段階切り替え**: `NYASH_JSON_PROVIDER=nyash|yyjson|serde`

## 📦 アーキテクチャ設計

### 🔍 yyjson分析結果
```c
// yyjson核心設計パターン
struct yyjson_val {
    uint64_t tag;        // 型+サブタイプ+長さ
    yyjson_val_uni uni;  // ペイロード（union）
};
```

### 🎨 Nyash版設計 (Everything is Box)
```nyash
// 🌟 JSON値を表現するBox
box JsonNode {
    kind: StringBox      // "null"|"bool"|"int"|"string"|"array"|"object"
    value: Box           // 実際の値（種類に応じて）
    children: ArrayBox   // 配列・オブジェクト用（オプション）
    keys: ArrayBox       // オブジェクトのキー配列（オプション）
}

// 🔧 JSON字句解析器
box JsonLexer {
    text: StringBox      // 入力文字列
    pos: IntegerBox      // 現在位置
    tokens: ArrayBox     // トークン配列
}

// 🏗️ JSON構文解析器
box JsonParser {
    lexer: JsonLexer     // 字句解析器
    current: IntegerBox  // 現在のトークン位置
}
```

## 📂 ファイル構造

```
apps/lib/json_native/
├── README.md          # この設計ドキュメント
├── lexer.nyash        # JSON字句解析器（状態機械ベース）
├── parser.nyash       # JSON構文解析器（再帰下降）
├── node.nyash         # JsonNode実装（Everything is Box）
├── tests/             # テストケース
│   ├── lexer_test.nyash
│   ├── parser_test.nyash
│   └── integration_test.nyash
└── examples/          # 使用例
    ├── simple_parse.nyash
    └── complex_object.nyash
```

## 🎯 実装戦略

### Phase 1: JsonNode基盤（1週目）
- [x] JsonNode Box実装
- [x] 基本的な値型サポート（null, bool, int, string）
- [x] 配列・オブジェクト構造サポート

### Phase 2: JsonLexer実装（2週目）
- [ ] トークナイザー実装（状態機械ベース）
- [ ] エラーハンドリング
- [ ] 位置情報追跡

### Phase 3: JsonParser実装（3週目）
- [ ] 再帰下降パーサー実装
- [ ] JsonNode構築
- [ ] ネストされた構造サポート

### Phase 4: 統合＆最適化（4週目）
- [ ] C ABI Bridge（最小実装）
- [ ] 既存JSONBoxとの互換性
- [ ] 性能測定・最適化

## 🔄 既存システムとの統合

### 段階的移行戦略
```bash
# 環境変数で切り替え
export NYASH_JSON_PROVIDER=nyash    # 新実装
export NYASH_JSON_PROVIDER=yyjson   # 既存C実装
export NYASH_JSON_PROVIDER=serde    # Rust実装
```

### 既存APIとの互換性
```nyash
// 既存JSONBox APIを維持
local json = new JSONBox()
json.parse("{\"key\": \"value\"}")  // 内部でNyash実装を使用
```

## 🎨 設計思想

### Everything is Box原則
- **JsonNode**: 完全なBox実装
- **境界明確化**: 各コンポーネントをBox化
- **差し替え可能**: プロバイダー切り替え対応

### 80/20ルール適用
- **80%**: まず動く実装（文字列操作ベース）
- **20%**: 後で最適化（バイナリ処理、SIMD等）

### フォールバック戦略
- **エラー時**: 既存実装に安全復帰
- **性能不足時**: yyjsonに切り替え可能
- **互換性**: 既存APIを100%維持

## 🧪 テスト戦略

### 基本テストケース
```json
{"null": null}
{"bool": true}
{"int": 42}
{"string": "hello"}
{"array": [1,2,3]}
{"object": {"nested": "value"}}
```

### エラーケース
```
{invalid}           // 不正なJSON
{"unclosed": "str   // 閉じられていない文字列
[1,2,              // 不完全な配列
```

### 性能テストケース
```
大きなJSONファイル（10MB+）
深くネストされた構造（100レベル+）
多数の小さなオブジェクト（10万個+）
```

## 🚀 実行例

```nyash
// 基本的な使用例
local JsonNative = include "apps/lib/json_native/node.nyash"

// JSON文字列をパース
local text = "{\"name\": \"Nyash\", \"version\": 1}"
local node = JsonNative.parse(text)

// 値にアクセス
print(node.get("name").str())     // "Nyash"
print(node.get("version").int())  // 1

// JSON文字列に戻す
print(node.stringify())           // {"name":"Nyash","version":1}
```

## 📊 進捗追跡

- [ ] Week 1: JsonNode基盤
- [ ] Week 2: JsonLexer実装
- [ ] Week 3: JsonParser実装
- [ ] Week 4: 統合＆最適化

**開始日**: 2025-09-22  
**目標完了**: 2025-10-20  
**実装者**: Claude × User協働