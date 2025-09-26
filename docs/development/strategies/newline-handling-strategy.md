# Nyash改行処理戦略
*作成日: 2025-09-23*
*ステータス: 承認済み・実装開始*

## 概要
Nyashパーサーにおける改行処理の統一戦略。現在の`skip_newlines()`散在問題を解決し、コンテキスト認識による自動改行処理を実現する。

## 現状の問題

### 1. skip_newlines()の散在
```rust
// 現状：各所で手動呼び出しが必要
fn parse_object_literal() {
    self.skip_newlines();  // ここにも
    self.consume(TokenType::COLON)?;
    self.skip_newlines();  // あそこにも
    let value = self.parse_expression()?;
    self.skip_newlines();  // どこにでも
}
```

### 2. 複数行構文のパースエラー
```nyash
// これがパースできない
local result = match "test" {
    "test" => {
        value: 42,  // ← 改行でエラー
        name: "answer"
    }
}
```

### 3. 保守性の問題
- 新しい構文追加時に`skip_newlines()`の配置を忘れやすい
- 一貫性のない改行処理
- デバッグが困難

## 設計原則

### 1. セミコロンオプショナル
- 改行もセミコロンも文の区切りとして扱う
- ユーザーは好みのスタイルを選択可能

### 2. コンテキスト認識
- `{}`, `[]`, `()`内では改行を自動的に無視
- 式コンテキストでは改行をスキップ
- 文コンテキストでは改行を文区切りとして扱う

### 3. 環境変数地獄の回避
- コンテキストベースの制御を優先
- 環境変数は互換性のためのみに限定

## 実装戦略（3段階）

### Phase 0: Quick Fix（即座実装）
**目的**: 緊急対応として最小限の修正で問題を解決

```rust
// src/parser/expr/primary.rs L77付近
self.skip_newlines(); // COLON前に追加
self.consume(TokenType::COLON)?;
self.skip_newlines(); // 値パース前に追加
let value_expr = self.parse_expression()?;
self.skip_newlines(); // COMMA判定前に追加
```

**期間**: 30分
**効果**: match式の複数行オブジェクトリテラルが動作

### Phase 1: TokenCursor導入（今週実装）
**目的**: 改行処理を一元管理し、`skip_newlines()`散在を根絶

```rust
// src/parser/cursor.rs（新規）
pub struct TokenCursor<'a> {
    tokens: &'a [Token],
    idx: usize,
    mode: NewlineMode,
    paren_depth: usize,
    brace_depth: usize,
    bracket_depth: usize,
}

pub enum NewlineMode {
    Stmt,  // 文モード：改行は文区切り
    Expr,  // 式モード：改行を自動スキップ
}

impl<'a> TokenCursor<'a> {
    pub fn with_expr_mode<F, T>(&mut self, f: F) -> T
    where F: FnOnce(&mut Self) -> T {
        let old = self.mode;
        self.mode = NewlineMode::Expr;
        let result = f(self);
        self.mode = old;
        result
    }

    fn should_skip_newline(&self) -> bool {
        // ブレース内 or 式モード or 行継続
        self.brace_depth > 0 ||
        self.mode == NewlineMode::Expr ||
        self.prev_is_line_continuation()
    }
}
```

**使用例**:
```rust
fn parse_expression(cursor: &mut TokenCursor) -> Result<Expr> {
    cursor.with_expr_mode(|c| {
        // この中では改行が自動的にスキップされる
        parse_binary_expr(c, 0)
    })
}
```

**期間**: 1週間
**効果**:
- 改行処理の一元化
- 新規構文追加時のミス防止
- デバッグの容易化

### Phase 2: LASI前処理（将来実装）
**目的**: トークンレベルで改行を正規化し、パーサーを単純化

```rust
// トークン正規化層
fn normalize_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut iter = tokens.into_iter();

    while let Some(token) = iter.next() {
        match token.token_type {
            TokenType::NEWLINE => {
                if should_insert_eol(&prev, &next) {
                    result.push(Token::EOL);
                }
                // それ以外のNEWLINEは削除
            }
            _ => result.push(token),
        }
    }
    result
}
```

**期間**: Phase 15完了後
**効果**:
- パーサーの大幅な簡略化
- 完全な改行処理の分離
- LosslessToken/Triviaとの統合

## 行継続ルール

### 継続と判定する記号（直前）
- 二項演算子: `+`, `-`, `*`, `/`, `%`, `&&`, `||`, `|`, `&`, `^`
- メンバアクセス: `.`, `::`
- Optional系: `?`, `?.`, `??`
- Arrow: `=>`, `->`
- カンマ: `,`

### 文終端強制（ハザード回避）
- `return`, `break`, `continue`の直後の改行は常に文終端
- JavaScriptのASIハザードと同様の考え方

## テストケース

### 基本ケース
```nyash
// 2文として解釈
a = 1
b = 2

// 1文として解釈（行継続）
a = 1 +
    2

// セミコロンも可
a = 1; b = 2
```

### 括弧内改行
```nyash
// すべてOK
f(
    arg1,
    arg2
)

[
    1,
    2,
    3
]

{
    key1: value1,
    key2: value2
}
```

### match式
```nyash
local result = match value {
    "test" => {
        name: "foo",
        value: 42
    },
    _ => {
        name: "default",
        value: 0
    }
}
```

### returnハザード
```nyash
return   // returnのみ
    42   // 別の文として解釈

return 42  // return 42として解釈
```

## 影響分析

### 後方互換性
- Phase 0: 完全互換
- Phase 1: 完全互換（内部実装の変更のみ）
- Phase 2: セミコロン使用時のみ互換性確認必要

### パフォーマンス
- Phase 0: 影響なし
- Phase 1: わずかなオーバーヘッド（カーソル管理）
- Phase 2: 前処理により若干の初期化コスト

## 参考：他言語の実装

### Go
- トークナイザレベルでセミコロン自動挿入
- 特定トークン後に改行があれば`;`を挿入

### JavaScript
- ASI（Automatic Semicolon Insertion）
- returnハザード等の特殊ルールあり

### Python
- インデントベース（Nyashとは異なるアプローチ）

### Rust
- セミコロン必須（式vs文の区別）

## 決定事項

1. **TokenCursorアプローチを採用**
   - ChatGPT提案のサンプルコードをベースに実装
   - 環境変数ではなくコンテキストベースの制御

2. **3段階実装で段階的改善**
   - まずQuick Fixで緊急対応
   - TokenCursorで本質的解決
   - 将来的にLASI前処理で完成形へ

3. **セミコロン完全オプショナル**
   - 改行とセミコロンを同等に扱う
   - ユーザーの好みに応じて選択可能

## 実装タイムライン

- **2025-09-23**: Quick Fix実装（30分）
- **2025-09-30**: TokenCursor Phase 1完了（1週間）
- **Phase 15後**: LASI前処理検討開始

## 関連ドキュメント

- [Parser Architecture](../../reference/parser/)
- [Language Syntax](../../reference/language/)
- [Phase 15 Roadmap](../../roadmap/phases/phase-15/)

## 承認

- ChatGPT Pro: 技術分析・実装提案
- Claude: 実装戦略決定・ドキュメント化
- 実装開始: 2025-09-23